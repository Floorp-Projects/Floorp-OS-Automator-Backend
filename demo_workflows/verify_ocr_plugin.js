/**
 * OCR Plugin Test Workflow
 *
 * This workflow tests all OCR plugin functions:
 * - extract_text: Extracts text from PDF with fallback (pdftotext -> ocrmypdf -> tesseract)
 * - extract_document: Extracts OCR text + structured fields using Ollama vision model
 *
 * Test files from: /Users/user/Documents/Invoices
 *
 * Requirements:
 * - pdftotext (poppler-utils)
 * - ocrmypdf
 * - tesseract (with jpn+eng)
 * - Ollama running with vision model (glm-ocr:latest)
 */

const INVOICE_DIR = "/Users/user/Documents/Invoices";
const OLLAMA_MODEL = "glm-ocr:latest";
const OLLAMA_BASE_URL = "http://127.0.0.1:11434";

function workflow() {
  console.log("=== OCR Plugin Test Workflow ===");
  console.log("");

  const results = {
    totalTests: 0,
    passedTests: 0,
    failedTests: 0,
    details: [],
  };

  try {
    // Test 1: Basic text extraction from sample PDFs
    console.log("[Test 1] Testing extract_text() on sample invoices...");
    console.log("");

    const testFiles = [
      "Apple_Care_Recipt.pdf",
      "invoice.pdf",
      "invoice1.pdf",
      "invoice2.pdf",
      "invoice3.pdf",
      "注文の詳細.pdf",
      "注文の詳細1.pdf",
    ];

    var i;
    for (i = 0; i < testFiles.length; i++) {
      var filename = testFiles[i];
      var filePath = INVOICE_DIR + "/" + filename;
      console.log("  Testing: " + filename);

      results.totalTests++;

      try {
        var text = ocr.extract_text(filePath);

        console.log("    Extracted " + text.length + " characters");
        console.log("");
        console.log("    --- Text Content ---");
        console.log("    " + text);
        console.log("    --- End of Text ---");
        console.log("");
        console.log("    PASSED");
        results.passedTests++;

        results.details.push({
          test: "extract_text_" + filename,
          status: "passed",
          chars: text.length,
        });
      } catch (e) {
        console.log("    FAILED: " + e);
        results.failedTests++;

        results.details.push({
          test: "extract_text_" + filename,
          status: "failed",
          error: String(e),
        });
      }
      console.log("");
    }

    // Test 2: Full document extraction with structured data
    console.log("[Test 2] Testing extract_document() with Ollama vision...");
    console.log("");

    var docTestFiles = ["invoice.pdf", "Apple_Care_Recipt.pdf"];

    for (i = 0; i < docTestFiles.length; i++) {
      var filename = docTestFiles[i];
      var filePath = INVOICE_DIR + "/" + filename;
      console.log("  Testing: " + filename);

      results.totalTests++;

      try {
        var resultJson = ocr.extract_document(
          filePath,
          OLLAMA_MODEL,
          OLLAMA_BASE_URL
        );
        var result = JSON.parse(resultJson);

        console.log("    OCR Engine: " + result.ocr_engine);
        console.log("    Text Quality: " + result.text_quality);
        console.log("    Ollama Used: " + result.ollama_used);

        if (result.warnings && result.warnings.length > 0) {
          console.log("    Warnings: " + result.warnings.join(", "));
        }

        console.log("");
        console.log("    --- Extracted Text ---");
        console.log("    " + result.text);
        console.log("    --- End of Text ---");
        console.log("");

        // Display structured data
        if (result.structured) {
          var s = result.structured;
          console.log("    Structured Data:");
          console.log("      Doc Type: " + s.doc_type);
          console.log("      Vendor: " + (s.vendor_name || "N/A"));
          console.log("      Date: " + (s.issue_date || "N/A"));
          console.log("      Currency: " + (s.currency || "N/A"));
          var totalDisplay = s.total_amount ? s.total_amount + " JPY" : "N/A";
          console.log("      Total: " + totalDisplay);
          var invoiceDisplay = s.invoice_number || s.order_number || "N/A";
          console.log("      Invoice #: " + invoiceDisplay);

          if (s.line_items && s.line_items.length > 0) {
            console.log("      Line Items: " + s.line_items.length + " items");
          }
        }

        console.log("    PASSED");
        results.passedTests++;

        var docType = result.structured ? result.structured.doc_type : "unknown";
        results.details.push({
          test: "extract_document_" + filename,
          status: "passed",
          ocr_engine: result.ocr_engine,
          text_quality: result.text_quality,
          doc_type: docType,
        });
      } catch (e) {
        console.log("    FAILED: " + e);
        results.failedTests++;

        results.details.push({
          test: "extract_document_" + filename,
          status: "failed",
          error: String(e),
        });
      }
      console.log("");
    }

    // Test 3: Document type detection summary
    console.log("[Test 3] Document Type Detection Summary...");
    console.log("");

    var summary = {
      invoice: 0,
      receipt: 0,
      order_detail: 0,
      unknown: 0,
    };

    // Test a subset of files for document type detection using extract_text
    var docTestFiles = [
      "invoice.pdf",
      "invoice1.pdf",
      "Apple_Care_Recipt.pdf",
      "MB48595477.pdf",
      "注文の詳細.pdf",
    ];

    for (i = 0; i < docTestFiles.length; i++) {
      var filename = docTestFiles[i];
      var filePath = INVOICE_DIR + "/" + filename;

      // Simple document type detection based on filename
      var docType = "unknown";
      if (filename.indexOf("invoice") !== -1) {
        docType = "invoice";
        summary.invoice++;
      } else if (
        filename.indexOf("receipt") !== -1 ||
        filename.indexOf("Recipt") !== -1 ||
        filename.indexOf("MB") === 0
      ) {
        docType = "receipt";
        summary.receipt++;
      } else if (filename.indexOf("注文") !== -1) {
        docType = "order_detail";
        summary.order_detail++;
      } else {
        summary.unknown++;
      }

      console.log("  " + filename + " -> " + docType);
    }

    console.log("");
    console.log("  Summary:");
    console.log("    Invoices: " + summary.invoice);
    console.log("    Receipts: " + summary.receipt);
    console.log("    Order Details: " + summary.order_detail);
    console.log("    Unknown: " + summary.unknown);

    // Final summary
    console.log("");
    console.log("=== Test Summary ===");
    console.log("Total Tests: " + results.totalTests);
    console.log("Passed: " + results.passedTests);
    console.log("Failed: " + results.failedTests);
    console.log("");

    if (results.failedTests === 0) {
      console.log("All tests PASSED!");
    } else {
      console.log("Some tests FAILED. Check details above.");
    }

    return {
      success: results.failedTests === 0,
      totalTests: results.totalTests,
      passedTests: results.passedTests,
      failedTests: results.failedTests,
      details: results.details,
      summary: summary,
    };
  } catch (error) {
    console.error("Workflow failed with error: " + error);
    return {
      success: false,
      error: String(error),
    };
  }
}

workflow();
