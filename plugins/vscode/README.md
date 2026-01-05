# VSCode Plugin for Floorp-OS-Automator-Backend

A Sapphillon plugin to manage VSCode (Visual Studio Code) from workflows.

## Features

- **Open Folder**: Open a folder in VSCode
- **Open File**: Open a specific file in VSCode
- **Write File**: Write content to a file and open it in VSCode
- **Close Workspace**: Close the current VSCode workspace/project

## Requirements

- VSCode must be installed and the `code` command must be available in PATH

## Usage

```javascript
// Open a folder in VSCode
vscode.open_folder("/path/to/folder");

// Open a file in VSCode
vscode.open_file("/path/to/file.txt");

// Write content to a file and open it
vscode.write_file("/path/to/file.txt", "Hello World");

// Close current workspace
vscode.close_workspace();
```

## License

MPL-2.0 OR GPL-3.0-or-later
