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
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

// use crate::workflow::generate_workflow_async;
// use crate::workflow::generate_workflow_async;
use vscode;
use iniad;

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
        Option<PluginFunctionPermissions>,
        Option<PluginFunctionPermissions>,
    ) {
        if workflow_code.allowed_permissions.is_empty() {
             // Fallback for migration/compatibility: if no explicit permissions, try to use first plugin ID
             if let Some(first_id) = workflow_code.plugin_function_ids.first() {
                let perms = PluginFunctionPermissions {
                    plugin_function_id: first_id.clone(),
                    permissions: Permissions::new(vec![]),
                };
                (Some(perms.clone()), Some(perms))
            } else {
                (None, None)
            }
        } else {
            // Aggregate ALL permissions from all AllowedPermission entries into a single list
            let mut all_perms_vec = Vec::new();
            for ap in &workflow_code.allowed_permissions {
                 // Deep clone the inner permissions from the proto definition
                 // We assume `ap.permissions` corresponds to a list of Permission objects
                 all_perms_vec.extend(ap.permissions.clone());
            }

            // Create a wildcard permission set that applies to ANY function ID ("*")
            // This ensures that identifying logic (index 0 bugs or ID mismatches) doesn't block valid permissions.
            let wildcard_perms = PluginFunctionPermissions {
                plugin_function_id: "*".to_string(),
                permissions: Permissions::new(all_perms_vec),
            };

            // We pass this aggregated set as 'allowed'.
            // For 'required', we can pass the same or None. Passing the same is safe as it just sets the "baseline".
            (Some(wildcard_perms.clone()), Some(wildcard_perms))
        }
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

        // TEMPORARY: Hardcode verification workflow for VSCode plugin
        // Ignoring AI generation for testing purposes as requested.
        let generated = r#"
/**
 * Demo Workflow: Verify VSCode Plugin Functions
 */
function workflow() {
    console.log("Starting VSCode Plugin Verification (v2)...");

    // Simplified workflow to avoid permission issues
    const filePath = "/tmp/sapphillon_vscode_test.txt";
    const testContent = "Hello from Floorp OS Automator!";

    try {
        // Skip directory creation, use /tmp directly
        console.log("Using file path: " + filePath);


        // 2. Test write_file
        console.log("Testing vscode.write_file...");
        const writeResult = vscode.write_file(filePath, testContent);
        console.log("write_file result: " + writeResult);
        
        // Busy wait loop
        const start1 = Date.now();
        while (Date.now() - start1 < 2000) {}
        // 3. Test open_file
        console.log("Testing vscode.open_file...");
        const openResult = vscode.open_file(filePath);
        console.log("open_file result: " + openResult);
        
        const start2 = Date.now();
        while (Date.now() - start2 < 1000) {}

        // 4. Test get_active_file_content
        console.log("Testing vscode.get_active_file_content...");
        const content = vscode.get_active_file_content();
        console.log("get_active_file_content result length: " + content.length);
        
        if (content.trim() === testContent.trim()) {
            console.log("SUCCESS: Content matches!");
        } else {
            console.error("FAILURE: Content mismatch!");
            console.error("Expected: " + testContent);
            console.error("Actual: " + content);
        }

        // 5. Test open_folder
        console.log("Testing vscode.open_folder...");
        const folderResult = vscode.open_folder("/tmp");
        console.log("open_folder result: " + folderResult);
        
        const start3 = Date.now();
        while (Date.now() - start3 < 2000) {}

        // 6. Test close_workspace
        console.log("Testing vscode.close_workspace...");
        const closeResult = vscode.close_workspace();
        console.log("close_workspace result: " + closeResult);

        console.log("Verification checks completed.");

    } catch (e) {
        console.error("Verification failed with error: " + e);
    }
}
"#.to_string();

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
                plugin_function_ids: vec![
                    "app.sapphillon.core.vscode.open_folder".to_string(),
                    "app.sapphillon.core.vscode.open_file".to_string(),
                    "app.sapphillon.core.vscode.write_file".to_string(),
                    "app.sapphillon.core.vscode.close_workspace".to_string(),
                    "app.sapphillon.core.vscode.get_active_file_content".to_string(),
                ],
                allowed_permissions: vec![
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.write_file".to_string(),
                        permissions: vscode::vscode_write_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.open_file".to_string(),
                        permissions: vscode::vscode_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.get_active_file_content".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.open_folder".to_string(),
                        permissions: vscode::vscode_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.close_workspace".to_string(),
                        permissions: vscode::vscode_plugin_permissions(),
                    },
                ],
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

        // Demo Workflow: VSCode to Floorp Form
        // VSCodeのアクティブファイル内容を取得し、Floorpのフォームに入力
        let generated = r##"
/**
 * Demo Workflow 1: VSCode to Floorp Form
 *
 * このワークフローは以下の処理を行います:
 * 1. VSCodeでアクティブに表示されているファイルの内容を取得
 * 2. Floorpでアクティブなタブのフォームに内容を入力
 *
 * 使用する前提:
 * - VSCodeでファイルが開かれている状態
 * - Floorp OSサーバーが起動している
 * - Floorpで入力先のフォームがあるページが開かれている
 */

function workflow() {
  try {
    // Step 1: VSCodeからワークスペースパスを取得
    console.log("Step 1: Getting workspace path and current branch...");
    let repoPath = "";
    try {
        repoPath = vscode.get_workspace_path();
        console.log("DEBUG: Raw workspace path returned: [" + repoPath + "]");
        console.log("DEBUG: Path length: " + repoPath.length);
    } catch (e) {
        console.log("Failed to get workspace path: " + e);
        throw new Error("Could not determine repository path from VSCode");
    }
    
    let currentBranch = "main";
    try {
        const branchJson = git.getBranch(repoPath);
        console.log("DEBUG: Raw branchJson: " + branchJson);
        const branchInfo = JSON.parse(branchJson);
        currentBranch = branchInfo.branch;
        console.log("DEBUG: Parsed branch: [" + currentBranch + "]");
    } catch (gitError) {
        console.log("Failed to get branch: " + gitError);
    }
    
    console.log("Final values - repoPath: [" + repoPath + "], branch: [" + currentBranch + "]");

    // Step 2: Get git diff for AI analysis
    console.log("Step 2: Getting git diff...");
    let diffContent = "";
    try {
        const diffJson = git.getDiff(repoPath);
        const diffInfo = JSON.parse(diffJson);
        diffContent = diffInfo.combined || "";
        console.log("Diff length: " + diffContent.length + " chars");
    } catch (e) {
        console.log("Failed to get diff: " + e);
    }

    // Step 3: Generate commit message using AI
    console.log("Step 3: Generating commit message with AI...");
    let commitMessage = "chore: automated commit";
    if (diffContent.length > 0) {
        try {
            commitMessage = iniad.generateCommitMessage(diffContent);
            console.log("AI generated commit message: " + commitMessage);
        } catch (e) {
            console.log("AI commit message failed: " + e + ", using default.");
        }
    }

    // Step 4: Stage, commit, and push
    console.log("Step 4: Git add, commit, push...");
    try {
        const addResult = git.add(repoPath);
        console.log("Git add: " + addResult);
    } catch (e) {
        console.log("Git add failed: " + e);
    }

    try {
        const commitResult = git.commit(repoPath, commitMessage);
        console.log("Git commit: " + commitResult);
    } catch (e) {
        console.log("Git commit failed (may be nothing to commit): " + e);
    }

    try {
        const pushResult = git.push(repoPath);
        console.log("Git push: " + pushResult);
    } catch (e) {
        console.log("Git push failed: " + e);
    }

    // Step 5: Prepare Floorp tab for PR creation
    console.log("Step 5: Preparing Floorp tab for PR...");
    // Use ?expand=1 to automatically expand the PR form without clicking "Create pull request" button
    const targetUrl = "https://github.com/Floorp-Projects/Floorp/compare/main..." + currentBranch + "?expand=1";
    console.log("Target URL: " + targetUrl);
    const tabsJson = floorp.listBrowserTabs();
    const tabs = JSON.parse(tabsJson);
    let targetTab = null;

    // Look for existing compare tab (with or without expand param)
    for (var i = 0; i < tabs.length; i++) {
        if (tabs[i].url && tabs[i].url.includes("compare/main..." + currentBranch)) {
            targetTab = tabs[i];
            break;
        }
    }

    let instanceId = "";
    if (targetTab) {
        console.log("Found existing target tab: " + targetTab.title);
        const attachResultJson = floorp.attachToTab(targetTab.browserId);
        const attachResult = JSON.parse(attachResultJson);
        instanceId = attachResult.instanceId;
        
        // Navigate to ?expand=1 URL to ensure form is expanded
        console.log("Navigating existing tab to expanded URL...");
        try {
            floorp.tabNavigate(instanceId, targetUrl);
            console.log("Navigation initiated to: " + targetUrl);
        } catch (navError) {
            console.log("Navigation warning: " + navError);
        }
    } else {
        console.log("Creating new tab...");
        try {
            const createResultJson = floorp.createTab(targetUrl, false);
            const createResult = JSON.parse(createResultJson);
            instanceId = createResult.instanceId;
            console.log("Created new tab, instanceId: " + instanceId);
        } catch (createError) {
            console.log("Failed to create tab: " + createError);
            throw createError;
        }
    }

    // Step 6: Wait for PR form (with extended waiting)
    console.log("Step 6: Waiting for PR form...");
    try {
        floorp.tabWaitForElement(instanceId, "#pull_request_title", "20000");
        console.log("PR form found!");
    } catch (e) {
        console.log("Wait for element warning (first attempt): " + e);
        // Try waiting again
        try {
            floorp.tabWaitForElement(instanceId, "#pull_request_title", "10000");
            console.log("PR form found on second attempt!");
        } catch (e2) {
            console.log("Wait for element warning (second attempt): " + e2);
        }
    }

    // Step 7: Generate PR title/body with AI
    console.log("Step 7: Generating PR title/body using AI...");
    let aiResult;
    try {
        const aiJson = iniad.generatePrDescription(diffContent);
        console.log("DEBUG: AI response JSON: " + aiJson);
        aiResult = JSON.parse(aiJson);
        console.log("DEBUG: Parsed aiResult.title: " + aiResult.title);
        console.log("DEBUG: Parsed aiResult.body length: " + (aiResult.body ? aiResult.body.length : 0));
    } catch (aiError) {
        console.log("AI generation failed: " + aiError);
        aiResult = {
            title: commitMessage,
            body: "Automated PR" + String.fromCharCode(10,10) + diffContent.substring(0, 1000)
        };
        console.log("DEBUG: Using fallback title: " + aiResult.title);
    }

    // Step 8: Fill PR form
    console.log("Step 8: Filling PR form...");
    console.log("DEBUG: About to fill title with: [" + aiResult.title + "]");
    
    let titleFillSuccess = false;
    let bodyFillSuccess = false;
    
    try {
        const titleResultJson = floorp.tabFillForm(instanceId, "#pull_request_title", aiResult.title);
        console.log("Title fill result: " + titleResultJson);
        const titleResult = JSON.parse(titleResultJson);
        titleFillSuccess = titleResult.ok === true;
    } catch (e) {
        console.log("Fill title error: " + e);
    }

    console.log("DEBUG: About to fill body...");
    try {
        const bodyResultJson = floorp.tabFillForm(instanceId, "#pull_request_body", aiResult.body);
        console.log("Body fill result: " + bodyResultJson);
        const bodyResult = JSON.parse(bodyResultJson);
        bodyFillSuccess = bodyResult.ok === true;
    } catch (e) {
        console.log("Fill body error: " + e);
    }

    // Abort if form fill failed
    if (!titleFillSuccess || !bodyFillSuccess) {
        console.log("ERROR: Form fill failed! Title: " + titleFillSuccess + ", Body: " + bodyFillSuccess);
        console.log("Aborting PR creation to prevent incomplete PR.");
        return {
            ok: false,
            message: "Form fill failed. Title filled: " + titleFillSuccess + ", Body filled: " + bodyFillSuccess,
            error: "フォーム入力に失敗しました。ページが正しく読み込まれていない可能性があります。"
        };
    }

    console.log("Form fill successful! Proceeding to create PR...");

    // Step 9: Click Create PR button
    console.log("Step 9: Clicking Create PR button...");
    try {
        floorp.tabClick(instanceId, ".hx_create-pr-button");
        console.log("Clicked creation button.");
    } catch (e) {
        console.log("Failed to click button: " + e);
        try {
            floorp.tabClick(instanceId, "button.btn-primary");
            console.log("Clicked fallback button.");
        } catch (e2) {
            console.log("Failed to click fallback: " + e2);
        }
    }

    return {
      ok: true,
      message: "Full automation complete: commit, push, PR created",
      commitMessage: commitMessage,
      prTitle: aiResult.title
    };

  } catch (error) {
    return {
      ok: false,
      message: "Workflow failed: " + error,
    };
  }
}
"##.to_string();
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
                plugin_function_ids: vec![
                    // VSCode plugin
                    "app.sapphillon.core.vscode.get_active_file_content".to_string(),
                    "app.sapphillon.core.vscode.get_workspace_path".to_string(),
                    // Floorp plugin (camelCase)
                    "app.sapphillon.core.floorp.listBrowserTabs".to_string(),
                    "app.sapphillon.core.floorp.attachToTab".to_string(),
                    "app.sapphillon.core.floorp.tabFillForm".to_string(),
                    "app.sapphillon.core.floorp.createTabInstance".to_string(),
                    "app.sapphillon.core.floorp.tabClickElement".to_string(),
                    "app.sapphillon.core.floorp.tabWaitForElement".to_string(),
                    // INIAD plugin
                    "app.sapphillon.core.iniad.generate_pr_description".to_string(),
                    "app.sapphillon.core.iniad.generate_commit_message".to_string(),
                    // Git plugin
                    "app.sapphillon.core.git.getBranch".to_string(),
                    "app.sapphillon.core.git.getDiff".to_string(),
                    "app.sapphillon.core.git.add".to_string(),
                    "app.sapphillon.core.git.commit".to_string(),
                    "app.sapphillon.core.git.push".to_string(),
                ],
                allowed_permissions: vec![
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.get_active_file_content".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.vscode.get_workspace_path".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.listBrowserTabs".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.attachToTab".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.tabFillForm".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.createTabInstance".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.tabClickElement".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.floorp.tabWaitForElement".to_string(),
                        permissions: vscode::vscode_get_content_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.iniad.generate_pr_description".to_string(),
                        permissions: iniad::iniad_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.iniad.generate_commit_message".to_string(),
                        permissions: iniad::iniad_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.git.getBranch".to_string(),
                        permissions: git::git_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.git.getDiff".to_string(),
                        permissions: git::git_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.git.add".to_string(),
                        permissions: git::git_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.git.commit".to_string(),
                        permissions: git::git_plugin_permissions(),
                    },
                    AllowedPermission {
                        plugin_function_id: "app.sapphillon.core.git.push".to_string(),
                        permissions: git::git_plugin_permissions(),
                    },
                ],
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

            workflow_core.run();

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
    use sapphillon_core::proto::google::protobuf::Timestamp;
    use sapphillon_core::proto::sapphillon::v1::WorkflowResultType;
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
}
