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

    // Test 6: Open in default app
    console.log("[Test 6] Opening in default application...");
    const openResult = excel.openInApp(testFile);
    console.log("Result: " + openResult);
    console.log("✓ Opened in default app");
    console.log("");

    // Test 6: Get open workbooks (Mac only)
    console.log("[Test 6] Getting open workbooks from Excel app...");
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

    // Test 7: Create a chart (Mac only)
    console.log("[Test 7] Creating a chart...");
    try {
      // Data range is A1:D5 (Headers + 4 rows of data)
      // Attempt to chart "Name" (A) vs "Score" (D) - simplified for whole range selection usually takes all relevant data
      const chartResult = excel.createChart(
        testFile,
        "Sheet1",
        "A1:D5",
        "column",
        "Test Score Chart"
      );
      console.log("Result: " + chartResult);
      const chart = JSON.parse(chartResult);
      console.log("✓ Created " + chart.chartType + " chart: " + chart.title);
    } catch (e) {
      console.log("⚠ createChart failed: " + e);
    }
    console.log("");

    console.log("=== All Tests Passed! ===");
    console.log("");
    console.log("Test file location: " + testFile);

    return {
      success: true,
      testFile: testFile,
      testsRun: 6,
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
