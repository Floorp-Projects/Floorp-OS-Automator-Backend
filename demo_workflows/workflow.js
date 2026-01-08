/**
 * Cross-Platform File Researcher
 * Searches files across OS (Spotlight) and Web (Google Drive)
 */

var NL = String.fromCharCode(10);

function workflow() {
  var searchQuery = "test";

  console.log("========================================");
  console.log("  Cross-Platform File Researcher");
  console.log("========================================");
  console.log("");
  console.log("Search Query: " + searchQuery);
  console.log("");

  // Phase 1: Local Search (Spotlight / mdfind)
  console.log("--- Phase 1: Local Search (Spotlight) ---");
  var localResults = [];

  try {
    var mdfindCommand = "mdfind -name " + searchQuery + " | head -20";
    console.log("  Executing: " + mdfindCommand);

    var mdfindOutput = exec(mdfindCommand);
    var lines = mdfindOutput.trim().split(NL);
    var localPaths = [];
    for (var x = 0; x < lines.length; x++) {
      if (lines[x].length > 0) {
        localPaths.push(lines[x]);
      }
    }

    for (var i = 0; i < localPaths.length; i++) {
      localResults.push({
        type: "local",
        path: localPaths[i],
        name: localPaths[i].split("/").pop(),
        source: "Spotlight",
      });
    }

    console.log("  Found " + localResults.length + " local files");
    for (var j = 0; j < Math.min(localResults.length, 5); j++) {
      console.log("    [Local] " + localResults[j].name);
    }
  } catch (e) {
    console.log("  Local search error: " + e);
  }

  // Phase 2: Web Search (Google Drive)
  console.log("");
  console.log("--- Phase 2: Web Search (Google Drive) ---");
  var gdriveResults = [];
  var gdriveTab = null;

  try {
    var gdriveSearchUrl =
      "https://drive.google.com/drive/search?q=" +
      encodeURIComponent(searchQuery);
    console.log("  Opening: " + gdriveSearchUrl);

    var existingTabs = floorp.listBrowserTabs();
    var existingTabsData = JSON.parse(existingTabs);
    var foundTab = null;

    for (var t = 0; t < existingTabsData.length; t++) {
      if (
        existingTabsData[t].url &&
        existingTabsData[t].url.indexOf("drive.google.com") >= 0
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

    console.log("  Waiting for search results to load...");
    sleep(5000);

    console.log("  Extracting search results...");

    var elementsJson = floorp.tabGetElements(gdriveTab, "tr[role=row]");
    var elementsData = JSON.parse(elementsJson);
    var rowElements = elementsData.elements || [];

    console.log("  Found " + rowElements.length + " rows in search results");

    var debugStats = { aria: 0, tooltip: 0, noMatch: 0 };

    for (var r = 0; r < rowElements.length; r++) {
      var rowHtml = rowElements[r];

      var fileId = extractAttr(rowHtml, "data-id");
      if (!fileId) continue;

      var fileName = "";
      var matchedBy = "";

      var ariaLabel = extractAttr(rowHtml, "aria-label");
      if (ariaLabel) {
        var commaIdx = ariaLabel.indexOf(",");
        if (commaIdx > 0) {
          fileName = ariaLabel.substring(0, commaIdx);
        } else {
          fileName = ariaLabel;
        }
        matchedBy = "aria-label";
        debugStats.aria++;
      }

      if (!fileName) {
        var tooltip = extractAttr(rowHtml, "data-tooltip");
        if (tooltip) {
          fileName = tooltip;
          matchedBy = "data-tooltip";
          debugStats.tooltip++;
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
          _matchedBy: matchedBy,
        });
      }
    }

    console.log(
      "  [Debug] Selector stats: aria=" +
        debugStats.aria +
        ", tooltip=" +
        debugStats.tooltip +
        ", noMatch=" +
        debugStats.noMatch
    );

    if (gdriveResults.length === 0 && rowElements.length > 0) {
      console.log("  [Debug] No files extracted. First row HTML sample:");
      console.log("  " + rowElements[0].substring(0, 500) + "...");
    }

    console.log("  Found " + gdriveResults.length + " files on Google Drive");
    for (var g = 0; g < Math.min(gdriveResults.length, 5); g++) {
      console.log("    [GDrive] " + gdriveResults[g].name);
    }
  } catch (e) {
    console.log("  Google Drive search error: " + e);
  }

  // Phase 3: Results Summary
  console.log("");
  console.log("--- Phase 3: Results Summary ---");

  var totalResults = localResults.length + gdriveResults.length;
  console.log("  Total files found: " + totalResults);
  console.log("    - Local (Spotlight): " + localResults.length);
  console.log("    - Web (Google Drive): " + gdriveResults.length);

  // Phase 4: AI Analysis
  console.log("");
  console.log("--- Phase 4: AI Analysis ---");

  if (totalResults > 0) {
    try {
      var analysisPrompt = "Analyze the following search results:" + NL + NL;
      analysisPrompt += "Search Query: " + searchQuery + NL + NL;

      analysisPrompt += "[Local (" + localResults.length + " files)]" + NL;
      for (var l = 0; l < Math.min(localResults.length, 10); l++) {
        analysisPrompt += "- " + localResults[l].path + NL;
      }

      analysisPrompt +=
        NL + "[Google Drive (" + gdriveResults.length + " files)]" + NL;
      for (var d = 0; d < Math.min(gdriveResults.length, 10); d++) {
        analysisPrompt +=
          "- " + gdriveResults[d].name + " (" + gdriveResults[d].url + ")" + NL;
      }

      analysisPrompt +=
        NL + "Briefly analyze the file distribution. Reply in Japanese.";

      console.log("  Generating AI analysis...");
      var aiAnalysis = iniad_ai_mop.chat(
        "You are a file management advisor.",
        analysisPrompt
      );

      console.log("");
      console.log("--- AI Analysis ---");
      var analysisLines = aiAnalysis.split(NL);
      for (var a = 0; a < analysisLines.length; a++) {
        console.log("  " + analysisLines[a]);
      }
      console.log("-------------------");
    } catch (e) {
      console.log("  AI analysis skipped: " + e);
    }
  } else {
    console.log("  No results to analyze.");
  }

  console.log("");
  console.log("========================================");
  console.log("  Cross-Platform Search Complete!");
  console.log("========================================");

  return {
    success: true,
    query: searchQuery,
    localCount: localResults.length,
    gdriveCount: gdriveResults.length,
  };
}

function extractAttr(html, attrName) {
  var searchStr = attrName + "=" + String.fromCharCode(34);
  var startIdx = html.indexOf(searchStr);
  if (startIdx < 0) return null;

  startIdx += searchStr.length;
  var endIdx = html.indexOf(String.fromCharCode(34), startIdx);
  if (endIdx < 0) return null;

  return html.substring(startIdx, endIdx);
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
