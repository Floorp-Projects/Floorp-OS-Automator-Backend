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
use entity::entity::provider;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

/// Inserts a new provider record into the database.
///
/// # Arguments
///
/// * `db` - The database connection on which to execute the insert.
/// * `provider` - The provider entity to persist.
///
/// # Returns
///
/// Returns `Ok(())` when the insert succeeds, or a [`DbErr`] otherwise.
pub async fn create_provider(
    db: &DatabaseConnection,
    provider: provider::Model,
) -> Result<(), DbErr> {
    let active_model: provider::ActiveModel = provider.into();
    // Use insert to ensure a new record is created. save() can try to perform an update
    // when the primary key is already set on the ActiveModel, which causes RecordNotFound
    // if the row doesn't exist yet.
    active_model.insert(db).await?;
    Ok(())
}

/// Retrieves a provider by its unique name.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `name` - The provider identifier to search for.
///
/// # Returns
///
/// Returns `Ok(Some(provider))` when found, `Ok(None)` when missing, or a [`DbErr`] if the query fails.
pub async fn get_provider(
    db: &DatabaseConnection,
    name: &str,
) -> Result<Option<provider::Model>, DbErr> {
    let provider = provider::Entity::find_by_id(name.to_string())
        .one(db)
        .await?;
    Ok(provider)
}

/// Updates the mutable fields of an existing provider.
///
/// # Arguments
///
/// * `db` - The database connection to use for persistence.
/// * `provider` - The provider data containing the new field values.
///
/// # Returns
///
/// Returns `Ok(())` once the update completes, even if the provider does not exist.
pub async fn update_provider(
    db: &DatabaseConnection,
    provider: provider::Model,
) -> Result<(), DbErr> {
    // Fetch existing record so we can mark fields as changed explicitly.
    // Converting the incoming model directly into an ActiveModel may leave
    // some fields as `NotSet` / `Unchanged` depending on conversions,
    // so we start from the existing row and set the fields we want to update.
    let existing = get_provider(db, &provider.name).await?;

    if let Some(existing) = existing {
        let mut active_model: provider::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.display_name = Set(provider.display_name);
        active_model.api_key = Set(provider.api_key);
        active_model.api_endpoint = Set(provider.api_endpoint);
        active_model.update(db).await?;
    }
    Ok(())
}

/// Lists providers using base64-encoded offset pagination.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `next_page_token` - An optional cursor specifying the next offset.
/// * `page_size` - An optional limit on the number of results to fetch.
///
/// # Returns
///
/// Returns a tuple of provider models and the next page token (empty when no more results).
pub async fn list_providers(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<provider::Model>, String), DbErr> {
    // Return a tuple of (list, next_page_token)
    // Decode next_page_token as base64 u64 offset. If missing or invalid, start at 0.
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

    // Determine limit, default to 100 if not provided or zero
    let limit = match page_size {
        Some(0) | None => 100u64,
        Some(sz) => sz as u64,
    };

    // Query with offset/limit. Fetch one extra row to detect whether a next page exists.
    let query_limit = limit.saturating_add(1);
    let mut providers = provider::Entity::find()
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (providers.len() as u64) > limit;

    // Trim to requested page size
    if has_next {
        providers.truncate(limit as usize);
    }

    let next_page_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        let bytes = next_offset.to_be_bytes();
        general_purpose::STANDARD.encode(bytes)
    } else {
        String::new()
    };

    Ok((providers, next_page_token))
}

/// Deletes a provider with the given name if it exists.
///
/// # Arguments
///
/// * `db` - The database connection used for deletion.
/// * `name` - The provider identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` regardless of whether the provider was present, or a [`DbErr`] on errors.
pub async fn delete_provider(db: &DatabaseConnection, name: &str) -> Result<(), DbErr> {
    let provider = provider::Entity::find_by_id(name.to_string())
        .one(db)
        .await?;
    if let Some(provider) = provider {
        let active_model: provider::ActiveModel = provider.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement,
    };

    /// Prepares an in-memory SQLite database with the provider schema for tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a connected [`DatabaseConnection`] ready for CRUD testing.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        // Use an in-memory SQLite database for testing
        let db = Database::connect("sqlite::memory:").await?;

        // Create the `provider` table matching the entity definition
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

    /// Verifies providers can be created and retrieved from storage.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the inserted provider matches expectations.
    #[tokio::test]
    async fn test_create_provider() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Prepare a provider model and call the function under test
        let model = provider::Model {
            name: "test_provider".to_string(),
            display_name: "Test Provider".to_string(),
            api_key: "secret_key".to_string(),
            api_endpoint: "https://example.test".to_string(),
        };

        create_provider(&db, model).await?;

        // Verify the provider was inserted
        let found = provider::Entity::find_by_id("test_provider".to_string())
            .one(&db)
            .await?;
        assert!(found.is_some(), "Inserted provider should be found");
        let found = found.unwrap();
        assert_eq!(found.name, "test_provider");
        assert_eq!(found.display_name, "Test Provider");
        assert_eq!(found.api_key, "secret_key");
        assert_eq!(found.api_endpoint, "https://example.test");

        Ok(())
    }

    /// Ensures inserting multiple providers stores each record.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying all inserted providers exist.
    #[tokio::test]
    async fn test_create_multiple_providers() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let a = provider::Model {
            name: "prov_a".to_string(),
            display_name: "Provider A".to_string(),
            api_key: "key_a".to_string(),
            api_endpoint: "https://a.test".to_string(),
        };
        let b = provider::Model {
            name: "prov_b".to_string(),
            display_name: "Provider B".to_string(),
            api_key: "key_b".to_string(),
            api_endpoint: "https://b.test".to_string(),
        };

        create_provider(&db, a).await?;
        create_provider(&db, b).await?;

        let found_a = provider::Entity::find_by_id("prov_a".to_string())
            .one(&db)
            .await?;
        let found_b = provider::Entity::find_by_id("prov_b".to_string())
            .one(&db)
            .await?;

        assert!(found_a.is_some(), "prov_a should be found");
        assert!(found_b.is_some(), "prov_b should be found");

        Ok(())
    }

    /// Confirms `get_provider` returns an existing provider.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after retrieving the expected provider data.
    #[tokio::test]
    async fn test_get_provider_found() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert a provider and retrieve it via get_provider
        let model = provider::Model {
            name: "test_provider_get".to_string(),
            display_name: "Get Provider".to_string(),
            api_key: "get_key".to_string(),
            api_endpoint: "https://get.example.test".to_string(),
        };

        create_provider(&db, model).await?;

        let found = get_provider(&db, "test_provider_get").await?;
        assert!(
            found.is_some(),
            "get_provider should return Some for existing provider"
        );
        let found = found.unwrap();
        assert_eq!(found.name, "test_provider_get");
        assert_eq!(found.display_name, "Get Provider");
        assert_eq!(found.api_key, "get_key");
        assert_eq!(found.api_endpoint, "https://get.example.test");

        Ok(())
    }

    /// Ensures requesting a missing provider yields `None`.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the lookup result is empty.
    #[tokio::test]
    async fn test_get_provider_not_found() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Ensure requesting a non-existent provider returns None
        let found = get_provider(&db, "nonexistent").await?;
        assert!(
            found.is_none(),
            "get_provider should return None for missing provider"
        );

        Ok(())
    }

    /// Validates pagination produces complete coverage when iterating providers.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once every inserted provider name is observed.
    #[tokio::test]
    async fn test_list_providers_pagination() -> Result<(), DbErr> {
        use std::collections::HashSet;

        let db = setup_db().await?;

        // Insert 5 providers
        for i in 1..=5 {
            let model = provider::Model {
                name: format!("prov_{i}"),
                display_name: format!("Provider {i}"),
                api_key: format!("key_{i}"),
                api_endpoint: format!("https://{i}.test"),
            };
            create_provider(&db, model).await?;
        }

        // Page size 2 should produce 3 pages: 2, 2, 1
        let mut token: Option<String> = None;
        let mut all_names = HashSet::new();

        loop {
            let (items, next) = list_providers(&db, token.clone(), Some(2)).await?;
            for it in items.iter() {
                all_names.insert(it.name.clone());
            }

            if next.is_empty() {
                break;
            }
            token = Some(next);
        }

        assert_eq!(all_names.len(), 5, "Should have collected all 5 providers");

        Ok(())
    }

    /// Confirms invalid pagination tokens fall back to the first page.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once results are returned despite the malformed token.
    #[tokio::test]
    async fn test_list_providers_invalid_token() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert 3 providers
        for i in 1..=3 {
            let model = provider::Model {
                name: format!("bad_{i}"),
                display_name: format!("Bad {i}"),
                api_key: format!("bkey_{i}"),
                api_endpoint: format!("https://bad{i}.test"),
            };
            create_provider(&db, model).await?;
        }

        // Provide an invalid token; function should treat it as offset 0 and return first page
        let (items, next) =
            list_providers(&db, Some("not-a-valid-token".to_string()), Some(2)).await?;
        assert!(
            !items.is_empty(),
            "Should return some items when token is invalid"
        );
        // Because there are 3 items and page_size=2, there should be a next token
        assert!(
            !next.is_empty(),
            "Should return a next_page_token when more pages exist"
        );

        Ok(())
    }

    /// Ensures pagination tokens encode and decode the expected offset.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the offset embedded in the token is correct.
    #[tokio::test]
    async fn test_page_token_roundtrip() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert 4 providers
        for i in 1..=4 {
            let model = provider::Model {
                name: format!("pt_{i}"),
                display_name: format!("PT {i}"),
                api_key: format!("ptkey_{i}"),
                api_endpoint: format!("https://pt{i}.test"),
            };
            create_provider(&db, model).await?;
        }

        // First page: size 2
        let (first_page, next_token) = list_providers(&db, None, Some(2)).await?;
        assert_eq!(first_page.len(), 2, "first page should contain 2 items");
        assert!(!next_token.is_empty(), "should have a next token");

        // Decode token and check it equals offset 2
        let decoded = general_purpose::STANDARD
            .decode(next_token.clone())
            .expect("decode token");
        assert_eq!(decoded.len(), 8, "token should decode to 8 bytes");
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&decoded);
        let offset = u64::from_be_bytes(arr);
        assert_eq!(offset, 2u64, "next offset encoded in token should be 2");

        // Use token to fetch second page
        let (second_page, final_token) = list_providers(&db, Some(next_token), Some(2)).await?;
        assert_eq!(second_page.len(), 2, "second page should contain 2 items");
        // Since there were 4 total and page size 2, final token should be empty
        assert!(
            final_token.is_empty(),
            "final token should be empty when no more pages"
        );

        Ok(())
    }

    /// Verifies providers are removed when deleted and extra deletes are safe.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once deletions succeed without errors.
    #[tokio::test]
    async fn test_delete_provider() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert a provider to delete
        let model = provider::Model {
            name: "del_test".to_string(),
            display_name: "Delete Test".to_string(),
            api_key: "del_key".to_string(),
            api_endpoint: "https://del.test".to_string(),
        };

        create_provider(&db, model).await?;

        // Ensure it exists
        let found = provider::Entity::find_by_id("del_test".to_string())
            .one(&db)
            .await?;
        assert!(found.is_some(), "provider should exist before deletion");

        // Delete the provider
        delete_provider(&db, "del_test").await?;

        // Ensure it's gone
        let found = provider::Entity::find_by_id("del_test".to_string())
            .one(&db)
            .await?;
        assert!(found.is_none(), "provider should be removed after deletion");

        // Deleting again should not error
        delete_provider(&db, "del_test").await?;

        Ok(())
    }
    /// Confirms updates overwrite provider fields as expected.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying the persisted provider reflects the new values.
    #[tokio::test]
    async fn test_update_provider() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert initial provider
        let initial = provider::Model {
            name: "up_test".to_string(),
            display_name: "Before Update".to_string(),
            api_key: "before_key".to_string(),
            api_endpoint: "https://before.test".to_string(),
        };
        create_provider(&db, initial).await?;

        // Prepare updated model with the same primary key (name)
        let updated = provider::Model {
            name: "up_test".to_string(),
            display_name: "After Update".to_string(),
            api_key: "after_key".to_string(),
            api_endpoint: "https://after.test".to_string(),
        };

        // Call the function under test
        update_provider(&db, updated).await?;

        // Verify the changes were persisted
        let found = provider::Entity::find_by_id("up_test".to_string())
            .one(&db)
            .await?;
        assert!(found.is_some(), "provider should exist after update");
        let found = found.unwrap();
        assert_eq!(found.name, "up_test");
        assert_eq!(found.display_name, "After Update");
        assert_eq!(found.api_key, "after_key");
        assert_eq!(found.api_endpoint, "https://after.test");

        Ok(())
    }
}
