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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct PermissionKey {
    plugin_function_id: String,
    display_name: Option<String>,
    description: Option<String>,
    permission_type: i32,
    resource_json: Option<String>,
    level: Option<i32>,
}

impl From<&entity::entity::permission::Model> for PermissionKey {
    fn from(model: &entity::entity::permission::Model) -> Self {
        Self {
            plugin_function_id: model.plugin_function_id.clone(),
            display_name: model.display_name.clone(),
            description: model.description.clone(),
            permission_type: model.r#type,
            resource_json: model.resource_json.clone(),
            level: model.level,
        }
    }
}

pub async fn init_register_plugins(
    db: &DatabaseConnection,
    plugins: Vec<ProtoPluginPackage>,
) -> Result<(), DbErr> {
    use entity::convert::plugin_code::{
        proto_to_permission, proto_to_plugin_function, proto_to_plugin_package,
    };
    use entity::entity::{permission, plugin_function, plugin_function_permission, plugin_package};
    use sea_orm::ActiveValue::{NotSet, Set};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
    use std::collections::{HashMap, HashSet};

    if plugins.is_empty() {
        return Ok(());
    }

    let mut package_models: HashMap<String, plugin_package::Model> = HashMap::new();
    let mut function_models: HashMap<String, plugin_function::Model> = HashMap::new();
    let mut permission_models: HashMap<PermissionKey, permission::Model> = HashMap::new();

    for pkg_proto in &plugins {
        let package_model = proto_to_plugin_package(pkg_proto);
        package_models.insert(package_model.package_id.clone(), package_model.clone());

        for func_proto in &pkg_proto.functions {
            let function_model =
                proto_to_plugin_function(func_proto, package_model.package_id.clone());
            function_models.insert(function_model.function_id.clone(), function_model.clone());

            for perm_proto in &func_proto.permissions {
                let permission_model =
                    proto_to_permission(perm_proto, function_model.function_id.clone(), None);
                let key = PermissionKey::from(&permission_model);
                permission_models.entry(key).or_insert(permission_model);
            }
        }
    }

    let package_ids: Vec<String> = package_models.keys().cloned().collect();
    let function_ids: Vec<String> = function_models.keys().cloned().collect();
    let package_values: Vec<plugin_package::Model> = package_models.values().cloned().collect();
    let function_values: Vec<plugin_function::Model> = function_models.values().cloned().collect();
    let permission_entries: Vec<(PermissionKey, permission::Model)> =
        permission_models.into_iter().collect();

    let txn = db.begin().await?;

    if !package_values.is_empty() {
        let existing_packages = plugin_package::Entity::find()
            .filter(plugin_package::Column::PackageId.is_in(package_ids.clone()))
            .all(&txn)
            .await?;
        let existing_package_ids: HashSet<String> = existing_packages
            .into_iter()
            .map(|pkg| pkg.package_id)
            .collect();

        let packages_to_insert: Vec<plugin_package::Model> = package_values
            .iter()
            .filter(|pkg| !existing_package_ids.contains(&pkg.package_id))
            .cloned()
            .collect();

        if !packages_to_insert.is_empty() {
            let active_packages: Vec<plugin_package::ActiveModel> = packages_to_insert
                .into_iter()
                .map(plugin_package::ActiveModel::from)
                .collect();

            plugin_package::Entity::insert_many(active_packages)
                .exec(&txn)
                .await?;
        }
        // Update existing packages if any fields differ.
        // Build a map for incoming package models for quick access.
        let mut incoming_packages: HashMap<String, plugin_package::Model> = HashMap::new();
        for pkg in package_values.iter() {
            incoming_packages.insert(pkg.package_id.clone(), pkg.clone());
        }
        // Re-query existing packages to get the actual records for comparison.
        let existing_packages = plugin_package::Entity::find()
            .filter(plugin_package::Column::PackageId.is_in(package_ids.clone()))
            .all(&txn)
            .await?;
        for existing in existing_packages.into_iter() {
            if let Some(incoming) = incoming_packages.get(&existing.package_id) {
                // Determine if we need to update and which fields changed. Compute this
                // while we still have `existing` available (before moving it).
                let mut needs_update = false;
                if existing.package_name != incoming.package_name {
                    needs_update = true;
                }
                if existing.package_version != incoming.package_version {
                    needs_update = true;
                }
                if existing.description.as_deref() != incoming.description.as_deref() {
                    needs_update = true;
                }
                if existing.plugin_store_url.as_deref() != incoming.plugin_store_url.as_deref() {
                    needs_update = true;
                }
                if existing.internal_plugin != incoming.internal_plugin {
                    needs_update = true;
                }
                if existing.verified != incoming.verified {
                    needs_update = true;
                }
                if existing.deprecated != incoming.deprecated {
                    needs_update = true;
                }
                if needs_update {
                    // Move existing into an ActiveModel only if we need to update it.
                    let mut active: plugin_package::ActiveModel = existing.into();
                    active.package_name = Set(incoming.package_name.clone());
                    active.package_version = Set(incoming.package_version.clone());
                    active.description = Set(incoming.description.clone());
                    active.plugin_store_url = Set(incoming.plugin_store_url.clone());
                    active.internal_plugin = Set(incoming.internal_plugin);
                    active.verified = Set(incoming.verified);
                    active.deprecated = Set(incoming.deprecated);
                    plugin_package::Entity::update(active).exec(&txn).await?;
                }
            }
        }
    }

    if !function_values.is_empty() {
        let existing_functions = plugin_function::Entity::find()
            .filter(plugin_function::Column::FunctionId.is_in(function_ids.clone()))
            .all(&txn)
            .await?;
        let existing_function_ids: HashSet<String> = existing_functions
            .into_iter()
            .map(|func| func.function_id)
            .collect();

        let functions_to_insert: Vec<plugin_function::Model> = function_values
            .iter()
            .filter(|func| !existing_function_ids.contains(&func.function_id))
            .cloned()
            .collect();

        if !functions_to_insert.is_empty() {
            let active_functions: Vec<plugin_function::ActiveModel> = functions_to_insert
                .into_iter()
                .map(plugin_function::ActiveModel::from)
                .collect();

            plugin_function::Entity::insert_many(active_functions)
                .exec(&txn)
                .await?;
        }
        // Update existing functions if fields differ.
        let existing_functions = plugin_function::Entity::find()
            .filter(plugin_function::Column::FunctionId.is_in(function_ids.clone()))
            .all(&txn)
            .await?;
        let mut incoming_functions: HashMap<String, plugin_function::Model> = HashMap::new();
        for f in function_values.iter() {
            incoming_functions.insert(f.function_id.clone(), f.clone());
        }
        for existing in existing_functions.into_iter() {
            if let Some(incoming) = incoming_functions.get(&existing.function_id) {
                let mut needs_update = false;
                if existing.function_name != incoming.function_name {
                    needs_update = true;
                }
                if existing.description.as_deref() != incoming.description.as_deref() {
                    needs_update = true;
                }
                if existing.arguments.as_deref() != incoming.arguments.as_deref() {
                    needs_update = true;
                }
                if existing.returns.as_deref() != incoming.returns.as_deref() {
                    needs_update = true;
                }
                if needs_update {
                    let mut active: plugin_function::ActiveModel = existing.into();
                    active.function_name = Set(incoming.function_name.clone());
                    active.description = Set(incoming.description.clone());
                    active.arguments = Set(incoming.arguments.clone());
                    active.returns = Set(incoming.returns.clone());
                    plugin_function::Entity::update(active).exec(&txn).await?;
                }
            }
        }
    }

    if !function_ids.is_empty() && !permission_entries.is_empty() {
        let mut existing_permissions: HashMap<PermissionKey, permission::Model> = HashMap::new();

        let found_permissions = permission::Entity::find()
            .filter(permission::Column::PluginFunctionId.is_in(function_ids.clone()))
            .all(&txn)
            .await?;
        for perm in found_permissions {
            existing_permissions.insert(PermissionKey::from(&perm), perm);
        }

        let mut new_permission_models = Vec::new();
        for (key, model) in &permission_entries {
            if !existing_permissions.contains_key(key) {
                new_permission_models.push(model.clone());
            }
        }

        if !new_permission_models.is_empty() {
            let active_permissions: Vec<permission::ActiveModel> = new_permission_models
                .into_iter()
                .map(|model| permission::ActiveModel {
                    id: NotSet,
                    plugin_function_id: Set(model.plugin_function_id.clone()),
                    display_name: Set(model.display_name.clone()),
                    description: Set(model.description.clone()),
                    r#type: Set(model.r#type),
                    resource_json: Set(model.resource_json.clone()),
                    level: Set(model.level),
                })
                .collect();

            permission::Entity::insert_many(active_permissions)
                .exec(&txn)
                .await?;
        }

        existing_permissions.clear();
        let refreshed_permissions = permission::Entity::find()
            .filter(permission::Column::PluginFunctionId.is_in(function_ids.clone()))
            .all(&txn)
            .await?;
        for perm in refreshed_permissions {
            existing_permissions.insert(PermissionKey::from(&perm), perm);
        }

        let existing_links = plugin_function_permission::Entity::find()
            .filter(
                plugin_function_permission::Column::PluginFunctionId.is_in(function_ids.clone()),
            )
            .all(&txn)
            .await?;

        let mut link_set: HashSet<(String, String)> = existing_links
            .into_iter()
            .map(|link| (link.plugin_function_id, link.permission_id))
            .collect();

        let mut new_link_models = Vec::new();
        for (key, _) in &permission_entries {
            if let Some(permission_model) = existing_permissions.get(key) {
                let perm_id_str = permission_model.id.to_string();
                let fn_id = permission_model.plugin_function_id.clone();
                if link_set.insert((fn_id.clone(), perm_id_str.clone())) {
                    new_link_models.push(plugin_function_permission::ActiveModel {
                        id: NotSet,
                        plugin_function_id: Set(fn_id),
                        permission_id: Set(perm_id_str),
                    });
                }
            }
        }

        if !new_link_models.is_empty() {
            plugin_function_permission::Entity::insert_many(new_link_models)
                .exec(&txn)
                .await?;
        }
    }

    txn.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait,
        Statement,
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
    async fn test_init_register_plugins_registers_data_once() -> Result<(), sea_orm::DbErr> {
        use sapphillon_core::proto::sapphillon::v1::{
            Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
        };

        let db = setup_db().await?;

        let permission_proto = Permission {
            display_name: "Network Access".to_string(),
            description: "Allows outbound requests".to_string(),
            permission_type: PermissionType::NetAccess as i32,
            resource: vec!["https://example.com".to_string()],
            permission_level: PermissionLevel::Medium as i32,
        };

        let function_proto = PluginFunction {
            function_id: "pkg.fn".to_string(),
            function_name: "Fn".to_string(),
            description: "Example function".to_string(),
            permissions: vec![permission_proto.clone()],
            arguments: String::new(),
            returns: String::new(),
        };

        let package_proto = PluginPackage {
            package_id: "pkg".to_string(),
            package_name: "Pkg".to_string(),
            package_version: "1.0.0".to_string(),
            description: "Example package".to_string(),
            functions: vec![function_proto],
            plugin_store_url: "builtin".to_string(),
            internal_plugin: Some(true),
            verified: Some(true),
            deprecated: Some(false),
            installed_at: None,
            updated_at: None,
        };

        init_register_plugins(&db, vec![package_proto.clone()]).await?;
        // Second call should be a no-op
        init_register_plugins(&db, vec![package_proto]).await?;

        let packages = entity::entity::plugin_package::Entity::find()
            .all(&db)
            .await?;
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package_id, "pkg");

        let functions = entity::entity::plugin_function::Entity::find()
            .all(&db)
            .await?;
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].function_id, "pkg.fn");

        let permissions = entity::entity::permission::Entity::find().all(&db).await?;
        assert_eq!(permissions.len(), 1);
        let permission_id = permissions[0].id;
        assert_eq!(permissions[0].plugin_function_id, "pkg.fn");

        let links = entity::entity::plugin_function_permission::Entity::find()
            .all(&db)
            .await?;
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(link.plugin_function_id, "pkg.fn");
        assert_eq!(link.permission_id, permission_id.to_string());

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

    #[tokio::test]
    async fn test_init_register_plugins_updates_on_diff() -> Result<(), sea_orm::DbErr> {
        use sapphillon_core::proto::sapphillon::v1::{
            Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
        };

        let db = setup_db().await?;

        let permission_proto = Permission {
            display_name: "Network Access".to_string(),
            description: "Allows outbound requests".to_string(),
            permission_type: PermissionType::NetAccess as i32,
            resource: vec!["https://example.com".to_string()],
            permission_level: PermissionLevel::Medium as i32,
        };

        // initial function/package
        let function_proto_initial = PluginFunction {
            function_id: "pkg.fn".to_string(),
            function_name: "Fn".to_string(),
            description: "Example function".to_string(),
            permissions: vec![permission_proto.clone()],
            arguments: String::new(),
            returns: String::new(),
        };

        let package_proto_initial = PluginPackage {
            package_id: "pkg".to_string(),
            package_name: "Pkg".to_string(),
            package_version: "1.0.0".to_string(),
            description: "Example package".to_string(),
            functions: vec![function_proto_initial.clone()],
            plugin_store_url: "builtin".to_string(),
            internal_plugin: Some(true),
            verified: Some(true),
            deprecated: Some(false),
            installed_at: None,
            updated_at: None,
        };

        // register initial package
        init_register_plugins(&db, vec![package_proto_initial.clone()]).await?;

        // re-register with changed package_version and function description
        let function_proto_changed = PluginFunction {
            description: "Updated description".to_string(),
            ..function_proto_initial
        };

        let package_proto_changed = PluginPackage {
            package_version: "1.0.1".to_string(),
            functions: vec![function_proto_changed.clone()],
            ..package_proto_initial
        };

        init_register_plugins(&db, vec![package_proto_changed]).await?;

        // Fetch the function and package records
        let functions = entity::entity::plugin_function::Entity::find()
            .all(&db)
            .await?;
        assert_eq!(functions.len(), 1);
        // NOTE: current implementation does NOT perform updates, so this assertion reflects
        // desired behavior (update should happen). If this test fails, behavior is the
        // current 'insert-only' semantics.
        assert_eq!(
            functions[0].description.as_deref(),
            Some("Updated description")
        );

        let packages = entity::entity::plugin_package::Entity::find()
            .all(&db)
            .await?;
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package_version, "1.0.1");

        Ok(())
    }
}
