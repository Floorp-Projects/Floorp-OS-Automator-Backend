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
use entity::entity::permission;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

/// Inserts a permission record into the database.
///
/// # Arguments
///
/// * `db` - The database connection used for persistence.
/// * `p` - The permission entity to insert.
///
/// # Returns
///
/// Returns `Ok(())` on success or a [`DbErr`] when insertion fails.
pub async fn create_permission(db: &DatabaseConnection, p: permission::Model) -> Result<(), DbErr> {
    let active_model: permission::ActiveModel = p.into();
    active_model.insert(db).await?;
    Ok(())
}

/// Retrieves a permission by primary key.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `id` - The permission identifier.
///
/// # Returns
///
/// Returns `Ok(Some(permission))` when found, `Ok(None)` when absent, or a [`DbErr`] on failure.
pub async fn get_permission(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<permission::Model>, DbErr> {
    let p = permission::Entity::find_by_id(id).one(db).await?;
    Ok(p)
}

/// Updates an existing permission's mutable fields when it exists.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `p` - The permission data containing the desired field values.
///
/// # Returns
///
/// Returns `Ok(())` after applying updates, regardless of whether the record existed.
pub async fn update_permission(db: &DatabaseConnection, p: permission::Model) -> Result<(), DbErr> {
    let existing = get_permission(db, p.id).await?;
    if let Some(existing) = existing {
        let mut active_model: permission::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.plugin_function_id = Set(p.plugin_function_id);
        active_model.display_name = Set(p.display_name);
        active_model.description = Set(p.description);
        active_model.r#type = Set(p.r#type);
        active_model.resource_json = Set(p.resource_json);
        active_model.level = Set(p.level);
        active_model.update(db).await?;
    }
    Ok(())
}

/// Lists permissions using encoded offsets for pagination.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `next_page_token` - An optional cursor indicating the starting offset.
/// * `page_size` - An optional limit on the number of rows to return.
///
/// # Returns
///
/// Returns the retrieved permission models and the next page token (empty when exhausted).
pub async fn list_permissions(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<permission::Model>, String), DbErr> {
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
    let mut items = permission::Entity::find()
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

/// Removes a permission by its identifier if it exists.
///
/// # Arguments
///
/// * `db` - The database connection to execute against.
/// * `id` - The permission identifier to delete.
///
/// # Returns
///
/// Returns `Ok(())` even if the permission was absent, or a [`DbErr`] if the delete fails.
pub async fn delete_permission(db: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let found = permission::Entity::find_by_id(id).one(db).await?;
    if let Some(found) = found {
        let active_model: permission::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::entity::plugin_function;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

    /// Creates a temporary SQLite database with the tables required for permission tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a ready-to-use [`DatabaseConnection`] for unit tests.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        // Create plugin_function table because permission references it
        let sql_pf = r#"
            CREATE TABLE plugin_function (
                function_id TEXT NOT NULL,
                package_id TEXT NOT NULL,
                function_name TEXT NOT NULL,
                description TEXT,
                arguments TEXT,
                returns TEXT,
                PRIMARY KEY (function_id, package_id)
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pf.to_string(),
        ))
        .await?;

        // Create permission table
        let sql_perm = r#"
            CREATE TABLE permission (
                id INTEGER PRIMARY KEY,
                plugin_function_id TEXT NOT NULL,
                display_name TEXT,
                description TEXT,
                type INTEGER NOT NULL,
                resource_json TEXT,
                level INTEGER
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_perm.to_string(),
        ))
        .await?;

        Ok(db)
    }

    /// Exercises the full lifecycle of a permission record, including create, update, and delete.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the lifecycle operations succeed.
    #[tokio::test]
    async fn test_create_get_update_delete_permission() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert plugin function required by FK
        let pf = plugin_function::Model {
            function_id: "f1".to_string(),
            package_id: "pkg".to_string(),
            function_name: "fn".to_string(),
            description: None,
            arguments: None,
            returns: None,
        };
        // Insert directly using ActiveModel
        let active_pf: plugin_function::ActiveModel = pf.into();
        active_pf.insert(&db).await?;

        let p = permission::Model {
            id: 1,
            plugin_function_id: "f1".to_string(),
            display_name: Some("Perm".to_string()),
            description: Some("desc".to_string()),
            r#type: 2,
            resource_json: Some("{}".to_string()),
            level: Some(1),
        };

        create_permission(&db, p.clone()).await?;

        let found = get_permission(&db, 1).await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, 1);
        assert_eq!(found.plugin_function_id, "f1");

        // Update
        let mut updated = found.clone();
        updated.display_name = Some("PermX".to_string());
        updated.description = None;
        update_permission(&db, updated).await?;

        let found = get_permission(&db, 1).await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.display_name.as_deref(), Some("PermX"));
        assert!(found.description.is_none());

        // Delete
        delete_permission(&db, 1).await?;
        let found = get_permission(&db, 1).await?;
        assert!(found.is_none());

        Ok(())
    }

    /// Validates pagination over the permission table returns all rows.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after iterating through pages covering all inserted permissions.
    #[tokio::test]
    async fn test_list_permissions_pagination() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert plugin function
        let pf = plugin_function::Model {
            function_id: "fx".to_string(),
            package_id: "pkg".to_string(),
            function_name: "fnx".to_string(),
            description: None,
            arguments: None,
            returns: None,
        };
        let active_pf: plugin_function::ActiveModel = pf.into();
        active_pf.insert(&db).await?;

        for i in 1..=5 {
            let p = permission::Model {
                id: i,
                plugin_function_id: "fx".to_string(),
                display_name: Some(format!("P{i}")),
                description: None,
                r#type: 1,
                resource_json: None,
                level: None,
            };
            create_permission(&db, p).await?;
        }

        let mut token: Option<String> = None;
        let mut count = 0;
        loop {
            let (items, next) = list_permissions(&db, token.clone(), Some(2)).await?;
            count += items.len();
            if next.is_empty() {
                break;
            }
            token = Some(next);
        }

        assert_eq!(count, 5);

        Ok(())
    }
}
