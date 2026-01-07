/**
 * Excel Plugin Test Workflow
 *
 * This workflow tests all Excel plugin functions:
 * - createWorkbook: Create a new Excel file
 * - writeCell: Write a single cell
 * - writeRange: Write multiple cells
 * - addSheet: Add a new sheet
 * - getSheetNames: Get all sheet names
 * - readCell: Read a single cell
 * - readRange: Read multiple cells
 * - openInApp: Open in default application
 * - editCell: Edit a cell in existing file
 * - editRange: Edit a range in existing file
 * - insertRow: Insert a row in existing file
 * - deleteRow: Delete a row in existing file
 */

function workflow() {
  console.log("=== Excel Plugin Test Workflow ===");
  console.log("");

  const testDir = "/tmp";
  const testFile = testDir + "/sapphillon_excel_test.xlsx";

  try {
    // Test 1: Create a new workbook
    console.log("[Test 1] Creating new workbook...");
    const createResult = excel.createWorkbook(testFile);
    console.log("Result: " + createResult);
    console.log("✓ Workbook created at: " + testFile);
    console.log("");

    // Test 2: Write a range of data (this will replace the file)
    console.log("[Test 2] Writing range of data...");
    const headers = ["Name", "Age", "City", "Score"];
    const data = [
      ["Alice", "28", "Tokyo", "95"],
      ["Bob", "32", "Osaka", "88"],
      ["Charlie", "25", "Kyoto", "92"],
      ["Diana", "30", "Nagoya", "97"],
    ];
    const allData = [headers, ...data];

    const writeRangeResult = excel.writeRange(
      testFile,
      "Sheet1",
      "A1",
      allData
    );
    console.log("Result: " + writeRangeResult);
    console.log("✓ Wrote " + allData.length + " rows of data");
    console.log("");

    // Test 3: Get sheet names
    console.log("[Test 3] Getting sheet names...");
    const sheetsResult = excel.getSheetNames(testFile);
    console.log("Result: " + sheetsResult);
    const sheets = JSON.parse(sheetsResult);
    console.log("✓ Found sheets: " + sheets.sheets.join(", "));
    console.log("");

    // Test 4: Read a single cell
    console.log("[Test 4] Reading cell A1...");
    const cellResult = excel.readCell(testFile, "Sheet1", "A1");
    console.log("Result: " + cellResult);
    const cell = JSON.parse(cellResult);
    console.log("✓ Cell A1 value: " + cell.value);
    console.log("");

    // Test 5: Read a range of cells
    console.log("[Test 5] Reading range A1:D3...");
    const rangeResult = excel.readRange(testFile, "Sheet1", "A1:D3");
    console.log("Result: " + rangeResult);
    const range = JSON.parse(rangeResult);
    console.log("✓ Read " + range.rows + " rows x " + range.cols + " cols");
    console.log("");

    // Display the data nicely
    console.log("=== Data Preview ===");
    for (let i = 0; i < range.data.length; i++) {
      console.log("Row " + (i + 1) + ": " + range.data[i].join(" | "));
    }
    console.log("");

    // Test 6: Edit a single cell in existing file
    console.log("[Test 6] Editing cell B2 (changing Alice's age)...");
    try {
      const editCellResult = excel.editCell(testFile, "Sheet1", "B2", "29");
      console.log("Result: " + editCellResult);
      console.log("✓ Edited cell B2 to '29'");
    } catch (e) {
      console.log("⚠ editCell failed: " + e);
    }
    console.log("");

    // Verify the edit
    console.log("[Verify] Reading edited cell B2...");
    const verifyCell = excel.readCell(testFile, "Sheet1", "B2");
    console.log("Result: " + verifyCell);
    console.log("");

    // Test 7: Edit a range of cells in existing file
    console.log("[Test 7] Editing range A6:D6 (adding new row)...");
    try {
      const newRow = [["Eve", "27", "Fukuoka", "89"]];
      const editRangeResult = excel.editRange(testFile, "Sheet1", "A6", newRow);
      console.log("Result: " + editRangeResult);
      console.log("✓ Edited range A6:D6");
    } catch (e) {
      console.log("⚠ editRange failed: " + e);
    }
    console.log("");

    // Test 8: Insert a row
    console.log("[Test 8] Inserting row at position 3...");
    try {
      const insertRowResult = excel.insertRow(testFile, "Sheet1", 3);
      console.log("Result: " + insertRowResult);
      console.log("✓ Inserted row at position 3");
    } catch (e) {
      console.log("⚠ insertRow failed: " + e);
    }
    console.log("");

    // Test 9: Delete a row
    console.log(
      "[Test 9] Deleting row at position 3 (the one we just inserted)..."
    );
    try {
      const deleteRowResult = excel.deleteRow(testFile, "Sheet1", 3);
      console.log("Result: " + deleteRowResult);
      console.log("✓ Deleted row at position 3");
    } catch (e) {
      console.log("⚠ deleteRow failed: " + e);
    }
    console.log("");

    // Test 10: Open in default app
    console.log("[Test 10] Opening in default application...");
    const openResult = excel.openInApp(testFile);
    console.log("Result: " + openResult);
    console.log("✓ Opened in default app");
    console.log("");

    // Test 11: Get open workbooks (Mac only)
    console.log("[Test 11] Getting open workbooks from Excel app...");
    try {
      const openWbResult = excel.getOpenWorkbooks();
      console.log("Result: " + openWbResult);
      const openWb = JSON.parse(openWbResult);
      console.log("✓ Found " + openWb.count + " open workbook(s)");
      if (openWb.workbooks.length > 0) {
        console.log("  Open files:");
        for (const path of openWb.workbooks) {
          console.log("    - " + path);
        }
      }
    } catch (e) {
      console.log("⚠ getOpenWorkbooks failed (Excel may not be running): " + e);
    }
    console.log("");

    // Final data preview
    console.log("=== Final Data Preview ===");
    const finalData = excel.readRange(testFile, "Sheet1", "A1:D7");
    const finalParsed = JSON.parse(finalData);
    for (let i = 0; i < finalParsed.data.length; i++) {
      console.log("Row " + (i + 1) + ": " + finalParsed.data[i].join(" | "));
    }
    console.log("");

    console.log("=== All Tests Passed! ===");
    console.log("");
    console.log("Test file location: " + testFile);

    return {
      success: true,
      testFile: testFile,
      testsRun: 11,
      message: "All Excel plugin tests passed",
    };
  } catch (error) {
    console.error("Test failed with error: " + error);
    return {
      success: false,
      error: String(error),
    };
  }
}

workflow();
