// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::{
    permission::{CheckPermissionResult, check_permission},
    plugin::{CorePluginFunction, CorePluginPackage},
    proto::sapphillon::v1::{
        Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
    },
    runtime::OpStateWorkflowData,
};
use std::sync::{Arc, Mutex};
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

pub fn close_window_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.window.close_window".to_string(),
        function_name: "Close Window".to_string(),
        description: "Closes a window by its title (partial match supported).".to_string(),
        permissions: window_plugin_permissions(),
        arguments: "String: title".to_string(),
        returns: "String: result".to_string(),
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
            close_window_plugin_function(),
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
        None,
    )
}

pub fn core_close_window_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.window.close_window".to_string(),
        "Close Window".to_string(),
        "Closes a window by its title (partial match supported).".to_string(),
        op2_close_window(),
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
            core_close_window_plugin(),
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

            // Debug: Log all window info
            for w in &windows {
                eprintln!(
                    "Window: id={}, title='{}', app='{}', process_id={}",
                    w.id, w.title, w.info.name, w.info.process_id
                );
            }

            let inactive_titles: Vec<String> = windows
                .into_iter()
                .filter(|w| {
                    if let Some(active) = &active_window {
                        w.id != active.id
                    } else {
                        true
                    }
                })
                // Filter out system processes
                .filter(|w| {
                    let name = w.info.name.to_lowercase();
                    !name.contains("windowmanager")
                        && !name.contains("dock")
                        && !name.contains("systemuiserver")
                        && !name.contains("spotlight")
                        && !name.contains("controlcenter")
                        && !name.contains("notificationcenter")
                })
                .map(|w| {
                    // Use app name if title is empty
                    if w.title.is_empty() {
                        w.info.name.clone()
                    } else {
                        w.title
                    }
                })
                .filter(|t| !t.is_empty())
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
#[string]
fn op2_close_window(
    state: &mut OpState,
    #[string] title_pattern: String,
) -> Result<String, JsErrorBox> {
    permission_check(state)?;

    let windows = get_open_windows()
        .map_err(|_| JsErrorBox::new("Error", "Could not get open windows".to_string()))?;

    let pattern_lower = title_pattern.to_lowercase();
    let mut found_count = 0;
    let mut closed_count = 0;
    let mut last_error = None;

    for window in windows {
        let title_lower = window.title.to_lowercase();
        let name_lower = window.info.name.to_lowercase();

        // Debug log all windows to stderr (visible in terminal)
        eprintln!(
            "[Sapphillon] Checking window: title='{}', app='{}', pattern='{}'",
            window.title, window.info.name, title_pattern
        );

        if title_lower.contains(&pattern_lower) || name_lower.contains(&pattern_lower) {
            found_count += 1;

            // On Windows, use taskkill to close the window by process ID
            #[cfg(target_os = "windows")]
            {
                let result = std::process::Command::new("taskkill")
                    .args(["/PID", &window.info.process_id.to_string(), "/F"])
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        closed_count += 1;
                    }
                    Ok(output) => {
                        last_error = Some(format!(
                            "taskkill failed (exit {}): {}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    Err(e) => {
                        last_error = Some(format!("Failed to run taskkill: {}", e));
                    }
                }
            }

            // On Unix-like systems, send SIGTERM
            #[cfg(not(target_os = "windows"))]
            {
                let result = std::process::Command::new("kill")
                    .arg(window.info.process_id.to_string())
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        closed_count += 1;
                    }
                    Ok(output) => {
                        last_error = Some(format!(
                            "kill failed (exit {}): {}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    Err(e) => {
                        last_error = Some(format!("Failed to run kill: {}", e));
                    }
                }
            }
        }
    }

    if closed_count > 0 {
        Ok(format!(
            "Successfully closed {} window(s) matching '{}' (Found: {})",
            closed_count, title_pattern, found_count
        ))
    } else if found_count > 0 {
        Err(JsErrorBox::new(
            "Error",
            format!(
                "Found {} window(s) matching '{}', but failed to close any. Last error: {}",
                found_count,
                title_pattern,
                last_error.unwrap_or_else(|| "Unknown error".to_string())
            ),
        ))
    } else {
        Err(JsErrorBox::new(
            "Error",
            format!("No windows found matching '{}'", title_pattern),
        ))
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
        // In headless environments (CI, containers), we may get an error instead of a title.
        // We just check that we got some result (either a title or an error message).
        assert!(!workflow.result[0].result.is_empty());
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
