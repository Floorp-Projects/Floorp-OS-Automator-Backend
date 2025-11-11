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

use std::sync::Arc;

use log::{debug, error, info};
use sea_orm::{DatabaseConnection, DbErr};
use tonic::{Request, Response, Status};

use database::provider as provider_db;
use sapphillon_core::proto::google::rpc::{Code as RpcCode, Status as RpcStatus};
use sapphillon_core::proto::sapphillon::ai::v1::provider_service_server::ProviderService;
use sapphillon_core::proto::sapphillon::ai::v1::{
    CreateProviderRequest, CreateProviderResponse, DeleteProviderRequest, DeleteProviderResponse,
    GetProviderRequest, GetProviderResponse, ListProvidersRequest, ListProvidersResponse, Provider,
    UpdateProviderRequest, UpdateProviderResponse,
};

#[derive(Clone, Debug)]
pub struct MyProviderService {
    db: Arc<DatabaseConnection>,
}

impl MyProviderService {
    /// Constructs a new provider service backed by the supplied database connection.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection used to persist provider records.
    ///
    /// # Returns
    ///
    /// Returns a [`MyProviderService`] wrapping the given connection inside an [`Arc`].
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Clears sensitive fields from a provider proto before returning it to clients.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider proto to sanitize.
    ///
    /// # Returns
    ///
    /// Returns a provider proto with the API key field cleared.
    fn sanitize_provider(mut provider: Provider) -> Provider {
        provider.api_key.clear();
        provider
    }

    /// Creates a success status wrapper to embed in API responses.
    ///
    /// # Arguments
    ///
    /// * `message` - Human-readable detail describing the successful operation.
    ///
    /// # Returns
    ///
    /// Returns a [`RpcStatus`] marked as `Ok` with the supplied message.
    fn ok_status(message: impl Into<String>) -> Option<RpcStatus> {
        Some(RpcStatus {
            code: RpcCode::Ok as i32,
            message: message.into(),
            details: vec![],
        })
    }

    /// Converts a SeaORM error into a gRPC [`Status`] while logging diagnostic details.
    ///
    /// # Arguments
    ///
    /// * `err` - The database error encountered during an operation.
    ///
    /// # Returns
    ///
    /// Returns an internal gRPC status suitable for surfacing to clients.
    fn map_db_error(err: DbErr) -> Status {
        error!("Database error occurred while handling provider request: {err:?}");
        Status::internal("database operation failed")
    }
}

#[tonic::async_trait]
impl ProviderService for MyProviderService {
    /// Handles provider creation requests by validating input and persisting the record.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the provider definition to create.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response with the sanitized provider or an error if validation or persistence fails.
    async fn create_provider(
        &self,
        request: Request<CreateProviderRequest>,
    ) -> Result<Response<CreateProviderResponse>, Status> {
        let req = request.into_inner();
        let incoming = req
            .provider
            .ok_or_else(|| Status::invalid_argument("provider field is required"))?;

        if incoming.display_name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "provider.display_name must not be empty",
            ));
        }
        if incoming.api_key.trim().is_empty() {
            return Err(Status::invalid_argument(
                "provider.api_key must not be empty",
            ));
        }
        if incoming.api_endpoint.trim().is_empty() {
            return Err(Status::invalid_argument(
                "provider.api_endpoint must not be empty",
            ));
        }

        let has_custom_name = !incoming.name.trim().is_empty();
        let has_api_key = !incoming.api_key.trim().is_empty();
        let api_endpoint = incoming.api_endpoint.trim().to_string();
        info!(
            "create_provider request received: has_custom_name={has_custom_name}, has_api_key={has_api_key}, api_endpoint={api_endpoint}"
        );

        let provider_name = if incoming.name.trim().is_empty() {
            format!("providers/{}", uuid::Uuid::new_v4())
        } else {
            incoming.name.clone()
        };

        let stored = provider_db::create_provider(
            &self.db,
            Provider {
                name: provider_name,
                display_name: incoming.display_name,
                api_key: incoming.api_key,
                api_endpoint: incoming.api_endpoint,
            },
        )
        .await
        .map_err(Self::map_db_error)?;

        info!(
            "provider created successfully: provider_name={provider_name}",
            provider_name = stored.name.as_str()
        );

        let response = CreateProviderResponse {
            provider: Some(Self::sanitize_provider(stored)),
            status: Self::ok_status("provider created"),
        };

        Ok(Response::new(response))
    }

    /// Fetches a provider by name and returns a sanitized representation.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request specifying the provider name to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response containing the provider when found, or a not-found error otherwise.
    async fn get_provider(
        &self,
        request: Request<GetProviderRequest>,
    ) -> Result<Response<GetProviderResponse>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument("name must not be empty"));
        }

        debug!(
            "get_provider request received: provider_name={provider_name}",
            provider_name = req.name.as_str()
        );

        let provider = provider_db::get_provider(&self.db, &req.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("provider '{}' not found", req.name)))?;

        debug!(
            "provider retrieved: provider_name={provider_name}",
            provider_name = req.name.as_str()
        );

        let response = GetProviderResponse {
            provider: Some(Self::sanitize_provider(provider)),
            status: Self::ok_status("provider retrieved"),
        };

        Ok(Response::new(response))
    }

    /// Lists providers with optional pagination support.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing pagination inputs.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response with the sanitized providers list and the next page token when available.
    async fn list_providers(
        &self,
        request: Request<ListProvidersRequest>,
    ) -> Result<Response<ListProvidersResponse>, Status> {
        let req = request.into_inner();

        debug!(
            "list_providers request received: page_size={}, page_token='{}'",
            req.page_size,
            req.page_token.as_str()
        );

        let page_size = if req.page_size <= 0 {
            None
        } else {
            Some(req.page_size as u32)
        };
        let page_token = if req.page_token.trim().is_empty() {
            None
        } else {
            Some(req.page_token)
        };

        let (providers, next_page_token) =
            provider_db::list_providers(&self.db, page_token, page_size)
                .await
                .map_err(Self::map_db_error)?;

        let returned_count = providers.len();
        let providers = providers.into_iter().map(Self::sanitize_provider).collect();

        let response = ListProvidersResponse {
            providers,
            next_page_token,
            status: Self::ok_status("providers listed"),
        };

        debug!("list_providers response ready: provider_count={returned_count}");

        Ok(Response::new(response))
    }

    /// Deletes a provider after confirming it exists.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request identifying which provider to remove.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response containing a success status, or an error if the provider is missing or deletion fails.
    async fn delete_provider(
        &self,
        request: Request<DeleteProviderRequest>,
    ) -> Result<Response<DeleteProviderResponse>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument("name must not be empty"));
        }

        info!(
            "delete_provider request received: provider_name={provider_name}",
            provider_name = req.name.as_str()
        );

        let existing = provider_db::get_provider(&self.db, &req.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("provider '{}' not found", req.name)))?;

        provider_db::delete_provider(&self.db, &existing.name)
            .await
            .map_err(Self::map_db_error)?;

        info!(
            "provider deleted: provider_name={provider_name}",
            provider_name = existing.name.as_str()
        );

        let response = DeleteProviderResponse {
            status: Self::ok_status("provider deleted"),
        };

        Ok(Response::new(response))
    }

    /// Applies updates to an existing provider record based on the supplied field mask.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the provider data and update mask.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response with the updated provider or an error if validation or persistence fails.
    async fn update_provider(
        &self,
        request: Request<UpdateProviderRequest>,
    ) -> Result<Response<UpdateProviderResponse>, Status> {
        let req = request.into_inner();
        let incoming = req
            .provider
            .ok_or_else(|| Status::invalid_argument("provider field is required"))?;

        if incoming.name.trim().is_empty() {
            return Err(Status::invalid_argument("provider.name must not be empty"));
        }

        let mut existing = provider_db::get_provider(&self.db, &incoming.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("provider '{}' not found", incoming.name)))?;

        let mask_paths = req.update_mask.map(|mask| mask.paths).unwrap_or_default();
        let update_all = mask_paths.is_empty();
        let has_update_mask = !mask_paths.is_empty();
        info!(
            "update_provider request received: provider_name={provider_name}, has_update_mask={has_update_mask}",
            provider_name = incoming.name.as_str(),
            has_update_mask = has_update_mask
        );

        if update_all || mask_paths.iter().any(|path| path == "display_name") {
            if incoming.display_name.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "provider.display_name must not be empty",
                ));
            }
            existing.display_name = incoming.display_name.clone();
        }
        if update_all || mask_paths.iter().any(|path| path == "api_key") {
            if incoming.api_key.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "provider.api_key must not be empty",
                ));
            }
            existing.api_key = incoming.api_key.clone();
        }
        if update_all || mask_paths.iter().any(|path| path == "api_endpoint") {
            if incoming.api_endpoint.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "provider.api_endpoint must not be empty",
                ));
            }
            existing.api_endpoint = incoming.api_endpoint.clone();
        }

        let updated = provider_db::update_provider(&self.db, existing)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::internal("provider missing after update"))?;

        info!(
            "provider updated successfully: provider_name={provider_name}",
            provider_name = updated.name.as_str()
        );

        let response = UpdateProviderResponse {
            provider: Some(Self::sanitize_provider(updated)),
            status: Self::ok_status("provider updated"),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sapphillon_core::proto::google::protobuf::FieldMask;

    /// Creates a provider service backed by an in-memory SQLite database for testing.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`MyProviderService`] connected to a temporary database.
    async fn setup_service() -> MyProviderService {
        let conn = sea_orm::Database::connect("sqlite::memory:?cache=shared")
            .await
            .expect("connect sqlite memory db");
        migration::Migrator::up(&conn, None)
            .await
            .expect("apply migrations");

        MyProviderService::new(conn)
    }

    /// Tests creating, retrieving, and sanitizing a provider end-to-end.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` once the round-trip assertions succeed.
    #[tokio::test]
    async fn create_and_get_provider_roundtrip() {
        let service = setup_service().await;

        let create_req = Request::new(CreateProviderRequest {
            provider: Some(Provider {
                name: String::new(),
                display_name: "Test Provider".to_string(),
                api_key: "super-secret".to_string(),
                api_endpoint: "https://example.test".to_string(),
            }),
        });

        let created = service
            .create_provider(create_req)
            .await
            .expect("create provider")
            .into_inner();

        let provider = created.provider.expect("provider in response");
        assert!(provider.name.starts_with("providers/"));
        assert!(provider.api_key.is_empty(), "api_key should be sanitized");

        let fetched = service
            .get_provider(Request::new(GetProviderRequest {
                name: provider.name.clone(),
                status: None,
            }))
            .await
            .expect("get provider")
            .into_inner();

        let fetched_provider = fetched.provider.expect("provider returned");
        assert_eq!(fetched_provider.name, provider.name);
        assert!(fetched_provider.api_key.is_empty());
        assert_eq!(fetched_provider.display_name, "Test Provider");
    }

    /// Validates update and list operations behave as expected for providers.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after the provider is updated, listed, and deleted successfully.
    #[tokio::test]
    async fn update_and_list_providers() {
        let service = setup_service().await;

        let create_req = Request::new(CreateProviderRequest {
            provider: Some(Provider {
                name: String::new(),
                display_name: "Initial".to_string(),
                api_key: "secret".to_string(),
                api_endpoint: "https://initial.test".to_string(),
            }),
        });

        let create_resp = service
            .create_provider(create_req)
            .await
            .expect("create")
            .into_inner();
        let created = create_resp.provider.expect("provider");

        let update_req = Request::new(UpdateProviderRequest {
            provider: Some(Provider {
                name: created.name.clone(),
                display_name: "Updated".to_string(),
                api_key: "secret".to_string(),
                api_endpoint: "https://updated.test".to_string(),
            }),
            update_mask: Some(FieldMask {
                paths: vec!["display_name".to_string(), "api_endpoint".to_string()],
            }),
        });

        let update_resp = service
            .update_provider(update_req)
            .await
            .expect("update")
            .into_inner();
        let updated = update_resp.provider.expect("provider");
        assert_eq!(updated.display_name, "Updated");
        assert_eq!(updated.api_endpoint, "https://updated.test");

        let list_resp = service
            .list_providers(Request::new(ListProvidersRequest {
                page_size: 10,
                page_token: String::new(),
            }))
            .await
            .expect("list")
            .into_inner();

        assert_eq!(list_resp.providers.len(), 1);
        let listed = &list_resp.providers[0];
        assert_eq!(listed.display_name, "Updated");
        assert!(listed.api_key.is_empty());
        assert!(list_resp.next_page_token.is_empty());

        service
            .delete_provider(Request::new(DeleteProviderRequest {
                name: created.name.clone(),
            }))
            .await
            .expect("delete");

        let err = service
            .get_provider(Request::new(GetProviderRequest {
                name: created.name.clone(),
                status: None,
            }))
            .await
            .expect_err("get should fail after delete");

        assert_eq!(err.code(), tonic::Code::NotFound);
    }
}
