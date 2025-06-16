// gRPC server startup logic

use crate::services::{MyGreeter, hello_world::greeter_server::GreeterServer};
use tonic::transport::Server;
use log::info;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    info!("gRPC Server starting on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
