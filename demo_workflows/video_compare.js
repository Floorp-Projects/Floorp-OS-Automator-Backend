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

  console.log("Starting Video Comparison for topic: " + topic);

  // --- 1. YouTube Scraping ---
  console.log("--- Scraping YouTube (Top 50) ---");
  var ytUrl = "https://www.youtube.com/results?search_query=" + topic;
  var ytTab = null;
  try {
    ytTab = floorp.createTab(ytUrl, false);

    // Initial wait
    try {
      floorp.tabWaitForElement(ytTab, "ytd-video-renderer #video-title", 10000);
      floorp.tabWaitForNetworkIdle(ytTab, 2000);
    } catch (e) {
      /* ignore */
    }

    // Scroll Loop to load more items (Lazy Loading) - Aiming for 100 items
    console.log("   Scrolling to load more items (Targeting Loader)...");

    for (var s = 0; s < 12; s++) {
      // Scroll loader into view to trigger next batch
      try {
        floorp.tabScrollTo(ytTab, "ytd-continuation-item-renderer");
      } catch (e) {
        // Fallback: Scroll to the last video we probably have loaded
        try {
          var index = (s + 1) * 10; // rough guess
          floorp.tabScrollTo(
            ytTab,
            "ytd-video-renderer:nth-of-type(" + index + ")"
          );
        } catch (ex) {}
      }
      sleep(ytTab, 3000); // 3s wait for load
    }

    // Nested Loop Strategy: Section -> Videos
    // We expect multiple sections due to infinite scroll
    var totalNeeded = 100;
    var currentRank = 1;

    for (var sec = 1; sec <= 20; sec++) {
      if (results.length >= totalNeeded) break;

      var sectionSel = "ytd-item-section-renderer:nth-of-type(" + sec + ")";

      for (var vid = 1; vid <= 50; vid++) {
        if (results.length >= totalNeeded) break;

        // Selector for video INSIDE section
        var baseSel =
          sectionSel + " ytd-video-renderer:nth-of-type(" + vid + ")";
        var titleSel = baseSel + " #video-title";
        var viewSel = baseSel + " #metadata-line span:nth-of-type(1)";
        var thumbSel = baseSel + " ytd-thumbnail";

        try {
          var title = getText(ytTab, titleSel);

          // If title is missing, it might be an ad or end of section.
          // Check if the NEXT video element exists/has title to decide if we should break.
          if (!title) {
            // Look ahead to vid+1 and vid+2 to be safe against ads/shorts gaps
            var nextCheck = getText(
              ytTab,
              sectionSel +
                " ytd-video-renderer:nth-of-type(" +
                (vid + 1) +
                ") #video-title"
            );
            if (!nextCheck) {
              // Double check (vid+2)
              var nextNext = getText(
                ytTab,
                sectionSel +
                  " ytd-video-renderer:nth-of-type(" +
                  (vid + 2) +
                  ") #video-title"
              );
              if (!nextNext) {
                break; // Assume end of section
              }
            }
            continue; // It was a gap/ad, but section continues
          }

          var viewRaw = getText(ytTab, viewSel);

          // Thumbnail
          var thumbPath = "";
          try {
            var tmp = "/tmp/yt_thumb_" + currentRank + ".png";
            saveElementScreenshot(ytTab, thumbSel, tmp);
            thumbPath = tmp;
          } catch (e) {}

          results.push({
            platform: "YouTube",
            rank: currentRank++,
            title: title,
            viewsRaw: viewRaw,
            views: parseViews(viewRaw),
            url: "https://www.youtube.com", // Generic link
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
          var tmp = "/tmp/nico_thumb_" + i + ".png";
          saveElementScreenshot(nicoTab, baseSel + " img", tmp);
          thumbPath = tmp;
        } catch (e) {}

        if (title) {
          results.push({
            platform: "Niconico",
            rank: i,
            title: title,
            viewsRaw: viewRaw,
            views: parseViews(viewRaw),
            url: "https://www.nicovideo.jp",
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

  // --- 3. Excel Export ---
  console.log("Creating Excel: " + excelPath);
  try {
    // Columns: "Platform", "Rank", "Image", "Title", "Views", "Raw Views", "Link"
    var matrix = [
      ["Platform", "Rank", "Image", "Title", "Views", "Raw Views", "Link"],
    ];
    for (var k = 0; k < results.length; k++) {
      var r = results[k];
      matrix.push([
        String(r.platform),
        String(r.rank),
        "",
        String(r.title),
        String(r.views),
        String(r.viewsRaw),
        String(r.url),
      ]);
    }

    excel.writeRange(excelPath, "Sheet1", "A1", matrix);
    console.log("Data written.");

    // Insert Thumbnails
    for (var k = 0; k < results.length; k++) {
      var r = results[k];
      if (r.thumbPath) {
        try {
          // Approximate position.
          // Row 1 is header. Data starts Row 2.
          // Top = 20 + k*15.
          // To prevent heavy overlap with 50 items, maybe resize rows?
          // We can't resize rows yet.
          // Just insert.
          var topPos = 20 + k * 15;

          excel.insertPicture(excelPath, "Sheet1", r.thumbPath, {
            left: 100, // Col C
            top: topPos,
            width: 40,
            height: 25,
          });
        } catch (e) {}
      }
    }

    // Create Chart
    // var totalRows = results.length + 1;
    // var range = "D1:E" + totalRows;

    // excel.createChart(
    //   excelPath,
    //   "Sheet1",
    //   range,
    //   "bar",
    //   "Floorp Video Views",
    //   { left: 400, top: 50, width: 800, height: 800 } // Taller chart for more items
    // );
    // console.log("Chart created.");
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
    var binStr = atob(obj.image);
    var len = binStr.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) {
      bytes[i] = binStr.charCodeAt(i);
    }
    Deno.writeFileSync(path, bytes);
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
