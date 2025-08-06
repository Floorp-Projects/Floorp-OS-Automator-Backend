// gRPC server startup logic

use sapphillon_core::proto::sapphillon::v1::version_service_server::VersionServiceServer;
use crate::services::MyVersionService;
use log::info;
use tonic::transport::Server;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let version_service = MyVersionService;

    info!("gRPC Server starting on {addr}");

    Server::builder()
        .add_service(VersionServiceServer::new(version_service))
        .serve(addr)
        .await?;

    Ok(())
}
