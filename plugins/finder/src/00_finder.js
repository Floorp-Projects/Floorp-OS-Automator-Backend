// ============================================================
// Finder Plugin - AppleScript-based file search
// ============================================================

// Embedded AppleScript (same as finder_search.applescript)
var FINDER_SEARCH_APPLESCRIPT = `on run argv
	if (count of argv) is less than 2 then
		return ""
	end if

	set targetPath to item 1 of argv
	set searchKeyword to item 2 of argv

	-- 検索範囲の自動修正
	try
		set currentScope to do shell script "defaults read com.apple.finder FXDefaultSearchScope"
	on error
		set currentScope to ""
	end try
	if currentScope is not "SCcf" then
		do shell script "defaults write com.apple.finder FXDefaultSearchScope -string 'SCcf'"
		tell application "Finder" to quit
		delay 0.5
		tell application "Finder" to activate
		delay 1.0
	end if

	set filterCommand to " | grep -vE '\\\\.(xcodeproj|app|xcworkspace|framework|bundle|plugin)/'"
	set shellCommand to "mdfind -onlyin " & quoted form of targetPath & " -name " & quoted form of searchKeyword & filterCommand

	-- Finder UI操作
	tell application "Finder"
		activate
		try
			set targetFolder to POSIX file targetPath as alias
			open targetFolder
		on error
			return ""
		end try
	end tell

	tell application "System Events"
		tell process "Finder"
			set frontmost to true
			delay 0.4
			keystroke "f" using {command down}
			delay 0.3
			set uiSearchQuery to "name:" & searchKeyword
			set the clipboard to uiSearchQuery
			delay 0.1
			keystroke "v" using {command down}
			delay 0.2
			key code 36
		end tell
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

// AppleScript を実行してファイル検索
function findFiles(rootPath, query, maxResults) {
  try {
    // 一時ファイルに書き出して実行
    var tempPath = "/tmp/sapphillon_finder_" + Date.now() + ".applescript";
    app.sapphillon.core.filesystem.write(tempPath, FINDER_SEARCH_APPLESCRIPT);

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
