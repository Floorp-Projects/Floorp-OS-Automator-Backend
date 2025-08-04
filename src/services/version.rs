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

mod test {
    use tonic::Request;
    use crate::proto_generated::{GetVersionRequest, Version};
    use crate::services::version::MyVersionService;
    use crate::proto_generated::version_service_server::VersionService;

#[tokio::test]
async fn test_get_version() {
    let service = MyVersionService::default();
    let request = Request::new(GetVersionRequest {});
    let response = service.get_version(request).await.unwrap().into_inner();
    assert!(response.version.is_some());
    let version = response.version.unwrap();
    // The version string should match the crate version
    assert_eq!(version.version, env!("CARGO_PKG_VERSION"));
}

}