/**
 * Floorp Deep Research - Comprehensive Web Analysis
 *
 * 1. Search "Floorp" on DuckDuckGo (30 results)
 * 2. Visit each result URL to extract detailed content
 * 3. Analyze and synthesize with LLM
 * 4. Generate comprehensive academic-style report
 */

function workflow() {
  var searchQuery = "Floorp";
  var maxResults = 30;
  var outputPath = "/Users/user/Desktop/Floorp_Deep_Research_Report.md";

  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║        Floorp Deep Research - Comprehensive Analysis       ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
  console.log("");
  console.log("Search Query: " + searchQuery);
  console.log("Target Results: " + maxResults);
  console.log("");

  // --- Phase 1: DuckDuckGo Search ---
  console.log("━━━ Phase 1: Search & Collect URLs ━━━");
  var searchResults = [];
  var ddgTab = null;

  try {
    var ddgUrl = "https://duckduckgo.com/?q=" + encodeURIComponent(searchQuery);
    ddgTab = floorp.createTab(ddgUrl, false);
    floorp.tabWaitForElement(ddgTab, "article[data-testid='result']", 15000);
    sleep(3000);

    // Load more results by clicking "More Results" button multiple times
    console.log("  Loading more results...");
    for (var click = 0; click < 3; click++) {
      try {
        // Click "More Results" button
        floorp.tabClick(ddgTab, "#more-results");
        sleep(3000);
        console.log("    Clicked 'More Results' - attempt " + (click + 1));
      } catch (e) {
        // Button may not be visible, try scrolling
        try {
          floorp.tabScrollTo(
            ddgTab,
            "article[data-testid='result']:last-of-type"
          );
          sleep(2000);
        } catch (scrollErr) {}
      }
    }

    // Collect search results using ol > li structure
    console.log("  Collecting search results...");

    // DuckDuckGo wraps each article in a separate li, so use li:nth-child instead
    for (var i = 1; i <= maxResults + 20; i++) {
      if (searchResults.length >= maxResults) break;

      // Use li:nth-child within the results list
      var baseSel =
        "ol.react-results--main > li:nth-child(" +
        i +
        ") article[data-testid='result']";
      try {
        var titleSel = baseSel + " a[data-testid='result-title-a']";
        var title = getText(ddgTab, titleSel);

        if (!title) continue;

        var linkEl = floorp.tabAttribute(ddgTab, titleSel, "href");
        var url = "";
        try {
          url = JSON.parse(linkEl).value || "";
        } catch (e) {
          continue;
        }

        // Skip certain domains
        if (
          url.includes("youtube.com") ||
          url.includes("twitter.com") ||
          url.includes("facebook.com") ||
          url.includes("instagram.com")
        ) {
          continue;
        }

        // Get snippet from article text
        var snippet = "";
        try {
          var snipJson = floorp.tabElementText(ddgTab, baseSel);
          snippet = JSON.parse(snipJson).text || "";
        } catch (e) {}

        searchResults.push({
          rank: searchResults.length + 1,
          title: cleanText(title),
          url: url,
          snippet: cleanText(snippet).substring(0, 300),
          domain: extractDomain(url),
          pageContent: "",
          pageTitle: "",
          extractedAt: null,
        });

        console.log(
          "  [" + searchResults.length + "] " + title.substring(0, 50) + "..."
        );
      } catch (e) {}
    }

    console.log("  ✓ Collected " + searchResults.length + " URLs");
  } catch (e) {
    console.log("  ✗ Search Error: " + e);
  } finally {
    if (ddgTab) floorp.destroyTabInstance(ddgTab);
  }

  // --- Phase 2: Visit Each Page ---
  console.log("");
  console.log("━━━ Phase 2: Deep Content Extraction ━━━");
  console.log(
    "  Visiting " + searchResults.length + " pages for detailed analysis..."
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
      // Additional wait for JavaScript-heavy sites like Floorp.app
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
          "       ✓ Extracted " + result.pageContent.length + " chars"
        );
      } else {
        failCount++;
        console.log(
          "       ⚠ Limited content (" + result.pageContent.length + " chars)"
        );
      }
    } catch (e) {
      failCount++;
      console.log("       ✗ Error: " + e);
      result.pageContent = result.snippet;
    } finally {
      if (pageTab) {
        try {
          floorp.destroyTabInstance(pageTab);
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
      progress + " Analyzing: " + result.title.substring(0, 40) + "..."
    );

    try {
      var summary = iniad_ai_mop.chat(
        "You are a research analyst. Analyze the following web page content about Floorp browser and provide: 1) Key information about Floorp, 2) The context/perspective of this source. Write in Japanese, 2-3 sentences.",
        "Page Title: " +
          result.pageTitle +
          "\n\nContent:\n" +
          result.pageContent.substring(0, 1500)
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
    analyzedResults.length
  );

  console.log("  → Overview...");
  var overviewText = generateAnalysis(
    "overview",
    searchQuery,
    allSummaries,
    analyzedResults.length
  );

  // Generate detailed Key Findings with multiple subsections
  console.log("  → Key Findings (15 detailed sections)...");

  var findingsList = [
    {
      id: "3.1",
      title: "プロジェクトの定義とビジョン",
      prompt:
        "プロジェクトの定義、設立の背景、根本的な開発思想、そして目指している最終的なビジョンについて、収集した情報源に基づいて詳細に分析してください。1000-1200語で記述してください。",
    },
    {
      id: "3.2",
      title: "技術的アーキテクチャの根幹",
      prompt:
        "ブラウザエンジンの選定理由、基盤となっている技術スタック、独自のレンダリングやスクリプト実行最適化、システムの堅牢性について、技術的側面から1000-1200語で記述してください。",
    },
    {
      id: "3.3",
      title: "コア機能と独自の実装品目",
      prompt:
        "他のブラウザには存在しない独自機能（サイドバー、多層タブ管理、ワークスペース機能、ツールバーの柔軟性等）の具体的な実装とその利便性について、1200-1500語で詳述してください。",
    },
    {
      id: "3.4",
      title: "カスタマイズ性とUIデザイン哲学",
      prompt:
        "ユーザーインターフェースのデザイン原則、CSS/JSによる高度なカスタマイズ性、テーマ機能の深さ、そしてそれがユーザーにとってどのような価値を持つかについて、1000-1200語で記述してください。",
    },
    {
      id: "3.5",
      title: "プライバシー保護とデータ主権",
      prompt:
        "トラッキング防止、データのローカル管理、匿名性の確保、他社製ブラウザ（ChromeやEdge等）とのプライバシー保護における決定的な違いについて、法規対応も含め1200-1500語で詳述してください。",
    },
    {
      id: "3.6",
      title: "セキュリティ設計と脅威対策",
      prompt:
        "脆弱性への対応体制、サンドボックス構造、フィッシングやマルウェア対策、ビルドプロセスの透明性とセキュリティについて、1000-1200語で記述してください。",
    },
    {
      id: "3.7",
      title: "パフォーマンスベンチマークと最適化",
      prompt:
        "実行速度、メモリ消費効率、リソース管理の仕組み、低スペックPCにおける動作状況など、客観的な評価指標を交えて1000-1200語で詳細に分析してください。",
    },
    {
      id: "3.8",
      title: "開発チームとガバナンス構造",
      prompt:
        "コア開発メンバーの構成、意思決定プロセス、開発ロードマップの策定方法、企業ではなくコミュニティ主導であることの利点と課題について、1000-1200語で記述してください。",
    },
    {
      id: "3.9",
      title: "オープンソースコミュニティの活動実態",
      prompt:
        "GitHub等での活動、世界中からの貢献者の関与、コミュニティによるサポート体制、ユーザーからのフィードバックがどのように製品に反映されるかについて、1000-1200語で詳述してください。",
    },
    {
      id: "3.10",
      title: "エコシステムと拡張機能の互換性",
      prompt:
        "既存の拡張機能エコシステム（Chromeストア等）との互換性、独自の拡張機能開発の可能性、アドオンによる機能拡張の柔軟性について、1000-1200語で記述してください。",
    },
    {
      id: "3.11",
      title: "ユーザー体験（UX）の多角的評価",
      prompt:
        "一般ユーザー、パワーユーザー、開発者それぞれの視点からの満足度、学習曲線の緩急、ワークフローの効率化における実働評価を、1200-1500語で詳細に分析してください。",
    },
    {
      id: "3.12",
      title: "多言語対応とグローバル展開",
      prompt:
        "日本語発のプロジェクトとしての特性、多言語化の進捗、海外ユーザーコミュニティの反応、地域ごとの普及状況とローカライズの質について、1000-1200語で記述してください。",
    },
    {
      id: "3.13",
      title: "ブラウザ市場における競合分析",
      prompt:
        "Chrome, Firefox, Safari, Edge, Vivaldi, Brave等との直接的な比較、独自のポジショニング、ニッチ層への訴求力について、市場シェアの観点も含め1200-1500語で分析してください。",
    },
    {
      id: "3.14",
      title: "直面している課題とリスク要因",
      prompt:
        "開発リソースの限界、大手ブラウザとの競争、プラットフォーム（OS）の制限、将来的な維持コストなど、プロジェクトが抱える潜在的なリスクと現在進行形の課題について、1000-1200語で真摯に記述してください。",
    },
    {
      id: "3.15",
      title: "将来の展望とロードマップ",
      prompt:
        "今後予定されている新機能、次世代ブラウザとしての役割、Webブラウザ全体の進化に与える影響、そこで果たすべきミッションについて1200-1500語で予測を交えて総括してください。",
    },
  ];

  var findingsTexts = [];
  for (var f = 0; f < findingsList.length; f++) {
    var item = findingsList[f];
    console.log("    → " + item.id + " " + item.title + "...");
    findingsTexts.push({
      id: item.id,
      title: item.title,
      content: generateDetailedFindings(
        searchQuery,
        allSummaries,
        item.id,
        item.prompt
      ),
    });
  }

  console.log("  → Discussion...");
  var discussionText = generateAnalysis(
    "discussion",
    searchQuery,
    allSummaries,
    analyzedResults.length
  );

  console.log("  → Conclusions...");
  var conclusionsText = generateAnalysis(
    "conclusions",
    searchQuery,
    allSummaries,
    analyzedResults.length
  );

  // --- Phase 4: Generate Report ---
  console.log("");
  console.log("━━━ Phase 4: Report Generation ━━━");

  var today = new Date().toISOString().split("T")[0];
  var report = "";

  // Title
  report += "# " + searchQuery + ": Comprehensive Web Analysis Report\n\n";
  report += "**Floorp Deep Research** | Generated: " + today + "\n\n";
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
  report += "7. [References](#7-references)\n\n";
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

  report += "---\n\n";
  report +=
    "*This report was automatically generated by Floorp Deep Research.*\n";
  report +=
    "*Analysis powered by AI-driven content extraction and synthesis.*\n";
  report += "*Generated: " + new Date().toISOString() + "*\n";

  // --- Phase 5: Save Report ---
  console.log("  Report size: " + report.length + " characters");
  console.log("  Saving to: " + outputPath);

  try {
    var result = fs.write(outputPath, report);
    console.log("  ✓ Report saved successfully!");
  } catch (e) {
    console.log("  ✗ Save error: " + e);
    console.log("");
    console.log("━━━ REPORT OUTPUT ━━━");
    console.log(report);
  }

  console.log("");
  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║          Floorp Deep Research Complete!                    ║");
  console.log(
    "║    Analyzed " +
      analyzedResults.length +
      " sources | Report: " +
      (report.length / 1000).toFixed(1) +
      " KB              ║"
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
    "Always cite source numbers like [1], [3], [5] when referring to specific information.";

  var fullPrompt = customPrompt + "\n\n情報源一覧:\n" + summaries;

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

workflow();
