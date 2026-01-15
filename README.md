# Sapphillon

## License
This project is licensed under the GNU Public License V3. See the [LICENSE](LICENSE) file for details

## Protocol Buffer Debug Tools

- evans
- buf

This repository generated from <https://github.com/Walkmana-25/rust-actions-example>

## System Requirements

- MacOS
    - Big Sur or Later
    - Apple Silicon
- Linux
    - glibc 2.31 or Later
    - AMD64 or ARM64
- Windows
    - Windows 10 or Later
    - x64

## Debug Workflow (Debug Build Only)

Debugビルド時のみ有効になる機能で、`debug_workflow` ディレクトリ内のJavaScriptファイルを定期的にスキャンして自動実行します。

### 機能

- **定期スキャン**: 10秒間隔で `debug_workflow` ディレクトリをスキャン
- **全権限付与**: デバッグワークフローには全てのプラグインへのアクセス権限が付与されます
- **自動実行**: 検出されたJSファイルは自動的にワークフローとして実行されます

### 使用方法

1. `debug_workflow` ディレクトリにJavaScriptファイルを配置
2. Debugビルドでアプリケーションを起動 (`cargo run`)
3. ログに実行結果が出力されます

### サンプル

```javascript
// debug_workflow/test.js
function workflow() {
    console.log("Debug workflow executed!");
    const result = fetch("https://api.example.com/data");
    console.log(result);
}
workflow();
```

> **注意**: この機能はDebugビルドでのみ有効です。Releaseビルド (`cargo build --release`) では無効化されます。