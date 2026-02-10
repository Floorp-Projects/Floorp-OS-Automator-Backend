/**
 * 確定申告 経費自動分類ワークフロー
 *
 * 領収書・請求書の PDF を OCR で読み取り、確定申告の勘定科目に自動分類して
 * JSON ファイルに出力するワークフローです。
 *
 * 処理フロー:
 *   1. 指定ディレクトリの PDF を一覧取得
 *   2. 各 PDF を ocr.extract_document() で構造化データ抽出（全ページ対応）
 *   3. llm_chat.chat() で勘定科目を推定
 *   4. JSON にまとめて出力
 *
 * 使用プラグイン:
 *   - filesystem (listFiles, write)
 *   - ocr (extract_document)
 *   - llm_chat (chat)
 *
 * 要件:
 *   - Ollama が glm-ocr:latest モデルで起動していること
 *   - Ollama の汎用チャットモデル (gemma3, llama3 等) が利用可能なこと
 *   - poppler-utils (pdftoppm/pdftotext) がインストール済み
 */

// ============================================================================
// 設定
// ============================================================================

/** PDF が格納されているディレクトリ */
var INVOICE_DIR = "/Users/user/Documents/Invoices";

/** 出力 JSON ファイルのパス */
var OUTPUT_JSON = "/Users/user/Documents/確定申告_経費一覧.json";

/** OCR 用 Ollama モデル */
var OCR_MODEL = "glm-ocr:latest";

/** LLM チャット用 Ollama ベースURL */
var OLLAMA_BASE_URL = "http://127.0.0.1:11434";

/** 対象年度 (この年度のものだけ出力する。0 なら全件) */
var TARGET_YEAR = 2025;

// ============================================================================
// 確定申告 勘定科目マスタ
// ============================================================================

/**
 * 個人事業主・フリーランス向けの主要な勘定科目
 * 確定申告書B の経費欄に対応
 */
var EXPENSE_CATEGORIES = {
  rent: { code: "地代家賃", description: "事務所・店舗の家賃" },
  utilities: { code: "水道光熱費", description: "電気・ガス・水道" },
  communication: {
    code: "通信費",
    description: "電話・インターネット・サーバー費用",
  },
  travel: { code: "旅費交通費", description: "電車・タクシー・飛行機・宿泊" },
  entertainment: { code: "接待交際費", description: "取引先との飲食・贈答品" },
  supplies: {
    code: "消耗品費",
    description: "文房具・日用品・1万円未満の備品",
  },
  equipment: {
    code: "工具器具備品",
    description: "10万円以上のPC・機器等（減価償却対象）",
  },
  software: {
    code: "消耗品費(ソフト)",
    description: "サブスクリプション・ソフトウェア・クラウドサービス",
  },
  outsourcing: {
    code: "外注工賃",
    description: "業務委託・フリーランスへの支払い",
  },
  advertising: { code: "広告宣伝費", description: "Web広告・印刷物・PR" },
  insurance: { code: "損害保険料", description: "事業用の保険" },
  books: { code: "新聞図書費", description: "書籍・技術書・新聞・雑誌" },
  membership: { code: "諸会費", description: "団体会費・業界団体" },
  repair: { code: "修繕費", description: "設備・建物の修理" },
  tax_payment: { code: "租税公課", description: "事業税・印紙税・固定資産税" },
  shipping: { code: "荷造運賃", description: "配送・梱包費用" },
  training: { code: "研修費", description: "セミナー・研修・資格取得" },
  miscellaneous: { code: "雑費", description: "上記に該当しないもの" },
  non_deductible: { code: "対象外", description: "経費計上不可（私的利用等）" },
};

// ============================================================================
// メイン処理
// ============================================================================

function workflow() {
  console.log("=== 確定申告 経費自動分類ワークフロー ===");
  console.log("対象ディレクトリ: " + INVOICE_DIR);
  console.log("出力先: " + OUTPUT_JSON);
  console.log("");

  // -----------------------------------------------------------------------
  // Step 1: PDF ファイル一覧取得
  // -----------------------------------------------------------------------
  console.log("[Step 1] PDF ファイル一覧を取得中...");

  var fileListRaw = app.sapphillon.core.filesystem.listFiles(INVOICE_DIR);
  var fileList = JSON.parse(fileListRaw);
  var pdfFiles = [];

  for (var i = 0; i < fileList.length; i++) {
    var entry = fileList[i];
    // listFiles は絶対パスまたはオブジェクトを返す場合がある
    var fullPath = "";
    if (typeof entry === "object" && entry.name) {
      fullPath = entry.name;
    } else if (typeof entry === "string") {
      fullPath = entry;
    } else {
      continue;
    }
    if (fullPath.toLowerCase().indexOf(".pdf") === -1) {
      continue;
    }
    // 絶対パスでなければディレクトリを付加
    if (fullPath.indexOf("/") !== 0) {
      fullPath = INVOICE_DIR + "/" + fullPath;
    }
    pdfFiles.push(fullPath);
  }

  console.log("  PDF ファイル数: " + pdfFiles.length);
  if (pdfFiles.length === 0) {
    console.log("  PDF ファイルが見つかりません。終了します。");
    return { success: false, error: "No PDF files found" };
  }
  console.log("");

  // -----------------------------------------------------------------------
  // Step 2: 各 PDF を OCR で構造化データ抽出
  // -----------------------------------------------------------------------
  console.log("[Step 2] OCR による構造化データ抽出...");
  console.log("");

  var allExpenses = [];
  var errors = [];

  for (var i = 0; i < pdfFiles.length; i++) {
    var filePath = pdfFiles[i];
    // ファイル名だけを抽出（表示・記録用）
    var filename = filePath;
    var lastSlash = filePath.lastIndexOf("/");
    if (lastSlash >= 0) {
      filename = filePath.substring(lastSlash + 1);
    }

    console.log("  [" + (i + 1) + "/" + pdfFiles.length + "] " + filename);

    try {
      var resultJson = ocr.extract_document(
        filePath,
        OCR_MODEL,
        OLLAMA_BASE_URL,
      );
      var result = JSON.parse(resultJson);

      var expense = {
        filename: filename,
        doc_type: "unknown",
        vendor_name: "",
        issue_date: "",
        total_amount: 0,
        tax_amount: 0,
        subtotal_amount: 0,
        currency: "JPY",
        invoice_number: "",
        order_number: "",
        payment_method: "",
        line_items: [],
        ocr_text: "",
        ocr_quality: 0,
      };

      if (result.structured) {
        var s = result.structured;
        expense.doc_type = s.doc_type || "unknown";
        expense.vendor_name = s.vendor_name || "";
        expense.issue_date = s.issue_date || "";
        expense.total_amount = s.total_amount || 0;
        expense.tax_amount = s.tax_amount || 0;
        expense.subtotal_amount = s.subtotal_amount || 0;
        expense.currency = s.currency || "JPY";
        expense.invoice_number = s.invoice_number || "";
        expense.order_number = s.order_number || "";
        expense.payment_method = s.payment_method || "";
        expense.line_items = s.line_items || [];
      }

      expense.ocr_text = result.text || "";
      expense.ocr_quality = result.text_quality || 0;

      // 年度フィルタ
      if (TARGET_YEAR > 0 && expense.issue_date) {
        var year = parseInt(expense.issue_date.substring(0, 4), 10);
        if (year !== TARGET_YEAR) {
          console.log("    → 対象年度外 (" + year + ") スキップ");
          continue;
        }
      }

      allExpenses.push(expense);
      console.log(
        "    → " +
          expense.doc_type +
          " | " +
          expense.vendor_name +
          " | " +
          expense.issue_date +
          " | " +
          formatAmount(expense.total_amount),
      );
    } catch (e) {
      console.log("    → ERROR: " + e);
      errors.push({ filename: filename, error: String(e) });
    }
  }

  console.log("");
  console.log(
    "  抽出完了: " +
      allExpenses.length +
      " 件, エラー: " +
      errors.length +
      " 件",
  );
  console.log("");

  // -----------------------------------------------------------------------
  // Step 3: LLM で勘定科目を推定
  // -----------------------------------------------------------------------
  console.log("[Step 3] LLM による勘定科目分類...");
  console.log("");

  for (var i = 0; i < allExpenses.length; i++) {
    var expense = allExpenses[i];
    console.log(
      "  [" + (i + 1) + "/" + allExpenses.length + "] " + expense.vendor_name,
    );

    try {
      var category = classifyExpense(expense);
      expense.category_key = category.key;
      expense.category_name = category.name;
      expense.category_reason = category.reason;
      expense.business_ratio = category.business_ratio;

      console.log(
        "    → " +
          category.name +
          " (事業割合: " +
          category.business_ratio +
          "%)",
      );
      if (category.reason) {
        console.log("    理由: " + category.reason);
      }
    } catch (e) {
      console.log("    → 分類失敗: " + e);
      expense.category_key = "miscellaneous";
      expense.category_name = "雑費";
      expense.category_reason = "自動分類失敗";
      expense.business_ratio = 100;
    }
  }

  console.log("");

  // -----------------------------------------------------------------------
  // Step 4: JSON 出力
  // -----------------------------------------------------------------------
  console.log("[Step 4] JSON ファイル出力...");

  try {
    writeJson(allExpenses, errors);
    console.log("  → " + OUTPUT_JSON + " に保存しました");
  } catch (e) {
    console.log("  JSON 出力失敗: " + e);
  }

  // -----------------------------------------------------------------------
  // Step 5: サマリー出力
  // -----------------------------------------------------------------------
  console.log("");
  console.log("[Step 5] 集計サマリー");
  console.log("");

  var summary = buildSummary(allExpenses);

  console.log("  ── 勘定科目別 合計 ──");
  for (var k = 0; k < summary.byCategory.length; k++) {
    var cat = summary.byCategory[k];
    console.log(
      "  " +
        cat.name +
        ": " +
        formatAmount(cat.total) +
        " (" +
        cat.count +
        "件)",
    );
  }
  console.log("");
  console.log("  経費合計 (税込): " + formatAmount(summary.grandTotal));
  console.log(
    "  経費合計 (事業按分後): " + formatAmount(summary.businessTotal),
  );
  console.log("  消費税合計: " + formatAmount(summary.taxTotal));
  console.log("  処理件数: " + summary.totalCount);
  console.log("  エラー件数: " + errors.length);
  console.log("");

  console.log("");
  console.log("=== 完了 ===");

  return {
    success: true,
    totalFiles: pdfFiles.length,
    processedCount: allExpenses.length,
    errorCount: errors.length,
    grandTotal: summary.grandTotal,
    businessTotal: summary.businessTotal,
    outputFile: OUTPUT_JSON,
  };
}

// ============================================================================
// LLM 勘定科目分類
// ============================================================================

/**
 * 1件の経費データを LLM に送り、勘定科目を推定する
 */
function classifyExpense(expense) {
  var categoryList = "";
  var keys = Object.keys(EXPENSE_CATEGORIES);
  for (var i = 0; i < keys.length; i++) {
    var k = keys[i];
    categoryList +=
      k +
      ": " +
      EXPENSE_CATEGORIES[k].code +
      " (" +
      EXPENSE_CATEGORIES[k].description +
      ")\n";
  }

  var systemPrompt =
    "あなたは日本の確定申告に精通した税理士AIです。\n" +
    "与えられた経費情報から、最も適切な勘定科目を1つ選び、事業利用割合（%）を推定してください。\n" +
    "個人事業主・フリーランスの青色申告を前提とします。\n\n" +
    "【勘定科目一覧】\n" +
    categoryList +
    "\n" +
    "【回答形式】以下のJSON形式のみで回答してください。\n" +
    '{"key":"カテゴリキー","name":"勘定科目名","reason":"分類理由（30文字以内）","business_ratio":100}\n' +
    "注意:\n" +
    "- business_ratio は事業利用割合（0-100）。私的利用が混在しそうなら適切な割合を設定\n" +
    "- Apple Care や私物のサブスクリプションは business_ratio を低めに\n" +
    "- サーバー費用・ドメイン・クラウドサービスは通信費か消耗品費(ソフト)\n" +
    "- PC・モニター等で10万円以上は工具器具備品\n" +
    "- 書籍・技術書は新聞図書費\n" +
    "- 正確な判断ができない場合は雑費(miscellaneous)を選択";

  var lineItemsText = "";
  if (expense.line_items && expense.line_items.length > 0) {
    lineItemsText = "\n明細:\n";
    for (var j = 0; j < expense.line_items.length; j++) {
      var item = expense.line_items[j];
      var itemStr = "";
      if (typeof item === "string") {
        itemStr = item;
      } else if (item.description || item.name) {
        itemStr =
          (item.description || item.name) +
          (item.amount ? " " + item.amount + "円" : "");
      } else {
        itemStr = JSON.stringify(item);
      }
      lineItemsText += "- " + itemStr + "\n";
    }
  }

  var ocrSnippet = "";
  if (expense.ocr_text) {
    ocrSnippet = expense.ocr_text.substring(0, 1500);
  }

  var userPrompt =
    "以下の経費を分類してください。\n\n" +
    "ファイル名: " +
    expense.filename +
    "\n" +
    "書類種別: " +
    expense.doc_type +
    "\n" +
    "取引先: " +
    (expense.vendor_name || "不明") +
    "\n" +
    "日付: " +
    (expense.issue_date || "不明") +
    "\n" +
    "合計金額: " +
    expense.total_amount +
    "円\n" +
    "税額: " +
    (expense.tax_amount || "不明") +
    "円\n" +
    "請求書番号: " +
    (expense.invoice_number || expense.order_number || "不明") +
    "\n" +
    "支払方法: " +
    (expense.payment_method || "不明") +
    "\n" +
    lineItemsText +
    "\n" +
    "OCRテキスト（先頭1500文字）:\n" +
    ocrSnippet;

  var response = llm_chat.chat(systemPrompt, userPrompt);

  // JSON 解析
  var parsed = parseJsonFromLlm(response);
  if (!parsed || !parsed.key) {
    return {
      key: "miscellaneous",
      name: "雑費",
      reason: "LLM応答のパース失敗",
      business_ratio: 100,
    };
  }

  // 勘定科目キーの検証
  if (!EXPENSE_CATEGORIES[parsed.key]) {
    parsed.key = "miscellaneous";
    parsed.name = "雑費";
  } else {
    parsed.name = EXPENSE_CATEGORIES[parsed.key].code;
  }

  return {
    key: parsed.key,
    name: parsed.name,
    reason: parsed.reason || "",
    business_ratio:
      typeof parsed.business_ratio === "number"
        ? Math.max(0, Math.min(100, parsed.business_ratio))
        : 100,
  };
}

/**
 * LLM の応答文字列から JSON オブジェクトを抽出
 */
function parseJsonFromLlm(text) {
  if (!text) return null;
  var trimmed = text.trim();

  // ```json ... ``` ブロック除去
  trimmed = trimmed
    .replace(/^```json\s*/i, "")
    .replace(/^```\s*/i, "")
    .replace(/\s*```$/i, "")
    .trim();

  try {
    return JSON.parse(trimmed);
  } catch (_) {
    // { ... } を探す
    var start = trimmed.indexOf("{");
    var end = trimmed.lastIndexOf("}");
    if (start >= 0 && end > start) {
      try {
        return JSON.parse(trimmed.substring(start, end + 1));
      } catch (_) {
        return null;
      }
    }
    return null;
  }
}

// ============================================================================
// JSON 出力
// ============================================================================

function writeJson(expenses, errors) {
  var summary = buildSummary(expenses);

  var items = [];
  for (var i = 0; i < expenses.length; i++) {
    var e = expenses[i];
    var deductibleAmount = Math.round(
      (e.total_amount * (e.business_ratio || 100)) / 100,
    );
    items.push({
      no: i + 1,
      filename: e.filename,
      issue_date: e.issue_date || null,
      vendor_name: e.vendor_name || null,
      doc_type: e.doc_type || "unknown",
      category_key: e.category_key || "miscellaneous",
      category_name: e.category_name || "雑費",
      category_reason: e.category_reason || null,
      total_amount: e.total_amount || 0,
      tax_amount: e.tax_amount || 0,
      subtotal_amount: e.subtotal_amount || 0,
      currency: e.currency || "JPY",
      business_ratio: e.business_ratio || 100,
      deductible_amount: deductibleAmount,
      invoice_number: e.invoice_number || null,
      order_number: e.order_number || null,
      payment_method: e.payment_method || null,
      line_items: e.line_items || [],
    });
  }

  var output = {
    generated_at: new Date().toISOString(),
    target_year: TARGET_YEAR || null,
    source_directory: INVOICE_DIR,
    summary: {
      total_count: summary.totalCount,
      grand_total: summary.grandTotal,
      business_total: summary.businessTotal,
      tax_total: summary.taxTotal,
      by_category: summary.byCategory,
    },
    expenses: items,
    errors: errors,
  };

  var jsonStr = JSON.stringify(output, null, 2);
  app.sapphillon.core.filesystem.write(OUTPUT_JSON, jsonStr);
}

// ============================================================================
// ユーティリティ
// ============================================================================

function formatAmount(amount) {
  if (!amount && amount !== 0) return "¥0";
  var str = String(Math.abs(amount));
  var result = "";
  for (var i = str.length - 1, count = 0; i >= 0; i--, count++) {
    if (count > 0 && count % 3 === 0) result = "," + result;
    result = str[i] + result;
  }
  return (amount < 0 ? "-¥" : "¥") + result;
}

function docTypeLabel(docType) {
  var labels = {
    invoice: "請求書",
    receipt: "領収書",
    order_detail: "注文明細",
    unknown: "不明",
  };
  return labels[docType] || docType || "不明";
}

function buildSummary(expenses) {
  var catTotals = {};
  var grandTotal = 0;
  var businessTotal = 0;
  var taxTotal = 0;

  for (var i = 0; i < expenses.length; i++) {
    var e = expenses[i];
    var catName = e.category_name || "雑費";
    var amount = e.total_amount || 0;
    var ratio = e.business_ratio || 100;
    var deductible = Math.round((amount * ratio) / 100);

    grandTotal += amount;
    businessTotal += deductible;
    taxTotal += e.tax_amount || 0;

    if (!catTotals[catName]) {
      catTotals[catName] = {
        name: catName,
        count: 0,
        total: 0,
        businessTotal: 0,
      };
    }
    catTotals[catName].count++;
    catTotals[catName].total += amount;
    catTotals[catName].businessTotal += deductible;
  }

  // ソート（金額降順）
  var categories = [];
  var keys = Object.keys(catTotals);
  for (var i = 0; i < keys.length; i++) {
    categories.push(catTotals[keys[i]]);
  }
  categories.sort(function (a, b) {
    return b.total - a.total;
  });

  return {
    byCategory: categories,
    grandTotal: grandTotal,
    businessTotal: businessTotal,
    taxTotal: taxTotal,
    totalCount: expenses.length,
  };
}

// ============================================================================
// 実行
// ============================================================================

workflow();
