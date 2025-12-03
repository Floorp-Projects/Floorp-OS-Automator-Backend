// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{
    check_permission, CheckPermissionResult, PluginFunctionPermissions,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub fn search_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.search.file".to_string(),
        function_name: "search.file".to_string(),
        description: "Searches for files on the local filesystem.".to_string(),
        permissions: search_plugin_permissions(),
        arguments: "String: root_path, String: query".to_string(),
        returns: "String: (JSON) list of file paths".to_string(),
    }
}

pub fn search_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.search".to_string(),
        package_name: "Search".to_string(),
        description: "A plugin to search for files on the local filesystem.".to_string(),
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
    let allowed = data.get_allowed_permissions().cloned().unwrap_or_default();

    let mut perm = search_plugin_permissions();
    let required_permissions = sapphillon_core::permission::Permissions { permissions: perm };

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

#[op2]
#[string]
fn op2_search_file(
    state: &mut OpState,
    #[string] root_path: String,
    #[string] query: String,
) -> std::result::Result<String, JsErrorBox> {
    #[cfg(not(test))]
    permission_check_search(state)?;

    let results: Vec<String> = WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains(&query))
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect();

    Ok(serde_json::to_string(&results).unwrap())
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

        let mut state = OpState::new(0);

        // Search for a file that exists.
        let result = op2_search_file(&mut state, dir_path.clone(), "file1".to_string()).unwrap();
        let results: Vec<String> = serde_json::from_str(&result).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("file1.txt"));

        // Search for a file that doesn't exist.
        let result = op2_search_file(&mut state, dir_path, "nonexistent".to_string()).unwrap();
        let results: Vec<String> = serde_json::from_str(&result).unwrap();
        assert_eq!(results.len(), 0);
    }
}
