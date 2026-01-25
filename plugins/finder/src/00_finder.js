// ============================================================
// Finder Plugin - AppleScript-based file search
// ============================================================

// Embedded AppleScript (same as finder_search.applescript)
// バックグラウンド検索（Finder UI を使わず mdfind のみ）- 高速で安定
var FINDER_SEARCH_APPLESCRIPT_BACKGROUND = `on run argv
	if (count of argv) is less than 2 then
		return ""
	end if

	set targetPath to item 1 of argv
	set searchKeyword to item 2 of argv

	set filterCommand to " | grep -vE '\\\\.(xcodeproj|app|xcworkspace|framework|bundle|plugin)/'"
	set shellCommand to "mdfind -onlyin " & quoted form of targetPath & " -name " & quoted form of searchKeyword & filterCommand

	-- 結果の整形と返却（UI操作なし）
	try
		set searchResult to do shell script shellCommand
		if searchResult is "" then
			return ""
		else
			set originalDelimiters to AppleScript's text item delimiters
			set AppleScript's text item delimiters to return
			set pathList to text items of searchResult
			set AppleScript's text item delimiters to linefeed
			set cleanResult to pathList as text
			set AppleScript's text item delimiters to originalDelimiters
			return cleanResult
		end if
	on error
		return ""
	end try
end run`;

// Finder UI を使用した検索（改善版 - ウィンドウ管理を修正）
var FINDER_SEARCH_APPLESCRIPT_UI = `on run argv
	if (count of argv) is less than 2 then
		return ""
	end if

	set targetPath to item 1 of argv
	set searchKeyword to item 2 of argv

	-- 検索範囲の自動修正（現在のフォルダ内を検索）
	try
		set currentScope to do shell script "defaults read com.apple.finder FXDefaultSearchScope"
	on error
		set currentScope to ""
	end try
	if currentScope is not "SCcf" then
		do shell script "defaults write com.apple.finder FXDefaultSearchScope -string 'SCcf'"
		tell application "Finder" to quit
		delay 1.0
		tell application "Finder" to activate
		delay 1.5
	end if

	set filterCommand to " | grep -vE '\\\\.(xcodeproj|app|xcworkspace|framework|bundle|plugin)/'"
	set shellCommand to "mdfind -onlyin " & quoted form of targetPath & " -name " & quoted form of searchKeyword & filterCommand

	-- 検索開始前に既存の Finder ウィンドウをすべて閉じる
	tell application "Finder"
		try
			close every Finder window
		end try
		delay 0.3
	end tell

	-- Finder UI操作（1つのウィンドウのみ使用）
	tell application "Finder"
		activate
		delay 0.5
		try
			set targetFolder to POSIX file targetPath as alias
			-- 新規ウィンドウを1つだけ開く
			open targetFolder
			delay 0.8
		on error errMsg
			return ""
		end try
	end tell

	-- System Events で検索フィールドを操作
	tell application "System Events"
		tell process "Finder"
			set frontmost to true
			delay 0.8

			-- Cmd+F で検索フィールドを開く
			keystroke "f" using {command down}
			delay 1.5

			-- 検索フィールドにフォーカスがあることを確認
			delay 0.5

			-- 検索クエリを準備（name: プレフィックス付き）
			set uiSearchQuery to "name:" & searchKeyword

			-- クリップボードをクリアしてからコピー（確実性を高める）
			do shell script "pbcopy < /dev/null"
			delay 0.5
			set the clipboard to uiSearchQuery
			delay 0.5

			-- 検索フィールドに貼り付け
			keystroke "v" using {command down}
			delay 0.8

			-- Enter で検索実行
			key code 36
			delay 0.5
		end tell
	end tell

	-- 検索結果が表示されるまで少し待つ
	delay 1.5

	-- すべての Finder ウィンドウを閉じる
	tell application "Finder"
		try
			close every Finder window
		end try
	end tell

	-- 結果の整形と返却
	try
		set searchResult to do shell script shellCommand
		if searchResult is "" then
			return ""
		else
			set originalDelimiters to AppleScript's text item delimiters
			set AppleScript's text item delimiters to return
			set pathList to text items of searchResult
			set AppleScript's text item delimiters to linefeed
			set cleanResult to pathList as text
			set AppleScript's text item delimiters to originalDelimiters
			return cleanResult
		end if
	on error
		return ""
	end try
end run`;

// ============================================================================
// 動的パス判定（ハードコードなし）
// ============================================================================

// ホームディレクトリを動的に取得（キャッシュ）
var _cachedHomeDir = null;
function getHomeDirectory() {
  if (_cachedHomeDir) return _cachedHomeDir;
  try {
    // exec プラグインを使用して $HOME を取得
    if (
      app &&
      app.sapphillon &&
      app.sapphillon.core &&
      app.sapphillon.core.exec
    ) {
      var result = app.sapphillon.core.exec.exec("echo $HOME");
      if (result) {
        _cachedHomeDir = result.toString().trim();
        return _cachedHomeDir;
      }
    }
  } catch (e) {}
  // フォールバック: 一般的なパターンから推測
  return (
    "/Users/" +
    (typeof process !== "undefined" && process.env ? process.env.USER : "user")
  );
}

// macOS のローカライズ別名を持つ標準フォルダ（相対パス）
// これらのフォルダは Finder UI 操作が不安定なためバックグラウンドモードを使用
var MACOS_LOCALIZED_FOLDERS = [
  "Documents",
  "Desktop",
  "Downloads",
  "Pictures",
  "Movies",
  "Music",
  "Public",
  "Library/Mobile Documents",
];

// iCloud 関連のパスパターン
var ICLOUD_PATTERNS = ["com~apple~CloudDocs", "Mobile Documents"];

// パスが macOS ローカライズフォルダかどうかを判定
function isMacOSLocalizedFolder(path) {
  if (!path) return false;
  var homeDir = getHomeDirectory();

  // ホームディレクトリ直下のローカライズフォルダをチェック
  for (var i = 0; i < MACOS_LOCALIZED_FOLDERS.length; i++) {
    var localizedPath = homeDir + "/" + MACOS_LOCALIZED_FOLDERS[i];
    if (path.indexOf(localizedPath) === 0) {
      return true;
    }
  }

  // iCloud 関連パスをチェック
  for (var j = 0; j < ICLOUD_PATTERNS.length; j++) {
    if (path.indexOf(ICLOUD_PATTERNS[j]) !== -1) {
      return true;
    }
  }

  return false;
}

// パスが開発用ディレクトリかどうかを判定（UI モードを使用）
// 開発ディレクトリは通常英語名のためローカライズ問題がない
function isDevelopmentDirectory(path) {
  if (!path) return false;

  // 開発ディレクトリのパターン（一般的な命名規則）
  // スラッシュなしのパターンを使用して、パスの任意の場所にマッチするように
  var devPatterns = [
    "dev-source",
    "dev",
    "Developer",
    "Projects",
    "repos",
    "git",
    "src",
    "code",
    "workspace",
    "workspaces",
    "floorp",
  ];

  // パスを正規化（末尾のスラッシュを除去）
  var normalizedPath = path.replace(/\/$/, "");

  for (var i = 0; i < devPatterns.length; i++) {
    // パスの各パスコンポーネントをチェック
    var pathParts = normalizedPath.split("/");
    for (var j = 0; j < pathParts.length; j++) {
      if (pathParts[j] === devPatterns[i]) {
        return true;
      }
    }
  }

  return false;
}

// パスが UI モードを使用するかどうかを判定
function shouldUseUI(path) {
  if (!path) return false;

  // 1. 開発ディレクトリは UI モードを優先（デモ用）
  if (isDevelopmentDirectory(path)) {
    return true;
  }

  // 2. macOS ローカライズフォルダはバックグラウンドモード
  if (isMacOSLocalizedFolder(path)) {
    return false;
  }

  // 3. デフォルトはバックグラウンドモード（安定性重視）
  return false;
}

// AppleScript を実行してファイル検索
function findFiles(rootPath, query, maxResults) {
  try {
    // Floorp/Sapphillon プロジェクトは UI モード、それ以外はバックグラウンド
    var useUI = shouldUseUI(rootPath);
    var scriptToUse = useUI
      ? FINDER_SEARCH_APPLESCRIPT_UI
      : FINDER_SEARCH_APPLESCRIPT_BACKGROUND;

    // デバッグログ: どのモードが選択されたか
    console.log(
      "  [Finder] Path: " +
        rootPath +
        " | Mode: " +
        (useUI ? "UI (Finder visible)" : "Background (mdfind only)"),
    );

    // 一時ファイルに書き出して実行
    var tempPath = "/tmp/sapphillon_finder_" + Date.now() + ".applescript";
    app.sapphillon.core.filesystem.write(tempPath, scriptToUse);

    var cmd =
      "osascript " +
      JSON.stringify(tempPath) +
      " " +
      JSON.stringify(rootPath || "/") +
      " " +
      JSON.stringify(query || "");
    var out = app.sapphillon.core.exec.exec(cmd) || "";

    // 一時ファイル削除
    try {
      app.sapphillon.core.exec.exec("rm -f " + JSON.stringify(tempPath));
    } catch (e) {}

    // 改行コード正規化
    var normalized = out.toString().replace(/\r\n/g, "\n").replace(/\r/g, "\n");
    var lines = normalized
      .split("\n")
      .map(function (l) {
        return l.trim();
      })
      .filter(function (l) {
        return l && l.length > 0;
      });

    if (maxResults && lines.length > maxResults) {
      lines = lines.slice(0, maxResults);
    }

    return JSON.stringify(lines);
  } catch (e) {
    return JSON.stringify([]);
  }
}

// Expose
globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};
globalThis.app.sapphillon.core.finder =
  globalThis.app.sapphillon.core.finder || {};
globalThis.app.sapphillon.core.finder.findFiles = findFiles;
