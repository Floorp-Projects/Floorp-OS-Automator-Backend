// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use base64::Engine as _;
use base64::engine::general_purpose;
use entity::entity::plugin_function;
use entity::entity::plugin_package;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    QuerySelect,
};

#[allow(dead_code)]
/// Inserts a plugin package record into the catalog.
///
/// # Arguments
///
/// * `db` - The database connection used to persist the package.
/// * `pkg` - The plugin package model containing metadata.
///
/// # Returns
///
/// Returns `Ok(())` when the insert succeeds, or a [`DbErr`] on failure.
pub(crate) async fn create_plugin_package(
    db: &DatabaseConnection,
    pkg: plugin_package::Model,
) -> Result<(), DbErr> {
    let active_model: plugin_package::ActiveModel = pkg.into();
    active_model.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
/// Retrieves a plugin package and its associated functions by package ID.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `package_id` - The package identifier to look up.
///
/// # Returns
///
/// Returns `Ok(Some((package, functions)))` when found, `Ok(None)` when missing, or a [`DbErr`] on failure.
pub(crate) async fn get_plugin_package(
    db: &DatabaseConnection,
    package_id: &str,
) -> Result<Option<(plugin_package::Model, Vec<plugin_function::Model>)>, DbErr> {
    let row = plugin_package::Entity::find()
        .filter(plugin_package::Column::PackageId.eq(package_id.to_string()))
        .one(db)
        .await?;
    if let Some(r) = row {
        let funcs = r.find_related(plugin_function::Entity).all(db).await?;
        Ok(Some((r, funcs)))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
/// Updates the stored metadata for an existing plugin package.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `pkg` - The package data whose fields should be persisted.
///
/// # Returns
///
/// Returns `Ok(())` after attempting the update, regardless of whether the package exists.
pub(crate) async fn update_plugin_package(
    db: &DatabaseConnection,
    pkg: plugin_package::Model,
) -> Result<(), DbErr> {
    let existing = plugin_package::Entity::find()
        .filter(plugin_package::Column::PackageId.eq(pkg.package_id.clone()))
        .one(db)
        .await?;
    if let Some(existing) = existing {
        let mut active_model: plugin_package::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.package_name = Set(pkg.package_name);
        active_model.package_version = Set(pkg.package_version);
        active_model.description = Set(pkg.description);
        active_model.plugin_store_url = Set(pkg.plugin_store_url);
        active_model.internal_plugin = Set(pkg.internal_plugin);
        active_model.verified = Set(pkg.verified);
        active_model.deprecated = Set(pkg.deprecated);
        active_model.installed_at = Set(pkg.installed_at);
        active_model.updated_at = Set(pkg.updated_at);
        active_model.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Lists plugin packages with pagination, including their associated functions.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `next_page_token` - An optional cursor identifying the next offset.
/// * `page_size` - An optional limit on the number of packages to fetch.
///
/// # Returns
///
/// Returns the retrieved packages paired with their functions and the next page token.
pub(crate) async fn list_plugin_packages(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<
    (
        Vec<(plugin_package::Model, Vec<plugin_function::Model>)>,
        String,
    ),
    DbErr,
> {
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

    let finder = plugin_package::Entity::find();
    let mut items = finder
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (items.len() as u64) > limit;
    if has_next {
        items.truncate(limit as usize);
    }

    let mut out = Vec::with_capacity(items.len());
    for it in items.into_iter() {
        let funcs = it.find_related(plugin_function::Entity).all(db).await?;
        out.push((it, funcs));
    }

    let next_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        let bytes = next_offset.to_be_bytes();
        general_purpose::STANDARD.encode(bytes)
    } else {
        String::new()
    };

    Ok((out, next_token))
}

#[allow(dead_code)]
/// Deletes a plugin package identified by its package ID.
///
/// # Arguments
///
/// * `db` - The database connection used to perform the deletion.
/// * `package_id` - The package identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` even if the package is absent, or a [`DbErr`] when deletion fails.
pub(crate) async fn delete_plugin_package(
    db: &DatabaseConnection,
    package_id: &str,
) -> Result<(), DbErr> {
    let found = plugin_package::Entity::find()
        .filter(plugin_package::Column::PackageId.eq(package_id.to_string()))
        .one(db)
        .await?;
    if let Some(found) = found {
        let active_model: plugin_package::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

    /// Configures an in-memory database with plugin package and function tables for testing.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] prepared for package CRUD tests.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

        // plugin_package table
        let sql_pkg = r#"
            CREATE TABLE plugin_package (
                package_id TEXT PRIMARY KEY,
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                description TEXT,
                plugin_store_url TEXT,
                internal_plugin INTEGER NOT NULL,
                verified INTEGER NOT NULL,
                deprecated INTEGER NOT NULL,
                installed_at TEXT,
                updated_at TEXT
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pkg.to_string(),
        ))
        .await?;

        // plugin_function table
        let sql_pf = r#"
            CREATE TABLE plugin_function (
                function_id TEXT NOT NULL UNIQUE,
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

        Ok(db)
    }

    /// Inserts a sample plugin package used across tests.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection on which to insert.
    /// * `id` - The package identifier to create.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the package is persisted.
    async fn insert_test_package(db: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
        let pkg = plugin_package::Model {
            package_id: id.to_string(),
            package_name: "P".to_string(),
            package_version: "0.1".to_string(),
            description: None,
            plugin_store_url: None,
            internal_plugin: false,
            verified: false,
            deprecated: false,
            installed_at: None,
            updated_at: None,
        };
        let active_pkg: plugin_package::ActiveModel = pkg.into();
        active_pkg.insert(db).await?;
        Ok(())
    }

    /// Inserts a plugin function associated with a given package for test scenarios.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection used for insertion.
    /// * `id` - The function identifier.
    /// * `pkg_id` - The package to associate with.
    /// * `name` - The function name to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the function is inserted.
    async fn insert_test_function(
        db: &DatabaseConnection,
        id: &str,
        pkg_id: &str,
        name: &str,
    ) -> Result<(), DbErr> {
        let pf = plugin_function::Model {
            function_id: id.to_string(),
            package_id: pkg_id.to_string(),
            function_name: name.to_string(),
            description: Some("D".to_string()),
            arguments: None,
            returns: None,
        };
        let active_pf: plugin_function::ActiveModel = pf.into();
        active_pf.insert(db).await?;
        Ok(())
    }

    /// Ensures plugin packages can be created and retrieved without functions.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the package exists and has no associated functions.
    #[tokio::test]
    async fn test_create_plugin_package() -> Result<(), DbErr> {
        let db = setup_db().await?;

        insert_test_package(&db, "pkgA").await?;

        let found = get_plugin_package(&db, "pkgA").await?;
        assert!(found.is_some());
        let (pkg, funcs) = found.unwrap();
        assert_eq!(pkg.package_id, "pkgA");
        assert!(funcs.is_empty());
        Ok(())
    }

    /// Verifies retrieving a package includes the associated functions.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after ensuring two functions are returned for the package.
    #[tokio::test]
    async fn test_get_plugin_package_with_functions() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db, "pkgB").await?;
        insert_test_function(&db, "f1", "pkgB", "F1").await?;
        insert_test_function(&db, "f2", "pkgB", "F2").await?;

        let found = get_plugin_package(&db, "pkgB").await?;
        assert!(found.is_some());
        let (_pkg, funcs) = found.unwrap();
        assert_eq!(funcs.len(), 2);
        Ok(())
    }

    /// Confirms updates modify stored plugin package data.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after the package name is updated successfully.
    #[tokio::test]
    async fn test_update_plugin_package() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db, "pkgU").await?;

        let found = get_plugin_package(&db, "pkgU").await?;
        assert!(found.is_some());
        let (mut pkg, _) = found.unwrap();
        pkg.package_name = "NewName".to_string();
        update_plugin_package(&db, pkg.clone()).await?;

        let found_after = get_plugin_package(&db, "pkgU").await?;
        let (pkg_after, _) = found_after.unwrap();
        assert_eq!(pkg_after.package_name, "NewName");
        Ok(())
    }

    /// Checks that packages can be listed and produce an empty pagination token when exhausted.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying listing returns the inserted packages.
    #[tokio::test]
    async fn test_list_plugin_packages_basic() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db, "pkgL1").await?;
        insert_test_package(&db, "pkgL2").await?;

        let (list, token) = list_plugin_packages(&db, None, Some(10)).await?;
        assert!(list.len() >= 2);
        assert!(token.is_empty());
        Ok(())
    }

    /// Ensures deleting a plugin package removes it from storage.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the package can no longer be retrieved.
    #[tokio::test]
    async fn test_delete_plugin_package() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db, "pkgD").await?;

        let found = get_plugin_package(&db, "pkgD").await?;
        assert!(found.is_some());

        delete_plugin_package(&db, "pkgD").await?;

        let found_after = get_plugin_package(&db, "pkgD").await?;
        assert!(found_after.is_none());
        Ok(())
    }
}
