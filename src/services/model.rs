// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later
// Sapphillon
//
//
//
//

use std::sync::Arc;

use log::{debug, error, info};
use sea_orm::{DatabaseConnection, DbErr};
use tonic::{Request, Response, Status};

use database::{model as model_db, provider as provider_db};
use sapphillon_core::proto::google::rpc::{Code as RpcCode, Status as RpcStatus};
use sapphillon_core::proto::sapphillon::ai::v1::model_service_server::ModelService;
use sapphillon_core::proto::sapphillon::ai::v1::{
    CreateModelRequest, CreateModelResponse, DeleteModelRequest, DeleteModelResponse,
    GetModelRequest, GetModelResponse, ListModelsRequest, ListModelsResponse, Models,
    UpdateModelRequest, UpdateModelResponse,
};

#[derive(Clone, Debug)]
pub struct MyModelService {
    db: Arc<DatabaseConnection>,
}

impl MyModelService {
    /// Constructs a new model service using the provided database connection.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection used to persist model records.
    ///
    /// # Returns
    ///
    /// Returns a [`MyModelService`] with the connection wrapped in an [`Arc`].
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Sanitizes a model proto before returning it to clients.
    ///
    /// # Arguments
    ///
    /// * `model` - The model proto to sanitize.
    ///
    /// # Returns
    ///
    /// Returns the sanitized model proto. Currently returns the input unchanged.
    fn sanitize_model(model: Models) -> Models {
        model
    }

    /// Builds a success status message for API responses.
    ///
    /// # Arguments
    ///
    /// * `message` - Human-readable text describing the successful operation.
    ///
    /// # Returns
    ///
    /// Returns a [`RpcStatus`] tagged with the `Ok` code and containing the provided message.
    fn ok_status(message: impl Into<String>) -> Option<RpcStatus> {
        Some(RpcStatus {
            code: RpcCode::Ok as i32,
            message: message.into(),
            details: vec![],
        })
    }

    /// Converts SeaORM errors into gRPC [`Status`] instances while logging the context.
    ///
    /// # Arguments
    ///
    /// * `err` - The database error that occurred.
    ///
    /// # Returns
    ///
    /// Returns an internal error status suitable for returning to API callers.
    fn map_db_error(err: DbErr) -> Status {
        error!("Database error occurred while handling model request: {err:?}");
        Status::internal("database operation failed")
    }

    /// Ensures a referenced provider exists before performing model updates.
    ///
    /// # Arguments
    ///
    /// * `provider_name` - The name of the provider to verify.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the provider is present, or a not-found status otherwise.
    async fn ensure_provider_exists(&self, provider_name: &str) -> Result<(), Status> {
        provider_db::get_provider(&self.db, provider_name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "provider '{provider_name}' not found for model operation"
                ))
            })?;
        Ok(())
    }
}

#[tonic::async_trait]
impl ModelService for MyModelService {
    /// Creates a new model after validating the payload and provider reference.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the model to persist.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response with the sanitized model or an error when validation fails.
    async fn create_model(
        &self,
        request: Request<CreateModelRequest>,
    ) -> Result<Response<CreateModelResponse>, Status> {
        let req = request.into_inner();
        let incoming = req
            .model
            .ok_or_else(|| Status::invalid_argument("model field is required"))?;

        if incoming.display_name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "model.display_name must not be empty",
            ));
        }
        if incoming.provider_name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "model.provider_name must not be empty",
            ));
        }

        let has_custom_name = !incoming.name.trim().is_empty();
        let provider_name_requested = incoming.provider_name.trim().to_string();
        info!(
            "create_model request received: has_custom_name={has_custom_name}, provider_name={provider_name}",
            has_custom_name = has_custom_name,
            provider_name = provider_name_requested.as_str()
        );

        let mut model = incoming;
        self.ensure_provider_exists(&provider_name_requested)
            .await?;
        model.provider_name = provider_name_requested.clone();

        let model_name = if has_custom_name {
            model.name.trim().to_string()
        } else {
            format!("models/{}", uuid::Uuid::new_v4())
        };
        model.name = model_name.clone();

        let stored = model_db::create_model(&self.db, model)
            .await
            .map_err(Self::map_db_error)?;

        info!(
            "model created successfully: model_name={model_name}, provider_name={provider_name}",
            model_name = stored.name.as_str(),
            provider_name = stored.provider_name.as_str()
        );

        let response = CreateModelResponse {
            model: Some(Self::sanitize_model(stored)),
            status: Self::ok_status("model created"),
        };

        Ok(Response::new(response))
    }

    /// Retrieves a model by name, returning a sanitized proto when found.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request specifying the model name to fetch.
    ///
    /// # Returns
    ///
    /// Returns the located model or a not-found status when it does not exist.
    async fn get_model(
        &self,
        request: Request<GetModelRequest>,
    ) -> Result<Response<GetModelResponse>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument("name must not be empty"));
        }

        debug!(
            "get_model request received: model_name={model_name}",
            model_name = req.name.as_str()
        );

        let model = model_db::get_model(&self.db, &req.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("model '{}' not found", req.name)))?;

        debug!(
            "model retrieved: model_name={model_name}",
            model_name = req.name.as_str()
        );

        let response = GetModelResponse {
            model: Some(Self::sanitize_model(model)),
            status: Self::ok_status("model retrieved"),
        };

        Ok(Response::new(response))
    }

    /// Lists models with optional pagination parameters.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing pagination options.
    ///
    /// # Returns
    ///
    /// Returns a paginated list of sanitized models plus the next page token when available.
    async fn list_models(
        &self,
        request: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        let req = request.into_inner();

        debug!(
            "list_models request received: page_size={page_size}, page_token='{page_token}'",
            page_size = req.page_size,
            page_token = req.page_token.as_str()
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

        let (models, next_page_token) = model_db::list_models(&self.db, page_token, page_size)
            .await
            .map_err(Self::map_db_error)?;

        let returned_count = models.len();
        let models: Vec<Models> = models.into_iter().map(Self::sanitize_model).collect();

        let response = ListModelsResponse {
            models,
            next_page_token,
            status: Self::ok_status("models listed"),
        };

        debug!("list_models response ready: model_count={returned_count}");

        Ok(Response::new(response))
    }

    /// Deletes a model after confirming it exists.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request identifying which model to remove.
    ///
    /// # Returns
    ///
    /// Returns a gRPC response conveying success or an error if the model is missing or deletion fails.
    async fn delete_model(
        &self,
        request: Request<DeleteModelRequest>,
    ) -> Result<Response<DeleteModelResponse>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument("name must not be empty"));
        }

        info!(
            "delete_model request received: model_name={model_name}",
            model_name = req.name.as_str()
        );

        let existing = model_db::get_model(&self.db, &req.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("model '{}' not found", req.name)))?;

        model_db::delete_model(&self.db, &existing.name)
            .await
            .map_err(Self::map_db_error)?;

        info!(
            "model deleted: model_name={model_name}",
            model_name = existing.name.as_str()
        );

        let response = DeleteModelResponse {
            status: Self::ok_status("model deleted"),
        };

        Ok(Response::new(response))
    }

    /// Applies updates to an existing model using an optional field mask.
    ///
    /// # Arguments
    ///
    /// * `request` - The gRPC request containing the model changes and mask.
    ///
    /// # Returns
    ///
    /// Returns the updated model or an error when validation, provider lookup, or persistence fails.
    async fn update_model(
        &self,
        request: Request<UpdateModelRequest>,
    ) -> Result<Response<UpdateModelResponse>, Status> {
        let req = request.into_inner();
        let incoming = req
            .model
            .ok_or_else(|| Status::invalid_argument("model field is required"))?;

        if incoming.name.trim().is_empty() {
            return Err(Status::invalid_argument("model.name must not be empty"));
        }

        let existing = model_db::get_model(&self.db, &incoming.name)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::not_found(format!("model '{}' not found", incoming.name)))?;

        let mask_paths = req.update_mask.map(|mask| mask.paths).unwrap_or_default();
        let update_all = mask_paths.is_empty();
        let has_update_mask = !update_all;
        info!(
            "update_model request received: model_name={model_name}, has_update_mask={has_update_mask}",
            model_name = incoming.name.as_str(),
            has_update_mask = has_update_mask
        );

        let mut desired = existing.clone();

        if update_all || mask_paths.iter().any(|path| path == "display_name") {
            if incoming.display_name.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "model.display_name must not be empty",
                ));
            }
            desired.display_name = incoming.display_name.clone();
        }

        if update_all || mask_paths.iter().any(|path| path == "description") {
            desired.description = incoming.description.clone();
        }

        if update_all || mask_paths.iter().any(|path| path == "provider_name") {
            if incoming.provider_name.trim().is_empty() {
                return Err(Status::invalid_argument(
                    "model.provider_name must not be empty",
                ));
            }
            let provider_name = incoming.provider_name.trim().to_string();
            self.ensure_provider_exists(&provider_name).await?;
            desired.provider_name = provider_name;
        }

        let updated = model_db::update_model(&self.db, desired)
            .await
            .map_err(Self::map_db_error)?
            .ok_or_else(|| Status::internal("model missing after update"))?;

        info!(
            "model updated successfully: model_name={model_name}",
            model_name = updated.name.as_str()
        );

        let response = UpdateModelResponse {
            model: Some(Self::sanitize_model(updated)),
            status: Self::ok_status("model updated"),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sapphillon_core::proto::google::protobuf::FieldMask;
    use sapphillon_core::proto::sapphillon::ai::v1::Provider;

    /// Creates a model service seeded with the provided providers for testing scenarios.
    ///
    /// # Arguments
    ///
    /// * `providers` - The providers to insert before constructing the service.
    ///
    /// # Returns
    ///
    /// Returns a [`MyModelService`] backed by an in-memory database populated with the given providers.
    async fn setup_service_with_providers(providers: Vec<Provider>) -> MyModelService {
        let conn = sea_orm::Database::connect("sqlite::memory:?cache=shared")
            .await
            .expect("connect sqlite memory db");
        migration::Migrator::up(&conn, None)
            .await
            .expect("apply migrations");

        for provider in providers {
            database::provider::create_provider(&conn, provider)
                .await
                .expect("seed provider");
        }

        MyModelService::new(conn)
    }

    /// Builds a provider model for use in tests.
    ///
    /// # Arguments
    ///
    /// * `name` - The provider resource name.
    /// * `display_name` - The human-readable label for the provider.
    ///
    /// # Returns
    ///
    /// Returns a populated provider model struct suitable for database insertion.
    fn provider(name: &str, display_name: &str) -> Provider {
        Provider {
            name: name.to_string(),
            display_name: display_name.to_string(),
            api_key: "key".to_string(),
            api_endpoint: "https://example.test".to_string(),
        }
    }

    /// Verifies creating and fetching a model persists the expected data.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` once the round-trip assertions complete successfully.
    #[tokio::test]
    async fn create_and_get_model_roundtrip() {
        let service = setup_service_with_providers(vec![provider("providers/test", "Test")]).await;

        let create_req = Request::new(CreateModelRequest {
            model: Some(Models {
                name: String::new(),
                display_name: "Sample Model".to_string(),
                description: Some("desc".to_string()),
                provider_name: "providers/test".to_string(),
                priority: Some(0),
            }),
        });

        let created = service
            .create_model(create_req)
            .await
            .expect("create model")
            .into_inner();

        let model = created.model.expect("model in response");
        assert!(model.name.starts_with("models/"));
        assert_eq!(model.display_name, "Sample Model");
        assert_eq!(model.description.as_deref(), Some("desc"));

        let fetched = service
            .get_model(Request::new(GetModelRequest {
                name: model.name.clone(),
            }))
            .await
            .expect("get model")
            .into_inner();

        let fetched_model = fetched.model.expect("model returned");
        assert_eq!(fetched_model.name, model.name);
        assert_eq!(fetched_model.display_name, "Sample Model");
        assert_eq!(fetched_model.description.as_deref(), Some("desc"));
    }

    /// Ensures updating a model and listing models reflects the latest changes.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after verifying update operations and paginated listing.
    #[tokio::test]
    async fn update_and_list_models() {
        let service = setup_service_with_providers(vec![provider("providers/base", "Base")]).await;

        let create_req = Request::new(CreateModelRequest {
            model: Some(Models {
                name: String::new(),
                display_name: "Initial".to_string(),
                description: Some("detail".to_string()),
                provider_name: "providers/base".to_string(),
                priority: Some(0),
            }),
        });

        let create_resp = service
            .create_model(create_req)
            .await
            .expect("create")
            .into_inner();
        let created = create_resp.model.expect("model");

        let update_req = Request::new(UpdateModelRequest {
            model: Some(Models {
                name: created.name.clone(),
                display_name: "Updated".to_string(),
                description: None,
                provider_name: "providers/base".to_string(),
                priority: Some(0),
            }),
            update_mask: Some(FieldMask {
                paths: vec!["display_name".to_string(), "description".to_string()],
            }),
        });

        let updated = service
            .update_model(update_req)
            .await
            .expect("update")
            .into_inner()
            .model
            .expect("model");
        assert_eq!(updated.display_name, "Updated");
        assert!(updated.description.is_none());

        let list_resp = service
            .list_models(Request::new(ListModelsRequest {
                page_size: 10,
                page_token: String::new(),
            }))
            .await
            .expect("list")
            .into_inner();

        assert_eq!(list_resp.models.len(), 1);
        let listed = &list_resp.models[0];
        assert_eq!(listed.display_name, "Updated");
        assert!(list_resp.next_page_token.is_empty());
    }

    /// Confirms deleting a model removes it from persistence.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` after asserting the model is no longer retrievable.
    #[tokio::test]
    async fn delete_model_removes_record() {
        let service = setup_service_with_providers(vec![provider("providers/main", "Main")]).await;

        let create_req = Request::new(CreateModelRequest {
            model: Some(Models {
                name: String::new(),
                display_name: "ToDelete".to_string(),
                description: None,
                provider_name: "providers/main".to_string(),
                priority: Some(0),
            }),
        });

        let created = service
            .create_model(create_req)
            .await
            .expect("create")
            .into_inner()
            .model
            .expect("model");

        service
            .delete_model(Request::new(DeleteModelRequest {
                name: created.name.clone(),
            }))
            .await
            .expect("delete");

        let err = service
            .get_model(Request::new(GetModelRequest { name: created.name }))
            .await
            .expect_err("get should fail after delete");

        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    /// Ensures model creation fails when the referenced provider is missing.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `()` once the expected not-found error is observed.
    #[tokio::test]
    async fn create_model_requires_existing_provider() {
        let service = setup_service_with_providers(vec![]).await;

        let err = service
            .create_model(Request::new(CreateModelRequest {
                model: Some(Models {
                    name: String::new(),
                    display_name: "Invalid".to_string(),
                    description: None,
                    provider_name: "providers/missing".to_string(),
                    priority: Some(0),
                }),
            }))
            .await
            .expect_err("creation should fail when provider is missing");

        assert_eq!(err.code(), tonic::Code::NotFound);
    }
}
