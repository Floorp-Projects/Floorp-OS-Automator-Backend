// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

pub mod provider_crud;

pub use provider_crud::{
    create_provider as create_provider_entity, delete_provider as delete_provider_entity,
    get_provider as get_provider_entity, list_providers as list_providers_entity,
    update_provider as update_provider_entity,
};

use entity::entity::provider::Model as EntityProvider;
use sapphillon_core::proto::sapphillon::ai::v1::Provider as ProtoProvider;
use sea_orm::{DatabaseConnection, DbErr};

/// Persists a provider described by its proto representation and returns the stored proto.
pub async fn create_provider(
    db: &DatabaseConnection,
    provider: ProtoProvider,
) -> Result<ProtoProvider, DbErr> {
    let entity: EntityProvider = provider.into();
    let inserted = provider_crud::create_provider(db, entity).await?;
    Ok(inserted.into())
}

/// Retrieves a provider by name and returns it as a proto message.
pub async fn get_provider(
    db: &DatabaseConnection,
    name: &str,
) -> Result<Option<ProtoProvider>, DbErr> {
    let provider = provider_crud::get_provider(db, name).await?;
    Ok(provider.map(Into::into))
}

/// Applies updates from a proto message to the stored provider.
///
/// Returns `Ok(Some(proto))` when the provider exists, `Ok(None)` otherwise.
pub async fn update_provider(
    db: &DatabaseConnection,
    provider: ProtoProvider,
) -> Result<Option<ProtoProvider>, DbErr> {
    let entity: EntityProvider = provider.into();
    let updated = provider_crud::update_provider(db, entity).await?;
    Ok(updated.map(Into::into))
}

/// Lists providers as proto messages alongside the next page token.
pub async fn list_providers(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<ProtoProvider>, String), DbErr> {
    let (entities, token) = provider_crud::list_providers(db, next_page_token, page_size).await?;
    let providers = entities.into_iter().map(Into::into).collect();
    Ok((providers, token))
}

/// Deletes a provider by name, returning whether a row was removed.
pub async fn delete_provider(db: &DatabaseConnection, name: &str) -> Result<bool, DbErr> {
    provider_crud::delete_provider(db, name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;
        let sql = r#"
            CREATE TABLE provider (
                name TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                api_key TEXT NOT NULL,
                api_endpoint TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await?;
        Ok(db)
    }

    #[tokio::test]
    async fn proto_create_and_get_roundtrip() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let provider = ProtoProvider {
            name: "providers/demo".to_string(),
            display_name: "Demo".to_string(),
            api_key: "secret".to_string(),
            api_endpoint: "https://example.test".to_string(),
        };

        let stored = create_provider(&db, provider).await?;
        assert_eq!(stored.name, "providers/demo");

        let fetched = get_provider(&db, "providers/demo").await?;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().display_name, "Demo");
        Ok(())
    }

    #[tokio::test]
    async fn proto_update_and_list() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for idx in 0..3 {
            let provider = ProtoProvider {
                name: format!("providers/{idx}"),
                display_name: format!("Provider {idx}"),
                api_key: format!("key{idx}"),
                api_endpoint: format!("https://{idx}.test"),
            };
            create_provider(&db, provider).await?;
        }

        let update_proto = ProtoProvider {
            name: "providers/1".to_string(),
            display_name: "Updated".to_string(),
            api_key: "new".to_string(),
            api_endpoint: "https://updated.test".to_string(),
        };
        let updated = update_provider(&db, update_proto).await?;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().display_name, "Updated");

        let (providers, token) = list_providers(&db, None, Some(10)).await?;
        assert_eq!(providers.len(), 3);
        assert!(token.is_empty());
        Ok(())
    }
}
