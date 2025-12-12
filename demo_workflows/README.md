# Demo Workflows

このディレクトリには、Sapphillon のデモ用ワークフローが含まれています。

## ワークフロー一覧

### 1. vscode_to_floorp_form.js

**概要**: VSCode でアクティブに表示されているファイルの内容を取得し、Floorp で開いているフォームに自動入力します。

**使用例**:

- コードレビューのためにコードをフォームに貼り付ける
- ドキュメントを Web フォームに入力する

**前提条件**:

- VSCode でファイルを開いている状態
- Floorp OS サーバーが起動している
- Floorp でフォームがあるページ（textarea/input フィールド）を開いている

### 2. workspace_to_vscode.js

**概要**: Floorp で現在開いているワークスペースのタブの URL を分析し、関連する VSCode プロジェクトを自動的に開きます。

**使用例**:

- GitHub でリポジトリを見ている時に、対応するローカルプロジェクトを開く
- 開発中の localhost ページから、そのプロジェクトを VSCode で開く

**設定** (`CONFIG`オブジェクトをカスタマイズ):

- `repoBaseDir`: リポジトリのベースディレクトリ
- `portMapping`: localhost ポートからプロジェクトパスへのマッピング
- `closeWindowPatterns`: 閉じるウィンドウのタイトルパターン

## ワークフローの実行方法

これらのワークフローは、Sapphillon の Workflow API を通じて実行できます。

```bash
# サーバーを起動
cargo run -- --db-url "sqlite://target/sqlite.db" start

# gRPC APIを通じてワークフローを生成・実行
```

## 使用するプラグイン関数

### VSCode Plugin

- `vscode.get_active_file_content()` - アクティブファイルの内容を取得
- `vscode.open_folder(path)` - フォルダを開く
- `vscode.close_workspace()` - ワークスペースを閉じる

### Floorp Plugin

- `floorp.listBrowserTabs()` - タブ一覧を取得
- `floorp.browserTabs()` - 詳細なタブ情報を取得
- `floorp.tabFillForm(id, selector, value)` - フォームに入力
- `floorp.getCurrentWorkspace()` - 現在のワークスペースを取得
- `floorp.attachToTab(id)` - タブにアタッチ

### Window Plugin

- `close_window(title)` - タイトルに一致するウィンドウを閉じる
- `get_active_window_title()` - アクティブウィンドウのタイトルを取得
- `get_inactive_window_titles()` - 非アクティブウィンドウのタイトル一覧
