/**
 * OCR + LLM Plugin Test Workflow
 *
 * This workflow:
 * 1. Scans all PDF files in the directory
 * 2. Extracts text from each PDF using OCR
 * 3. Uses LLM to explain each document content
 *
 * Test files from: /Users/user/Documents/Invoices
 */

const INVOICE_DIR = "/Users/user/Documents/Invoices";

// All PDF files to process
const testFiles = [
  "Apple_Care_Recipt.pdf",
  "invoice.pdf",
  "invoice1.pdf",
  "invoice2.pdf",
  "invoice3.pdf",
  "注文の詳細.pdf",
  "注文の詳細1.pdf",
];

function workflow() {
  console.log("=== OCR + LLM Document Analysis ===");
  console.log("");
  console.log("Processing " + testFiles.length + " PDF files...");
  console.log("");

  const results = {
    totalTests: 0,
    passedTests: 0,
    failedTests: 0,
    summaries: [],
  };

  try {
    var i;
    for (i = 0; i < testFiles.length; i++) {
      var filename = testFiles[i];
      var filePath = INVOICE_DIR + "/" + filename;

      console.log("[" + (i + 1) + "/" + testFiles.length + "] Processing: " + filename);
      console.log("");

      results.totalTests++;

      // Step 1: Extract text
      var text;
      try {
        text = ocr.extract_text(filePath);
        console.log("  Extracted " + text.length + " characters");
      } catch (e) {
        console.log("  OCR FAILED: " + e);
        results.failedTests++;
        console.log("");
        continue;
      }

      // Step 2: Analyze with LLM
      var systemPrompt = "You are a helpful assistant that analyzes invoice and receipt documents. Provide a clear, concise summary in Japanese.";

      var userPrompt = "以下のPDFから抽出されたテキストを分析し、ドキュメントの内容を日本語で説明してください。" +
        String.fromCharCode(10) +
        "含めるべき情報: ドキュメントタイプ、販売元/ベンダー、日付、合計金額、購入した商品。" +
        String.fromCharCode(10) +
        String.fromCharCode(10) +
        "抽出されたテキスト:" +
        String.fromCharCode(10) +
        text;

      try {
        var explanation = llm_chat.chat(systemPrompt, userPrompt);
        console.log("");
        console.log("  --- LLM Explanation ---");
        console.log("  " + explanation);
        console.log("  --- End of Explanation ---");
        console.log("");

        results.summaries.push({
          file: filename,
          explanation: explanation,
        });
        results.passedTests++;
      } catch (e) {
        console.log("  LLM FAILED: " + e);
        results.failedTests++;
      }

      console.log("");
    }

    console.log("=== Analysis Complete ===");
    console.log("Total: " + testFiles.length);
    console.log("Passed: " + results.passedTests);
    console.log("Failed: " + results.failedTests);

    return {
      success: results.failedTests === 0,
      totalTests: results.totalTests,
      passedTests: results.passedTests,
      failedTests: results.failedTests,
      summaries: results.summaries,
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
