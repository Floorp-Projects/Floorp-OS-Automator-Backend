# Sapphillon

## Floorp OS について

Sapphillon は、汎用的なワークフロー自動化プラットフォームです。**Floorp OS** は Sapphillon をベースとした派生プロダクトであり、Sapphillon とは独立したプロジェクトです。Ubuntu が Debian をベースにしつつも独自のプロジェクトとして開発されているのと同様に、Floorp OS は Sapphillon のコア技術を基盤としながら、[Floorp ブラウザ](https://floorp.app) との統合や OS レベルの自動化機能など、独自の機能を追加しています。

本リポジトリ（Floorp-OS-Automator-Backend）は、Floorp OS の**バックエンドサーバー**として機能します。

### Sapphillon の役割

- **gRPC サーバー**: Floorp ブラウザおよびフロントエンドアプリケーションとの通信を担当
- **ワークフローエンジン**: [Sapphillon](https://github.com/Sapphillon/Sapphillon) をベースに、ワークフローの実行・管理を提供
- **プラグインシステム**: ファイルシステム操作、Web リクエスト、ウィンドウ管理、OCR など、OS レベルの自動化プラグインを搭載
- **データ永続化**: SQLite（Sea-ORM）によるワークフローおよびプラグインデータの管理

### アーキテクチャ概要

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Floorp ブラウザ                                │
│                                                                      │
│  プロセス管理（起動・停止）         ローカル HTTP サーバー (:58261)         │
│       │           │                        ▲                         │
└───────┼───────────┼────────────────────────┼─────────────────────────┘
        │           │                        │ HTTP/JSON
        ▼           ▼                        │ (floorp プラグイン)
  ┌──────────┐  ┌──────────────────────────────────────────────┐
  │ Frontend │  │          Sapphillon Backend (:50051)          │
  │ (React)  │  │                                               │
  │          │──│─── ConnectRPC / gRPC-Web ──▶ gRPC Server      │
  │          │  │                              │                │
  └──────────┘  │   Sapphillon-Core  ◀---------┘                │
                │      ├── ワークフロー実行エンジン (Deno)          │
                │      └── プラグインシステム                      │
                │                                               │
                │   プラグイン:                                   │
                │     exec, fetch, filesystem, floorp,          │
                │     llm-chat, ocr, search, window             │
                │                                               │
                │   SQLite データベース (Sea-ORM)                 │
                └───────────────────────────────────────────────┘
```

- **Floorp ブラウザ** は Backend と Frontend の両バイナリを子プロセスとして起動・管理します（直接 gRPC 通信はしません）
- **Frontend** は ConnectRPC（gRPC-Web）を使用して Backend と通信します（デフォルト: `http://localhost:50051`）
- **Backend の floorp プラグイン** は Floorp ブラウザのローカル HTTP サーバー（`http://localhost:58261`）を呼び出し、タブ操作やスクレイピングを行います

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

This feature is only enabled in debug builds. It periodically scans the `debug_workflow` directory for JavaScript files and automatically registers them to the database.

### Features

- **Periodic Scan**: Scans the `debug_workflow` directory every 10 seconds
- **Full Permissions**: Debug workflows are granted access to all plugins
- **Auto-Registration**: Detected JS files are automatically registered as workflows in the database

### Usage

1. Place JavaScript files in the `debug_workflow` directory
2. Run the application with a debug build (`cargo run`)
3. Workflows will be registered with the `[DEBUG]` prefix in the database

### Sample

```javascript
// debug_workflow/test.js
function workflow() {
  console.log("Debug workflow executed!");
  const result = fetch("https://api.example.com/data");
  console.log(result);
}
workflow();
```

> **Note**: This feature is only available in debug builds. It is disabled in release builds (`cargo build --release`).
