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
//
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon;
use sapphillon_core::proto::sapphillon::v1::{PluginFunction, PluginPackage, Permission, PermissionType, PermissionLevel};
use sapphillon_core::permission::{check_permission, CheckPermissionResult, PluginFunctionPermissions};
use sapphillon_core::runtime::OpStateWorkflowData;


pub fn fetch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.fetch.fetch".to_string(),
        function_name: "Fetch".to_string(),
        description: "Fetches the content of a URL using reqwest and returns it as a string.".to_string(),
        permissions: fetch_plugin_permissions(),
        arguments: "String: url".to_string(),
        returns: "String: content".to_string(),
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
        verified: Some(true)
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

pub fn core_fetch_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.fetch".to_string(),
        "Fetch".to_string(),
        vec![core_fetch_plugin()],
    )
}

fn permission_check(state: &mut OpState, url: String) -> Result<(), JsErrorBox> {
    // Requrements Permission of this func
    let mut perm = fetch_plugin_permissions();
    perm[0].resource = vec![url.clone()];

    let required_permissions = sapphillon_core::permission::Permissions { permissions: perm };

    // Allowed Permission in this func
    let allowed_permissions = {
        let data = state.borrow::<OpStateWorkflowData>();
        let permissions = data.get_allowed_permissions().clone().unwrap_or_else(|| vec![PluginFunctionPermissions {
            plugin_function_id: fetch_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions { permissions: vec![] },
        }]);
        
        permissions.into_iter().find(|p| p.plugin_function_id == fetch_plugin_function().function_id)
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions { permissions: vec![] })
        
    };
    
    let permission_check_result = check_permission(&required_permissions, &allowed_permissions);

    if let CheckPermissionResult::MissingPermission(perm) = permission_check_result {
        return Err(JsErrorBox::new("PermissionDenied. Missing Permissions:", perm.to_string()));
    }
    Ok(())
}

#[op2]
#[string]
fn op2_fetch(state: &mut OpState, #[string] url: String) -> std::result::Result<String, JsErrorBox> {
    // Permission Check
    permission_check(state, url.clone())?;
    

    match fetch(&url) {
        Ok(body) => Ok(body),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn fetch(url: &str) -> anyhow::Result<String> {
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;
    Ok(body)
}

fn fetch_plugin_permissions() -> Vec<Permission> {
    vec![
        Permission {
            display_name: "Network Access".to_string(),
            description: "Allows the plugin to make network requests.".to_string(),
            permission_type: PermissionType::NetAccess as i32,
            permission_level: PermissionLevel::Unspecified as i32,
            resource: vec![]
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_fetch_in_workflow() {
        let code = r#"
            const url = "https://dummyjson.com/test";
            const response = fetch(url);
            console.log(response);
        "#;

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_fetch_plugin_package()],
            1,
            None,
            None,
        );
        workflow.run();
        assert_eq!(workflow.result.len(), 1);

        let url = "https://dummyjson.com/test";
        let expected = fetch(url).unwrap() + "\n";

        assert_eq!(workflow.result[0].result, expected)
    }
}
