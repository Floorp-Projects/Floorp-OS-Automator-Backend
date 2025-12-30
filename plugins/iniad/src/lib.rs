// INIAD-AI-MOP Plugin for Sapphillon
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

// ============================================================================
// Plugin Function Definitions
// ============================================================================

pub fn iniad_ai_mop_chat_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.iniad_ai_mop.chat".to_string(),
        function_name: "iniad_ai_mop.chat".to_string(),
        description: "Sends a chat request to INIAD-AI-MOP OpenAI API with custom system and user prompts."
            .to_string(),
        permissions: iniad_ai_mop_plugin_permissions(),
        arguments: "String: systemPrompt, String: userPrompt".to_string(),
        returns: "String: AI response content".to_string(),
    }
}

pub fn iniad_ai_mop_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.iniad_ai_mop".to_string(),
        package_name: "INIAD-AI-MOP | GPT 5 nano".to_string(),
        description: "A plugin to interact with INIAD-AI-MOP OpenAI API.".to_string(),
        functions: vec![iniad_ai_mop_chat_plugin_function()],
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

pub fn core_iniad_ai_mop_chat_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.iniad_ai_mop.chat".to_string(),
        "iniad_ai_mop.chat".to_string(),
        "Sends a chat request to INIAD-AI-MOP OpenAI API with custom system and user prompts.".to_string(),
        op2_iniad_ai_mop_chat(),
        Some(include_str!("00_iniad.js").to_string()),
    )
}

pub fn core_iniad_ai_mop_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.iniad_ai_mop".to_string(),
        "INIAD-AI-MOP | GPT 5 nano".to_string(),
        vec![core_iniad_ai_mop_chat_plugin()],
    )
}

// ============================================================================
// Permission Definitions
// ============================================================================

pub fn iniad_ai_mop_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "AI Access".to_string(),
        description: "Allows the plugin to access INIAD-AI-MOP OpenAI API.".to_string(),
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
        None => vec![],
    };
    _permission_check_backend(allowed, function_id, required_perms)?;
    Ok(())
}

// ============================================================================
// Op2 Functions (Deno Runtime Operations)
// ============================================================================

#[op2]
#[string]
pub fn op2_iniad_ai_mop_chat(
    state: &mut OpState,
    #[string] system_prompt: String,
    #[string] user_prompt: String,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(
        state,
        &iniad_ai_mop_chat_plugin_function().function_id,
        iniad_ai_mop_plugin_permissions(),
    )?;

    let api_key = std::env::var("INIAD_API_KEY")
        .map_err(|_| JsErrorBox::new("Error", "INIAD_API_KEY environment variable not set"))?;

    let agent = ureq::Agent::new_with_defaults();

    let request_body = json!({
        "model": "gpt-5-nano",
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

    let mut res = agent
        .post("https://api.openai.iniad.org/api/v1/chat/completions")
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send(serde_json::to_string(&request_body).unwrap())
        .map_err(|e| JsErrorBox::new("Error", format!("API request failed: {}", e)))?;

    if res.status() != 200 {
        let status = res.status();
        let text = res.body_mut().read_to_string().unwrap_or_default();
        return Err(JsErrorBox::new(
            "Error",
            format!("API returned error {}: {}", status, text),
        ));
    }

    let response_text = res
        .body_mut()
        .read_to_string()
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to read response body: {}", e)))?;

    let response_body: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to parse response JSON: {}", e)))?;

    let content = response_body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| JsErrorBox::new("Error", "Invalid API response format"))?;

    Ok(content.trim().to_string())
}
