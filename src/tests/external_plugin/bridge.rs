// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Tests for rsjs_bridge_core function execution.
//!
//! These tests verify that the bridge can correctly execute JavaScript functions
//! from external plugins. They require the external plugin server binary to be
//! built and accessible.
//!
//! Run with: `cargo test --test external_plugin -- --ignored`

use super::common::*;
use sapphillon_core::ext_plugin::{RsJsBridgeArgs, RsJsBridgeReturns};
use sapphillon_core::extplugin_rsjs_bridge::rsjs_bridge_core;
use serde_json::json;

/// Integration Test: Basic Function Execution via rsjs_bridge_core
///
/// **Purpose:**
/// Verify that the `rsjs_bridge_core` can correctly execute a simple function (`add`)
/// from an external plugin package.
///
/// **Note:** This test requires the external plugin server binary to be built.
/// Run with `cargo test -- --ignored` after building the ext_plugin server.
#[test]
fn test_bridge_basic_function_execution() {
    let math_plugin = read_fixture("math_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&math_plugin, "math-plugin", "com.sapphillon.test");

    let args = RsJsBridgeArgs {
        func_name: "add".to_string(),
        args: vec![("a".to_string(), json!(10)), ("b".to_string(), json!(20))]
            .into_iter()
            .collect(),
    };
    let args_json = args.to_string().unwrap();

    let result = rsjs_bridge_core(&mut op_state, &args_json, "com.sapphillon.test.math-plugin");
    assert!(
        result.is_ok(),
        "Bridge execution failed: {:?}",
        result.err()
    );

    let result_json = result.unwrap();
    let returns = RsJsBridgeReturns::new_from_str(&result_json).expect("Failed to parse returns");

    assert_eq!(returns.args.get("result"), Some(&json!(30)));
}

/// Integration Test: Complex Object Handling via rsjs_bridge_core
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_bridge_complex_object_handling() {
    let math_plugin = read_fixture("math_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&math_plugin, "math-plugin", "com.sapphillon.test");

    let input_data = json!({
        "value": 50,
        "multiplier": 2
    });

    let args = RsJsBridgeArgs {
        func_name: "process_data".to_string(),
        args: vec![("data".to_string(), input_data)].into_iter().collect(),
    };
    let args_json = args.to_string().unwrap();

    let result = rsjs_bridge_core(&mut op_state, &args_json, "com.sapphillon.test.math-plugin");
    assert!(
        result.is_ok(),
        "Bridge execution failed: {:?}",
        result.err()
    );

    let result_json = result.unwrap();
    let returns = RsJsBridgeReturns::new_from_str(&result_json).expect("Failed to parse returns");

    let result_obj = returns.args.get("result").expect("No result returned");

    assert_eq!(result_obj.get("original"), Some(&json!(50)));
    assert_eq!(result_obj.get("result"), Some(&json!(100)));
    assert!(result_obj.get("timestamp").is_some());
}

/// Integration Test: Plugin Throws Error
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_bridge_error_handling() {
    use std::collections::HashMap;

    let error_plugin = read_fixture("error_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&error_plugin, "error-plugin", "com.sapphillon.test");

    let args_immediate = RsJsBridgeArgs {
        func_name: "throw_immediate".to_string(),
        args: HashMap::new(),
    };
    let result_immediate = rsjs_bridge_core(
        &mut op_state,
        &args_immediate.to_string().unwrap(),
        "com.sapphillon.test.error-plugin",
    );

    assert!(
        result_immediate.is_err(),
        "Expected error from throw_immediate, got Ok"
    );
    let err_msg = result_immediate.err().unwrap().to_string();
    assert!(
        err_msg.contains("This is an immediate error"),
        "Expected error message to contain 'This is an immediate error', got: {err_msg}"
    );
}

/// Integration Test: Unknown Function Call
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_bridge_unknown_function() {
    use std::collections::HashMap;

    let math_plugin = read_fixture("math_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&math_plugin, "math-plugin", "com.sapphillon.test");

    let args = RsJsBridgeArgs {
        func_name: "non_existent_func".to_string(),
        args: HashMap::new(),
    };

    let result = rsjs_bridge_core(
        &mut op_state,
        &args.to_string().unwrap(),
        "com.sapphillon.test.math-plugin",
    );

    assert!(
        result.is_err(),
        "Expected error for unknown function, got Ok"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Unknown function") || err_msg.contains("schema not found"),
        "Expected 'Unknown function' error, got: {err_msg}"
    );
}

/// Integration Test: Loose Type Handling
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_bridge_loose_type_handling() {
    let math_plugin = read_fixture("math_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&math_plugin, "math-plugin", "com.sapphillon.test");

    // Pass strings instead of numbers
    let args = RsJsBridgeArgs {
        func_name: "add".to_string(),
        args: vec![
            ("a".to_string(), json!("10")),
            ("b".to_string(), json!("20")),
        ]
        .into_iter()
        .collect(),
    };

    let result = rsjs_bridge_core(
        &mut op_state,
        &args.to_string().unwrap(),
        "com.sapphillon.test.math-plugin",
    );

    assert!(
        result.is_ok(),
        "Bridge execution should succeed (JS is loose typed): {:?}",
        result.err()
    );

    let result_json = result.unwrap();
    let returns = RsJsBridgeReturns::new_from_str(&result_json).expect("Failed to parse returns");

    // JS `+` operator with strings does concatenation
    assert_eq!(
        returns.args.get("result"),
        Some(&json!("1020")),
        "Expected string concatenation result '1020'"
    );
}

/// Integration Test: Async Function Success
///
/// **Note:** This test requires the external plugin server binary.
#[test]
fn test_bridge_async_function_success() {
    let error_plugin = read_fixture("error_plugin.js");
    let (mut op_state, _tokio_rt) =
        create_opstate_with_package(&error_plugin, "error-plugin", "com.sapphillon.test");

    let args = RsJsBridgeArgs {
        func_name: "async_success".to_string(),
        args: vec![("value".to_string(), json!("test-value"))]
            .into_iter()
            .collect(),
    };

    let result = rsjs_bridge_core(
        &mut op_state,
        &args.to_string().unwrap(),
        "com.sapphillon.test.error-plugin",
    );

    assert!(
        result.is_ok(),
        "Async function should succeed: {:?}",
        result.err()
    );

    let result_json = result.unwrap();
    let returns = RsJsBridgeReturns::new_from_str(&result_json).expect("Failed to parse returns");

    assert_eq!(
        returns.args.get("result"),
        Some(&json!("async: test-value")),
        "Expected async transformed result"
    );
}
