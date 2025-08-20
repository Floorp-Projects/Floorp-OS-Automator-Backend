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

// gRPC server startup logic

use crate::services::{MyVersionService, MyWorkflowService};
use log::info;
use sapphillon_core::proto::sapphillon::v1::version_service_server::VersionServiceServer;
use sapphillon_core::proto::sapphillon::v1::workflow_service_server::WorkflowServiceServer;
use tonic::transport::Server;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let version_service = MyVersionService {};
    let workflow_service = MyWorkflowService {};


    info!("gRPC Server starting on {addr}");

    Server::builder()
        .add_service(VersionServiceServer::new(version_service))
        .add_service(WorkflowServiceServer::new(workflow_service))
        .serve(addr)
        .await?;

    Ok(())
}
