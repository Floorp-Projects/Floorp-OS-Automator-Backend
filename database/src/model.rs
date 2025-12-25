// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

pub mod model_crud;

pub use model_crud::{
    create_model as create_model_entity, delete_model as delete_model_entity,
    get_model as get_model_entity, list_models as list_models_entity,
    update_model as update_model_entity,
};

use entity::entity::model::Model as EntityModel;
use sapphillon_core::proto::sapphillon::ai::v1::Models as ProtoModel;
use sea_orm::{DatabaseConnection, DbErr};

/// Persists a model described by the proto message and returns the stored proto.
pub async fn create_model(db: &DatabaseConnection, model: ProtoModel) -> Result<ProtoModel, DbErr> {
    let entity: EntityModel = model.into();
    let inserted = model_crud::create_model(db, entity).await?;
    Ok(inserted.into())
}

/// Retrieves a model and returns it as a proto message.
pub async fn get_model(db: &DatabaseConnection, name: &str) -> Result<Option<ProtoModel>, DbErr> {
    let model = model_crud::get_model(db, name).await?;
    Ok(model.map(Into::into))
}

/// Applies the supplied proto fields to the stored model.
///
/// Returns `Ok(Some(proto))` when the model exists, `Ok(None)` otherwise.
pub async fn update_model(
    db: &DatabaseConnection,
    model: ProtoModel,
) -> Result<Option<ProtoModel>, DbErr> {
    let entity: EntityModel = model.into();
    let updated = model_crud::update_model(db, entity).await?;
    Ok(updated.map(Into::into))
}

/// Lists models as proto messages along with the next page token.
pub async fn list_models(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<ProtoModel>, String), DbErr> {
    let (entities, token) = model_crud::list_models(db, next_page_token, page_size).await?;
    let models = entities.into_iter().map(Into::into).collect();
    Ok((models, token))
}

/// Deletes a model by name, returning whether a row was removed.
pub async fn delete_model(db: &DatabaseConnection, name: &str) -> Result<bool, DbErr> {
    model_crud::delete_model(db, name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, Statement,
    };

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

        let sql_provider = r#"
            CREATE TABLE provider (
                name TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                api_key TEXT NOT NULL,
                api_endpoint TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_provider.to_string(),
        ))
        .await?;

        let sql_model = r#"
            CREATE TABLE model (
                name TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                description TEXT,
                provider_name TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_model.to_string(),
        ))
        .await?;

        let provider = entity::entity::provider::Model {
            name: "providers/base".to_string(),
            display_name: "Base".to_string(),
            api_key: "key".to_string(),
            api_endpoint: "https://example.test".to_string(),
        };
        let active: entity::entity::provider::ActiveModel = provider.into();
        active.insert(&db).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn proto_create_and_get_roundtrip() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let model = ProtoModel {
            name: "models/demo".to_string(),
            display_name: "Demo".to_string(),
            description: Some("desc".to_string()),
            provider_name: "providers/base".to_string(),
        };

        let stored = create_model(&db, model).await?;
        assert_eq!(stored.name, "models/demo");

        let fetched = get_model(&db, "models/demo").await?;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().display_name, "Demo");
        Ok(())
    }

    #[tokio::test]
    async fn proto_update_delete_and_list() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for idx in 0..3 {
            let model = ProtoModel {
                name: format!("models/{idx}"),
                display_name: format!("Model {idx}"),
                description: Some(format!("desc{idx}")),
                provider_name: "providers/base".to_string(),
            };
            create_model(&db, model).await?;
        }

        let update_proto = ProtoModel {
            name: "models/1".to_string(),
            display_name: "Updated".to_string(),
            description: None,
            provider_name: "providers/base".to_string(),
        };
        let updated = update_model(&db, update_proto).await?;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().display_name, "Updated");

        let deleted = delete_model(&db, "models/2").await?;
        assert!(deleted);

        let (models, token) = list_models(&db, None, Some(10)).await?;
        assert_eq!(models.len(), 2);
        assert!(token.is_empty());
        Ok(())
    }
}
