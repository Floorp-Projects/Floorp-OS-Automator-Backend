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
use entity::entity::workflow_code;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

#[allow(dead_code)]
pub(crate) async fn create_workflow_code(
    db: &DatabaseConnection,
    wc: workflow_code::Model,
) -> Result<(), DbErr> {
    let active_model: workflow_code::ActiveModel = wc.into();
    active_model.insert(db).await?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn get_workflow_code(
    db: &DatabaseConnection,
    id: &str,
) -> Result<Option<workflow_code::Model>, DbErr> {
    let r = workflow_code::Entity::find_by_id(id.to_string())
        .one(db)
        .await?;
    Ok(r)
}

#[allow(dead_code)]
pub(crate) async fn update_workflow_code(
    db: &DatabaseConnection,
    wc: workflow_code::Model,
) -> Result<(), DbErr> {
    let existing = get_workflow_code(db, &wc.id).await?;
    if let Some(existing) = existing {
        let mut active_model: workflow_code::ActiveModel = existing.into();
        use sea_orm::ActiveValue::Set;
        active_model.workflow_id = Set(wc.workflow_id);
        active_model.code_revision = Set(wc.code_revision);
        active_model.code = Set(wc.code);
        active_model.language = Set(wc.language);
        active_model.update(db).await?;
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn list_workflow_codes(
    db: &DatabaseConnection,
    next_page_token: Option<String>,
    page_size: Option<u32>,
) -> Result<(Vec<workflow_code::Model>, String), DbErr> {
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
    let mut items = workflow_code::Entity::find()
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
pub(crate) async fn delete_workflow_code(db: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let found = workflow_code::Entity::find_by_id(id.to_string())
        .one(db)
        .await?;
    if let Some(found) = found {
        let active_model: workflow_code::ActiveModel = found.into();
        active_model.delete(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::entity::workflow as entity_wf;
    use sea_orm::{
        ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait, Statement,
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

        Ok(db)
    }

    #[tokio::test]
    async fn test_create_workflow_code() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert referenced workflow
        let wf = entity_wf::Model {
            id: "wf1".to_string(),
            display_name: "WF".to_string(),
            description: None,
            workflow_language: 0,
            created_at: None,
            updated_at: None,
        };
        let active_wf: entity_wf::ActiveModel = wf.into();
        active_wf.insert(&db).await?;

        let wc = workflow_code::Model {
            id: "wc1".to_string(),
            workflow_id: "wf1".to_string(),
            code_revision: 1,
            code: "print('hi')".to_string(),
            language: 0,
            created_at: None,
        };

        create_workflow_code(&db, wc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_workflow_code_and_update() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert referenced workflow
        let wf = entity_wf::Model {
            id: "wf1".to_string(),
            display_name: "WF".to_string(),
            description: None,
            workflow_language: 0,
            created_at: None,
            updated_at: None,
        };
        let active_wf: entity_wf::ActiveModel = wf.into();
        active_wf.insert(&db).await?;

        let wc = workflow_code::Model {
            id: "wc1".to_string(),
            workflow_id: "wf1".to_string(),
            code_revision: 1,
            code: "print('hi')".to_string(),
            language: 0,
            created_at: None,
        };

        create_workflow_code(&db, wc.clone()).await?;

        let found = get_workflow_code(&db, "wc1").await?;
        assert!(found.is_some());
        let mut found = found.unwrap();
        assert_eq!(found.id, "wc1");
        assert_eq!(found.workflow_id, "wf1");
        assert_eq!(found.code_revision, 1);

        // Update
        found.code = "print('bye')".to_string();
        found.code_revision = 2;
        update_workflow_code(&db, found).await?;

        let found = get_workflow_code(&db, "wc1").await?;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.code, "print('bye')");
        assert_eq!(found.code_revision, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_workflow_code() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert referenced workflow
        let wf = entity_wf::Model {
            id: "wf1".to_string(),
            display_name: "WF".to_string(),
            description: None,
            workflow_language: 0,
            created_at: None,
            updated_at: None,
        };
        let active_wf: entity_wf::ActiveModel = wf.into();
        active_wf.insert(&db).await?;

        let wc = workflow_code::Model {
            id: "wc1".to_string(),
            workflow_id: "wf1".to_string(),
            code_revision: 1,
            code: "print('hi')".to_string(),
            language: 0,
            created_at: None,
        };

        create_workflow_code(&db, wc).await?;

        delete_workflow_code(&db, "wc1").await?;
        let found = get_workflow_code(&db, "wc1").await?;
        assert!(found.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_list_workflow_codes_pagination() -> Result<(), DbErr> {
        let db = setup_db().await?;

        // Insert referenced workflow
        let wf = entity_wf::Model {
            id: "wf2".to_string(),
            display_name: "WF2".to_string(),
            description: None,
            workflow_language: 0,
            created_at: None,
            updated_at: None,
        };
        let active_wf: entity_wf::ActiveModel = wf.into();
        active_wf.insert(&db).await?;

        for i in 1..=5 {
            let wc = workflow_code::Model {
                id: format!("wc{i}"),
                workflow_id: "wf2".to_string(),
                code_revision: 1,
                code: "c".to_string(),
                language: 0,
                created_at: None,
            };
            create_workflow_code(&db, wc).await?;
        }

        let mut token: Option<String> = None;
        let mut collected = std::collections::HashSet::new();
        loop {
            let (items, next) = list_workflow_codes(&db, token.clone(), Some(2)).await?;
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
}
