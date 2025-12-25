// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

// Filesystem plugin - provides simple text file IO (read) with permission checks
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{permission_check, PluginFunctionPermissions};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::fs;
use std::sync::{Arc, Mutex};

pub fn filesystem_read_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.filesystem.read".to_string(),
        function_name: "fs.read".to_string(),
        description:
            "Reads a text file from the local filesystem and returns its contents as a string."
                .to_string(),
        permissions: filesystem_read_plugin_permissions(),
        arguments: "String: path".to_string(),
        returns: "String: content".to_string(),
    }
}

pub fn filesystem_list_files_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.filesystem.list_files".to_string(),
        function_name: "fs.list".to_string(),
        description: "List files in a directory.".to_string(),
        permissions: filesystem_list_files_plugin_permissions(),
        arguments: "String: path".to_string(),
        returns: "String: content".to_string(),
    }
}

pub fn filesystem_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.filesystem".to_string(),
        package_name: "Filesystem".to_string(),
        description: "A plugin to read and write text files from the local filesystem.".to_string(),
        functions: vec![
            filesystem_read_plugin_function(),
            filesystem_write_plugin_function(),
            filesystem_list_files_plugin_function(),
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

pub fn core_filesystem_read_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.filesystem.read".to_string(),
        "ReadFile".to_string(),
        "Reads a text file from the local filesystem and returns its contents as a string."
            .to_string(),
        op2_filesystem_read(),
        Some(include_str!("00_filesystem.js").to_string()),
    )
}

pub fn core_filesystem_list_files_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.filesystem.list_files".to_string(),
        "ListFiles".to_string(),
        "List files in a directory.".to_string(),
        op2_filesystem_list_files(),
        Some(include_str!("00_filesystem.js").to_string()),
    )
}

pub fn core_filesystem_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.filesystem".to_string(),
        "Filesystem".to_string(),
        vec![
            core_filesystem_read_plugin(),
            core_filesystem_write_plugin(),
            core_filesystem_list_files_plugin(),
        ],
    )
}

pub fn filesystem_write_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.filesystem.write".to_string(),
        function_name: "WriteFile".to_string(),
        description: "Writes text to a file on the local filesystem.".to_string(),
        permissions: filesystem_write_plugin_permissions(),
        arguments: "String: path, String: content".to_string(),
        returns: "String: result".to_string(),
    }
}

pub fn core_filesystem_write_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.filesystem.write".to_string(),
        "WriteFile".to_string(),
        "Writes text to a file on the local filesystem.".to_string(),
        op2_filesystem_write(),
        Some(include_str!("00_filesystem.js").to_string()),
    )
}

#[op2]
#[string]
fn op2_filesystem_write(
    state: &mut OpState,
    #[string] path: String,
    #[string] content: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission check
    permission_check(
        state,
        &filesystem_write_plugin_function().function_id,
        filesystem_write_plugin_permissions(),
        &path,
    )?;

    match write_file_text_filesystem_write(&path, &content) {
        Ok(_) => Ok("ok".to_string()),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_filesystem_list_files(
    state: &mut OpState,
    #[string] path: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission check
    permission_check(
        state,
        &filesystem_list_files_plugin_function().function_id,
        filesystem_list_files_plugin_permissions(),
        &path,
    )?;

    match list_files_in_directory(&path) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn list_files_in_directory(path: &str) -> anyhow::Result<String> {
    let paths = fs::read_dir(path)?;
    let files: Vec<String> = paths
        .map(|res| res.map(|e| e.path().display().to_string()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    Ok(serde_json::to_string(&files)?)
}

fn write_file_text_filesystem_write(path: &str, content: &str) -> anyhow::Result<()> {
    fs::write(path, content)?;
    Ok(())
}

fn filesystem_write_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Filesystem Write".to_string(),
        description: "Allows the plugin to write files to the local filesystem.".to_string(),
        permission_type: PermissionType::FilesystemWrite as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

fn filesystem_list_files_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Filesystem Read".to_string(),
        description: "Allows the plugin to list files from the local filesystem.".to_string(),
        permission_type: PermissionType::FilesystemRead as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

#[op2]
#[string]
fn op2_filesystem_read(
    state: &mut OpState,
    #[string] path: String,
) -> std::result::Result<String, JsErrorBox> {
    // Permission check
    permission_check(
        state,
        &filesystem_read_plugin_function().function_id,
        filesystem_read_plugin_permissions(),
        &path,
    )?;

    match read_file_text_filesystem_read(&path) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

fn read_file_text_filesystem_read(path: &str) -> anyhow::Result<String> {
    let s = fs::read_to_string(path)?;
    Ok(s)
}

fn filesystem_read_plugin_permissions() -> Vec<Permission> {
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
    use serial_test::serial;
    use std::io::Write;

    // Tests below use std::env::temp_dir() to construct temporary file paths so
    // they work both on Unix-like systems and Windows (avoids hard-coded paths
    // like /tmp/... and handles backslashes in JS string literals).

    #[test]
    #[serial]
    fn test_read_file_text() {
        // create a temp file
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "hello world").unwrap();
        let path = f.path().to_str().unwrap().to_string();

        let res = read_file_text_filesystem_read(&path);
        assert!(res.is_ok());
        let s = res.unwrap();
        assert!(s.contains("hello world"));
    }

    #[test]
    #[serial]
    fn test_write_file_text() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let res = write_file_text_filesystem_write(&path, "written-content");
        assert!(res.is_ok());
        let s = std::fs::read_to_string(&path).unwrap();
        assert_eq!(s, "written-content");
    }

    #[test]
    #[serial]
    fn test_list_files() {
        let dir = tempfile::tempdir().unwrap();
        let file1_path = dir.path().join("file1.txt");
        let file2_path = dir.path().join("file2.txt");
        std::fs::File::create(&file1_path).unwrap();
        std::fs::File::create(&file2_path).unwrap();

        let res = list_files_in_directory(dir.path().to_str().unwrap());
        assert!(res.is_ok());
        let s = res.unwrap();
        let files: Vec<String> = serde_json::from_str(&s).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f == file1_path.to_str().unwrap()));
        assert!(files.iter().any(|f| f == file2_path.to_str().unwrap()));
    }

    #[test]
    #[serial]
    fn test_permission_in_workflow() {
        // Create a platform-appropriate temp path and write the file
        let mut tmp_path_buf = std::env::temp_dir();
        tmp_path_buf.push("__sapphillon_test__");
        let tmp_path = tmp_path_buf.to_str().unwrap().to_string();
        std::fs::write(&tmp_path_buf, "workflow-test").unwrap();

        let tmp_path = tmp_path.replace(r"\", r"\\");

        // Build JS code with the properly-escaped path string so backslashes on Windows
        // don't create invalid escape sequences in the JS literal.
        let code = format!(
            "const path = {tmp_path:?}; const content = readFile(path); console.log(content);"
        );

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: filesystem_read_plugin_function().function_id,
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

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_filesystem_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let expected = std::fs::read_to_string(&tmp_path).unwrap() + "\n";
        let actual = &workflow.result[0].result;
        assert_eq!(actual, &expected);

        // Clean up
        let _ = std::fs::remove_file(&tmp_path);
    }

    #[test]
    #[serial]
    fn test_permission_write_in_workflow() {
        // Ensure file does not exist then grant permission
        let mut tmp_path_buf = std::env::temp_dir();
        tmp_path_buf.push("__sapphillon_test_write__");
        let tmp_path = tmp_path_buf.to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&tmp_path_buf);

        let tmp_path = tmp_path.replace(r"\", r"\\");

        // Build JS code with escaped path literal
        let code = format!(
            "const path = {tmp_path:?}; fs.write(path, \"workflow-write\"); console.log(\"done\");"
        );

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: filesystem_write_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "Filesystem Write".to_string(),
                    description: "Allows writing tests".to_string(),
                    permission_type: PermissionType::FilesystemWrite as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec![tmp_path.clone()],
                }],
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test-write".to_string(),
            code.to_string(),
            vec![core_filesystem_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // workflow prints "done\n"
        let actual = &workflow.result[0].result;
        assert_eq!(actual, &"done\n".to_string());
        // verify the file was written
        let s = std::fs::read_to_string(&tmp_path).unwrap();
        assert_eq!(s, "workflow-write");

        // Read back the file to verify content
        let file_content = std::fs::read_to_string(&tmp_path).unwrap();
        assert_eq!(file_content, "workflow-write");

        // Clean up
        let _ = std::fs::remove_file(&tmp_path);
    }

    #[test]
    #[serial]
    fn test_permission_list_files_in_workflow() {
        // Create a directory and some files in it
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.path().to_str().unwrap().to_string();
        std::fs::File::create(tmp_dir.path().join("file1.txt")).unwrap();
        std::fs::File::create(tmp_dir.path().join("file2.txt")).unwrap();

        let escaped_path = tmp_path.replace(r"\", r"\\");

        // Build JS code with the properly-escaped path string so backslashes on Windows
        // don't create invalid escape sequences in the JS literal.
        let code = format!(
            "const path = {escaped_path:?}; const content = fs.list(path); console.log(content);"
        );

        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: filesystem_list_files_plugin_function().function_id,
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

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_filesystem_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let actual = &workflow.result[0].result;
        // The expected result is a JSON string of a list of files, followed by a newline.
        let file1 = tmp_dir.path().join("file1.txt");
        let file2 = tmp_dir.path().join("file2.txt");
        let expected_files: Vec<String> = vec![
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        let actual_files: Vec<String> = serde_json::from_str(actual.trim_end()).unwrap();

        expected_files.iter().for_each(|f| {
            assert!(actual_files.contains(f));
        });
        actual_files.iter().for_each(|f| {
            assert!(expected_files.contains(f));
        });
    }

    #[test]
    #[serial]
    fn test_permission_denied_list_files_in_workflow() {
        // Create a directory and some files in it
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.path().to_str().unwrap().to_string();
        std::fs::File::create(tmp_dir.path().join("file1.txt")).unwrap();
        std::fs::File::create(tmp_dir.path().join("file2.txt")).unwrap();

        let escaped_path = tmp_path.replace(r"\", r"\\");

        // Build JS code with the properly-escaped path string so backslashes on Windows
        // don't create invalid escape sequences in the JS literal.
        let code = format!(
            "const path = {escaped_path:?}; const content = fs.list(path); console.log(content);"
        );

        // Create a permission with an empty permissions list to trigger permission denial
        let perm: PluginFunctionPermissions = PluginFunctionPermissions {
            plugin_function_id: filesystem_list_files_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![],
            },
        };

        let workflow_permissions = vec![perm.clone()];
        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_filesystem_plugin_package()],
            1,
            workflow_permissions.clone(),
            workflow_permissions,
        );

        workflow.run();
        println!("workflow.result: {:?}", workflow.result);
        assert_eq!(workflow.result.len(), 1);
        // assert!(
        //     workflow.result[0]
        //         .result
        //         .to_string()
        //         .contains("PermissionDenied. Missing Permissions:")
        // );
        assert!(workflow.result[0].result.to_string().contains("Uncaught"))
    }
}
