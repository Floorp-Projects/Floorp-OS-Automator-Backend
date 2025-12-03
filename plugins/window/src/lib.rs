// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::{
    permission::{CheckPermissionResult, PluginFunctionPermissions, check_permission},
    plugin::{CorePluginFunction, CorePluginPackage},
    proto::sapphillon::v1::{
        Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
    },
    runtime::OpStateWorkflowData,
};
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use win_control::{Window, WindowManager};
use x_win::{get_active_window, get_open_windows};

pub fn get_active_window_title_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.get_active_window_title".to_string(),
        function_name: "Get Active Window Title".to_string(),
        description: "Gets the title of the currently active window.".to_string(),
        permissions: window_plugin_permissions(),
        arguments: "".to_string(),
        returns: "String: title".to_string(),
    }
}

pub fn get_inactive_window_titles_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.get_inactive_window_titles".to_string(),
        function_name: "Get Inactive Window Titles".to_string(),
        description: "Gets the titles of all inactive windows.".to_string(),
        permissions: window_plugin_permissions(),
        arguments: "".to_string(),
        returns: "Array<String>: titles".to_string(),
    }
}

pub fn minimize_window_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.minimize_window".to_string(),
        function_name: "Minimize Window".to_string(),
        description: "Minimizes a window.".to_string(),
        permissions: window_plugin_permissions(),
        arguments: "String: title".to_string(),
        returns: "".to_string(),
    }
}

pub fn activate_window_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.activate_window".to_string(),
        function_name: "Activate Window".to_string(),
        description: "Activates a window.".to_string(),
        permissions: window_plugin_permissions(),
        arguments: "String: title".to_string(),
        returns: "".to_string(),
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
            minimize_window_plugin_function(),
            activate_window_plugin_function(),
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
        None,
    )
}

pub fn core_get_inactive_window_titles_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.get_inactive_window_titles".to_string(),
        "Get Inactive Window Titles".to_string(),
        "Gets the titles of all inactive windows.".to_string(),
        op2_get_inactive_window_titles(),
        None,
    )
}

pub fn core_minimize_window_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.minimize_window".to_string(),
        "Minimize Window".to_string(),
        "Minimizes a window.".to_string(),
        op2_minimize_window(),
        None,
    )
}

pub fn core_activate_window_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.activate_window".to_string(),
        "Activate Window".to_string(),
        "Activates a window.".to_string(),
        op2_activate_window(),
        None,
    )
}

pub fn core_window_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.window".to_string(),
        "Window".to_string(),
        vec![
            core_get_active_window_title_plugin(),
            core_get_inactive_window_titles_plugin(),
            core_minimize_window_plugin(),
            core_activate_window_plugin(),
        ],
    )
}

fn permission_check(state: &mut OpState) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p,
        None => &vec![],
    };

    let required_permissions = sapphillon_core::permission::Permissions {
        permissions: window_plugin_permissions(),
    };

    let allowed_permissions = {
        let permissions_vec = allowed.clone();
        permissions_vec
            .into_iter()
            .find(|p| {
                p.plugin_function_id == get_active_window_title_plugin_function().function_id
                    || p.plugin_function_id == "*"
            })
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions {
                permissions: vec![],
            })
    };

    let permission_check_result = check_permission(&allowed_permissions, &required_permissions);

    match permission_check_result {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "PermissionDenied. Missing Permissions:",
            perm.to_string(),
        )),
    }
}

#[op2]
#[string]
fn op2_get_active_window_title(state: &mut OpState) -> Result<String, JsErrorBox> {
    permission_check(state)?;
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
    permission_check(state)?;
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

#[op2]
#[cfg(windows)]
fn op2_minimize_window(state: &mut OpState, #[string] title: String) -> Result<(), JsErrorBox> {
    permission_check(state)?;
    let window_manager = WindowManager::new();
    let windows: Vec<Window> = window_manager.get_windows(None).unwrap_or_default();
    for window in windows {
        if window.get_title().unwrap_or_default() == title {
            window.minimize().unwrap();
            return Ok(());
        }
    }
    Err(JsErrorBox::new("Error", "Window not found".to_string()))
}

#[op2]
#[cfg(not(windows))]
fn op2_minimize_window(_state: &mut OpState, _title: String) -> Result<(), JsErrorBox> {
    Err(JsErrorBox::new(
        "Error",
        "Window minimization is only supported on Windows".to_string(),
    ))
}

#[op2]
#[cfg(windows)]
fn op2_activate_window(state: &mut OpState, #[string] title: String) -> Result<(), JsErrorBox> {
    permission_check(state)?;
    let window_manager = WindowManager::new();
    let windows: Vec<Window> = window_manager.get_windows(None).unwrap_or_default();
    for window in windows {
        if window.get_title().unwrap_or_default() == title {
            window.show().unwrap();
            window.focus().unwrap();
            return Ok(());
        }
    }
    Err(JsErrorBox::new("Error", "Window not found".to_string()))
}

#[op2]
#[cfg(not(windows))]
fn op2_activate_window(_state: &mut OpState, _title: String) -> Result<(), JsErrorBox> {
    Err(JsErrorBox::new(
        "Error",
        "Window activation is only supported on Windows".to_string(),
    ))
}

fn window_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Window Management".to_string(),
        description: "Allows the plugin to manage windows.".to_string(),
        permission_type: PermissionType::WindowManagement as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::workflow::CoreWorkflowCode;

    #[test]
    fn test_get_active_window_title_in_workflow() {
        let code = r#"
            const title = get_active_window_title();
            console.log(title);
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: get_active_window_title_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // We can't know the exact title, but we can check that it's a string.
        assert!(workflow.result[0].result.len() > 0);
    }

    #[test]
    fn test_get_inactive_window_titles_in_workflow() {
        let code = r#"
            const titles = get_inactive_window_titles();
            console.log(JSON.stringify(titles));
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: get_inactive_window_titles_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // We can't know the exact titles, but we can check that it's a JSON array.
        assert!(workflow.result[0].result.starts_with('['));
        assert!(workflow.result[0].result.ends_with("]\n"));
    }

    #[test]
    fn test_minimize_window_in_workflow() {
        let code = r#"
            const title = get_active_window_title();
            minimize_window(title);
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: minimize_window_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 0);
    }

    #[test]
    fn test_activate_window_in_workflow() {
        let code = r#"
            const title = get_active_window_title();
            activate_window(title);
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: activate_window_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: window_plugin_permissions(),
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_window_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 0);
    }
}
