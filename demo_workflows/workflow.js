/**
 * Deep Research - Comprehensive Web Analysis
 *
 * ENHANCED FEATURES:
 * 1. Query Expansion - Multi-angle search with synonyms and related terms
 * 2. Source Reliability Scoring - Trustworthiness evaluation
 * 3. Cross-Reference Verification - Fact validation across sources
 * 4. Academic Citation Format - Professional referencing
 * 5. Insight Generation - Pattern recognition and predictions
 * 6. Interactive Exploration - User-guided investigation
 * 7. Metadata Extraction - Authority and social signals
 * 8. Knowledge Graph - Relationship mapping
 * 9. Quality Assurance - Multi-dimensional quality metrics
 *
 * PIPELINE:
 * 1. DuckDuckGo Search (50 results) with Query Expansion
 * 2. AI relevance filtering + Source Reliability Scoring
 * 3. Visit each relevant URL to extract detailed content + Metadata
 * 4. Analyze and synthesize with LLM
 * 5. Cross-Reference Verification + Insight Extraction
 * 6. Generate comprehensive academic-style report with fact-checking
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

  // --- Phase 2.5: Temporal Analysis ---
  console.log("");
  console.log("━━━ Phase 2.5: Temporal Analysis ━━━");

  var temporalTrends = analyzeTemporalTrends(searchResults);
  searchResults = filterByFreshness(searchResults, 365);

  console.log("  ✓ Temporal analysis complete");

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

  // --- Phase 3.2: Contradiction Detection ---
  console.log("");
  console.log("━━━ Phase 3.2: Contradiction Detection ━━━");

  var contradictions = detectContradictions(analyzedResults);

  if (contradictions.length > 0) {
    console.log("  ⚠ Detected " + contradictions.length + " contradictions");
    var highCount = contradictions.filter(function (c) {
      return c.severity === "high";
    }).length;
    var mediumCount = contradictions.filter(function (c) {
      return c.severity === "medium";
    }).length;
    console.log("    - High severity: " + highCount);
    console.log("    - Medium severity: " + mediumCount);
  } else {
    console.log("  ✓ No contradictions detected");
  }

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

  // --- Phase 3.6: Source Reliability Scoring ---
  console.log("");
  console.log("━━━ Phase 3.6: Source Reliability Scoring ━━━");

  for (var i = 0; i < analyzedResults.length; i++) {
    var reliability = calculateSourceReliability(analyzedResults[i].result);
    analyzedResults[i].reliability = reliability;

    var scoreIcon = reliability.score >= 8 ? "🟢" : reliability.score >= 6 ? "🟡" : "🔴";
    console.log(
      "    " + scoreIcon + " [" + (i + 1) + "] " +
      analyzedResults[i].result.title.substring(0, 40) + "... " +
      reliability.score.toFixed(1) + "/10 (" + reliability.level + ")"
    );
  }

  // --- Phase 3.7: Cross-Reference Verification ---
  console.log("");
  console.log("━━━ Phase 3.7: Cross-Reference Verification ━━━");

  var crossRefResults = verifyCrossSource(analyzedResults, 2);
  analyzedResults.forEach(function (a) {
    a.crossRefScore = 0;
  });

  // Mark sources with verified facts
  crossRefResults.verified.forEach(function (vf) {
    vf.sources.forEach(function (sourceIdx) {
      if (analyzedResults[sourceIdx]) {
        analyzedResults[sourceIdx].crossRefScore += vf.sourceCount;
      }
    });
  });

  // --- Phase 3.8: Metadata Extraction ---
  console.log("");
  console.log("━━━ Phase 3.8: Rich Metadata Extraction ━━━");

  for (var i = 0; i < Math.min(5, analyzedResults.length); i++) {
    if (!analyzedResults[i].result.isLocalFile) {
      console.log("    Extracting metadata for source " + (i + 1) + "...");
      var metadata = extractRichMetadata(analyzedResults[i].result, null);
      analyzedResults[i].metadata = metadata;
      console.log(
        "      Authority: " + metadata.authority.toFixed(1) + "/10" +
        (metadata.author ? " | Author: " + metadata.author : "")
      );
    }
  }

  // --- Phase 3.9: Knowledge Graph Construction ---
  console.log("");
  console.log("━━━ Phase 3.9: Knowledge Graph Construction ━━━");

  var knowledgeGraph = buildKnowledgeGraph(analyzedResults);

  // --- Phase 3.10: Insight Generation ---
  console.log("");
  console.log("━━━ Phase 3.10: Insight Generation ━━━");

  var insightsText = extractInsights(analyzedResults, searchQuery);

  // --- Phase 3.11: Interactive Exploration Setup ---
  console.log("");
  console.log("━━━ Phase 3.11: Interactive Exploration Setup ━━━");

  var interactiveQuestions = performInteractiveExploration(analyzedResults, searchQuery);

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
  report += "8. [References](#11-references)\n";
  report += "9. [Fact-Check Summary](#12-fact-check-summary)\n";
  report +=
    "10. [Temporal & Contradiction Analysis](#13-temporal--contradiction-analysis)\n";
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
    "検索 → URL収集 → ページ訪問 → コンテンツ抽出 → 時系列分析 → AI分析 → 矛盾検出 → 要約生成 → レポート作成\n";
  report += "```\n\n";

  report += "### 2.3 Statistics\n\n";
  report += "| 項目 | 数値 |\n";
  report += "|------|-----|\n";
  report += "| 検索結果取得数 | " + searchResults.length + " |\n";
  report += "| コンテンツ抽出成功 | " + successCount + " |\n";
  report += "| 抽出制限/失敗 | " + failCount + " |\n";
  report += "| 分析完了 | " + analyzedResults.length + " |\n";
  report += "| 検出された矛盾 | " + contradictions.length + " |\n\n";

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


  // Cross-Reference Verification
  report += "---\n\n";
  report += "## 8. Cross-Reference Verification\n\n";
  report += "> **クロスリファレンス検証**\n>\n";
  report += "> 複数の情報源で同じ事実が確認されているかを検証しました。\n\n";

  report += "### 8.1 検証済みの事実\n\n";
  report += "以下の事実は2つ以上の情報源で確認されています：\n\n";
  report += "| 事実 | 確認数 | 信頼度 | 情報源 |\n";
  report += "|------|--------|--------|---------|\n";

  crossRefResults.verified.forEach(function (vf) {
    var confidenceBar = "";
    for (var c = 0; c < 5; c++) {
      confidenceBar += c < Math.floor(vf.confidence * 5) ? "●" : "○";
    }
    var sourcesStr = vf.sources.map(function (s) { return "[" + (s + 1) + "]"; }).join(", ");
    report +=
      "| " + vf.fact.substring(0, 30) + " | " +
      vf.sourceCount + " | " + confidenceBar + " | " +
      sourcesStr + " |\n";
  });

  report += "\n";
  report += "### 8.2 未検証の事実\n\n";
  report += "以下の事実は単一の情報源からのみ確認されています（追加検証推奨）：\n\n";
  var unverifiedToShow = crossRefResults.unverified.slice(0, 10);
  unverifiedToShow.forEach(function (uf) {
    var sourcesStr = uf.sources.map(function (s) { return "[" + (s + 1) + "]"; }).join(", ");
    report += "- " + uf.fact + " (情報源: " + sourcesStr + ")\n";
  });
  if (crossRefResults.unverified.length > 10) {
    report += "  ... 他 " + (crossRefResults.unverified.length - 10) + "件\n";
  }
  report += "\n";

  // Insights & Patterns
  report += "---\n\n";
  report += "## 9. Insights & Patterns\n\n";
  report += insightsText + "\n\n";

  // Knowledge Graph
  report += "---\n\n";
  report += generateKnowledgeGraphReport(knowledgeGraph);


  // References
  report += "---\n\n";
  report += "## 11. References\n\n";
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
  console.log(
    "  Note: Temporal analysis and contradiction detection were performed in Phases 2.5 and 3.2",
  );
  var factCheckSection = generateFactCheckReport(
    findingsTexts,
    allSummaries,
    globalRawContent,
  );
  report += factCheckSection;

  report += "---\n\n";
  report += "## 13. Temporal & Contradiction Analysis\n\n";
  report += generateTemporalReport(temporalTrends, searchResults);
  report += generateContradictionReport(contradictions, analyzedResults);
  report += "---\n\n";


  report += "---\n\n";
  report += "## 14. Quality Assurance\n\n";

  // Perform quality check
  console.log("");
  console.log("━━━ Phase 6: Quality Assurance ━━━");

  var qaResults = performQualityCheck(analyzedResults, report, searchResults);
  var qaReport = generateQualityReport(qaResults);
  report += qaReport;

  report += "---\n\n";  report += "*This report was automatically generated by Deep Research.*\n";
  report +=
    "*Analysis powered by AI-driven content extraction and synthesis.*\n";
  report +=
    "*Fact-checking enabled: Claims verified against source documents.*\n";
  report += "*Temporal analysis: Information freshness and trends analyzed.*\n";
  report += "*Contradiction detection: Cross-source verification performed.*\n";
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
  console.log(
    "  → Temporal analysis and contradiction detection completed in Phase 10",
  );

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
  var report = "## 12. Fact-Check Summary\n\n";
  report += "> **検証結果概要**\n>\n";
  report += "> 本レポートの内容を情報源と照合し、事実確認を実施しました。\n";
  report += "> また、セクション10で時系列分析と矛盾検出も実施しています。\n\n";
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
    report += "### 9.3 要確認事項\n\n";
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

// ============================================================================
// Temporal Analysis Functions
// ============================================================================

// Extract date from URL or content
function extractDateFromContent(content, url) {
  // Try to extract from URL first (most reliable)
  var urlPatterns = [
    /\/(\d{4})\/(\d{2})\/(\d{2})\//, // /2024/01/15/
    /\/(\d{4})-(\d{2})-(\d{2})\//, // /2024-01-15/
    /\/(\d{4})(\d{2})(\d{2})\//, // /20240115/
    /(\d{4})\/(\d{2})\/(\d{2})/, // 2024/01/15
    /(\d{4})-(\d{2})-(\d{2})/, // 2024-01-15
  ];

  for (var i = 0; i < urlPatterns.length; i++) {
    var match = url.match(urlPatterns[i]);
    if (match) {
      return new Date(
        parseInt(match[1]),
        parseInt(match[2]) - 1,
        parseInt(match[3]),
      );
    }
  }

  // Try to extract from content
  if (!content) return null;

  var datePatterns = [
    /(\d{4})年(\d{1,2})月(\d{1,2})日/, // 2024年1月15日
    /(\d{4})\/(\d{1,2})\/(\d{1,2})/, // 2024/1/15
    /(\d{4})-(\d{1,2})-(\d{1,2})/, // 2024-1-15
    /(\d{1,2})月(\d{1,2})日.*?(\d{4})年/, // 1月15日 2024年
    /(\d{4})\s*年\s*(\d{1,2})\s*月\s*(\d{1,2})\s*日/, // 2024 年 1 月 15 日
  ];

  for (var i = 0; i < datePatterns.length; i++) {
    var matches = content.match(new RegExp(datePatterns[i].source, "g"));
    if (matches && matches.length > 0) {
      var match = matches[0].match(datePatterns[i]);
      if (match) {
        var year, month, day;
        if (match.length >= 4) {
          year = parseInt(match[1]);
          month = parseInt(match[2]);
          day = parseInt(match[3]);
        } else {
          // Handle patterns where year might be last
          year = parseInt(match[3]);
          month = parseInt(match[1]);
          day = parseInt(match[2]);
        }
        if (
          year > 1900 &&
          year < 2100 &&
          month >= 1 &&
          month <= 12 &&
          day >= 1 &&
          day <= 31
        ) {
          return new Date(year, month - 1, day);
        }
      }
    }
  }

  return null;
}

// Calculate freshness score based on content indicators
function calculateFreshnessScore(content, url) {
  var score = 0;
  var lowerContent = (content || "").toLowerCase();
  var lowerUrl = (url || "").toLowerCase();

  // Recent indicators in content
  var recentKeywords = [
    "最新",
    "2024",
    "2025",
    "recent",
    "latest",
    "new",
    "updated",
    "アップデート",
    "更新",
    "新機能",
    "new feature",
    "recently",
  ];

  for (var i = 0; i < recentKeywords.length; i++) {
    if (lowerContent.indexOf(recentKeywords[i]) >= 0) {
      score += 0.2;
    }
  }

  // URL indicators
  if (lowerUrl.match(/\/202[4-5]\//)) score += 0.5;
  if (lowerUrl.match(/\/archive\//)) score -= 0.3;
  if (lowerUrl.match(/\/old\//)) score -= 0.3;

  return Math.min(Math.max(score, 0), 1);
}

// Filter results by freshness
function filterByFreshness(results, daysThreshold) {
  if (!daysThreshold) daysThreshold = 365;
  var cutoff = new Date();
  cutoff.setDate(cutoff.getDate() - daysThreshold);

  var filtered = [];
  var oldCount = 0;
  var recentCount = 0;

  for (var i = 0; i < results.length; i++) {
    var result = results[i];
    var date = extractDateFromContent(result.pageContent, result.url);
    var freshnessScore = calculateFreshnessScore(
      result.pageContent,
      result.url,
    );

    result.extractedDate = date ? date.toISOString().split("T")[0] : null;
    result.freshnessScore = freshnessScore;
    result.isRecent = date ? date >= cutoff : freshnessScore > 0.5;

    if (result.isRecent) {
      recentCount++;
    } else {
      oldCount++;
    }

    filtered.push(result);
  }

  console.log(
    "  時系列分析: 最新 " + recentCount + "件 | 過去 " + oldCount + "件",
  );

  return filtered;
}

// Analyze temporal trends in the results
function analyzeTemporalTrends(results) {
  var trends = {
    byYear: {},
    byMonth: {},
    recentTrend: "stable",
    oldestDate: null,
    newestDate: null,
  };

  var dates = [];
  for (var i = 0; i < results.length; i++) {
    var date = extractDateFromContent(results[i].pageContent, results[i].url);
    if (date) {
      dates.push(date);
      var year = date.getFullYear();
      var month = date.getMonth();

      if (!trends.byYear[year]) trends.byYear[year] = 0;
      trends.byYear[year]++;

      var monthKey = year + "-" + (month + 1);
      if (!trends.byMonth[monthKey]) trends.byMonth[monthKey] = 0;
      trends.byMonth[monthKey]++;
    }
  }

  if (dates.length > 0) {
    dates.sort(function (a, b) {
      return a - b;
    });
    trends.oldestDate = dates[0].toISOString().split("T")[0];
    trends.newestDate = dates[dates.length - 1].toISOString().split("T")[0];

    // Determine trend
    if (dates.length >= 3) {
      var recent = dates.slice(-3);
      var older = dates.slice(-6, -3);
      var recentAvg =
        recent.reduce(function (sum, d) {
          return sum + d.getTime();
        }, 0) / recent.length;
      var olderAvg =
        older.reduce(function (sum, d) {
          return sum + d.getTime();
        }, 0) / older.length;

      if (recentAvg > olderAvg + 30 * 24 * 60 * 60 * 1000) {
        trends.recentTrend = "increasing";
      } else if (recentAvg < olderAvg - 30 * 24 * 60 * 60 * 1000) {
        trends.recentTrend = "decreasing";
      }
    }
  }

  return trends;
}

// ============================================================================
// Contradiction Detection Functions
// ============================================================================

// Extract structured facts from fact list
function extractStructuredFacts(factList) {
  var facts = [];
  if (!factList) return facts;

  var lines = factList.split("\n");
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i].trim();
    if (line.length > 5) {
      // Extract numerical values
      var numberMatch = line.match(
        /(\d+(?:\.\d+)?)(?:\s*(?:%|倍|倍|時間|時間|円|ドル|\$|GB|MB|TB|mm|g|kg))/,
      );
      var value = numberMatch ? numberMatch[1] : null;

      // Extract dates
      var dateMatch = line.match(/(\d{4})年(\d{1,2})月(\d{1,2})日/);
      var date = dateMatch ? dateMatch[0] : null;

      // Extract key phrases
      var keyPhrase = line
        .replace(/^- /, "")
        .replace(/：.*$/, "")
        .replace(/:.*$/, "")
        .trim();

      facts.push({
        original: line,
        keyPhrase: keyPhrase,
        value: value,
        date: date,
        normalized: normalizeFactForComparison(line),
      });
    }
  }

  return facts;
}

// Normalize fact for comparison
function normalizeFactForComparison(fact) {
  var normalized = fact
    // Remove bullet points
    .replace(/^[-*•]\s*/, "")
    // Normalize dates
    .replace(/\d{4}年\d{1,2}月\d{1,2}日/g, "DATE")
    .replace(/\d{4}\/\d{1,2}\/\d{1,2}/g, "DATE")
    .replace(/\d{4}-\d{1,2}-\d{1,2}/g, "DATE")
    // Normalize percentages
    .replace(/\d+%/g, "PERCENTAGE")
    .replace(/\d+(?:\.\d+)?\s*%/g, "PERCENTAGE")
    // Normalize multipliers
    .replace(/\d+(?:\.\d+)?\s*倍/g, "MULTIPLIER")
    .replace(/\d+(?:\.\d+)?\s*x/g, "MULTIPLIER")
    // Normalize prices
    .replace(/(?:\$|¥|円)\s*\d+(?:,\d{3})*/g, "PRICE")
    // Normalize measurements
    .replace(
      /\d+(?:\.\d+)?\s*(?:GB|MB|TB|mm|g|kg|時間|hours?|hrs?)/g,
      "MEASUREMENT",
    )
    // Remove extra whitespace
    .replace(/\s+/g, " ")
    .trim();

  return normalized.toLowerCase();
}

// Detect contradictions between sources
function detectContradictions(analyzedResults) {
  var contradictions = [];
  var factMap = {};

  for (var i = 0; i < analyzedResults.length; i++) {
    var result = analyzedResults[i];
    var facts = extractStructuredFacts(result.factList);

    for (var j = 0; j < facts.length; j++) {
      var fact = facts[j];
      var key = fact.normalized;

      if (factMap[key]) {
        var existing = factMap[key];

        // Check for value contradictions
        if (fact.value && existing.value && fact.value !== existing.value) {
          contradictions.push({
            type: "value_contradiction",
            fact: fact.keyPhrase,
            sourceA: existing.sourceIndex,
            sourceB: i,
            valueA: existing.value,
            valueB: fact.value,
            severity: calculateContradictionSeverity(
              existing.value,
              fact.value,
            ),
          });
        }

        // Check for date contradictions
        if (fact.date && existing.date && fact.date !== existing.date) {
          contradictions.push({
            type: "date_contradiction",
            fact: fact.keyPhrase,
            sourceA: existing.sourceIndex,
            sourceB: i,
            dateA: existing.date,
            dateB: fact.date,
            severity: "medium",
          });
        }
      } else {
        factMap[key] = {
          sourceIndex: i,
          value: fact.value,
          date: fact.date,
          original: fact.original,
        };
      }
    }
  }

  return contradictions;
}

// Calculate severity of contradiction
function calculateContradictionSeverity(valueA, valueB) {
  var numA = parseFloat(valueA);
  var numB = parseFloat(valueB);

  if (isNaN(numA) || isNaN(numB)) return "low";

  var diff = Math.abs(numA - numB);
  var avg = (numA + numB) / 2;
  var ratio = diff / avg;

  if (ratio > 0.5) return "high";
  if (ratio > 0.2) return "medium";
  return "low";
}

// Generate contradiction report section
function generateContradictionReport(contradictions, analyzedResults) {
  var report = "";

  if (contradictions.length === 0) {
    report += "### 10.3 矛盾検出結果\n\n";
    report +=
      "> ✅ **矛盾なし**: 情報源間に重大な矛盾は検出されませんでした。\n\n";
    return report;
  }

  report += "### 10.3 矛盾検出結果\n\n";
  report +=
    "> ⚠️ **" +
    contradictions.length +
    "件の矛盾を検出**: 情報源間で矛盾する記述が見つかりました。\n\n";

  // Group by severity
  var highSeverity = contradictions.filter(function (c) {
    return c.severity === "high";
  });
  var mediumSeverity = contradictions.filter(function (c) {
    return c.severity === "medium";
  });
  var lowSeverity = contradictions.filter(function (c) {
    return c.severity === "low";
  });

  if (highSeverity.length > 0) {
    report += "#### 🔴 高度な矛盾 (" + highSeverity.length + "件)\n\n";
    report += "情報源間で大きな食い違いがあります。確認が必要です。\n\n";

    for (var i = 0; i < highSeverity.length; i++) {
      var c = highSeverity[i];
      var sourceA = analyzedResults[c.sourceA];
      var sourceB = analyzedResults[c.sourceB];
      report += "- **" + c.fact + "**\n";
      report +=
        "  - 情報源[" +
        (c.sourceA + 1) +
        "]: " +
        c.valueA +
        " (" +
        sourceA.result.title +
        ")\n";
      report +=
        "  - 情報源[" +
        (c.sourceB + 1) +
        "]: " +
        c.valueB +
        " (" +
        sourceB.result.title +
        ")\n\n";
    }
  }

  if (mediumSeverity.length > 0) {
    report += "#### 🟡 中程度の矛盾 (" + mediumSeverity.length + "件)\n\n";

    for (var i = 0; i < mediumSeverity.length; i++) {
      var c = mediumSeverity[i];
      var sourceA = analyzedResults[c.sourceA];
      var sourceB = analyzedResults[c.sourceB];
      report += "- **" + c.fact + "**\n";
      report +=
        "  - 情報源[" + (c.sourceA + 1) + "]: " + (c.valueA || c.dateA) + "\n";
      report +=
        "  - 情報源[" +
        (c.sourceB + 1) +
        "]: " +
        (c.valueB || c.dateB) +
        "\n\n";
    }
  }

  if (lowSeverity.length > 0) {
    report += "#### 🟢 軽微な矛盾 (" + lowSeverity.length + "件)\n\n";
    report += "軽微な違いですが、注意が必要です。\n\n";

    for (var i = 0; i < Math.min(5, lowSeverity.length); i++) {
      var c = lowSeverity[i];
      report += "- " + c.fact + ": " + c.valueA + " vs " + c.valueB + "\n";
    }
    if (lowSeverity.length > 5) {
      report += "  ... 他 " + (lowSeverity.length - 5) + "件\n";
    }
    report += "\n";
  }

  return report;
}

// Generate temporal analysis report section
function generateTemporalReport(trends, results) {
  var report = "### 10.1 時系列分析\n\n";

  if (trends.oldestDate && trends.newestDate) {
    report +=
      "**期間**: " + trends.oldestDate + " 〜 " + trends.newestDate + "\n\n";
    report += "**傾向**: ";

    switch (trends.recentTrend) {
      case "increasing":
        report += "📈 増加傾向（最近の情報が多い）\n\n";
        break;
      case "decreasing":
        report += "📉 減少傾向（最近の情報が少ない）\n\n";
        break;
      default:
        report += "➡️ 安定（情報が均等に分布）\n\n";
    }
  } else {
    report += "**期間**: 不明（日付情報を抽出できませんでした）\n\n";
  }

  // Year distribution
  if (Object.keys(trends.byYear).length > 0) {
    report += "**年別分布**:\n\n";
    var years = Object.keys(trends.byYear).sort();
    for (var i = 0; i < years.length; i++) {
      var year = years[i];
      report += "- " + year + "年: " + trends.byYear[year] + "件\n";
    }
    report += "\n";
  }

  // Recent sources
  var recentSources = results.filter(function (r) {
    return r.isRecent;
  });
  if (recentSources.length > 0) {
    report += "**最新の情報源** (" + recentSources.length + "件):\n\n";
    for (var i = 0; i < Math.min(5, recentSources.length); i++) {
      var r = recentSources[i];
      report +=
        "- [" +
        (i + 1) +
        "] " +
        r.title +
        " (鮮度スコア: " +
        r.freshnessScore.toFixed(2) +
        ")\n";
    }
    if (recentSources.length > 5) {
      report += "  ... 他 " + (recentSources.length - 5) + "件\n";
    }
    report += "\n";
  }

  return report;
}

// ============================================================================
// NEW: Query Expansion Functions
// ============================================================================

// Generate expanded search queries from original query
function expandSearchQuery(originalQuery) {
  console.log("  Expanding query for multi-angle search...");

  var expansions = [];

  // Use LLM to generate query expansions
  var expansionPrompt =
    "以下の検索クエリについて、包括的な調査を行うための検索クエリの拡張を提案してください。\n\n" +
    "【タスク】\n" +
    "以下のカテゴリごとに1〜2つの検索クエリを生成してください：\n" +
    "1. 基本情報 - 定義、概要、歴史\n" +
    "2. 技術的詳細 - 仕様、アーキテクチャ、実装\n" +
    "3. 比較・評価 - 他との比較、レビュー、評価\n" +
    "4. 最新動向 - ニュース、アップデート、今後の展望\n" +
    "5. 課題・問題 - 既知の問題、課題、改善点\n\n" +
    "【重要】\n" +
    "- クエリは簡潔に（3〜8語程度）\n" +
    "- 専門用語や技術的な用語を含める\n" +
    "- 日本語と英語の両方を含める\n" +
    "- 元のクエリを含めること\n\n" +
    "【出力形式】JSON配列のみ:\n" +
    '[{"category":"basic","query":"検索クエリ1"},{"category":"technical","query":"検索クエリ2"},...]';

  try {
    var expansionResponse = iniad_ai_mop.chat(
      "Generate search query expansions. Output JSON array only.",
      expansionPrompt
    );

    expansionResponse = expansionResponse
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();

    var jsonStart = expansionResponse.indexOf("[");
    var jsonEnd = expansionResponse.lastIndexOf("]") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      expansions = JSON.parse(expansionResponse.slice(jsonStart, jsonEnd));
    }

    console.log("    Generated " + expansions.length + " expanded queries");
  } catch (e) {
    console.log("    ⚠ Query expansion failed: " + e.message);
    // Fallback to manual expansions
    expansions = [
      { category: "basic", query: originalQuery },
      { category: "basic", query: originalQuery + " 概要" },
      { category: "basic", query: originalQuery + " history" },
      { category: "technical", query: originalQuery + " 仕様" },
      { category: "technical", query: originalQuery + " 設計" },
      { category: "comparison", query: originalQuery + " 比較" },
      { category: "comparison", query: originalQuery + " review" },
      { category: "trend", query: originalQuery + " 最新" },
      { category: "trend", query: originalQuery + " news" },
      { category: "problem", query: originalQuery + " 課題" },
      { category: "problem", query: originalQuery + " issues" }
    ];
  }

  // Ensure original query is first
  var finalQueries = [{ category: "original", query: originalQuery }];
  for (var i = 0; i < expansions.length; i++) {
    if (expansions[i].query !== originalQuery) {
      finalQueries.push(expansions[i]);
    }
  }

  return finalQueries;
}

// ============================================================================
// NEW: Source Reliability Scoring Functions
// ============================================================================

// Calculate reliability score for a source
function calculateSourceReliability(result) {
  var score = 5.0; // Base score
  var reasons = [];
  var domain = (result.domain || "").toLowerCase();
  var url = (result.url || "").toLowerCase();
  var title = (result.title || "").toLowerCase();

  // High trust domains
  var highTrustDomains = [
    "github.com", "gitlab.com", "bitbucket.org", // Official code
    "docs.microsoft.com", "developer.mozilla.org", // Official docs
    "w3.org", "ietf.org", "ecma-international.org", // Standards
    "stackexchange.com", "stackoverflow.com", // Expert Q&A
    "medium.com", "dev.to", // Tech blogs
    "juejin.cn", "qiita.com", "zenn.dev" // Developer communities
  ];

  // Academic sources
  var academicDomains = [
    "scholar.google.com", "arxiv.org", "researchgate.net",
    "acm.org", "ieee.org", "springer.com", "sciencedirect.com"
  ];

  // Official documentation patterns
  var officialPatterns = [
    "/docs/", "/documentation/", "/api/", "/reference/",
    "developer.", "developers.", "docs."
  ];

  // Low trust indicators
  var lowTrustPatterns = [
    "spam", "clickbait", "fake", "hoax", "scam",
    "ads", "affiliate", "sponsored"
  ];

  // Check high trust domains
  for (var i = 0; i < highTrustDomains.length; i++) {
    if (domain.indexOf(highTrustDomains[i]) >= 0) {
      score += 2.0;
      reasons.push("信頼できる技術プラットフォーム");
      break;
    }
  }

  // Check academic sources
  for (var j = 0; j < academicDomains.length; j++) {
    if (domain.indexOf(academicDomains[j]) >= 0) {
      score += 3.0;
      reasons.push("学術的情報源");
      break;
    }
  }

  // Check for official documentation
  for (var k = 0; k < officialPatterns.length; k++) {
    if (url.indexOf(officialPatterns[k]) >= 0) {
      score += 1.5;
      reasons.push("公式ドキュメント");
      break;
    }
  }

  // Low trust indicators
  for (var l = 0; l < lowTrustPatterns.length; l++) {
    if (title.indexOf(lowTrustPatterns[l]) >= 0) {
      score -= 1.5;
      reasons.push("低信頼度の可能性");
      break;
    }
  }

  // HTTPS bonus
  if (url.indexOf("https://") === 0) {
    score += 0.5;
  }

  // Age bonus (based on content)
  if (result.pageContent) {
    var recentYears = ["2024", "2025"];
    for (var m = 0; m < recentYears.length; m++) {
      if (result.pageContent.indexOf(recentYears[m]) >= 0) {
        score += 0.3;
        break;
      }
    }
  }

  // Depth bonus (longer content suggests comprehensive coverage)
  if (result.pageContent && result.pageContent.length > 2000) {
    score += 0.5;
  }

  // Clamp score
  score = Math.max(1.0, Math.min(10.0, score));

  return {
    score: score,
    level: score >= 8 ? "high" : score >= 6 ? "medium" : "low",
    reasons: reasons
  };
}

// ============================================================================
// NEW: Cross-Reference Verification Functions
// ============================================================================

// Verify facts across multiple sources
function verifyCrossSource(analyzedResults, threshold) {
  if (!threshold) threshold = 2; // Minimum sources to confirm

  console.log("  Performing cross-reference verification...");

  var factMap = {};
  var verifiedFacts = [];
  var unverifiedFacts = [];

  // Collect all facts from all sources
  for (var i = 0; i < analyzedResults.length; i++) {
    var result = analyzedResults[i];
    var facts = extractStructuredFacts(result.factList);

    for (var j = 0; j < facts.length; j++) {
      var fact = facts[j];
      var key = fact.normalized;

      if (!factMap[key]) {
        factMap[key] = {
          original: fact.original,
          keyPhrase: fact.keyPhrase,
          sources: [],
          value: fact.value,
          date: fact.date,
          count: 0
        };
      }

      factMap[key].sources.push(i);
      factMap[key].count++;
    }
  }

  // Classify facts
  Object.keys(factMap).forEach(function (key) {
    var fact = factMap[key];
    if (fact.count >= threshold) {
      verifiedFacts.push({
        fact: fact.keyPhrase,
        original: fact.original,
        sourceCount: fact.count,
        sources: fact.sources,
        confidence: Math.min(1.0, fact.count / analyzedResults.length * 2)
      });
    } else {
      unverifiedFacts.push({
        fact: fact.keyPhrase,
        sourceCount: fact.count,
        sources: fact.sources
      });
    }
  });

  console.log("    Verified: " + verifiedFacts.length + " | Unverified: " + unverifiedFacts.length);

  return {
    verified: verifiedFacts,
    unverified: unverifiedFacts,
    total: Object.keys(factMap).length
  };
}

// ============================================================================
// NEW: Academic Citation Format Functions
// ============================================================================

// Generate academic-style citations
function generateAcademicCitations(searchResults, analyzedResults) {
  var citations = [];

  for (var i = 0; i < searchResults.length; i++) {
    var result = searchResults[i];
    var analyzed = analyzedResults[i];
    var citationNumber = i + 1;

    var citation = "";

    if (result.isLocalFile) {
      // Local file citation
      citation =
        "[" + citationNumber + "] " +
        (result.fileName || result.title) + ". " +
        "Local file: " + result.filePath + ". ";
    } else {
      // Web source citation
      var authors = extractAuthors(result.pageContent, result);
      var date = extractPublicationDate(result);
      var title = result.pageTitle || result.title;
      var domain = result.domain;

      if (authors.length > 0) {
        citation += "[" + citationNumber + "] " + authors + ". ";
      } else {
        citation += "[" + citationNumber + "] ";
      }

      citation += '"' + title + '". ';

      if (date) {
        citation += date + ". ";
      }

      citation += domain + ". " + result.url;

      // Access date
      var today = new Date().toISOString().split("T")[0];
      citation += ". Accessed: " + today + ".";
    }

    citations.push(citation);
  }

  return citations;
}

// Extract authors from content
function extractAuthors(content, result) {
  // Simple heuristic - look for author patterns
  var patterns = [
    /by\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)/i,
    /author:\s*([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)/i,
    /筆者:\s*(.+?)(?:\n|$)/
  ];

  for (var i = 0; i < patterns.length; i++) {
    var match = content.match(patterns[i]);
    if (match && match[1]) {
      return match[1];
    }
  }

  return "";
}

// Extract publication date
function extractPublicationDate(result) {
  if (result.extractedDate) {
    return result.extractedDate;
  }

  // Try to extract from content
  var content = result.pageContent || "";
  var date = extractDateFromContent(content, result.url);

  if (date) {
    return date.toISOString().split("T")[0];
  }

  return "";
}

// ============================================================================
// NEW: Insight Generation Functions
// ============================================================================

// Generate insights from analyzed results
function extractInsights(analyzedResults, searchQuery) {
  console.log("  Extracting insights and patterns...");

  var insightPrompt =
    "あなたは調査分析の専門家です。収集した情報を深く分析し、表面的な要約ではない「洞察」を抽出してください。\n\n" +
    "【分析対象】\n" +
    "トピック: " + searchQuery + "\n" +
    "情報源数: " + analyzedResults.length + "\n\n" +
    "【抽出すべき洞察のカテゴリ】\n" +
    "1. パターン認識\n" +
    "   - 複数の情報源で見られる共通の傾向\n" +
    "   - 暗黙の前提や仮定\n" +
    "   - 反復される構造や関係性\n\n" +
    "2. 予測・トレンド分析\n" +
    "   - 現在のデータに基づく将来の展望\n" +
    "   - 技術的進化の方向性\n" +
    "   - 市場の変化予測\n\n" +
    "3. 未解決の問題\n" +
    "   - 複数の情報源で言及されている課題\n" +
    "   - 懸念点やリスク\n" +
    "   - 解決策が提示されていない問題\n\n" +
    "4. 革新的なアプローチ\n" +
    "   - 従来の方法と異なるアプローチ\n" +
    "   - 新しいパラダイムや考え方\n" +
    "   - ユニークな解決策\n\n" +
    "5. 関連性の発見\n" +
    "   - 一見無関係に見える要素の関連性\n" +
    "   - 隠れたつながり\n" +
    "   - 類似した概念の統合\n\n" +
    "【重要】\n" +
    "- 単なる事実の羅列ではなく、分析・統合・解釈を含めること\n" +
    "- 情報源番号を引用すること\n" +
    "- 具体的な例を挙げつつ、普遍的な洞察を提示すること\n" +
    "- 1000-1500語で詳細に記述すること\n\n" +
    "【出力形式】\n" +
    "マークダウン形式で、カテゴリごとに見出しをつけて整理すること。";

  var allFacts = analyzedResults
    .map(function (a, i) {
      return "[" + (i + 1) + "] " + a.result.title + ":\n" + a.factList;
    })
    .join("\n\n---\n\n");

  try {
    var insights = iniad_ai_mop.chat(
      "You are an expert researcher generating deep insights. Write in Japanese with formal academic tone.",
      insightPrompt + "\n\n【情報源】\n" + allFacts.substring(0, 8000)
    );
    console.log("    ✓ Generated insights");
    return insights;
  } catch (e) {
    console.log("    ⚠ Insight generation failed: " + e.message);
    return "洞察の生成に失敗しました。";
  }
}

// ============================================================================
// NEW: Interactive Exploration Functions
// ============================================================================

// Interactive exploration phase
function performInteractiveExploration(analyzedResults, searchQuery) {
  console.log("");
  console.log("━━━ Phase X: Interactive Exploration ━━━");

  // Generate clarification questions
  var clarificationPrompt =
    "あなたは調査のファシリテーターです。「" + searchQuery + "」について調査を深めるための質問を生成してください。\n\n" +
    "【現在の状況】\n" +
    "情報源数: " + analyzedResults.length + "\n" +
    "収集済みの事実: 既に基本的な情報は収集済み\n\n" +
    "【タスク】\n" +
    "以下の観点から、調査を深めるための質問を3つ生成してください：\n" +
    "1. 不明点の特定 - 詳細が不足している点\n" +
    "2. 興味深い視点 - さらに掘り下げる価値のある点\n" +
    "3. 代替アプローチ - 別の角度からのアプローチ\n\n" +
    "【出力形式】JSON配列のみ:\n" +
    '[{"question":"質問文","category":"不明点|興味深い視点|代替アプローチ","priority":"high|medium"}]';

  try {
    var response = iniad_ai_mop.chat(
      "Generate clarification questions. Output JSON array only.",
      clarificationPrompt
    );

    response = response
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();

    var jsonStart = response.indexOf("[");
    var jsonEnd = response.lastIndexOf("]") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      var questions = JSON.parse(response.slice(jsonStart, jsonEnd));

      console.log("  Clarification Questions:");
      for (var i = 0; i < questions.length; i++) {
        var q = questions[i];
        var priorityIcon = q.priority === "high" ? "🔴" : "🟡";
        console.log("    " + priorityIcon + " Q" + (i + 1) + ": " + q.question);
        console.log("       (" + q.category + ")");
      }

      // Note: In a real implementation, this would pause for user input
      console.log("");
      console.log("  💡 Note: This is an automated run. For interactive exploration,");
      console.log("     provide these questions to the user and perform additional");
      console.log("     searches based on their priorities.");
    }

    return questions || [];
  } catch (e) {
    console.log("  ⚠ Interactive exploration setup failed: " + e.message);
    return [];
  }
}

// ============================================================================
// NEW: Metadata Extraction Functions
// ============================================================================

// Extract rich metadata from sources
function extractRichMetadata(result, pageTab) {
  var metadata = {
    authority: 0,
    socialSignals: {},
    technicalMetrics: {},
    contentQuality: {}
  };

  if (!pageTab || !result.url) return metadata;

  try {
    // Extract authority indicators
    var authorityPrompt =
      "以下のウェブページの権威性を評価するためのメタデータを抽出してください。\n\n" +
      "【ページ情報】\n" +
      "URL: " + result.url + "\n" +
      "タイトル: " + result.title + "\n" +
      "ドメイン: " + result.domain + "\n" +
      "コンテンツ: " + (result.pageContent || "").substring(0, 500) + "\n\n" +
      "【抽出項目】\n" +
      "1. 著者・組織情報（ページ内から）\n" +
      "2. 最終更新日\n" +
      "3. 関連リンクの数（外部リンク）\n" +
      "4. キーワード密度（重要キーワード）\n" +
      "5. 構造化データの有無\n\n" +
      "【出力形式】JSONのみ:\n" +
      '{"author":"","lastUpdated":"","externalLinks":0,"keywords":[],"hasStructuredData":false}';

    var metaResponse = iniad_ai_mop.chat(
      "Extract metadata from webpage. Output JSON only.",
      authorityPrompt
    );

    metaResponse = metaResponse
      .replace(/```json\n?/g, "")
      .replace(/```\n?/g, "")
      .trim();

    var jsonStart = metaResponse.indexOf("{");
    var jsonEnd = metaResponse.lastIndexOf("}") + 1;
    if (jsonStart >= 0 && jsonEnd > jsonStart) {
      var parsed = JSON.parse(metaResponse.slice(jsonStart, jsonEnd));

      // Authority score calculation
      var score = 0;
      if (parsed.author) score += 2;
      if (parsed.lastUpdated) score += 1.5;
      if (parsed.externalLinks > 10) score += 1;
      if (parsed.hasStructuredData) score += 1.5;
      if (parsed.keywords && parsed.keywords.length > 3) score += 1;

      metadata.authority = Math.min(10, score);
      metadata.author = parsed.author;
      metadata.lastUpdated = parsed.lastUpdated;
      metadata.externalLinks = parsed.externalLinks;
      metadata.keywords = parsed.keywords;
      metadata.hasStructuredData = parsed.hasStructuredData;
    }
  } catch (e) {
    console.log("    ⚠ Metadata extraction failed: " + e.message);
  }

  return metadata;
}

// ============================================================================
// NEW: Knowledge Graph Functions
// ============================================================================

// Build knowledge graph from analyzed results
function buildKnowledgeGraph(analyzedResults) {
  console.log("  Building knowledge graph...");

  var graph = {
    nodes: [],
    edges: []
  };

  var entityMap = {};

  // Extract entities from each source
  for (var i = 0; i < analyzedResults.length; i++) {
    var result = analyzedResults[i];
    var content = result.result.pageContent || "";
    var facts = result.factList || "";

    // Extract entities (people, organizations, products, concepts)
    var entityPrompt =
      "以下のテキストから重要なエンティティ（概念・人物・組織・製品・技術）を抽出してください。\n\n" +
      "【テキスト】\n" +
      facts + "\n" + content.substring(0, 1000) + "\n\n" +
      "【タスク】\n" +
      "1. 固有名詞（人名、組織名、製品名）を抽出\n" +
      "2. 重要な技術的用語・概念を抽出\n" +
      "3. 各エンティティのタイプを分類\n" +
      "4. 関連する情報源番号を記録\n\n" +
      "【出力形式】JSON配列のみ:\n" +
      '[{"name":"エンティティ名","type":"person|org|product|concept|technology","sources":[1,3]},...]';

    try {
      var entityResponse = iniad_ai_mop.chat(
        "Extract entities from text. Output JSON array only.",
        entityPrompt
      );

      entityResponse = entityResponse
        .replace(/```json\n?/g, "")
        .replace(/```\n?/g, "")
        .trim();

      var jsonStart = entityResponse.indexOf("[");
      var jsonEnd = entityResponse.lastIndexOf("]") + 1;
      if (jsonStart >= 0 && jsonEnd > jsonStart) {
        var entities = JSON.parse(entityResponse.slice(jsonStart, jsonEnd));

        for (var j = 0; j < entities.length; j++) {
          var entity = entities[j];
          var key = entity.name.toLowerCase();

          if (!entityMap[key]) {
            entityMap[key] = {
              id: "node_" + graph.nodes.length,
              name: entity.name,
              type: entity.type,
              sources: [],
              weight: 0
            };
            graph.nodes.push(entityMap[key]);
          }

          // Add this source to the entity
          var sourceId = i + 1;
          if (entityMap[key].sources.indexOf(sourceId) < 0) {
            entityMap[key].sources.push(sourceId);
            entityMap[key].weight++;
          }
        }
      }
    } catch (e) {
      console.log("    ⚠ Entity extraction failed for source " + (i + 1));
    }
  }

  // Build edges based on co-occurrence
  var nodeList = graph.nodes;
  for (var k = 0; k < nodeList.length; k++) {
    for (var l = k + 1; l < nodeList.length; l++) {
      var nodeA = nodeList[k];
      var nodeB = nodeList[l];

      // Check if they appear together in sources
      var commonSources = nodeA.sources.filter(function (s) {
        return nodeB.sources.indexOf(s) >= 0;
      });

      if (commonSources.length > 0) {
        graph.edges.push({
          source: nodeA.id,
          target: nodeB.id,
          weight: commonSources.length,
          sources: commonSources
        });
      }
    }
  }

  console.log("    ✓ Built graph with " + graph.nodes.length + " nodes, " + graph.edges.length + " edges");

  return graph;
}

// Generate knowledge graph visualization (text-based)
function generateKnowledgeGraphReport(graph) {
  var report = "";

  report += "### Knowledge Graph\n\n";
  report += "**エンティティ数**: " + graph.nodes.length + "\n";
  report += "**関連数**: " + graph.edges.length + "\n\n";

  // Group nodes by type
  var nodesByType = {};
  graph.nodes.forEach(function (node) {
    if (!nodesByType[node.type]) nodesByType[node.type] = [];
    nodesByType[node.type].push(node);
  });

  Object.keys(nodesByType).forEach(function (type) {
    report += "#### " + type.toUpperCase() + " (" + nodesByType[type].length + ")\n\n";
    nodesByType[type].forEach(function (node) {
      report += "- " + node.name + " (出現: " + node.sources.length + "回)\n";
    });
    report += "\n";
  });

  // Top relationships
  if (graph.edges.length > 0) {
    report += "#### 主要な関連\n\n";
    var sortedEdges = graph.edges.slice().sort(function (a, b) {
      return b.weight - a.weight;
    });

    for (var i = 0; i < Math.min(10, sortedEdges.length); i++) {
      var edge = sortedEdges[i];
      var nodeA = graph.nodes.find(function (n) { return n.id === edge.source; });
      var nodeB = graph.nodes.find(function (n) { return n.id === edge.target; });

      if (nodeA && nodeB) {
        report += "- " + nodeA.name + " ↔ " + nodeB.name + " (" + edge.weight + ")\n";
      }
    }
  }

  return report;
}

// ============================================================================
// NEW: Quality Assurance Functions
// ============================================================================

// Perform comprehensive quality check
function performQualityCheck(analyzedResults, report, searchResults) {
  console.log("");
  console.log("━━━ Phase X: Quality Assurance ━━━");

  var qaResults = {
    overallScore: 0,
    metrics: {}
  };

  // 1. Source Diversity
  var domains = {};
  searchResults.forEach(function (r) {
    if (r.domain && !r.isLocalFile) {
      domains[r.domain] = (domains[r.domain] || 0) + 1;
    }
  });
  var diversityScore = Math.min(10, Object.keys(domains).length);
  qaResults.metrics.sourceDiversity = {
    score: diversityScore,
    domainCount: Object.keys(domains).length,
    topDomains: Object.keys(domains).sort(function (a, b) {
      return domains[b] - domains[a];
    }).slice(0, 3)
  };

  // 2. Temporal Relevance
  var recentSources = searchResults.filter(function (r) {
    return r.isRecent;
  });
  var temporalScore = recentSources.length / searchResults.length * 10;
  qaResults.metrics.temporalRelevance = {
    score: temporalScore,
    recentCount: recentSources.length,
    totalCount: searchResults.length
  };

  // 3. Content Depth
  var avgContentLength = analyzedResults.reduce(function (sum, a) {
    return sum + (a.result.pageContent || "").length;
  }, 0) / analyzedResults.length;
  var depthScore = Math.min(10, avgContentLength / 500);
  qaResults.metrics.contentDepth = {
    score: depthScore,
    avgLength: avgContentLength
  };

  // 4. Citation Completeness
  var citationCount = (report.match(/\[\d+\]/g) || []).length;
  var completenessScore = Math.min(10, citationCount / 10);
  qaResults.metrics.citationCompleteness = {
    score: completenessScore,
    citationCount: citationCount
  };

  // 5. Cross-Source Validation
  var crossRef = verifyCrossSource(analyzedResults, 2);
  var validationScore = crossRef.verified.length / crossRef.total * 10;
  qaResults.metrics.crossSourceValidation = {
    score: validationScore,
    verified: crossRef.verified.length,
    total: crossRef.total
  };

  // Calculate overall score
  var scores = Object.keys(qaResults.metrics).map(function (key) {
    return qaResults.metrics[key].score;
  });
  qaResults.overallScore = scores.reduce(function (a, b) { return a + b; }, 0) / scores.length;

  console.log("  Overall Quality Score: " + qaResults.overallScore.toFixed(1) + "/10");
  Object.keys(qaResults.metrics).forEach(function (key) {
    var metric = qaResults.metrics[key];
    console.log("    " + key + ": " + metric.score.toFixed(1) + "/10");
  });

  return qaResults;
}

// Generate quality assurance report section
function generateQualityReport(qaResults) {
  var report = "";

  report += "## 11. Quality Assurance\n\n";
  report += "> **品質評価結果**\n>\n";

  var grade = "";
  if (qaResults.overallScore >= 9) grade = "A (優秀)";
  else if (qaResults.overallScore >= 7) grade = "B (良好)";
  else if (qaResults.overallScore >= 5) grade = "C (標準)";
  else grade = "D (要改善)";

  report += "> 総合評価: **" + grade + "** (" + qaResults.overallScore.toFixed(1) + "/10)\n\n";

  report += "### 11.1 評価指標\n\n";
  report += "| 評価項目 | スコア | 詳細 |\n";
  report += "|---------|--------|------|\n";

  Object.keys(qaResults.metrics).forEach(function (key) {
    var metric = qaResults.metrics[key];
    var score = metric.score.toFixed(1);
    var details = "";

    switch (key) {
      case "sourceDiversity":
        details = metric.domainCount + "つのドメイン";
        break;
      case "temporalRelevance":
        details = metric.recentCount + "/" + metric.totalCount + "件が最新";
        break;
      case "contentDepth":
        details = "平均 " + Math.round(metric.avgLength) + " 文字";
        break;
      case "citationCompleteness":
        details = metric.citationCount + "件の引用";
        break;
      case "crossSourceValidation":
        details = metric.verified + "/" + metric.total + "件が複数情報源で確認";
        break;
    }

    report += "| " + key + " | " + score + "/10 | " + details + " |\n";
  });

  report += "\n";

  // Recommendations
  report += "### 11.2 改善推奨事項\n\n";

  var recommendations = [];

  if (qaResults.metrics.sourceDiversity.score < 6) {
    recommendations.push("- 情報源の多様性を向上させてください。特定のドメインに偏りがあります。");
  }
  if (qaResults.metrics.temporalRelevance.score < 6) {
    recommendations.push("- より新しい情報源を追加してください。情報の鮮度が不足しています。");
  }
  if (qaResults.metrics.contentDepth.score < 6) {
    recommendations.push("- 各情報源の詳細分析を深めてください。コンテンツの深度が不十分です。");
  }
  if (qaResults.metrics.citationCompleteness.score < 6) {
    recommendations.push("- 引用情報を充実させてください。出典の明示が不足しています。");
  }
  if (qaResults.metrics.crossSourceValidation.score < 6) {
    recommendations.push("- 複数の情報源で事実を確認してください。クロス検証が不十分です。");
  }

  if (recommendations.length > 0) {
    recommendations.forEach(function (rec) {
      report += rec + "\n";
    });
  } else {
    report += "✅ すべての評価項目で良好な結果が得られました。\n\n";
  }

  report += "---\n\n";

  return report;
}

workflow();
