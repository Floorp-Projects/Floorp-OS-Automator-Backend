// @ts-nocheck
// JavaScript glue exposing Excel operations to workflows

// --- Read Operations ---

function excelGetSheetNames(filePath) {
  return Deno.core.ops.op_excel_get_sheet_names(filePath);
}

function excelReadCell(filePath, sheetName, cellRef) {
  return Deno.core.ops.op_excel_read_cell(filePath, sheetName, cellRef);
}

function excelReadRange(filePath, sheetName, rangeRef) {
  return Deno.core.ops.op_excel_read_range(filePath, sheetName, rangeRef);
}

// --- Write Operations ---

function excelCreateWorkbook(filePath) {
  return Deno.core.ops.op_excel_create_workbook(filePath);
}

function excelWriteCell(filePath, sheetName, cellRef, value) {
  return Deno.core.ops.op_excel_write_cell(
    filePath,
    sheetName,
    cellRef,
    String(value)
  );
}

function excelWriteRange(filePath, sheetName, startCell, values) {
  const valuesJson = JSON.stringify(values);
  return Deno.core.ops.op_excel_write_range(
    filePath,
    sheetName,
    startCell,
    valuesJson
  );
}

function excelAddSheet(filePath, sheetName) {
  return Deno.core.ops.op_excel_add_sheet(filePath, sheetName);
}

// --- Utility Operations ---

function excelOpenInApp(filePath) {
  return Deno.core.ops.op_excel_open_in_app(filePath);
}

function excelGetOpenWorkbooks() {
  return Deno.core.ops.op_excel_get_open_workbooks();
}

// --- Export ---

globalThis.excel = {
  // Read operations
  getSheetNames: excelGetSheetNames,
  readCell: excelReadCell,
  readRange: excelReadRange,
  // Write operations
  createWorkbook: excelCreateWorkbook,
  writeCell: excelWriteCell,
  writeRange: excelWriteRange,
  addSheet: excelAddSheet,
  // Utility operations
  openInApp: excelOpenInApp,
  getOpenWorkbooks: excelGetOpenWorkbooks,
};
