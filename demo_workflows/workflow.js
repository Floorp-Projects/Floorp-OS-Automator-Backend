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
  var searchQuery = "Floorp";
  var maxResults = 50;
  var outputPath = "/Users/user/Desktop/Deep_Research_Report.md";

  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║           Deep Research - Comprehensive Analysis           ║");
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
            "article[data-testid='result']:last-of-type",
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
          "  [" + searchResults.length + "] " + title.substring(0, 50) + "...",
        );
      } catch (e) {}
    }

    console.log("  ✓ Collected " + searchResults.length + " URLs");
  } catch (e) {
    console.log("  ✗ Search Error: " + e);
  } finally {
    if (ddgTab) floorp.closeTab(ddgTab);
  }

  // --- Phase 1B: Local Finder Search ---
  console.log("");
  console.log("━━━ Phase 1B: Local File Search ━━━");
  console.log("  Searching local files in /Users/user for: " + searchQuery);

  var localResults = [];
  var localMaxResults = 20;

  try {
    if (
      app &&
      app.sapphillon &&
      app.sapphillon.core &&
      app.sapphillon.core.finder &&
      app.sapphillon.core.finder.findFiles
    ) {
      var finderStartTime = Date.now();
      var pathsJson = app.sapphillon.core.finder.findFiles(
        "/Users/user",
        searchQuery,
        localMaxResults,
      );
      var finderElapsed = Date.now() - finderStartTime;

      var foundPaths = [];
      try {
        foundPaths = JSON.parse(pathsJson || "[]");
      } catch (e) {
        foundPaths = [];
      }

      console.log(
        "  ✓ Found " +
          foundPaths.length +
          " local files in " +
          finderElapsed +
          "ms",
      );

      // Readable text file extensions
      var readableExtensions = [
        ".txt",
        ".md",
        ".js",
        ".ts",
        ".json",
        ".html",
        ".css",
        ".xml",
        ".yaml",
        ".yml",
        ".sh",
        ".py",
        ".rb",
        ".rs",
        ".c",
        ".cpp",
        ".h",
        ".java",
        ".go",
        ".swift",
        ".kt",
        ".toml",
        ".ini",
        ".cfg",
        ".conf",
        ".log",
        ".csv",
        ".plist",
        ".entitlements",
        ".strings",
      ];

      for (var f = 0; f < foundPaths.length; f++) {
        var filePath = foundPaths[f];
        var fileName = filePath.split("/").pop() || filePath;
        var extension = "";
        var lastDot = fileName.lastIndexOf(".");
        if (lastDot > 0) {
          extension = fileName.substring(lastDot).toLowerCase();
        }

        // Skip build artifacts, cache directories, and other non-useful paths
        var excludedPaths = [
          "/target/",
          "/node_modules/",
          "/.git/",
          "/build/",
          "/dist/",
          "/.cache/",
          "/__pycache__/",
          "/venv/",
          "/.venv/",
          "/vendor/",
          "/Pods/",
          "/.next/",
          "/out/",
          "/.nuxt/",
          "/coverage/",
          "/.nyc_output/",
          "/deps/",
          "/debug/",
          "/release/",
        ];

        var shouldSkip = false;
        for (var ep = 0; ep < excludedPaths.length; ep++) {
          if (filePath.indexOf(excludedPaths[ep]) !== -1) {
            shouldSkip = true;
            break;
          }
        }
        if (shouldSkip) {
          console.log("  [SKIP] " + fileName + " (build artifact/cache)");
          continue;
        }

        // Check if it's a readable text file
        var isReadable = false;
        for (var ext = 0; ext < readableExtensions.length; ext++) {
          if (extension === readableExtensions[ext]) {
            isReadable = true;
            break;
          }
        }

        // Only add readable text files to search results
        if (!isReadable) {
          console.log(
            "  [SKIP] " +
              fileName +
              " (unsupported extension: " +
              extension +
              ")",
          );
          continue;
        }

        var fileContent = "";
        var fileDescription = "";

        if (isReadable) {
          // Read file content
          try {
            if (
              app.sapphillon.core.filesystem &&
              app.sapphillon.core.filesystem.read
            ) {
              fileContent = app.sapphillon.core.filesystem.read(filePath) || "";
              if (fileContent.length > 3000) {
                fileContent = fileContent.substring(0, 3000);
              }
            }
          } catch (readErr) {
            fileContent = "";
          }
        }

        // Generate file description based on path and extension
        fileDescription = describeLocalFile(filePath, extension, fileContent);

        // Add to search results as a local source
        searchResults.push({
          rank: searchResults.length + 1,
          title: "[LOCAL] " + fileName,
          url: "file://" + filePath,
          snippet: fileDescription,
          domain: "local:" + extension.replace(".", ""),
          pageContent: fileContent || fileDescription,
          pageTitle: fileName,
          extractedAt: new Date().toISOString(),
          isLocalFile: true,
          filePath: filePath,
          fileExtension: extension,
        });

        localResults.push({
          path: filePath,
          name: fileName,
          extension: extension,
          isReadable: isReadable,
          contentLength: fileContent.length,
        });

        console.log(
          "  [LOCAL] " +
            fileName +
            (isReadable
              ? " (" + fileContent.length + " chars)"
              : " (folder/binary)"),
        );
      }
    } else {
      console.log("  ⚠ Finder plugin not available, skipping local search");
    }
  } catch (finderErr) {
    console.log("  ✗ Finder Error: " + finderErr);
  }

  console.log("  ✓ Added " + localResults.length + " local sources");

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

    // Skip local files - they already have content extracted
    if (result.isLocalFile) {
      console.log(
        progress +
          " [LOCAL] " +
          result.title.substring(0, 40) +
          " (already loaded)",
      );
      if (result.pageContent && result.pageContent.length > 50) {
        successCount++;
      }
      continue;
    }

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

      // Extract main content with structure preservation
      var structuredExtraction = extractStructuredContent(pageTab, searchQuery);

      // Store both full and relevant content
      result.pageContent = structuredExtraction.fullText.substring(0, 4000);
      result.relevantChunks = structuredExtraction.relevantChunks || [];
      result.headings = structuredExtraction.headings || [];

      // Keep raw paragraphs for fact-checking
      result.rawParagraphs = structuredExtraction.paragraphs || [];
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

  // Extract facts from each page (NOT summarization)
  var analyzedResults = [];
  var allRawContent = []; // Keep raw content for fact-checking

  for (var i = 0; i < searchResults.length; i++) {
    var result = searchResults[i];
    var progress = "[" + (i + 1) + "/" + searchResults.length + "]";
    console.log(
      progress + " Extracting facts: " + result.title.substring(0, 40) + "...",
    );

    try {
      // Store raw content for later verification
      allRawContent.push({
        index: i + 1,
        title: result.title,
        content: result.pageContent,
        rawParagraphs: result.rawParagraphs || [],
      });

      // NEW: Extract FACTS instead of summarizing
      var factExtractionPrompt =
        "以下のページから「" +
        searchQuery +
        "」に関する**具体的な事実**のみを抽出してください。\n\n" +
        "【抽出ルール】\n" +
        "1. 抽象的な要約は不要。具体的な情報のみ箇条書きで列挙\n" +
        "2. 数値データ（日付、価格、性能値、割合など）は必ず含める\n" +
        "3. 固有名詞（人名、製品名、企業名、技術名）を明記\n" +
        "4. 「〜とされている」「〜という」などの曖昧な表現は使わない\n" +
        "5. 情報が不明確な場合は「不明」と記載\n" +
        "6. 最大10項目まで\n\n" +
        "【出力形式】\n" +
        "- [事実1]：具体的な記述\n" +
        "- [事実2]：具体的な記述\n" +
        "...\n\n" +
        "【ページタイトル】" +
        result.pageTitle +
        "\n\n" +
        "【コンテンツ】\n" +
        result.pageContent.substring(0, 3000);

      var factList = iniad_ai_mop.chat(
        "You are a fact extractor. Extract ONLY concrete facts, numbers, names, and specific claims. " +
          "Do NOT summarize or interpret. Output in Japanese bullet points.",
        factExtractionPrompt,
      );

      // Also extract key claims for verification
      var keyClaimsPrompt =
        "上記の事実リストから、検証可能な重要な主張（数値、日付、仕様など）を3つ選んでJSON配列で出力:\n" +
        '[{"claim":"主張内容","type":"数値|日付|仕様|比較","importance":"high|medium"}]\n\n' +
        "事実リスト:\n" +
        factList;

      var keyClaims = [];
      try {
        var claimsJson = iniad_ai_mop.chat(
          "Extract 3 verifiable claims as JSON array only.",
          keyClaimsPrompt,
        );
        claimsJson = claimsJson
          .replace(/```json\n?/g, "")
          .replace(/```\n?/g, "")
          .trim();
        var jsonStart = claimsJson.indexOf("[");
        var jsonEnd = claimsJson.lastIndexOf("]") + 1;
        if (jsonStart >= 0 && jsonEnd > jsonStart) {
          keyClaims = JSON.parse(claimsJson.slice(jsonStart, jsonEnd));
        }
      } catch (claimErr) {}

      analyzedResults.push({
        result: result,
        factList: factList, // Facts instead of summary
        keyClaims: keyClaims, // Verifiable claims
        summary: factList.split("\n").slice(0, 3).join(" "), // Short summary from facts
        category: categorizeContent(result.pageContent, result),
      });

      console.log(
        "       ✓ Extracted " + (factList.match(/-/g) || []).length + " facts",
      );
    } catch (e) {
      analyzedResults.push({
        result: result,
        factList: "(抽出エラー)",
        keyClaims: [],
        summary: "(分析エラー)",
        category: result.isLocalFile ? "local" : "other",
      });
    }
  }

  // Store raw content globally for fact-checking
  var globalRawContent = allRawContent;

  // Build FACT-BASED summaries (not abstractive summaries)
  console.log("");
  console.log("  Generating fact-based analysis...");

  // Create detailed fact list for each source
  var allFactLists = analyzedResults
    .map(function (a, i) {
      return "[" + (i + 1) + "] " + a.result.title + ":\n" + a.factList;
    })
    .join("\n\n---\n\n");

  // Keep legacy summary format for backward compatibility
  var allSummaries = analyzedResults
    .map(function (a, i) {
      return "[" + (i + 1) + "] " + a.result.title + ": " + a.summary;
    })
    .join("\n\n");

  // --- Phase 3.5: Recursive Search for Missing Information ---
  console.log("");
  console.log("━━━ Phase 3.5: Gap Analysis & Recursive Search ━━━");

  var additionalResults = performRecursiveSearch(
    searchQuery,
    allFactLists,
    searchResults,
  );

  if (additionalResults.length > 0) {
    console.log(
      "  ✓ Added " +
        additionalResults.length +
        " additional sources from recursive search",
    );

    // Merge additional results
    for (var ar = 0; ar < additionalResults.length; ar++) {
      searchResults.push(additionalResults[ar].result);
      analyzedResults.push(additionalResults[ar]);
      globalRawContent.push({
        index: searchResults.length,
        title: additionalResults[ar].result.title,
        content: additionalResults[ar].result.pageContent,
        rawParagraphs: additionalResults[ar].result.rawParagraphs || [],
      });
    }

    // Rebuild fact lists with new sources
    allFactLists = analyzedResults
      .map(function (a, i) {
        return "[" + (i + 1) + "] " + a.result.title + ":\n" + a.factList;
      })
      .join("\n\n---\n\n");

    allSummaries = analyzedResults
      .map(function (a, i) {
        return "[" + (i + 1) + "] " + a.result.title + ": " + a.summary;
      })
      .join("\n\n");
  }

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
  report += "5. [Web-Local Correlation](#5-web-local-correlation)\n";
  report += "6. [Discussion](#6-discussion)\n";
  report += "7. [Conclusions](#7-conclusions)\n";
  report += "8. [References](#8-references)\n";
  report += "9. [Fact-Check Summary](#9-fact-check-summary)\n\n";
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
    local: { name: "ローカルファイル", items: [] },
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
        if (a.result.isLocalFile) {
          report += "- **パス**: `" + a.result.filePath + "`\n";
          report += "- **種類**: " + (a.result.fileExtension || "不明") + "\n";
        } else {
          report +=
            "- **URL**: [" + a.result.domain + "](" + a.result.url + ")\n";
        }
        report += "- **分析**: " + a.summary + "\n\n";
      });
    }
  });

  // Web-Local Correlation Section
  report += "## 5. Web-Local Correlation\n\n";
  report +=
    "本セクションでは、ウェブ上の情報とローカルファイルの関連性を分析し、";
  report += "両者がどのように相互補完し合うかを考察する。\n\n";

  // Separate web and local sources
  var webSources = analyzedResults.filter(function (a) {
    return !a.result.isLocalFile;
  });
  var localSources = analyzedResults.filter(function (a) {
    return a.result.isLocalFile;
  });

  if (localSources.length > 0 && webSources.length > 0) {
    // Generate correlation analysis using AI
    var correlationPrompt =
      "以下のウェブ情報源とローカルファイルの関連性を分析してください。" +
      "各ローカルファイルがウェブ情報とどう関連するか、" +
      "ローカルリソースが提供する独自の価値、" +
      "両者を組み合わせた包括的な理解について説明してください（400-500語）。\n\n" +
      "■ ウェブ情報源:\n";

    webSources.forEach(function (w, i) {
      correlationPrompt +=
        i + 1 + ". " + w.result.title + " (" + w.result.domain + ")\n";
      correlationPrompt +=
        "   要約: " + (w.summary || "").substring(0, 150) + "...\n";
    });

    correlationPrompt += "\n■ ローカルファイル:\n";
    localSources.forEach(function (l, i) {
      var fileName = l.result.filePath
        ? l.result.filePath.split("/").pop()
        : "不明";
      correlationPrompt +=
        i +
        1 +
        ". " +
        fileName +
        " (" +
        (l.result.fileType || l.result.fileExtension) +
        ")\n";
      correlationPrompt +=
        "   要約: " + (l.summary || "").substring(0, 150) + "...\n";
    });

    try {
      var correlationAnalysis = iniad_ai_mop.chat(
        "You are an expert researcher analyzing the relationship between web sources and local files. Write in Japanese.",
        correlationPrompt,
      );
      report += "### 5.1 関連性分析\n\n";
      report += correlationAnalysis + "\n\n";
    } catch (e) {
      console.log("  ⚠ Correlation analysis error: " + e);
    }

    // Summary table
    report += "### 5.2 情報源マッピング\n\n";
    report += "| ローカルファイル | 種類 | 関連ウェブソース |\n";
    report += "|-----------------|------|-----------------|\n";

    localSources.forEach(function (l) {
      var fileName = l.result.filePath
        ? l.result.filePath.split("/").pop()
        : "不明";
      var fileType = l.result.fileType || l.result.fileExtension || "不明";
      // Find related web sources by keyword matching
      var relatedWeb = [];
      webSources.forEach(function (w) {
        if (w.result.title && fileName) {
          var lowerTitle = w.result.title.toLowerCase();
          var lowerFile = fileName.toLowerCase();
          // Check for common keywords
          if (
            lowerTitle.indexOf(searchQuery.toLowerCase()) !== -1 ||
            lowerFile.indexOf(searchQuery.toLowerCase()) !== -1
          ) {
            relatedWeb.push("[" + w.result.rank + "]" + w.result.domain);
          }
        }
      });
      if (relatedWeb.length === 0) {
        relatedWeb.push("（検索トピック関連）");
      }
      report +=
        "| `" +
        fileName +
        "` | " +
        fileType +
        " | " +
        relatedWeb.slice(0, 3).join(", ") +
        " |\n";
    });
    report += "\n";

    // Value proposition
    report += "### 5.3 ローカルリソースの付加価値\n\n";
    report += "ローカルファイルは以下の点でウェブ情報を補完する：\n\n";
    report += "- **実装詳細**: ソースコードやスクリプトによる技術的な実装例\n";
    report += "- **設定情報**: 実際の運用に基づいた設定ファイルや構成\n";
    report += "- **ローカル知識**: ウェブ上にない組織固有の情報や経験\n";
    report += "- **作業履歴**: プロジェクトの進行過程やノート\n\n";
  } else if (localSources.length > 0) {
    report += "### ローカルリソースのみ\n\n";
    report +=
      "本調査ではローカルファイルのみが発見されました。" +
      "ウェブ検索結果との比較分析は利用できません。\n\n";
  } else if (webSources.length > 0) {
    report += "### ウェブ情報源のみ\n\n";
    report +=
      "本調査ではローカルファイルが発見されませんでした。" +
      "ウェブ情報源のみに基づく分析となります。\n\n";
  } else {
    report +=
      "情報源が不足しているため、関連性分析を実施できませんでした。\n\n";
  }

  // Discussion
  report += "## 6. Discussion\n\n";
  report += discussionText + "\n\n";

  // Conclusions
  report += "## 7. Conclusions\n\n";
  report += conclusionsText + "\n\n";

  // References
  report += "---\n\n";
  report += "## 8. References\n\n";
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

  // Fact-Check Section (now using raw content for verification)
  console.log("");
  console.log("━━━ Phase 5: Fact-Checking (with Raw Data) ━━━");
  var factCheckSection = generateFactCheckReport(
    findingsTexts,
    allSummaries,
    globalRawContent,
  );
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

function categorizeContent(content, result) {
  // Check if it's a local file first
  if (result && result.isLocalFile) {
    return "local";
  }

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

// Enhanced: Extract structured content preserving important elements
function extractStructuredContent(tab, query) {
  var result = {
    fullText: "",
    headings: [],
    lists: [],
    paragraphs: [],
    relevantChunks: [],
  };

  try {
    // Extract headings (H1-H3)
    var headingSelectors = ["h1", "h2", "h3"];
    for (var h = 0; h < headingSelectors.length; h++) {
      try {
        var headingJson = floorp.tabElementText(tab, headingSelectors[h]);
        var headingText = JSON.parse(headingJson).text || "";
        if (headingText.length > 2) {
          result.headings.push({
            level: h + 1,
            text: cleanText(headingText),
          });
        }
      } catch (e) {}
    }

    // Extract main content with structure hints
    var contentSelectors = [
      "article",
      "main",
      "[role='main']",
      ".content",
      ".post-content",
      ".entry-content",
      ".article-body",
      "#content",
    ];

    var mainContent = "";
    for (var s = 0; s < contentSelectors.length; s++) {
      if (mainContent.length > 1000) break;
      try {
        var contentJson = floorp.tabElementText(tab, contentSelectors[s]);
        var content = JSON.parse(contentJson).text || "";
        if (content.length > mainContent.length) {
          mainContent = content;
        }
      } catch (e) {}
    }

    // Fallback to body
    if (mainContent.length < 200) {
      try {
        var bodyJson = floorp.tabElementText(tab, "body");
        mainContent = JSON.parse(bodyJson).text || "";
      } catch (e) {}
    }

    // Split into paragraphs preserving structure
    var rawParagraphs = mainContent.split(/\n\n+|。(?=\s)|\. (?=[A-Z])/);
    result.paragraphs = rawParagraphs
      .map(function (p) {
        return cleanText(p);
      })
      .filter(function (p) {
        return p.length > 30;
      });

    // Smart chunk selection: find query-relevant sections
    var queryKeywords = query.toLowerCase().split(/\s+/);
    var scoredParagraphs = result.paragraphs.map(function (p, idx) {
      var lowerP = p.toLowerCase();
      var score = 0;
      queryKeywords.forEach(function (kw) {
        if (kw.length > 1 && lowerP.indexOf(kw) >= 0) {
          score += 2;
        }
      });
      // Boost paragraphs with numbers/data
      if (/\d+/.test(p)) score += 1;
      // Boost paragraphs with comparisons
      if (/より|比べ|against|than|versus/i.test(p)) score += 1;
      return { text: p, index: idx, score: score };
    });

    // Sort by score and select top chunks
    scoredParagraphs.sort(function (a, b) {
      return b.score - a.score;
    });

    // Get top scoring paragraphs plus context
    var selectedIndices = {};
    for (var i = 0; i < Math.min(10, scoredParagraphs.length); i++) {
      var idx = scoredParagraphs[i].index;
      // Include surrounding context
      for (var offset = -1; offset <= 1; offset++) {
        var contextIdx = idx + offset;
        if (contextIdx >= 0 && contextIdx < result.paragraphs.length) {
          selectedIndices[contextIdx] = true;
        }
      }
    }

    // Build relevant chunks in order
    var orderedIndices = Object.keys(selectedIndices)
      .map(Number)
      .sort(function (a, b) {
        return a - b;
      });
    result.relevantChunks = orderedIndices.map(function (idx) {
      return result.paragraphs[idx];
    });

    // Build structured full text
    var structuredText = "";
    if (result.headings.length > 0) {
      structuredText += "【見出し】\n";
      result.headings.forEach(function (h) {
        structuredText += "  " + h.text + "\n";
      });
      structuredText += "\n";
    }
    structuredText += "【本文（関連部分）】\n";
    structuredText += result.relevantChunks.join("\n\n");

    result.fullText = structuredText;
  } catch (e) {
    console.log("       ⚠ Structured extraction error: " + e);
    result.fullText = "";
  }

  return result;
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
// Recursive Search Function - Deep Research Enhancement
// ============================================================================

// Analyze gaps in current information and perform additional targeted searches
function performRecursiveSearch(originalQuery, currentFacts, existingResults) {
  var additionalResults = [];
  var maxAdditionalSearches = 2;
  var maxResultsPerSearch = 5;

  console.log("  Analyzing information gaps...");

  // Use LLM to identify what information is missing
  var gapAnalysisPrompt =
    "あなたは調査分析の専門家です。「" +
    originalQuery +
    "」について収集した情報を分析し、" +
    "不足している重要な情報を特定してください。\n\n" +
    "【現在収集済みの情報】\n" +
    currentFacts.substring(0, 4000) +
    "\n\n" +
    "【タスク】\n" +
    "上記の情報を分析し、包括的なレポートを作成するために不足している観点を特定してください。\n" +
    "具体的な追加検索クエリを2つ提案してください。\n\n" +
    "【出力形式】JSON配列のみ:\n" +
    '[{"gap":"不足している情報の説明","query":"追加検索クエリ","priority":"high|medium"}]';

  var gapsToFill = [];

  try {
    var gapResponse = iniad_ai_mop.chat(
      "Identify information gaps and suggest search queries. Output JSON array only.",
      gapAnalysisPrompt,
    );

    gapResponse = gapResponse
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();
    var jsonStart = gapResponse.indexOf("[");
    var jsonEnd = gapResponse.lastIndexOf("]") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      gapsToFill = JSON.parse(gapResponse.slice(jsonStart, jsonEnd));
    }

    console.log("  ✓ Identified " + gapsToFill.length + " information gaps");
  } catch (e) {
    console.log("  ⚠ Gap analysis failed: " + e.message);
    return additionalResults;
  }

  // Filter to high priority gaps only
  gapsToFill = gapsToFill
    .filter(function (g) {
      return g.priority === "high" || g.priority === "medium";
    })
    .slice(0, maxAdditionalSearches);

  if (gapsToFill.length === 0) {
    console.log(
      "  ✓ No significant gaps identified - information is comprehensive",
    );
    return additionalResults;
  }

  // Collect existing URLs to avoid duplicates
  var existingUrls = {};
  existingResults.forEach(function (r) {
    existingUrls[r.url] = true;
  });

  // Perform additional targeted searches
  for (var g = 0; g < gapsToFill.length; g++) {
    var gap = gapsToFill[g];
    console.log("  → Searching for: " + gap.query);

    var ddgTab = null;
    try {
      var ddgUrl = "https://duckduckgo.com/?q=" + encodeURIComponent(gap.query);
      ddgTab = floorp.createTab(ddgUrl, false);
      floorp.tabWaitForElement(ddgTab, "article[data-testid='result']", 15000);
      sleep(2000);

      var foundCount = 0;
      for (var i = 1; i <= 15 && foundCount < maxResultsPerSearch; i++) {
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

          // Skip duplicates and social media
          if (existingUrls[url]) continue;
          if (
            url.includes("youtube.com") ||
            url.includes("twitter.com") ||
            url.includes("facebook.com") ||
            url.includes("instagram.com")
          )
            continue;

          existingUrls[url] = true;
          foundCount++;

          // Visit page and extract content
          var pageTab = null;
          try {
            pageTab = floorp.createTab(url, false);
            try {
              floorp.tabWaitForNetworkIdle(pageTab, 8000);
            } catch (e) {}
            sleep(3000);

            var pageTitle = title;
            try {
              var titleJson = floorp.tabElementText(pageTab, "title");
              pageTitle = cleanText(JSON.parse(titleJson).text || title);
            } catch (e) {}

            // Use structured extraction
            var structuredExtraction = extractStructuredContent(
              pageTab,
              originalQuery,
            );

            var newResult = {
              rank: existingResults.length + additionalResults.length + 1,
              title: "[追加] " + cleanText(title),
              url: url,
              snippet: gap.gap,
              domain: extractDomain(url),
              pageContent: structuredExtraction.fullText.substring(0, 4000),
              pageTitle: pageTitle,
              extractedAt: new Date().toISOString(),
              isRecursiveResult: true,
              gapFilled: gap.gap,
              relevantChunks: structuredExtraction.relevantChunks || [],
              headings: structuredExtraction.headings || [],
              rawParagraphs: structuredExtraction.paragraphs || [],
            };

            // Extract facts from this new page
            var factExtractionPrompt =
              "以下のページから「" +
              originalQuery +
              "」に関する**具体的な事実**のみを抽出してください。\n" +
              "特に「" +
              gap.gap +
              "」に関連する情報を重点的に抽出してください。\n\n" +
              "【抽出ルール】\n" +
              "1. 抽象的な要約は不要。具体的な情報のみ箇条書きで列挙\n" +
              "2. 数値データは必ず含める\n" +
              "3. 最大8項目まで\n\n" +
              "【コンテンツ】\n" +
              newResult.pageContent.substring(0, 2500);

            var factList = "(抽出エラー)";
            try {
              factList = iniad_ai_mop.chat(
                "You are a fact extractor. Extract ONLY concrete facts. Output in Japanese bullet points.",
                factExtractionPrompt,
              );
            } catch (fe) {}

            additionalResults.push({
              result: newResult,
              factList: factList,
              keyClaims: [],
              summary: factList.split("\n").slice(0, 2).join(" "),
              category: categorizeContent(newResult.pageContent, newResult),
            });

            console.log("    ✓ Added: " + title.substring(0, 40) + "...");
          } catch (pageErr) {
            console.log("    ⚠ Page error: " + pageErr);
          } finally {
            if (pageTab) {
              try {
                floorp.closeTab(pageTab);
                floorp.destroyTabInstance(pageTab);
                console.log("       - Page tab destroyed");
              } catch (e) {}
            }
          }
        } catch (e) {}
      }
    } catch (searchErr) {
      console.log("    ⚠ Search error: " + searchErr);
    } finally {
      if (ddgTab) {
        try {
          floorp.closeTab(ddgTab);
          floorp.destroyTabInstance(ddgTab);
          console.log("    - Search tab destroyed");
        } catch (e) {}
      }
    }
  }

  return additionalResults;
}

// ============================================================================
// Local File Description Function
// ============================================================================

// Generate a description of a local file based on its path, extension, and content
function describeLocalFile(filePath, extension, content) {
  var fileName = filePath.split("/").pop() || filePath;
  var pathParts = filePath.split("/");
  var parentFolder =
    pathParts.length > 1 ? pathParts[pathParts.length - 2] : "";

  // Determine file type description
  var fileTypeDescriptions = {
    ".js": "JavaScript ソースコード",
    ".ts": "TypeScript ソースコード",
    ".json": "JSON データファイル",
    ".md": "Markdown ドキュメント",
    ".txt": "テキストファイル",
    ".html": "HTML ウェブページ",
    ".css": "CSS スタイルシート",
    ".py": "Python スクリプト",
    ".rs": "Rust ソースコード",
    ".swift": "Swift ソースコード",
    ".kt": "Kotlin ソースコード",
    ".java": "Java ソースコード",
    ".c": "C ソースコード",
    ".cpp": "C++ ソースコード",
    ".h": "C/C++ ヘッダファイル",
    ".go": "Go ソースコード",
    ".rb": "Ruby スクリプト",
    ".sh": "シェルスクリプト",
    ".yaml": "YAML 設定ファイル",
    ".yml": "YAML 設定ファイル",
    ".toml": "TOML 設定ファイル",
    ".xml": "XML データファイル",
    ".plist": "macOS プロパティリスト",
    ".entitlements": "macOS エンタイトルメント設定",
    ".log": "ログファイル",
    ".csv": "CSV データファイル",
    ".ini": "INI 設定ファイル",
    ".cfg": "設定ファイル",
    ".conf": "設定ファイル",
    ".strings": "ローカライズ文字列ファイル",
  };

  var typeDesc = fileTypeDescriptions[extension] || "ファイル";

  // Build description
  var description = "ローカル" + typeDesc + ": " + fileName;

  // Add path context
  if (parentFolder) {
    description += " (場所: " + parentFolder + "/)";
  }

  // Add content summary if available
  if (content && content.length > 0) {
    var contentPreview = content.substring(0, 200).replace(/\s+/g, " ").trim();
    if (content.length > 200) {
      contentPreview += "...";
    }
    description += " | 内容プレビュー: " + contentPreview;
  }

  // Detect project type from path
  if (filePath.indexOf("/node_modules/") > -1) {
    description += " [Node.js 依存関係]";
  } else if (filePath.indexOf("/.git/") > -1) {
    description += " [Git リポジトリ]";
  } else if (filePath.indexOf("/src/") > -1) {
    description += " [ソースコード]";
  } else if (
    filePath.indexOf("/docs/") > -1 ||
    filePath.indexOf("/doc/") > -1
  ) {
    description += " [ドキュメント]";
  } else if (
    filePath.indexOf("/test/") > -1 ||
    filePath.indexOf("/tests/") > -1
  ) {
    description += " [テストコード]";
  } else if (filePath.indexOf("/config/") > -1) {
    description += " [設定]";
  }

  return description;
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

// Use LLM to cross-verify important claims with RAW DATA (not summaries)
function verifyClaimsWithLLM(content, sourceSummaries, rawContentArray) {
  var systemPrompt =
    "あなたは厳密なファクトチェッカーです。AIが生成したコンテンツを**元の生データ**と比較し、事実の正確性を検証します。\n\n" +
    "【検証カテゴリ】\n" +
    "1. 数値データ: 数量、価格、割合、サイズ、時間、距離など\n" +
    "2. 日付情報: 発売日、設立日、イベント日時、期間など\n" +
    "3. 仕様・スペック: 技術仕様、バージョン、規格、型番など\n" +
    "4. 機能・特徴: 製品・サービスの機能、特性、性能など\n" +
    "5. 比較表現:「〜倍」「〜%向上」「最大〜」「業界初」等の定量的比較\n\n" +
    "【検証基準】\n" +
    "- verified: 情報源の**生データ**に明確に記載されており、数値も一致\n" +
    "- partially_verified: 生データに類似の記述があるが、数値や詳細が異なる\n" +
    "- unverified: 生データに該当する記述が見つからない\n" +
    "- fabricated: 生データと明らかに矛盾、または存在しない情報\n\n" +
    "【ハルシネーションの兆候】\n" +
    "- 過度に具体的な数値（小数点以下まで、または非公開の内部情報）\n" +
    "- 情報源に存在しない固有名詞や専門用語\n" +
    "- 将来の予測を事実として記述\n" +
    "- 曖昧な引用（「〜という報告がある」「〜とされている」）\n\n" +
    "【出力形式】\n" +
    "必ず以下のJSON配列のみを出力してください（説明文不要）:\n" +
    '[{"claim":"検証対象の主張","category":"数値|日付|仕様|機能|比較","status":"verified|partially_verified|unverified|fabricated","confidence":1-5,"source_ref":"該当する情報源番号（例:[1][3]）またはnull","reason":"20字以内の根拠","raw_evidence":"生データからの引用（30字以内）"}]';

  // Build raw content reference (prioritize over summaries)
  var rawReference = "";
  if (rawContentArray && rawContentArray.length > 0) {
    rawReference = "【生データ参照】\n";
    for (var r = 0; r < Math.min(5, rawContentArray.length); r++) {
      var rawItem = rawContentArray[r];
      rawReference += "[" + rawItem.index + "] " + rawItem.title + ":\n";
      rawReference += (rawItem.content || "").substring(0, 800) + "\n---\n";
    }
  }

  var checkPrompt =
    "【タスク】以下の生成コンテンツに含まれる事実的主張を、**生データ**と照合して検証してください。\n\n" +
    "【重要】要約ではなく、元のページから抽出した生テキストを参照して検証すること。\n\n" +
    "【検証手順】\n" +
    "1. 生成コンテンツから具体的な数値・日付・仕様を抽出\n" +
    "2. 各主張を**生データ**の該当箇所と直接比較\n" +
    "3. 一致度を評価し、ステータスを決定\n" +
    "4. 特にハルシネーション（捏造）の可能性が高いものを重点的にチェック\n\n" +
    "【生成コンテンツ】\n" +
    content.slice(0, 2000) +
    "\n\n" +
    rawReference +
    "\n\n【情報源要約（参考）】\n" +
    sourceSummaries.slice(0, 2000) +
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

// Generate fact-check summary report section (now accepts raw content)
function generateFactCheckReport(
  findingsTexts,
  sourceSummaries,
  rawContentArray,
) {
  console.log("  → Fact-checking generated content against RAW DATA...");

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

  // LLM-based deep verification using RAW DATA (sample up to 3 sections)
  var llmResults = [];
  for (var i = 0; i < Math.min(3, findingsTexts.length); i++) {
    var sectionResults = verifyClaimsWithLLM(
      findingsTexts[i].content,
      sourceSummaries,
      rawContentArray, // Pass raw content for verification
    );
    llmResults = llmResults.concat(sectionResults);
  }
  console.log(
    "    → LLM verification (with raw data): " +
      llmResults.length +
      " claims analyzed",
  );

  // Generate report section
  var report = "## 9. Fact-Check Summary\n\n";
  report += "> **検証結果概要**\n>\n";
  report += "> 本レポートの内容を情報源と照合し、事実確認を実施しました。\n\n";

  report += "### 9.1 パターンベース検証\n\n";
  report += "| 項目 | 数値 |\n";
  report += "|------|------|\n";
  report += "| 抽出された主張数 | " + basicResults.total + " |\n";
  report += "| 情報源で確認 | " + basicResults.verified + " |\n";
  report += "| 未確認 | " + basicResults.unverified + " |\n";
  report += "| 高信頼度 | " + basicResults.highConfidence + " |\n";
  report += "| 中信頼度 | " + basicResults.mediumConfidence + " |\n";
  report += "| 要確認 | " + basicResults.lowConfidence + " |\n\n";

  if (llmResults.length > 0) {
    report += "### 9.2 AI検証結果\n\n";
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

workflow();
