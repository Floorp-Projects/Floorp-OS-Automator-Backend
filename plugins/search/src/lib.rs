// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Search plugin for Sapphillon.
//!
//! This plugin provides file search functionality using native OS search APIs:
//! - **Windows**: Windows Search API (Windows Index Search)
//! - **macOS**: Spotlight (MDQuery)
//! - **Linux**: GNOME Tracker, KDE Baloo, or locate
//! - **Fallback**: walkdir-based filesystem traversal

use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{check_permission, CheckPermissionResult};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::sync::{Arc, Mutex, OnceLock};

// Platform-specific modules
#[cfg(target_os = "windows")]
mod windows_search;

#[cfg(target_os = "macos")]
mod macos_search;

#[cfg(target_os = "linux")]
mod linux_search;

mod searcher;
mod walkdir_search;

use searcher::FileSearcher;
use walkdir_search::WalkdirSearcher;

/// Get the best available file searcher for the current platform.
///
/// This function checks for native OS search capabilities and falls back
/// to walkdir-based traversal if no native search is available.
fn get_searcher() -> &'static dyn FileSearcher {
    static SEARCHER: OnceLock<Box<dyn FileSearcher>> = OnceLock::new();

    SEARCHER
        .get_or_init(|| {
            // Try platform-specific searchers first
            #[cfg(target_os = "windows")]
            if let Some(searcher) = windows_search::get_windows_searcher() {
                return searcher;
            }

            #[cfg(target_os = "macos")]
            if let Some(searcher) = macos_search::get_macos_searcher() {
                return searcher;
            }

            #[cfg(target_os = "linux")]
            if let Some(searcher) = linux_search::get_linux_searcher() {
                return searcher;
            }

            // Fallback to walkdir
            Box::new(WalkdirSearcher::new())
        })
        .as_ref()
}

/// Get the name of the currently active searcher for debugging.
pub fn get_active_searcher_name() -> &'static str {
    get_searcher().name()
}

pub fn search_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.search.file".to_string(),
        function_name: "search.file".to_string(),
        description: "Searches for files on the local filesystem using native OS search APIs."
            .to_string(),
        permissions: search_plugin_permissions(),
        arguments: "String: root_path, String: query".to_string(),
        returns: "String: (JSON) list of file paths".to_string(),
    }
}

pub fn search_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.search".to_string(),
        package_name: "Search".to_string(),
        description: "A plugin to search for files on the local filesystem using native OS search APIs (Windows Search/Everything, macOS Spotlight, Linux Tracker/Baloo).".to_string(),
        functions: vec![search_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_search_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        search_plugin_function().function_id,
        "SearchFile".to_string(),
        search_plugin_function().description,
        op2_search_file(),
        Some(include_str!("00_search.js").to_string()),
    )
}

pub fn core_search_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        search_plugin_package().package_id,
        "Search".to_string(),
        vec![core_search_plugin()],
    )
}

fn search_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Execute".to_string(),
        description: "Allows the plugin to execute commands.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

fn permission_check_search(state: &mut OpState) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = data.get_allowed_permissions().clone().unwrap_or_default();

    let required_permissions = sapphillon_core::permission::Permissions {
        permissions: search_plugin_permissions(),
    };

    let allowed_permissions = {
        allowed
            .into_iter()
            .find(|p| {
                p.plugin_function_id == search_plugin_function().function_id
                    || p.plugin_function_id == "*"
            })
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions {
                permissions: vec![],
            })
    };

    match check_permission(&allowed_permissions, &required_permissions) {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "PermissionDenied. Missing Permissions:",
            perm.to_string(),
        )),
    }
}

/// Core search logic using the best available searcher.
fn search_file_logic(root_path: String, query: String) -> Result<String, JsErrorBox> {
    let searcher = get_searcher();
    let results = searcher.search(&root_path, &query)?;
    Ok(serde_json::to_string(&results).unwrap())
}

#[op2]
#[string]
fn op2_search_file(
    state: &mut OpState,
    #[string] root_path: String,
    #[string] query: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check_search(state)?;
    search_file_logic(root_path, query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_file() {
        // Create a temporary directory.
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap().to_string();

        // Create a subdirectory and some files.
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("file1.txt"), "hello").unwrap();
        fs::write(dir.path().join("subdir/file2.log"), "world").unwrap();
        fs::write(dir.path().join("another.file"), "test").unwrap();

        // Use WalkdirSearcher directly since native search APIs don't index temp directories.
        // The OnceLock in get_searcher() would cause cross-test pollution if Windows Search
        // API is selected, which doesn't work well with temporary directories.
        let searcher = WalkdirSearcher::new();

        // Search for a file that exists.
        let results = searcher.search(&dir_path, "file1").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("file1.txt"));

        // Search for a file that doesn't exist.
        let results = searcher.search(&dir_path, "nonexistent").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_active_searcher() {
        // Just verify we can get a searcher name without panicking
        let name = get_active_searcher_name();
        assert!(!name.is_empty());
        println!("Active searcher: {name}");
    }
}
