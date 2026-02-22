# Sapphillon

## Overview

Sapphillon is an extensible workflow orchestration system developed in Rust, featuring AI-powered workflow generation. It achieves flexible workflow automation through a gRPC-based architecture, Deno-based custom runtime, and plugin system.


## Key Features

### Core Features
- Workflow Orchestration (JavaScript/TypeScript support)
- gRPC Server (port 50051)
- SQLite Database Management (SeaORM)
- Extensible Plugin System
- AI-Powered Workflow Generation

### Built-in Plugins
- **fetch**: HTTP Requests
- **filesystem**: File System Operations
- **window**: Window Management
- **exec**: Command Execution
- **search**: File Search

## Installation

### System Requirements

- macOS
    - Big Sur or Later
    - Apple Silicon
- Linux
    - glibc 2.31 or Later
    - AMD64 or ARM64
- Windows (Paused)
    - Windows 10 or Later
    - x64

### Dependencies
- Rust 2024 Edition
- Cargo
- SQLite

### Build Instructions
```bash
git clone https://github.com/Walkmana-25/Sapphillon.git
cd Sapphillon
cargo build --workspace --all-features
```

## Quick Start

### Starting the Server
```bash
# Default settings (in-memory DB)
cargo run -- start

# Debug mode (file-based DB)
cargo run -- --loglevel debug --db-url ./debug/sqlite.db start
```

### Command Line Options
| Option | Description | Default Value |
|-----------|------|------------|
| `--loglevel` | Log level | info |
| `--db-url` | Database URL | In-memory SQLite |
| `--ext-plugin-save-dir` | External plugin save directory | System temporary directory |

## Project Structure

```
Sapphillon/
├── src/                    # Main source code
│   ├── main.rs            # Entry point
│   ├── server.rs          # gRPC server
│   └── services/          # gRPC services
├── entity/                # SeaORM entities
├── database/             # Database operations
├── migration/            # Migrations
├── plugins/              # Built-in plugins
└── docs/                 # Documentation
```

## Developer Information

For detailed development information, see [`DEVELOPERS.md`](DEVELOPERS.md).

### Makefile Targets

- `make test`: Run tests
- `make build`: Build
- `make fmt`: Code check
- `make fix_fmt`: Code fix
- `make migrate`: Run migrations
- `make run`: Local execution

### Development Workflow

```bash
# Initialize database
make gen_empty_db && make migrate && make entity_generate

# Start development server
make run
```

## Debug Workflow (Debug Build Only)

This feature is only available in debug builds. It periodically scans the `debug_workflow` directory and automatically registers JavaScript files to the database.

### Features

- **Periodic Scanning**: Scans the `debug_workflow` directory every 5 seconds
- **Full Permissions**: Debug workflows have access to all plugins
- **Auto Registration**: Detected JS files are automatically registered as workflows in the database

### Usage

1. Place JavaScript files in the `debug_workflow` directory
2. Run the application with debug build (`cargo run`)
3. Workflows are registered in the database with a `[DEBUG]` prefix

### Example

```javascript
// debug_workflow/test.js
function workflow() {
    console.log("Debug workflow executed!");
    const result = fetch("https://api.example.com/data");
    console.log(result);
}
workflow();
```

> **Note**: This feature is only available in debug builds. It will be disabled in release builds (`cargo build --release`).

## License

MPL-2.0 OR GPL-3.0-or-later

For details, see [`LICENSE`](LICENSE), [`LICENSE-MPL`](LICENSE-MPL), [`LICENSE-GPL`](LICENSE-GPL).

## Copyright

© 2025 Yuta Takahashi

## Related Repositories

- [Sapphillon](https://github.com/Sapphillon/Sapphillon)
- [Sapphillon Front](https://github.com/Sapphillon/Sapphillon-front)
- [Sapphillon Core (Core Library)](https://github.com/Sapphillon/Sapphillon-core)
- [Sapphillon CLI (Command Line Tool)](https://github.com/Sapphillon/Sapphillon_cli)
- [Repository Template](https://github.com/Walkmana-25/rust-actions-template)


## Links

- [GitHub Repository](https://github.com/Walkmana-25/Sapphillon)
- [Developer Documentation](DEVELOPERS.md)
- [Test Documentation](src/tests/README.md)

## Special Thanks

- [Floorp Projects](https://floorp.app)
- [Repository Template](https://github.com/Walkmana-25/rust-actions-template)
- [IPA Mitou IT Human Resources Development Project](https://www.ipa.go.jp/jinzai/mitou/)
