// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Debug workflow feature - only active in debug builds.
//!
//! Periodically scans `debug_workflow` directory for JS files and registers them
//! as workflows with full permissions.

use std::fs;
use std::path::Path;

use anyhow::Result;
use log::{debug, info, warn};
use tokio::time::{Duration, interval};

use sapphillon_core::permission::{Permissions, PluginFunctionPermissions};
use sapphillon_core::proto::sapphillon::v1::{Permission, PermissionLevel, PermissionType};
use sapphillon_core::workflow::CoreWorkflowCode;

use crate::sysconfig::sysconfig;

/// Directory name (relative path from execution directory)
const DEBUG_WORKFLOW_DIR: &str = "debug_workflow";
/// Scan interval in seconds
const SCAN_INTERVAL_SECS: u64 = 10;

/// Creates all-encompassing permissions that grant access to everything.
///
/// # Returns
///
/// Returns a vector of `PluginFunctionPermissions` with wildcard access to all plugins.
/// Uses `*` as plugin_function_id and `PermissionType::Unspecified` to allow all operations.
pub fn create_all_permissions() -> Vec<PluginFunctionPermissions> {
    vec![PluginFunctionPermissions {
        plugin_function_id: "*".to_string(), // Wildcard - all plugins
        permissions: Permissions::new(vec![Permission {
            display_name: "All Permissions".to_string(),
            description: "Full access for debug workflows - allows all operations".to_string(),
            permission_type: PermissionType::Unspecified as i32, // Unspecified = allow all
            permission_level: PermissionLevel::Unspecified as i32,
            resource: vec!["*".to_string()],
        }]),
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
        debug!(
            "Debug workflow directory does not exist: {}",
            DEBUG_WORKFLOW_DIR
        );
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

/// Runs a debug workflow with full permissions.
///
/// # Arguments
///
/// * `workflow` - The debug workflow file to execute
/// * `handle` - Tokio runtime handle for async operations
///
/// # Returns
///
/// Returns a vector of `WorkflowResult` containing the execution results.
pub fn run_debug_workflow(
    workflow: &DebugWorkflowFile,
    handle: tokio::runtime::Handle,
) -> Vec<sapphillon_core::proto::sapphillon::v1::WorkflowResult> {
    let permissions = create_all_permissions();
    let core_plugins = sysconfig().core_plugin_package;

    let mut workflow_code = CoreWorkflowCode::new(
        workflow.name.clone(),
        workflow.code.clone(),
        core_plugins,
        1,
        permissions.clone(),
        permissions,
    );

    workflow_code.run(handle);
    workflow_code.result
}

/// Starts the periodic debug workflow scanner.
///
/// This function runs in a loop, scanning the debug_workflow directory every
/// `SCAN_INTERVAL_SECS` seconds and executing any workflows found.
pub async fn start_debug_workflow_scanner() {
    info!(
        "[DEBUG] Starting debug workflow scanner (interval: {}s)",
        SCAN_INTERVAL_SECS
    );

    let mut scanner_interval = interval(Duration::from_secs(SCAN_INTERVAL_SECS));
    let handle = tokio::runtime::Handle::current();

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
                    info!(
                        "[DEBUG] Running debug workflow: {} ({})",
                        workflow.name, workflow.path
                    );

                    let results = run_debug_workflow(&workflow, handle.clone());

                    for result in results {
                        info!(
                            "[DEBUG] Workflow '{}' result: {}",
                            workflow.name, result.result
                        );
                    }
                }
            }
            Err(e) => {
                warn!("[DEBUG] Failed to scan debug_workflow directory: {}", e);
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
        assert!(!permissions[0].permissions.permissions.is_empty());
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
}
