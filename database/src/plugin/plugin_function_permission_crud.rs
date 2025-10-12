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

use entity::entity::permission;
use entity::entity::plugin_function;
use entity::entity::plugin_function_permission;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
};

#[allow(dead_code)]
pub(crate) async fn create_plugin_function_permission(
    db: &DatabaseConnection,
    pfp: plugin_function_permission::Model,
) -> Result<(), DbErr> {
    let active: plugin_function_permission::ActiveModel = pfp.into();
    active.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
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
    use sea_orm::{
        ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement,
    };

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

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
}
