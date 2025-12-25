// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use base64::Engine as _;
use base64::engine::general_purpose;
use entity::entity::provider;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

/// Inserts a new provider row into the database and returns the persisted model.
pub async fn create_provider(
    db: &DatabaseConnection,
    provider: provider::Model,
) -> Result<provider::Model, DbErr> {
    let active_model: provider::ActiveModel = provider.into();
    let inserted = active_model.insert(db).await?;
    Ok(inserted)
}

/// Fetches a provider by its unique resource name.
pub async fn get_provider(
    db: &DatabaseConnection,
    name: &str,
) -> Result<Option<provider::Model>, DbErr> {
    let provider = provider::Entity::find_by_id(name.to_string())
        .one(db)
        .await?;
    Ok(provider)
}

/// Applies the supplied provider fields to the stored record if it exists.
///
/// Returns `Ok(Some(model))` when the record is updated, `Ok(None)` when the
/// provider is missing, or a [`DbErr`] when the update fails.
pub async fn update_provider(
    db: &DatabaseConnection,
    provider: provider::Model,
) -> Result<Option<provider::Model>, DbErr> {
    if let Some(existing) = get_provider(db, &provider.name).await? {
        let mut active_model: provider::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.display_name = Set(provider.display_name);
        active_model.api_key = Set(provider.api_key);
        active_model.api_endpoint = Set(provider.api_endpoint);
        let updated = active_model.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

/// Returns providers using an opaque base64-encoded offset pagination scheme.
pub async fn list_providers(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<provider::Model>, String), DbErr> {
    let offset: u64 = match next_page_token {
        Some(token) => match general_purpose::STANDARD.decode(token) {
            Ok(bytes) if bytes.len() == 8 => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                u64::from_be_bytes(arr)
            }
            _ => 0u64,
        },
        None => 0u64,
    };

    let limit = match page_size {
        Some(0) | None => 100u64,
        Some(sz) => sz as u64,
    };

    let query_limit = limit.saturating_add(1);
    let mut providers = provider::Entity::find()
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (providers.len() as u64) > limit;
    if has_next {
        providers.truncate(limit as usize);
    }

    let next_page_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        general_purpose::STANDARD.encode(next_offset.to_be_bytes())
    } else {
        String::new()
    };

    Ok((providers, next_page_token))
}

/// Deletes a provider by name. Returns `true` when a row is deleted, `false` otherwise.
pub async fn delete_provider(db: &DatabaseConnection, name: &str) -> Result<bool, DbErr> {
    if let Some(existing) = provider::Entity::find_by_id(name.to_string())
        .one(db)
        .await?
    {
        let active_model: provider::ActiveModel = existing.into();
        active_model.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

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
    async fn create_and_get_roundtrip() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let model = provider::Model {
            name: "providers/demo".to_string(),
            display_name: "Demo".to_string(),
            api_key: "secret".to_string(),
            api_endpoint: "https://example.test".to_string(),
        };

        let inserted = create_provider(&db, model.clone()).await?;
        assert_eq!(inserted.name, model.name);

        let fetched = get_provider(&db, &model.name).await?;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().display_name, "Demo");
        Ok(())
    }

    #[tokio::test]
    async fn update_and_delete_provider() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let model = provider::Model {
            name: "providers/source".to_string(),
            display_name: "Source".to_string(),
            api_key: "key".to_string(),
            api_endpoint: "https://a.test".to_string(),
        };
        create_provider(&db, model.clone()).await?;

        let updated = provider::Model {
            name: model.name.clone(),
            display_name: "Updated".to_string(),
            api_key: "changed".to_string(),
            api_endpoint: "https://b.test".to_string(),
        };
        let updated = update_provider(&db, updated).await?;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().display_name, "Updated");

        let deleted = delete_provider(&db, &model.name).await?;
        assert!(deleted);
        assert!(!delete_provider(&db, &model.name).await?);
        Ok(())
    }

    #[tokio::test]
    async fn list_providers_paginates() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for idx in 0..5 {
            let model = provider::Model {
                name: format!("providers/{idx}"),
                display_name: format!("P{idx}"),
                api_key: format!("key{idx}"),
                api_endpoint: format!("https://{idx}.test"),
            };
            create_provider(&db, model).await?;
        }

        let (page_one, token) = list_providers(&db, None, Some(2)).await?;
        assert_eq!(page_one.len(), 2);
        assert!(!token.is_empty());

        let (page_two, _) = list_providers(&db, Some(token), Some(3)).await?;
        assert_eq!(page_two.len(), 3);
        Ok(())
    }
}
