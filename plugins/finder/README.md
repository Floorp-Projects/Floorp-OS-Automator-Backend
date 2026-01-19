# Finder Plugin

macOS Finder 検索プラグイン。**AppleScript のみ**を使用。

## エクスポート関数

`globalThis.app.sapphillon.core.finder.findFiles(rootPath, query, maxResults)`

- AppleScript でファイル検索を実行
- Finder UI を開いて検索クエリを入力
- JSON 配列（ファイルパスのリスト）を返す

## 必要な権限

- **アクセシビリティ/オートメーション権限**: System Preferences → Privacy & Security → Accessibility
