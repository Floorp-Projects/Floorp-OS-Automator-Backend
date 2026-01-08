/**
 * Cross-Platform File Researcher
 *
 * OS (Spotlight) と Web (Google Drive) を横断してファイルを検索し、
 * 結果を統合して報告するオートメーション
 */

function workflow() {
  // === 検索クエリ (ハードコードまたはUI入力から取得) ===
  var searchQuery = "未踏";

  console.log(
    "╔════════════════════════════════════════════════════════════════╗"
  );
  console.log(
    "║      Cross-Platform File Researcher                            ║"
  );
  console.log(
    "╚════════════════════════════════════════════════════════════════╝"
  );
  console.log("");
  console.log("Search Query: " + searchQuery);
  console.log("");

  // === Phase 1: ローカル検索 (Spotlight / mdfind) ===
  console.log("━━━ Phase 1: Local Search (Spotlight) ━━━");
  var localResults = [];

  try {
    // macOS の mdfind を使用してファイル検索
    var mdfindCommand = 'mdfind -name "' + searchQuery + '" | head -20';
    console.log("  Executing: " + mdfindCommand);

    var mdfindOutput = exec(mdfindCommand);
    var localPaths = mdfindOutput
      .trim()
      .split("\n")
      .filter(function (p) {
        return p.length > 0;
      });

    for (var i = 0; i < localPaths.length; i++) {
      localResults.push({
        type: "local",
        path: localPaths[i],
        name: localPaths[i].split("/").pop(),
        source: "Spotlight",
      });
    }

    console.log("  ✓ Found " + localResults.length + " local files");
    for (var j = 0; j < Math.min(localResults.length, 5); j++) {
      console.log("    [Local] " + localResults[j].name);
    }
    if (localResults.length > 5) {
      console.log("    ... and " + (localResults.length - 5) + " more");
    }
  } catch (e) {
    console.log("  ✗ Local search error: " + e);
  }

  // === Phase 2: Web検索 (Google Drive) ===
  console.log("");
  console.log("━━━ Phase 2: Web Search (Google Drive) ━━━");
  var gdriveResults = [];
  var gdriveTab = null;

  try {
    var gdriveSearchUrl =
      "https://drive.google.com/drive/search?q=" +
      encodeURIComponent(searchQuery);
    console.log("  Opening: " + gdriveSearchUrl);

    // 既存タブを検索 or 新規タブを開く
    var existingTabs = floorp.listBrowserTabs();
    var existingTabsData = JSON.parse(existingTabs);
    var foundTab = null;

    for (var t = 0; t < existingTabsData.length; t++) {
      if (
        existingTabsData[t].url &&
        existingTabsData[t].url.includes("drive.google.com")
      ) {
        foundTab = existingTabsData[t];
        break;
      }
    }

    if (foundTab) {
      console.log("  Found existing Google Drive tab, navigating...");
      gdriveTab = String(foundTab.id);
      floorp.attachToTab(gdriveTab);
      floorp.navigateTab(gdriveTab, gdriveSearchUrl);
    } else {
      console.log("  Creating new tab for Google Drive...");
      gdriveTab = floorp.createTab(gdriveSearchUrl, false);
    }

    // ページ読み込み待機
    console.log("  Waiting for search results to load...");
    sleep(5000);

    // 検索結果を抽出
    console.log("  Extracting search results...");

    // tr[role="row"] でファイル行を取得
    var elementsJson = floorp.tabGetElements(gdriveTab, 'tr[role="row"]');
    var elementsData = JSON.parse(elementsJson);
    var rowElements = elementsData.elements || [];

    console.log("  Found " + rowElements.length + " rows in search results");

    // 各行から data-id とファイル名を抽出
    // 安定したセレクタを優先順位付きで使用:
    // 1. aria-label: アクセシビリティ要件のため最も安定
    // 2. data-tooltip: ホバー時のツールチップ用
    // 3. jsname: Google内部属性（変わる可能性あり）
    var debugStats = { aria: 0, tooltip: 0, jsname: 0, noMatch: 0 };

    for (var r = 0; r < rowElements.length; r++) {
      var rowHtml = rowElements[r];

      // data-id を抽出（不変属性）
      var idMatch = rowHtml.match(/data-id="([^"]+)"/);
      if (!idMatch) continue;
      var fileId = idMatch[1];

      var fileName = "";
      var matchedBy = "";

      // 優先1: aria-label からファイル名を取得（最も安定）
      // 形式: aria-label="ファイル名, その他の情報"
      var ariaMatch = rowHtml.match(/aria-label="([^",]+)/);
      if (ariaMatch) {
        fileName = ariaMatch[1].trim();
        matchedBy = "aria-label";
        debugStats.aria++;
      }

      // 優先2: data-tooltip から取得
      if (!fileName) {
        var tooltipMatch = rowHtml.match(/data-tooltip="([^"]+)"/);
        if (tooltipMatch) {
          fileName = tooltipMatch[1].trim();
          matchedBy = "data-tooltip";
          debugStats.tooltip++;
        }
      }

      // 優先3: jsname="vtaz5c" 属性を持つ要素から取得
      if (!fileName) {
        var jsnameMatch = rowHtml.match(/jsname="vtaz5c"[^>]*>([^<]+)</);
        if (jsnameMatch) {
          fileName = jsnameMatch[1].trim();
          matchedBy = "jsname";
          debugStats.jsname++;
        }
      }

      if (!fileName) {
        debugStats.noMatch++;
      }

      if (fileName && fileId) {
        gdriveResults.push({
          type: "gdrive",
          id: fileId,
          name: fileName,
          url: "https://drive.google.com/file/d/" + fileId + "/view",
          source: "Google Drive",
          _matchedBy: matchedBy, // デバッグ用
        });
      }
    }

    // デバッグ統計を出力
    console.log(
      "  [Debug] Selector stats: aria=" +
        debugStats.aria +
        ", tooltip=" +
        debugStats.tooltip +
        ", jsname=" +
        debugStats.jsname +
        ", noMatch=" +
        debugStats.noMatch
    );

    // マッチしなかった場合、最初の行のHTMLサンプルを出力（トラブルシュート用）
    if (gdriveResults.length === 0 && rowElements.length > 0) {
      console.log("  [Debug] No files extracted. First row HTML sample:");
      console.log("  " + rowElements[0].substring(0, 500) + "...");
    }

    console.log("  ✓ Found " + gdriveResults.length + " files on Google Drive");
    for (var g = 0; g < Math.min(gdriveResults.length, 5); g++) {
      console.log("    [GDrive] " + gdriveResults[g].name);
    }
    if (gdriveResults.length > 5) {
      console.log("    ... and " + (gdriveResults.length - 5) + " more");
    }
  } catch (e) {
    console.log("  ✗ Google Drive search error: " + e);
  } finally {
    // タブは閉じない（ユーザーが確認できるように）
  }

  // === Phase 3: 結果の統合と報告 ===
  console.log("");
  console.log("━━━ Phase 3: Results Summary ━━━");

  var totalResults = localResults.length + gdriveResults.length;
  console.log("  Total files found: " + totalResults);
  console.log("    - Local (Spotlight): " + localResults.length);
  console.log("    - Web (Google Drive): " + gdriveResults.length);

  // === Phase 4: AI分析 (オプション) ===
  console.log("");
  console.log("━━━ Phase 4: AI Analysis ━━━");

  if (totalResults > 0) {
    try {
      var analysisPrompt =
        "以下の検索結果を分析し、ファイルの分布について簡潔に報告してください:\n\n";
      analysisPrompt += '検索クエリ: "' + searchQuery + '"\n\n';

      analysisPrompt += "【ローカル (" + localResults.length + "件)】\n";
      for (var l = 0; l < Math.min(localResults.length, 10); l++) {
        analysisPrompt += "- " + localResults[l].path + "\n";
      }

      analysisPrompt += "\n【Google Drive (" + gdriveResults.length + "件)】\n";
      for (var d = 0; d < Math.min(gdriveResults.length, 10); d++) {
        analysisPrompt +=
          "- " + gdriveResults[d].name + " (" + gdriveResults[d].url + ")\n";
      }

      analysisPrompt +=
        "\n上記の結果から、ファイルの場所（ローカル/クラウド）について何が言えますか？最新版はどこにありそうですか？";

      console.log("  Generating AI analysis...");
      var aiAnalysis = iniad_ai_mop.chat(
        "あなたはファイル管理アドバイザーです。ローカルとクラウドのファイル分布について簡潔に分析してください。",
        analysisPrompt
      );

      console.log("");
      console.log(
        "┌─────────────────────────────────────────────────────────────────┐"
      );
      console.log(
        "│ AI Analysis                                                     │"
      );
      console.log(
        "├─────────────────────────────────────────────────────────────────┤"
      );
      // AI分析結果を行ごとに表示
      var analysisLines = aiAnalysis.split("\n");
      for (var a = 0; a < analysisLines.length; a++) {
        console.log("│ " + analysisLines[a]);
      }
      console.log(
        "└─────────────────────────────────────────────────────────────────┘"
      );
    } catch (e) {
      console.log("  ⚠ AI analysis skipped: " + e);
    }
  } else {
    console.log("  No results to analyze.");
  }

  console.log("");
  console.log(
    "╔════════════════════════════════════════════════════════════════╗"
  );
  console.log(
    "║      Cross-Platform Search Complete!                           ║"
  );
  console.log(
    "╚════════════════════════════════════════════════════════════════╝"
  );

  return {
    success: true,
    query: searchQuery,
    localCount: localResults.length,
    gdriveCount: gdriveResults.length,
    localResults: localResults.slice(0, 10),
    gdriveResults: gdriveResults.slice(0, 10),
  };
}

// === Helper Functions ===

function sleep(ms) {
  try {
    var sab = new SharedArrayBuffer(4);
    var int32 = new Int32Array(sab);
    Atomics.wait(int32, 0, 0, ms);
  } catch (e) {
    var start = Date.now();
    while (Date.now() - start < ms) {}
  }
}
