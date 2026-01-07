/**
 * Deep Research Demo - Browser Paper Research
 *
 * Searches 3 sources for browser-related papers:
 * 1. Google Scholar
 * 2. arXiv
 * 3. Semantic Scholar
 *
 * Uses LLM to summarize and analyze papers, outputs Markdown report.
 */

function workflow() {
  var topic = "web browser security";
  var outputPath = "/Users/user/Desktop/Research_Report_Browser.md";
  var allPapers = [];

  console.log("=== Deep Research: " + topic + " ===");
  console.log("");

  // --- 1. arXiv Scraping ---
  console.log("--- Scraping arXiv ---");
  var arxivUrl =
    "https://arxiv.org/search/?query=" +
    encodeURIComponent(topic) +
    "&searchtype=all";
  var arxivTab = null;
  try {
    arxivTab = floorp.createTab(arxivUrl, false);
    floorp.tabWaitForElement(arxivTab, "li.arxiv-result", 10000);
    sleep(2000);

    for (var i = 1; i <= 5; i++) {
      var baseSel = "li.arxiv-result:nth-of-type(" + i + ")";
      try {
        var title = getText(arxivTab, baseSel + " .title");
        var authors = getText(arxivTab, baseSel + " .authors");
        var abstract = getText(arxivTab, baseSel + " .abstract-full");
        var linkEl = floorp.tabAttribute(
          arxivTab,
          baseSel + " p.list-title a",
          "href"
        );
        var link = JSON.parse(linkEl).value || "";

        if (title) {
          allPapers.push({
            source: "arXiv",
            title: cleanText(title),
            authors: cleanText(authors),
            abstract: cleanText(abstract).substring(0, 300),
            link: link,
            citations: "N/A",
          });
        }
      } catch (e) {}
    }
    console.log(
      "  Found " +
        allPapers.filter((p) => p.source === "arXiv").length +
        " papers"
    );
  } catch (e) {
    console.log("arXiv Error: " + e);
  } finally {
    if (arxivTab) floorp.destroyTabInstance(arxivTab);
  }

  // --- 2. Google Scholar Scraping ---
  console.log("--- Scraping Google Scholar ---");
  var gsUrl =
    "https://scholar.google.com/scholar?q=" + encodeURIComponent(topic);
  var gsTab = null;
  try {
    gsTab = floorp.createTab(gsUrl, false);
    floorp.tabWaitForElement(gsTab, ".gs_ri", 10000);
    sleep(2000);

    for (var i = 1; i <= 5; i++) {
      var baseSel = ".gs_r.gs_or.gs_scl:nth-of-type(" + i + ")";
      try {
        var title = getText(gsTab, baseSel + " h3.gs_rt a");
        var authors = getText(gsTab, baseSel + " .gs_a");
        var snippet = getText(gsTab, baseSel + " .gs_rs");
        var linkEl = floorp.tabAttribute(
          gsTab,
          baseSel + " h3.gs_rt a",
          "href"
        );
        var link = JSON.parse(linkEl).value || "";

        // Get citations
        var citations = "0";
        try {
          var citText = getText(gsTab, baseSel + " .gs_fl a:nth-of-type(3)");
          if (citText && citText.includes("被引用数")) {
            citations = citText.replace(/[^0-9]/g, "");
          }
        } catch (e) {}

        if (title) {
          allPapers.push({
            source: "Google Scholar",
            title: cleanText(title),
            authors: cleanText(authors),
            abstract: cleanText(snippet).substring(0, 300),
            link: link,
            citations: citations,
          });
        }
      } catch (e) {}
    }
    console.log(
      "  Found " +
        allPapers.filter((p) => p.source === "Google Scholar").length +
        " papers"
    );
  } catch (e) {
    console.log("Google Scholar Error: " + e);
  } finally {
    if (gsTab) floorp.destroyTabInstance(gsTab);
  }

  // --- 3. Semantic Scholar Scraping ---
  console.log("--- Scraping Semantic Scholar ---");
  var ssUrl =
    "https://www.semanticscholar.org/search?q=" + encodeURIComponent(topic);
  var ssTab = null;
  try {
    ssTab = floorp.createTab(ssUrl, false);
    floorp.tabWaitForElement(ssTab, ".cl-paper-title", 15000);
    sleep(3000);

    for (var i = 1; i <= 5; i++) {
      var titleSel = "h2.cl-paper-title:nth-of-type(" + i + ")";
      try {
        // Semantic Scholar has different structure, get all titles first
        var titles = [];
        for (var j = 1; j <= 5; j++) {
          var t = getText(
            ssTab,
            "[data-test-id='search-result']:nth-of-type(" +
              j +
              ") h2.cl-paper-title"
          );
          if (t) titles.push(t);
        }

        // Get first 5 unique papers
        if (i <= titles.length) {
          allPapers.push({
            source: "Semantic Scholar",
            title: cleanText(titles[i - 1]),
            authors: "See link",
            abstract: "(TLDR available on site)",
            link: ssUrl,
            citations: "N/A",
          });
        }
      } catch (e) {}
    }
    console.log(
      "  Found " +
        allPapers.filter((p) => p.source === "Semantic Scholar").length +
        " papers"
    );
  } catch (e) {
    console.log("Semantic Scholar Error: " + e);
  } finally {
    if (ssTab) floorp.destroyTabInstance(ssTab);
  }

  console.log("");
  console.log("Total papers collected: " + allPapers.length);

  // --- 4. LLM Summarization ---
  console.log("");
  console.log("--- AI Analysis ---");

  var summaries = [];
  for (var i = 0; i < Math.min(allPapers.length, 10); i++) {
    var paper = allPapers[i];
    console.log("  Summarizing: " + paper.title.substring(0, 40) + "...");
    try {
      var summary = iniad_ai_mop.chat(
        "You are a research paper summarizer. Summarize in 1-2 sentences in Japanese.",
        "Title: " + paper.title + "\nAbstract: " + paper.abstract
      );
      summaries.push({
        paper: paper,
        summary: summary,
      });
    } catch (e) {
      console.log("    LLM Error: " + e);
      summaries.push({
        paper: paper,
        summary: "(要約生成失敗)",
      });
    }
  }

  // --- 5. LLM Analysis for Academic Report ---
  console.log("  Generating academic analysis...");

  // Prepare paper data for LLM
  var paperData = allPapers.map(function (p, i) {
    var s = summaries.find(function (x) {
      return x.paper.title === p.title;
    });
    return {
      num: i + 1,
      title: p.title,
      authors: p.authors,
      abstract: p.abstract,
      summary: s ? s.summary : "",
      source: p.source,
      link: p.link,
    };
  });

  // Generate Abstract
  console.log("  Generating Abstract...");
  var abstractText = "";
  try {
    abstractText = iniad_ai_mop.chat(
      "You are an academic researcher. Write a concise research abstract (150-200 words) in Japanese that summarizes the current state of research on the given topic. Include: background, scope of this survey, key findings, and implications.",
      "Topic: " +
        topic +
        "\n\nPapers analyzed:\n" +
        paperData
          .map(function (p) {
            return "- " + p.title;
          })
          .join("\n")
    );
  } catch (e) {
    abstractText = "（Abstract生成エラー）";
  }

  // Generate Introduction
  console.log("  Generating Introduction...");
  var introText = "";
  try {
    introText = iniad_ai_mop.chat(
      "You are an academic researcher. Write an Introduction section (200-300 words) in Japanese for a survey paper. Include: 1) Background and importance of the topic, 2) Current challenges, 3) Purpose of this survey, 4) Structure overview. Use formal academic tone.",
      "Topic: " +
        topic +
        "\n\nThis survey covers " +
        allPapers.length +
        " papers from arXiv and Google Scholar."
    );
  } catch (e) {
    introText = "（Introduction生成エラー）";
  }

  // Generate Findings (Trend Analysis)
  console.log("  Generating Key Findings...");
  var findingsText = "";
  try {
    var paperSummaries = paperData
      .map(function (p) {
        return p.num + ". " + p.title + ": " + p.summary;
      })
      .join("\n");
    findingsText = iniad_ai_mop.chat(
      "You are an academic researcher. Analyze the following paper summaries and write a detailed Findings section (400-500 words) in Japanese. Organize into 3-4 thematic categories. For each category: provide a heading, explain the key research direction, and cite relevant papers by number (e.g., [1], [3,5]). Use formal academic tone.",
      "Topic: " + topic + "\n\nPaper summaries:\n" + paperSummaries
    );
  } catch (e) {
    findingsText = "（Findings生成エラー）";
  }

  // Generate Discussion
  console.log("  Generating Discussion...");
  var discussionText = "";
  try {
    discussionText = iniad_ai_mop.chat(
      "You are an academic researcher. Write a Discussion section (200-300 words) in Japanese that: 1) Synthesizes the key trends, 2) Identifies research gaps, 3) Suggests future research directions, 4) Discusses practical implications. Use formal academic tone.",
      "Topic: " +
        topic +
        "\n\nKey themes found in " +
        allPapers.length +
        " papers about " +
        topic
    );
  } catch (e) {
    discussionText = "（Discussion生成エラー）";
  }

  // Generate Conclusions
  console.log("  Generating Conclusions...");
  var conclusionsText = "";
  try {
    conclusionsText = iniad_ai_mop.chat(
      "You are an academic researcher. Write a brief Conclusions section (100-150 words) in Japanese that summarizes: 1) Main contributions of this survey, 2) Key takeaways, 3) Final remarks on the importance of the topic. Use formal academic tone.",
      "Topic: " +
        topic +
        "\n\nThis survey analyzed " +
        allPapers.length +
        " papers on " +
        topic
    );
  } catch (e) {
    conclusionsText = "（Conclusions生成エラー）";
  }

  // --- 6. Generate Academic Report ---
  console.log("");
  console.log("--- Generating Academic Report ---");

  var today = new Date().toISOString().split("T")[0];

  var report = "";
  report +=
    "# " +
    topic.charAt(0).toUpperCase() +
    topic.slice(1) +
    ": A Survey of Current Research\n\n";
  report += "**Floorp Deep Research** | Generated: " + today + "\n\n";
  report += "---\n\n";

  // Abstract
  report += "## Abstract\n\n";
  report += abstractText + "\n\n";
  report +=
    "**Keywords**: " +
    topic +
    ", security, privacy, AI agents, browser extensions\n\n";
  report += "---\n\n";

  // Table of Contents
  report += "## Table of Contents\n\n";
  report += "1. [Introduction](#introduction)\n";
  report += "2. [Methodology](#methodology)\n";
  report += "3. [Key Findings](#key-findings)\n";
  report += "4. [Discussion](#discussion)\n";
  report += "5. [Conclusions](#conclusions)\n";
  report += "6. [References](#references)\n\n";
  report += "---\n\n";

  // Introduction
  report += "## 1. Introduction\n\n";
  report += introText + "\n\n";

  // Methodology
  report += "## 2. Methodology\n\n";
  report += "### 2.1 Search Strategy\n\n";
  report +=
    "本調査では、以下の学術データベースを用いて文献検索を実施した：\n\n";
  report += "| データベース | 検索クエリ | 取得件数 |\n";
  report += "|-------------|-----------|--------|\n";
  report +=
    '| arXiv | "' +
    topic +
    '" | ' +
    allPapers.filter(function (p) {
      return p.source === "arXiv";
    }).length +
    " |\n";
  report +=
    '| Google Scholar | "' +
    topic +
    '" | ' +
    allPapers.filter(function (p) {
      return p.source === "Google Scholar";
    }).length +
    " |\n";
  report += "| **合計** | | **" + allPapers.length + "** |\n\n";
  report += "### 2.2 Selection Criteria\n\n";
  report +=
    "検索結果から上位5件ずつを選定し、タイトル・著者・アブストラクトを収集した。";
  report +=
    "各論文についてAI（GPT-5 nano）による要約を生成し、主要なテーマを抽出した。\n\n";

  // Key Findings
  report += "## 3. Key Findings\n\n";
  report += findingsText + "\n\n";

  // Paper Details
  report += "### 3.1 Analyzed Papers\n\n";
  for (var i = 0; i < paperData.length; i++) {
    var p = paperData[i];
    report += "#### [" + p.num + "] " + p.title + "\n\n";
    report += "- **著者**: " + p.authors + "\n";
    report += "- **出典**: " + p.source + "\n";
    report += "- **要約**: " + p.summary + "\n";
    report +=
      "- **リンク**: [" + p.link.substring(0, 50) + "...](" + p.link + ")\n\n";
  }

  // Discussion
  report += "## 4. Discussion\n\n";
  report += discussionText + "\n\n";

  // Conclusions
  report += "## 5. Conclusions\n\n";
  report += conclusionsText + "\n\n";

  // References
  report += "---\n\n";
  report += "## 6. References\n\n";
  for (var i = 0; i < allPapers.length; i++) {
    var p = allPapers[i];
    report += "[" + (i + 1) + "] " + p.authors.replace("Authors: ", "") + ". ";
    report += '"' + p.title + '." ';
    report += "*" + p.source + "*. ";
    report += "Available: " + p.link + "\n\n";
  }

  report += "---\n\n";
  report +=
    "*This report was automatically generated by Floorp Deep Research using AI-powered literature analysis.*\n";

  // --- 7. Save Report ---
  console.log("Saving report to: " + outputPath);
  console.log("Report length: " + report.length + " characters");

  try {
    var result = fs.write(outputPath, report);
    console.log("Write result: " + result);
    console.log("✓ Report saved to: " + outputPath);
  } catch (e) {
    console.log("Save error: " + e);
    console.log("Outputting report to console instead:");
    console.log("");
    console.log(report);
  }

  console.log("");
  console.log("=== Deep Research Complete! ===");
}

// Helpers
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
