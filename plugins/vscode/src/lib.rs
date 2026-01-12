// VSCode Plugin for Sapphillon
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

// ============================================================================
// Plugin Function Definitions
// ============================================================================

pub fn vscode_open_folder_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.open_folder".to_string(),
        function_name: "Open Folder".to_string(),
        description: "Opens a folder in VSCode.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_open_file_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.open_file".to_string(),
        function_name: "Open File".to_string(),
        description: "Opens a file in VSCode.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_write_file_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.write_file".to_string(),
        function_name: "Write File".to_string(),
        description: "Writes content to a file and opens it in VSCode.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_close_workspace_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.close_workspace".to_string(),
        function_name: "Close Workspace".to_string(),
        description: "Closes the current VSCode workspace/project.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_get_active_file_content_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.get_active_file_content".to_string(),
        function_name: "Get Active File Content".to_string(),
        description: "Gets the content of the currently active file in VSCode.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_get_workspace_path_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.vscode.get_workspace_path".to_string(),
        function_name: "Get Workspace Path".to_string(),
        description: "Gets the path of the current workspace folder in VSCode.".to_string(),
        permissions: vscode_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn vscode_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.vscode".to_string(),
        package_name: "VSCode".to_string(),
        provider_id: "".to_string(),
        description: "A plugin to manage VSCode (Visual Studio Code).".to_string(),
        functions: vec![
            vscode_open_folder_plugin_function(),
            vscode_open_file_plugin_function(),
            vscode_write_file_plugin_function(),
            vscode_close_workspace_plugin_function(),
            vscode_get_active_file_content_plugin_function(),
            vscode_get_workspace_path_plugin_function(),
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

// ============================================================================
// Core Plugin Functions (for Deno runtime integration)
// ============================================================================

pub fn core_vscode_open_folder_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.open_folder".to_string(),
        "vscode.open_folder".to_string(),
        "Opens a folder in VSCode.".to_string(),
        op2_vscode_open_folder(),
        Some(include_str!("00_vscode.js").to_string()),
    )
}

pub fn core_vscode_open_file_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.open_file".to_string(),
        "vscode.open_file".to_string(),
        "Opens a file in VSCode.".to_string(),
        op2_vscode_open_file(),
        Some(include_str!("00_vscode.js").to_string()),
    )
}

pub fn core_vscode_write_file_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.write_file".to_string(),
        "vscode.write_file".to_string(),
        "Writes content to a file and opens it in VSCode.".to_string(),
        op2_vscode_write_file(),
        Some(include_str!("00_vscode.js").to_string()),
    )
}

pub fn core_vscode_close_workspace_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.close_workspace".to_string(),
        "vscode.close_workspace".to_string(),
        "Closes the current VSCode workspace/project.".to_string(),
        op2_vscode_close_workspace(),
        Some(include_str!("00_vscode.js").to_string()),
    )
}

pub fn core_vscode_get_active_file_content_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.get_active_file_content".to_string(),
        "vscode.get_active_file_content".to_string(),
        "Gets the content of the currently active file in VSCode.".to_string(),
        op2_vscode_get_active_file_content(),
        Some(include_str!("00_vscode.js").to_string()),
    )
}

pub fn core_vscode_get_workspace_path_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.vscode.get_workspace_path".to_string(),
        "vscode.get_workspace_path".to_string(),
        "Gets the path of the current workspace folder in VSCode.".to_string(),
        op2_vscode_get_workspace_path(),
        None,
    )
}

pub fn core_vscode_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.vscode".to_string(),
        "VSCode".to_string(),
        vec![
            core_vscode_open_folder_plugin(),
            core_vscode_open_file_plugin(),
            core_vscode_write_file_plugin(),
            core_vscode_close_workspace_plugin(),
            core_vscode_get_active_file_content_plugin(),
            core_vscode_get_workspace_path_plugin(),
        ],
    )
}

// ============================================================================
// Permission Definitions
// ============================================================================

pub fn vscode_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "VSCode Access".to_string(),
        description: "Allows the plugin to control VSCode application.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

pub fn vscode_write_plugin_permissions() -> Vec<Permission> {
    vec![
        Permission {
            display_name: "VSCode Access".to_string(),
            description: "Allows the plugin to control VSCode application.".to_string(),
            permission_type: PermissionType::Execute as i32,
            permission_level: PermissionLevel::Unspecified as i32,
            resource: vec![],
        },
        Permission {
            display_name: "Filesystem Write".to_string(),
            description: "Allows the plugin to write files to the local filesystem.".to_string(),
            permission_type: PermissionType::FilesystemWrite as i32,
            permission_level: PermissionLevel::Unspecified as i32,
            resource: vec![],
        },
    ]
}

pub fn vscode_get_content_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "VSCode Access".to_string(),
        description: "Allows the plugin to control VSCode application.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

// ============================================================================
// Permission Check Helpers
// ============================================================================

fn _permission_check_backend(
    allow: Vec<PluginFunctionPermissions>,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let required_permissions = sapphillon_core::permission::Permissions {
        permissions: required_perms,
    };

    let allowed_permissions = {
        let permissions_vec = allow;
        permissions_vec
            .into_iter()
            .find(|p| p.plugin_function_id == function_id || p.plugin_function_id == "*")
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions {
                permissions: vec![],
            })
    };

    let permission_check_result = check_permission(&allowed_permissions, &required_permissions);

    match permission_check_result {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "Error",
            format!("PermissionDenied. Missing Permissions: {}", perm),
        )),
    }
}

fn permission_check(
    state: &mut OpState,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p.clone(),
        None => vec![],
    };
    _permission_check_backend(allowed, function_id, required_perms)?;
    Ok(())
}

// ============================================================================
// VSCode Command Execution Helpers
// ============================================================================

/// Opens a path in VSCode using the vscode:// URL scheme (macOS/Linux)
/// or falls back to CLI on Windows
fn open_in_vscode(path: &str, line: Option<u32>, column: Option<u32>) -> anyhow::Result<String> {
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        // Use vscode:// URL scheme - no special permissions needed
        let url = match (line, column) {
            (Some(l), Some(c)) => format!("vscode://file{}:{}:{}", path, l, c),
            (Some(l), None) => format!("vscode://file{}:{}", path, l),
            _ => format!("vscode://file{}", path),
        };

        let output = Command::new("open").arg(&url).output()?;

        if output.status.success() {
            Ok("ok".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow::anyhow!("Failed to open VSCode URL: {}", stderr))
        }
    } else if cfg!(target_os = "windows") {
        // Windows: Use CLI approach
        let mut args = vec![path];
        if let Some(l) = line {
            args.push("--goto");
            let goto = match column {
                Some(c) => format!("{}:{}:{}", path, l, c),
                None => format!("{}:{}", path, l),
            };
            // Need to handle this differently for Windows
            let output = Command::new("cmd")
                .arg("/C")
                .arg("code")
                .arg("--goto")
                .arg(&goto)
                .output()?;

            if output.status.success() {
                return Ok("ok".to_string());
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(anyhow::anyhow!("VSCode command failed: {}", stderr));
            }
        }

        let output = Command::new("cmd")
            .arg("/C")
            .arg("code")
            .args(&args)
            .output()?;

        if output.status.success() {
            Ok("ok".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow::anyhow!("VSCode command failed: {}", stderr))
        }
    } else {
        Err(anyhow::anyhow!("Unsupported OS"))
    }
}

/// Runs a VSCode CLI command (for operations not supported by URL scheme)
fn run_vscode_cli(args: &[&str]) -> anyhow::Result<String> {
    if cfg!(target_os = "windows") {
        let output = Command::new("cmd")
            .arg("/C")
            .arg("code")
            .args(args)
            .output()?;

        if output.status.success() {
            return Ok("ok".to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(anyhow::anyhow!("VSCode command failed: {}", stderr));
        }
    }

    // macOS/Linux: Try `code` from PATH
    let result = Command::new("code").args(args).output();

    let output = match result {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if cfg!(target_os = "macos") {
                let mac_path =
                    "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";
                if std::path::Path::new(mac_path).exists() {
                    Command::new(mac_path).args(args).output()?
                } else {
                    return Err(anyhow::anyhow!("VSCode 'code' command not found"));
                }
            } else {
                return Err(
                    anyhow::Error::new(e).context("VSCode 'code' command not found in PATH")
                );
            }
        }
        Err(e) => return Err(anyhow::Error::new(e).context("Failed to execute VSCode command")),
    };

    if output.status.success() {
        Ok("ok".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(anyhow::anyhow!("VSCode command failed: {}", stderr))
    }
}

// ============================================================================
// Op2 Functions (Deno Runtime Operations)
// ============================================================================

#[op2]
#[string]
fn op2_vscode_open_folder(
    state: &mut OpState,
    #[string] path: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_open_folder_plugin_function().function_id,
        vscode_plugin_permissions(),
    )?;

    match open_in_vscode(&path, None, None) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_vscode_open_file(
    state: &mut OpState,
    #[string] path: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_open_file_plugin_function().function_id,
        vscode_plugin_permissions(),
    )?;

    match open_in_vscode(&path, None, None) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_vscode_write_file(
    state: &mut OpState,
    #[string] path: String,
    #[string] content: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_write_file_plugin_function().function_id,
        vscode_plugin_permissions(),
    )?;

    // Write content to file
    std::fs::write(&path, &content).map_err(|e| JsErrorBox::new("Error", e.to_string()))?;

    // Open file in VSCode using URL scheme
    match open_in_vscode(&path, None, None) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_vscode_close_workspace(state: &mut OpState) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_close_workspace_plugin_function().function_id,
        vscode_plugin_permissions(),
    )?;

    // Close requires CLI - URL scheme doesn't support this
    match run_vscode_cli(&["--close-all"]) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

#[op2]
#[string]
fn op2_vscode_get_active_file_content(
    state: &mut OpState,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_get_active_file_content_plugin_function().function_id,
        vscode_plugin_permissions(),
    )?;

    // Try to get active file from VSCode's state database first
    if let Ok(path) = get_active_file_from_vscode_state()
        && let Ok(content) = std::fs::read_to_string(&path)
    {
        return Ok(content);
    }

    // Fallback to lsof approach
    get_active_file_via_lsof()
}

#[op2]
#[string]
fn op2_vscode_get_workspace_path(state: &mut OpState) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &vscode_get_workspace_path_plugin_function().function_id,
        vscode_get_content_plugin_permissions(),
    )?;

    match get_workspace_path_from_vscode_state() {
        Ok(path) => Ok(path),
        Err(e) => Err(JsErrorBox::new("Error", e.to_string())),
    }
}

/// Gets the workspace folder path from VSCode's state.vscdb
fn get_workspace_path_from_vscode_state() -> anyhow::Result<String> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    let workspace_storage_base = format!(
        "{}/Library/Application Support/Code/User/workspaceStorage",
        home
    );

    // Find the most recently modified workspace state.vscdb
    let mut newest_db: Option<(String, std::time::SystemTime, String)> = None;

    if let Ok(entries) = std::fs::read_dir(&workspace_storage_base) {
        for entry in entries.flatten() {
            let workspace_json = entry.path().join("workspace.json");
            let state_db = entry.path().join("state.vscdb");

            if state_db.exists()
                && workspace_json.exists()
                && let Ok(meta) = state_db.metadata()
                && let Ok(modified) = meta.modified()
            {
                // Read the workspace.json to get the folder path
                if let Ok(json_str) = std::fs::read_to_string(&workspace_json)
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str)
                {
                    // The folder field contains the workspace URI like "file:///path/to/folder"
                    if let Some(folder) = json.get("folder").and_then(|f| f.as_str())
                        && (newest_db.is_none() || modified > newest_db.as_ref().unwrap().1)
                    {
                        let folder_path =
                            folder.strip_prefix("file://").unwrap_or(folder).to_string();
                        newest_db = Some((
                            state_db.to_string_lossy().to_string(),
                            modified,
                            folder_path,
                        ));
                    }
                }
            }
        }
    }

    newest_db
        .map(|(_, _, path)| path)
        .ok_or_else(|| anyhow::anyhow!("No workspace folder found in VSCode state"))
}

/// Reads VSCode's workspace state.vscdb SQLite database to find the currently active file
fn get_active_file_from_vscode_state() -> anyhow::Result<String> {
    use rusqlite::Connection;

    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    let workspace_storage_base = format!(
        "{}/Library/Application Support/Code/User/workspaceStorage",
        home
    );

    // Find the most recently modified workspace state.vscdb
    let mut newest_db: Option<(String, std::time::SystemTime)> = None;

    if let Ok(entries) = std::fs::read_dir(&workspace_storage_base) {
        for entry in entries.flatten() {
            let state_db = entry.path().join("state.vscdb");
            if state_db.exists()
                && let Ok(meta) = state_db.metadata()
                && let Ok(modified) = meta.modified()
                && (newest_db.is_none() || modified > newest_db.as_ref().unwrap().1)
            {
                newest_db = Some((state_db.to_string_lossy().to_string(), modified));
            }
        }
    }

    let state_db_path = newest_db
        .map(|(path, _)| path)
        .ok_or_else(|| anyhow::anyhow!("No workspace state database found"))?;

    // Open database in read-only mode
    let conn = Connection::open_with_flags(
        &state_db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Query for the editor state - this contains the active file
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'memento/workbench.parts.editor'",
        [],
        |row| row.get(0),
    );

    if let Ok(json_str) = result
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str)
    {
        // Navigate: editorpart.state -> serializedGrid -> root -> data[0] -> data -> editors[0] -> value (JSON string)
        if let Some(state) = json.get("editorpart.state")
            && let Some(grid) = state.get("serializedGrid")
            && let Some(root) = grid.get("root")
            && let Some(path) = find_fs_path_in_editor_tree(root)
            && std::path::Path::new(&path).exists()
        {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!(
        "Could not find active file in VSCode workspace state"
    ))
}

/// Recursively search the editor tree for fsPath
fn find_fs_path_in_editor_tree(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::Object(map) => {
            // Check if this node has editors array
            if let Some(serde_json::Value::Array(editors)) = map.get("editors") {
                for editor in editors {
                    if let Some(serde_json::Value::String(value_str)) = editor.get("value") {
                        // Parse the nested JSON string
                        if let Ok(inner) = serde_json::from_str::<serde_json::Value>(value_str)
                            && let Some(serde_json::Value::Object(resource)) =
                                inner.get("resourceJSON")
                            && let Some(serde_json::Value::String(fs_path)) = resource.get("fsPath")
                        {
                            return Some(fs_path.clone());
                        }
                    }
                }
            }

            // Check data array (for branch nodes)
            if let Some(serde_json::Value::Array(data)) = map.get("data") {
                for item in data {
                    if let Some(path) = find_fs_path_in_editor_tree(item) {
                        return Some(path);
                    }
                }
            }

            // Also check direct data object
            if let Some(data) = map.get("data")
                && let Some(path) = find_fs_path_in_editor_tree(data)
            {
                return Some(path);
            }

            None
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(path) = find_fs_path_in_editor_tree(item) {
                    return Some(path);
                }
            }
            None
        }
        _ => None,
    }
}

/// Fallback: Use lsof to find files opened by VSCode
fn get_active_file_via_lsof() -> std::result::Result<String, JsErrorBox> {
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        let lsof_output = Command::new("lsof")
            .args(["-c", "Code", "-Fn"])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to run lsof: {}", e)))?;

        if !lsof_output.status.success() {
            return Err(JsErrorBox::new(
                "Error",
                "Failed to list VSCode open files".to_string(),
            ));
        }

        let lsof_str = String::from_utf8_lossy(&lsof_output.stdout);

        let code_extensions = [
            ".js", ".ts", ".jsx", ".tsx", ".py", ".rs", ".go", ".java", ".c", ".cpp", ".h", ".hpp",
            ".cs", ".rb", ".php", ".swift", ".kt", ".scala", ".html", ".css", ".scss", ".sass",
            ".less", ".json", ".yaml", ".yml", ".xml", ".md", ".txt", ".sh", ".bash", ".vue",
            ".svelte", ".astro", ".toml", ".cfg", ".ini", ".env",
        ];

        let mut candidates: Vec<(String, std::time::SystemTime)> = Vec::new();

        for line in lsof_str.lines() {
            if !line.starts_with('n') {
                continue;
            }

            let file_path = &line[1..];

            if file_path.contains(".app/")
                || file_path.contains("/Library/")
                || file_path.starts_with("/dev/")
                || file_path.starts_with("/private/var/")
                || file_path.contains("node_modules/")
                || !file_path.starts_with("/")
            {
                continue;
            }

            let has_code_ext = code_extensions.iter().any(|ext| file_path.ends_with(ext));
            if !has_code_ext {
                continue;
            }

            if let Ok(metadata) = std::fs::metadata(file_path)
                && let Ok(modified) = metadata.modified()
            {
                candidates.push((file_path.to_string(), modified));
            }
        }

        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((path, _)) = candidates.first() {
            match std::fs::read_to_string(path) {
                Ok(content) => return Ok(content),
                Err(e) => {
                    return Err(JsErrorBox::new(
                        "Error",
                        format!("Failed to read file {}: {}", path, e),
                    ));
                }
            }
        }

        Err(JsErrorBox::new(
            "Error",
            "No active code files found in VSCode".to_string(),
        ))
    } else if cfg!(target_os = "windows") {
        Err(JsErrorBox::new(
            "Error",
            "get_active_file_content is not yet supported on Windows".to_string(),
        ))
    } else {
        Err(JsErrorBox::new(
            "Error",
            "get_active_file_content is not supported on this OS".to_string(),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use sapphillon_core::proto::sapphillon::v1::PermissionType;
    use sapphillon_core::workflow::CoreWorkflowCode;

    #[test]
    fn test_vscode_open_folder_in_workflow() {
        // Create a temp directory
        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.to_str().unwrap().to_string();
        let escaped_path = tmp_path.replace(r"\", r"\\");

        let code = format!(
            "const path = {escaped_path:?}; const result = vscode.open_folder(path); console.log(result);"
        );

        let perm = PluginFunctionPermissions {
            plugin_function_id: vscode_open_folder_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![Permission {
                    display_name: "VSCode Access".to_string(),
                    description: "Allows VSCode control".to_string(),
                    permission_type: PermissionType::Execute as i32,
                    permission_level: PermissionLevel::Unspecified as i32,
                    resource: vec![],
                }],
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_vscode_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        // The result might be "ok" if VSCode is installed, or an error if not
        // We just check that we got some result
        assert!(!workflow.result[0].result.is_empty());
    }

    #[test]
    fn test_permission_denied_in_workflow() {
        let code = r#"
            vscode.open_folder("/tmp");
        "#;

        // Use empty permissions list to trigger permission denial
        let perm = PluginFunctionPermissions {
            plugin_function_id: vscode_open_folder_plugin_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: vec![],
            },
        };

        let mut workflow = CoreWorkflowCode::new(
            "test".to_string(),
            code.to_string(),
            vec![core_vscode_plugin_package()],
            1,
            Some(perm.clone()),
            Some(perm),
        );

        workflow.run();
        assert_eq!(workflow.result.len(), 1);
        let actual = &workflow.result[0].result;
        assert!(
            actual.to_lowercase().contains("permissiondenied")
                || actual.to_lowercase().contains("permission denied")
                || actual.contains("Uncaught"),
            "Unexpected workflow result: {actual}"
        );
    }
}
