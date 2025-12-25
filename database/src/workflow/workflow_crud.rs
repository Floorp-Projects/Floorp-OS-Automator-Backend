// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use base64::Engine as _;
use base64::engine::general_purpose;
use entity::entity::workflow;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

#[allow(dead_code)]
/// Inserts a workflow record into the database.
///
/// # Arguments
///
/// * `db` - The database connection to use for insertion.
/// * `wf` - The workflow model to persist.
///
/// # Returns
///
/// Returns `Ok(())` when creation succeeds, or a [`DbErr`] on failure.
pub(crate) async fn create_workflow(
    db: &DatabaseConnection,
    wf: workflow::Model,
) -> Result<(), DbErr> {
    let active_model: workflow::ActiveModel = wf.into();
    // Insert to ensure creation
    active_model.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
/// Retrieves a workflow by its identifier.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `id` - The workflow identifier to fetch.
///
/// # Returns
///
/// Returns `Ok(Some(workflow))` when found, `Ok(None)` when missing, or a [`DbErr`] on failure.
pub(crate) async fn get_workflow(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<workflow::Model>, DbErr> {
    let w = workflow::Entity::find_by_id(id.to_string()).one(db).await?;
    Ok(w)
}

#[allow(dead_code)]
/// Updates an existing workflow's metadata when present.
///
/// # Arguments
///
/// * `db` - The database connection used for persistence.
/// * `wf` - The workflow data containing updated fields.
///
/// # Returns
///
/// Returns `Ok(())` after attempting the update, even if the workflow was absent.
pub(crate) async fn update_workflow(
    db: &DatabaseConnection,
    wf: workflow::Model,
) -> Result<(), DbErr> {
    let existing = get_workflow(db, &wf.id).await?;
    if let Some(existing) = existing {
        let mut active_model: workflow::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.display_name = Set(wf.display_name);
        active_model.description = Set(wf.description);
        active_model.workflow_language = Set(wf.workflow_language);
        active_model.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Lists workflows with base64-encoded offset pagination.
///
/// # Arguments
///
/// * `db` - The database connection to query.
/// * `next_page_token` - An optional cursor indicating the next offset.
/// * `page_size` - An optional limit on the number of workflows to fetch.
///
/// # Returns
///
/// Returns the retrieved workflows and the next page token (empty when no further results exist).
pub(crate) async fn list_workflows(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<workflow::Model>, String), DbErr> {
    // Pagination by base64-encoded big-endian u64 offset
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
    let mut items = workflow::Entity::find()
        .offset(Some(offset))
        .limit(Some(query_limit))
        .all(db)
        .await?;

    let has_next = (items.len() as u64) > limit;
    if has_next {
        items.truncate(limit as usize);
    }

    let next_token = if has_next {
        let next_offset = offset.saturating_add(limit);
        let bytes = next_offset.to_be_bytes();
        general_purpose::STANDARD.encode(bytes)
    } else {
        String::new()
    };

    Ok((items, next_token))
}

#[allow(dead_code)]
/// Deletes a workflow by its identifier if it exists.
///
/// # Arguments
///
/// * `db` - The database connection used for deletion.
/// * `id` - The workflow identifier to remove.
///
/// # Returns
///
/// Returns `Ok(())` even if the workflow is absent, or a [`DbErr`] if deletion fails.
pub(crate) async fn delete_workflow(db: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let found = workflow::Entity::find_by_id(id.to_string()).one(db).await?;
    if let Some(found) = found {
        let active_model: workflow::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::entity::workflow as entity_wf;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

    /// Creates an in-memory workflow table used by the unit tests.
    ///
    /// # Arguments
    ///
    /// This helper takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns a [`DatabaseConnection`] prepared for workflow CRUD tests.
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let state = crate::global_state_for_tests!();
        let db = state.get_db_connection().await?;

        let sql = r#"
            CREATE TABLE workflow (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                description TEXT,
                workflow_language INTEGER NOT NULL,
                created_at TEXT,
                updated_at TEXT
            )
        "#;
        db.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await?;

        Ok(db)
    }

    /// Ensures workflows can be inserted successfully.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the workflow creation helper completes without error.
    #[tokio::test]
    async fn test_create_workflow() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let w = entity_wf::Model {
            id: "w1".to_string(),
            display_name: "Workflow One".to_string(),
            description: Some("desc".to_string()),
            workflow_language: 1,
            created_at: None,
            updated_at: None,
        };

        // create should succeed
        create_workflow(&db, w).await?;
        Ok(())
    }

    /// Validates workflows can be retrieved after insertion.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after confirming the stored workflow matches expected values.
    #[tokio::test]
    async fn test_get_workflow() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let w = entity_wf::Model {
            id: "w1".to_string(),
            display_name: "Workflow One".to_string(),
            description: Some("desc".to_string()),
            workflow_language: 1,
            created_at: None,
            updated_at: None,
        };

        create_workflow(&db, w).await?;

        let found = get_workflow(&db, "w1").await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "w1");
        assert_eq!(found.display_name, "Workflow One");
        assert_eq!(found.description.as_deref(), Some("desc"));
        assert_eq!(found.workflow_language, 1);

        Ok(())
    }
    /// Verifies pagination iterates over all workflows.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once all inserted workflow identifiers have been collected.
    #[tokio::test]
    async fn test_list_workflows_pagination() -> Result<(), DbErr> {
        let db = setup_db().await?;

        for i in 1..=5 {
            let w = entity_wf::Model {
                id: format!("wf{i}"),
                display_name: format!("WF {i}"),
                description: None,
                workflow_language: 0,
                created_at: None,
                updated_at: None,
            };
            create_workflow(&db, w).await?;
        }

        let mut token: Option<String> = None;
        let mut collected = std::collections::HashSet::new();
        loop {
            let (items, next) = list_workflows(&db, token.clone(), Some(2)).await?;
            for it in items.iter() {
                collected.insert(it.id.clone());
            }
            if next.is_empty() {
                break;
            }
            token = Some(next);
        }

        assert_eq!(collected.len(), 5);
        Ok(())
    }

    /// Confirms updates persist new workflow metadata.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after verifying fields are updated as expected.
    #[tokio::test]
    async fn test_update_workflow() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let initial = entity_wf::Model {
            id: "u1".to_string(),
            display_name: "Before".to_string(),
            description: Some("d".to_string()),
            workflow_language: 2,
            created_at: None,
            updated_at: None,
        };
        create_workflow(&db, initial).await?;

        let updated = entity_wf::Model {
            id: "u1".to_string(),
            display_name: "After".to_string(),
            description: None,
            workflow_language: 3,
            created_at: None,
            updated_at: None,
        };

        update_workflow(&db, updated).await?;

        let found = get_workflow(&db, "u1").await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.display_name, "After");
        assert!(found.description.is_none());
        assert_eq!(found.workflow_language, 3);

        Ok(())
    }

    /// Ensures workflows can be deleted and no longer retrieved.
    ///
    /// # Arguments
    ///
    /// This asynchronous test takes no arguments.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` once the workflow has been removed.
    #[tokio::test]
    async fn test_delete_workflow() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let initial = entity_wf::Model {
            id: "u1".to_string(),
            display_name: "Before".to_string(),
            description: Some("d".to_string()),
            workflow_language: 2,
            created_at: None,
            updated_at: None,
        };
        create_workflow(&db, initial).await?;

        delete_workflow(&db, "u1").await?;
        let found = get_workflow(&db, "u1").await?;
        assert!(found.is_none());

        Ok(())
    }
}
