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
use entity::entity::plugin_function;
use entity::entity::plugin_package;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    QuerySelect,
};

#[allow(dead_code)]
/// Inserts a plugin function definition into the catalog.
///
/// # Arguments
///
/// * `db` - The database connection used for persistence.
/// * `pf` - The plugin function model to insert.
///
/// # Returns
///
/// Returns `Ok(())` when the record is created, or a [`DbErr`] if insertion fails.
pub(crate) async fn create_plugin_function(
    db: &DatabaseConnection,
    pf: plugin_function::Model,
) -> Result<(), DbErr> {
    let active_model: plugin_function::ActiveModel = pf.into();
    active_model.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
/// Fetches a plugin function along with its optional package metadata.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `function_id` - The unique function identifier to locate.
///
/// # Returns
///
/// Returns `Ok(Some((function, package)))` when found, `Ok(None)` otherwise, or a [`DbErr`] on failure.
pub(crate) async fn get_plugin_function(
    db: &DatabaseConnection,
    function_id: &str,
) -> Result<Option<(plugin_function::Model, Option<plugin_package::Model>)>, DbErr> {
    let row = plugin_function::Entity::find()
        .filter(plugin_function::Column::FunctionId.eq(function_id.to_string()))
        .one(db)
        .await?;
    if let Some(r) = row {
        let pkg = r.find_related(plugin_package::Entity).one(db).await?;
        Ok(Some((r, pkg)))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
/// Updates an existing plugin function with new metadata.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `pf` - The updated plugin function data.
///
/// # Returns
///
/// Returns `Ok(())` after applying changes, regardless of whether the record existed.
pub(crate) async fn update_plugin_function(
    db: &DatabaseConnection,
    pf: plugin_function::Model,
) -> Result<(), DbErr> {
    let existing = plugin_function::Entity::find()
        .filter(plugin_function::Column::FunctionId.eq(pf.function_id.clone()))
        .one(db)
        .await?;
    if let Some(existing) = existing {
        let mut active_model: plugin_function::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.package_id = Set(pf.package_id);
        active_model.function_name = Set(pf.function_name);
        active_model.description = Set(pf.description);
        active_model.arguments = Set(pf.arguments);
        active_model.returns = Set(pf.returns);
        active_model.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Lists plugin functions, optionally filtered by package, with pagination support.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `package_id` - Optional package identifier to filter results.
/// * `next_page_token` - An optional cursor representing the next offset.
/// * `page_size` - An optional limit on the number of rows to return.
///
/// # Returns
///
/// Returns the matching plugin functions paired with any related package and the next page token.
pub(crate) async fn list_plugin_functions(
    db: &DatabaseConnection,
    package_id: Option<String>,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<
    (
        Vec<(plugin_function::Model, Option<plugin_package::Model>)>,
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

    let mut finder = plugin_function::Entity::find();
    if let Some(ref pid) = package_id {
        finder = finder.filter(plugin_function::Column::PackageId.eq(pid.clone()));
    }

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
        let pkg = it.find_related(plugin_package::Entity).one(db).await?;
        out.push((it, pkg));
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
/// Deletes a plugin function identified by its function ID.
///
/// # Arguments
///
/// * `db` - The database connection used for deletion.
/// * `function_id` - The identifier of the function to remove.
///
/// # Returns
///
/// Returns `Ok(())` even if the function did not exist, or a [`DbErr`] on errors.
pub(crate) async fn delete_plugin_function(
    db: &DatabaseConnection,
    function_id: &str,
) -> Result<(), DbErr> {
    let found = plugin_function::Entity::find()
        .filter(plugin_function::Column::FunctionId.eq(function_id.to_string()))
        .one(db)
        .await?;
    if let Some(found) = found {
        let active_model: plugin_function::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

    /// Sets up an in-memory SQLite database with plugin tables for testing.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] ready for plugin function tests.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

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

    // helpers for tests
    /// Inserts a sample plugin package required for foreign keys in tests.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection used to insert the package.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the package is inserted.
    async fn insert_test_package(db: &DatabaseConnection) -> Result<(), DbErr> {
        let pkg = plugin_package::Model {
            package_id: "pkg1".to_string(),
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

    /// Inserts a plugin function row for use within tests.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection on which to insert.
    /// * `id` - The function identifier.
    /// * `name` - The function name to store.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the function entry is persisted.
    async fn insert_test_function(
        db: &DatabaseConnection,
        id: &str,
        name: &str,
    ) -> Result<(), DbErr> {
        let pf = plugin_function::Model {
            function_id: id.to_string(),
            package_id: "pkg1".to_string(),
            function_name: name.to_string(),
            description: Some("D".to_string()),
            arguments: None,
            returns: None,
        };
        create_plugin_function(db, pf).await
    }

    /// Validates plugin functions can be created and retrieved with their associated package.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the stored function and package are returned.
    #[tokio::test]
    async fn test_create_plugin_function() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db).await?;

        insert_test_function(&db, "f_create", "FnCreate").await?;

        let found = get_plugin_function(&db, "f_create").await?;
        assert!(found.is_some());
        let (found_pf, found_pkg) = found.unwrap();
        assert_eq!(found_pf.function_id, "f_create");
        assert!(found_pkg.is_some());
        Ok(())
    }

    /// Ensures fetching a missing plugin function returns `None`.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying the lookup yields no result.
    #[tokio::test]
    async fn test_get_plugin_function_when_missing() -> Result<(), DbErr> {
        let db = setup_db().await?;
        // No package or function inserted
        let found = get_plugin_function(&db, "does_not_exist").await?;
        assert!(found.is_none());
        Ok(())
    }

    /// Confirms updates replace stored plugin function metadata.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the updated function name is observed.
    #[tokio::test]
    async fn test_update_plugin_function() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db).await?;
        insert_test_function(&db, "f_upd", "FnOld").await?;

        // fetch, modify and update
        let found = get_plugin_function(&db, "f_upd").await?;
        assert!(found.is_some());
        let (mut found_pf, _) = found.unwrap();
        found_pf.function_name = "FnNew".to_string();
        update_plugin_function(&db, found_pf.clone()).await?;

        let found_after = get_plugin_function(&db, "f_upd").await?;
        let (found_pf_after, _) = found_after.unwrap();
        assert_eq!(found_pf_after.function_name, "FnNew");
        Ok(())
    }

    /// Checks that plugin functions can be listed and paginated successfully.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying the list contains the seeded function.
    #[tokio::test]
    async fn test_list_plugin_functions_basic() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db).await?;
        insert_test_function(&db, "f_list1", "L1").await?;

        let (list, token) = list_plugin_functions(&db, None, None, Some(10)).await?;
        assert_eq!(list.len(), 1);
        assert!(token.is_empty());
        Ok(())
    }

    /// Ensures deleting a plugin function removes it from storage.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the deleted function can no longer be fetched.
    #[tokio::test]
    async fn test_delete_plugin_function() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_package(&db).await?;
        insert_test_function(&db, "f_del", "ToDelete").await?;

        let found = get_plugin_function(&db, "f_del").await?;
        assert!(found.is_some());

        delete_plugin_function(&db, "f_del").await?;

        let found_after = get_plugin_function(&db, "f_del").await?;
        assert!(found_after.is_none());
        Ok(())
    }
}
