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

use fetch::fetch_plugin_package;
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
            required_permissions: vec![],
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

    async fn run_workflow(
        &self,
        request: tonic::Request<RunWorkflowRequest>,
    ) -> std::result::Result<tonic::Response<RunWorkflowResponse>, tonic::Status> {
        let req = request.into_inner();
        if req.workflow_id.is_empty() && req.workflow_code_id.is_empty() {
            return Err(tonic::Status::invalid_argument(
                "Either workflow_id or workflow_code_id is required",
            ));
        }
        // Construct a placeholder Workflow until a real storage lookup is implemented.
        let mut workflow = if !req.workflow_id.is_empty() {
            Workflow {
                id: req.workflow_id.clone(),
                ..Default::default()
            }
        } else {
            let wc = WorkflowCode {
                id: req.workflow_code_id.clone(),
                code_revision: 1,
                code: "".to_string(),
                language: 0,
                created_at: None,
                result: vec![],
                required_permissions: vec![],
                plugin_packages: vec![],
                plugin_function_ids: vec![],
                allowed_permissions: vec![],
            };
            Workflow {
                workflow_code: vec![wc],
                ..Default::default()
            }
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

        let mut workflow_core = CoreWorkflowCode::new_from_proto(
            workflow_code,
            vec![fetch_plugin_package()],
            None,
            None,
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
