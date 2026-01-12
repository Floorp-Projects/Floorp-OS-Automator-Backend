// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! CRUD operations for external plugin packages.
//!
//! This module provides functions to manage external plugin packages that are
//! installed from the filesystem and tracked in the database.

use entity::entity::ext_plugin_package::{self, ActiveModel, Entity as ExtPluginPackage, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

/// Creates a new external plugin package record in the database.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `plugin_package_id` - Unique identifier for the plugin (author_id/package_id/version)
/// * `install_dir` - Directory path where the plugin is installed
///
/// # Returns
///
/// Returns the created `Model` on success, or a database error.
pub async fn create_ext_plugin_package(
    db: &DatabaseConnection,
    plugin_package_id: String,
    install_dir: String,
) -> Result<Model, DbErr> {
    let active_model = ActiveModel {
        plugin_package_id: Set(plugin_package_id),
        install_dir: Set(install_dir),
        missing: Set(false),
    };

    active_model.insert(db).await
}

/// Retrieves an external plugin package by its ID.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `plugin_package_id` - The unique identifier of the plugin
///
/// # Returns
///
/// Returns `Some(Model)` if found, `None` otherwise.
pub async fn get_ext_plugin_package(
    db: &DatabaseConnection,
    plugin_package_id: &str,
) -> Result<Option<Model>, DbErr> {
    ExtPluginPackage::find_by_id(plugin_package_id.to_string())
        .one(db)
        .await
}

/// Lists all external plugin packages in the database.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a vector of all external plugin package models.
pub async fn list_ext_plugin_packages(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    ExtPluginPackage::find().all(db).await
}

/// Updates the missing status of an external plugin package.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `plugin_package_id` - The unique identifier of the plugin
/// * `missing` - Whether the plugin files are missing from the filesystem
///
/// # Returns
///
/// Returns the updated model or an error if the plugin was not found.
pub async fn mark_ext_plugin_missing(
    db: &DatabaseConnection,
    plugin_package_id: &str,
    missing: bool,
) -> Result<Model, DbErr> {
    let existing = ExtPluginPackage::find_by_id(plugin_package_id.to_string())
        .one(db)
        .await?;

    match existing {
        Some(model) => {
            let mut active_model: ActiveModel = model.into();
            active_model.missing = Set(missing);
            active_model.update(db).await
        }
        None => Err(DbErr::RecordNotFound(format!(
            "External plugin package not found: {}",
            plugin_package_id
        ))),
    }
}

/// Deletes an external plugin package record from the database.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `plugin_package_id` - The unique identifier of the plugin to delete
///
/// # Returns
///
/// Returns the number of deleted records (0 or 1).
pub async fn delete_ext_plugin_package(
    db: &DatabaseConnection,
    plugin_package_id: &str,
) -> Result<u64, DbErr> {
    let result = ExtPluginPackage::delete_by_id(plugin_package_id.to_string())
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

/// Lists all external plugin packages that are marked as missing.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a vector of external plugin packages where `missing = true`.
pub async fn list_missing_ext_plugin_packages(
    db: &DatabaseConnection,
) -> Result<Vec<Model>, DbErr> {
    ExtPluginPackage::find()
        .filter(ext_plugin_package::Column::Missing.eq(true))
        .all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

        // ext_plugin_package table
        let sql = r#"
            CREATE TABLE ext_plugin_package (
                plugin_package_id TEXT NOT NULL PRIMARY KEY,
                install_dir TEXT NOT NULL,
                missing INTEGER NOT NULL DEFAULT 0
            )
        "#;
        db.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_create_and_get_ext_plugin_package() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let created = create_ext_plugin_package(
            &db,
            "test-author/test-package/1.0.0".to_string(),
            "/tmp/test-author/test-package/1.0.0".to_string(),
        )
        .await?;

        assert_eq!(created.plugin_package_id, "test-author/test-package/1.0.0");
        assert_eq!(created.install_dir, "/tmp/test-author/test-package/1.0.0");
        assert!(!created.missing);

        let fetched = get_ext_plugin_package(&db, "test-author/test-package/1.0.0").await?;
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.plugin_package_id, "test-author/test-package/1.0.0");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_ext_plugin_packages() -> Result<(), DbErr> {
        let db = setup_db().await?;

        create_ext_plugin_package(
            &db,
            "author1/pkg1/1.0.0".to_string(),
            "/tmp/author1/pkg1/1.0.0".to_string(),
        )
        .await?;

        create_ext_plugin_package(
            &db,
            "author2/pkg2/2.0.0".to_string(),
            "/tmp/author2/pkg2/2.0.0".to_string(),
        )
        .await?;

        let all = list_ext_plugin_packages(&db).await?;
        assert_eq!(all.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_missing() -> Result<(), DbErr> {
        let db = setup_db().await?;

        create_ext_plugin_package(
            &db,
            "test/pkg/1.0.0".to_string(),
            "/tmp/test/pkg/1.0.0".to_string(),
        )
        .await?;

        // Mark as missing
        let updated = mark_ext_plugin_missing(&db, "test/pkg/1.0.0", true).await?;
        assert!(updated.missing);

        // Verify via list
        let missing_list = list_missing_ext_plugin_packages(&db).await?;
        assert_eq!(missing_list.len(), 1);
        assert_eq!(missing_list[0].plugin_package_id, "test/pkg/1.0.0");

        // Mark as not missing
        let updated = mark_ext_plugin_missing(&db, "test/pkg/1.0.0", false).await?;
        assert!(!updated.missing);

        let missing_list = list_missing_ext_plugin_packages(&db).await?;
        assert_eq!(missing_list.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_ext_plugin_package() -> Result<(), DbErr> {
        let db = setup_db().await?;

        create_ext_plugin_package(
            &db,
            "delete-test/pkg/1.0.0".to_string(),
            "/tmp/delete-test/pkg/1.0.0".to_string(),
        )
        .await?;

        let deleted = delete_ext_plugin_package(&db, "delete-test/pkg/1.0.0").await?;
        assert_eq!(deleted, 1);

        let fetched = get_ext_plugin_package(&db, "delete-test/pkg/1.0.0").await?;
        assert!(fetched.is_none());

        Ok(())
    }
}
