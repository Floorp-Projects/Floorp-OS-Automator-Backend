// LLM Chat Plugin for Sapphillon (Ollama backend)
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{
    CheckPermissionResult, PluginFunctionPermissions, check_permission,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use serde_json::json;
use std::sync::{Arc, Mutex};

const DEFAULT_OLLAMA_MODEL: &str = "gemma3:4b";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

// ============================================================================
// Plugin Function Definitions
// ============================================================================

pub fn llm_chat_chat_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.llm_chat.chat".to_string(),
        function_name: "llm_chat.chat".to_string(),
        description: "Sends a chat request to a local Ollama instance with system and user prompts.".to_string(),
        permissions: llm_chat_plugin_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn llm_chat_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.llm_chat".to_string(),
        package_name: "LLM | Ollama".to_string(),
        provider_id: "".to_string(),
        description: "A plugin to interact with a local Ollama LLM instance.".to_string(),
        functions: vec![llm_chat_chat_plugin_function()],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

// ============================================================================
// Core Plugin Functions (for Deno runtime integration)
// ============================================================================

pub fn core_llm_chat_chat_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.llm_chat.chat".to_string(),
        "llm_chat.chat".to_string(),
        "Sends a chat request to a local Ollama instance with system and user prompts.".to_string(),
        op2_llm_chat_chat(),
        Some(include_str!("00_llm_chat.js").to_string()),
    )
}

pub fn core_llm_chat_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.llm_chat".to_string(),
        "LLM Chat | Ollama".to_string(),
        vec![core_llm_chat_chat_plugin()],
    )
}

// ============================================================================
// Permission Definitions
// ============================================================================

pub fn llm_chat_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "AI Access".to_string(),
        description: "Allows the plugin to access local Ollama LLM APIs.".to_string(),
        permission_type: PermissionType::NetAccess as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

// ============================================================================
// Permission Check Helpers
// ============================================================================

fn _permission_check_backend(
    allow: Vec<PluginFunctionPermissions>,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let required_permissions = sapphillon_core::permission::Permissions {
        permissions: required_perms,
    };

    let allowed_permissions = {
        let permissions_vec = allow;
        permissions_vec
            .into_iter()
            .find(|p| p.plugin_function_id == function_id || p.plugin_function_id == "*")
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions {
                permissions: vec![],
            })
    };

    let permission_check_result = check_permission(&allowed_permissions, &required_permissions);

    match permission_check_result {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "Error",
            format!("PermissionDenied. Missing Permissions: {}", perm),
        )),
    }
}

fn permission_check(
    state: &mut OpState,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p.clone(),
        // Default: grant required permissions when none are set
        None => vec![PluginFunctionPermissions {
            plugin_function_id: function_id.to_string(),
            permissions: sapphillon_core::permission::Permissions {
                permissions: required_perms.clone(),
            },
        }],
    };
    _permission_check_backend(allowed, function_id, required_perms)?;
    Ok(())
}

// ============================================================================
// Op2 Functions (Deno Runtime Operations)
// ============================================================================

#[op2]
#[string]
pub fn op2_llm_chat_chat(
    state: &mut OpState,
    #[string] system_prompt: String,
    #[string] user_prompt: String,
) -> std::result::Result<String, JsErrorBox> {
    let model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.to_string());

    let base_url = std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_OLLAMA_BASE_URL.to_string());

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));

    let request_body = json!({
        "model": model,
        "stream": false,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ]
    });

    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent();

    let body_str = serde_json::to_string(&request_body)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to serialize request: {}", e)))?;

    // Retry up to 3 times with exponential backoff (5s, 10s, 20s)
    let body = {
        let mut last_err: Option<String> = None;
        let mut result_body: Option<String> = None;
        let backoff_secs: &[u64] = &[5, 10, 20];
        for attempt in 0..3u32 {
            if attempt > 0 {
                let wait = backoff_secs.get(attempt as usize - 1).copied().unwrap_or(20);
                log::info!(
                    "Retrying Ollama request (attempt {}, backoff {}s)...",
                    attempt + 1,
                    wait
                );
                std::thread::sleep(std::time::Duration::from_secs(wait));
            }
            match agent
                .post(&url)
                .header("Content-Type", "application/json")
                .send(&body_str)
            {
                Ok(mut response) => {
                    if response.status() != 200 {
                        let status = response.status();
                        let err_body = response.body_mut().read_to_string().unwrap_or_default();
                        log::warn!(
                            "Ollama returned status {} (attempt {}): {}",
                            status,
                            attempt + 1,
                            err_body
                        );
                        last_err = Some(format!("Ollama status {}: {}", status, err_body));
                        continue;
                    }
                    match response.body_mut().read_to_string() {
                        Ok(b) => {
                            result_body = Some(b);
                            break;
                        }
                        Err(e) => {
                            last_err =
                                Some(format!("Failed to read Ollama response: {}", e));
                            continue;
                        }
                    }
                }
                Err(err) => {
                    log::warn!("Ollama request error (attempt {}): {}", attempt + 1, err);
                    last_err = Some(format!("Ollama request failed: {}", err));
                    continue;
                }
            }
        }
        match result_body {
            Some(b) => b,
            None => {
                return Err(JsErrorBox::new(
                    "Error",
                    last_err.unwrap_or_else(|| "Ollama request failed after retries".to_string()),
                ));
            }
        }
    };

    let response_body: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to parse Ollama response JSON: {}", e)))?;

    // Ollama /api/chat returns { "message": { "content": "..." }, ... }
    let content = response_body
        .get("message")
        .and_then(|msg| msg.get("content"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| JsErrorBox::new("Error", "Invalid Ollama response format: missing message.content"))?;

    Ok(content.trim().to_string())
}
