// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Tests for external plugin installation and filesystem operations.
//!
//! These tests verify plugin installation, directory structure creation,
//! scanning, and file operations. They do not require the external plugin
//! server and can run independently.

use super::common::*;
use tempfile::TempDir;

/// Integration Test: Plugin Installation Creates Correct Directory Structure
///
/// **Purpose:**
/// Verify that installing an external plugin creates the correct directory
/// structure and writes the package.js file to the expected location.
#[test]
fn test_plugin_installation_creates_directory_structure() {
    let author_id = "com.test.author";
    let package_id = "my-plugin";
    let version = "1.0.0";
    let plugin_content = read_fixture("math_plugin.js");

    let (temp_dir, package_js_path) =
        create_temp_plugin(author_id, package_id, version, &plugin_content);

    // Verify the file was created
    assert!(
        package_js_path.exists(),
        "package.js should exist at {:?}",
        package_js_path
    );

    // Verify the content
    let content = std::fs::read_to_string(&package_js_path).expect("Failed to read package.js");
    assert_eq!(content, plugin_content);

    // Verify the directory structure
    let expected_dir = temp_dir
        .path()
        .join(author_id)
        .join(package_id)
        .join(version);
    assert!(expected_dir.exists(), "Plugin directory should exist");
    assert!(expected_dir.is_dir(), "Plugin path should be a directory");
}

/// Integration Test: Plugin Scan Finds Installed Plugins
///
/// **Purpose:**
/// Verify that the plugin scanning functionality can find plugins installed
/// in the expected directory structure.
#[test]
fn test_plugin_scan_finds_installed_plugins() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let math_plugin = read_fixture("math_plugin.js");
    let error_plugin = read_fixture("error_plugin.js");

    // Create plugin 1
    let plugin1_dir = temp_dir.path().join("author1/plugin1/1.0.0");
    std::fs::create_dir_all(&plugin1_dir).expect("Failed to create plugin1 dir");
    std::fs::write(plugin1_dir.join("package.js"), math_plugin).expect("Failed to write plugin1");

    // Create plugin 2
    let plugin2_dir = temp_dir.path().join("author2/plugin2/2.0.0");
    std::fs::create_dir_all(&plugin2_dir).expect("Failed to create plugin2 dir");
    std::fs::write(plugin2_dir.join("package.js"), error_plugin).expect("Failed to write plugin2");

    // Create incomplete plugin (no package.js)
    let incomplete_dir = temp_dir.path().join("author3/incomplete/1.0.0");
    std::fs::create_dir_all(&incomplete_dir).expect("Failed to create incomplete dir");

    // Scan for plugins
    let save_dir = temp_dir.path().to_string_lossy().to_string();
    let plugins = scan_plugin_directory(&save_dir);

    assert_eq!(plugins.len(), 2, "Should find exactly 2 valid plugins");
    assert!(
        plugins.contains(&"author1/plugin1/1.0.0".to_string()),
        "Should find plugin1"
    );
    assert!(
        plugins.contains(&"author2/plugin2/2.0.0".to_string()),
        "Should find plugin2"
    );
    assert!(
        !plugins.contains(&"author3/incomplete/1.0.0".to_string()),
        "Should not find incomplete plugin"
    );
}

/// Integration Test: Plugin Content Validation
///
/// **Purpose:**
/// Verify that plugin JavaScript content can be read and parsed correctly.
#[test]
fn test_plugin_content_validation() {
    let math_plugin = read_fixture("math_plugin.js");
    let (_temp_dir, package_js_path) =
        create_temp_plugin("com.test", "test-plugin", "1.0.0", &math_plugin);

    let content = std::fs::read_to_string(&package_js_path).expect("Failed to read package.js");

    // Verify the content contains expected structure
    assert!(
        content.contains("globalThis.Sapphillon"),
        "Should contain globalThis.Sapphillon"
    );
    assert!(content.contains("Package"), "Should contain Package");
    assert!(content.contains("functions"), "Should contain functions");
    assert!(content.contains("add"), "Should contain add function");
    assert!(
        content.contains("process_data"),
        "Should contain process_data function"
    );
}

/// Integration Test: Multiple Plugin Versions
///
/// **Purpose:**
/// Verify that multiple versions of the same plugin can coexist.
#[test]
fn test_multiple_plugin_versions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create version 1.0.0
    let v1_dir = temp_dir.path().join("author/plugin/1.0.0");
    std::fs::create_dir_all(&v1_dir).expect("Failed to create v1 dir");
    std::fs::write(v1_dir.join("package.js"), "// v1.0.0").expect("Failed to write v1");

    // Create version 2.0.0
    let v2_dir = temp_dir.path().join("author/plugin/2.0.0");
    std::fs::create_dir_all(&v2_dir).expect("Failed to create v2 dir");
    std::fs::write(v2_dir.join("package.js"), "// v2.0.0").expect("Failed to write v2");

    // Create version 3.0.0-beta
    let v3_dir = temp_dir.path().join("author/plugin/3.0.0-beta");
    std::fs::create_dir_all(&v3_dir).expect("Failed to create v3 dir");
    std::fs::write(v3_dir.join("package.js"), "// v3.0.0-beta").expect("Failed to write v3");

    // Scan and verify all versions are found
    let save_dir = temp_dir.path().to_string_lossy().to_string();
    let plugins = scan_plugin_directory(&save_dir);

    assert_eq!(plugins.len(), 3, "Should find 3 plugin versions");
    assert!(plugins.contains(&"author/plugin/1.0.0".to_string()));
    assert!(plugins.contains(&"author/plugin/2.0.0".to_string()));
    assert!(plugins.contains(&"author/plugin/3.0.0-beta".to_string()));
}

/// Integration Test: Plugin Overwrite
///
/// **Purpose:**
/// Verify that a plugin can be overwritten with new content.
#[test]
fn test_plugin_overwrite() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plugin_dir = temp_dir.path().join("author/plugin/1.0.0");
    std::fs::create_dir_all(&plugin_dir).expect("Failed to create dir");

    let package_js_path = plugin_dir.join("package.js");

    // Write initial content
    std::fs::write(&package_js_path, "// initial content").expect("Failed to write initial");
    assert_eq!(
        std::fs::read_to_string(&package_js_path).unwrap(),
        "// initial content"
    );

    // Overwrite with new content
    std::fs::write(&package_js_path, "// updated content").expect("Failed to write updated");
    assert_eq!(
        std::fs::read_to_string(&package_js_path).unwrap(),
        "// updated content"
    );
}

/// Integration Test: Plugin Removal
///
/// **Purpose:**
/// Verify that a plugin can be removed and is no longer detected.
#[test]
fn test_plugin_removal() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let math_plugin = read_fixture("math_plugin.js");

    // Create plugin
    let plugin_dir = temp_dir.path().join("author/plugin/1.0.0");
    std::fs::create_dir_all(&plugin_dir).expect("Failed to create dir");
    std::fs::write(plugin_dir.join("package.js"), math_plugin).expect("Failed to write");

    // Verify plugin exists
    let save_dir = temp_dir.path().to_string_lossy().to_string();
    let plugins = scan_plugin_directory(&save_dir);
    assert_eq!(plugins.len(), 1);

    // Remove plugin
    std::fs::remove_dir_all(&plugin_dir).expect("Failed to remove plugin");

    // Verify plugin is gone
    let plugins_after = scan_plugin_directory(&save_dir);
    assert_eq!(plugins_after.len(), 0, "Plugin should be removed");
}
