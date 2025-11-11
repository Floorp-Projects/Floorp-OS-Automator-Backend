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
// along with this program.  If not, see <https://www.gnu.org/licenses/>

pub mod plugin_function_crud;
pub mod plugin_function_permission_crud;
pub mod plugin_package_crud;

use sea_orm::{DatabaseConnection, DbErr};

// entity models are converted via helpers in `entity::convert::plugin_code`.

use sapphillon_core::proto::sapphillon::v1::PluginPackage as ProtoPluginPackage;

/// Lists plugin packages and returns protobuf `PluginPackage` messages.
///
/// This wraps the lower-level CRUD implementation in `plugin_package_crud` and
/// converts entity models into their protobuf counterparts. Function-level
/// permissions are loaded per-function when available and attached to the
/// converted `PluginFunction` messages.
pub async fn list_plugins(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<ProtoPluginPackage>, String), DbErr> {
    // Get packages and their functions from the CRUD layer.
    let (items, token) =
        plugin_package_crud::list_plugin_packages(db, next_page_token, page_size).await?;

    let mut out: Vec<ProtoPluginPackage> = Vec::with_capacity(items.len());

    // Collect all function ids for batch loading permissions.
    let mut all_function_ids: Vec<String> = Vec::new();
    for (_pkg_entity, func_entities) in items.iter() {
        for func in func_entities.iter() {
            all_function_ids.push(func.function_id.clone());
        }
    }

    // Batch-load all function permission relations in one query.
    let perm_relations =
        plugin_function_permission_crud::list_plugin_function_permissions_for_function_ids(
            db,
            &all_function_ids,
        )
        .await?;

    // Map function_id -> Vec<permission::Model>
    use std::collections::HashMap;
    let mut perms_by_function: HashMap<String, Vec<entity::entity::permission::Model>> =
        HashMap::new();
    for (_rel, perm_opt, _func_opt) in perm_relations.into_iter() {
        if let Some(perm) = perm_opt {
            perms_by_function
                .entry(perm.plugin_function_id.clone())
                .or_default()
                .push(perm);
        }
    }

    // Now convert packages & their functions to proto using the in-memory map.
    for (pkg_entity, func_entities) in items.into_iter() {
        let mut proto_funcs: Vec<sapphillon_core::proto::sapphillon::v1::PluginFunction> =
            Vec::new();
        for func in func_entities.into_iter() {
            let proto_perms: Vec<sapphillon_core::proto::sapphillon::v1::Permission> =
                match perms_by_function.get(&func.function_id) {
                    Some(perm_entities) => perm_entities
                        .iter()
                        .map(entity::convert::plugin::permission_to_proto)
                        .collect(),
                    None => Vec::new(),
                };

            let proto_fn =
                entity::convert::plugin::plugin_function_to_proto(&func, Some(&proto_perms));
            proto_funcs.push(proto_fn);
        }

        let proto_pkg = entity::convert::plugin::plugin_package_to_proto_with_functions(
            &pkg_entity,
            Some(&proto_funcs),
        );
        out.push(proto_pkg);
    }

    Ok((out, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
    };

    async fn setup_db() -> Result<DatabaseConnection, sea_orm::DbErr> {
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

        // permission table
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

    async fn insert_package(db: &DatabaseConnection, id: &str) -> Result<(), sea_orm::DbErr> {
        let pkg = entity::entity::plugin_package::Model {
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
        let active: entity::entity::plugin_package::ActiveModel = pkg.into();
        active.insert(db).await?;
        Ok(())
    }

    async fn insert_function(
        db: &DatabaseConnection,
        fid: &str,
        pkg: &str,
        name: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let pf = entity::entity::plugin_function::Model {
            function_id: fid.to_string(),
            package_id: pkg.to_string(),
            function_name: name.to_string(),
            description: Some("D".to_string()),
            arguments: None,
            returns: None,
        };
        let active: entity::entity::plugin_function::ActiveModel = pf.into();
        active.insert(db).await?;
        Ok(())
    }

    async fn insert_permission(
        db: &DatabaseConnection,
        id: i32,
        func_id: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let perm = entity::entity::permission::Model {
            id,
            plugin_function_id: func_id.to_string(),
            display_name: Some("dn".to_string()),
            description: None,
            r#type: 0,
            resource_json: None,
            level: None,
        };
        let active: entity::entity::permission::ActiveModel = perm.into();
        active.insert(db).await?;
        Ok(())
    }

    async fn link_permission(
        db: &DatabaseConnection,
        func_id: &str,
        perm_id: &str,
    ) -> Result<(), sea_orm::DbErr> {
        use sea_orm::ActiveValue::{NotSet, Set};
        let active = entity::entity::plugin_function_permission::ActiveModel {
            id: NotSet,
            plugin_function_id: Set(func_id.to_string()),
            permission_id: Set(perm_id.to_string()),
        };
        active.insert(db).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_list_plugins_includes_function_permissions() -> Result<(), sea_orm::DbErr> {
        let db = setup_db().await?;

        insert_package(&db, "pkg1").await?;
        insert_function(&db, "pkg1.fn1", "pkg1", "F1").await?;
        insert_function(&db, "pkg1.fn2", "pkg1", "F2").await?;

        insert_permission(&db, 101, "pkg1.fn1").await?;
        insert_permission(&db, 102, "pkg1.fn2").await?;

        link_permission(&db, "pkg1.fn1", "101").await?;
        link_permission(&db, "pkg1.fn2", "102").await?;

        let (pkgs, token) = list_plugins(&db, None, Some(10)).await?;
        assert!(token.is_empty());
        assert_eq!(pkgs.len(), 1);
        let pkg = &pkgs[0];
        assert_eq!(pkg.package_id, "pkg1");
        // two functions
        assert_eq!(pkg.functions.len(), 2);
        // each function should have its permission attached
        let mut perm_counts: Vec<usize> =
            pkg.functions.iter().map(|f| f.permissions.len()).collect();
        perm_counts.sort();
        assert_eq!(perm_counts, vec![1, 1]);

        Ok(())
    }
}
