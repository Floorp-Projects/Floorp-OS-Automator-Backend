// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use base64::Engine as _;
use base64::engine::general_purpose;
use entity::entity::model;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

/// Inserts a new model row and returns the persisted entity.
pub async fn create_model(
    db: &DatabaseConnection,
    model: model::Model,
) -> Result<model::Model, DbErr> {
    let active_model: model::ActiveModel = model.into();
    let inserted = active_model.insert(db).await?;
    Ok(inserted)
}

/// Retrieves a model by its resource name.
pub async fn get_model(db: &DatabaseConnection, name: &str) -> Result<Option<model::Model>, DbErr> {
    let model = model::Entity::find_by_id(name.to_string()).one(db).await?;
    Ok(model)
}

/// Applies the supplied fields to the existing model if present.
///
/// Returns `Ok(Some(model))` when the update succeeds, `Ok(None)` when the row is
/// missing, or a [`DbErr`] when persistence fails.
pub async fn update_model(
    db: &DatabaseConnection,
    model: model::Model,
) -> Result<Option<model::Model>, DbErr> {
    if let Some(existing) = get_model(db, &model.name).await? {
        let mut active_model: model::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.display_name = Set(model.display_name);
        active_model.description = Set(model.description);
        active_model.provider_name = Set(model.provider_name);
        let updated = active_model.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

/// Lists models with base64 offset pagination identical to the provider module.
pub async fn list_models(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<model::Model>, String), DbErr> {
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
    let mut models = model::Entity::find()
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (models.len() as u64) > limit;
    if has_next {
        models.truncate(limit as usize);
    }

    let next_page_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        general_purpose::STANDARD.encode(next_offset.to_be_bytes())
    } else {
        String::new()
    };

    Ok((models, next_page_token))
}

/// Deletes a model by name. Returns `true` when a row is removed, `false` otherwise.
pub async fn delete_model(db: &DatabaseConnection, name: &str) -> Result<bool, DbErr> {
    if let Some(existing) = model::Entity::find_by_id(name.to_string()).one(db).await? {
        let active_model: model::ActiveModel = existing.into();
        active_model.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::entity::provider;
    use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

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

        // Seed a provider for FK constraints when the schema enforces it.
        let provider_active: provider::ActiveModel = provider::Model {
            name: "providers/base".to_string(),
            display_name: "Base".to_string(),
            api_key: "key".to_string(),
            api_endpoint: "https://example.test".to_string(),
        }
        .into();
        provider_active.insert(&db).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn create_get_and_delete_model() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let model = model::Model {
            name: "models/demo".to_string(),
            display_name: "Demo".to_string(),
            description: Some("desc".to_string()),
            provider_name: "providers/base".to_string(),
        };

        let inserted = create_model(&db, model.clone()).await?;
        assert_eq!(inserted.name, model.name);

        let fetched = get_model(&db, &model.name).await?;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().display_name, "Demo");

        let deleted = delete_model(&db, &model.name).await?;
        assert!(deleted);
        assert!(!delete_model(&db, &model.name).await?);
        Ok(())
    }

    #[tokio::test]
    async fn update_and_list_models() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for idx in 0..4 {
            let model = model::Model {
                name: format!("models/{idx}"),
                display_name: format!("Model {idx}"),
                description: Some(format!("desc{idx}")),
                provider_name: "providers/base".to_string(),
            };
            create_model(&db, model).await?;
        }

        let target = model::Model {
            name: "models/0".to_string(),
            display_name: "Updated".to_string(),
            description: None,
            provider_name: "providers/base".to_string(),
        };
        let updated = update_model(&db, target).await?;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().display_name, "Updated");

        let (first_page, token) = list_models(&db, None, Some(2)).await?;
        assert_eq!(first_page.len(), 2);
        assert!(!token.is_empty());

        let (second_page, _) = list_models(&db, Some(token), Some(5)).await?;
        assert_eq!(second_page.len(), 2);
        Ok(())
    }
}
