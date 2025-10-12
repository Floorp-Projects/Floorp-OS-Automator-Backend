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

use crate::services::{MyModelService, MyProviderService, MyVersionService, MyWorkflowService};
use log::info;
use sapphillon_core::proto::sapphillon::ai::v1::model_service_server::ModelServiceServer;
use sapphillon_core::proto::sapphillon::ai::v1::provider_service_server::ProviderServiceServer;
use sapphillon_core::proto::sapphillon::v1::version_service_server::VersionServiceServer;
use sapphillon_core::proto::sapphillon::v1::workflow_service_server::WorkflowServiceServer;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;

/// Boots the gRPC server, wiring service implementations and enabling web compatibility.
///
/// # Arguments
///
/// This asynchronous function takes no arguments.
///
/// # Returns
///
/// Returns `Ok(())` when the server shuts down cleanly or an error if any initialization step fails.
pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let version_service = MyVersionService {};
    let workflow_service = MyWorkflowService {};
    let provider_connection = crate::GLOBAL_STATE
        .wait_init_and_get_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to obtain database connection for provider service: {err:?}");
            err
        })?;
    let provider_service = MyProviderService::new(provider_connection);

    let model_connection = crate::GLOBAL_STATE
        .wait_init_and_get_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to obtain database connection for model service: {err:?}");
            err
        })?;
    let model_service = MyModelService::new(model_connection);

    let reflection_service_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::sapphillon::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::sapphillon::ai::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::rpc::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::rpc::context::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::r#type::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::expr::v1alpha1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::expr::v1beta1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::bytestream::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::longrunning::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::geo::r#type::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::protobuf::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::protobuf::compiler::FILE_DESCRIPTOR_SET,
        )
        .build_v1()
        .unwrap();

    let reflection_service_v1_alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::sapphillon::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::sapphillon::ai::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::rpc::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::rpc::context::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::r#type::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::expr::v1alpha1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::api::expr::v1beta1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::bytestream::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::longrunning::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::geo::r#type::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::protobuf::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            sapphillon_core::proto::google::protobuf::compiler::FILE_DESCRIPTOR_SET,
        )
        .build_v1alpha()
        .unwrap();

    info!("gRPC Server starting on {addr}");

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("grpc_server")) // Add tracing span
        .accept_http1(true)
        .layer(cors)
        .add_service(tonic_web::enable(reflection_service_v1_alpha))
        .add_service(tonic_web::enable(reflection_service_v1))
        .add_service(tonic_web::enable(VersionServiceServer::new(
            version_service,
        )))
        .add_service(tonic_web::enable(WorkflowServiceServer::new(
            workflow_service,
        )))
        .add_service(tonic_web::enable(ModelServiceServer::new(model_service)))
        .add_service(tonic_web::enable(ProviderServiceServer::new(
            provider_service,
        )))
        .serve(addr)
        .await?;

    Ok(())
}
