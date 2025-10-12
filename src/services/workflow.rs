// Sapphillon
// Copyright 2025 Yuta Takahashi
//
// This file is part of Sapphillon
//
// Sapphillon is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use sapphillon_core::proto::sapphillon::v1::workflow_service_server::WorkflowService;
use sapphillon_core::proto::sapphillon::v1::{
    DeleteWorkflowRequest, DeleteWorkflowResponse, FixWorkflowRequest, FixWorkflowResponse,
    GenerateWorkflowRequest, GenerateWorkflowResponse, GetWorkflowRequest, GetWorkflowResponse,
    ListWorkflowsRequest, ListWorkflowsResponse, RunWorkflowRequest, RunWorkflowResponse,
    UpdateWorkflowRequest, UpdateWorkflowResponse, Workflow, WorkflowCode,
};
use sapphillon_core::workflow::CoreWorkflowCode;

use crate::workflow::generate_workflow_async;
use std::pin::Pin;
use tokio_stream::Stream;

#[derive(Debug, Default)]
pub struct MyWorkflowService;

#[tonic::async_trait]
impl WorkflowService for MyWorkflowService {
    type FixWorkflowStream = Pin<
        Box<
            dyn Stream<Item = std::result::Result<FixWorkflowResponse, tonic::Status>>
                + Send
                + 'static,
        >,
    >;
    type GenerateWorkflowStream = Pin<
        Box<
            dyn Stream<Item = std::result::Result<GenerateWorkflowResponse, tonic::Status>>
                + Send
                + 'static,
        >,
    >;

    /// Returns an unimplemented status for workflow updates until persistence is supported.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming update payload (currently unused).
    ///
    /// # Returns
    ///
    /// Always returns an unimplemented error status.
    async fn update_workflow(
        &self,
        request: tonic::Request<UpdateWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<UpdateWorkflowResponse>, tonic::Status> {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented(
            "update_workflow is not implemented",
        ))
    }
    /// Returns an unimplemented status for workflow deletion until persistence is supported.
    ///
    /// # Arguments
    ///
    /// * `request` - The deletion request payload (currently unused).
    ///
    /// # Returns
    ///
    /// Always returns an unimplemented error status.
    async fn delete_workflow(
        &self,
        request: tonic::Request<DeleteWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<DeleteWorkflowResponse>, tonic::Status> {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented(
            "delete_workflow is not implemented",
        ))
    }
    /// Returns an unimplemented status because listing workflows is not yet supported.
    ///
    /// # Arguments
    ///
    /// * `request` - The list request payload (currently unused).
    ///
    /// # Returns
    ///
    /// Always returns an unimplemented error status.
    async fn list_workflows(
        &self,
        request: tonic::Request<ListWorkflowsRequest>,
    ) -> std::result::Result<tonic::Response<ListWorkflowsResponse>, tonic::Status> {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented(
            "list_workflow is not implemented",
        ))
    }
    /// Returns an unimplemented status because workflow fixing streams are not yet implemented.
    ///
    /// # Arguments
    ///
    /// * `request` - The fix request payload (currently unused).
    ///
    /// # Returns
    ///
    /// Always returns an unimplemented error status.
    async fn fix_workflow(
        &self,
        request: tonic::Request<FixWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<Self::FixWorkflowStream>, tonic::Status> {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented(
            "fix_workflow is not implemented",
        ))
    }

    /// Returns an unimplemented status because workflow retrieval is not yet supported.
    ///
    /// # Arguments
    ///
    /// * `request` - The get request payload (currently unused).
    ///
    /// # Returns
    ///
    /// Always returns an unimplemented error status.
    async fn get_workflow(
        &self,
        request: tonic::Request<GetWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<GetWorkflowResponse>, tonic::Status> {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented(
            "get_workflow is not implemented",
        ))
    }
    /// Generates a workflow by invoking the asynchronous LLM helper and streaming the result.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the prompt used to synthesize a workflow.
    ///
    /// # Returns
    ///
    /// Returns a streaming response with a single generated workflow or an error if generation fails.
    async fn generate_workflow(
        &self,
        request: tonic::Request<GenerateWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<Self::GenerateWorkflowStream>, tonic::Status> {
        // 未実装のためエラーを返す
        let prompt = request.into_inner().prompt;

        // Generate Workflow Code
        let workflow_code_raw = generate_workflow_async(&prompt)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to generate workflow: {e}")))?;
        let workflow_code_raw = workflow_code_raw + "workflow();";

        let workflow_code = WorkflowCode {
            id: uuid::Uuid::new_v4().to_string(),
            code_revision: 1,
            code: workflow_code_raw,
            language: 0,
            created_at: None,
            result: vec![],
            plugin_packages: vec![],
            plugin_function_ids: vec![],
            allowed_permissions: vec![],
        };

        let workflow = Workflow {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Generated Workflow".to_string(),
            description: "This is a generated workflow".to_string(),
            workflow_language: 0,
            workflow_code: vec![workflow_code],
            created_at: None,
            updated_at: None,
            workflow_results: vec![],
        };

        let response = GenerateWorkflowResponse {
            workflow_definition: Some(workflow),
            status: Some(sapphillon_core::proto::google::rpc::Status {
                code: 0,
                message: "Workflow generated successfully".to_string(),
                details: vec![],
            }),
        };

        // return the response
        // stream the single response back to the client
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        // move the response into a background task so we can return a stream immediately
        tokio::spawn(async move {
            // send the response; ignore error if receiver was dropped
            let _ = tx.send(Ok(response)).await;
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let boxed_stream: Self::GenerateWorkflowStream =
            Box::pin(stream) as Self::GenerateWorkflowStream;

        Ok(tonic::Response::new(boxed_stream))
    }

    /// Executes the provided workflow definition and returns the latest result entry.
    ///
    /// # Arguments
    ///
    /// * `request` - Either a workflow identifier or an inline definition to execute.
    ///
    /// # Returns
    ///
    /// Returns the most recent execution result, or an error when validation fails or the workflow cannot be located.
    async fn run_workflow(
        &self,
        request: tonic::Request<RunWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<RunWorkflowResponse>, tonic::Status> {
        let req = request.into_inner();

        // The proto defines RunWorkflowRequest with a `oneof source` which is either
        // ById(WorkflowSourceById) or WorkflowDefinition(Workflow). Validate and
        // extract the chosen source here.
        let source = req.source.ok_or_else(|| {
            tonic::Status::invalid_argument(
                "RunWorkflowRequest.source is required (ById or WorkflowDefinition)",
            )
        })?;

        // Construct a placeholder Workflow until a real storage lookup is implemented.
        let mut workflow: Workflow = match source {
            sapphillon_core::proto::sapphillon::v1::run_workflow_request::Source::ById(byid) => {
                log::debug!("Received request, Workflow Code Id: {}, Workflow Id: {}", byid.workflow_code_id, byid.workflow_id);
                Err(tonic::Status::unimplemented(
                    "RunWorkflowRequest ById is not implemented",
                ))?
            }
            sapphillon_core::proto::sapphillon::v1::run_workflow_request::Source::WorkflowDefinition(wf) => wf,
        };
        let latest_workflow_code_revision = workflow
            .workflow_code
            .iter()
            .map(|code| code.code_revision)
            .max()
            .unwrap_or(0);

        let workflow_code = workflow
            .workflow_code
            .iter_mut()
            .find(|code| code.code_revision == latest_workflow_code_revision)
            .ok_or_else(|| tonic::Status::not_found("Latest workflow code not found"))?;
        workflow_code.code = unescaper::unescape(&workflow_code.code).unwrap();

        log::debug!("Parsed workflow code: {}", workflow_code.code);

        // Convert protobuf AllowedPermission entries into the core PluginFunctionPermissions
        // Use the first allowed_permissions entry if present, otherwise, if plugin_function_ids
        // contains at least one id, create a default permissive entry for that function id
        // with empty Resources (meaning no specific resources granted).
        let allowed_permissions_proto = &workflow_code.allowed_permissions;
        let allowed_permissions: Option<sapphillon_core::permission::PluginFunctionPermissions> =
            if !allowed_permissions_proto.is_empty() {
                let ap = &allowed_permissions_proto[0];
                Some(sapphillon_core::permission::PluginFunctionPermissions {
                    plugin_function_id: ap.plugin_function_id.clone(),
                    permissions: sapphillon_core::permission::Permissions::new(
                        ap.permissions.clone(),
                    ),
                })
            } else if !workflow_code.plugin_function_ids.is_empty() {
                // fallback: use the first plugin_function_id with empty permissions
                Some(sapphillon_core::permission::PluginFunctionPermissions {
                    plugin_function_id: workflow_code.plugin_function_ids[0].clone(),
                    permissions: sapphillon_core::permission::Permissions::new(vec![]),
                })
            } else {
                None
            };

        // For required permissions, the proto currently doesn't have a separate field on the
        // WorkflowCode message; treat required as same as allowed for now if present.
        let required_permissions: Option<sapphillon_core::permission::PluginFunctionPermissions> =
            allowed_permissions.clone();

        let mut workflow_core = CoreWorkflowCode::new_from_proto(
            workflow_code,
            crate::sysconfig::sysconfig().core_plugin_package,
            required_permissions,
            allowed_permissions,
        );
        workflow_core.run();

        let latest_result_revision = workflow_core
            .result
            .iter()
            .map(|r| r.workflow_result_revision)
            .max()
            .unwrap_or(0);

        let workflow_core_result_latest = workflow_core
            .result
            .iter()
            .find(|r| r.workflow_result_revision == latest_result_revision);

        let res = RunWorkflowResponse {
            workflow_result: Some(workflow_core_result_latest.unwrap().clone()),
            status: Some(sapphillon_core::proto::google::rpc::Status {
                code: 0,
                message: "Workflow executed successfully".to_string(),
                details: vec![],
            }),
        };

        // Return the response
        Ok(tonic::Response::new(res))
    }
}
