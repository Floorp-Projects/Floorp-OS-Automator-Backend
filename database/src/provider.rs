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

use entity::entity::provider;
use sea_orm::{DatabaseConnection, DbErr, ActiveModelTrait, EntityTrait};


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

pub async fn get_provider(
    db: &DatabaseConnection,
    name: &str,
) -> Result<Option<provider::Model>, DbErr> {
    let provider = provider::Entity::find_by_id(name.to_string()).one(db).await?;
    Ok(provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement};

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
        db.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string())).await?;

        Ok(db)
    }

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
        let found = provider::Entity::find_by_id("test_provider".to_string()).one(&db).await?;
        assert!(found.is_some(), "Inserted provider should be found");
        let found = found.unwrap();
        assert_eq!(found.name, "test_provider");
        assert_eq!(found.display_name, "Test Provider");
        assert_eq!(found.api_key, "secret_key");
        assert_eq!(found.api_endpoint, "https://example.test");

        Ok(())
    }

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

        let found_a = provider::Entity::find_by_id("prov_a".to_string()).one(&db).await?;
        let found_b = provider::Entity::find_by_id("prov_b".to_string()).one(&db).await?;

        assert!(found_a.is_some(), "prov_a should be found");
        assert!(found_b.is_some(), "prov_b should be found");

        Ok(())
    }

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
        assert!(found.is_some(), "get_provider should return Some for existing provider");
        let found = found.unwrap();
        assert_eq!(found.name, "test_provider_get");
        assert_eq!(found.display_name, "Get Provider");
        assert_eq!(found.api_key, "get_key");
        assert_eq!(found.api_endpoint, "https://get.example.test");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_provider_not_found() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Ensure requesting a non-existent provider returns None
        let found = get_provider(&db, "nonexistent").await?;
        assert!(found.is_none(), "get_provider should return None for missing provider");

        Ok(())
    }
}