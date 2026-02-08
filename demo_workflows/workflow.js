/**
 * Video Site Comparison Workflow (YouTube vs Niconico)
 * Topic: "Floorp"
 *
 * Updates:
 * - Scrape top 50 items (YouTube).
 * - Scroll to trigger lazy loading.
 * - Thumbnail download & insertion.
 */

function workflow() {
  var topic = "Floorp";
  var excelPath = "/Users/user/Desktop/Video_Comparison_Floorp.xlsx";
  var results = [];
  var slackUrl = "https://app.slack.com/client/T0A62PPRD7G/C0A68CVNZFE";
  var enableSlackUpload = true;

  console.log("Starting Video Comparison for topic: " + topic);

  // --- 1. YouTube Scraping ---
  console.log("--- Scraping YouTube (Top 50) ---");
  var ytUrl = "https://www.youtube.com/results?search_query=" + topic;
  var ytTab = null;
  try {
    ytTab = floorp.createTab(ytUrl, false);

    // Initial wait - ensure content is fully loaded
    try {
      floorp.tabWaitForElement(ytTab, "ytd-video-renderer #video-title", 10000);
      floorp.tabWaitForNetworkIdle(ytTab, 3000);
      sleep(ytTab, 3000); // Extra wait for thumbnail images to fully load
    } catch (e) {
      /* ignore */
    }

    // Scroll to top first to ensure first items are visible
    try {
      floorp.tabScrollTo(ytTab, "ytd-video-renderer:nth-of-type(1)");
      sleep(ytTab, 1000);
    } catch (e) {}

    // Scroll Loop to load more items (Lazy Loading) - Aiming for 100 items
    console.log("   Scrolling to load more items (Targeting Loader)...");

    for (var s = 0; s < 15; s++) {
      // Scroll loader into view to trigger next batch
      try {
        floorp.tabScrollTo(ytTab, "ytd-continuation-item-renderer");
      } catch (e) {
        // Fallback: Scroll to a section
        try {
          floorp.tabScrollTo(
            ytTab,
            "ytd-item-section-renderer:nth-of-type(" + (s + 1) + ")"
          );
        } catch (ex) {}
      }
      sleep(ytTab, 2500); // 2.5s wait for load
    }

    // Nested Loop: Section -> Videos (YouTube structure)
    var totalNeeded = 70;
    var currentRank = 1;

    for (var sec = 1; sec <= 25; sec++) {
      if (results.length >= totalNeeded) break;

      var sectionSel = "ytd-item-section-renderer:nth-of-type(" + sec + ")";

      for (var vid = 1; vid <= 20; vid++) {
        if (results.length >= totalNeeded) break;

        var baseSel =
          sectionSel + " ytd-video-renderer:nth-of-type(" + vid + ")";
        var titleSel = baseSel + " #video-title";
        var viewSel = baseSel + " #metadata-line span:nth-of-type(1)";
        var thumbSel = baseSel + " ytd-thumbnail img";

        try {
          var title = getText(ytTab, titleSel);
          if (!title) {
            // Check next to see if section continues
            var nextTitle = getText(
              ytTab,
              sectionSel +
                " ytd-video-renderer:nth-of-type(" +
                (vid + 1) +
                ") #video-title"
            );
            if (!nextTitle) break; // End of section
            continue; // Skip this slot (might be ad)
          }

          var viewRaw = getText(ytTab, viewSel);

          // Get video URL from title link href
          var videoUrl = "https://www.youtube.com";
          try {
            var attrJson = floorp.tabAttribute(ytTab, titleSel, "href");
            var attrData = JSON.parse(attrJson);
            var href = attrData.value || "";
            if (href) {
              // href might be relative like "/watch?v=xxx" or absolute
              if (href.startsWith("/")) {
                videoUrl = "https://www.youtube.com" + href;
              } else {
                videoUrl = href;
              }
            }
          } catch (e) {}

          // Thumbnail capture - scroll and wait, then capture
          var thumbPath = "";
          try {
            var desktop = "/Users/user/Desktop";
            var tmp = desktop + "/thumbs/yt_thumb_" + currentRank + ".png";

            // Scroll to video element
            try {
              floorp.tabScrollTo(ytTab, baseSel);
            } catch (e) {}
            sleep(ytTab, 800);

            saveElementScreenshot(ytTab, thumbSel, tmp);
            thumbPath = tmp;
          } catch (e) {
            console.log("YT Thumb Error (" + currentRank + "): " + e);
          }

          results.push({
            platform: "YouTube",
            rank: currentRank++,
            title: title,
            viewsRaw: viewRaw,
            views: parseViews(viewRaw),
            url: videoUrl,
            thumbPath: thumbPath,
          });
        } catch (e) {
          // Ignore transient errors
        }
      }
    }
  } catch (e) {
    console.log("YouTube Error: " + e);
  } finally {
    if (ytTab) floorp.destroyTabInstance(ytTab);
  }

  // --- 2. Niconico Scraping ---
  console.log("--- Scraping Niconico (Page 1) ---");
  var nicoUrl = "https://www.nicovideo.jp/search/" + topic + "?sort=f&order=d";
  var nicoTab = null;
  try {
    nicoTab = floorp.createTab(nicoUrl, false);
    try {
      floorp.tabWaitForElement(nicoTab, ".Pressable", 10000);
      floorp.tabWaitForNetworkIdle(nicoTab, 3000);
    } catch (e) {
      /* ignore */
    }

    // Niconico is paginated usually, getting top ~30 from page 1
    for (var i = 1; i <= 40; i++) {
      var baseSel = ".Pressable:nth-of-type(" + i + ")";
      var titleSel = baseSel + " a.fw_bold.lc_2";

      try {
        var title = getText(nicoTab, titleSel);
        var viewJson = floorp.tabElementText(
          nicoTab,
          baseSel + " p:nth-of-type(1)"
        );
        var viewRaw = "";
        try {
          viewRaw = JSON.parse(viewJson).text;
        } catch (e) {
          viewRaw = viewJson;
        }

        // Thumb
        var thumbPath = "";
        try {
          var desktop = "/Users/user/Desktop";
          var tmp = desktop + "/thumbs/nico_thumb_" + i + ".png";
          saveElementScreenshot(nicoTab, baseSel + " img", tmp);
          thumbPath = tmp;
        } catch (e) {
          console.log("Nico Thumb Error: " + e);
        }

        // Get video URL from title link href
        var videoUrl = "https://www.nicovideo.jp";
        try {
          var attrJson = floorp.tabAttribute(nicoTab, titleSel, "href");
          var attrData = JSON.parse(attrJson);
          var href = attrData.value || "";
          if (href) {
            if (href.startsWith("/")) {
              videoUrl = "https://www.nicovideo.jp" + href;
            } else {
              videoUrl = href;
            }
          }
        } catch (e) {}

        if (title) {
          results.push({
            platform: "Niconico",
            rank: i,
            title: title,
            viewsRaw: viewRaw,
            views: parseViews(viewRaw),
            url: videoUrl,
            thumbPath: thumbPath,
          });
        }
      } catch (e) {}
    }
  } catch (e) {
    console.log("Niconico Error: " + e);
  } finally {
    if (nicoTab) floorp.destroyTabInstance(nicoTab);
  }

  console.log("Collected " + results.length + " videos.");

  // --- 3. Excel Export (Using Native Rust Image Embedding) ---
  console.log("Creating Excel: " + excelPath);
  try {
    // Columns: "Platform", "Rank", "Image", "Title", "Views", "Raw Views", "Link"
    var matrix = [
      ["Platform", "Rank", "Image", "Title", "Views", "Raw Views", "Link"],
    ];

    // Build image list for native embedding
    // Format: { file_path, row, col }
    var imageList = [];

    for (var k = 0; k < results.length; k++) {
      var r = results[k];
      matrix.push([
        String(r.platform),
        String(r.rank),
        "", // Empty cell for image
        String(r.title),
        String(r.views),
        String(r.viewsRaw),
        String(r.url),
      ]);

      // Add image to list if thumbnail exists
      if (r.thumbPath) {
        imageList.push({
          file_path: r.thumbPath,
          row: k + 1, // Row 0 is header, data starts at row 1
          col: 2, // Column C (0-indexed: A=0, B=1, C=2)
        });
      }
    }

    // Use the new native Rust function that writes data AND embeds images
    console.log(
      "Writing " +
        matrix.length +
        " rows with " +
        imageList.length +
        " images..."
    );

    // Formatting options (customizable)
    var formatOptions = {
      row_height: 80, // Reduced to fit better
      column_widths: [12, 6, 22, 50, 14, 15, 25], // Column C: balanced
      image_scale: 0.3, // Smaller to fit in cell
      font_size: 14, // Larger font (default is 11)
      // Chart configuration
      chart: {
        chart_type: "pie", // Pie chart for views distribution
        title: "Views Distribution (Top 15)",
        category_col: 3, // Column D (Title)
        value_col: 4, // Column E (Views)
        position_row: 1,
        position_col: 12, // Column M (further right)
        width: 2400, // 5x default
        height: 1440, // 5x default
        top_n: 15, // Show only top 15 + Others
      },
    };

    var result = excel.writeRangeWithImages(
      excelPath,
      "Sheet1",
      "A1",
      matrix,
      imageList,
      formatOptions
    );
    console.log("Write complete: " + result);

    // Open the file
    excel.openInApp(excelPath);
    console.log("Excel file opened.");

    // Optional: Upload the file to Slack after opening Excel
    if (enableSlackUpload) {
      try {
        uploadFileToSlack(excelPath, slackUrl, "動画比較Excelを共有します。");
      } catch (e) {
        console.log("Slack Upload Error: " + e);
      }
    }
  } catch (e) {
    console.log("Excel Error: " + e);
  }
}

// Helpers
function getText(tab, sel) {
  try {
    var json = floorp.tabElementText(tab, sel);
    try {
      return JSON.parse(json).text;
    } catch (e) {
      return json;
    }
  } catch (e) {
    return "";
  }
}

function saveElementScreenshot(tab, sel, path) {
  var jsonStr = floorp.tabElementScreenshot(tab, sel);
  var obj = JSON.parse(jsonStr);
  if (obj.image) {
    // Strip "data:image/png;base64," prefix. Safe split.
    var parts = obj.image.split(",");
    var b64Data = parts.length > 1 ? parts[1] : parts[0];
    try {
      excel.saveBase64Image(path, b64Data);
    } catch (e) {
      throw new Error("excel.saveBase64Image failed: " + e);
    }
  }
}

function parseViews(str) {
  if (!str) return 0;
  str = str.toString().trim().toLowerCase();
  str = str.replace(/views|回視聴|回再生|,| |　/g, "");
  var mult = 1;
  if (str.indexOf("万") !== -1) {
    mult = 10000;
    str = str.replace("万", "");
  } else if (str.indexOf("億") !== -1) {
    mult = 100000000;
    str = str.replace("億", "");
  } else if (str.indexOf("k") !== -1) {
    mult = 1000;
    str = str.replace("k", "");
  } else if (str.indexOf("m") !== -1) {
    mult = 1000000;
    str = str.replace("m", "");
  }
  var val = parseFloat(str);
  return isNaN(val) ? 0 : val * mult;
}

function sleep(tab, ms) {
  // Robust blocking sleep using Atomics
  try {
    var sab = new SharedArrayBuffer(4);
    var int32 = new Int32Array(sab);
    Atomics.wait(int32, 0, 0, ms);
  } catch (e) {
    var start = Date.now();
    while (Date.now() - start < ms) {}
  }
}

function uploadFileToSlack(filePath, slackUrl, message) {
  console.log("--- Uploading Excel to Slack ---");
  var createdTab = false;
  var tabId = null;

  try {
    // Find existing Slack tab
    var tabsResponse = floorp.browserTabs();
    var tabsData = JSON.parse(tabsResponse);
    var tabs = tabsData.tabs || tabsData;

    var slackTab = null;
    for (var i = 0; i < tabs.length; i++) {
      var url = tabs[i].url || "";
      if (
        url.indexOf("slack.com") !== -1 ||
        url.indexOf("app.slack.com") !== -1
      ) {
        slackTab = tabs[i];
        console.log("Found Slack tab: " + (tabs[i].title || "Slack"));
        break;
      }
    }

    if (!slackTab) {
      console.log("No Slack tab found. Opening Slack...");
      var createResult = floorp.createTab(slackUrl, false);
      try {
        var createData = JSON.parse(createResult);
        tabId = String(createData.instance_id || createData.id);
      } catch (e) {
        tabId = String(createResult);
      }
      createdTab = true;

      // Wait for load
      floorp.tabWaitForNetworkIdle(tabId, "15000");
      floorp.tabWaitForElement(tabId, "[data-qa='channel_sidebar']", 10000);

      slackTab = {
        instance_id: tabId,
        url: slackUrl,
        title: "Slack",
      };
    }

    // Attach to Slack tab
    tabId = String(slackTab.instance_id || slackTab.id);
    floorp.attachToTab(tabId);

    // Upload file + send message
    var inputSelector = '[role="textbox"] p';
    var fileInputSelector = 'input[data-qa="file_upload"]';
    var sendButtonSelector = '[data-qa="texty_send_button"]';

    floorp.tabWaitForElement(tabId, fileInputSelector, 5000);
    floorp.tabUploadFile(tabId, fileInputSelector, filePath);
    sleep(tabId, 1500);

    if (message) {
      floorp.tabWaitForElement(tabId, inputSelector, 5000);
      floorp.tabSetInnerHTML(tabId, inputSelector, message);
      sleep(tabId, 800);
      // floorp.tabClick(tabId, sendButtonSelector);
    }

    console.log("Slack upload complete: " + filePath);
  } finally {
    if (tabId) {
      try {
        floorp.destroyTabInstance(tabId);
      } catch (e) {}

      if (createdTab) {
        try {
          floorp.closeTab(tabId);
        } catch (e2) {}
      }
    }
  }
}
