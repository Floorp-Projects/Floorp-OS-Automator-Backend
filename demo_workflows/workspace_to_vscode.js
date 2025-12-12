/**
 * Demo Workflow 2: Workspace Tabs to VSCode Projects
 * 
 * このワークフローは以下の処理を行います:
 * 1. Floorpで現在のワークスペースとそのタブを取得
 * 2. タブのURLからプロジェクトのパスを推測
 * 3. 関連するVSCodeプロジェクトを開く
 * 4. 関係ないウィンドウを閉じる
 * 
 * URL to Path マッピングのルール:
 * - GitHub/GitLab URL → 設定されたローカルディレクトリ内の対応するリポジトリ
 * - localhost URL → 設定されたプロジェクトディレクトリ
 */

// 設定: ユーザーがカスタマイズ可能
const CONFIG = {
    // リポジトリのベースディレクトリ
    repoBaseDir: "F:\\sapphillon-dev",
    // プロジェクトディレクトリのマッピング (localhost port -> project path)
    portMapping: {
        "3000": "F:\\sapphillon-dev\\Floorp-Projects\\floorp-browser",
        "8080": "F:\\sapphillon-dev\\Floorp-OS-Automator-Backend",
        "5173": "F:\\sapphillon-dev\\Floorp-Projects\\floorp-browser"
    },
    // 閉じるウィンドウのパターン（これらを含むタイトルを閉じる）
    closeWindowPatterns: []
};

function workflow() {
    try {
        // Step 1: ワークスペース情報を取得
        console.log("Step 1: Getting current workspace...");
        const workspaceResponse = floorp.getCurrentWorkspace();
        const workspace = JSON.parse(workspaceResponse);
        console.log("Current workspace: " + JSON.stringify(workspace));

        // Step 2: ブラウザタブを取得
        console.log("Step 2: Getting browser tabs...");
        const tabsResponse = floorp.browserTabs();
        const tabsData = JSON.parse(tabsResponse);
        const tabs = tabsData.tabs || tabsData;
        console.log("Found " + tabs.length + " tabs");
        
        // Step 3: タブのURLからプロジェクトパスを推測
        console.log("Step 3: Analyzing tab URLs...");
        const projectPaths = [];
        
        for (const tab of tabs) {
            const url = tab.url || "";
            console.log("Analyzing tab: " + url);
            
            const projectPath = extractProjectPath(url);
            if (projectPath && !projectPaths.includes(projectPath)) {
                projectPaths.push(projectPath);
                console.log("Found project path: " + projectPath);
            }
        }

        if (projectPaths.length === 0) {
            console.log(JSON.stringify({
                ok: false,
                reason: "No project paths could be extracted from tabs"
            }));
            return;
        }

        // Step 4: VSCodeでプロジェクトを開く
        console.log("Step 4: Opening projects in VSCode...");
        for (const path of projectPaths) {
            try {
                vscode.open_folder(path);
                console.log("Opened: " + path);
            } catch (e) {
                console.log("Failed to open: " + path + " - " + String(e));
            }
        }
        
        // Step 5: 関係ないウィンドウを閉じる
        console.log("Step 5: Closing unrelated windows...");
        let closedCount = 0;
        
        for (const pattern of CONFIG.closeWindowPatterns) {
            try {
                const result = close_window(pattern);
                console.log("Closed windows matching: " + pattern + " - " + result);
                closedCount++;
            } catch (e) {
                // ウィンドウが見つからない場合は無視
            }
        }
        
        console.log(JSON.stringify({ 
            ok: true, 
            message: "Workflow completed",
            openedProjects: projectPaths,
            closedWindowPatterns: closedCount
        }));
        
    } catch (e) {
        console.log(JSON.stringify({ 
            ok: false, 
            reason: "Workflow failed", 
            error: String(e) 
        }));
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
                const owner = match[1];
                const repo = match[2];
                // Floorp-Projects のリポジトリはそのディレクトリに、
                // その他はベースディレクトリ直下と仮定
                if (owner.toLowerCase() === "floorp-projects") {
                    return CONFIG.repoBaseDir + "\\Floorp-Projects\\" + repo;
                }
                return CONFIG.repoBaseDir + "\\" + repo;
            }
        }

        // GitLab URL: https://gitlab.com/owner/repo
        if (url.includes("gitlab.com")) {
            const match = url.match(/gitlab\.com\/([^\/]+)\/([^\/?#]+)/);
            if (match) {
                const repo = match[2];
                return CONFIG.repoBaseDir + "\\" + repo;
            }
        }

        // localhost URL: http://localhost:PORT
        if (url.includes("localhost")) {
            const match = url.match(/localhost:(\d+)/);
            if (match) {
                const port = match[1];
                if (CONFIG.portMapping[port]) {
                    return CONFIG.portMapping[port];
                }
            }
        }

        // file:// URL: file:///path/to/project
        if (url.startsWith("file://")) {
            const match = url.match(/file:\/\/\/([^?#]+)/);
            if (match) {
                const filePath = decodeURIComponent(match[1]);
                // ファイルパスからプロジェクトルートを推測（最初の3階層まで）
                const parts = filePath.split(/[\/\\]/);
                if (parts.length >= 3) {
                    return parts.slice(0, 3).join("\\");
                }
            }
        }

        return null;
    } catch (e) {
        return null;
    }
}

workflow();
