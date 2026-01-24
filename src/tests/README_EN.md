# External Plugin Integration Tests

This directory contains integration tests that verify the complete flow of external plugin installation, loading, and execution.

## Directory Structure

```
src/tests/
├── mod.rs                    # Test module definition
├── external_plugin/          # External plugin test module
│   ├── mod.rs                # Module definition
│   ├── common.rs             # Common helper functions
│   ├── installation.rs       # Installation and filesystem tests
│   ├── bridge.rs             # rsjs_bridge_core execution tests
│   ├── workflow.rs           # Workflow execution tests
│   └── e2e.rs                # End-to-end tests
└── fixtures/                  # Test plugin fixtures
    ├── math_plugin.js        # Math operations plugin
    ├── error_plugin.js       # Error handling test plugin
    └── file_plugin.js        # Filesystem permission test plugin
```

## Test Modules

### 1. `common.rs` - Common Helper Functions

Common utilities used across all tests:

- `get_fixtures_dir()` - Get the fixtures directory path
- `get_fixture_path(filename)` - Get the path to a specific fixture file
- `read_fixture(filename)` - Read the contents of a fixture file
- `create_temp_plugin()` - Create a temporary plugin directory
- `create_opstate_with_package()` - Create a test OpState
- `scan_plugin_directory()` - Scan the plugin directory

### 2. `installation.rs` - Installation and Filesystem Tests

**No external dependencies** - These tests can run without an external plugin server:

- `test_plugin_installation_creates_directory_structure` - Verifies directory structure creation
- `test_plugin_scan_finds_installed_plugins` - Verifies plugin scanning functionality
- `test_plugin_content_validation` - Verifies plugin content validation
- `test_multiple_plugin_versions` - Verifies coexistence of multiple versions
- `test_plugin_overwrite` - Verifies plugin overwriting
- `test_plugin_removal` - Verifies plugin removal

### 3. `bridge.rs` - rsjs_bridge_core Execution Tests

**External plugin server required** - Marked with `#[ignore]`:

- `test_bridge_basic_function_execution` - Basic function execution
- `test_bridge_complex_object_handling` - Complex object handling
- `test_bridge_error_handling` - Error handling
- `test_bridge_unknown_function` - Unknown function calls
- `test_bridge_loose_type_handling` - Loose type handling
- `test_bridge_async_function_success` - Async function success

### 4. `workflow.rs` - Workflow Execution Tests

**External plugin server required** - Marked with `#[ignore]`:

- `test_workflow_with_external_plugin_add` - Execution of external plugin functions
- `test_workflow_with_external_plugin_process_data` - Complex data processing
- `test_workflow_without_permission_requirement` - Functions without permission requirements
- `test_multiple_plugins_in_workflow` - Simultaneous use of multiple plugins

### 5. `e2e.rs` - End-to-End Tests

**External plugin server required** - Marked with `#[ignore]`:

- `test_complete_install_load_execute_flow` - Complete flow: install → load → execute
- `test_plugin_reinstallation_workflow` - Plugin reinstallation

## How to Run Tests

### Run Tests Without External Dependencies

```bash
cargo test --lib external_plugin
```

This runs all tests in `installation.rs` (6 tests).

### Run Tests Requiring External Plugin Server

```bash
# 1. Build the external plugin server
cargo build --release -p ext_plugin

# 2. Run ignored tests
cargo test --lib external_plugin -- --ignored
```

This runs all tests in `bridge.rs`, `workflow.rs`, and `e2e.rs` (12 tests).

### Run All Tests

```bash
# After building the external plugin server
cargo test --lib external_plugin -- --include-ignored
```

## Fixture Files

### `math_plugin.js`

Test plugin for mathematical operations:
- `add(a, b)` - Adds two numbers
- `process_data(data)` - Processes a data object

### `error_plugin.js`

Plugin for testing error handling:
- `throw_immediate()` - Throws an error immediately
- `throw_async()` - Throws an error asynchronously
- `async_success(value)` - Returns a value asynchronously
- `return_null()` - Returns null
- `no_op()` - Does nothing

### `file_plugin.js`

Plugin for testing filesystem permissions:
- `read_file(path)` - Reads a file (requires FilesystemRead permission)
- `simple_function(message)` - Echoes a message (no permission required)

## Example Test Results

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

## Adding New Tests

When adding new tests:

1. Choose the appropriate module (`installation.rs`, `bridge.rs`, `workflow.rs`, `e2e.rs`)
2. Add the `#[ignore]` attribute if the external plugin server is required
3. Utilize helper functions from `common.rs`
4. Add new fixture files to `fixtures/` as needed

## Notes

- Tests in `installation.rs` can run without external dependencies
- Tests in `bridge.rs`, `workflow.rs`, and `e2e.rs` require the external plugin server binary
- Fixture files are in JavaScript format and located in `src/tests/fixtures/`
- All helper functions are consolidated in `common.rs`
