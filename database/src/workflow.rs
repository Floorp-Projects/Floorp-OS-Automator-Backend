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

pub mod workflow_code_allowed_permission_crud;
pub mod workflow_code_crud;
pub mod workflow_crud;
pub mod workflow_result_crud;

use sapphillon_core::proto::sapphillon::v1::{Workflow, WorkflowCode};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use uuid::Uuid;

pub async fn create_workflow_code(
    db: &DatabaseConnection,
    code: String,
    workflow_id: i32,
    plugin_function_ids: Vec<String>,
    plugin_package_ids: Vec<String>,
) -> Result<WorkflowCode, DbErr> {
    // Build an entity model and delegate insertion to the CRUD helper.
    // Note: workflow IDs are stored as strings in the entity model; convert the
    // incoming integer id to string here. Start code_revision at 1 and set a
    // default language of 0 (WORKFLOW_LANGUAGE_UNSPECIFIED).
    let wc = entity::entity::workflow_code::Model {
        id: Uuid::new_v4().to_string(),
        workflow_id: workflow_id.to_string(),
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
) -> Result<Workflow, DbErr> {
    // TODO: Support different workflow languages.
    const WORKFLOW_LANGUAGE: i32 = 0; // WORKFLOW_LANGUAGE_UNSPECIFIED

    let wm = entity::entity::workflow::Model {
        id: Uuid::new_v4().to_string(),
        display_name: display_name.clone(),
        description: description.clone(),
        workflow_language: WORKFLOW_LANGUAGE,
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
            return Err(DbErr::Custom(format!(
                "workflow not found: {workflow_id}"
            )));
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
        let proto =
            create_workflow_code(&db, code, 1, vec![func_id.clone()], vec![pkg_id.clone()]).await?;

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

        let proto = create_workflow(&db, display_name.clone(), description.clone()).await?;

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
        assert_eq!(proto.description, description.unwrap());

        // created_at/updated_at should be present in the proto (mapping from chrono)
        assert!(proto.created_at.is_some());
        assert!(proto.updated_at.is_some());

        Ok(())
    }
}
