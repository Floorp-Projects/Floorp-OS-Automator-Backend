// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

// gRPC server startup logic

use crate::services::{
    MyModelService, MyPluginService, MyProviderService, MyVersionService, MyWorkflowService,
};
use log::info;
use sapphillon_core::proto::sapphillon::ai::v1::model_service_server::ModelServiceServer;
use sapphillon_core::proto::sapphillon::ai::v1::provider_service_server::ProviderServiceServer;
use sapphillon_core::proto::sapphillon::v1::plugin_service_server::PluginServiceServer;
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
    let workflow_connection = crate::GLOBAL_STATE
        .wait_init_and_get_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to obtain database connection for workflow service: {err:?}");
            err
        })?;
    let workflow_service = MyWorkflowService::new(workflow_connection);
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

    let plugin_connection = crate::GLOBAL_STATE
        .wait_init_and_get_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to obtain database connection for plugin service: {err:?}");
            err
        })?;
    let plugin_service = MyPluginService::new(plugin_connection);

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
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(reflection_service_v1_alpha)
        .add_service(reflection_service_v1)
        .add_service(VersionServiceServer::new(version_service))
        .add_service(WorkflowServiceServer::new(workflow_service))
        .add_service(ModelServiceServer::new(model_service))
        .add_service(ProviderServiceServer::new(provider_service))
        .add_service(PluginServiceServer::new(plugin_service))
        .serve(addr)
        .await?;

    Ok(())
}
