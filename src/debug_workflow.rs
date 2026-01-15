// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Debug workflow feature - only active in debug builds.
//!
//! Periodically scans `debug_workflow` directory for JS files and registers them
//! to the database with full permissions.

use std::fs;
use std::path::Path;

use anyhow::Result;
use log::{debug, info, warn};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use tokio::time::{Duration, interval};

use sapphillon_core::proto::sapphillon::v1::{
    AllowedPermission, Permission, PermissionLevel, PermissionType,
};

use crate::GLOBAL_STATE;

/// Directory name (relative path from execution directory)
const DEBUG_WORKFLOW_DIR: &str = "debug_workflow";
/// Scan interval in seconds
const SCAN_INTERVAL_SECS: u64 = 5;
/// Workflow language constant for JavaScript
const WORKFLOW_LANGUAGE_JS: i32 = 2;

fn should_skip_update(
    latest_code: Option<&entity::entity::workflow_code::Model>,
    new_code: &str,
) -> bool {
    latest_code
        .map(|code| code.code.as_str() == new_code)
        .unwrap_or(false)
}

fn resolve_workflow_code_id_and_revision<F>(
    latest_code: Option<&entity::entity::workflow_code::Model>,
    new_id: F,
) -> (String, i32)
where
    F: FnOnce() -> String,
{
    if let Some(code) = latest_code {
        (code.id.clone(), code.code_revision)
    } else {
        (new_id(), 1)
    }
}

/// Creates all-encompassing permissions that grant access to everything.
///
/// # Returns
///
/// Returns a vector of `AllowedPermission` with wildcard access to all plugins.
/// Uses `*` as plugin_function_id and `PermissionType::Unspecified` to allow all operations.
pub fn create_all_permissions() -> Vec<AllowedPermission> {
    vec![AllowedPermission {
        plugin_function_id: "*".to_string(), // Wildcard - all plugins
        permissions: vec![Permission {
            display_name: "All Permissions".to_string(),
            description: "Full access for debug workflows - allows all operations".to_string(),
            permission_type: PermissionType::Unspecified as i32, // Unspecified = allow all
            permission_level: PermissionLevel::Unspecified as i32,
            resource: vec!["*".to_string()],
        }],
    }]
}

/// Represents a debug workflow file found in the debug_workflow directory.
#[derive(Debug, Clone)]
pub struct DebugWorkflowFile {
    pub name: String,
    pub path: String,
    pub code: String,
}

/// Scans the debug_workflow directory for JS files.
///
/// # Returns
///
/// Returns a vector of `DebugWorkflowFile` representing each JS file found,
/// or an error if directory reading fails.
pub fn scan_debug_workflow_dir() -> Result<Vec<DebugWorkflowFile>> {
    let dir_path = Path::new(DEBUG_WORKFLOW_DIR);

    if !dir_path.exists() {
        debug!("Debug workflow directory does not exist: {DEBUG_WORKFLOW_DIR}");
        return Ok(vec![]);
    }

    let mut workflows = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("js") {
            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let code = fs::read_to_string(&path)?;

            workflows.push(DebugWorkflowFile {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                code,
            });
        }
    }

    Ok(workflows)
}

/// Registers a debug workflow to the database with full permissions.
///
/// # Arguments
///
/// * `workflow` - The debug workflow file to register
///
/// # Returns
///
/// Returns Ok(()) on success, or an error if database operations fail.
pub async fn register_debug_workflow(workflow: &DebugWorkflowFile) -> Result<()> {
    use database::workflow::update_workflow_from_proto;
    use entity::entity::workflow_code as workflow_code_entity;
    use sapphillon_core::proto::sapphillon::v1::{Workflow, WorkflowCode};

    let db = GLOBAL_STATE.get_db_connection().await?;

    // Check if workflow with this display_name already exists
    let display_name = format!("[DEBUG] {}", workflow.name);
    let existing = entity::entity::workflow::Entity::find()
        .filter(entity::entity::workflow::Column::DisplayName.eq(&display_name))
        .one(&db)
        .await?;

    let now = chrono::Utc::now();
    let now_ts = sapphillon_core::proto::google::protobuf::Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    };

    let (workflow_id, created_at_ts, latest_code) = if let Some(existing) = existing {
        let created_at = existing.created_at.unwrap_or(now);
        let created_at_ts = sapphillon_core::proto::google::protobuf::Timestamp {
            seconds: created_at.timestamp(),
            nanos: created_at.timestamp_subsec_nanos() as i32,
        };

        let latest_code = workflow_code_entity::Entity::find()
            .filter(workflow_code_entity::Column::WorkflowId.eq(existing.id.clone()))
            .order_by_desc(workflow_code_entity::Column::CodeRevision)
            .one(&db)
            .await?;

        (existing.id, created_at_ts, latest_code)
    } else {
        (uuid::Uuid::new_v4().to_string(), now_ts, None)
    };

    if should_skip_update(latest_code.as_ref(), &workflow.code) {
        debug!("[DEBUG] Workflow already up to date: {display_name} - skipping");
        return Ok(());
    }

    let (workflow_code_id, code_revision) =
        resolve_workflow_code_id_and_revision(latest_code.as_ref(), || {
            uuid::Uuid::new_v4().to_string()
        });

    info!("[DEBUG] Registering debug workflow: {display_name}");
    let permissions = create_all_permissions();

    // Build the Workflow proto with WorkflowCode containing allowed_permissions
    let wf_proto = Workflow {
        id: workflow_id,
        display_name: display_name.clone(),
        description: format!("Debug workflow loaded from: {}", workflow.path),
        workflow_language: WORKFLOW_LANGUAGE_JS,
        workflow_code: vec![WorkflowCode {
            id: workflow_code_id,
            code_revision,
            code: workflow.code.clone(),
            language: WORKFLOW_LANGUAGE_JS,
            created_at: Some(now_ts),
            result: vec![],
            plugin_packages: vec![],
            plugin_function_ids: vec!["*".to_string()],
            allowed_permissions: permissions,
        }],
        created_at: Some(created_at_ts),
        updated_at: Some(now_ts),
        workflow_results: vec![],
    };

    update_workflow_from_proto(&db, &wf_proto).await?;

    info!("[DEBUG] Successfully registered debug workflow: {display_name}");

    Ok(())
}

/// Starts the periodic debug workflow scanner.
///
/// This function runs in a loop, scanning the debug_workflow directory every
/// `SCAN_INTERVAL_SECS` seconds and registering any new workflows to the database.
pub async fn start_debug_workflow_scanner() {
    info!("[DEBUG] Starting debug workflow scanner (interval: {SCAN_INTERVAL_SECS}s)");

    let mut scanner_interval = interval(Duration::from_secs(SCAN_INTERVAL_SECS));

    loop {
        scanner_interval.tick().await;

        debug!("[DEBUG] Scanning debug_workflow directory...");

        match scan_debug_workflow_dir() {
            Ok(workflows) => {
                if workflows.is_empty() {
                    debug!("[DEBUG] No debug workflows found");
                    continue;
                }

                for workflow in workflows {
                    if let Err(e) = register_debug_workflow(&workflow).await {
                        warn!(
                            "[DEBUG] Failed to register workflow '{workflow_name}': {error}",
                            workflow_name = workflow.name,
                            error = e
                        );
                    }
                }
            }
            Err(e) => {
                warn!("[DEBUG] Failed to scan debug_workflow directory: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_all_permissions() {
        let permissions = create_all_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].plugin_function_id, "*");
        assert!(!permissions[0].permissions.is_empty());
    }

    #[test]
    fn test_scan_debug_workflow_dir_empty() {
        // When directory doesn't exist, should return empty vec
        // Note: This test assumes the debug_workflow directory doesn't exist in the test environment
        let result = scan_debug_workflow_dir();
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_debug_workflow_dir_with_files() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let debug_dir = temp_dir.path().join("debug_workflow");
        fs::create_dir(&debug_dir).unwrap();

        // Create a test JS file
        let test_file = debug_dir.join("test_workflow.js");
        fs::write(&test_file, "console.log('test');").unwrap();

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = scan_debug_workflow_dir();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let workflows = result.unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "test_workflow");
        assert_eq!(workflows[0].code, "console.log('test');");
    }

    #[test]
    fn test_debug_workflow_file_structure() {
        let workflow = DebugWorkflowFile {
            name: "test".to_string(),
            path: "/path/to/test.js".to_string(),
            code: "console.log('hello');".to_string(),
        };

        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.path, "/path/to/test.js");
        assert_eq!(workflow.code, "console.log('hello');");
    }

    #[test]
    fn test_scan_ignores_non_js_files() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let debug_dir = temp_dir.path().join("debug_workflow");
        fs::create_dir(&debug_dir).unwrap();

        // Create both JS and non-JS files
        fs::write(debug_dir.join("workflow.js"), "console.log('js');").unwrap();
        fs::write(debug_dir.join("readme.txt"), "This is a readme").unwrap();
        fs::write(debug_dir.join("config.json"), "{}").unwrap();

        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = scan_debug_workflow_dir();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let workflows = result.unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "workflow");
    }

    #[test]
    fn test_should_skip_update_when_same_code() {
        let code = entity::entity::workflow_code::Model {
            id: "code-1".to_string(),
            workflow_id: "wf-1".to_string(),
            code_revision: 1,
            code: "console.log('same');".to_string(),
            language: WORKFLOW_LANGUAGE_JS,
            created_at: None,
        };

        assert!(should_skip_update(Some(&code), "console.log('same');"));
        assert!(!should_skip_update(
            Some(&code),
            "console.log('different');"
        ));
        assert!(!should_skip_update(None, "console.log('same');"));
    }

    #[test]
    fn test_resolve_workflow_code_id_and_revision() {
        let code = entity::entity::workflow_code::Model {
            id: "code-1".to_string(),
            workflow_id: "wf-1".to_string(),
            code_revision: 4,
            code: "console.log('v4');".to_string(),
            language: WORKFLOW_LANGUAGE_JS,
            created_at: None,
        };

        let (id, rev) = resolve_workflow_code_id_and_revision(Some(&code), || "new".to_string());
        assert_eq!(id, "code-1");
        assert_eq!(rev, 4);

        let (id, rev) = resolve_workflow_code_id_and_revision(None, || "new-id".to_string());
        assert_eq!(id, "new-id");
        assert_eq!(rev, 1);
    }
}
