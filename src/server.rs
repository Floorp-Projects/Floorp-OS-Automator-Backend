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
    
    let reflection_service_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(sapphillon_core::proto::sapphillon::v1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::rpc::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::rpc::context::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::r#type::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::expr::v1alpha1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::expr::v1beta1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::bytestream::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::longrunning::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::geo::r#type::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::protobuf::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::protobuf::compiler::FILE_DESCRIPTOR_SET)
        .build_v1().unwrap();

    let reflection_service_v1_alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(sapphillon_core::proto::sapphillon::v1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::rpc::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::rpc::context::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::r#type::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::expr::v1alpha1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::api::expr::v1beta1::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::bytestream::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::longrunning::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::geo::r#type::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::protobuf::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(sapphillon_core::proto::google::protobuf::compiler::FILE_DESCRIPTOR_SET)
        .build_v1alpha().unwrap();

    info!("gRPC Server starting on {addr}");

    Server::builder()
        .add_service(reflection_service_v1_alpha)
        .add_service(reflection_service_v1)
        .add_service(VersionServiceServer::new(version_service))
        .add_service(WorkflowServiceServer::new(workflow_service))
        .serve(addr)
        .await?;

    Ok(())
}
