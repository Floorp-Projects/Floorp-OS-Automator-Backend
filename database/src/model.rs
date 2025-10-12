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

use base64::Engine as _;
use base64::engine::general_purpose;
use entity::entity::model;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

/// Inserts a new model row into the database.
///
/// # Arguments
///
/// * `db` - The database connection to use for the insert.
/// * `model` - The model entity to persist.
///
/// # Returns
///
/// Returns `Ok(())` when the row is created successfully, or a [`DbErr`] on failure.
pub async fn create_model(db: &DatabaseConnection, model: model::Model) -> Result<(), DbErr> {
    let active_model: model::ActiveModel = model.into();
    // Use insert to guarantee creation of a new row.
    active_model.insert(db).await?;
    Ok(())
}

/// Fetches a model by its resource name.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `name` - The model resource identifier.
///
/// # Returns
///
/// Returns `Ok(Some(model))` when found, `Ok(None)` when missing, or a [`DbErr`] on query failure.
pub async fn get_model(db: &DatabaseConnection, name: &str) -> Result<Option<model::Model>, DbErr> {
    let m = model::Entity::find_by_id(name.to_string()).one(db).await?;
    Ok(m)
}

/// Updates an existing model record if it exists.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `model` - The model data whose fields should be applied.
///
/// # Returns
///
/// Returns `Ok(())` after the update completes, even if the model was not present.
pub async fn update_model(db: &DatabaseConnection, model: model::Model) -> Result<(), DbErr> {
    let existing = get_model(db, &model.name).await?;
    if let Some(existing) = existing {
        let mut active_model: model::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.display_name = Set(model.display_name);
        active_model.description = Set(model.description);
        active_model.provider_name = Set(model.provider_name);
        active_model.update(db).await?;
    }
    Ok(())
}

/// Lists models using base64-encoded offset pagination.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `next_page_token` - An optional opaque cursor indicating the starting offset.
/// * `page_size` - An optional limit on the number of models to fetch.
///
/// # Returns
///
/// Returns a tuple containing the fetched models and the next page token (empty if none).
pub async fn list_models(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<model::Model>, String), DbErr> {
    // Pagination by base64-encoded big-endian u64 offset (same as provider.rs)
    let offset: u64 = match next_page_token {
        Some(token) => match general_purpose::STANDARD.decode(token) {
            Ok(bytes) => {
                if bytes.len() == 8 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes);
                    u64::from_be_bytes(arr)
                } else {
                    0u64
                }
            }
            Err(_) => 0u64,
        },
        None => 0u64,
    };

    let limit = match page_size {
        Some(0) | None => 100u64,
        Some(sz) => sz as u64,
    };

    let query_limit = limit.saturating_add(1);
    let mut items = model::Entity::find()
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (items.len() as u64) > limit;
    if has_next {
        items.truncate(limit as usize);
    }

    let next_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        let bytes = next_offset.to_be_bytes();
        general_purpose::STANDARD.encode(bytes)
    } else {
        String::new()
    };

    Ok((items, next_token))
}

/// Deletes a model if one exists with the supplied name.
///
/// # Arguments
///
/// * `db` - The database connection to execute against.
/// * `name` - The model identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` regardless of whether the record existed, or a [`DbErr`] on failure.
pub async fn delete_model(db: &DatabaseConnection, name: &str) -> Result<(), DbErr> {
    let found = model::Entity::find_by_id(name.to_string()).one(db).await?;
    if let Some(found) = found {
        let active_model: model::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::provider as entity_provider;
    use sea_orm::{
        ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement,
    };

    /// Sets up an in-memory database schema used by model tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a connected [`DatabaseConnection`] with the required tables created.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        // In-memory SQLite for tests
        let db = Database::connect("sqlite::memory:").await?;

        // Create provider table (model references provider)
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

        // Create model table
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

        Ok(db)
    }

    /// Ensures models can be created and fetched using the helper functions.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the inserted model is retrieved and validated.
    #[tokio::test]
    async fn test_create_model() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert a provider first to satisfy FK semantics if needed by application logic
        let prov = entity_provider::Model {
            name: "p".to_string(),
            display_name: "P".to_string(),
            api_key: "k".to_string(),
            api_endpoint: "https://p.test".to_string(),
        };
        crate::provider::create_provider(&db, prov).await?;

        let m = model::Model {
            name: "m1".to_string(),
            display_name: "Model One".to_string(),
            description: Some("desc".to_string()),
            provider_name: "p".to_string(),
        };
        create_model(&db, m).await?;

        let found = model::Entity::find_by_id("m1".to_string()).one(&db).await?;
        assert!(found.is_some(), "Inserted model should be found");
        let found = found.unwrap();
        assert_eq!(found.name, "m1");
        assert_eq!(found.display_name, "Model One");
        assert_eq!(found.description.as_deref(), Some("desc"));
        assert_eq!(found.provider_name, "p");

        Ok(())
    }

    /// Confirms fetching a missing model returns `None`.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying a non-existent model yields `None`.
    #[tokio::test]
    async fn test_get_model_not_found() -> Result<(), DbErr> {
        let db = setup_db().await?;
        let found = get_model(&db, "nope").await?;
        assert!(found.is_none(), "should return None for missing model");
        Ok(())
    }

    /// Validates pagination logic over the model table.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after iterating through multiple pages of results.
    #[tokio::test]
    async fn test_list_models_pagination() -> Result<(), DbErr> {
        use std::collections::HashSet;

        let db = setup_db().await?;

        // Insert provider
        let prov = entity_provider::Model {
            name: "prov".to_string(),
            display_name: "Prov".to_string(),
            api_key: "k".to_string(),
            api_endpoint: "https://prov.test".to_string(),
        };
        crate::provider::create_provider(&db, prov).await?;

        for i in 1..=5 {
            let m = model::Model {
                name: format!("m{i}"),
                display_name: format!("Model {i}"),
                description: Some(format!("d{i}")),
                provider_name: "prov".to_string(),
            };
            create_model(&db, m).await?;
        }

        let mut token: Option<String> = None;
        let mut all = HashSet::new();
        loop {
            let (items, next) = list_models(&db, token.clone(), Some(2)).await?;
            for it in items.iter() {
                all.insert(it.name.clone());
            }
            if next.is_empty() {
                break;
            }
            token = Some(next);
        }

        assert_eq!(all.len(), 5, "should collect all inserted models");
        Ok(())
    }

    /// Ensures invalid pagination tokens are treated as offset zero.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once items are returned and a next token is emitted.
    #[tokio::test]
    async fn test_list_models_invalid_token() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert provider & some models
        let prov = entity_provider::Model {
            name: "pv".to_string(),
            display_name: "PV".to_string(),
            api_key: "k".to_string(),
            api_endpoint: "https://pv.test".to_string(),
        };
        crate::provider::create_provider(&db, prov).await?;

        for i in 1..=3 {
            let m = model::Model {
                name: format!("b{i}"),
                display_name: format!("B{i}"),
                description: None,
                provider_name: "pv".to_string(),
            };
            create_model(&db, m).await?;
        }

        let (items, next) = list_models(&db, Some("not-a-token".to_string()), Some(2)).await?;
        assert!(
            !items.is_empty(),
            "should return items with invalid token treated as offset 0"
        );
        assert!(
            !next.is_empty(),
            "should return next token when more pages exist"
        );
        Ok(())
    }

    /// Verifies pagination tokens round-trip correctly through encoding and decoding.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the decoded offset matches expectations and pages are exhausted.
    #[tokio::test]
    async fn test_page_token_roundtrip() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let prov = entity_provider::Model {
            name: "pr".to_string(),
            display_name: "PR".to_string(),
            api_key: "k".to_string(),
            api_endpoint: "https://pr.test".to_string(),
        };
        crate::provider::create_provider(&db, prov).await?;

        for i in 1..=4 {
            let m = model::Model {
                name: format!("pt{i}"),
                display_name: format!("PT {i}"),
                description: None,
                provider_name: "pr".to_string(),
            };
            create_model(&db, m).await?;
        }

        let (first, token) = list_models(&db, None, Some(2)).await?;
        assert_eq!(first.len(), 2);
        assert!(!token.is_empty());

        let decoded = general_purpose::STANDARD
            .decode(token.clone())
            .expect("decode");
        assert_eq!(decoded.len(), 8);
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&decoded);
        let offset = u64::from_be_bytes(arr);
        assert_eq!(offset, 2u64);

        let (second, final_token) = list_models(&db, Some(token), Some(2)).await?;
        assert_eq!(second.len(), 2);
        assert!(final_token.is_empty());

        Ok(())
    }

    /// Ensures updates overwrite fields and deletions remove models as expected.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying update and delete operations.
    #[tokio::test]
    async fn test_update_and_delete_model() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let prov = entity_provider::Model {
            name: "pp".to_string(),
            display_name: "PP".to_string(),
            api_key: "k".to_string(),
            api_endpoint: "https://pp.test".to_string(),
        };
        crate::provider::create_provider(&db, prov).await?;

        let initial = model::Model {
            name: "u1".to_string(),
            display_name: "Before".to_string(),
            description: Some("x".to_string()),
            provider_name: "pp".to_string(),
        };
        create_model(&db, initial).await?;

        let updated = model::Model {
            name: "u1".to_string(),
            display_name: "After".to_string(),
            description: None,
            provider_name: "pp".to_string(),
        };
        update_model(&db, updated).await?;

        let found = get_model(&db, "u1").await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.display_name, "After");
        assert!(found.description.is_none());

        delete_model(&db, "u1").await?;
        let found = get_model(&db, "u1").await?;
        assert!(found.is_none());

        // Deleting again should not error
        delete_model(&db, "u1").await?;

        Ok(())
    }
}
