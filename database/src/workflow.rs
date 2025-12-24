// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

pub mod workflow_code_allowed_permission_crud;
pub mod workflow_code_crud;
pub mod workflow_crud;
pub mod workflow_result_crud;

use entity::convert::{
    proto_allowed_permissions_to_entities, proto_string_to_option, proto_timestamp_to_datetime,
    proto_to_plugin_function, proto_to_plugin_package, proto_to_workflow_code,
    proto_to_workflow_code_plugin_functions, proto_to_workflow_code_plugin_packages,
    proto_to_workflow_result,
};
use entity::entity::{
    permission, plugin_function, plugin_package, workflow, workflow_code,
    workflow_code_allowed_permission, workflow_code_plugin_function, workflow_code_plugin_package,
    workflow_result,
};
use sapphillon_core::proto::sapphillon::v1::{Workflow, WorkflowCode};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use uuid::Uuid;

pub async fn create_workflow_code(
    db: &DatabaseConnection,
    code: String,
    workflow_id: String,
    plugin_function_ids: Vec<String>,
    plugin_package_ids: Vec<String>,
) -> Result<WorkflowCode, DbErr> {
    // Build an entity model and delegate insertion to the CRUD helper.
    // Note: workflow IDs are stored as strings in the entity model.
    // Start code_revision at 1 and set a
    // default language of 0 (WORKFLOW_LANGUAGE_UNSPECIFIED).
    let wc = entity::entity::workflow_code::Model {
        id: Uuid::new_v4().to_string(),
        workflow_id,
        code_revision: 1,
        code,
        language: 0,
        created_at: None,
    };
    // Use the internal CRUD helper which returns Result<(), DbErr>.
    workflow_code_crud::create_workflow_code(db, wc.clone()).await?;

    // Insert related plugin functions.
    let plugin_function_ids = plugin_function_ids
        .iter()
        .map(
            |pf_id| entity::entity::workflow_code_plugin_function::ActiveModel {
                workflow_code_id: Set(wc.id.to_owned()),
                plugin_function_id: Set(pf_id.clone()),
                ..Default::default()
            },
        )
        .collect::<Vec<_>>();
    entity::entity::workflow_code_plugin_function::Entity::insert_many(plugin_function_ids)
        .on_empty_do_nothing()
        .exec(db)
        .await?;

    // Insert related plugin packages.
    let plugin_package_ids = plugin_package_ids
        .iter()
        .map(
            |pp_id| entity::entity::workflow_code_plugin_package::ActiveModel {
                workflow_code_id: Set(wc.id.to_owned()),
                plugin_package_id: Set(pp_id.clone()),
                ..Default::default()
            },
        )
        .collect::<Vec<_>>();
    entity::entity::workflow_code_plugin_package::Entity::insert_many(plugin_package_ids)
        .on_empty_do_nothing()
        .exec(db)
        .await?;

    // Convert the model into the proto type to return.
    let proto = WorkflowCode {
        id: wc.id,
        code_revision: wc.code_revision,
        code: wc.code,
        language: wc.language,
        created_at: None, // keep None for now; mapping timestamps requires conversion
        result: Vec::new(),
        plugin_packages: Vec::new(),
        plugin_function_ids: Vec::new(),
        allowed_permissions: Vec::new(),
    };

    Ok(proto)
}

pub async fn create_workflow(
    db: &DatabaseConnection,
    display_name: String,
    description: Option<String>,
    workflow_language: i32,
) -> Result<Workflow, DbErr> {
    let wm = entity::entity::workflow::Model {
        id: Uuid::new_v4().to_string(),
        display_name: display_name.clone(),
        description: description.clone(),
        workflow_language,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    };

    workflow_crud::create_workflow(db, wm.clone()).await?;

    // Return a Workflow proto object
    let proto = Workflow {
        id: wm.id,
        display_name: wm.display_name,
        description: wm.description.unwrap_or("".to_string()),
        workflow_language: wm.workflow_language,
        workflow_code: Vec::new(),
        created_at: wm
            .created_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        updated_at: wm
            .updated_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        workflow_results: Vec::new(),
    };

    Ok(proto)
}

pub async fn get_workflow_by_id(
    db: &DatabaseConnection,
    workflow_id: &str,
) -> Result<Workflow, DbErr> {
    let workflow = entity::entity::workflow::Entity::find_by_id(workflow_id.to_string())
        .one(db)
        .await?;

    let wm = match workflow {
        Some(m) => m,
        None => {
            return Err(DbErr::Custom(format!("workflow not found: {workflow_id}")));
        }
    };

    // Load all workflow_code rows for this workflow.
    let wcs = entity::entity::workflow_code::Entity::find()
        .filter(entity::entity::workflow_code::Column::WorkflowId.eq(workflow_id.to_string()))
        .all(db)
        .await?;

    // continue to load relations and build the proto below
    let mut proto_wcs: Vec<WorkflowCode> = Vec::new();

    for wc in wcs.iter() {
        // Load results for this workflow code and convert to proto
        let results_entities = entity::entity::workflow_result::Entity::find()
            .filter(entity::entity::workflow_result::Column::WorkflowCodeId.eq(wc.id.clone()))
            .all(db)
            .await?;

        let proto_results: Vec<sapphillon_core::proto::sapphillon::v1::WorkflowResult> =
            results_entities
                .iter()
                .map(|r| sapphillon_core::proto::sapphillon::v1::WorkflowResult {
                    id: r.id.clone(),
                    display_name: r.display_name.clone().unwrap_or_default(),
                    description: r.description.clone().unwrap_or_default(),
                    result: r.result.clone().unwrap_or_default(),
                    ran_at: r.ran_at.map(|dt| {
                        sapphillon_core::proto::google::protobuf::Timestamp {
                            seconds: dt.timestamp(),
                            nanos: dt.timestamp_subsec_nanos() as i32,
                        }
                    }),
                    result_type: r.result_type,
                    exit_code: r.exit_code.unwrap_or_default(),
                    workflow_result_revision: r.workflow_result_revision,
                })
                .collect();

        // Load plugin_package entities attached to this workflow_code
        let wcpp = entity::entity::workflow_code_plugin_package::Entity::find()
            .filter(
                entity::entity::workflow_code_plugin_package::Column::WorkflowCodeId
                    .eq(wc.id.clone()),
            )
            .find_also_related(entity::entity::plugin_package::Entity)
            .all(db)
            .await?;

        let plugin_packages: Vec<entity::entity::plugin_package::Model> = wcpp
            .into_iter()
            .filter_map(|(_link, pkg_opt)| pkg_opt)
            .collect();

        // Load plugin function ids attached to this workflow_code
        let wcpf = entity::entity::workflow_code_plugin_function::Entity::find()
            .filter(
                entity::entity::workflow_code_plugin_function::Column::WorkflowCodeId
                    .eq(wc.id.clone()),
            )
            .all(db)
            .await?;

        let plugin_function_ids: Vec<String> =
            wcpf.iter().map(|e| e.plugin_function_id.clone()).collect();

        // Load allowed permissions relation tuples (allowed_permission, permission)
        let allowed = entity::entity::workflow_code_allowed_permission::Entity::find()
            .filter(
                entity::entity::workflow_code_allowed_permission::Column::WorkflowCodeId
                    .eq(wc.id.clone()),
            )
            .find_also_related(entity::entity::permission::Entity)
            .all(db)
            .await?;

        let allowed_tuples: Vec<(
            entity::entity::workflow_code_allowed_permission::Model,
            Option<entity::entity::permission::Model>,
        )> = allowed.into_iter().collect();

        // Convert the workflow_code entity into proto, attaching relations where available
        let wc_proto = entity::convert::workflow_code::workflow_code_to_proto_with_relations(
            wc,
            Some(&proto_results),
            Some(&plugin_packages),
            Some(&plugin_function_ids),
            Some(&allowed_tuples),
        );

        proto_wcs.push(wc_proto);
    }

    // Load workflow-level results (all results for the workflow)
    let wf_results_entities = entity::entity::workflow_result::Entity::find()
        .filter(entity::entity::workflow_result::Column::WorkflowId.eq(workflow_id.to_string()))
        .all(db)
        .await?;

    let proto_wf_results: Vec<sapphillon_core::proto::sapphillon::v1::WorkflowResult> =
        wf_results_entities
            .iter()
            .map(|r| sapphillon_core::proto::sapphillon::v1::WorkflowResult {
                id: r.id.clone(),
                display_name: r.display_name.clone().unwrap_or_default(),
                description: r.description.clone().unwrap_or_default(),
                result: r.result.clone().unwrap_or_default(),
                ran_at: r
                    .ran_at
                    .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }),
                result_type: r.result_type,
                exit_code: r.exit_code.unwrap_or_default(),
                workflow_result_revision: r.workflow_result_revision,
            })
            .collect();

    let proto = Workflow {
        id: wm.id,
        display_name: wm.display_name,
        description: wm.description.unwrap_or_default(),
        workflow_language: wm.workflow_language,
        workflow_code: proto_wcs,
        created_at: wm
            .created_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        updated_at: wm
            .updated_at
            .map(|dt| sapphillon_core::proto::google::protobuf::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        workflow_results: proto_wf_results,
    };

    Ok(proto)
}

/// Updates a workflow record and its related workflow code metadata based on the provided
/// protobuf message. All nested structures are synchronized by converting the proto data into
/// SeaORM models through the shared `entity::convert` helpers.
///
/// The implementation performs simple delete-and-replace synchronization for relation tables.
/// Callers should ensure any required rows (e.g. plugin packages/functions) referenced by the
/// proto are present or included in the payload.
pub async fn update_workflow_from_proto(
    db: &DatabaseConnection,
    proto: &Workflow,
) -> Result<Workflow, DbErr> {
    let description = proto_string_to_option(&proto.description);
    let created_at = proto
        .created_at
        .as_ref()
        .and_then(proto_timestamp_to_datetime);
    let updated_at = proto
        .updated_at
        .as_ref()
        .and_then(proto_timestamp_to_datetime);

    let workflow_model = workflow::Model {
        id: proto.id.clone(),
        display_name: proto.display_name.clone(),
        description,
        workflow_language: proto.workflow_language,
        created_at,
        updated_at,
    };

    // Upsert the workflow itself.
    if let Some(existing) = workflow::Entity::find_by_id(workflow_model.id.clone())
        .one(db)
        .await?
    {
        let mut active: workflow::ActiveModel = existing.into();
        active.display_name = Set(workflow_model.display_name.clone());
        active.description = Set(workflow_model.description.clone());
        active.workflow_language = Set(workflow_model.workflow_language);
        active.created_at = Set(workflow_model.created_at);
        active.updated_at = Set(workflow_model.updated_at);
        active.update(db).await?;
    } else {
        let active: workflow::ActiveModel = workflow_model.clone().into();
        active.insert(db).await?;
    }

    for code_proto in &proto.workflow_code {
        let code_entity = proto_to_workflow_code(code_proto, workflow_model.id.clone());

        if let Some(existing) = workflow_code::Entity::find_by_id(code_entity.id.clone())
            .one(db)
            .await?
        {
            let mut active: workflow_code::ActiveModel = existing.into();
            active.workflow_id = Set(code_entity.workflow_id.clone());
            active.code_revision = Set(code_entity.code_revision);
            active.code = Set(code_entity.code.clone());
            active.language = Set(code_entity.language);
            active.created_at = Set(code_entity.created_at);
            active.update(db).await?;
        } else {
            let active: workflow_code::ActiveModel = code_entity.clone().into();
            active.insert(db).await?;
        }

        // Ensure plugin packages referenced by the workflow code exist and are up to date.
        for pkg_proto in &code_proto.plugin_packages {
            let pkg_entity = proto_to_plugin_package(pkg_proto);
            if let Some(existing) =
                plugin_package::Entity::find_by_id(pkg_entity.package_id.clone())
                    .one(db)
                    .await?
            {
                let mut active: plugin_package::ActiveModel = existing.into();
                active.package_name = Set(pkg_entity.package_name.clone());
                active.package_version = Set(pkg_entity.package_version.clone());
                active.description = Set(pkg_entity.description.clone());
                active.plugin_store_url = Set(pkg_entity.plugin_store_url.clone());
                active.internal_plugin = Set(pkg_entity.internal_plugin);
                active.verified = Set(pkg_entity.verified);
                active.deprecated = Set(pkg_entity.deprecated);
                active.installed_at = Set(pkg_entity.installed_at);
                active.updated_at = Set(pkg_entity.updated_at);
                active.update(db).await?;
            } else {
                let active: plugin_package::ActiveModel = pkg_entity.clone().into();
                active.insert(db).await?;
            }

            for func_proto in &pkg_proto.functions {
                let func_entity =
                    proto_to_plugin_function(func_proto, pkg_entity.package_id.clone());
                if let Some(existing) = plugin_function::Entity::find_by_id((
                    func_entity.function_id.clone(),
                    func_entity.package_id.clone(),
                ))
                .one(db)
                .await?
                {
                    let mut active: plugin_function::ActiveModel = existing.into();
                    active.package_id = Set(func_entity.package_id.clone());
                    active.function_name = Set(func_entity.function_name.clone());
                    active.description = Set(func_entity.description.clone());
                    active.arguments = Set(func_entity.arguments.clone());
                    active.returns = Set(func_entity.returns.clone());
                    active.update(db).await?;
                } else {
                    let active: plugin_function::ActiveModel = func_entity.clone().into();
                    active.insert(db).await?;
                }
            }
        }

        // Reset plugin package links for this workflow code.
        workflow_code_plugin_package::Entity::delete_many()
            .filter(workflow_code_plugin_package::Column::WorkflowCodeId.eq(code_entity.id.clone()))
            .exec(db)
            .await?;

        let package_links = proto_to_workflow_code_plugin_packages(
            code_entity.id.clone(),
            &code_proto.plugin_packages,
        );
        if !package_links.is_empty() {
            let active_models: Vec<_> = package_links
                .into_iter()
                .map(|link| workflow_code_plugin_package::ActiveModel {
                    id: NotSet,
                    workflow_code_id: Set(link.workflow_code_id),
                    plugin_package_id: Set(link.plugin_package_id),
                })
                .collect();
            workflow_code_plugin_package::Entity::insert_many(active_models)
                .exec(db)
                .await?;
        }

        // Reset plugin function links for this workflow code.
        workflow_code_plugin_function::Entity::delete_many()
            .filter(
                workflow_code_plugin_function::Column::WorkflowCodeId.eq(code_entity.id.clone()),
            )
            .exec(db)
            .await?;

        let function_links = proto_to_workflow_code_plugin_functions(
            code_entity.id.clone(),
            &code_proto.plugin_function_ids,
        );
        if !function_links.is_empty() {
            let active_models: Vec<_> = function_links
                .into_iter()
                .map(|link| workflow_code_plugin_function::ActiveModel {
                    id: NotSet,
                    workflow_code_id: Set(link.workflow_code_id),
                    plugin_function_id: Set(link.plugin_function_id),
                })
                .collect();
            workflow_code_plugin_function::Entity::insert_many(active_models)
                .exec(db)
                .await?;
        }

        // Replace allowed permissions for this workflow code.
        let existing_relations = workflow_code_allowed_permission::Entity::find()
            .filter(
                workflow_code_allowed_permission::Column::WorkflowCodeId.eq(code_entity.id.clone()),
            )
            .all(db)
            .await?;

        if !existing_relations.is_empty() {
            workflow_code_allowed_permission::Entity::delete_many()
                .filter(
                    workflow_code_allowed_permission::Column::WorkflowCodeId
                        .eq(code_entity.id.clone()),
                )
                .exec(db)
                .await?;

            let permission_ids: Vec<i32> = existing_relations
                .iter()
                .map(|rel| rel.permission_id)
                .collect();
            if !permission_ids.is_empty() {
                permission::Entity::delete_many()
                    .filter(permission::Column::Id.is_in(permission_ids))
                    .exec(db)
                    .await?;
            }
        }

        let allowed_pairs = proto_allowed_permissions_to_entities(
            code_entity.id.clone(),
            &code_proto.allowed_permissions,
        );

        for (relation_model, permission_model) in allowed_pairs {
            let permission_active = permission::ActiveModel {
                id: NotSet,
                plugin_function_id: Set(permission_model.plugin_function_id.clone()),
                display_name: Set(permission_model.display_name.clone()),
                description: Set(permission_model.description.clone()),
                r#type: Set(permission_model.r#type),
                resource_json: Set(permission_model.resource_json.clone()),
                level: Set(permission_model.level),
            };
            let inserted_permission = permission_active.insert(db).await?;

            let relation_active = workflow_code_allowed_permission::ActiveModel {
                id: NotSet,
                workflow_code_id: Set(relation_model.workflow_code_id),
                permission_id: Set(inserted_permission.id),
            };
            relation_active.insert(db).await?;
        }

        // Refresh workflow results for this code.
        workflow_result::Entity::delete_many()
            .filter(workflow_result::Column::WorkflowCodeId.eq(code_entity.id.clone()))
            .exec(db)
            .await?;

        for result_proto in &code_proto.result {
            let result_model = proto_to_workflow_result(
                result_proto,
                workflow_model.id.clone(),
                code_entity.id.clone(),
            );

            let active = workflow_result::ActiveModel {
                id: Set(result_model.id),
                workflow_id: Set(result_model.workflow_id),
                workflow_code_id: Set(result_model.workflow_code_id),
                display_name: Set(result_model.display_name),
                description: Set(result_model.description),
                result: Set(result_model.result),
                ran_at: Set(result_model.ran_at),
                result_type: Set(result_model.result_type),
                exit_code: Set(result_model.exit_code),
                workflow_result_revision: Set(result_model.workflow_result_revision),
            };
            active.insert(db).await?;
        }
    }

    // Note: Top-level workflow results (Workflow.workflow_results) are not synchronized here,
    // because the proto message omits the workflow_code_id required by the schema.

    get_workflow_by_id(db, &workflow_model.id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ColumnTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, QueryFilter,
        Statement,
    };

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        // workflow table (referenced)
        let sql_wf = r#"
            CREATE TABLE workflow (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                description TEXT,
                workflow_language INTEGER NOT NULL,
                created_at TEXT,
                updated_at TEXT
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_wf.to_string(),
        ))
        .await?;

        // workflow_code table
        let sql_wc = r#"
            CREATE TABLE workflow_code (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                code_revision INTEGER NOT NULL,
                code TEXT NOT NULL,
                language INTEGER NOT NULL,
                created_at TEXT
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_wc.to_string(),
        ))
        .await?;

        // plugin_package table
        let sql_pp = r#"
            CREATE TABLE plugin_package (
                package_id TEXT PRIMARY KEY,
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                description TEXT,
                plugin_store_url TEXT,
                internal_plugin BOOLEAN NOT NULL,
                verified BOOLEAN NOT NULL,
                deprecated BOOLEAN NOT NULL,
                installed_at TEXT,
                updated_at TEXT
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pp.to_string(),
        ))
        .await?;

        // plugin_function table
        let sql_pf = r#"
            CREATE TABLE plugin_function (
                function_id TEXT PRIMARY KEY,
                package_id TEXT NOT NULL,
                function_name TEXT NOT NULL,
                description TEXT,
                arguments TEXT,
                returns TEXT
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_pf.to_string(),
        ))
        .await?;

        // workflow_code_plugin_function table
        let sql_wcpf = r#"
            CREATE TABLE workflow_code_plugin_function (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_code_id TEXT NOT NULL,
                plugin_function_id TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_wcpf.to_string(),
        ))
        .await?;

        // workflow_code_plugin_package table
        let sql_wcpp = r#"
            CREATE TABLE workflow_code_plugin_package (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_code_id TEXT NOT NULL,
                plugin_package_id TEXT NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_wcpp.to_string(),
        ))
        .await?;

        Ok(db)
    }

    async fn setup_full_db() -> Result<DatabaseConnection, DbErr> {
        let db = setup_db().await?;

        let sql_permission = r#"
            CREATE TABLE permission (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
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
            sql_permission.to_string(),
        ))
        .await?;

        let sql_wc_allowed = r#"
            CREATE TABLE workflow_code_allowed_permission (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_code_id TEXT NOT NULL,
                permission_id INTEGER NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_wc_allowed.to_string(),
        ))
        .await?;

        let sql_workflow_result = r#"
            CREATE TABLE workflow_result (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                workflow_code_id TEXT NOT NULL,
                display_name TEXT,
                description TEXT,
                result TEXT,
                ran_at TEXT,
                result_type INTEGER NOT NULL,
                exit_code INTEGER,
                workflow_result_revision INTEGER NOT NULL
            )
        "#;
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            sql_workflow_result.to_string(),
        ))
        .await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_create_workflow_code_inserts_plugins() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert referenced workflow
        let wf_id = "wf1".to_string();
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            format!(
                "INSERT INTO workflow (id, display_name, workflow_language) VALUES ('{wf_id}','WF', 0)"
            ),
        ))
        .await?;

        // Insert plugin package and function
        let pkg_id = "pkg1".to_string();
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            format!("INSERT INTO plugin_package (package_id, package_name, package_version, internal_plugin, verified, deprecated) VALUES ('{pkg_id}','P', 'v1', 0, 0, 0)"),
        ))
        .await?;

        let func_id = "func1".to_string();
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            format!("INSERT INTO plugin_function (function_id, package_id, function_name) VALUES ('{func_id}','{pkg_id}','f')"),
        ))
        .await?;

        // Call the function under test
        let code = "print('hi')".to_string();
        let proto = create_workflow_code(
            &db,
            code,
            wf_id.clone(),
            vec![func_id.clone()],
            vec![pkg_id.clone()],
        )
        .await?;

        // Verify workflow_code row exists
        let found_wc = entity::entity::workflow_code::Entity::find_by_id(proto.id.clone())
            .one(&db)
            .await?;
        assert!(found_wc.is_some());

        // Verify plugin function relation exists
        let wcpf = entity::entity::workflow_code_plugin_function::Entity::find()
            .filter(
                entity::entity::workflow_code_plugin_function::Column::PluginFunctionId
                    .eq(func_id.clone()),
            )
            .all(&db)
            .await?;
        assert!(!wcpf.is_empty());

        // Verify plugin package relation exists
        let wcpp = entity::entity::workflow_code_plugin_package::Entity::find()
            .filter(
                entity::entity::workflow_code_plugin_package::Column::PluginPackageId
                    .eq(pkg_id.clone()),
            )
            .all(&db)
            .await?;
        assert!(!wcpp.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_workflow_inserts_row_and_returns_proto() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Call create_workflow with a display name and description
        let display_name = "My Workflow".to_string();
        let description = Some("A test workflow".to_string());
        let description_text = description.clone().expect("description seeded");

        let proto = create_workflow(&db, display_name.clone(), description.clone(), 0).await?;

        // Ensure a workflow row was inserted
        let found = entity::entity::workflow::Entity::find_by_id(proto.id.clone())
            .one(&db)
            .await?;
        assert!(found.is_some());
        let model = found.unwrap();

        // Check model fields match inputs
        assert_eq!(model.display_name, display_name);
        assert_eq!(model.description, description.clone());

        // Check returned proto fields
        assert_eq!(proto.display_name, display_name);
        assert_eq!(proto.description, description_text);

        // created_at/updated_at should be present in the proto (mapping from chrono)
        assert!(proto.created_at.is_some());
        assert!(proto.updated_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_workflow_from_proto_synchronizes_relations() -> Result<(), DbErr> {
        use sapphillon_core::proto::sapphillon::v1::{
            AllowedPermission, Permission, PermissionLevel, PermissionType, PluginFunction,
            PluginPackage as ProtoPluginPackage, WorkflowCode as ProtoWorkflowCode,
            WorkflowResult as ProtoWorkflowResult, WorkflowResultType,
        };

        let db = setup_full_db().await?;

        let workflow_proto = Workflow {
            id: "wf1".to_string(),
            display_name: "Updated Workflow".to_string(),
            description: "Updated description".to_string(),
            workflow_language: 1,
            workflow_code: vec![ProtoWorkflowCode {
                id: "wc1".to_string(),
                code_revision: 3,
                code: "console.log('hello')".to_string(),
                language: 1,
                created_at: None,
                result: vec![ProtoWorkflowResult {
                    id: "res1".to_string(),
                    display_name: "Latest".to_string(),
                    description: String::new(),
                    result: String::new(),
                    ran_at: None,
                    result_type: WorkflowResultType::SuccessUnspecified as i32,
                    exit_code: 0,
                    workflow_result_revision: 7,
                }],
                plugin_packages: vec![ProtoPluginPackage {
                    package_id: "pkg1".to_string(),
                    package_name: "Demo Package".to_string(),
                    package_version: "1.0.0".to_string(),
                    description: "Plugin description".to_string(),
                    functions: vec![PluginFunction {
                        function_id: "pkg.fn".to_string(),
                        function_name: "Fn".to_string(),
                        description: "Run function".to_string(),
                        permissions: Vec::new(),
                        arguments: "{}".to_string(),
                        returns: "{}".to_string(),
                    }],
                    plugin_store_url: "https://example.com".to_string(),
                    internal_plugin: Some(false),
                    verified: Some(true),
                    deprecated: Some(false),
                    installed_at: None,
                    updated_at: None,
                }],
                plugin_function_ids: vec!["pkg.fn".to_string()],
                allowed_permissions: vec![AllowedPermission {
                    plugin_function_id: "pkg.fn".to_string(),
                    permissions: vec![Permission {
                        display_name: "Read".to_string(),
                        description: "Read secret".to_string(),
                        permission_type: PermissionType::FilesystemRead as i32,
                        resource: vec!["secrets/path".to_string()],
                        permission_level: PermissionLevel::High as i32,
                    }],
                }],
            }],
            created_at: None,
            updated_at: None,
            workflow_results: Vec::new(),
        };

        let updated = update_workflow_from_proto(&db, &workflow_proto).await?;

        let stored_workflow = workflow::Entity::find_by_id("wf1".to_string())
            .one(&db)
            .await?
            .expect("workflow inserted");
        assert_eq!(stored_workflow.display_name, "Updated Workflow");
        assert_eq!(
            stored_workflow.description.as_deref(),
            Some("Updated description")
        );

        assert_eq!(updated.workflow_code.len(), 1);

        let stored_package = plugin_package::Entity::find_by_id("pkg1".to_string())
            .one(&db)
            .await?;
        assert!(stored_package.is_some());

        let stored_function =
            plugin_function::Entity::find_by_id(("pkg.fn".to_string(), "pkg1".to_string()))
                .one(&db)
                .await?;
        assert!(stored_function.is_some());

        let package_links = workflow_code_plugin_package::Entity::find()
            .filter(workflow_code_plugin_package::Column::WorkflowCodeId.eq("wc1".to_string()))
            .all(&db)
            .await?;
        assert_eq!(package_links.len(), 1);

        let function_links = workflow_code_plugin_function::Entity::find()
            .filter(workflow_code_plugin_function::Column::WorkflowCodeId.eq("wc1".to_string()))
            .all(&db)
            .await?;
        assert_eq!(function_links.len(), 1);

        let allowed_links = workflow_code_allowed_permission::Entity::find()
            .filter(workflow_code_allowed_permission::Column::WorkflowCodeId.eq("wc1".to_string()))
            .all(&db)
            .await?;
        assert_eq!(allowed_links.len(), 1);

        let permission_record = permission::Entity::find_by_id(allowed_links[0].permission_id)
            .one(&db)
            .await?
            .expect("permission inserted");
        let resources: Vec<String> = permission_record
            .resource_json
            .as_ref()
            .map(|json| serde_json::from_str(json).expect("valid json"))
            .unwrap_or_default();
        assert_eq!(resources, vec!["secrets/path".to_string()]);

        let results = workflow_result::Entity::find()
            .filter(workflow_result::Column::WorkflowCodeId.eq("wc1".to_string()))
            .all(&db)
            .await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].workflow_result_revision, 7);

        Ok(())
    }
}
