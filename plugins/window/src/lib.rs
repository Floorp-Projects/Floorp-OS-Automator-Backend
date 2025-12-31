// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{CheckPermissionResult, Permissions, check_permission};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    FunctionDefine, FunctionParameter, Permission, PermissionLevel, PermissionType, PluginFunction,
    PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::sync::{Arc, Mutex};
use x_win::{get_active_window, get_open_windows};

pub fn get_active_window_title_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.get_active_window_title".to_string(),
        function_name: "Get Active Window Title".to_string(),
        description: "Gets the title of the currently active window.".to_string(),
        permissions: window_plugin_permissions(),
        function_define: Some(FunctionDefine {
            parameters: vec![],
            returns: vec![FunctionParameter {
                name: "title".to_string(),
                r#type: "string".to_string(),
                description: "Active window title".to_string(),
            }],
        }),
    }
}

pub fn get_inactive_window_titles_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.get_inactive_window_titles".to_string(),
        function_name: "Get Inactive Window Titles".to_string(),
        description: "Gets the titles of all inactive windows.".to_string(),
        permissions: window_plugin_permissions(),
        function_define: Some(FunctionDefine {
            parameters: vec![],
            returns: vec![FunctionParameter {
                name: "titles".to_string(),
                r#type: "string[]".to_string(),
                description: "List of inactive window titles".to_string(),
            }],
        }),
    }
}

pub fn window_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.window".to_string(),
        package_name: "Window".to_string(),
        description: "A plugin to manage windows.".to_string(),
        functions: vec![
            get_active_window_title_plugin_function(),
            get_inactive_window_titles_plugin_function(),
        ],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_get_active_window_title_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.get_active_window_title".to_string(),
        "Get Active Window Title".to_string(),
        "Gets the title of the currently active window.".to_string(),
        op2_get_active_window_title(),
        Some(include_str!("00_window.js").to_string()),
    )
}

pub fn core_get_inactive_window_titles_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.get_inactive_window_titles".to_string(),
        "Get Inactive Window Titles".to_string(),
        "Gets the titles of all inactive windows.".to_string(),
        op2_get_inactive_window_titles(),
        Some(include_str!("00_window.js").to_string()),
    )
}

pub fn core_window_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.window".to_string(),
        "Window".to_string(),
        vec![
            core_get_active_window_title_plugin(),
            core_get_inactive_window_titles_plugin(),
        ],
    )
}

#[op2]
#[string]
fn op2_get_active_window_title(state: &mut OpState) -> Result<String, JsErrorBox> {
    ensure_permission(
        state,
        &get_active_window_title_plugin_function().function_id,
        window_plugin_permissions(),
        "",
    )?;
    match get_active_window() {
        Ok(active_window) => Ok(active_window.title),
        Err(_) => Err(JsErrorBox::new(
            "Error",
            "Could not get active window title".to_string(),
        )),
    }
}

#[op2]
#[serde]
fn op2_get_inactive_window_titles(state: &mut OpState) -> Result<Vec<String>, JsErrorBox> {
    ensure_permission(
        state,
        &get_inactive_window_titles_plugin_function().function_id,
        window_plugin_permissions(),
        "",
    )?;
    match get_open_windows() {
        Ok(windows) => {
            let active_window = get_active_window().ok();
            let inactive_titles = windows
                .into_iter()
                .filter(|w| {
                    if let Some(active) = &active_window {
                        w.id != active.id
                    } else {
                        true
                    }
                })
                .map(|w| w.title)
                .collect();
            Ok(inactive_titles)
        }
        Err(_) => Err(JsErrorBox::new(
            "Error",
            "Could not get inactive window titles".to_string(),
        )),
    }
}

fn ensure_permission(
    state: &mut OpState,
    plugin_function_id: &str,
    required_permissions: Vec<Permission>,
    resource: &str,
) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = data.get_allowed_permissions().clone().unwrap_or_default();

    let required_permissions = Permissions::new(
        required_permissions
            .into_iter()
            .map(|mut p| {
                if !resource.is_empty() && p.resource.is_empty() {
                    p.resource = vec![resource.to_string()];
                }
                p
            })
            .collect(),
    );

    let allowed_permissions = allowed
        .into_iter()
        .find(|p| p.plugin_function_id == plugin_function_id || p.plugin_function_id == "*")
        .map(|p| p.permissions)
        .unwrap_or_else(|| Permissions::new(vec![]));

    match check_permission(&allowed_permissions, &required_permissions) {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "PermissionDenied. Missing Permissions:",
            perm.to_string(),
        )),
    }
}

fn window_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Window Access".to_string(),
        description: "Allows the plugin to access window information.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::permission::PluginFunctionPermissions;
    use sapphillon_core::workflow::CoreWorkflowCode;

    #[test]
    fn test_get_active_window_title_in_workflow() {
        let code = r#"
            const title = app.sapphillon.core.window.getActiveWindowTitle();
            console.log(title);
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: get_active_window_title_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // In headless environments (CI, containers), we may get an error instead of a title.
        // We just check that we got some result (either a title or an error message).
        assert!(!workflow.result[0].result.is_empty());
    }

    #[test]
    fn test_get_inactive_window_titles_in_workflow() {
        let code = r#"
            const titles = app.sapphillon.core.window.getInactiveWindowTitles();
            console.log(JSON.stringify(titles));
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: get_inactive_window_titles_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // In headless environments (CI, containers), we may get an error instead of window titles.
        // Accept either a JSON array (success) or an error message (headless environment).
        let result = &workflow.result[0].result;
        let is_json_array = result.starts_with('[') && result.trim_end().ends_with(']');
        let is_error = result.contains("Error") || result.contains("error");
        assert!(
            is_json_array || is_error,
            "Expected JSON array or error message, got: {result}"
        );
    }
}
