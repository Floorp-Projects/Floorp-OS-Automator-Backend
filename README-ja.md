# Sapphillon

## 概要

Sapphillonは、Rustで開発された拡張可能なワークフローオーケストレーションシステムで、AIによるワークフロー自動生成機能を備えています。gRPCベースのアーキテクチャ、Denoベースのカスタムランタイムとプラグインシステムにより、柔軟なワークフロー自動化を実現します。


## 主な機能

### コア機能
- ワークフローオーケストレーション（JavaScript/TypeScript対応）
- gRPCサーバー（ポート50051）
- SQLiteデータベース管理（SeaORM）
- 拡張可能なプラグインシステム
- AIによるワークフロー自動生成

### ビルトインプラグイン
- **fetch**: HTTPリクエスト
- **filesystem**: ファイルシステム操作
- **window**: ウィンドウ管理
- **exec**: コマンド実行
- **search**: ファイル検索

## インストール

### システム要件

- macOS
    - Big Sur or Later
    - Apple Silicon
- Linux
    - glibc 2.31 or Later
    - AMD64 or ARM64
- Windows (中断中)
    - Windows 10 or Later
    - x64

### 依存関係
- Rust 2024 Edition
- Cargo
- SQLite

### ビルド手順
```bash
git clone https://github.com/Walkmana-25/Sapphillon.git
cd Sapphillon
cargo build --workspace --all-features
```

## クイックスタート

### サーバーの起動
```bash
# デフォルト設定（インメモリDB）
cargo run -- start

# デバッグモード（ファイルベースDB）
cargo run -- --loglevel debug --db-url ./debug/sqlite.db start
```

### コマンドラインオプション
| オプション | 説明 | デフォルト値 |
|-----------|------|------------|
| `--loglevel` | ログレベル | info |
| `--db-url` | データベースURL | インメモリSQLite |
| `--ext-plugin-save-dir` | 外部プラグイン保存ディレクトリ | システム一時ディレクトリ |

## プロジェクト構造

```
Sapphillon/
├── src/                    # メインソースコード
│   ├── main.rs            # エントリーポイント
│   ├── server.rs          # gRPCサーバー
│   └── services/          # gRPCサービス
├── entity/                # SeaORMエンティティ
├── database/             # データベース操作
├── migration/            # マイグレーション
├── plugins/              # ビルトインプラグイン
└── docs/                 # ドキュメント
```

## 開発者向け情報

詳細な開発情報は [`DEVELOPERS.md`](DEVELOPERS.md) を参照してください。

### Makefileターゲット

- `make test`: テスト実行
- `make build`: ビルド
- `make fmt`: コードチェック
- `make fix_fmt`: コード修正
- `make migrate`: マイグレーション実行
- `make run`: ローカル実行

### 開発ワークフロー

```bash
# データベース初期化
make gen_empty_db && make migrate && make entity_generate

# 開発サーバー起動
make run
```

## デバッグワークフロー（デバッグビルドのみ）

この機能はデバッグビルドでのみ有効になります。`debug_workflow` ディレクトリを定期的にスキャンし、JavaScriptファイルを自動的にデータベースに登録します。

### 機能

- **定期スキャン**: `debug_workflow` ディレクトリを5秒ごとにスキャン
- **フル権限**: デバッグワークフローはすべてのプラグインにアクセス可能
- **自動登録**: 検出されたJSファイルは自動的にワークフローとしてデータベースに登録

### 使用方法

1. JavaScriptファイルを `debug_workflow` ディレクトリに配置
2. デバッグビルドでアプリケーションを実行（`cargo run`）
3. ワークフローは `[DEBUG]` プレフィックス付きでデータベースに登録されます

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

> **注**: この機能はデバッグビルドでのみ使用可能です。リリースビルド（`cargo build --release`）では無効になります。

## ライセンス

MPL-2.0 OR GPL-3.0-or-later

詳細は [`LICENSE`](LICENSE)、[`LICENSE-MPL`](LICENSE-MPL)、[`LICENSE-GPL`](LICENSE-GPL) を参照してください。

## 著作権

© 2025 Yuta Takahashi

## 関連リポジトリ

- [Sapphillon](https://github.com/Sapphillon/Sapphillon)
- [Sapphillon Front](https://github.com/Sapphillon/Sapphillon-front)
- [Sapphillon Core (コアライブラリ)](https://github.com/Sapphillon/Sapphillon-Core)
- [Sapphillon CLI (コマンドラインツール)](https://github.com/Sapphillon/Sapphillon_cli)
- [リポジトリテンプレート](https://github.com/Walkmana-25/rust-actions-template)


## リンク

- [GitHubリポジトリ](https://github.com/Walkmana-25/Sapphillon)
- [開発者ドキュメント](DEVELOPERS.md)
- [テストドキュメント](src/tests/README.md)

## Special Thanks

- [Floorp Projects](https://floorp.app)
- [リポジトリテンプレート](https://github.com/Walkmana-25/rust-actions-template)
- [IPA 未踏IT人材発掘・育成事業](https://www.ipa.go.jp/jinzai/mitou/)
