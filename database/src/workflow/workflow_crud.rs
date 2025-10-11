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
use entity::entity::workflow;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

#[allow(dead_code)]
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
pub(crate) async fn get_workflow(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<workflow::Model>, DbErr> {
    let w = workflow::Entity::find_by_id(id.to_string()).one(db).await?;
    Ok(w)
}

#[allow(dead_code)]
pub(crate) async fn update_workflow(db: &DatabaseConnection, wf: workflow::Model) -> Result<(), DbErr> {
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
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement};

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

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

    #[tokio::test]
    async fn test_create_and_get_workflow() -> Result<(), DbErr> {
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

    #[tokio::test]
    async fn test_update_and_delete_workflow() -> Result<(), DbErr> {
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

        delete_workflow(&db, "u1").await?;
        let found = get_workflow(&db, "u1").await?;
        assert!(found.is_none());

        Ok(())
    }
}
