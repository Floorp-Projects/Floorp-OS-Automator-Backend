// Sapphillon
// Filesystem plugin - provides simple text file IO (read) with permission checks
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{check_permission, CheckPermissionResult, PluginFunctionPermissions};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::fs;
use std::sync::{Arc, Mutex};

pub fn filesystem_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.filesystem.read".to_string(),
        function_name: "ReadFile".to_string(),
        description: "Reads a text file from the local filesystem and returns its contents as a string.".to_string(),
        permissions: filesystem_plugin_permissions(),
        arguments: "String: path".to_string(),
        returns: "String: content".to_string(),
    }
}

pub fn filesystem_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.filesystem".to_string(),
        package_name: "Filesystem".to_string(),
        description: "A plugin to read text files from the local filesystem.".to_string(),
        functions: vec![filesystem_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

pub fn core_filesystem_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.filesystem.read".to_string(),
        "ReadFile".to_string(),
        "Reads a text file from the local filesystem and returns its contents as a string.".to_string(),
        op2_read_file(),
        Some(include_str!("00_filesystem.js").to_string()),
    )
}

pub fn core_filesystem_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.filesystem".to_string(),
        "Filesystem".to_string(),
        vec![core_filesystem_plugin()],
    )
}

fn _permission_check_backend(
    allow: Vec<PluginFunctionPermissions>,
    path: String,
) -> Result<(), JsErrorBox> {
    let mut perm = filesystem_plugin_permissions();
    perm[0].resource = vec![path.clone()];
    let required_permissions = sapphillon_core::permission::Permissions { permissions: perm };

    let allowed_permissions = {
        let permissions_vec = allow;

        permissions_vec
            .into_iter()
            .find(|p| p.plugin_function_id == filesystem_plugin_function().function_id)
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions { permissions: vec![] })
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

fn permission_check(state: &mut OpState, path: String) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p,
        None => &vec![PluginFunctionPermissions {
            plugin_function_id: filesystem_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions { permissions: filesystem_plugin_permissions() },
        }],
    };
    _permission_check_backend(allowed.clone(), path)?;
    Ok(())
}

#[op2]
#[string]
fn op2_read_file(
    state: &mut OpState,
    #[string] path: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission check
    permission_check(state, path.clone())?;

    match read_file_text(&path) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn read_file_text(path: &str) -> anyhow::Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s)
}

fn filesystem_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Filesystem Read".to_string(),
        description: "Allows the plugin to read files from the local filesystem.".to_string(),
        permission_type: PermissionType::FilesystemRead as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::proto::sapphillon::v1::PermissionType;
    use sapphillon_core::workflow::CoreWorkflowCode;
    use std::io::Write;

    #[test]
    fn test_read_file_text() {
        // create a temp file
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "hello world").unwrap();
        let path = f.path().to_str().unwrap().to_string();

        let res = read_file_text(&path);
        assert!(res.is_ok());
        let s = res.unwrap();
        assert!(s.contains("hello world"));
    }

    #[test]
    fn test_permission_in_workflow() {
        let code = r#"
            const path = "/tmp/__sapphillon_test__";
            const content = readFile(path);
            console.log(content);
        "#;

        // Create the file that workflow will try to read
        let tmp_path = "/tmp/__sapphillon_test__".to_string();
        std::fs::write(&tmp_path, "workflow-test").unwrap();

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: filesystem_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Filesystem Read".to_string(),
                    description: "Allows reading tests".to_string(),
                    permission_type: PermissionType::FilesystemRead as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec![tmp_path.clone()],
                }],
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_filesystem_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let expected = std::fs::read_to_string(&tmp_path).unwrap() + "\n";
        let actual = &workflow.result[0].result;
        assert_eq!(actual, &expected);
    }
}
