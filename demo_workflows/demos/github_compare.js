/**
 * GitHub Repository Comparison Workflow
 */

function workflow() {
  var repos = [
    { name: "Floorp", url: "https://github.com/Floorp-Projects/Floorp" },
    { name: "Zen Browser", url: "https://github.com/zen-browser/desktop" },
    { name: "Waterfox", url: "https://github.com/BrowserWorks/waterfox" },
    { name: "Pulse Browser", url: "https://github.com/pulse-browser/browser" },
    {
      name: "Firefox (Mirror)",
      url: "https://github.com/mozilla-firefox/firefox",
    },
    { name: "Midori", url: "https://github.com/goastian/midori-desktop" },
    {
      name: "FireDragon",
      url: "https://gitlab.com/garuda-linux/firedragon/firedragon12",
    },
  ];

  var data = [];
  console.log("Starting Browser Repository Comparison...");

  for (var i = 0; i < repos.length; i++) {
    var repo = repos[i];
    console.log("Scraping data for: " + repo.name);

    var tabId = null;
    try {
      tabId = floorp.createTab(repo.url, false);
      console.log("   Opened tab: " + tabId);

      var isGitLab = repo.url.indexOf("gitlab.com") !== -1;
      var starSelector = "";
      var forkSelector = "";

      if (isGitLab) {
        starSelector = "a.star-count span, [data-testid='star-count'] span";
        forkSelector = "a.forks, [data-testid='fork-count'], a[href$='/forks']";
      } else {
        starSelector = "#repo-stars-counter-star";
        forkSelector = "#repo-network-counter";
      }

      console.log("   Waiting for elements...");
      try {
        floorp.tabWaitForElement(tabId, starSelector, 10000);
      } catch (e) {
        console.log("   Timeout waiting for star element.");
      }

      var stars = "0";
      var forks = "0";

      try {
        var starJson = floorp.tabElementText(tabId, starSelector);
        try {
          var sObj = JSON.parse(starJson);
          stars = sObj.text || "0";
        } catch (e) {
          stars = starJson;
        }
      } catch (e) {
        console.log("   Could not find Star count.");
      }

      try {
        var forkJson = floorp.tabElementText(tabId, forkSelector);
        try {
          var fObj = JSON.parse(forkJson);
          forks = fObj.text || "0";
        } catch (e) {
          forks = forkJson;
        }
      } catch (e) {
        console.log("   Could not find Fork count.");
      }

      console.log("   Raw Stars: " + stars + ", Raw Forks: " + forks);

      // Simple parse function
      var val = 0;
      // Stars
      var sStr = stars ? stars.toString().trim().toLowerCase() : "0";
      var sMult = 1;
      if (sStr.endsWith("k")) {
        sMult = 1000;
        sStr = sStr.substring(0, sStr.length - 1);
      } else if (sStr.endsWith("m")) {
        sMult = 1000000;
        sStr = sStr.substring(0, sStr.length - 1);
      }
      val = parseFloat(sStr.replace(/[,+]/g, "")) * sMult;
      var starsNum = val || 0;

      // Forks
      var fStr = forks ? forks.toString().trim().toLowerCase() : "0";
      var fMult = 1;
      if (fStr.endsWith("k")) {
        fMult = 1000;
        fStr = fStr.substring(0, fStr.length - 1);
      } else if (fStr.endsWith("m")) {
        fMult = 1000000;
        fStr = fStr.substring(0, fStr.length - 1);
      }
      val = parseFloat(fStr.replace(/[,+]/g, "")) * fMult;
      var forksNum = val || 0;

      data.push({
        name: repo.name,
        stars: starsNum,
        forks: forksNum,
      });
    } catch (e) {
      console.log("   Failed to scrape " + repo.name + ": " + e);
      data.push({ name: repo.name, stars: 0, forks: 0 });
    } finally {
      if (tabId) {
        floorp.destroyTabInstance(tabId);
      }
    }
  }

  console.log("Collected Data: " + JSON.stringify(data));

  // Excel
  var excelPath = "/Users/user/Desktop/Browser_Comparison.xlsx";
  console.log("Creating Excel file: " + excelPath);

  try {
    // Prepare data matrix
    var matrix = [["Browser", "Stars", "Forks"]];
    for (var j = 0; j < data.length; j++) {
      // Serialize to strings because Rust backend expects Vec<Vec<String>>
      matrix.push([
        String(data[j].name),
        String(data[j].stars),
        String(data[j].forks),
      ]);
    }

    // Write all data at once (Creates new workbook)
    excel.writeRange(excelPath, "Sheet1", "A1", matrix);

    console.log("   Data written to Sheet1.");

    var totalRows = data.length + 1;
    var dataRange = "A1:C" + totalRows;

    console.log("   Generating chart for range: " + dataRange);
    excel.createChart(
      excelPath,
      "Sheet1",
      dataRange,
      "column",
      "Browser Popularity",
      // AppleScript needs integer/float
      { left: 300, top: 50, width: 800, height: 450 }
    );

    console.log("Workflow completed successfully!");
  } catch (e) {
    console.log("Excel error: " + e);
  }
}
