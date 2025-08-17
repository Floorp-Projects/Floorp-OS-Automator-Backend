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

use log::info;
use tonic::{Request, Response, Status};

// Import the generated protobuf types
use sapphillon_core::proto::sapphillon::v1::version_service_server::VersionService;
use sapphillon_core::proto::sapphillon::v1::{GetVersionRequest, GetVersionResponse, Version};

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

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_get_version() {
        let service = MyVersionService;
        let request = Request::new(GetVersionRequest {});
        let response = service.get_version(request).await.unwrap().into_inner();
        assert!(response.version.is_some());
        let version = response.version.unwrap();
        // The version string should match the crate version
        assert_eq!(version.version, env!("CARGO_PKG_VERSION"));
    }
}
