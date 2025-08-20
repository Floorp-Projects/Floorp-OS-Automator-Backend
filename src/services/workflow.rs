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
    FixWorkflowRequest, FixWorkflowResponse, GenerateWorkflowRequest, GenerateWorkflowResponse,
    WorkflowCode, Workflow
};

use tokio_stream::Stream;
use std::pin::Pin;
use crate::workflow::generate_workflow;
use fetch::fetch_plugin_package;

#[derive(Debug, Default)]
pub struct MyWorkflowService {}

#[tonic::async_trait]
impl WorkflowService for MyWorkflowService {
    type FixWorkflowStream = Pin<
        Box<dyn Stream<Item = std::result::Result<FixWorkflowResponse, tonic::Status>> + Send + 'static>,
    >;
    type GenerateWorkflowStream =
        Pin<Box<dyn Stream<Item = std::result::Result<GenerateWorkflowResponse, tonic::Status>> + Send + 'static>>;

    async fn fix_workflow(
        &self,
        request: tonic::Request<FixWorkflowRequest>,
    ) -> std::result::Result<
        tonic::Response<Self::FixWorkflowStream>,
        tonic::Status,
    > {
        // 未実装のためエラーを返す
        let _ = request;
        Err(tonic::Status::unimplemented("fix_workflow is not implemented"))
    }

    async fn generate_workflow(
        &self,
        request: tonic::Request<GenerateWorkflowRequest>,
    ) -> std::result::Result<
        tonic::Response<Self::GenerateWorkflowStream>,
        tonic::Status,
    > {
        // 未実装のためエラーを返す
        let prompt = request.into_inner().prompt;
        
        // Generate Workflow Code
        let workflow_code_raw = generate_workflow(&prompt)
            .map_err(|e| tonic::Status::internal(format!("Failed to generate workflow: {e}")))?;
        
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



        Err(tonic::Status::unimplemented("generate_workflow is not implemented"))
    }
}
    