// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{
    CheckPermissionResult, PluginFunctionPermissions, check_permission,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub fn exec_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.exec.exec".to_string(),
        function_name: "Exec".to_string(),
        description: "Executes a command in the default shell and returns its output.".to_string(),
        permissions: exec_plugin_permissions(),
        arguments: "String: command".to_string(),
        returns: "String: output".to_string(),
    }
}

pub fn exec_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.exec".to_string(),
        package_name: "Exec".to_string(),
        description: "A plugin to execute shell commands.".to_string(),
        functions: vec![exec_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_exec_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.exec.exec".to_string(),
        "Exec".to_string(),
        "Executes a command in the default shell and returns its output.".to_string(),
        op2_exec(),
        Some(include_str!("00_exec.js").to_string()),
    )
}

pub fn core_exec_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.exec".to_string(),
        "Exec".to_string(),
        vec![core_exec_plugin()],
    )
}

fn _permission_check_backend(
    allow: Vec<PluginFunctionPermissions>,
    command: String,
) -> Result<(), JsErrorBox> {
    let mut perm = exec_plugin_permissions();
    perm[0].resource = vec![command.clone()];
    let required_permissions = sapphillon_core::permission::Permissions { permissions: perm };

    let allowed_permissions = {
        let permissions_vec = allow;
        permissions_vec
            .into_iter()
            .find(|p| {
                p.plugin_function_id == exec_plugin_function().function_id
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

fn permission_check(state: &mut OpState, command: String) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p,
        None => &vec![PluginFunctionPermissions {
            plugin_function_id: exec_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: exec_plugin_permissions(),
            },
        }],
    };
    _permission_check_backend(allowed.clone(), command)?;
    Ok(())
}

#[op2]
#[string]
fn op2_exec(
    state: &mut OpState,
    #[string] command: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(state, command.clone())?;

    match exec(&command) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn exec(command: &str) -> anyhow::Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(command).output()
    } else {
        Command::new("sh").arg("-c").arg(command).output()
    }?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(anyhow::anyhow!(
            "Command failed with status {}: {}",
            output.status,
            stderr
        ))
    }
}

fn exec_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Command Access".to_string(),
        description: "Allows the plugin to execute shell commands.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::workflow::CoreWorkflowCode;

    #[test]
    fn test_exec_success() {
        let result = exec("echo hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_exec_failure() {
        let result = exec("invalid_command_that_does_not_exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_exec_in_workflow() {
        let code = r#"
            const output = exec("echo test_workflow");
            console.log(output);
        "#;

        let perm = PluginFunctionPermissions {
            plugin_function_id: exec_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Command Access".to_string(),
                    description: "Allows the plugin to execute shell commands.".to_string(),
                    permission_type: PermissionType::Execute as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["echo test_workflow".to_string()],
                }],
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_exec_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let result_str = workflow.result[0].result.trim();
        assert_eq!(result_str, "test_workflow");
    }

    #[test]
    fn test_permission_error_in_workflow() {
        let code = r#"
            exec("echo should_fail");
        "#;

        // Use empty permissions list to trigger permission denial
        let perm = PluginFunctionPermissions {
            plugin_function_id: exec_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![],
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_exec_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let actual = &workflow.result[0].result;
        assert!(
            actual.to_lowercase().contains("permission denied") || actual.contains("Uncaught"),
            "Unexpected workflow result: {actual}"
        );
    }
}
