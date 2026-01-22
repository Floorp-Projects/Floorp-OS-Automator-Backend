// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Common utilities and helpers for external plugin tests.

use sapphillon_core::deno_core;
use sapphillon_core::plugin::CorePluginExternalPackage;
use sapphillon_core::runtime::OpStateWorkflowData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Returns the path to the test fixtures directory.
pub fn get_fixtures_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("src");
    d.push("tests");
    d.push("fixtures");
    d
}

/// Returns the path to a specific fixture file.
pub fn get_fixture_path(filename: &str) -> PathBuf {
    let mut path = get_fixtures_dir();
    path.push(filename);
    path
}

/// Reads a fixture file and returns its contents as a String.
pub fn read_fixture(filename: &str) -> String {
    let path = get_fixture_path(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", filename, e))
}

/// Creates a temporary directory with plugin structure and returns the path.
///
/// This simulates what `install_ext_plugin` does: creating the directory
/// structure `{save_dir}/{author_id}/{package_id}/{version}/package.js`
pub fn create_temp_plugin(
    author_id: &str,
    package_id: &str,
    version: &str,
    package_js_content: &str,
) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plugin_dir = temp_dir
        .path()
        .join(author_id)
        .join(package_id)
        .join(version);
    std::fs::create_dir_all(&plugin_dir).expect("Failed to create plugin dir");

    let package_js_path = plugin_dir.join("package.js");
    std::fs::write(&package_js_path, package_js_content).expect("Failed to write package.js");

    (temp_dir, package_js_path)
}

/// Creates OpState with workflow data containing the test external package.
/// Returns the OpState and the tokio Runtime to keep the runtime alive.
#[allow(dead_code)]
pub fn create_opstate_with_package(
    package_js: &str,
    package_name: &str,
    author_id: &str,
) -> (deno_core::OpState, tokio::runtime::Runtime) {
    let mut op_state = deno_core::OpState::new(None);

    // Create the external package
    let package_id = format!("{author_id}.{package_name}");

    let package = CorePluginExternalPackage::new(
        package_id,
        package_name.to_string(),
        vec![], // functions list not needed for this test
        package_js.to_string(),
    );

    // Create OpStateWorkflowData with the external package
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let workflow_data = OpStateWorkflowData::new(
        "test_workflow",
        false,
        None,
        None,
        tokio_runtime.handle().clone(),
        vec![Arc::new(package)],
        None,
        None,
    );

    // Put workflow data into OpState
    op_state.put(Arc::new(Mutex::new(workflow_data)));

    (op_state, tokio_runtime)
}

/// Helper function to scan a directory for installed plugins.
/// Returns a vector of plugin IDs in the format "author/package/version".
pub fn scan_plugin_directory(save_dir: &str) -> Vec<String> {
    use std::fs;
    use std::path::Path;

    let mut plugin_ids = Vec::new();
    let base_path = Path::new(save_dir);

    if !base_path.exists() {
        return plugin_ids;
    }

    // Traverse: author-id/package-id/version/package.js
    if let Ok(author_entries) = fs::read_dir(base_path) {
        for author_entry in author_entries.flatten() {
            if !author_entry
                .file_type()
                .map(|t| t.is_dir())
                .unwrap_or(false)
            {
                continue;
            }
            let author_id = author_entry.file_name().to_string_lossy().to_string();

            if let Ok(package_entries) = fs::read_dir(author_entry.path()) {
                for package_entry in package_entries.flatten() {
                    if !package_entry
                        .file_type()
                        .map(|t| t.is_dir())
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    let package_id = package_entry.file_name().to_string_lossy().to_string();

                    if let Ok(version_entries) = fs::read_dir(package_entry.path()) {
                        for version_entry in version_entries.flatten() {
                            if !version_entry
                                .file_type()
                                .map(|t| t.is_dir())
                                .unwrap_or(false)
                            {
                                continue;
                            }
                            let version = version_entry.file_name().to_string_lossy().to_string();

                            // Check if package.js exists
                            let package_js = version_entry.path().join("package.js");
                            if package_js.exists() {
                                plugin_ids.push(format!("{author_id}/{package_id}/{version}"));
                            }
                        }
                    }
                }
            }
        }
    }

    plugin_ids
}
