// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use entity::entity::permission;
use entity::entity::plugin_function;
use entity::entity::plugin_function_permission;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
};

#[allow(dead_code)]
/// Creates an association between a plugin function and a permission.
///
/// # Arguments
///
/// * `db` - The database connection used to insert the relation.
/// * `pfp` - The relation model to store.
///
/// # Returns
///
/// Returns `Ok(())` when the link is persisted, or a [`DbErr`] on failure.
pub(crate) async fn create_plugin_function_permission(
    db: &DatabaseConnection,
    pfp: plugin_function_permission::Model,
) -> Result<(), DbErr> {
    // The entity's `Model` contains an `id: i32` field which may be 0 in
    // tests. If we convert the model directly into an ActiveModel the id
    // will be treated as a set value and an explicit `0` may cause
    // UNIQUE constraint violations on `id` for subsequent inserts. To
    // ensure the database assigns the autoincremented primary key, build
    // the ActiveModel explicitly and leave `id` as NotSet so SQLite will
    // generate the value.
    use sea_orm::ActiveValue::{NotSet, Set};
    let active = plugin_function_permission::ActiveModel {
        id: NotSet,
        plugin_function_id: Set(pfp.plugin_function_id),
        permission_id: Set(pfp.permission_id),
    };
    active.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
/// Retrieves a plugin-function permission link with optional related entities.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `id` - The primary key of the relation to fetch.
///
/// # Returns
///
/// Returns `Ok(Some((relation, permission, function)))` when found, `Ok(None)` when missing, or a [`DbErr`] on failure.
pub(crate) async fn get_plugin_function_permission(
    db: &DatabaseConnection,
    id: i32,
) -> Result<
    Option<(
        plugin_function_permission::Model,
        Option<permission::Model>,
        Option<plugin_function::Model>,
    )>,
    DbErr,
> {
    let row = plugin_function_permission::Entity::find()
        .filter(plugin_function_permission::Column::Id.eq(id))
        .one(db)
        .await?;
    if let Some(r) = row {
        let perm = r.find_related(permission::Entity).one(db).await?;
        let func = r.find_related(plugin_function::Entity).one(db).await?;
        Ok(Some((r, perm, func)))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
/// Updates an existing plugin function permission relation.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `pfp` - The updated relation data.
///
/// # Returns
///
/// Returns `Ok(())` after applying changes, regardless of whether the relation existed.
pub(crate) async fn update_plugin_function_permission(
    db: &DatabaseConnection,
    pfp: plugin_function_permission::Model,
) -> Result<(), DbErr> {
    let existing = plugin_function_permission::Entity::find()
        .filter(plugin_function_permission::Column::Id.eq(pfp.id))
        .one(db)
        .await?;
    if let Some(existing) = existing {
        let mut active: plugin_function_permission::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active.plugin_function_id = Set(pfp.plugin_function_id);
        active.permission_id = Set(pfp.permission_id);
        active.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Lists plugin function permission relations, optionally filtered by plugin function ID.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `plugin_function_id` - Optional function identifier to filter results.
///
/// # Returns
///
/// Returns the matching relations paired with related permission and function records.
pub(crate) async fn list_plugin_function_permissions(
    db: &DatabaseConnection,
    plugin_function_id: Option<String>,
) -> Result<
    Vec<(
        plugin_function_permission::Model,
        Option<permission::Model>,
        Option<plugin_function::Model>,
    )>,
    DbErr,
> {
    let mut finder = plugin_function_permission::Entity::find();
    if let Some(ref pfid) = plugin_function_id {
        finder =
            finder.filter(plugin_function_permission::Column::PluginFunctionId.eq(pfid.clone()));
    }
    let items = finder.all(db).await?;
    let mut out = Vec::with_capacity(items.len());
    for it in items.into_iter() {
        let perm = it.find_related(permission::Entity).one(db).await?;
        let func = it.find_related(plugin_function::Entity).one(db).await?;
        out.push((it, perm, func));
    }
    Ok(out)
}

#[allow(dead_code)]
/// Batch-load plugin function permission relations for many function IDs.
///
/// This returns the same shape as `list_plugin_function_permissions` but fetches
/// all matching relations and their related permission and function records in a
/// single database round-trip using `find_also_related`.
pub(crate) async fn list_plugin_function_permissions_for_function_ids(
    db: &DatabaseConnection,
    function_ids: &[String],
) -> Result<
    Vec<(
        plugin_function_permission::Model,
        Option<permission::Model>,
        Option<plugin_function::Model>,
    )>,
    DbErr,
> {
    if function_ids.is_empty() {
        return Ok(Vec::new());
    }

    // SeaORM allows finding related models in the same query.
    // We filter by PluginFunctionId IN (..function_ids..) and also fetch the
    // related permission and plugin_function records.
    let items = plugin_function_permission::Entity::find()
        .filter(plugin_function_permission::Column::PluginFunctionId.is_in(function_ids.to_vec()))
        .find_also_related(permission::Entity)
        .find_also_related(plugin_function::Entity)
        .all(db)
        .await?;

    // `find_also_related` returns Vec<(Model, Option<Related1>, Option<Related2>)>
    Ok(items)
}

#[allow(dead_code)]
/// Deletes a plugin function permission relation by primary key.
///
/// # Arguments
///
/// * `db` - The database connection used to perform the deletion.
/// * `id` - The relation identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` even if the relation was absent, or a [`DbErr`] when deletion fails.
pub(crate) async fn delete_plugin_function_permission(
    db: &DatabaseConnection,
    id: i32,
) -> Result<(), DbErr> {
    let found = plugin_function_permission::Entity::find()
        .filter(plugin_function_permission::Column::Id.eq(id))
        .one(db)
        .await?;
    if let Some(found) = found {
        let active: plugin_function_permission::ActiveModel = found.into();
        active.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

    /// Configures an in-memory database with the tables required for relation tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] ready for link CRUD operations.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

        // permission table (match entity schema)
        let sql_perm = r#"
            CREATE TABLE permission (
                id INTEGER PRIMARY KEY,
                plugin_function_id TEXT NOT NULL,
                display_name TEXT,
                description TEXT,
                "type" INTEGER NOT NULL,
                resource_json TEXT,
                level INTEGER
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_perm.to_string(),
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

        // plugin_function_permission table
        let sql_pfp = r#"
            CREATE TABLE plugin_function_permission (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plugin_function_id TEXT NOT NULL,
                permission_id TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pfp.to_string(),
        ))
        .await?;

        Ok(db)
    }

    /// Inserts a permission used by tests to satisfy foreign key constraints.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection on which to insert.
    /// * `id` - The numeric identifier to assign.
    /// * `plugin_function_id` - The related plugin function identifier.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the permission row is written.
    async fn insert_test_permission(
        db: &DatabaseConnection,
        id: i32,
        plugin_function_id: &str,
    ) -> Result<(), DbErr> {
        let perm = permission::Model {
            id,
            plugin_function_id: plugin_function_id.to_string(),
            display_name: Some("dn".to_string()),
            description: None,
            r#type: 0,
            resource_json: None,
            level: None,
        };
        let active: permission::ActiveModel = perm.into();
        active.insert(db).await?;
        Ok(())
    }

    /// Inserts a plugin function to link against during tests.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection used for insertion.
    /// * `id` - The function identifier to create.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the function is persisted.
    async fn insert_test_function(db: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
        let pf = plugin_function::Model {
            function_id: id.to_string(),
            package_id: "pkg1".to_string(),
            function_name: "F".to_string(),
            description: Some("D".to_string()),
            arguments: None,
            returns: None,
        };
        let active: plugin_function::ActiveModel = pf.into();
        active.insert(db).await?;
        Ok(())
    }

    /// Exercises creating and retrieving plugin function permission links with relations.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after asserting the created link can be retrieved with relations.
    #[tokio::test]
    async fn test_create_and_get_permission_link() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_function(&db, "func1").await?;
        insert_test_permission(&db, 1, "func1").await?;

        let pfp = plugin_function_permission::Model {
            id: 0,
            plugin_function_id: "func1".to_string(),
            permission_id: "1".to_string(),
        };
        create_plugin_function_permission(&db, pfp).await?;

        // find inserted (id will be 1)
        let list = list_plugin_function_permissions(&db, Some("func1".to_string())).await?;
        assert_eq!(list.len(), 1);
        let (found, perm, func) = &list[0];
        assert_eq!(found.plugin_function_id, "func1");
        assert!(perm.is_some());
        assert!(func.is_some());
        Ok(())
    }

    /// Validates updating and deleting plugin function permission links.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the link updates and deletes behave as expected.
    #[tokio::test]
    async fn test_update_and_delete_permission_link() -> Result<(), DbErr> {
        let db = setup_db().await?;
        insert_test_function(&db, "func2").await?;
        insert_test_permission(&db, 2, "func2").await?;
        insert_test_permission(&db, 3, "func2").await?;

        let mut pfp = plugin_function_permission::Model {
            id: 0,
            plugin_function_id: "func2".to_string(),
            permission_id: "2".to_string(),
        };
        create_plugin_function_permission(&db, pfp.clone()).await?;

        let mut list = list_plugin_function_permissions(&db, Some("func2".to_string())).await?;
        assert_eq!(list.len(), 1);
        let id = list.remove(0).0.id;

        // update permission to 3
        pfp.id = id;
        pfp.permission_id = "3".to_string();
        update_plugin_function_permission(&db, pfp.clone()).await?;

        let got = get_plugin_function_permission(&db, id).await?;
        assert!(got.is_some());
        let (_got, perm, _) = got.unwrap();
        assert!(perm.is_some());
        assert_eq!(perm.unwrap().id, 3);

        delete_plugin_function_permission(&db, id).await?;
        let got_after = get_plugin_function_permission(&db, id).await?;
        assert!(got_after.is_none());
        Ok(())
    }

    /// Ensures the batched loader returns relations for multiple function ids in one call.
    #[tokio::test]
    async fn test_batch_list_plugin_function_permissions() -> Result<(), DbErr> {
        let db = setup_db().await?;
        // insert two functions and two permissions
        insert_test_function(&db, "funcA").await?;
        insert_test_function(&db, "funcB").await?;
        insert_test_permission(&db, 11, "funcA").await?;
        insert_test_permission(&db, 12, "funcB").await?;

        let pfp_a = plugin_function_permission::Model {
            id: 0,
            plugin_function_id: "funcA".to_string(),
            permission_id: "11".to_string(),
        };
        let pfp_b = plugin_function_permission::Model {
            id: 0,
            plugin_function_id: "funcB".to_string(),
            permission_id: "12".to_string(),
        };
        create_plugin_function_permission(&db, pfp_a).await?;
        create_plugin_function_permission(&db, pfp_b).await?;

        let ids = vec!["funcA".to_string(), "funcB".to_string()];
        let list = list_plugin_function_permissions_for_function_ids(&db, &ids).await?;
        // should return two relation tuples
        assert_eq!(list.len(), 2);

        // check that each tuple contains the related permission entity
        let mut found_permissions = vec![];
        for (_rel, perm_opt, func_opt) in list.into_iter() {
            assert!(func_opt.is_some());
            assert!(perm_opt.is_some());
            let perm = perm_opt.unwrap();
            found_permissions.push((perm.plugin_function_id.clone(), perm.id));
        }

        found_permissions.sort();
        assert_eq!(
            found_permissions,
            vec![("funcA".to_string(), 11), ("funcB".to_string(), 12)]
        );

        Ok(())
    }
}
