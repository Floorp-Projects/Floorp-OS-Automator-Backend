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
use entity::entity::permission as entity_permission;
use entity::entity::workflow_code_allowed_permission;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    QuerySelect,
};

#[allow(dead_code)]
/// Inserts an allowed permission mapping for a workflow code.
///
/// # Arguments
///
/// * `db` - The database connection used for persistence.
/// * `a` - The allowed permission relation to store.
///
/// # Returns
///
/// Returns `Ok(())` when the mapping is saved, or a [`DbErr`] on failure.
pub(crate) async fn create_workflow_code_allowed_permission(
    db: &DatabaseConnection,
    a: workflow_code_allowed_permission::Model,
) -> Result<(), DbErr> {
    let active_model: workflow_code_allowed_permission::ActiveModel = a.into();
    active_model.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
/// Retrieves an allowed permission relation and its optional permission record.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `id` - The relation identifier to fetch.
///
/// # Returns
///
/// Returns `Ok(Some((relation, permission)))` when found, `Ok(None)` when missing, or a [`DbErr`] on failure.
pub(crate) async fn get_workflow_code_allowed_permission(
    db: &DatabaseConnection,
    id: i32,
) -> Result<
    Option<(
        workflow_code_allowed_permission::Model,
        Option<entity_permission::Model>,
    )>,
    DbErr,
> {
    // Find the allowed permission by id and also load the related permission if exists
    let row = workflow_code_allowed_permission::Entity::find_by_id(id)
        .one(db)
        .await?;
    if let Some(r) = row {
        // Try to load related permission
        let perm = r.find_related(entity_permission::Entity).one(db).await?;
        Ok(Some((r, perm)))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
/// Updates an existing allowed permission mapping with new references.
///
/// # Arguments
///
/// * `db` - The database connection to use.
/// * `a` - The relation containing updated fields.
///
/// # Returns
///
/// Returns `Ok(())` after the update attempt, regardless of whether the relation existed.
pub(crate) async fn update_workflow_code_allowed_permission(
    db: &DatabaseConnection,
    a: workflow_code_allowed_permission::Model,
) -> Result<(), DbErr> {
    let existing = workflow_code_allowed_permission::Entity::find_by_id(a.id)
        .one(db)
        .await?;
    if let Some(existing) = existing {
        let mut active_model: workflow_code_allowed_permission::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.workflow_code_id = Set(a.workflow_code_id);
        active_model.permission_id = Set(a.permission_id);
        active_model.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Lists allowed permission mappings, optionally filtered by workflow code ID.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `workflow_code_id` - Optional workflow code identifier to filter results.
/// * `next_page_token` - An optional cursor specifying the next offset.
/// * `page_size` - An optional limit on the number of rows to fetch.
///
/// # Returns
///
/// Returns the matching relations paired with their permissions plus the next page token.
pub(crate) async fn list_workflow_code_allowed_permissions(
    db: &DatabaseConnection,
    workflow_code_id: Option<String>,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<
    (
        Vec<(
            workflow_code_allowed_permission::Model,
            Option<entity_permission::Model>,
        )>,
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

    let mut finder = workflow_code_allowed_permission::Entity::find();
    if let Some(ref wcid) = workflow_code_id {
        finder = finder
            .filter(workflow_code_allowed_permission::Column::WorkflowCodeId.eq(wcid.clone()));
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

    // For each item, attempt to load related permission
    let mut out = Vec::with_capacity(items.len());
    for it in items.into_iter() {
        let perm = it.find_related(entity_permission::Entity).one(db).await?;
        out.push((it, perm));
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
/// Deletes an allowed permission relation by its identifier.
///
/// # Arguments
///
/// * `db` - The database connection used for deletion.
/// * `id` - The relation identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` even when the relation is absent, or a [`DbErr`] if deletion fails.
pub(crate) async fn delete_workflow_code_allowed_permission(
    db: &DatabaseConnection,
    id: i32,
) -> Result<(), DbErr> {
    let found = workflow_code_allowed_permission::Entity::find_by_id(id)
        .one(db)
        .await?;
    if let Some(found) = found {
        let active_model: workflow_code_allowed_permission::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::entity::{permission as entity_permission, workflow_code as entity_wc};
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

    /// Sets up in-memory tables required for workflow code permission tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] prepared for allowed permission CRUD tests.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        // permission table
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

        // workflow_code_allowed_permission table
        let sql_a = r#"
            CREATE TABLE workflow_code_allowed_permission (
                id INTEGER PRIMARY KEY,
                workflow_code_id TEXT NOT NULL,
                permission_id INTEGER NOT NULL
            )
        "#;
        db.execute(Statement::from_string(DbBackend::Sqlite, sql_a.to_string()))
            .await?;

        Ok(db)
    }

    /// Validates allowed permissions can be created after inserting dependencies.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the relation is inserted without error.
    #[tokio::test]
    async fn test_create_allowed_permission() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert permission and workflow_code
        let perm = entity_permission::Model {
            id: 10,
            plugin_function_id: "pf1".to_string(),
            display_name: Some("P".to_string()),
            description: None,
            r#type: 1,
            resource_json: None,
            level: None,
        };
        let active_perm: entity_permission::ActiveModel = perm.into();
        active_perm.insert(&db).await?;

        let wc = entity_wc::Model {
            id: "wcx".to_string(),
            workflow_id: "wfx".to_string(),
            code_revision: 1,
            code: "c".to_string(),
            language: 0,
            created_at: None,
        };
        let active_wc: entity_wc::ActiveModel = wc.into();
        active_wc.insert(&db).await?;

        let a = workflow_code_allowed_permission::Model {
            id: 100,
            workflow_code_id: "wcx".to_string(),
            permission_id: 10,
        };

        create_workflow_code_allowed_permission(&db, a).await?;

        Ok(())
    }

    /// Ensures allowed permissions can be fetched, updated, and listed with related permissions.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying the relation and associated permission remain accessible.
    #[tokio::test]
    async fn test_get_and_update_allowed_permission() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert permission and workflow_code
        let perm = entity_permission::Model {
            id: 10,
            plugin_function_id: "pf1".to_string(),
            display_name: Some("P".to_string()),
            description: None,
            r#type: 1,
            resource_json: None,
            level: None,
        };
        let active_perm: entity_permission::ActiveModel = perm.into();
        active_perm.insert(&db).await?;

        let wc = entity_wc::Model {
            id: "wcx".to_string(),
            workflow_id: "wfx".to_string(),
            code_revision: 1,
            code: "c".to_string(),
            language: 0,
            created_at: None,
        };
        let active_wc: entity_wc::ActiveModel = wc.into();
        active_wc.insert(&db).await?;

        let a = workflow_code_allowed_permission::Model {
            id: 100,
            workflow_code_id: "wcx".to_string(),
            permission_id: 10,
        };

        create_workflow_code_allowed_permission(&db, a.clone()).await?;

        let found = get_workflow_code_allowed_permission(&db, 100).await?;
        assert!(found.is_some());
        let (found_a, found_p) = found.unwrap();
        assert_eq!(found_a.id, 100);
        assert_eq!(found_a.workflow_code_id, "wcx");
        assert!(found_p.is_some());
        let found_p = found_p.unwrap();
        assert_eq!(found_p.id, 10);

        // Update
        let mut updated = found_a.clone();
        updated.permission_id = 10;
        update_workflow_code_allowed_permission(&db, updated).await?;

        let res =
            list_workflow_code_allowed_permissions(&db, Some("wcx".to_string()), None, Some(10))
                .await?;
        assert_eq!(res.0.len(), 1);
        assert_eq!(res.0[0].0.workflow_code_id, "wcx");
        assert!(res.0[0].1.is_some());

        Ok(())
    }

    /// Confirms deleting an allowed permission removes the mapping.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the relation cannot be retrieved anymore.
    #[tokio::test]
    async fn test_delete_allowed_permission() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert permission and workflow_code
        let perm = entity_permission::Model {
            id: 10,
            plugin_function_id: "pf1".to_string(),
            display_name: Some("P".to_string()),
            description: None,
            r#type: 1,
            resource_json: None,
            level: None,
        };
        let active_perm: entity_permission::ActiveModel = perm.into();
        active_perm.insert(&db).await?;

        let wc = entity_wc::Model {
            id: "wcx".to_string(),
            workflow_id: "wfx".to_string(),
            code_revision: 1,
            code: "c".to_string(),
            language: 0,
            created_at: None,
        };
        let active_wc: entity_wc::ActiveModel = wc.into();
        active_wc.insert(&db).await?;

        let a = workflow_code_allowed_permission::Model {
            id: 100,
            workflow_code_id: "wcx".to_string(),
            permission_id: 10,
        };

        create_workflow_code_allowed_permission(&db, a).await?;

        // Delete
        delete_workflow_code_allowed_permission(&db, 100).await?;
        let found = get_workflow_code_allowed_permission(&db, 100).await?;
        assert!(found.is_none());

        Ok(())
    }
}
