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

// --- Utility Operations ---

function excelOpenInApp(filePath) {
  return Deno.core.ops.op_excel_open_in_app(filePath);
}

function excelGetOpenWorkbooks() {
  return Deno.core.ops.op_excel_get_open_workbooks();
}

function excelSetColumnWidth(filePath, sheetName, columnRange, width) {
  return Deno.core.ops.op_excel_set_column_width(
    filePath,
    sheetName,
    columnRange,
    String(width)
  );
}

function excelSetRowHeight(filePath, sheetName, rowRange, height) {
  return Deno.core.ops.op_excel_set_row_height(
    filePath,
    sheetName,
    rowRange,
    String(height)
  );
}

function excelSaveBase64Image(filePath, base64Content) {
  return Deno.core.ops.op_excel_save_base64_image(filePath, base64Content);
}

function excelCreateChart(
  filePath,
  sheetName,
  dataRange,
  chartType,
  chartTitle,
  options
) {
  const { left, top, width, height } = options || {};
  return Deno.core.ops.op_excel_create_chart(
    filePath,
    sheetName,
    dataRange,
    chartType,
    chartTitle,
    left ?? null,
    top ?? null,
    width ?? null,
    height ?? null
  );
}

function excelInsertPicture(filePath, sheetName, imagePath, options) {
  const { left, top, width, height } = options || {};
  return Deno.core.ops.op_excel_insert_picture(
    filePath,
    sheetName,
    imagePath,
    left ?? null,
    top ?? null,
    width ?? null,
    height ?? null
  );
}

function excelInsertPicturesBatch(filePath, sheetName, items) {
  return Deno.core.ops.op_excel_insert_pictures_batch(
    filePath,
    sheetName,
    JSON.stringify(items)
  );
}

function excelWriteRangeWithImages(
  filePath,
  sheetName,
  startCell,
  values,
  images
) {
  return Deno.core.ops.op_excel_write_range_with_images(
    filePath,
    sheetName,
    startCell,
    JSON.stringify(values),
    JSON.stringify(images)
  );
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
  writeRangeWithImages: excelWriteRangeWithImages,
  addSheet: excelAddSheet,
  // Utility operations
  openInApp: excelOpenInApp,
  getOpenWorkbooks: excelGetOpenWorkbooks,
  createChart: excelCreateChart,
  insertPicture: excelInsertPicture,
  insertPicturesBatch: excelInsertPicturesBatch,
  setColumnWidth: excelSetColumnWidth,
  setRowHeight: excelSetRowHeight,
  saveBase64Image: excelSaveBase64Image,
};
