/**
 * Workspace to VSCode Workflow
 *
 * このワークフローは以下の処理を行います:
 * 1. Floorpで開いているタブを取得
 * 2. タブのURLからGitHub/GitLabリポジトリ名を抽出
 * 3. 対応するローカルフォルダをVSCodeで開く
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

    // Step 4: 不要なVSCodeウィンドウを閉じる
    console.log("Step 4: Closing unused VSCode windows...");
    let closedCount = 0;

    try {
      // Get all window titles
      const allTitles = get_inactive_window_titles();
      console.log("Found " + allTitles.length + " inactive windows");

      for (const title of allTitles) {
        // Check if this is a VSCode window
        if (title.includes("Visual Studio Code") || title.includes("- Code")) {
          // Check if it's NOT related to any of our projects
          let isRelated = false;
          for (const projectPath of projectPaths) {
            // Extract folder name from path
            const folderName = projectPath.split("/").pop();
            if (title.includes(folderName)) {
              isRelated = true;
              break;
            }
          }

          if (!isRelated) {
            console.log("Closing unrelated VSCode window: " + title);
            try {
              close_window(title);
              closedCount++;
            } catch (e) {
              console.log("Failed to close: " + title + " - " + String(e));
            }
          }
        }
      }
    } catch (e) {
      console.log("Could not get window list: " + String(e));
    }

    console.log("Closed " + closedCount + " unused VSCode windows");

    return {
      ok: true,
      message: "Workflow completed",
      openedProjects: opened,
      failedProjects: failed,
    };
  } catch (e) {
    return {
      ok: false,
      message: "Workflow failed",
      error: String(e),
    };
  }
}

/**
 * URLからプロジェクトパスを抽出する
 * @param {string} url タブのURL
 * @returns {string|null} プロジェクトパス
 */
function extractProjectPath(url) {
  try {
    // GitHub URL: https://github.com/owner/repo
    if (url.includes("github.com")) {
      const match = url.match(/github\.com\/([^\/]+)\/([^\/?#]+)/);
      if (match) {
        const owner = match[1].toLowerCase();
        const repo = match[2];

        // Check owner mapping
        const localDir = OWNER_MAPPING[owner];
        if (localDir) {
          return "/Users/user/dev-source/" + localDir + "/" + repo;
        }

        // Try to find the repo in known directories
        for (const baseDir of REPO_BASE_DIRS) {
          const possiblePath = baseDir + "/" + repo;
          // We'll try this path
          return possiblePath;
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
