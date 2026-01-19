/**
 * Deep Research - Comprehensive Web Analysis
 *
 * 1. Search on DuckDuckGo (50 results)
 * 2. AI relevance filtering to select most relevant results
 * 3. Visit each relevant URL to extract detailed content
 * 4. Analyze and synthesize with LLM
 * 5. Generate comprehensive academic-style report with fact-checking
 */

function workflow() {
  var searchQuery = "バンブーラボ";
  var maxResults = 50;
  var outputPath = "/Users/user/Desktop/Deep_Research_Report.md";

  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║           Deep Research - Comprehensive Analysis           ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
  console.log("");
  console.log("Search Query: " + searchQuery);
  console.log("Target Results: " + maxResults);
  console.log("");

  // --- Phase 1: Local Finder Search ---
  console.log("━━━ Phase 1: Local Finder Search (Finder/mdfind) ━━━");
  var searchResults = [];

  try {
    if (
      app &&
      app.sapphillon &&
      app.sapphillon.core &&
      app.sapphillon.core.finder
    ) {
      console.log("  Using Finder plugin: findFilesWithMdfind");
      var pathsJson = app.sapphillon.core.finder.findFilesWithMdfind(
        "/Users/user/Desktop",
        searchQuery,
        maxResults,
      );
      var paths = [];
      try {
        paths = JSON.parse(pathsJson || "[]");
      } catch (e) {
        paths = [];
      }

      console.log("  Finder returned " + paths.length + " paths");

      for (
        var i = 0;
        i < paths.length && searchResults.length < maxResults;
        i++
      ) {
        try {
          var p = paths[i];
          var content = "";
          try {
            content = app.sapphillon.core.finder.extractText(p) || "";
          } catch (e) {
            content = "";
          }
          var title = p.split("/").pop() || p;

          searchResults.push({
            rank: searchResults.length + 1,
            title: title,
            url: "file://" + p,
            snippet: (content || "").substring(0, 300),
            domain: "local",
            pageContent: content,
            pageTitle: title,
            extractedAt: new Date().toISOString(),
          });

          console.log(
            "  [" + searchResults.length + "] " + title + " (" + p + ")",
          );
        } catch (e) {
          console.log("  Finder item error: " + e);
        }
      }
    } else {
      console.log(
        "  Finder plugin not available; no local collection performed.",
      );
    }
  } catch (e) {
    console.log("  Finder Error: " + e);
  } finally {
    // noop
  }

  // --- Phase 1.5: AI Relevance Filtering ---
  console.log("");
  console.log("━━━ Phase 1.5: Relevance Filtering ━━━");
  console.log('  Filtering results for relevance to: "' + searchQuery + '"');

  searchResults = filterRelevantResults(searchResults, searchQuery);

  console.log("  ✓ " + searchResults.length + " relevant results selected");

  // --- Phase 2: Visit Each Page ---
  console.log("");
  console.log("━━━ Phase 2: Deep Content Extraction ━━━");
  console.log(
    "  Visiting " + searchResults.length + " pages for detailed analysis...",
  );
  console.log("");

  var successCount = 0;
  var failCount = 0;

  for (var i = 0; i < searchResults.length; i++) {
    var result = searchResults[i];
    var progress = "[" + (i + 1) + "/" + searchResults.length + "]";
    console.log(progress + " Visiting: " + result.domain);

    var pageTab = null;
    try {
      pageTab = floorp.createTab(result.url, false);

      // Wait for page to load - increased timeout for slower sites
      try {
        floorp.tabWaitForNetworkIdle(pageTab, 10000);
      } catch (e) {}
      // Additional wait for JavaScript-heavy sites
      sleep(5000);

      // Extract page title
      try {
        var titleJson = floorp.tabElementText(pageTab, "title");
        result.pageTitle = cleanText(JSON.parse(titleJson).text || "");
      } catch (e) {
        result.pageTitle = result.title;
      }

      // Extract main content
      var contentSelectors = [
        "main",
        "article",
        ".content",
        "#content",
        ".post-content",
        ".entry-content",
        "body",
      ];

      var extractedContent = "";
      for (var s = 0; s < contentSelectors.length; s++) {
        if (extractedContent.length > 500) break;
        try {
          var contentJson = floorp.tabElementText(pageTab, contentSelectors[s]);
          var content = JSON.parse(contentJson).text || "";
          if (content.length > extractedContent.length) {
            extractedContent = content;
          }
        } catch (e) {}
      }

      // Clean and limit content
      result.pageContent = cleanText(extractedContent).substring(0, 2000);
      result.extractedAt = new Date().toISOString();

      if (result.pageContent.length > 100) {
        successCount++;
        console.log(
          "       ✓ Extracted " + result.pageContent.length + " chars",
        );
      } else {
        failCount++;
        console.log(
          "       ⚠ Limited content (" + result.pageContent.length + " chars)",
        );
      }
    } catch (e) {
      failCount++;
      console.log("       ✗ Error: " + e);
      result.pageContent = result.snippet;
    } finally {
      if (pageTab) {
        try {
          floorp.closeTab(pageTab);
          console.log("       - Tab destroyed");
        } catch (err) {}
      }
    }
    // Small delay between requests
    sleep(500);
  }

  console.log("");
  console.log("  ✓ Successfully extracted: " + successCount + " pages");
  console.log("  ⚠ Limited/Failed: " + failCount + " pages");

  // --- Phase 3: LLM Analysis ---
  console.log("");
  console.log("━━━ Phase 3: AI Analysis ━━━");

  // Summarize each page
  var analyzedResults = [];
  for (var i = 0; i < searchResults.length; i++) {
    var result = searchResults[i];
    var progress = "[" + (i + 1) + "/" + searchResults.length + "]";
    console.log(
      progress + " Analyzing: " + result.title.substring(0, 40) + "...",
    );

    try {
      var summary = iniad_ai_mop.chat(
        "You are a research analyst. Analyze the following web page content about '" +
          searchQuery +
          "' and provide: 1) Key information related to '" +
          searchQuery +
          "', 2) The context/perspective of this source. Write in Japanese, 2-3 sentences.",
        "Page Title: " +
          result.pageTitle +
          "\n\nContent:\n" +
          result.pageContent.substring(0, 1500),
      );

      analyzedResults.push({
        result: result,
        summary: summary,
        category: categorizeContent(result.pageContent),
      });
    } catch (e) {
      analyzedResults.push({
        result: result,
        summary: "(分析エラー)",
        category: "other",
      });
    }
  }

  // Generate comprehensive sections
  console.log("");
  console.log("  Generating comprehensive analysis...");

  var allSummaries = analyzedResults
    .map(function (a, i) {
      return "[" + (i + 1) + "] " + a.result.title + ": " + a.summary;
    })
    .join("\n\n");

  console.log("  → Abstract...");
  var abstractText = generateAnalysis(
    "abstract",
    searchQuery,
    allSummaries,
    analyzedResults.length,
  );

  console.log("  → Overview...");
  var overviewText = generateAnalysis(
    "overview",
    searchQuery,
    allSummaries,
    analyzedResults.length,
  );

  // Generate detailed Key Findings with multiple subsections
  console.log("  → Key Findings (15 detailed sections)...");

  // Use LLM to generate appropriate section titles and prompts based on the topic
  console.log("    → Generating dynamic section structure...");
  var sectionStructurePrompt =
    "あなたは調査レポートの構成を設計する専門家です。「" +
    searchQuery +
    "」について包括的な調査レポートを作成するために、15個のセクションタイトルとそれぞれの分析プロンプトを生成してください。\n\n" +
    "以下のJSON形式で出力してください（他の文章は不要、JSONのみ出力）：\n" +
    '[{"id":"3.1","title":"セクションタイトル","prompt":"このセクションで分析すべき内容の詳細な指示（1000-1500語で記述するよう指定）"},...]\n\n' +
    "セクションは以下の観点を網羅してください：\n" +
    "1. 定義と概要\n2. 技術的特徴\n3. 主要機能\n4. ユーザー体験\n5. 差別化要因\n6. 開発体制\n7. コミュニティ\n8. セキュリティ/安全性\n9. パフォーマンス\n10. エコシステム\n11. 市場ポジション\n12. 競合比較\n13. 課題とリスク\n14. ユーザー評価\n15. 将来展望\n\n" +
    "各プロンプトは1000-1500語の詳細な分析を要求し、情報源番号の引用を指示してください。「" +
    searchQuery +
    "」の性質に合わせて適切な内容にしてください。";

  var findingsList = [];

  try {
    var sectionJson = retryWithBackoff(
      function () {
        return iniad_ai_mop.chat(
          "You are a JSON generator. Output ONLY valid JSON array, no markdown, no explanation.",
          sectionStructurePrompt,
        );
      },
      3,
      2000,
    );

    // Parse JSON response
    sectionJson = sectionJson
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();

    // Find JSON array in response
    var jsonStart = sectionJson.indexOf("[");
    var jsonEnd = sectionJson.lastIndexOf("]") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      sectionJson = sectionJson.slice(jsonStart, jsonEnd);
    }

    findingsList = JSON.parse(sectionJson);
    console.log("    ✓ Generated " + findingsList.length + " dynamic sections");
  } catch (e) {
    console.log("    ⚠ Section generation failed: " + e.message);
    console.log("    → Using default section structure");

    // Fallback to default sections
    findingsList = [
      {
        id: "3.1",
        title: "定義と概要",
        prompt:
          "「" +
          searchQuery +
          "」の基本的な定義、歴史、背景を1000-1500語で詳細に分析してください。情報源番号を引用してください。",
      },
      {
        id: "3.2",
        title: "技術的特徴",
        prompt:
          "「" +
          searchQuery +
          "」の技術的な特徴、仕様、アーキテクチャを1000-1500語で分析してください。",
      },
      {
        id: "3.3",
        title: "主要機能",
        prompt:
          "「" +
          searchQuery +
          "」の主要な機能と特徴を1000-1500語で説明してください。",
      },
      {
        id: "3.4",
        title: "ユーザー体験",
        prompt:
          "「" +
          searchQuery +
          "」のユーザー体験、使い勝手を1000-1500語で分析してください。",
      },
      {
        id: "3.5",
        title: "市場分析",
        prompt:
          "「" +
          searchQuery +
          "」の市場でのポジション、競合との比較を1000-1500語で分析してください。",
      },
    ];
    console.log("    ✓ Using " + findingsList.length + " fallback sections");
  }

  // Ensure we have exactly 15 sections
  while (findingsList.length < 15) {
    findingsList.push({
      id: "3." + (findingsList.length + 1),
      title: "追加分析 " + (findingsList.length + 1),
      prompt:
        "「" +
        searchQuery +
        "」に関する追加的な分析を1000-1200語で行ってください。情報源番号を引用してください。",
    });
  }
  findingsList = findingsList.slice(0, 15);

  var findingsTexts = [];
  var mentionedKeywords = []; // Track keywords across sections

  for (var f = 0; f < findingsList.length; f++) {
    var item = findingsList[f];
    console.log("    → " + item.id + " " + item.title + "...");

    // Use the enhanced generation function with retry and keyword awareness
    var sectionContent = generateDetailedFindingsWithRetry(
      searchQuery,
      allSummaries,
      item.id,
      item.prompt,
      mentionedKeywords,
    );

    findingsTexts.push({
      id: item.id,
      title: item.title,
      content: sectionContent,
    });

    // Extract and accumulate keywords from this section
    var newKeywords = extractKeywords(sectionContent);
    newKeywords.forEach(function (kw) {
      if (mentionedKeywords.indexOf(kw) < 0) {
        mentionedKeywords.push(kw);
      }
    });

    // Keep keyword list manageable (max 50)
    if (mentionedKeywords.length > 50) {
      mentionedKeywords = mentionedKeywords.slice(-50);
    }

    console.log(
      "      ✓ " +
        sectionContent.length +
        " chars | Keywords: " +
        newKeywords.length,
    );
  }

  console.log("  → Discussion...");
  var discussionText = generateAnalysis(
    "discussion",
    searchQuery,
    allSummaries,
    analyzedResults.length,
  );

  console.log("  → Conclusions...");
  var conclusionsText = generateAnalysis(
    "conclusions",
    searchQuery,
    allSummaries,
    analyzedResults.length,
  );

  // --- Phase 4: Generate Report ---
  console.log("");
  console.log("━━━ Phase 4: Report Generation ━━━");

  var today = new Date().toISOString().split("T")[0];
  var report = "";

  // Title
  report += "# " + searchQuery + ": Comprehensive Web Analysis Report\n\n";
  report += "**Deep Research** | Generated: " + today + "\n\n";
  report += "---\n\n";

  // Executive Summary Box
  report += "> **Executive Summary**\n>\n";
  report +=
    "> 本レポートは、「" +
    searchQuery +
    "」に関する " +
    analyzedResults.length +
    " 件のWeb情報源を自動収集・分析した包括的調査報告書である。\n";
  report +=
    "> DuckDuckGo検索結果から各ページを個別に訪問し、詳細なコンテンツを抽出した上で、AIによる分析・要約を実施した。\n\n";

  // Abstract
  report += "## Abstract\n\n";
  report += abstractText + "\n\n";
  report += "---\n\n";

  // Table of Contents
  report += "## Table of Contents\n\n";
  report += "1. [Overview](#1-overview)\n";
  report += "2. [Methodology](#2-methodology)\n";
  report += "3. [Key Findings](#3-key-findings)\n";
  report += "4. [Source Analysis](#4-source-analysis)\n";
  report += "5. [Discussion](#5-discussion)\n";
  report += "6. [Conclusions](#6-conclusions)\n";
  report += "7. [References](#7-references)\n";
  report += "8. [Fact-Check Summary](#8-fact-check-summary)\n\n";
  report += "---\n\n";

  // Overview
  report += "## 1. Overview\n\n";
  report += overviewText + "\n\n";

  // Methodology
  report += "## 2. Methodology\n\n";
  report += "### 2.1 Data Collection\n\n";
  report += "本調査では、以下のプロセスでデータを収集した：\n\n";
  report += "1. **検索フェーズ**: DuckDuckGoで「" + searchQuery + "」を検索\n";
  report += "2. **URL収集**: 上位 " + searchResults.length + " 件のURLを取得\n";
  report += "3. **ページ訪問**: 各URLを実際にブラウザで訪問\n";
  report += "4. **コンテンツ抽出**: 各ページの本文テキストを自動抽出\n\n";

  report += "### 2.2 Analysis Pipeline\n\n";
  report += "```\n";
  report +=
    "検索 → URL収集 → ページ訪問 → コンテンツ抽出 → AI分析 → 要約生成 → レポート作成\n";
  report += "```\n\n";

  report += "### 2.3 Statistics\n\n";
  report += "| 項目 | 数値 |\n";
  report += "|------|-----|\n";
  report += "| 検索結果取得数 | " + searchResults.length + " |\n";
  report += "| コンテンツ抽出成功 | " + successCount + " |\n";
  report += "| 抽出制限/失敗 | " + failCount + " |\n";
  report += "| 分析完了 | " + analyzedResults.length + " |\n\n";

  // Key Findings - 15 Detailed Sections
  report += "## 3. Key Findings\n\n";
  report +=
    "本調査では、収集した " +
    analyzedResults.length +
    " 件の情報源を多角的に分析し、以下の 15 の専門的観点から詳細な発見事項を網羅的にまとめた。\n\n";

  for (var t = 0; t < findingsTexts.length; t++) {
    var f = findingsTexts[t];
    report += "### " + f.id + " " + f.title + "\n\n";
    report += f.content + "\n\n";
    if (t < findingsTexts.length - 1) report += "---\n\n";
  }

  // Source Analysis
  report += "## 4. Source Analysis\n\n";

  // Group by category
  var categories = {
    official: { name: "公式・開発者情報", items: [] },
    news: { name: "ニュース・メディア", items: [] },
    review: { name: "レビュー・比較", items: [] },
    community: { name: "コミュニティ・フォーラム", items: [] },
    other: { name: "その他", items: [] },
  };

  analyzedResults.forEach(function (a) {
    var cat = a.category || "other";
    if (categories[cat]) {
      categories[cat].items.push(a);
    } else {
      categories.other.items.push(a);
    }
  });

  Object.keys(categories).forEach(function (catKey) {
    var cat = categories[catKey];
    if (cat.items.length > 0) {
      report +=
        "### 4." +
        (Object.keys(categories).indexOf(catKey) + 1) +
        " " +
        cat.name +
        " (" +
        cat.items.length +
        "件)\n\n";

      cat.items.forEach(function (a, idx) {
        report += "#### [" + a.result.rank + "] " + a.result.title + "\n\n";
        report +=
          "- **URL**: [" + a.result.domain + "](" + a.result.url + ")\n";
        report += "- **分析**: " + a.summary + "\n\n";
      });
    }
  });

  // Discussion
  report += "## 5. Discussion\n\n";
  report += discussionText + "\n\n";

  // Conclusions
  report += "## 6. Conclusions\n\n";
  report += conclusionsText + "\n\n";

  // References
  report += "---\n\n";
  report += "## 7. References\n\n";
  searchResults.forEach(function (r, i) {
    report +=
      "[" +
      (i + 1) +
      '] "' +
      r.title +
      '." *' +
      r.domain +
      "*. " +
      r.url +
      "\n\n";
  });

  // Fact-Check Section
  console.log("");
  console.log("━━━ Phase 5: Fact-Checking ━━━");
  var factCheckSection = generateFactCheckReport(findingsTexts, allSummaries);
  report += factCheckSection;

  report += "---\n\n";
  report += "*This report was automatically generated by Deep Research.*\n";
  report +=
    "*Analysis powered by AI-driven content extraction and synthesis.*\n";
  report +=
    "*Fact-checking enabled: Claims verified against source documents.*\n";
  report += "*Generated: " + new Date().toISOString() + "*\n";

  // --- Phase 5: Save Report ---
  console.log("  Report size: " + report.length + " characters");
  console.log("  Saving to: " + outputPath);

  try {
    var result = app.sapphillon.core.filesystem.write(outputPath, report);
    console.log("  ✓ Report saved successfully!");
  } catch (e) {
    console.log("  ✗ Save error: " + e);
    console.log("");
    console.log("━━━ REPORT OUTPUT ━━━");
    console.log(report);
  }

  console.log("");
  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║               Deep Research Complete!                      ║");
  console.log(
    "║    Analyzed " +
      analyzedResults.length +
      " sources | Report: " +
      (report.length / 1000).toFixed(1) +
      " KB              ║",
  );
  console.log("╚════════════════════════════════════════════════════════════╝");
}

// --- Helper Functions ---

function generateAnalysis(type, topic, summaries, count) {
  var systemPrompt =
    "You are an expert researcher writing in Japanese. Use formal academic tone.";
  var prompt = "";

  switch (type) {
    case "abstract":
      prompt =
        "以下の" +
        count +
        "件のWeb情報源の分析に基づき、「" +
        topic +
        "」に関する包括的なAbstract（200-250語）を書いてください。背景、調査範囲、主要な発見、意義を含めてください。\n\n情報源の要約:\n" +
        summaries;
      break;
    case "overview":
      prompt =
        "「" +
        topic +
        "」とは何か、その特徴、歴史、現在の状況について、収集した情報に基づいてOverviewセクション（300-400語）を書いてください。";
      break;
    case "findings":
      prompt =
        "以下の情報源から抽出した主要な発見を3-5つのテーマに分類し、各テーマについて詳しく説明してください（400-500語）。情報源番号を引用（例：[1][3]）してください。\n\n情報源:\n" +
        summaries;
      break;
    case "discussion":
      prompt =
        "「" +
        topic +
        "」に関する調査結果を総括し、Discussionセクション（250-300語）を書いてください。傾向、強み、課題、将来の展望を含めてください。";
      break;
    case "conclusions":
      prompt =
        "「" +
        topic +
        "」に関する本調査の結論（150-200語）を書いてください。主要なポイントと今後の発展可能性を含めてください。";
      break;
  }

  try {
    return iniad_ai_mop.chat(systemPrompt, prompt);
  } catch (e) {
    return "（" + type + "の生成に失敗しました）";
  }
}

// Generate detailed findings for a specific topic
function generateDetailedFindings(topic, summaries, sectionType, customPrompt) {
  var systemPrompt =
    "You are an expert research analyst writing in Japanese. Use formal academic tone with detailed explanations. " +
    "Structure your response with clear paragraphs and comprehensive analysis. " +
    "Always cite source numbers like [1], [3], [5] when referring to specific information. " +
    "IMPORTANT: This is one section of a multi-section report. Avoid repeating the same examples, facility names, or project names that are commonly mentioned. " +
    "Focus on NEW insights and unique perspectives specific to this section's theme. " +
    "If you must reference a commonly mentioned item, do so briefly without re-explaining it.";

  var fullPrompt =
    customPrompt +
    "\n\n【重要】他のセクションで既に詳述されている内容（施設名、プロジェクト名、基本的な学部概要など）は簡潔に触れるにとどめ、このセクション固有の新しい観点・分析に重点を置いてください。\n\n情報源一覧:\n" +
    summaries;

  try {
    return iniad_ai_mop.chat(systemPrompt, fullPrompt);
  } catch (e) {
    return "（" + sectionType + "の詳細分析の生成に失敗しました）";
  }
}

function categorizeContent(content) {
  var lowerContent = content.toLowerCase();

  if (
    lowerContent.includes("github") ||
    lowerContent.includes("開発") ||
    lowerContent.includes("developer") ||
    lowerContent.includes("ablaze")
  ) {
    return "official";
  }
  if (
    lowerContent.includes("news") ||
    lowerContent.includes("ニュース") ||
    lowerContent.includes("発表") ||
    lowerContent.includes("リリース")
  ) {
    return "news";
  }
  if (
    lowerContent.includes("review") ||
    lowerContent.includes("レビュー") ||
    lowerContent.includes("比較") ||
    lowerContent.includes("おすすめ")
  ) {
    return "review";
  }
  if (
    lowerContent.includes("reddit") ||
    lowerContent.includes("forum") ||
    lowerContent.includes("コミュニティ") ||
    lowerContent.includes("質問")
  ) {
    return "community";
  }
  return "other";
}

function getText(tab, sel) {
  try {
    var json = floorp.tabElementText(tab, sel);
    var parsed = JSON.parse(json);
    return parsed.text || "";
  } catch (e) {
    return "";
  }
}

function cleanText(str) {
  if (!str) return "";
  return str.replace(/\s+/g, " ").trim();
}

function extractDomain(url) {
  try {
    var match = url.match(/https?:\/\/([^\/]+)/);
    return match ? match[1] : url;
  } catch (e) {
    return url;
  }
}

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

// ============================================================================
// Relevance Filtering Function
// ============================================================================

// Filter search results for relevance to the query using AI
function filterRelevantResults(results, query) {
  if (results.length === 0) return results;

  var systemPrompt =
    "あなたは検索結果の関連性を評価する専門家です。\n\n" +
    "【タスク】\n" +
    "検索クエリと各検索結果（タイトル・URL・スニペット）を比較し、関連性をスコアリングしてください。\n\n" +
    "【スコア基準】\n" +
    "5: 非常に関連性が高い（クエリに直接回答する内容）\n" +
    "4: 関連性が高い（クエリのトピックに関連する重要な情報）\n" +
    "3: やや関連性あり（関連はあるが間接的）\n" +
    "2: 関連性が低い（ほとんど関連がない）\n" +
    "1: 無関連（全く関連がない、広告、スパム等）\n\n" +
    "【除外すべきもの】\n" +
    "- 単なる商品ページ（比較やレビューがないもの）\n" +
    "- ニュース以外の速報サイト\n" +
    "- フォーラムの質問ページ（回答がないもの）\n" +
    "- 明らかに異なるトピックのページ\n\n" +
    "【出力形式】\n" +
    "各結果のインデックスとスコアをJSON配列で出力:\n" +
    '[{"index": 0, "score": 5, "reason": "直接比較記事"}, ...]';

  // Process in batches of 10 for efficiency
  var batchSize = 10;
  var scoredResults = [];

  for (
    var batchStart = 0;
    batchStart < results.length;
    batchStart += batchSize
  ) {
    var batch = results.slice(
      batchStart,
      Math.min(batchStart + batchSize, results.length),
    );

    var resultsList = batch
      .map(function (r, idx) {
        return (
          batchStart +
          idx +
          ". " +
          r.title +
          " | " +
          r.domain +
          " | " +
          (r.snippet || "").slice(0, 100)
        );
      })
      .join("\n");

    var checkPrompt =
      "【検索クエリ】\n" +
      query +
      "\n\n" +
      "【検索結果一覧】\n" +
      resultsList +
      "\n\n" +
      "上記の各結果について、検索クエリとの関連性をスコアリングしてJSONで出力してください。";

    try {
      var response = iniad_ai_mop.chat(systemPrompt, checkPrompt);
      response = response
        .replace(/```json\n?/g, "")
        .replace(/```\n?/g, "")
        .trim();

      // Find JSON array
      var jsonStart = response.indexOf("[");
      var jsonEnd = response.lastIndexOf("]") + 1;
      if (jsonStart >= 0 && jsonEnd > jsonStart) {
        response = response.slice(jsonStart, jsonEnd);
      }

      var scores = JSON.parse(response);
      scores.forEach(function (s) {
        if (s.score >= 3) {
          var originalIdx =
            typeof s.index === "number" ? s.index : parseInt(s.index, 10);
          if (originalIdx >= 0 && originalIdx < results.length) {
            scoredResults.push({
              result: results[originalIdx],
              score: s.score,
              reason: s.reason || "",
            });
          }
        }
      });

      console.log(
        "    Batch " +
          Math.floor(batchStart / batchSize + 1) +
          ": " +
          scores.filter(function (s) {
            return s.score >= 3;
          }).length +
          "/" +
          batch.length +
          " relevant",
      );
    } catch (e) {
      console.log(
        "    ⚠ Batch " +
          Math.floor(batchStart / batchSize + 1) +
          " filter error: " +
          e.message,
      );
      // On error, include all from this batch
      batch.forEach(function (r) {
        scoredResults.push({ result: r, score: 3, reason: "auto-included" });
      });
    }
  }

  // Sort by score descending and extract results
  scoredResults.sort(function (a, b) {
    return b.score - a.score;
  });

  // Log top results
  console.log("  Top results by relevance:");
  for (var i = 0; i < Math.min(5, scoredResults.length); i++) {
    console.log(
      "    [" +
        scoredResults[i].score +
        "] " +
        scoredResults[i].result.title.slice(0, 40) +
        "...",
    );
  }

  return scoredResults.map(function (sr) {
    return sr.result;
  });
}

// ============================================================================
// Quality Improvement Helper Functions
// ============================================================================

// Check if content is empty or too short (likely generation failure)
function isContentEmpty(content) {
  if (!content) return true;
  var trimmed = content.trim();
  // Consider empty if less than 100 characters (typical failure message is ~50 chars)
  return trimmed.length < 100;
}

// Check if content is truncated (ends mid-sentence)
function isContentTruncated(content) {
  if (!content) return false;
  var trimmed = content.trim();
  // Check if it ends with incomplete patterns
  var lastChar = trimmed.slice(-1);
  var lastTwoChars = trimmed.slice(-2);

  // Good endings: 。！？」）.!?")
  var validEndings = [
    "。",
    "！",
    "？",
    "」",
    "）",
    ".",
    "!",
    "?",
    '"',
    ")",
    "]",
  ];
  if (validEndings.indexOf(lastChar) >= 0) return false;

  // Likely truncated if ends with: 、, incomplete word, etc.
  var badEndings = [
    "、",
    "の",
    "は",
    "が",
    "を",
    "に",
    "で",
    "と",
    "も",
    "し",
    "や",
  ];
  if (badEndings.indexOf(lastChar) >= 0) return true;

  // Check for cut-off patterns
  if (trimmed.match(/[a-zA-Z]$/)) return true; // Ends with letter (English word cut)

  return false;
}

// Retry LLM call with exponential backoff
function retryWithBackoff(fn, maxRetries, initialDelay) {
  var lastError = null;
  var delay = initialDelay || 1000;

  for (var attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      var result = fn();
      return result;
    } catch (e) {
      lastError = e;
      console.log(
        "      ⚠ Attempt " +
          attempt +
          " failed, retrying in " +
          delay +
          "ms...",
      );
      sleep(delay);
      delay = delay * 2; // Exponential backoff
    }
  }

  throw lastError || new Error("All retry attempts failed");
}

// Extract key concepts/keywords from generated content
function extractKeywords(content) {
  if (!content) return [];

  // Extract quoted terms, proper nouns, and repeated keywords
  var keywords = [];

  // Match Japanese quoted terms 「...」
  var jpQuoted = content.match(/「([^」]+)」/g) || [];
  jpQuoted.forEach(function (m) {
    var term = m.replace(/[「」]/g, "");
    if (term.length >= 2 && term.length <= 30) {
      keywords.push(term);
    }
  });

  // Match English proper nouns (capitalized words)
  var engProper =
    content.match(/\b[A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)?\b/g) || [];
  engProper.forEach(function (w) {
    if (w.length >= 3 && keywords.indexOf(w) < 0) {
      keywords.push(w);
    }
  });

  // Limit to top 20 unique keywords
  var unique = [];
  keywords.forEach(function (k) {
    if (unique.indexOf(k) < 0 && unique.length < 20) {
      unique.push(k);
    }
  });

  return unique;
}

// Enhanced generateDetailedFindings with retry, truncation repair, and keyword awareness
function generateDetailedFindingsWithRetry(
  topic,
  summaries,
  sectionType,
  customPrompt,
  mentionedKeywords,
) {
  var MAX_RETRIES = 3;
  var content = "";
  var attempt = 0;

  // Build keyword context if provided
  var keywordContext = "";
  if (mentionedKeywords && mentionedKeywords.length > 0) {
    keywordContext =
      "\n\n【既出のキーワード（詳述不要）】: " +
      mentionedKeywords.slice(0, 15).join("、");
  }

  var systemPrompt =
    "You are an expert research analyst writing in Japanese. Use formal academic tone with detailed explanations. " +
    "Structure your response with clear paragraphs and comprehensive analysis. " +
    "Always cite source numbers like [1], [3], [5] when referring to specific information. " +
    "IMPORTANT: This is one section of a multi-section report. Avoid repeating the same examples, facility names, or project names that are commonly mentioned. " +
    "Focus on NEW insights and unique perspectives specific to this section's theme. " +
    "If you must reference a commonly mentioned item, do so briefly without re-explaining it. " +
    "CRITICAL: Complete your response with a proper conclusion. Do not end mid-sentence.";

  var basePrompt =
    customPrompt +
    "\n\n【重要】他のセクションで既に詳述されている内容（施設名、プロジェクト名、基本的な学部概要など）は簡潔に触れるにとどめ、このセクション固有の新しい観点・分析に重点を置いてください。" +
    keywordContext +
    "\n\n情報源一覧:\n" +
    summaries;

  while (attempt < MAX_RETRIES) {
    attempt++;

    try {
      content = retryWithBackoff(
        function () {
          return iniad_ai_mop.chat(systemPrompt, basePrompt);
        },
        2,
        500,
      );

      // Check for empty content
      if (isContentEmpty(content)) {
        console.log(
          "      ⚠ Empty content detected, retrying (" +
            attempt +
            "/" +
            MAX_RETRIES +
            ")...",
        );
        sleep(1000);
        continue;
      }

      // Check for truncation
      if (isContentTruncated(content)) {
        console.log(
          "      ⚠ Truncated content detected, requesting completion...",
        );

        // Try to complete the truncated content
        var completionPrompt =
          "以下の文章は途中で切れています。最後の段落を適切に完結させてください。新しい内容は追加せず、結論文のみを追加してください。\n\n" +
          "【途中の文章】\n" +
          content.slice(-500);

        try {
          var completion = iniad_ai_mop.chat(
            "Complete the following Japanese text naturally. Only add the ending, no repetition.",
            completionPrompt,
          );

          if (completion && completion.length > 10) {
            // Find where to append (after the last complete sentence)
            var lastPeriod = content.lastIndexOf("。");
            if (lastPeriod > content.length - 100) {
              content =
                content.substring(0, lastPeriod + 1) +
                "\n\n" +
                completion.trim();
            } else {
              content = content + completion.trim();
            }
            console.log("      ✓ Content completion added");
          }
        } catch (compErr) {
          console.log("      ⚠ Completion failed, using original");
        }
      }

      // Content is valid, break the loop
      if (!isContentEmpty(content)) {
        break;
      }
    } catch (e) {
      console.log("      ✗ Generation failed: " + e.message);
      if (attempt >= MAX_RETRIES) {
        return (
          "（" +
          sectionType +
          "の詳細分析の生成に失敗しました。エラー: " +
          e.message +
          "）"
        );
      }
      sleep(2000 * attempt);
    }
  }

  // Final fallback if still empty
  if (isContentEmpty(content)) {
    return (
      "（" +
      sectionType +
      "の詳細分析は、情報が不足しているため生成できませんでした。）"
    );
  }

  return content;
}

// ============================================================================
// Fact-Checking Functions
// ============================================================================

// Extract specific claims (numbers, dates, specs) from generated content
function extractClaims(content) {
  if (!content) return [];

  var claims = [];

  // Extract numerical claims (e.g., "8時間", "4倍", "$249", "2025年")
  var numberPatterns = [
    /(\d+(?:\.\d+)?)\s*(?:時間|h|hours?)/gi, // Battery hours
    /(\d+(?:\.\d+)?)\s*(?:倍|x|times)/gi, // Multiplier claims
    /(?:\$|¥|円)?\s*(\d{1,3}(?:,\d{3})*(?:\.\d+)?)\s*(?:円|ドル|\$)?/gi, // Prices
    /(\d{4})年(\d{1,2})月?(\d{1,2})?日?/g, // Dates
    /(\d+)\s*(?:mm|ミリ|g|グラム|mAh)/gi, // Measurements
    /IP\d{2}/gi, // IP ratings
    /Bluetooth\s*\d+\.\d+/gi, // Bluetooth versions
    /H\d+\s*チップ/gi, // Chip names
  ];

  numberPatterns.forEach(function (pattern) {
    var matches = content.match(pattern) || [];
    matches.forEach(function (m) {
      if (claims.indexOf(m) < 0) {
        claims.push(m.trim());
      }
    });
  });

  // Extract proper nouns and technical terms (generic patterns)
  var properNounPatterns = [
    /[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+/g, // Multi-word proper nouns (e.g., "Apple Watch")
    /[A-Z][A-Z0-9]+(?:\s*[A-Z0-9]+)*/g, // Acronyms (e.g., "USB-C", "ANC")
    /[A-Z][a-z]+\s*\d+(?:\s*(?:世代|nd|rd|th|st))?/gi, // Product with version (e.g., "Pro 3", "Series 7")
    /(?:第|Ver\.?|v|version)\s*\d+(?:\.\d+)?/gi, // Version numbers
  ];

  properNounPatterns.forEach(function (pattern) {
    var matches = content.match(pattern) || [];
    matches.forEach(function (m) {
      if (claims.indexOf(m) < 0 && m.length > 2) {
        claims.push(m.trim());
      }
    });
  });

  // Limit to 30 unique claims
  return claims.slice(0, 30);
}

// Verify a specific claim against source summaries
function verifyClaim(claim, sourceSummaries) {
  // Check if the claim appears in any source summary
  var lowerClaim = claim.toLowerCase();
  var foundInSources = [];

  sourceSummaries.forEach(function (summary, index) {
    if (summary.toLowerCase().indexOf(lowerClaim) >= 0) {
      foundInSources.push(index + 1);
    }
  });

  return {
    claim: claim,
    verified: foundInSources.length > 0,
    sources: foundInSources,
    confidence:
      foundInSources.length >= 2
        ? "high"
        : foundInSources.length === 1
          ? "medium"
          : "low",
  };
}

// Run fact-check on generated content
function factCheckContent(content, sourceSummaries) {
  var claims = extractClaims(content);
  var results = {
    total: claims.length,
    verified: 0,
    unverified: 0,
    highConfidence: 0,
    mediumConfidence: 0,
    lowConfidence: 0,
    details: [],
  };

  claims.forEach(function (claim) {
    var verification = verifyClaim(claim, sourceSummaries);
    results.details.push(verification);

    if (verification.verified) {
      results.verified++;
    } else {
      results.unverified++;
    }

    if (verification.confidence === "high") {
      results.highConfidence++;
    } else if (verification.confidence === "medium") {
      results.mediumConfidence++;
    } else {
      results.lowConfidence++;
    }
  });

  return results;
}

// Use LLM to cross-verify important claims with enhanced prompts
function verifyClaimsWithLLM(content, sourceSummaries) {
  var systemPrompt =
    "あなたは厳密なファクトチェッカーです。AIが生成したコンテンツを情報源と比較し、事実の正確性を検証します。\n\n" +
    "【検証カテゴリ】\n" +
    "1. 数値データ: 数量、価格、割合、サイズ、時間、距離など\n" +
    "2. 日付情報: 発売日、設立日、イベント日時、期間など\n" +
    "3. 仕様・スペック: 技術仕様、バージョン、規格、型番など\n" +
    "4. 機能・特徴: 製品・サービスの機能、特性、性能など\n" +
    "5. 比較表現:「〜倍」「〜%向上」「最大〜」「業界初」等の定量的比較\n\n" +
    "【検証基準】\n" +
    "- verified: 情報源に明確に記載されており、数値も一致\n" +
    "- partially_verified: 情報源に類似の記述があるが、数値や詳細が異なる\n" +
    "- unverified: 情報源に該当する記述が見つからない\n" +
    "- fabricated: 情報源と明らかに矛盾、または存在しない情報\n\n" +
    "【ハルシネーションの兆候】\n" +
    "- 過度に具体的な数値（小数点以下まで、または非公開の内部情報）\n" +
    "- 情報源に存在しない固有名詞や専門用語\n" +
    "- 将来の予測を事実として記述\n" +
    "- 曖昧な引用（「〜という報告がある」「〜とされている」）\n\n" +
    "【出力形式】\n" +
    "必ず以下のJSON配列のみを出力してください（説明文不要）:\n" +
    '[{"claim":"検証対象の主張","category":"数値|日付|仕様|機能|比較","status":"verified|partially_verified|unverified|fabricated","confidence":1-5,"source_ref":"該当する情報源番号（例:[1][3]）またはnull","reason":"20字以内の根拠"}]';

  var checkPrompt =
    "【タスク】以下の生成コンテンツに含まれる事実的主張を、情報源と照合して検証してください。\n\n" +
    "【検証手順】\n" +
    "1. 生成コンテンツから具体的な数値・日付・仕様を抽出\n" +
    "2. 各主張を情報源の該当箇所と比較\n" +
    "3. 一致度を評価し、ステータスを決定\n" +
    "4. 特にハルシネーション（捏造）の可能性が高いものを重点的にチェック\n\n" +
    "【生成コンテンツ】\n" +
    content.slice(0, 2500) +
    "\n\n【情報源一覧】\n" +
    sourceSummaries.slice(0, 4000) +
    "\n\n最も重要な10件の主張について検証結果をJSON配列で出力してください。";

  try {
    var result = iniad_ai_mop.chat(systemPrompt, checkPrompt);
    // Parse JSON from response
    result = result
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();

    // Find JSON array in response
    var jsonStart = result.indexOf("[");
    var jsonEnd = result.lastIndexOf("]") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      result = result.slice(jsonStart, jsonEnd);
    }

    return JSON.parse(result);
  } catch (e) {
    console.log("    ⚠ LLM fact-check failed: " + e.message);
    return [];
  }
}

// Generate fact-check summary report section
function generateFactCheckReport(findingsTexts, sourceSummaries) {
  console.log("  → Fact-checking generated content...");

  var allContent = findingsTexts
    .map(function (f) {
      return f.content;
    })
    .join("\n\n");
  var summaryList = sourceSummaries.split("\n\n").map(function (s) {
    return s;
  });

  // Basic pattern-based fact-check
  var basicResults = factCheckContent(allContent, summaryList);
  console.log(
    "    → Basic check: " +
      basicResults.verified +
      "/" +
      basicResults.total +
      " claims found in sources",
  );

  // LLM-based deep verification (sample up to 3 sections)
  var llmResults = [];
  for (var i = 0; i < Math.min(3, findingsTexts.length); i++) {
    var sectionResults = verifyClaimsWithLLM(
      findingsTexts[i].content,
      sourceSummaries,
    );
    llmResults = llmResults.concat(sectionResults);
  }
  console.log(
    "    → LLM verification: " + llmResults.length + " claims analyzed",
  );

  // Generate report section
  var report = "## 8. Fact-Check Summary\n\n";
  report += "> **検証結果概要**\n>\n";
  report += "> 本レポートの内容を情報源と照合し、事実確認を実施しました。\n\n";

  report += "### 8.1 パターンベース検証\n\n";
  report += "| 項目 | 数値 |\n";
  report += "|------|------|\n";
  report += "| 抽出された主張数 | " + basicResults.total + " |\n";
  report += "| 情報源で確認 | " + basicResults.verified + " |\n";
  report += "| 未確認 | " + basicResults.unverified + " |\n";
  report += "| 高信頼度 | " + basicResults.highConfidence + " |\n";
  report += "| 中信頼度 | " + basicResults.mediumConfidence + " |\n";
  report += "| 要確認 | " + basicResults.lowConfidence + " |\n\n";

  if (llmResults.length > 0) {
    report += "### 8.2 AI検証結果\n\n";
    report += "| カテゴリ | 主張 | ステータス | 信頼度 | 情報源 | 根拠 |\n";
    report += "|:------:|------|:--------:|:-----:|:-----:|------|\n";

    llmResults.forEach(function (r) {
      // Status icons with 4 levels
      var statusIcon = "❓";
      if (r.status === "verified") statusIcon = "✅";
      else if (r.status === "partially_verified") statusIcon = "⚠️";
      else if (r.status === "unverified") statusIcon = "❓";
      else if (r.status === "fabricated") statusIcon = "❌";

      // Category icons
      var catIcon = "📋";
      if (r.category === "数値") catIcon = "🔢";
      else if (r.category === "日付") catIcon = "📅";
      else if (r.category === "仕様") catIcon = "⚙️";
      else if (r.category === "機能") catIcon = "✨";
      else if (r.category === "比較") catIcon = "⚖️";

      // Confidence display
      var confidence = r.confidence || 3;
      var confidenceStr = "";
      for (var c = 0; c < 5; c++) {
        confidenceStr += c < confidence ? "●" : "○";
      }

      report +=
        "| " +
        catIcon +
        " | " +
        (r.claim || "").slice(0, 40) +
        " | " +
        statusIcon +
        " | " +
        confidenceStr +
        " | " +
        (r.source_ref || "-") +
        " | " +
        (r.reason || "").slice(0, 25) +
        " |\n";
    });
    report += "\n";

    // Fabricated content warning
    var fabricated = llmResults.filter(function (r) {
      return r.status === "fabricated";
    });
    if (fabricated.length > 0) {
      report += "> [!CAUTION]\n";
      report +=
        "> **ハルシネーション検出**: 以下の主張は情報源と矛盾するか、捏造の可能性があります。\n>\n";
      fabricated.forEach(function (f) {
        report += "> - " + (f.claim || "").slice(0, 60) + "\n";
      });
      report += "\n";
    }

    // Partially verified content note
    var partial = llmResults.filter(function (r) {
      return r.status === "partially_verified";
    });
    if (partial.length > 0) {
      report += "> [!WARNING]\n";
      report +=
        "> **要確認**: 以下の主張は情報源と部分的に一致しますが、詳細が異なる可能性があります。\n>\n";
      partial.slice(0, 5).forEach(function (p) {
        report += "> - " + (p.claim || "").slice(0, 60) + "\n";
      });
      report += "\n";
    }
  }

  // Add unverified claims warning
  if (basicResults.lowConfidence > 0) {
    report += "### 8.3 要確認事項\n\n";
    report += "> [!WARNING]\n";
    report +=
      "> 以下の主張は情報源で直接確認できませんでした。公式情報との照合を推奨します。\n\n";

    var unverifiedList = basicResults.details
      .filter(function (d) {
        return !d.verified;
      })
      .slice(0, 10);
    unverifiedList.forEach(function (item) {
      report += "- `" + item.claim + "`\n";
    });
    report += "\n";
  }

  report += "---\n\n";

  return report;
}

// --- TEST WORKFLOW: Finder Plugin (AppleScript のみ) ---
function workflow() {
  const testDir = "/Users/user/Desktop";
  const query = "Floorp";
  const maxResults = 20;
  const outPath = testDir + "/finder_test_results.json";

  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║           Finder Plugin Test (AppleScript Only)            ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
  console.log("");
  console.log("  Search Directory: " + testDir);
  console.log("  Search Query: " + query);
  console.log("  Max Results: " + maxResults);
  console.log("");

  const results = {
    startedAt: new Date().toISOString(),
    query: query,
    directory: testDir,
    test: null,
  };

  try {
    // Check plugin availability
    if (!app || !app.sapphillon || !app.sapphillon.core || !app.sapphillon.core.finder) {
      throw new Error("Finder plugin not available (app.sapphillon.core.finder missing)");
    }

    console.log("━━━ Running findFiles (AppleScript) ━━━");
    console.log("");

    const startTime = Date.now();
    const json = app.sapphillon.core.finder.findFiles(testDir, query, maxResults);
    const elapsed = Date.now() - startTime;
    const arr = JSON.parse(json || "[]");

    results.test = {
      name: "findFiles",
      ok: true,
      count: arr.length,
      elapsedMs: elapsed,
      sample: arr.slice(0, 10),
    };

    console.log("  ✓ findFiles returned " + arr.length + " results in " + elapsed + "ms");
    console.log("");

    if (arr.length > 0) {
      console.log("━━━ File List ━━━");
      arr.forEach(function(path, i) {
        console.log("  " + (i + 1) + ". " + path);
      });
      console.log("");
    } else {
      console.log("  ⚠ No results found for query: " + query);
    }

  } catch (e) {
    results.test = {
      name: "findFiles",
      ok: false,
      error: String(e),
    };
    console.log("  ✗ Error: " + e);
  }

  // Save results
  try {
    results.finishedAt = new Date().toISOString();
    const outStr = JSON.stringify(results, null, 2);
    app.sapphillon.core.filesystem.write(outPath, outStr);
    console.log("");
    console.log("━━━ Results saved ━━━");
    console.log("  → " + outPath);
  } catch (e) {
    console.log("  ✗ Failed to write results: " + e);
  }

  console.log("");
  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║                      Test Complete                         ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
}

// Auto-run
workflow();
