# External Plugin Integration Tests

このディレクトリには、外部プラグインのインストール、ロード、実行の完全なフローを検証する統合テストが含まれています。

## ディレクトリ構造

```
src/tests/
├── mod.rs                    # テストモジュール定義
├── external_plugin/          # 外部プラグインテストモジュール
│   ├── mod.rs                # モジュール定義
│   ├── common.rs             # 共通ヘルパー関数
│   ├── installation.rs       # インストール・ファイルシステムテスト
│   ├── bridge.rs             # rsjs_bridge_core実行テスト
│   ├── workflow.rs           # ワークフロー実行テスト
│   └── e2e.rs                # エンドツーエンドテスト
└── fixtures/                  # テスト用プラグインフィクスチャ
    ├── math_plugin.js        # 数学演算プラグイン
    ├── error_plugin.js       # エラーハンドリングテスト用プラグイン
    └── file_plugin.js        # ファイルシステム権限テスト用プラグイン
```

## テストモジュール

### 1. `common.rs` - 共通ヘルパー関数

すべてのテストで使用される共通ユーティリティ:

- `get_fixtures_dir()` - フィクスチャディレクトリのパスを取得
- `get_fixture_path(filename)` - 特定のフィクスチャファイルのパスを取得
- `read_fixture(filename)` - フィクスチャファイルの内容を読み込み
- `create_temp_plugin()` - 一時プラグインディレクトリを作成
- `create_opstate_with_package()` - テスト用OpStateを作成
- `scan_plugin_directory()` - プラグインディレクトリをスキャン

### 2. `installation.rs` - インストール・ファイルシステムテスト

**外部依存なし** - これらのテストは外部プラグインサーバーなしで実行可能:

- `test_plugin_installation_creates_directory_structure` - ディレクトリ構造の作成を検証
- `test_plugin_scan_finds_installed_plugins` - プラグインスキャン機能を検証
- `test_plugin_content_validation` - プラグインコンテンツの検証
- `test_multiple_plugin_versions` - 複数バージョンの共存を検証
- `test_plugin_overwrite` - プラグインの上書きを検証
- `test_plugin_removal` - プラグインの削除を検証

### 3. `bridge.rs` - rsjs_bridge_core実行テスト

**外部プラグインサーバー必要** - `#[ignore]`でマーク:

- `test_bridge_basic_function_execution` - 基本的な関数実行
- `test_bridge_complex_object_handling` - 複雑なオブジェクト処理
- `test_bridge_error_handling` - エラーハンドリング
- `test_bridge_unknown_function` - 未知の関数呼び出し
- `test_bridge_loose_type_handling` - 緩い型処理
- `test_bridge_async_function_success` - 非同期関数の成功

### 4. `workflow.rs` - ワークフロー実行テスト

**外部プラグインサーバー必要** - `#[ignore]`でマーク:

- `test_workflow_with_external_plugin_add` - 外部プラグイン関数の実行
- `test_workflow_with_external_plugin_process_data` - 複雑なデータ処理
- `test_workflow_without_permission_requirement` - 権限不要の関数
- `test_multiple_plugins_in_workflow` - 複数プラグインの同時使用

### 5. `e2e.rs` - エンドツーエンドテスト

**外部プラグインサーバー必要** - `#[ignore]`でマーク:

- `test_complete_install_load_execute_flow` - インストール→ロード→実行の完全フロー
- `test_plugin_reinstallation_workflow` - プラグインの再インストール

## テストの実行方法

### 外部依存なしのテストを実行

```bash
cargo test --lib external_plugin
```

これにより、`installation.rs`内のすべてのテストが実行されます（6テスト）。

### 外部プラグインサーバーが必要なテストを実行

```bash
# 1. 外部プラグインサーバーをビルド
cargo build --release -p ext_plugin

# 2. 無視されたテストを実行
cargo test --lib external_plugin -- --ignored
```

これにより、`bridge.rs`、`workflow.rs`、`e2e.rs`内のすべてのテストが実行されます（12テスト）。

### すべてのテストを実行

```bash
# 外部プラグインサーバーをビルド後
cargo test --lib external_plugin -- --include-ignored
```

## フィクスチャファイル

### `math_plugin.js`

数学演算を行うテスト用プラグイン:
- `add(a, b)` - 2つの数値を加算
- `process_data(data)` - データオブジェクトを処理

### `error_plugin.js`

エラーハンドリングをテストするためのプラグイン:
- `throw_immediate()` - 即座にエラーをスロー
- `throw_async()` - 非同期でエラーをスロー
- `async_success(value)` - 非同期で値を返す
- `return_null()` - nullを返す
- `no_op()` - 何もしない

### `file_plugin.js`

ファイルシステム権限をテストするためのプラグイン:
- `read_file(path)` - ファイルを読み込む（FilesystemRead権限必要）
- `simple_function(message)` - メッセージをエコー（権限不要）

## テスト結果の例

```
running 18 tests
test external_plugin::bridge::test_bridge_async_function_success ... ignored
test external_plugin::bridge::test_bridge_basic_function_execution ... ignored
test external_plugin::bridge::test_bridge_complex_object_handling ... ignored
test external_plugin::bridge::test_bridge_error_handling ... ignored
test external_plugin::bridge::test_bridge_loose_type_handling ... ignored
test external_plugin::bridge::test_bridge_unknown_function ... ignored
test external_plugin::e2e::test_complete_install_load_execute_flow ... ignored
test external_plugin::e2e::test_plugin_reinstallation_workflow ... ignored
test external_plugin::workflow::test_multiple_plugins_in_workflow ... ignored
test external_plugin::workflow::test_workflow_with_external_plugin_add ... ignored
test external_plugin::workflow::test_workflow_with_external_plugin_process_data ... ignored
test external_plugin::workflow::test_workflow_without_permission_requirement ... ignored
test external_plugin::installation::test_plugin_overwrite ... ok
test external_plugin::installation::test_plugin_installation_creates_directory_structure ... ok
test external_plugin::installation::test_plugin_content_validation ... ok
test external_plugin::installation::test_plugin_removal ... ok
test external_plugin::installation::test_multiple_plugin_versions ... ok
test external_plugin::installation::test_plugin_scan_finds_installed_plugins ... ok

test result: ok. 6 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out
```

## 新しいテストの追加

新しいテストを追加する場合:

1. 適切なモジュール（`installation.rs`、`bridge.rs`、`workflow.rs`、`e2e.rs`）を選択
2. 外部プラグインサーバーが必要な場合は`#[ignore]`属性を追加
3. `common.rs`のヘルパー関数を活用
4. 必要に応じて新しいフィクスチャファイルを`fixtures/`に追加

## 注意事項

- `installation.rs`のテストは外部依存なしで実行可能
- `bridge.rs`、`workflow.rs`、`e2e.rs`のテストは外部プラグインサーバーバイナリが必要
- フィクスチャファイルはJavaScript形式で`src/tests/fixtures/`に配置
- すべてのヘルパー関数は`common.rs`に集約
