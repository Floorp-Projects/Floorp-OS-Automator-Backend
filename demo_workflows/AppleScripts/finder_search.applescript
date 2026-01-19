on run argv
	-- ============================================================
	-- 1. 設定チェックと引数処理
	-- ============================================================

	if (count of argv) is less than 2 then
		return "エラー: 引数が足りません。パスと検索キーワードを指定してください。"
	end if

	set targetPath to item 1 of argv
	set searchKeyword to item 2 of argv

	-- 検索範囲の自動修正（前回と同じロジック）
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

	-- ============================================================
	-- 2. メイン処理：検索コマンドの強化
	-- ============================================================

	-- mdfindの結果から、パッケージの中身（.xcodeproj/ や .app/ の中）を除外するフィルタを追加
	-- grep -vE '\.(xcodeproj|app|xcworkspace|framework|bundle)/' は、
	-- パスの中に「.xcodeproj/」などが含まれる行を結果から削除します。
	set filterCommand to " | grep -vE '\\.(xcodeproj|app|xcworkspace|framework|bundle|plugin)/'"

	set shellCommand to "mdfind -onlyin " & quoted form of targetPath & " -name " & quoted form of searchKeyword & filterCommand

	-- ============================================================
	-- 3. Finder UI操作（視覚フィードバック）
	-- ============================================================
	tell application "Finder"
		activate
		try
			set targetFolder to POSIX file targetPath as alias
			open targetFolder
		on error
			return "エラー: 指定されたフォルダが開けませんでした。"
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

	-- ============================================================
	-- 4. 結果の整形と返却
	-- ============================================================

	try
		set searchResult to do shell script shellCommand

		if searchResult is "" then
			return "見つかりませんでした"
		else
			-- 【重要】AppleScriptの改行(\r)を、ターミナル用の改行(\n)に変換して返す
			-- これで出力が一行に重なるバグを防ぎます
			set originalDelimiters to AppleScript's text item delimiters
			set AppleScript's text item delimiters to return -- \r
			set pathList to text items of searchResult
			set AppleScript's text item delimiters to linefeed -- \n
			set cleanResult to pathList as text
			set AppleScript's text item delimiters to originalDelimiters

			return cleanResult
		end if

	on error errStr
		return "検索実行エラー: " & errStr
	end try
end run