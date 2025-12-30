// Excel Plugin for Sapphillon
// SPDX-FileCopyrightText: 2025 Floorp Projects
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use calamine::{Reader, Xlsx, open_workbook};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use rust_xlsxwriter::Workbook;
use sapphillon_core::permission::{
    CheckPermissionResult, PluginFunctionPermissions, check_permission,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::sync::{Arc, Mutex};

// ============================================================================
// Plugin Function Definitions
// ============================================================================

// --- Read Operations ---

pub fn excel_get_sheet_names_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.getSheetNames".to_string(),
        function_name: "Get Sheet Names".to_string(),
        description: "Get all sheet names from an Excel file".to_string(),
        permissions: excel_read_permissions(),
        arguments: "filePath: string".to_string(),
        returns: "string: JSON array of sheet names".to_string(),
    }
}

pub fn excel_read_cell_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.readCell".to_string(),
        function_name: "Read Cell".to_string(),
        description: "Read a single cell value from an Excel file".to_string(),
        permissions: excel_read_permissions(),
        arguments: "filePath: string, sheetName: string, cellRef: string".to_string(),
        returns: "string: cell value".to_string(),
    }
}

pub fn excel_read_range_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.readRange".to_string(),
        function_name: "Read Range".to_string(),
        description: "Read a range of cells from an Excel file".to_string(),
        permissions: excel_read_permissions(),
        arguments: "filePath: string, sheetName: string, rangeRef: string".to_string(),
        returns: "string: JSON 2D array of cell values".to_string(),
    }
}

// --- Write Operations ---

pub fn excel_create_workbook_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.createWorkbook".to_string(),
        function_name: "Create Workbook".to_string(),
        description: "Create a new Excel workbook".to_string(),
        permissions: excel_write_permissions(),
        arguments: "filePath: string".to_string(),
        returns: "string: result".to_string(),
    }
}

pub fn excel_write_cell_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.writeCell".to_string(),
        function_name: "Write Cell".to_string(),
        description: "Write a value to a cell in an Excel file".to_string(),
        permissions: excel_write_permissions(),
        arguments: "filePath: string, sheetName: string, cellRef: string, value: string".to_string(),
        returns: "string: result".to_string(),
    }
}

pub fn excel_write_range_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.writeRange".to_string(),
        function_name: "Write Range".to_string(),
        description: "Write a 2D array of values to a range in an Excel file".to_string(),
        permissions: excel_write_permissions(),
        arguments: "filePath: string, sheetName: string, startCell: string, valuesJson: string".to_string(),
        returns: "string: result".to_string(),
    }
}

pub fn excel_add_sheet_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.addSheet".to_string(),
        function_name: "Add Sheet".to_string(),
        description: "Add a new sheet to an Excel file".to_string(),
        permissions: excel_write_permissions(),
        arguments: "filePath: string, sheetName: string".to_string(),
        returns: "string: result".to_string(),
    }
}

// --- Utility Operations ---

pub fn excel_open_in_app_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.openInApp".to_string(),
        function_name: "Open In App".to_string(),
        description: "Open an Excel file in the default application".to_string(),
        permissions: excel_read_permissions(),
        arguments: "filePath: string".to_string(),
        returns: "string: result".to_string(),
    }
}

pub fn excel_get_open_workbooks_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.getOpenWorkbooks".to_string(),
        function_name: "Get Open Workbooks".to_string(),
        description: "Get file paths of all workbooks currently open in Excel (Mac only)".to_string(),
        permissions: excel_read_permissions(),
        arguments: "".to_string(),
        returns: "string: JSON array of file paths".to_string(),
    }
}

pub fn excel_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.excel".to_string(),
        package_name: "Excel".to_string(),
        description: "Cross-platform Excel file operations for workflow automation.".to_string(),
        functions: vec![
            // Read operations
            excel_get_sheet_names_plugin_function(),
            excel_read_cell_plugin_function(),
            excel_read_range_plugin_function(),
            // Write operations
            excel_create_workbook_plugin_function(),
            excel_write_cell_plugin_function(),
            excel_write_range_plugin_function(),
            excel_add_sheet_plugin_function(),
            // Utility operations
            excel_open_in_app_plugin_function(),
            excel_get_open_workbooks_plugin_function(),
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

pub fn core_excel_get_sheet_names_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.getSheetNames".to_string(),
        "Get Sheet Names".to_string(),
        "Get all sheet names from an Excel file".to_string(),
        op_excel_get_sheet_names(),
        Some(include_str!("00_excel.js").to_string()),
    )
}

pub fn core_excel_read_cell_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.readCell".to_string(),
        "Read Cell".to_string(),
        "Read a single cell value".to_string(),
        op_excel_read_cell(),
        None,
    )
}

pub fn core_excel_read_range_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.readRange".to_string(),
        "Read Range".to_string(),
        "Read a range of cells".to_string(),
        op_excel_read_range(),
        None,
    )
}

pub fn core_excel_create_workbook_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.createWorkbook".to_string(),
        "Create Workbook".to_string(),
        "Create a new Excel workbook".to_string(),
        op_excel_create_workbook(),
        None,
    )
}

pub fn core_excel_write_cell_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.writeCell".to_string(),
        "Write Cell".to_string(),
        "Write a value to a cell".to_string(),
        op_excel_write_cell(),
        None,
    )
}

pub fn core_excel_write_range_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.writeRange".to_string(),
        "Write Range".to_string(),
        "Write values to a range".to_string(),
        op_excel_write_range(),
        None,
    )
}

pub fn core_excel_add_sheet_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.addSheet".to_string(),
        "Add Sheet".to_string(),
        "Add a new sheet".to_string(),
        op_excel_add_sheet(),
        None,
    )
}

pub fn core_excel_open_in_app_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.openInApp".to_string(),
        "Open In App".to_string(),
        "Open file in default application".to_string(),
        op_excel_open_in_app(),
        None,
    )
}

pub fn core_excel_get_open_workbooks_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.getOpenWorkbooks".to_string(),
        "Get Open Workbooks".to_string(),
        "Get file paths of workbooks open in Excel (Mac only)".to_string(),
        op_excel_get_open_workbooks(),
        None,
    )
}

pub fn core_excel_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.excel".to_string(),
        "Excel".to_string(),
        vec![
            core_excel_get_sheet_names_plugin(),
            core_excel_read_cell_plugin(),
            core_excel_read_range_plugin(),
            core_excel_create_workbook_plugin(),
            core_excel_write_cell_plugin(),
            core_excel_write_range_plugin(),
            core_excel_add_sheet_plugin(),
            core_excel_open_in_app_plugin(),
            core_excel_get_open_workbooks_plugin(),
        ],
    )
}

// ============================================================================
// Permission Definitions
// ============================================================================

pub fn excel_read_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "File Read Access".to_string(),
        description: "Allows reading Excel files.".to_string(),
        permission_type: PermissionType::FilesystemRead as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

pub fn excel_write_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "File Write Access".to_string(),
        description: "Allows writing Excel files.".to_string(),
        permission_type: PermissionType::FilesystemWrite as i32,
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
// Cell Reference Parsing Helpers
// ============================================================================

fn parse_cell_ref(cell_ref: &str) -> Result<(u32, u32), JsErrorBox> {
    let cell_ref = cell_ref.to_uppercase();
    let mut col_part = String::new();
    let mut row_part = String::new();

    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            col_part.push(c);
        } else if c.is_ascii_digit() {
            row_part.push(c);
        }
    }

    if col_part.is_empty() || row_part.is_empty() {
        return Err(JsErrorBox::new(
            "Error",
            format!("Invalid cell reference: {}", cell_ref),
        ));
    }

    // Convert column letters to 0-based index (A=0, B=1, ..., Z=25, AA=26, ...)
    let col: u32 = col_part
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1))
        - 1;

    // Convert row to 0-based index
    let row: u32 = row_part
        .parse::<u32>()
        .map_err(|_| JsErrorBox::new("Error", format!("Invalid row number: {}", row_part)))?
        - 1;

    Ok((row, col))
}

fn parse_range_ref(range_ref: &str) -> Result<((u32, u32), (u32, u32)), JsErrorBox> {
    let parts: Vec<&str> = range_ref.split(':').collect();
    if parts.len() != 2 {
        return Err(JsErrorBox::new(
            "Error",
            format!("Invalid range reference: {}. Expected format: A1:B2", range_ref),
        ));
    }

    let start = parse_cell_ref(parts[0])?;
    let end = parse_cell_ref(parts[1])?;

    Ok((start, end))
}

// ============================================================================
// Op2 Functions (Deno Runtime Operations)
// ============================================================================

// --- Read Operations ---

#[op2]
#[string]
pub fn op_excel_get_sheet_names(
    state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.getSheetNames",
        excel_read_permissions(),
    )?;

    let workbook: Xlsx<_> = open_workbook(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to open workbook: {}", e)))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    let result = serde_json::json!({
        "sheets": sheet_names,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_read_cell(
    state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] cell_ref: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.readCell",
        excel_read_permissions(),
    )?;

    let (row, col) = parse_cell_ref(&cell_ref)?;

    let mut workbook: Xlsx<_> = open_workbook(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to open workbook: {}", e)))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

    let value = range
        .get((row as usize, col as usize))
        .map(|cell| format!("{}", cell))
        .unwrap_or_default();

    let result = serde_json::json!({
        "value": value,
        "cell": cell_ref,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_read_range(
    state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] range_ref: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.readRange",
        excel_read_permissions(),
    )?;

    let ((start_row, start_col), (end_row, end_col)) = parse_range_ref(&range_ref)?;

    let mut workbook: Xlsx<_> = open_workbook(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to open workbook: {}", e)))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

    let mut data: Vec<Vec<String>> = Vec::new();

    for row in start_row..=end_row {
        let mut row_data: Vec<String> = Vec::new();
        for col in start_col..=end_col {
            let value = range
                .get((row as usize, col as usize))
                .map(|cell| format!("{}", cell))
                .unwrap_or_default();
            row_data.push(value);
        }
        data.push(row_data);
    }

    let result = serde_json::json!({
        "data": data,
        "range": range_ref,
        "rows": data.len(),
        "cols": if data.is_empty() { 0 } else { data[0].len() },
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Write Operations ---

#[op2]
#[string]
pub fn op_excel_create_workbook(
    state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.createWorkbook",
        excel_write_permissions(),
    )?;

    let mut workbook = Workbook::new();
    let _worksheet = workbook.add_worksheet();

    workbook
        .save(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save workbook: {}", e)))?;

    let result = serde_json::json!({
        "success": true,
        "filePath": file_path,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_write_cell(
    state: &mut OpState,
    #[string] file_path: String,
    #[string] _sheet_name: String,
    #[string] cell_ref: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.writeCell",
        excel_write_permissions(),
    )?;

    let (row, col) = parse_cell_ref(&cell_ref)?;

    // rust_xlsxwriter creates new files; for editing existing files we'd need to
    // read first then write. For now, we'll create/overwrite.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Try to write as number if possible, otherwise as string
    if let Ok(num) = value.parse::<f64>() {
        worksheet
            .write_number(row, col.try_into().unwrap(), num)
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
    } else {
        worksheet
            .write_string(row, col.try_into().unwrap(), &value)
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
    }

    workbook
        .save(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save workbook: {}", e)))?;

    let result = serde_json::json!({
        "success": true,
        "cell": cell_ref,
        "value": value,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_write_range(
    state: &mut OpState,
    #[string] file_path: String,
    #[string] _sheet_name: String,
    #[string] start_cell: String,
    #[string] values_json: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.writeRange",
        excel_write_permissions(),
    )?;

    let (start_row, start_col) = parse_cell_ref(&start_cell)?;

    let values: Vec<Vec<String>> = serde_json::from_str(&values_json)
        .map_err(|e| JsErrorBox::new("Error", format!("Invalid JSON array: {}", e)))?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (row_idx, row_values) in values.iter().enumerate() {
        for (col_idx, value) in row_values.iter().enumerate() {
            let row = start_row + row_idx as u32;
            let col: u16 = (start_col + col_idx as u32).try_into().unwrap();

            if let Ok(num) = value.parse::<f64>() {
                worksheet
                    .write_number(row, col, num)
                    .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
            } else {
                worksheet
                    .write_string(row, col, value)
                    .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
            }
        }
    }

    workbook
        .save(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save workbook: {}", e)))?;

    let result = serde_json::json!({
        "success": true,
        "startCell": start_cell,
        "rows": values.len(),
        "cols": if values.is_empty() { 0 } else { values[0].len() },
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_add_sheet(
    state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.addSheet",
        excel_write_permissions(),
    )?;

    // Note: rust_xlsxwriter creates new workbooks. For true "add sheet" to existing file,
    // we would need a more complex approach (read with calamine, write with xlsxwriter).
    // For now, this creates a new workbook with the named sheet.

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(&sheet_name)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to set sheet name: {}", e)))?;

    workbook
        .save(&file_path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save workbook: {}", e)))?;

    let result = serde_json::json!({
        "success": true,
        "sheetName": sheet_name,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Utility Operations ---

#[op2]
#[string]
pub fn op_excel_open_in_app(
    state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.openInApp",
        excel_read_permissions(),
    )?;

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open")
        .arg(&file_path)
        .status();

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", &file_path])
        .status();

    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("xdg-open")
        .arg(&file_path)
        .status();

    status.map_err(|e| JsErrorBox::new("Error", format!("Failed to open file: {}", e)))?;

    let result = serde_json::json!({
        "success": true,
        "filePath": file_path,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_excel_get_open_workbooks(
    state: &mut OpState,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.excel.getOpenWorkbooks",
        excel_read_permissions(),
    )?;

    #[cfg(target_os = "macos")]
    {
        let script = r#"
            tell application "Microsoft Excel"
                set wbPaths to {}
                set wbCount to count of workbooks
                repeat with i from 1 to wbCount
                    set wb to workbook i
                    set wbName to name of wb
                    set wbPath to path of wb
                    set end of wbPaths to wbPath & "/" & wbName
                end repeat
                return wbPaths
            end tell
        "#;

        let output = std::process::Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JsErrorBox::new(
                "Error",
                format!("AppleScript failed: {}. Is Excel running?", stderr),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // AppleScript returns comma-separated list: "path1, path2, path3"
        let paths: Vec<String> = if stdout.is_empty() {
            vec![]
        } else {
            stdout.split(", ").map(|s| s.trim().to_string()).collect()
        };

        let result = serde_json::json!({
            "workbooks": paths,
            "count": paths.len(),
        });

        Ok(serde_json::to_string(&result).unwrap())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(JsErrorBox::new(
            "Error",
            "getOpenWorkbooks is only supported on macOS",
        ))
    }
}
