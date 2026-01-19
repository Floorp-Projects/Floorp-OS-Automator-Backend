/**
 * Finder File Content Reader
 *
 * 1. Search files using Finder plugin (AppleScript)
 * 2. Read file contents using filesystem plugin
 * 3. Display results
 */

function workflow() {
  var searchQuery = "Floorp";
  var searchDirectory = "/Users/user/Desktop";
  var maxResults = 20;
  var outputPath = "/Users/user/Desktop/finder_content_results.json";

  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║         Finder File Content Reader                         ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
  console.log("");
  console.log("  Search Directory: " + searchDirectory);
  console.log("  Search Query: " + searchQuery);
  console.log("  Max Results: " + maxResults);
  console.log("");

  // --- Phase 1: Find files using Finder plugin ---
  console.log("━━━ Phase 1: Find Files ━━━");
  var files = [];

  try {
    if (
      app &&
      app.sapphillon &&
      app.sapphillon.core &&
      app.sapphillon.core.finder &&
      app.sapphillon.core.finder.findFiles
    ) {
      var startTime = Date.now();
      var pathsJson = app.sapphillon.core.finder.findFiles(
        searchDirectory,
        searchQuery,
        maxResults,
      );
      var elapsed = Date.now() - startTime;

      try {
        files = JSON.parse(pathsJson || "[]");
      } catch (e) {
        console.log("  JSON parse error: " + e);
        files = [];
      }

      console.log("  ✓ Found " + files.length + " files in " + elapsed + "ms");
    } else {
      console.log("  ✗ Finder plugin not available");
      return;
    }
  } catch (e) {
    console.log("  ✗ Finder error: " + e);
    return;
  }

  if (files.length === 0) {
    console.log("  No files found matching query.");
    return;
  }

  // --- Phase 2: Read file contents ---
  console.log("");
  console.log("━━━ Phase 2: Read File Contents ━━━");

  var results = [];
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
  ];

  for (var i = 0; i < files.length; i++) {
    var filePath = files[i];
    var fileName = filePath.split("/").pop() || filePath;
    var extension = "";
    var lastDot = fileName.lastIndexOf(".");
    if (lastDot > 0) {
      extension = fileName.substring(lastDot).toLowerCase();
    }

    // Check if it's a readable text file
    var isReadable = false;
    for (var j = 0; j < readableExtensions.length; j++) {
      if (extension === readableExtensions[j]) {
        isReadable = true;
        break;
      }
    }

    var fileResult = {
      index: i + 1,
      path: filePath,
      name: fileName,
      extension: extension,
      isReadable: isReadable,
      content: null,
      contentLength: 0,
      error: null,
    };

    if (isReadable) {
      try {
        if (
          app.sapphillon.core.filesystem &&
          app.sapphillon.core.filesystem.read
        ) {
          var content = app.sapphillon.core.filesystem.read(filePath);
          fileResult.content = content;
          fileResult.contentLength = content ? content.length : 0;
          console.log(
            "  [" +
              (i + 1) +
              "] ✓ " +
              fileName +
              " (" +
              fileResult.contentLength +
              " chars)",
          );
        } else {
          fileResult.error = "filesystem.read not available";
          console.log(
            "  [" +
              (i + 1) +
              "] ✗ " +
              fileName +
              " - filesystem plugin not available",
          );
        }
      } catch (e) {
        fileResult.error = String(e);
        console.log("  [" + (i + 1) + "] ✗ " + fileName + " - " + e);
      }
    } else {
      console.log(
        "  [" + (i + 1) + "] ⊘ " + fileName + " (skipped: not a text file)",
      );
    }

    results.push(fileResult);
  }

  // --- Phase 3: Display content previews ---
  console.log("");
  console.log("━━━ Phase 3: Content Previews ━━━");

  var readCount = 0;
  for (var i = 0; i < results.length; i++) {
    var r = results[i];
    if (r.content && r.contentLength > 0) {
      readCount++;
      console.log("");
      console.log("┌─ " + r.name + " (" + r.contentLength + " chars) ─");
      // Show first 500 chars as preview
      var preview = r.content.substring(0, 500);
      if (r.contentLength > 500) {
        preview += " ... (truncated)";
      }
      console.log(preview);
      console.log("└─────────────────────────────────────────");
    }
  }

  console.log("");
  console.log("━━━ Summary ━━━");
  console.log("  Total files found: " + files.length);
  console.log("  Files read: " + readCount);
  console.log("  Files skipped: " + (files.length - readCount));

  // --- Save results to JSON ---
  console.log("");
  console.log("━━━ Saving Results ━━━");
  try {
    // Create summary without full content for JSON (to avoid huge output)
    var summaryResults = [];
    for (var i = 0; i < results.length; i++) {
      var r = results[i];
      summaryResults.push({
        index: r.index,
        path: r.path,
        name: r.name,
        extension: r.extension,
        isReadable: r.isReadable,
        contentLength: r.contentLength,
        contentPreview: r.content ? r.content.substring(0, 200) : null,
        error: r.error,
      });
    }

    var jsonOutput = JSON.stringify(
      {
        query: searchQuery,
        directory: searchDirectory,
        timestamp: new Date().toISOString(),
        totalFiles: files.length,
        filesRead: readCount,
        results: summaryResults,
      },
      null,
      2,
    );

    app.sapphillon.core.filesystem.write(outputPath, jsonOutput);
    console.log("  → " + outputPath);
  } catch (e) {
    console.log("  ✗ Save error: " + e);
  }

  console.log("");
  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║                      Complete                              ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
}

// Auto-run
workflow();
