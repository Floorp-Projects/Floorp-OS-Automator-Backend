// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later
// Sapphillon
//
//
//
//

use std::pin::Pin;
use std::sync::Arc;

use chrono::Utc;
use database::workflow::{get_workflow_by_id, update_workflow_from_proto};
use entity::entity::workflow as workflow_entity;
use log::{debug, error, info, warn};
use sapphillon_core::permission::{Permissions, PluginFunctionPermissions};
use sapphillon_core::proto::google::protobuf::Timestamp;
use sapphillon_core::proto::google::rpc::{Code as RpcCode, Status as RpcStatus};
use sapphillon_core::proto::sapphillon::v1::workflow_service_server::WorkflowService;
use sapphillon_core::proto::sapphillon::v1::{
    AllowedPermission, DeleteWorkflowRequest, DeleteWorkflowResponse, FixWorkflowRequest,
    FixWorkflowResponse, GenerateWorkflowRequest, GenerateWorkflowResponse, GetWorkflowRequest,
    GetWorkflowResponse, ListWorkflowsRequest, ListWorkflowsResponse, RunWorkflowRequest,
    RunWorkflowResponse, UpdateWorkflowRequest, UpdateWorkflowResponse, Workflow, WorkflowCode,
    WorkflowResult,
};
use sapphillon_core::workflow::CoreWorkflowCode;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder, QuerySelect};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::workflow::generate_workflow_async;

/// Maximum number of characters to keep when deriving workflow display names from prompts.
const MAX_DISPLAY_NAME_LEN: usize = 64;
const DEFAULT_PAGE_SIZE: u64 = 100;
const WORKFLOW_LANGUAGE_JS: i32 = 2;
const WORKFLOW_LANGUAGE_UNSPECIFIED: i32 = 0;

#[derive(Clone, Debug)]
pub struct MyWorkflowService {
    db: Arc<DatabaseConnection>,
}

impl MyWorkflowService {
    /// Creates a new workflow service backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    fn ok_status(message: impl Into<String>) -> Option<RpcStatus> {
        Some(RpcStatus {
            code: RpcCode::Ok as i32,
            message: message.into(),
            details: vec![],
        })
    }

    fn map_db_error(err: DbErr) -> Status {
        error!("database operation failed: {err:?}");
        Status::internal("database operation failed")
    }

    fn map_not_found(err: DbErr, resource: impl Into<String>) -> Status {
        match err {
            DbErr::RecordNotFound(_) => Status::not_found(resource.into()),
            DbErr::Custom(msg) if msg.contains("not found") => Status::not_found(resource.into()),
            other => Self::map_db_error(other),
        }
    }

    fn now_timestamp() -> Timestamp {
        let now = Utc::now();
        Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        }
    }

    fn derive_display_name(prompt: &str) -> String {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return "Generated Workflow".to_string();
        }
        let mut name = trimmed
            .lines()
            .next()
            .unwrap_or("Generated Workflow")
            .trim()
            .to_string();
        if name.len() > MAX_DISPLAY_NAME_LEN {
            let mut index = MAX_DISPLAY_NAME_LEN;
            while !name.is_char_boundary(index) {
                index -= 1;
            }
            name.truncate(index);
        }
        name
    }

    fn make_plugin_permission(permission: &AllowedPermission) -> PluginFunctionPermissions {
        PluginFunctionPermissions {
            plugin_function_id: permission.plugin_function_id.clone(),
            permissions: Permissions::new(permission.permissions.clone()),
        }
    }

    fn apply_update_mask(
        existing: &Workflow,
        incoming: &Workflow,
        mask_paths: &[String],
    ) -> Result<Workflow, Box<Status>> {
        if mask_paths.is_empty() {
            return Ok(Self::merge_workflow(existing, incoming, true));
        }

        let mut desired = existing.clone();
        for path in mask_paths {
            match path.as_str() {
                "display_name" => desired.display_name = incoming.display_name.clone(),
                "description" => desired.description = incoming.description.clone(),
                "workflow_language" => desired.workflow_language = incoming.workflow_language,
                "workflow_code" => desired.workflow_code = incoming.workflow_code.clone(),
                "workflow_results" => desired.workflow_results = incoming.workflow_results.clone(),
                "updated_at" => desired.updated_at = incoming.updated_at,
                "created_at" => desired.created_at = incoming.created_at.or(existing.created_at),
                other => {
                    return Err(Box::new(Status::invalid_argument(format!(
                        "unsupported update_mask path: {other}"
                    ))));
                }
            }
        }

        Ok(desired)
    }

    fn merge_workflow(existing: &Workflow, incoming: &Workflow, overwrite_all: bool) -> Workflow {
        let mut desired = existing.clone();

        if overwrite_all || !incoming.display_name.is_empty() {
            desired.display_name = incoming.display_name.clone();
        }

        if overwrite_all || !incoming.description.is_empty() {
            desired.description = incoming.description.clone();
        }

        if overwrite_all || incoming.workflow_language != 0 {
            desired.workflow_language = incoming.workflow_language;
        }

        if overwrite_all || !incoming.workflow_code.is_empty() {
            desired.workflow_code = incoming.workflow_code.clone();
        }

        if overwrite_all || !incoming.workflow_results.is_empty() {
            desired.workflow_results = incoming.workflow_results.clone();
        }

        if overwrite_all {
            if let Some(created_at) = incoming.created_at {
                desired.created_at = Some(created_at);
            }
            desired.updated_at = incoming.updated_at;
        }

        desired
    }

    fn sanitize_generated_code(code: &str) -> String {
        let trimmed = code.trim();
        if trimmed.ends_with("workflow();") {
            trimmed.to_string()
        } else {
            format!("{trimmed}\nworkflow();")
        }
    }

    fn decode_page_token(token: &str) -> u64 {
        token.trim().parse::<u64>().unwrap_or(0)
    }

    fn encode_page_token(offset: u64) -> String {
        offset.to_string()
    }

    async fn persist_workflow_results(
        &self,
        workflow: &mut Workflow,
        workflow_code_id: &str,
        new_results: &[WorkflowResult],
    ) -> Result<(), Status> {
        if let Some(code) = workflow
            .workflow_code
            .iter_mut()
            .find(|c| c.id == workflow_code_id)
        {
            let mut combined = code.result.clone();
            combined.extend_from_slice(new_results);
            combined.sort_by_key(|r| r.workflow_result_revision);
            combined.dedup_by(|a, b| a.id == b.id);
            code.result = combined;
        }

        if !new_results.is_empty() {
            let mut combined = workflow.workflow_results.clone();
            combined.extend_from_slice(new_results);
            combined.sort_by_key(|r| r.workflow_result_revision);
            combined.dedup_by(|a, b| a.id == b.id);
            workflow.workflow_results = combined;
        }

        workflow.updated_at = Some(Self::now_timestamp());

        update_workflow_from_proto(&self.db, workflow)
            .await
            .map_err(Self::map_db_error)?;
        Ok(())
    }

    fn build_core_permissions(
        workflow_code: &WorkflowCode,
    ) -> (
        Vec<PluginFunctionPermissions>,
        Vec<PluginFunctionPermissions>,
    ) {
        let allowed_permissions = if workflow_code.allowed_permissions.is_empty() {
            if let Some(first_id) = workflow_code.plugin_function_ids.first() {
                vec![PluginFunctionPermissions {
                    plugin_function_id: first_id.clone(),
                    permissions: Permissions::new(vec![]),
                }]
            } else {
                Vec::new()
            }
        } else {
            workflow_code
                .allowed_permissions
                .iter()
                .map(Self::make_plugin_permission)
                .collect()
        };

        let required_permissions = allowed_permissions.clone();
        (required_permissions, allowed_permissions)
    }
}

#[tonic::async_trait]
impl WorkflowService for MyWorkflowService {
    type FixWorkflowStream =
        Pin<Box<dyn Stream<Item = Result<FixWorkflowResponse, Status>> + Send + 'static>>;
    type GenerateWorkflowStream =
        Pin<Box<dyn Stream<Item = Result<GenerateWorkflowResponse, Status>> + Send + 'static>>;

    async fn update_workflow(
        &self,
        request: Request<UpdateWorkflowRequest>,
    ) -> Result<Response<UpdateWorkflowResponse>, Status> {
        let req = request.into_inner();
        let incoming = req
            .workflow
            .ok_or_else(|| Status::invalid_argument("workflow is required"))?;

        if incoming.id.trim().is_empty() {
            return Err(Status::invalid_argument("workflow.id must not be empty"));
        }
        if incoming.display_name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "workflow.display_name must not be empty",
            ));
        }

        let has_update_mask = req
            .update_mask
            .as_ref()
            .map(|m| !m.paths.is_empty())
            .unwrap_or(false);
        info!(
            "update_workflow request received: workflow_id={workflow_id}, has_update_mask={has_update_mask}",
            workflow_id = incoming.id.as_str(),
            has_update_mask = has_update_mask
        );

        let existing = get_workflow_by_id(&self.db, &incoming.id)
            .await
            .map_err(|err| Self::map_not_found(err, format!("workflow '{}'", incoming.id)))?;

        let mask_paths = req.update_mask.map(|mask| mask.paths).unwrap_or_default();
        let mut desired =
            Self::apply_update_mask(&existing, &incoming, &mask_paths).map_err(|e| *e)?;
        desired.updated_at = Some(Self::now_timestamp());
        if desired.created_at.is_none() {
            desired.created_at = existing.created_at;
        }

        let updated = update_workflow_from_proto(&self.db, &desired)
            .await
            .map_err(Self::map_db_error)?;

        let response = UpdateWorkflowResponse {
            workflow: Some(updated),
            status: Self::ok_status("workflow updated"),
        };

        info!(
            "workflow updated successfully: workflow_id={workflow_id}",
            workflow_id = incoming.id.as_str()
        );

        Ok(Response::new(response))
    }

    async fn delete_workflow(
        &self,
        request: Request<DeleteWorkflowRequest>,
    ) -> Result<Response<DeleteWorkflowResponse>, Status> {
        let req = request.into_inner();
        if req.workflow_id.trim().is_empty() {
            return Err(Status::invalid_argument("workflow_id must not be empty"));
        }

        info!(
            "delete_workflow request received: workflow_id={workflow_id}",
            workflow_id = req.workflow_id.as_str()
        );

        get_workflow_by_id(&self.db, &req.workflow_id)
            .await
            .map_err(|err| Self::map_not_found(err, format!("workflow '{}'", req.workflow_id)))?;

        workflow_entity::Entity::delete_by_id(req.workflow_id.clone())
            .exec(&*self.db)
            .await
            .map_err(Self::map_db_error)?;

        info!(
            "workflow deleted: workflow_id={workflow_id}",
            workflow_id = req.workflow_id.as_str()
        );

        Ok(Response::new(DeleteWorkflowResponse {}))
    }

    async fn list_workflows(
        &self,
        request: Request<ListWorkflowsRequest>,
    ) -> Result<Response<ListWorkflowsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "list_workflows request received: page_size={page_size}, page_token='{page_token}', has_filter={has_filter}",
            page_size = req.page_size,
            page_token = req.page_token.as_str(),
            has_filter = req.filter.is_some()
        );
        let page_size = if req.page_size <= 0 {
            None
        } else {
            Some(req.page_size as u32)
        };
        let (filter_name, filter_language) = req
            .filter
            .map(|f| {
                let name = if f.display_name.trim().is_empty() {
                    None
                } else {
                    Some(f.display_name)
                };
                let lang = if f.workflow_language == WORKFLOW_LANGUAGE_UNSPECIFIED {
                    None
                } else {
                    Some(f.workflow_language)
                };
                (name, lang)
            })
            .unwrap_or((None, None));

        let offset = Self::decode_page_token(&req.page_token);
        let limit = page_size
            .map(|v| v as u64)
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .max(1);

        let mut items = workflow_entity::Entity::find()
            .order_by_asc(workflow_entity::Column::Id)
            .offset(offset)
            .limit(limit.saturating_add(1))
            .all(&*self.db)
            .await
            .map_err(Self::map_db_error)?;

        let has_next = (items.len() as u64) > limit;
        if has_next {
            items.truncate(limit as usize);
        }

        let next_page_token = if has_next {
            Self::encode_page_token(offset.saturating_add(limit))
        } else {
            String::new()
        };

        let mut workflows = Vec::with_capacity(items.len());
        for item in items {
            let workflow = get_workflow_by_id(&self.db, &item.id)
                .await
                .map_err(|err| Self::map_not_found(err, format!("workflow '{}'", item.id)))?;

            if matches!(filter_name.as_deref(), Some(name) if !workflow.display_name.contains(name))
            {
                continue;
            }
            if matches!(filter_language, Some(lang) if workflow.workflow_language != lang) {
                continue;
            }

            workflows.push(workflow);
        }

        let response = ListWorkflowsResponse {
            workflows,
            next_page_token,
            status: Self::ok_status("workflows listed"),
        };

        debug!(
            "list_workflows response ready: workflow_count={workflow_count}",
            workflow_count = response.workflows.len()
        );

        Ok(Response::new(response))
    }

    async fn fix_workflow(
        &self,
        request: Request<FixWorkflowRequest>,
    ) -> Result<Response<Self::FixWorkflowStream>, Status> {
        let req = request.into_inner();
        let definition = req.workflow_definition.trim().to_string();
        if definition.is_empty() {
            return Err(Status::invalid_argument(
                "workflow_definition must not be empty",
            ));
        }
        let description = req.description.trim().to_string();
        if description.is_empty() {
            return Err(Status::invalid_argument("description must not be empty"));
        }

        info!(
            "fix_workflow request received: definition_len={definition_len}, description_len={description_len}",
            definition_len = definition.len(),
            description_len = description.len()
        );

        let prompt = format!(
            "Fix the following workflow definition based on the issues described.\\n\\nDefinition:```\\n{definition}\\n```\\n\\nIssues: {description}.\\n\\nProduce an updated workflow.js implementation.",
        );

        if let Err(err) = generate_workflow_async(&prompt).await {
            warn!("failed to fix workflow via generator (ignoring for demo): {err}");
        }

        // OVERRIDE: use local file for demo
        let generated = std::fs::read_to_string("demo_workflows/workflow.js")
            .map_err(|e| Status::internal(format!("Failed to read workflow.js: {}", e)))?;

        let workflow_id = uuid::Uuid::new_v4().to_string();
        let workflow_code_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Self::now_timestamp();
        let workflow = Workflow {
            id: workflow_id.clone(),
            display_name: "Fixed Workflow".to_string(),
            description,
            workflow_language: WORKFLOW_LANGUAGE_JS,
            workflow_code: vec![WorkflowCode {
                id: workflow_code_id,
                code_revision: 1,
                code: Self::sanitize_generated_code(&generated),
                language: WORKFLOW_LANGUAGE_JS,
                created_at: Some(timestamp),
                result: vec![],
                plugin_packages: vec![],
                plugin_function_ids: vec![],
                allowed_permissions: vec![],
            }],
            created_at: Some(timestamp),
            updated_at: Some(timestamp),
            workflow_results: vec![],
        };

        let stored = update_workflow_from_proto(&self.db, &workflow)
            .await
            .map_err(Self::map_db_error)?;

        let response = FixWorkflowResponse {
            fixed_workflow_definition: Some(stored),
            change_summary: "Generated updated workflow definition".to_string(),
            status: Self::ok_status("workflow fixed"),
        };

        info!("workflow fix generated: workflow_id={workflow_id}");

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx.send(Ok(response)).await;
        });

        Ok(Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::FixWorkflowStream
        ))
    }

    async fn get_workflow(
        &self,
        request: Request<GetWorkflowRequest>,
    ) -> Result<Response<GetWorkflowResponse>, Status> {
        let req = request.into_inner();
        if req.workflow_id.trim().is_empty() {
            return Err(Status::invalid_argument("workflow_id must not be empty"));
        }

        debug!(
            "get_workflow request received: workflow_id={workflow_id}",
            workflow_id = req.workflow_id.as_str()
        );

        let workflow = get_workflow_by_id(&self.db, &req.workflow_id)
            .await
            .map_err(|err| Self::map_not_found(err, format!("workflow '{}'", req.workflow_id)))?;

        let response = GetWorkflowResponse {
            workflow: Some(workflow),
            status: Self::ok_status("workflow retrieved"),
        };

        debug!(
            "workflow retrieved: workflow_id={workflow_id}",
            workflow_id = req.workflow_id.as_str()
        );

        Ok(Response::new(response))
    }

    async fn generate_workflow(
        &self,
        request: Request<GenerateWorkflowRequest>,
    ) -> Result<Response<Self::GenerateWorkflowStream>, Status> {
        let req = request.into_inner();
        if req.prompt.trim().is_empty() {
            return Err(Status::invalid_argument("prompt must not be empty"));
        }

        info!(
            "generate_workflow request received: prompt_len={prompt_len}",
            prompt_len = req.prompt.len()
        );

        if let Err(err) = generate_workflow_async(&req.prompt).await {
            warn!("failed to generate workflow via generator (ignoring for demo): {err}");
        }

        // OVERRIDE: use local file for demo
        let generated = std::fs::read_to_string("demo_workflows/workflow.js")
            .map_err(|e| Status::internal(format!("Failed to read workflow.js: {}", e)))?;

        let workflow_id = uuid::Uuid::new_v4().to_string();
        let workflow_code_id = uuid::Uuid::new_v4().to_string();
        let now_ts = Self::now_timestamp();

        // TODO: generate display_name from prompt by ai
        let workflow = Workflow {
            id: workflow_id,
            display_name: Self::derive_display_name(&req.prompt),
            description: req.prompt.clone(),
            workflow_language: WORKFLOW_LANGUAGE_JS,
            workflow_code: vec![WorkflowCode {
                id: workflow_code_id,
                code_revision: 1,
                code: Self::sanitize_generated_code(&generated),
                language: WORKFLOW_LANGUAGE_JS,
                created_at: Some(now_ts),
                result: vec![],
                plugin_packages: vec![],
                plugin_function_ids: vec![],
                allowed_permissions: vec![],
            }],
            created_at: Some(now_ts),
            updated_at: Some(now_ts),
            workflow_results: vec![],
        };

        let stored = update_workflow_from_proto(&self.db, &workflow)
            .await
            .map_err(Self::map_db_error)?;

        let response = GenerateWorkflowResponse {
            workflow_definition: Some(stored),
            status: Self::ok_status("workflow generated"),
        };

        let generated_workflow_id = response
            .workflow_definition
            .as_ref()
            .map(|wf| wf.id.as_str())
            .unwrap_or("unknown");
        info!("workflow generated: workflow_id={generated_workflow_id}");

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx.send(Ok(response)).await;
        });

        Ok(Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::GenerateWorkflowStream
        ))
    }

    async fn run_workflow(
        &self,
        request: Request<RunWorkflowRequest>,
    ) -> Result<Response<RunWorkflowResponse>, Status> {
        let req = request.into_inner();
        let persist_results = true;
        let target_code_id: Option<String>;

        let source_label = match &req.by_id {
            Some(_) => "by_id",
            None => "missing",
        };
        info!("run_workflow request received: source={source_label}");

        let mut workflow = match req.by_id {
            Some(by_id) => {
                if by_id.workflow_id.trim().is_empty() {
                    return Err(Status::invalid_argument("workflow_id must not be empty"));
                }
                if by_id.workflow_code_id.trim().is_empty() {
                    return Err(Status::invalid_argument(
                        "workflow_code_id must not be empty",
                    ));
                }
                target_code_id = Some(by_id.workflow_code_id.clone());
                get_workflow_by_id(&self.db, &by_id.workflow_id)
                    .await
                    .map_err(|err| {
                        Self::map_not_found(err, format!("workflow '{}'", by_id.workflow_id))
                    })?
            }
            None => {
                return Err(Status::invalid_argument(
                    "RunWorkflowRequest.by_id is required",
                ));
            }
        };

        let latest_revision = workflow
            .workflow_code
            .iter()
            .map(|code| code.code_revision)
            .max()
            .unwrap_or(0);

        let workflow_code = if let Some(ref code_id) = target_code_id {
            workflow
                .workflow_code
                .iter_mut()
                .find(|code| code.id == *code_id)
                .ok_or_else(|| Status::not_found(format!("workflow code '{code_id}' not found")))?
        } else {
            workflow
                .workflow_code
                .iter_mut()
                .find(|code| code.code_revision == latest_revision)
                .ok_or_else(|| Status::not_found("Latest workflow code not found"))?
        };

        workflow_code.code = match unescaper::unescape(&workflow_code.code) {
            Ok(code) => code,
            Err(err) => {
                warn!("failed to unescape workflow code: {err}");
                workflow_code.code.clone()
            }
        };

        let workflow_code_id = workflow_code.id.clone();

        let (required_permissions, allowed_permissions) =
            Self::build_core_permissions(workflow_code);

        let results = {
            let mut workflow_core = CoreWorkflowCode::new_from_proto(
                workflow_code,
                crate::sysconfig::sysconfig().core_plugin_package,
                required_permissions,
                allowed_permissions,
            );

            workflow_core.run(Handle::current());

            if workflow_core.result.is_empty() {
                return Err(Status::internal("workflow execution produced no result"));
            }

            workflow_core.result.clone()
        };

        let latest_result_revision = results
            .iter()
            .map(|r| r.workflow_result_revision)
            .max()
            .unwrap_or(0);

        let latest_result = results
            .iter()
            .find(|r| r.workflow_result_revision == latest_result_revision)
            .cloned()
            .ok_or_else(|| Status::not_found("workflow result missing"))?;

        if persist_results {
            let mut workflow_clone = workflow.clone();
            self.persist_workflow_results(&mut workflow_clone, &workflow_code_id, &results)
                .await?;
        }

        let response = RunWorkflowResponse {
            workflow_result: Some(latest_result.clone()),
            status: Self::ok_status("workflow executed successfully"),
        };

        info!(
            "workflow executed: workflow_id={workflow_id}, workflow_code_id={workflow_code_id}, result_revision={result_revision}",
            workflow_id = workflow.id.as_str(),
            workflow_code_id = workflow_code_id.as_str(),
            result_revision = latest_result.workflow_result_revision
        );

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::permission::{CheckPermissionResult, check_permission};
    use sapphillon_core::proto::google::protobuf::Timestamp;
    use sapphillon_core::proto::sapphillon::v1::{
        Permission, PermissionLevel, PermissionType, WorkflowResultType,
    };
    use tonic::Code;

    fn base_timestamp() -> Timestamp {
        Timestamp {
            seconds: 1,
            nanos: 0,
        }
    }

    fn base_workflow() -> Workflow {
        Workflow {
            id: "wf-1".to_string(),
            display_name: "Original Workflow".to_string(),
            description: "baseline".to_string(),
            workflow_language: WORKFLOW_LANGUAGE_JS,
            workflow_code: vec![WorkflowCode {
                id: "code-1".to_string(),
                code_revision: 1,
                code: "function workflow() {}".to_string(),
                language: WORKFLOW_LANGUAGE_JS,
                created_at: Some(base_timestamp()),
                result: vec![WorkflowResult {
                    id: "result-1".to_string(),
                    display_name: "Initial".to_string(),
                    description: "seed".to_string(),
                    result: "{}".to_string(),
                    ran_at: Some(base_timestamp()),
                    result_type: WorkflowResultType::SuccessUnspecified as i32,
                    exit_code: 0,
                    workflow_result_revision: 1,
                }],
                plugin_packages: vec![],
                plugin_function_ids: vec![],
                allowed_permissions: vec![],
            }],
            created_at: Some(base_timestamp()),
            updated_at: Some(base_timestamp()),
            workflow_results: vec![WorkflowResult {
                id: "result-1".to_string(),
                display_name: "Initial".to_string(),
                description: "seed".to_string(),
                result: "{}".to_string(),
                ran_at: Some(base_timestamp()),
                result_type: WorkflowResultType::SuccessUnspecified as i32,
                exit_code: 0,
                workflow_result_revision: 1,
            }],
        }
    }

    #[test]
    fn sanitize_generated_code_appends_workflow_call() {
        let raw = "function workflow() {\n  return 42;\n}";
        let sanitized = MyWorkflowService::sanitize_generated_code(raw);
        assert!(sanitized.ends_with("workflow();"));
    }

    #[test]
    fn sanitize_generated_code_preserves_existing_call() {
        let raw = "function workflow() {}\nworkflow();";
        let sanitized = MyWorkflowService::sanitize_generated_code(raw);
        assert_eq!(sanitized, "function workflow() {}\nworkflow();");
    }

    #[test]
    fn derive_display_name_truncates_long_input() {
        let long = "a".repeat(200);
        let derived = MyWorkflowService::derive_display_name(&long);
        assert!(!derived.is_empty());
        assert!(derived.len() <= MAX_DISPLAY_NAME_LEN);
    }

    #[test]
    fn encode_decode_page_token_round_trip() {
        let offset = 12345_u64;
        let token = MyWorkflowService::encode_page_token(offset);
        assert_eq!(MyWorkflowService::decode_page_token(&token), offset);
    }

    #[test]
    fn apply_update_mask_overrides_listed_fields() {
        let existing = base_workflow();
        let mut incoming = existing.clone();
        incoming.display_name = "Updated Workflow".to_string();
        incoming.description = "new description".to_string();
        incoming.workflow_language = WORKFLOW_LANGUAGE_UNSPECIFIED;
        incoming.updated_at = Some(base_timestamp());

        let mask = vec!["display_name".to_string(), "description".to_string()];
        let result = MyWorkflowService::apply_update_mask(&existing, &incoming, &mask).unwrap();

        assert_eq!(result.display_name, "Updated Workflow");
        assert_eq!(result.description, "new description");
        assert_eq!(result.workflow_language, existing.workflow_language);
    }

    #[test]
    fn apply_update_mask_rejects_unknown_field() {
        let existing = base_workflow();
        let incoming = existing.clone();
        let mask = vec!["unsupported".to_string()];
        let err = MyWorkflowService::apply_update_mask(&existing, &incoming, &mask).unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn build_core_permissions_preserves_multiple_permissions() {
        let mut workflow = base_workflow();
        let workflow_code = workflow
            .workflow_code
            .get_mut(0)
            .expect("base workflow has at least one code");

        workflow_code.plugin_function_ids = vec!["func1".to_string(), "func2".to_string()];
        workflow_code.allowed_permissions = vec![
            AllowedPermission {
                plugin_function_id: "func1".to_string(),
                permissions: vec![Permission {
                    display_name: "p1".to_string(),
                    description: "d1".to_string(),
                    permission_type: PermissionType::NetAccess as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["r1".to_string()],
                }],
            },
            AllowedPermission {
                plugin_function_id: "func2".to_string(),
                permissions: vec![Permission {
                    display_name: "p2".to_string(),
                    description: "d2".to_string(),
                    permission_type: PermissionType::FilesystemRead as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["r2".to_string()],
                }],
            },
        ];

        let (required, allowed) = MyWorkflowService::build_core_permissions(workflow_code);

        assert_eq!(required.len(), 2);
        assert_eq!(allowed.len(), 2);
        assert_eq!(required[0].plugin_function_id, "func1");
        assert_eq!(required[1].plugin_function_id, "func2");
        assert_eq!(allowed[0].plugin_function_id, "func1");
        assert_eq!(allowed[1].plugin_function_id, "func2");

        assert_eq!(required[0].permissions.permissions.len(), 1);
        assert_eq!(required[1].permissions.permissions.len(), 1);
    }

    #[test]
    fn missing_allowed_permission_results_in_denial() {
        let mut workflow = base_workflow();
        let workflow_code = workflow
            .workflow_code
            .get_mut(0)
            .expect("base workflow has at least one code");

        workflow_code.plugin_function_ids = vec!["func1".to_string(), "func2".to_string()];
        workflow_code.allowed_permissions = vec![
            AllowedPermission {
                plugin_function_id: "func1".to_string(),
                permissions: vec![Permission {
                    display_name: "p1".to_string(),
                    description: "d1".to_string(),
                    permission_type: PermissionType::NetAccess as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["r1".to_string()],
                }],
            },
            AllowedPermission {
                plugin_function_id: "func2".to_string(),
                permissions: vec![Permission {
                    display_name: "p2".to_string(),
                    description: "d2".to_string(),
                    permission_type: PermissionType::FilesystemRead as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["r2".to_string()],
                }],
            },
        ];

        let (required, allowed) = MyWorkflowService::build_core_permissions(workflow_code);

        // Only grant permissions for func1, omit func2
        let granted_for_func1 = allowed[0].permissions.clone();
        let required_func1 = required[0].permissions.clone();
        let required_func2 = required[1].permissions.clone();

        // func1 is permitted
        let ok = check_permission(&granted_for_func1, &required_func1);
        assert!(matches!(ok, CheckPermissionResult::Ok));

        // func2 is missing, should be denied
        let denied = check_permission(&granted_for_func1, &required_func2);
        assert!(matches!(
            denied,
            CheckPermissionResult::MissingPermission(_)
        ));
    }
}
