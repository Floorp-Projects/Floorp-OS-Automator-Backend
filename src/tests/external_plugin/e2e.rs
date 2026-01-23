// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! End-to-end tests for complete plugin install→load→execute flow.
//!
//! These tests verify the complete lifecycle of external plugins from
//! installation through execution. They require the external plugin server
//! binary to be built and accessible.
//!
//! Run with: `cargo test --test external_plugin -- --ignored`

use super::common::*;
use sapphillon_core::plugin::{
    CorePluginExternalFunction, CorePluginExternalPackage, PluginPackageTrait,
};
use sapphillon_core::workflow::CoreWorkflowCode;
use std::sync::Arc;
use tempfile::TempDir;

/// Integration Test: Complete Install → Load → Execute Flow
///
/// **Purpose:**
/// Verify the complete end-to-end flow of:
/// 1. Installing an external plugin to the filesystem
/// 2. Loading the plugin from the installed location
/// 3. Executing a workflow that uses the installed plugin
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_complete_install_load_execute_flow() {
    let math_plugin = read_fixture("math_plugin.js");

    // Step 1: Simulate plugin installation
    let author_id = "com.sapphillon.test";
    let package_name = "installed-plugin";
    let version = "1.0.0";

    let (_temp_dir, package_js_path) =
        create_temp_plugin(author_id, package_name, version, &math_plugin);

    // Step 2: Verify installation
    assert!(
        package_js_path.exists(),
        "Plugin should be installed at {package_js_path:?}"
    );

    // Step 3: Load plugin from installed location
    let loaded_js =
        std::fs::read_to_string(&package_js_path).expect("Failed to read installed plugin");

    // Step 4: Create plugin package from loaded code
    let add_func = CorePluginExternalFunction::new(
        "installed-plugin-add".to_string(),
        "add".to_string(),
        "Adds two numbers".to_string(),
        "installedPlugin".to_string(),
        loaded_js.clone(),
        author_id.to_string(),
    );

    let ext_package = CorePluginExternalPackage::new(
        format!("{author_id}.installedPlugin"),
        "installedPlugin".to_string(),
        vec![add_func],
        loaded_js,
    );

    // Step 5: Execute workflow with loaded plugin
    let workflow_code = format!(
        r#"
        const result = {author_id}.installedPlugin.add(100, 200);
        console.log("Installed plugin result:", result);
    "#
    );

    let mut code = CoreWorkflowCode::new(
        "test_install_load_execute".to_string(),
        workflow_code,
        vec![Arc::new(ext_package) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    // 既存のランタイムハンドルを取得
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    let external_package_runner_path = get_debug_binary_path();

    code.run(
        tokio_runtime.handle().clone(),
        external_package_runner_path,
        Some(vec!["ext".to_string()]),
    );

    // Step 6: Verify results
    assert_eq!(code.result.len(), 1);
    let res = &code.result[0];
    assert_eq!(
        res.exit_code, 0,
        "Workflow should succeed, but got error: {}",
        res.result
    );
    assert!(
        res.result.contains("300"),
        "Expected '300' (100+200), got: {}",
        res.result
    );
}

/// Integration Test: Plugin Reinstallation (Overwrite)
///
/// **Purpose:**
/// Verify that a plugin can be reinstalled (overwritten) with new code.
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_plugin_reinstallation_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plugin_dir = temp_dir.path().join("author/plugin/1.0.0");
    std::fs::create_dir_all(&plugin_dir).expect("Failed to create dir");

    // Initial installation - multiplies by 2
    let initial_plugin = r#"
globalThis.Sapphillon = {
    Package: {
        meta: { 
            name: "calc", 
            version: "1.0.0", 
            author_id: "test", 
            package_id: "test.calc",
            description: "Calculator plugin"
        },
        functions: {
            calculate: {
                description: "Calculates value",
                permissions: [],
                parameters: [{ idx: 0, name: "x", type: "number", description: "Input number" }],
                returns: [{ idx: 0, type: "number", description: "Result" }],
                handler: (x) => x * 2
            }
        }
    }
};
"#;

    let package_path = plugin_dir.join("package.js");
    std::fs::write(&package_path, initial_plugin).expect("Failed to write initial plugin");

    // Load and execute initial version
    let loaded_v1 = std::fs::read_to_string(&package_path).expect("Failed to read v1");

    let calc_func_v1 = CorePluginExternalFunction::new(
        "calc".to_string(),
        "calculate".to_string(),
        "Calculates".to_string(),
        "calcPlugin".to_string(),
        loaded_v1.clone(),
        "test".to_string(),
    );

    let package_v1 = CorePluginExternalPackage::new(
        "test.calcPlugin".to_string(),
        "calcPlugin".to_string(),
        vec![calc_func_v1],
        loaded_v1,
    );

    let workflow_v1 = r#"
        console.log(test.calcPlugin.calculate(10));
    "#;

    let mut code_v1 = CoreWorkflowCode::new(
        "test_v1".to_string(),
        workflow_v1.to_string(),
        vec![Arc::new(package_v1) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    // 既存のランタイムハンドルを取得
    let handle = tokio::runtime::Runtime::new().unwrap().handle().clone();
    code_v1.run(handle.clone(), None, None);

    assert!(
        code_v1.result[0].result.contains("20"),
        "V1 should return 20 (10*2), got: {}",
        code_v1.result[0].result
    );

    // Reinstall with new version - multiplies by 3
    let updated_plugin = r#"
globalThis.Sapphillon = {
    Package: {
        meta: { 
            name: "calc", 
            version: "1.0.0", 
            author_id: "test", 
            package_id: "test.calc",
            description: "Calculator plugin updated"
        },
        functions: {
            calculate: {
                description: "Calculates value",
                permissions: [],
                parameters: [{ idx: 0, name: "x", type: "number", description: "Input number" }],
                returns: [{ idx: 0, type: "number", description: "Result" }],
                handler: (x) => x * 3
            }
        }
    }
};
"#;

    std::fs::write(&package_path, updated_plugin).expect("Failed to write updated plugin");

    // Load and execute updated version
    let loaded_v2 = std::fs::read_to_string(&package_path).expect("Failed to read v2");

    let calc_func_v2 = CorePluginExternalFunction::new(
        "calc".to_string(),
        "calculate".to_string(),
        "Calculates".to_string(),
        "calcPlugin".to_string(),
        loaded_v2.clone(),
        "test".to_string(),
    );

    let package_v2 = CorePluginExternalPackage::new(
        "test.calcPlugin".to_string(),
        "calcPlugin".to_string(),
        vec![calc_func_v2],
        loaded_v2,
    );

    let workflow_v2 = r#"
        console.log(test.calcPlugin.calculate(10));
    "#;

    let mut code_v2 = CoreWorkflowCode::new(
        "test_v2".to_string(),
        workflow_v2.to_string(),
        vec![Arc::new(package_v2) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    // 既存のランタイムハンドルを取得
    let handle = tokio::runtime::Handle::try_current()
        .expect("Tokio runtime must be available");
    code_v2.run(handle.clone(), None, None);

    assert!(
        code_v2.result[0].result.contains("30"),
        "V2 should return 30 (10*3), got: {}",
        code_v2.result[0].result
    );
}
