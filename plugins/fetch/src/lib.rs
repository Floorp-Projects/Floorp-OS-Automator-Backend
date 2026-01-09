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
use std::time::Duration;

pub fn post_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.fetch.post".to_string(),
        function_name: "Post".to_string(),
        description: "Posts the content of a URL using reqwest and returns it as a string."
            .to_string(),
        permissions: fetch_plugin_permissions(),
        function_define: Some(FunctionDefine {
            parameters: vec![
                FunctionParameter {
                    name: "url".to_string(),
                    r#type: "string".to_string(),
                    description: "Target URL".to_string(),
                },
                FunctionParameter {
                    name: "body".to_string(),
                    r#type: "string".to_string(),
                    description: "Request body".to_string(),
                },
            ],
            returns: vec![FunctionParameter {
                name: "content".to_string(),
                r#type: "string".to_string(),
                description: "Response body as string".to_string(),
            }],
        }),
    }
}

pub fn fetch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.fetch.fetch".to_string(),
        function_name: "Fetch".to_string(),
        description: "Fetches the content of a URL using reqwest and returns it as a string."
            .to_string(),
        permissions: fetch_plugin_permissions(),
        function_define: Some(FunctionDefine {
            parameters: vec![FunctionParameter {
                name: "url".to_string(),
                r#type: "string".to_string(),
                description: "Target URL".to_string(),
            }],
            returns: vec![FunctionParameter {
                name: "content".to_string(),
                r#type: "string".to_string(),
                description: "Response body as string".to_string(),
            }],
        }),
    }
}

pub fn fetch_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.fetch".to_string(),
        package_name: "Fetch".to_string(),
        description: "A plugin to fetch the content of a URL.".to_string(),
        functions: vec![fetch_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_fetch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.fetch.fetch".to_string(),
        "Fetch".to_string(),
        "Fetches the content of a URL using reqwest and returns it as a string.".to_string(),
        op2_fetch(),
        Some(include_str!("00_fetch.js").to_string()),
    )
}

pub fn core_post_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.fetch.post".to_string(),
        "Post".to_string(),
        "Posts the content of a URL using reqwest and returns it as a string.".to_string(),
        op2_post(),
        Some(include_str!("00_fetch.js").to_string()),
    )
}

pub fn core_fetch_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.fetch".to_string(),
        "Fetch".to_string(),
        vec![core_fetch_plugin(), core_post_plugin()],
    )
}

#[op2]
#[string]
fn op2_fetch(
    state: &mut OpState,
    #[string] url: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission Check
    ensure_permission(
        state,
        &fetch_plugin_function().function_id,
        fetch_plugin_permissions(),
        &url,
    )?;

    match fetch(&url) {
        Ok(body) => Ok(body),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_post(
    state: &mut OpState,
    #[string] url: String,
    #[string] body: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission Check
    ensure_permission(
        state,
        &post_plugin_function().function_id,
        fetch_plugin_permissions(),
        &url,
    )?;

    match post(&url, &body) {
        Ok(body) => Ok(body),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn fetch(url: &str) -> anyhow::Result<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let body = agent.get(url).call()?.body_mut().read_to_string()?;
    Ok(body)
}

fn post(url: &str, body: &str) -> anyhow::Result<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let response_body = agent.post(url).send(body)?.body_mut().read_to_string()?;
    Ok(response_body)
}

fn fetch_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Network Access".to_string(),
        description: "Allows the plugin to make network requests.".to_string(),
        permission_type: PermissionType::NetAccess as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
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

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::permission::PluginFunctionPermissions;
    use sapphillon_core::proto::sapphillon::v1::PermissionType;
    use sapphillon_core::workflow::CoreWorkflowCode;

    #[test]
    fn test_fetch() {
        let url = "https://dummyjson.com/test";
        let result = fetch(url);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("ok"));
        println!("Fetched content: {body}");
    }

    #[test]
    fn test_post() {
        let url = "https://dummyjson.com/products/add";
        let result = post(url, r#"{"title":"test"}"#);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("id"));
        println!("Posted content: {body}");
    }

    #[tokio::test]
    async fn test_permission_error() {
        let code = r#"
            const url = "https://dummyjson.com/test";
            const response = app.sapphillon.core.fetch.fetch(url);
            console.log(response);
        "#;

        // Provide allowed permissions so permission_check inside the plugin passes.
        let _url = "https://dummyjson.com/test".to_string();

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: fetch_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Network Access".to_string(),
                    description: "Allows the plugin to make network requests.".to_string(),
                    permission_type: PermissionType::NetAccess as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["dummyjson.com/test".to_string()],
                }],
            },
        };
        let allowed_permissions = vec![perm];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![Arc::new(core_fetch_plugin_package())],
            1,
            vec![],
            allowed_permissions,
        );

        workflow.run(tokio::runtime::Handle::current());
        assert_eq!(workflow.result.len(), 1);

        let actual = &workflow.result[0].result;
        // Accept either a successful fetch result or a permission-denied message depending on test environment.
        assert!(
            actual.to_lowercase().contains("permission denied") || actual.contains("Uncaught"),
            "Unexpected workflow result: {actual}"
        );
    }
    #[tokio::test]
    async fn test_fetch_in_workflow() {
        let code = r#"
            const url = "https://dummyjson.com/test";
            const response = app.sapphillon.core.fetch.fetch(url);
            console.log(response);
        "#;

        // Provide allowed permissions so permission_check inside the plugin passes.
        let url = "https://dummyjson.com/test".to_string();

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: fetch_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Network Access".to_string(),
                    description: "Allows the plugin to make network requests.".to_string(),
                    permission_type: PermissionType::NetAccess as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["https://dummyjson.com/test".to_string()],
                }],
            },
        };
        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![Arc::new(core_fetch_plugin_package())],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run(tokio::runtime::Handle::current());
        assert_eq!(workflow.result.len(), 1);

        let expected = fetch(&url).unwrap() + "\n";

        let actual = &workflow.result[0].result;
        // Accept either a successful fetch result or a permission-denied message depending on test environment.
        assert!(actual == &expected, "Unexpected workflow result: {actual}");
    }

    #[tokio::test]
    async fn test_post_in_workflow() {
        let code = r#"
            const url = "https://dummyjson.com/products/add";
            const response = app.sapphillon.core.fetch.post(url, '{"title":"test"}');
            console.log(response);
        "#;

        // Provide allowed permissions so permission_check inside the plugin passes.
        let url = "https://dummyjson.com/products/add".to_string();

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: post_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Network Access".to_string(),
                    description: "Allows the plugin to make network requests.".to_string(),
                    permission_type: PermissionType::NetAccess as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec!["https://dummyjson.com/products/add".to_string()],
                }],
            },
        };
        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![Arc::new(core_fetch_plugin_package())],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run(tokio::runtime::Handle::current());
        assert_eq!(workflow.result.len(), 1);

        let expected = post(&url, r#"{"title":"test"}"#).unwrap() + "\n";

        let actual = &workflow.result[0].result;
        // Accept either a successful fetch result or a permission-denied message depending on test environment.
        assert!(actual == &expected, "Unexpected workflow result: {actual}");
    }

    #[test]
    fn test_fetch_plugin_permissions() {
        let perms = fetch_plugin_permissions();
        assert_eq!(perms.len(), 1, "Expected exactly one permission");
        let p = &perms[0];
        assert_eq!(p.display_name, "Network Access");
        assert!(p.description.contains("network"));
        assert_eq!(p.permission_type, PermissionType::NetAccess as i32);
        assert!(p.resource.is_empty());
    }

    #[test]
    fn test_fetch_plugin_package() {
        let pkg = fetch_plugin_package();
        assert_eq!(pkg.package_id, "app.sapphillon.core.fetch");
        assert_eq!(pkg.package_name, "Fetch");
        // package_version is set from crate version; ensure it's non-empty and matches env.
        let expected_version = env!("CARGO_PKG_VERSION").to_string();
        assert_eq!(pkg.package_version, expected_version);
        // Ensure functions includes the fetch function id
        let func_ids: Vec<String> = pkg.functions.into_iter().map(|f| f.function_id).collect();
        assert!(
            func_ids.contains(&fetch_plugin_function().function_id),
            "Package functions must include fetch_plugin_function"
        );
    }

    #[test]
    fn test_core_fetch_plugin_and_package() {
        // Construct core plugin and package to ensure creation succeeds.
        let _core_fn = core_fetch_plugin();
        // The JS bundled with the core plugin should exist in the source file.
        let js = include_str!("00_fetch.js");
        assert!(!js.is_empty(), "Embedded JS source must not be empty");

        // Construct core package; creation should succeed without panics.
        let _core_pkg = core_fetch_plugin_package();
    }
}
