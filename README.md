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