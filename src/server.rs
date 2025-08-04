// gRPC server startup logic

use crate::services::MyVersionService;
use crate::proto_generated::version_service_server::VersionServiceServer;
use log::info;
use tonic::transport::Server;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let version_service = MyVersionService::default();

    info!("gRPC Server starting on {addr}");

    Server::builder()
        .add_service(VersionServiceServer::new(version_service))
        .serve(addr)
        .await?;

    Ok(())
}
