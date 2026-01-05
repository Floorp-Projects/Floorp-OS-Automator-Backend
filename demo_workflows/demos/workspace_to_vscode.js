/**
 * Workspace to VSCode Workflow
 *
 * このワークフローは以下の処理を行います:
 * 1. Floorpで開いているタブを取得
 * 2. タブのURLからGitHub/GitLabリポジトリ名を抽出
 * 3. 対応するローカルフォルダをVSCodeで開く
 * 4. 不要なVSCodeウィンドウを閉じる
 */

// 設定: リポジトリのベースディレクトリ
const REPO_BASE_DIRS = [
  "/Users/user/dev-source/floorp-dev",
  "/Users/user/dev-source/sapphillon-dev",
  "/Users/user/dev-source",
];

// GitHub owner to local directory mapping
const OWNER_MAPPING = {
  "floorp-projects": "floorp-dev",
  sapphillon: "sapphillon-dev",
};

function workflow() {
  try {
    // Step 1: ブラウザタブを取得
    console.log("Step 1: Getting browser tabs...");
    const tabsResponse = floorp.browserTabs();
    const tabsData = JSON.parse(tabsResponse);
    const tabs = tabsData.tabs || tabsData;
    console.log("Found " + tabs.length + " tabs");

    // Step 2: タブのURLからプロジェクトパスを推測
    console.log("Step 2: Analyzing tab URLs...");
    const projectPaths = [];

    for (const tab of tabs) {
      const url = tab.url || "";
      const projectPath = extractProjectPath(url);
      if (projectPath && !projectPaths.includes(projectPath)) {
        projectPaths.push(projectPath);
        console.log("Found project path: " + projectPath);
      }
    }

    if (projectPaths.length === 0) {
      return {
        ok: false,
        message: "No project paths could be extracted from tabs",
      };
    }

    // Step 3: VSCodeでプロジェクトを開く
    console.log("Step 3: Opening projects in VSCode...");
    const opened = [];
    const failed = [];

    for (const path of projectPaths) {
      try {
        const result = vscode.open_folder(path);
        console.log("Opened: " + path + " - " + result);
        opened.push(path);
      } catch (e) {
        console.log("Failed to open: " + path + " - " + String(e));
        failed.push(path);
      }
    }

    // Step 4: AIを使って不要なウィンドウを閉じる
    console.log("Step 4: Analyzing windows with AI...");
    let closedCount = 0;

    try {
      const allTitles = get_inactive_window_titles();
      console.log("Found " + allTitles.length + " inactive windows");

      if (allTitles.length > 0) {
        // デバッグ: 全ウィンドウをログ
        console.log("=== All window titles ===");
        for (let i = 0; i < allTitles.length; i++) {
          console.log("[" + i + "] " + allTitles[i]);
        }
        console.log("=========================");

        // AIにウィンドウを分析してもらう
        const windowsJson = JSON.stringify(allTitles);
        console.log("Sending to AI for analysis...");
        const toCloseJson = iniad.analyzeWindows(windowsJson);
        console.log("AI response: " + toCloseJson);

        try {
          const windowsToClose = JSON.parse(toCloseJson);
          console.log("=== Windows AI wants to close ===");

          // 保護するアプリのリスト
          const protectedApps = [
            "floorp",
            "firefox",
            "chrome",
            "safari",
            "edge",
            "code",
            "vscode",
            "visual studio",
            "cursor",
            "terminal",
            "iterm",
            "warp",
            "alacritty",
            "antigravity",
            "windowmanager",
            "dock",
            "finder",
            "systemuiserver",
            "spotlight",
            "controlcenter",
          ];

          for (const titleToClose of windowsToClose) {
            const lowerTitle = titleToClose.toLowerCase();

            // 保護対象かチェック
            let isProtected = false;
            for (const app of protectedApps) {
              if (lowerTitle.includes(app)) {
                isProtected = true;
                console.log("[SKIP] Protected: " + titleToClose);
                break;
              }
            }

            if (!isProtected) {
              console.log("[CLOSING] " + titleToClose);
              try {
                close_window(titleToClose);
                closedCount++;
              } catch (e) {
                console.log(
                  "Failed to close: " + titleToClose + " - " + String(e)
                );
              }
            }
          }
          console.log("=================================");
        } catch (parseError) {
          console.log("Failed to parse AI response: " + parseError);
        }
      }
    } catch (e) {
      console.log("Could not get window list: " + String(e));
    }

    console.log("Closed " + closedCount + " non-development windows");

    return {
      ok: true,
      message: "Workflow completed",
      openedProjects: opened,
      failedProjects: failed,
      closedWindows: closedCount,
    };
  } catch (e) {
    return {
      ok: false,
      message: "Workflow failed",
      error: String(e),
    };
  }
}

function extractProjectPath(url) {
  try {
    // GitHub URL: https://github.com/owner/repo
    if (url.includes("github.com")) {
      const match = url.match(/github\.com\/([^\/]+)\/([^\/?#]+)/);
      if (match) {
        const owner = match[1].toLowerCase();
        const repo = match[2];

        const localDir = OWNER_MAPPING[owner];
        if (localDir) {
          return "/Users/user/dev-source/" + localDir + "/" + repo;
        }

        for (const baseDir of REPO_BASE_DIRS) {
          return baseDir + "/" + repo;
        }
      }
    }

    // GitLab URL: https://gitlab.com/owner/repo
    if (url.includes("gitlab.com")) {
      const match = url.match(/gitlab\.com\/([^\/]+)\/([^\/?#]+)/);
      if (match) {
        const repo = match[2];
        return REPO_BASE_DIRS[0] + "/" + repo;
      }
    }

    return null;
  } catch (e) {
    return null;
  }
}

workflow();
