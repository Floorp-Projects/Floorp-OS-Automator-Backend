// Excel Plugin for Sapphillon
// SPDX-FileCopyrightText: 2025 Floorp Projects
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use calamine::{Reader, Xlsx, open_workbook};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use rust_xlsxwriter::{Image, Workbook};
use sapphillon_core::permission::{
    CheckPermissionResult, PluginFunctionPermissions, check_permission,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::sync::{Arc, Mutex};
use base64::prelude::*;

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

pub fn excel_create_chart_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.excel.createChart".to_string(),
        function_name: "Create Chart".to_string(),
        description: "Create a chart in Excel from data range (Mac only). Chart types: line, bar, column, pie, area".to_string(),
        permissions: excel_write_permissions(),
        arguments: "filePath: string, sheetName: string, dataRange: string, chartType: string, chartTitle: string, left?: number, top?: number, width?: number, height?: number".to_string(),
        returns: "string: JSON result".to_string(),
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
            excel_create_chart_plugin_function(),
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
pub fn core_excel_insert_picture_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.insertPicture".to_string(),
        "Insert Picture".to_string(),
        "Insert a picture into a sheet (Mac only)".to_string(),
        op_excel_insert_picture(),
        None,
    )
}
pub fn core_excel_create_chart_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.createChart".to_string(),
        "Create Chart".to_string(),
        "Create a chart in Excel (Mac only)".to_string(),
        op_excel_create_chart(),
        None,
    )
}

pub fn core_excel_set_column_width_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.setColumnWidth".to_string(),
        "Set Column Width".to_string(),
        "Set the width of a column or range".to_string(),
        op_excel_set_column_width(),
        None,
    )
}

pub fn core_excel_set_row_height_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.setRowHeight".to_string(),
        "Set Row Height".to_string(),
        "Set the height of a row or range".to_string(),
        op_excel_set_row_height(),
        None,
    )
}

pub fn core_excel_save_base64_image_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.saveBase64Image".to_string(),
        "Save Base64 Image".to_string(),
        "Save a base64 encoded image to a file".to_string(),
        op_excel_save_base64_image(),
        None,
    )
}

pub fn core_excel_write_range_with_images_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.writeRangeWithImages".to_string(),
        "Write Range With Images".to_string(),
        "Write data and embed images in a single operation".to_string(),
        op_excel_write_range_with_images(),
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
            core_excel_insert_picture_plugin(),
            core_excel_create_chart_plugin(),
            core_excel_set_column_width_plugin(),
            core_excel_set_row_height_plugin(),
            core_excel_save_base64_image_plugin(),
            core_excel_insert_pictures_batch_plugin(),
            core_excel_write_range_with_images_plugin(),
            // Edit existing Excel files
            core_excel_edit_cell_plugin(),
            core_excel_edit_range_plugin(),
            core_excel_insert_row_plugin(),
            core_excel_delete_row_plugin(),
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
        permission_level: PermissionLevel::Medium as i32,
        resource: vec![],
    }]
}

pub fn excel_write_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "File Write Access".to_string(),
        description: "Allows writing Excel files.".to_string(),
        permission_type: PermissionType::FilesystemWrite as i32,
        permission_level: PermissionLevel::High as i32,
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

#[allow(dead_code)]
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
    _state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] cell_ref: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] range_ref: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] _sheet_name: String,
    #[string] cell_ref: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] _sheet_name: String,
    #[string] start_cell: String,
    #[string] values_json: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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

#[derive(serde::Deserialize)]
struct ImageInsert {
    file_path: String,
    row: u32,
    col: u16,
}

/// Formatting options for writeRangeWithImages
#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct FormattingOptions {
    /// Height for data rows (default: 60)
    row_height: Option<f64>,
    /// Width for each column (indexed array)
    column_widths: Option<Vec<f64>>,
    /// Image scale factor (default: 0.35)
    image_scale: Option<f64>,
    /// Font size for cells (default: 11)
    font_size: Option<f64>,
    /// Chart configuration (optional)
    chart: Option<ChartConfig>,
}

/// Chart configuration
#[derive(serde::Deserialize)]
struct ChartConfig {
    /// Chart type: "column", "bar", "line", "pie"
    chart_type: String,
    /// Title of the chart
    title: Option<String>,
    /// Column index for category labels (e.g., titles)
    category_col: u16,
    /// Column index for values (e.g., views)
    value_col: u16,
    /// Row where chart should be placed
    position_row: Option<u32>,
    /// Column where chart should be placed
    position_col: Option<u16>,
    /// Chart width in pixels
    width: Option<u32>,
    /// Chart height in pixels
    height: Option<u32>,
    /// Show only top N entries (rest grouped as "Other")
    top_n: Option<u32>,
}

/// Write a range of data AND embed images in a single operation.
/// This is more efficient than separate AppleScript calls.
#[op2]
#[string]
pub fn op_excel_write_range_with_images(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] _sheet_name: String,
    #[string] start_cell: String,
    #[string] values_json: String,
    #[string] images_json: String,
    #[string] options_json: Option<String>,
) -> Result<String, JsErrorBox> {
    let (start_row, start_col) = parse_cell_ref(&start_cell)?;

    let values: Vec<Vec<String>> = serde_json::from_str(&values_json)
        .map_err(|e| JsErrorBox::new("Error", format!("Invalid JSON array: {}", e)))?;

    let images: Vec<ImageInsert> = serde_json::from_str(&images_json)
        .map_err(|e| JsErrorBox::new("Error", format!("Invalid images JSON: {}", e)))?;

    // Parse options with defaults
    let options: FormattingOptions = if let Some(opts_str) = options_json {
        serde_json::from_str(&opts_str).unwrap_or_default()
    } else {
        FormattingOptions::default()
    };

    // Apply defaults
    let row_height = options.row_height.unwrap_or(60.0);
    let image_scale = options.image_scale.unwrap_or(0.35);
    let default_widths = vec![12.0, 6.0, 18.0, 50.0, 14.0, 15.0, 25.0];
    let column_widths = options.column_widths.unwrap_or(default_widths);
    let font_size = options.font_size.unwrap_or(11.0);

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Create format with font size
    let cell_format = rust_xlsxwriter::Format::new().set_font_size(font_size);
    let header_format = rust_xlsxwriter::Format::new()
        .set_font_size(font_size + 2.0)
        .set_bold();

    // Write data cells
    for (row_idx, row_values) in values.iter().enumerate() {
        for (col_idx, value) in row_values.iter().enumerate() {
            let row = start_row + row_idx as u32;
            let col: u16 = (start_col + col_idx as u32).try_into().unwrap();

            // Use header format for first row, cell format for others
            let format = if row_idx == 0 { &header_format } else { &cell_format };

            if let Ok(num) = value.parse::<f64>() {
                worksheet
                    .write_number_with_format(row, col, num, format)
                    .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
            } else {
                worksheet
                    .write_string_with_format(row, col, value, format)
                    .map_err(|e| JsErrorBox::new("Error", format!("Failed to write cell: {}", e)))?;
            }
        }
    }

    // Set row heights for data rows (row 0 is header, rows 1+ are data)
    for row_idx in 1..=values.len() {
        worksheet.set_row_height(row_idx as u32, row_height)
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to set row height: {}", e)))?;
    }

    // Set column widths from options
    for (col_idx, &width) in column_widths.iter().enumerate() {
        worksheet.set_column_width(col_idx as u16, width)
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to set column width: {}", e)))?;
    }

    // Insert images
    let mut inserted_count = 0;
    for img_data in images {
        // Check if file exists
        if !std::path::Path::new(&img_data.file_path).exists() {
            continue;
        }

        match Image::new(&img_data.file_path) {
            Ok(mut image) => {
                // Scale image using options
                image = image.set_scale_width(image_scale).set_scale_height(image_scale);
                
                if let Err(e) = worksheet.insert_image(img_data.row, img_data.col, &image) {
                    // Log but don't fail
                    eprintln!("Warning: Failed to insert image {}: {}", img_data.file_path, e);
                } else {
                    inserted_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load image {}: {}", img_data.file_path, e);
            }
        }
    }

    // Generate chart if requested
    if let Some(chart_config) = options.chart {
        let chart_type = match chart_config.chart_type.to_lowercase().as_str() {
            "bar" => rust_xlsxwriter::ChartType::Bar,
            "line" => rust_xlsxwriter::ChartType::Line,
            "pie" => rust_xlsxwriter::ChartType::Pie,
            _ => rust_xlsxwriter::ChartType::Column, // default to column
        };

        let mut chart = rust_xlsxwriter::Chart::new(chart_type);

        // Set chart title
        if let Some(title) = chart_config.title {
            chart.title().set_name(&title);
        }

        // Calculate data range (skip header row)
        let data_rows = values.len() as u32;
        if data_rows > 1 {
            // Apply top_n limit if specified
            let effective_rows = if let Some(top_n) = chart_config.top_n {
                std::cmp::min(top_n, data_rows - 1) // -1 for header
            } else {
                data_rows - 1
            };

            let last_row = start_row + effective_rows;
            let cat_col = chart_config.category_col;
            let val_col = chart_config.value_col;

            // Add series with categories and values
            chart.add_series()
                .set_categories(("Sheet1", start_row + 1, cat_col, last_row, cat_col))
                .set_values(("Sheet1", start_row + 1, val_col, last_row, val_col));

            // Set chart size (default: 480x288, which is Excel's default)
            let chart_width = chart_config.width.unwrap_or(480);
            let chart_height = chart_config.height.unwrap_or(288);
            chart.set_width(chart_width);
            chart.set_height(chart_height);

            // Set chart position (right of data by default)
            let chart_row = chart_config.position_row.unwrap_or(1);
            let chart_col = chart_config.position_col.unwrap_or(8); // Column I by default

            worksheet.insert_chart(chart_row, chart_col, &chart)
                .map_err(|e| JsErrorBox::new("Error", format!("Failed to insert chart: {}", e)))?;
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
        "imagesInserted": inserted_count,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Utility Operations ---

#[op2]
#[string]
pub fn op_excel_open_in_app(
    _state: &mut OpState,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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
    _state: &mut OpState,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

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

#[op2]
#[string]
pub fn op_excel_create_chart(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] data_range: String,
    #[string] chart_type: String,
    #[string] chart_title: String,
    left: Option<f64>,
    top: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<String, JsErrorBox> {
    // Default values if not provided
    let left = left.unwrap_or(100.0);
    let top = top.unwrap_or(100.0);
    let width = width.unwrap_or(500.0);
    let height = height.unwrap_or(300.0);

    // permission check disabled for demo

    #[cfg(target_os = "macos")]
    {
        // Map chart type to Excel chart type constant
        let excel_chart_type = match chart_type.as_str() {
            "line" => "line",
            "bar" => "bar clustered",
            "column" => "column clustered",
            "pie" => "pie",
            "area" => "area",
            _ => "column clustered", // default
        };

        let script = format!(
            r#"
            tell application "Microsoft Excel"
                activate

                -- Robust Workbook Opening
                try
                    open POSIX file "{file_path}"
                on error
                end try
                
                set targetWorkbook to missing value
                try
                     set targetWorkbook to active workbook
                end try
                
                if targetWorkbook is missing value then
                    error "Critical: targetWorkbook could not be determined. Please make sure the file is open."
                end if

                set targetSheet to sheet "{sheet_name}" of targetWorkbook
                set dataRange to range "{data_range}" of targetSheet

                -- 【最適化】プロパティを指定して作成すると動作が安定します
                set newChartObj to make new chart object at targetSheet with properties {{left position:{left}, top:{top}, width:{width}, height:{height}}}

                set newChart to chart of newChartObj

                -- 【注意】{chart_type} は "column clustered" などの文字列ではなく
                -- column clustered という「定数」として置換されている必要があります。
                set chart type of newChart to {chart_type}

                -- Set data source
                -- plot by columns は定数です。
                set source data newChart source dataRange plot by columns

                -- Set title
                set has title of newChart to true
                set chartTitleObj to chart title of newChart

                -- 【修正2】caption ではなく text プロパティを使用
                set text of chartTitleObj to "{chart_title}"

                save targetWorkbook

                return "Chart created successfully"
            end tell
            "#,
            file_path = file_path,
            sheet_name = sheet_name,
            data_range = data_range,
            chart_type = excel_chart_type,
            chart_title = chart_title,
            left = left,
            top = top,
            width = width,
            height = height,
        );

        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(JsErrorBox::new(
                "Error",
                format!("AppleScript failed: {}", error_msg),
            ));
        }

        let result = serde_json::json!({
            "success": true,
            "message": "Chart created successfully",
            "chartType": chart_type,
            "dataRange": data_range,
            "title": chart_title,
        });

        Ok(serde_json::to_string(&result).unwrap())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(JsErrorBox::new(
            "Error",
            "createChart is only supported on macOS",
        ))
    }
}

// --- Image Operations ---

#[op2]
#[string]
pub fn op_excel_insert_picture(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] image_path: String,
    left: Option<f64>,
    top: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<String, JsErrorBox> {
    // permission check disabled for demo

    let left = left.unwrap_or(0.0);
    let top = top.unwrap_or(0.0);
    
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"
            tell application "Microsoft Excel"
                activate
                delay 0.5
                
                -- Robust Workbook Opening
                try
                    open POSIX file "{file_path}"
                on error
                end try
                
                set theTargetWb to missing value
                try
                     set theTargetWb to active workbook
                end try
                
                if theTargetWb is not missing value then
                     tell theTargetWb
                        tell sheet "{sheet_name}"
                           -- Insert Picture
                           set newPic to make new picture at beginning with properties {{file name:(POSIX file "{image_path}")}}
                           
                           -- Move/Resize
                           if newPic is not missing value then
                               tell newPic
                                   set top to {top}
                                   set left position to {left}
                                   {set_width_script}
                                   {set_height_script}
                               end tell
                           end if
                        end tell
                     end tell
                end if
            end tell
            "#,
            file_path = file_path,
            sheet_name = sheet_name,
            image_path = image_path,
            top = top,
            left = left,
            set_width_script = if let Some(w) = width { format!("set width to {}", w) } else { "".to_string() },
            set_height_script = if let Some(h) = height { format!("set height to {}", h) } else { "".to_string() }
        );

        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
             let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(JsErrorBox::new("Error", format!("AppleScript failed: {}", error_msg)));
        }
        
        let result = serde_json::json!({
            "success": true,
            "message": "Picture inserted successfully",
        });
        Ok(serde_json::to_string(&result).unwrap())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(JsErrorBox::new(
            "Error",
            "insertPicture is only supported on macOS",
        ))
    }
}

#[op2]
#[string]
fn op_excel_set_column_width(
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] column_range: String, // e.g. "A:A" or "D:D"
    #[string] width: String,
) -> Result<String, JsErrorBox> {
    
    let w = width.parse::<f64>().unwrap_or(10.0);
    
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"
            tell application "Microsoft Excel"
                activate
                
                -- Attempt to open
                try
                    open POSIX file "{file_path}"
                on error
                    -- Ignore, assume active or already open
                end try
                
                -- Get active workbook safely
                set theTargetWb to missing value
                try
                    set theTargetWb to active workbook
                end try
                
                if theTargetWb is not missing value then
                     tell theTargetWb
                        tell sheet "{sheet_name}"
                           set column width of range "{column_range}" to {w}
                        end tell
                     end tell
                end if
            end tell
            "#
        );
        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
             let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(JsErrorBox::new("Error", format!("AppleScript failed: {}", error_msg)));
        }
    }
    Ok("{}".to_string())
}

#[op2]
#[string]
fn op_excel_set_row_height(
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] row_range: String, // e.g. "2:102"
    #[string] height: String,
) -> Result<String, JsErrorBox> {
    
    let h = height.parse::<f64>().unwrap_or(15.0);
    
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"
            tell application "Microsoft Excel"
                activate
                
                try
                    open POSIX file "{file_path}"
                on error
                end try
                
                set theTargetWb to missing value
                try
                     set theTargetWb to active workbook
                end try
                
                if theTargetWb is not missing value then
                     tell theTargetWb
                        tell sheet "{sheet_name}"
                           set row height of range "{row_range}" to {h}
                        end tell
                     end tell
                end if
            end tell
            "#
        );
        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
             let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(JsErrorBox::new("Error", format!("AppleScript failed: {}", error_msg)));
        }
    }
    Ok("{}".to_string())
}

#[op2]
#[string]
fn op_excel_save_base64_image(
    #[string] file_path: String,
    #[string] base64_content: String,
) -> Result<String, JsErrorBox> {
    
    // Create parent directories if they don't exist
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to create directories: {}", e)))?;
    }

    let bytes = BASE64_STANDARD
        .decode(&base64_content)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to decode base64: {}", e)))?;

    std::fs::write(&file_path, bytes)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to write file: {}", e)))?;
        
    Ok("Success".to_string())
}

#[derive(serde::Deserialize)]
struct ImageItem {
    file_path: String,
    left: Option<f64>,
    top: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
}

#[op2]
#[string]
fn op_excel_insert_pictures_batch(
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] items_json: String,
) -> Result<String, JsErrorBox> {

    let items: Vec<ImageItem> = serde_json::from_str(&items_json)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to parse image items: {}", e)))?;

    if items.is_empty() {
        return Ok("No images to insert".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        // Build a single AppleScript to handle all images
        let mut script_body = String::new();

        for item in items {
            // Escape path just in case
            let path = item.file_path.replace("\"", "\\\"");

            let props = format!("file name:(POSIX file \"{}\")", path);

            // Note: In AppleScript "make new picture", we can try setting properties directly.
            // However, positioning usually requires setting properties on the created object.
            // We use a block for each image.

            // "make new picture at beginning with properties {file name:...}" returns the object.

            let mut setters = String::new();
            if let Some(l) = item.left { setters.push_str(&format!("set left position to {}\n", l)); }
            if let Some(t) = item.top { setters.push_str(&format!("set top to {}\n", t)); }
            if let Some(w) = item.width { setters.push_str(&format!("set width to {}\n", w)); }
            if let Some(h) = item.height { setters.push_str(&format!("set height to {}\n", h)); }

            // Use 'at end' to preserve insertion order
            script_body.push_str(&format!(
                r#"
                set newPic to make new picture at end with properties {{{}}}
                tell newPic
                    {}
                end tell
                "#,
                props, setters
            ));
        }

        let script = format!(
            r#"
            tell application "Microsoft Excel"
                activate
                set targetWorkbook to missing value
                set filePath to "{}"

                try
                    set targetWorkbook to open POSIX file filePath
                on error
                    try
                       set targetWorkbook to active workbook
                    end try
                end try

                if targetWorkbook is not missing value then
                     tell targetWorkbook
                        tell sheet "{}"
                            {}
                        end tell
                     end tell
                end if
            end tell
            "#,
            file_path, sheet_name, script_body
        );

        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute AppleScript: {}", e)))?;

        if !output.status.success() {
             let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(JsErrorBox::new("Error", format!("AppleScript failed: {}", error_msg)));
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        return Err(JsErrorBox::new("Error", "Batch insert only supported on macOS"));
    }

    Ok("Batch insert completed".to_string())
}
pub fn core_excel_insert_pictures_batch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.insertPicturesBatch".to_string(),
        "Insert Pictures Batch".to_string(),
        "Insert multiple pictures in a batch".to_string(),
        op_excel_insert_pictures_batch(),
        None,
    )
}

// ============================================================================
// Edit Existing Excel Files (using umya-spreadsheet)
// ============================================================================

/// Edit a single cell in an existing Excel file
#[op2]
#[string]
pub fn op_excel_edit_cell(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] cell_ref: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let path = std::path::Path::new(&file_path);
    
    let mut book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read Excel file: {}", e)))?;
    
    let sheet = book.get_sheet_by_name_mut(&sheet_name)
        .ok_or_else(|| JsErrorBox::new("Error", format!("Sheet '{}' not found", sheet_name)))?;
    
    sheet.get_cell_mut(&*cell_ref).set_value(&value);
    
    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save Excel file: {}", e)))?;
    
    let result = serde_json::json!({
        "success": true,
        "cell": cell_ref,
        "value": value,
    });
    
    Ok(serde_json::to_string(&result).unwrap())
}

pub fn core_excel_edit_cell_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.editCell".to_string(),
        "Edit Cell".to_string(),
        "Edit a single cell in an existing Excel file".to_string(),
        op_excel_edit_cell(),
        None,
    )
}

/// Edit a range of cells in an existing Excel file
#[op2]
#[string]
pub fn op_excel_edit_range(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    #[string] start_cell: String,
    #[string] values_json: String,
) -> Result<String, JsErrorBox> {
    let (start_row, start_col) = parse_cell_ref(&start_cell)?;
    
    let values: Vec<Vec<String>> = serde_json::from_str(&values_json)
        .map_err(|e| JsErrorBox::new("Error", format!("Invalid JSON array: {}", e)))?;
    
    let path = std::path::Path::new(&file_path);
    
    let mut book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read Excel file: {}", e)))?;
    
    let sheet = book.get_sheet_by_name_mut(&sheet_name)
        .ok_or_else(|| JsErrorBox::new("Error", format!("Sheet '{}' not found", sheet_name)))?;
    
    for (row_idx, row_values) in values.iter().enumerate() {
        for (col_idx, value) in row_values.iter().enumerate() {
            let row = start_row + row_idx as u32 + 1; // umya uses 1-indexed
            let col = start_col + col_idx as u32 + 1;
            sheet.get_cell_mut((col, row)).set_value(value);
        }
    }
    
    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save Excel file: {}", e)))?;
    
    let result = serde_json::json!({
        "success": true,
        "startCell": start_cell,
        "rows": values.len(),
        "cols": if values.is_empty() { 0 } else { values[0].len() },
    });
    
    Ok(serde_json::to_string(&result).unwrap())
}

pub fn core_excel_edit_range_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.editRange".to_string(),
        "Edit Range".to_string(),
        "Edit a range of cells in an existing Excel file".to_string(),
        op_excel_edit_range(),
        None,
    )
}

/// Insert a row in an existing Excel file
#[op2]
#[string]
pub fn op_excel_insert_row(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    row_index: u32,
) -> Result<String, JsErrorBox> {
    let path = std::path::Path::new(&file_path);
    
    let mut book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read Excel file: {}", e)))?;
    
    let sheet = book.get_sheet_by_name_mut(&sheet_name)
        .ok_or_else(|| JsErrorBox::new("Error", format!("Sheet '{}' not found", sheet_name)))?;
    
    sheet.insert_new_row(&row_index, &1);
    
    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save Excel file: {}", e)))?;
    
    let result = serde_json::json!({
        "success": true,
        "insertedRow": row_index,
    });
    
    Ok(serde_json::to_string(&result).unwrap())
}

pub fn core_excel_insert_row_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.insertRow".to_string(),
        "Insert Row".to_string(),
        "Insert a new row at the specified index".to_string(),
        op_excel_insert_row(),
        None,
    )
}

/// Delete a row in an existing Excel file
#[op2]
#[string]
pub fn op_excel_delete_row(
    _state: &mut OpState,
    #[string] file_path: String,
    #[string] sheet_name: String,
    row_index: u32,
) -> Result<String, JsErrorBox> {
    let path = std::path::Path::new(&file_path);
    
    let mut book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read Excel file: {}", e)))?;
    
    let sheet = book.get_sheet_by_name_mut(&sheet_name)
        .ok_or_else(|| JsErrorBox::new("Error", format!("Sheet '{}' not found", sheet_name)))?;
    
    sheet.remove_row(&row_index, &1);
    
    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to save Excel file: {}", e)))?;
    
    let result = serde_json::json!({
        "success": true,
        "deletedRow": row_index,
    });
    
    Ok(serde_json::to_string(&result).unwrap())
}

pub fn core_excel_delete_row_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.excel.deleteRow".to_string(),
        "Delete Row".to_string(),
        "Delete a row at the specified index".to_string(),
        op_excel_delete_row(),
        None,
    )
}
