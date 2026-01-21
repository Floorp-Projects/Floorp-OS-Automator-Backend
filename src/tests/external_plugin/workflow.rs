// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Tests for workflow execution with external plugins.
//!
//! These tests verify that CoreWorkflowCode can correctly execute workflows
//! that use external plugin functions. They require the external plugin server
//! binary to be built and accessible.
//!
//! Run with: `cargo test --test external_plugin -- --ignored`

use super::common::*;
use sapphillon_core::plugin::{CorePluginExternalFunction, CorePluginExternalPackage, PluginPackageTrait};
use sapphillon_core::workflow::CoreWorkflowCode;
use std::sync::Arc;

/// Integration Test: External Plugin Execution via CoreWorkflowCode
///
/// **Purpose:**
/// Verify that `CoreWorkflowCode` can correctly execute external plugin functions.
///
/// **Note:** This test requires the external plugin server binary.
#[test]
#[ignore = "Requires external plugin server binary"]
fn test_workflow_with_external_plugin_add() {
    let math_plugin = read_fixture("math_plugin.js");

    let add_func = CorePluginExternalFunction::new(
        "math-plugin-add".to_string(),
        "add".to_string(),
        "Adds two numbers".to_string(),
        "mathPlugin".to_string(),
        math_plugin.clone(),
        "com.sapphillon.test".to_string(),
    );

    let ext_package = CorePluginExternalPackage::new(
        "com.sapphillon.test.mathPlugin".to_string(),
        "mathPlugin".to_string(),
        vec![add_func],
        math_plugin,
    );

    let workflow_code = r#"
        const result = com.sapphillon.test.mathPlugin.add(5, 7);
        console.log(result);
    "#;

    let mut code = CoreWorkflowCode::new(
        "test_wf_ext".to_string(),
        workflow_code.to_string(),
        vec![Arc::new(ext_package) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    code.run(tokio_runtime.handle().clone(), None, None);

    assert_eq!(code.result.len(), 1);
    let res = &code.result[0];
    assert_eq!(
        res.exit_code, 0,
        "Workflow should succeed, but got error: {}",
        res.result
    );
    assert!(
        res.result.contains("12"),
        "Expected result to contain '12', got: {}",
        res.result
    );
}

/// Integration Test: External Plugin Complex Object via CoreWorkflowCode
///
/// **Note:** This test requires the external plugin server binary.
#[test]
#[ignore = "Requires external plugin server binary"]
fn test_workflow_with_external_plugin_process_data() {
    let math_plugin = read_fixture("math_plugin.js");

    let process_func = CorePluginExternalFunction::new(
        "math-plugin-process-data".to_string(),
        "process_data".to_string(),
        "Processes data object".to_string(),
        "mathPlugin".to_string(),
        math_plugin.clone(),
        "com.sapphillon.test".to_string(),
    );

    let ext_package = CorePluginExternalPackage::new(
        "com.sapphillon.test.mathPlugin".to_string(),
        "mathPlugin".to_string(),
        vec![process_func],
        math_plugin,
    );

    let workflow_code = r#"
        const data = { value: 25, multiplier: 4 };
        const result = com.sapphillon.test.mathPlugin.process_data(data);
        console.log("Original:", result.original);
        console.log("Result:", result.result);
    "#;

    let mut code = CoreWorkflowCode::new(
        "test_wf_ext_complex".to_string(),
        workflow_code.to_string(),
        vec![Arc::new(ext_package) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    code.run(tokio_runtime.handle().clone(), None, None);

    assert_eq!(code.result.len(), 1);
    let res = &code.result[0];
    assert_eq!(
        res.exit_code, 0,
        "Workflow should succeed, but got error: {}",
        res.result
    );
    assert!(
        res.result.contains("Original: 25"),
        "Expected result to contain 'Original: 25', got: {}",
        res.result
    );
    assert!(
        res.result.contains("Result: 100"),
        "Expected result to contain 'Result: 100', got: {}",
        res.result
    );
}

/// Integration Test: Workflow Without Permission Requirement
///
/// **Note:** This test requires the external plugin server binary.
#[test]
#[ignore = "Requires external plugin server binary"]
fn test_workflow_without_permission_requirement() {
    let file_plugin = read_fixture("file_plugin.js");

    let simple_func = CorePluginExternalFunction::new(
        "file-plugin-simple-function".to_string(),
        "simple_function".to_string(),
        "A simple function".to_string(),
        "filePlugin".to_string(),
        file_plugin.clone(),
        "com.sapphillon.test".to_string(),
    );

    let ext_package = CorePluginExternalPackage::new(
        "com.sapphillon.test.filePlugin".to_string(),
        "filePlugin".to_string(),
        vec![simple_func],
        file_plugin,
    );

    let workflow_code = r#"
        const result = com.sapphillon.test.filePlugin.simple_function("Hello World");
        console.log("Result:", result);
    "#;

    let mut code = CoreWorkflowCode::new(
        "test_wf_no_permission_needed".to_string(),
        workflow_code.to_string(),
        vec![Arc::new(ext_package) as Arc<dyn PluginPackageTrait>],
        1,
        vec![],
        vec![],
    );

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    code.run(tokio_runtime.handle().clone(), None, None);

    assert_eq!(code.result.len(), 1);
    let res = &code.result[0];
    assert_eq!(
        res.exit_code, 0,
        "Workflow should succeed, but got error: {}",
        res.result
    );
    assert!(
        res.result.contains("Echo: Hello World"),
        "Expected echoed text, got: {}",
        res.result
    );
}

/// Integration Test: Multiple Plugins in Single Workflow
///
/// **Note:** This test requires the external plugin server binary.
#[test]
#[ignore = "Requires external plugin server binary"]
fn test_multiple_plugins_in_workflow() {
    let math_plugin = read_fixture("math_plugin.js");
    let file_plugin = read_fixture("file_plugin.js");

    let add_func = CorePluginExternalFunction::new(
        "math-add".to_string(),
        "add".to_string(),
        "Adds two numbers".to_string(),
        "mathPlugin".to_string(),
        math_plugin.clone(),
        "com.sapphillon.test".to_string(),
    );

    let math_package = CorePluginExternalPackage::new(
        "com.sapphillon.test.mathPlugin".to_string(),
        "mathPlugin".to_string(),
        vec![add_func],
        math_plugin,
    );

    let simple_func = CorePluginExternalFunction::new(
        "file-simple".to_string(),
        "simple_function".to_string(),
        "Echoes a message".to_string(),
        "filePlugin".to_string(),
        file_plugin.clone(),
        "com.sapphillon.test".to_string(),
    );

    let file_package = CorePluginExternalPackage::new(
        "com.sapphillon.test.filePlugin".to_string(),
        "filePlugin".to_string(),
        vec![simple_func],
        file_plugin,
    );

    let workflow_code = r#"
        const sum = com.sapphillon.test.mathPlugin.add(10, 20);
        const message = com.sapphillon.test.filePlugin.simple_function("Sum is: " + sum);
        console.log(message);
    "#;

    let mut code = CoreWorkflowCode::new(
        "test_multi_plugin".to_string(),
        workflow_code.to_string(),
        vec![
            Arc::new(math_package) as Arc<dyn PluginPackageTrait>,
            Arc::new(file_package) as Arc<dyn PluginPackageTrait>,
        ],
        1,
        vec![],
        vec![],
    );

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    code.run(tokio_runtime.handle().clone(), None, None);

    assert_eq!(code.result.len(), 1);
    let res = &code.result[0];
    assert_eq!(
        res.exit_code, 0,
        "Multi-plugin workflow should succeed, but got error: {}",
        res.result
    );
    assert!(
        res.result.contains("Echo: Sum is: 30"),
        "Expected echoed sum message, got: {}",
        res.result
    );
}
