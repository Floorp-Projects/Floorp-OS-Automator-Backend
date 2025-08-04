use log::info;
use tonic::{Request, Response, Status};

// Import the generated protobuf types
use crate::proto_generated::{GetVersionRequest, GetVersionResponse, Version};
use crate::proto_generated::version_service_server::VersionService;

#[derive(Debug, Default)]
pub struct MyVersionService;

#[tonic::async_trait]
impl VersionService for MyVersionService {
    async fn get_version(
        &self,
        request: Request<GetVersionRequest>,
    ) -> Result<Response<GetVersionResponse>, Status> {
        info!("Got a version request: {:?}", request);
        
        let response = GetVersionResponse {
            version: Some(Version {
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
        };
        
        Ok(Response::new(response))
    }
}
