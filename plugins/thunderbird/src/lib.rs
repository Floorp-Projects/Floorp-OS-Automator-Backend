// Sapphillon - Thunderbird Plugin
// SPDX-FileCopyrightText: 2025 Floorp Projects
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
use std::process::Command;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Serialize, Deserialize, Debug)]
pub struct ThunderbirdIdentity {
    pub name: String,
    pub email: String,
    pub profile: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalendarEvent {
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub date: String,
}

// ============================================================================
// Plugin Functions Metadata
// ============================================================================

pub fn thunderbird_get_identity_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getIdentity".to_string(),
        function_name: "Get Thunderbird Identity".to_string(),
        description: "Retrieves the user's name and email from Thunderbird.".to_string(),
        permissions: thunderbird_read_permissions(),
        arguments: "None".to_string(),
        returns: "JSON: { name, email, profile }".to_string(),
    }
}

pub fn thunderbird_get_calendar_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getCalendarEvents".to_string(),
        function_name: "Get Thunderbird Calendar Events".to_string(),
        description: "Retrieves upcoming calendar events from Thunderbird.".to_string(),
        permissions: thunderbird_read_permissions(),
        arguments: "Number: days (optional, default 14)".to_string(),
        returns: "JSON: Array of { title, start_time, end_time, date }".to_string(),
    }
}

pub fn thunderbird_get_profile_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getProfile".to_string(),
        function_name: "Get Thunderbird Profile".to_string(),
        description: "Finds and returns the default Thunderbird profile name.".to_string(),
        permissions: thunderbird_read_permissions(),
        arguments: "None".to_string(),
        returns: "String: profile name".to_string(),
    }
}

pub fn thunderbird_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.thunderbird".to_string(),
        package_name: "Thunderbird".to_string(),
        description: "A plugin to read Thunderbird identity and calendar data.".to_string(),
        functions: vec![
            thunderbird_get_identity_function(),
            thunderbird_get_calendar_function(),
            thunderbird_get_profile_function(),
        ],
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
// Core Plugin Functions
// ============================================================================

pub fn core_thunderbird_get_identity() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.thunderbird.getIdentity".to_string(),
        "Get Thunderbird Identity".to_string(),
        "Retrieves the user's name and email from Thunderbird.".to_string(),
        op2_thunderbird_get_identity(),
        None,
    )
}

pub fn core_thunderbird_get_calendar_events() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.thunderbird.getCalendarEvents".to_string(),
        "Get Thunderbird Calendar Events".to_string(),
        "Retrieves upcoming calendar events from Thunderbird.".to_string(),
        op2_thunderbird_get_calendar_events(),
        None,
    )
}

pub fn core_thunderbird_get_profile() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.thunderbird.getProfile".to_string(),
        "Get Thunderbird Profile".to_string(),
        "Finds and returns the default Thunderbird profile name.".to_string(),
        op2_thunderbird_get_profile(),
        Some(include_str!("00_thunderbird.js").to_string()),
    )
}

pub fn core_thunderbird_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.thunderbird".to_string(),
        "Thunderbird".to_string(),
        vec![
            core_thunderbird_get_identity(),
            core_thunderbird_get_calendar_events(),
            core_thunderbird_get_profile(),
        ],
    )
}

// ============================================================================
// Permission Handling
// ============================================================================

fn thunderbird_read_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Thunderbird Read Access".to_string(),
        description: "Allows reading Thunderbird profile data (identity, calendar).".to_string(),
        permission_type: PermissionType::FilesystemRead as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

fn permission_check(state: &mut OpState) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p,
        None => &vec![PluginFunctionPermissions {
            plugin_function_id: thunderbird_get_identity_function().function_id,
            permissions: sapphillon_core::permission::Permissions {
                permissions: thunderbird_read_permissions(),
            },
        }],
    };

    let required = sapphillon_core::permission::Permissions {
        permissions: thunderbird_read_permissions(),
    };

    let allowed_perms = allowed
        .iter()
        .find(|p| {
            p.plugin_function_id.starts_with("app.sapphillon.thunderbird")
                || p.plugin_function_id == "*"
        })
        .map(|p| p.permissions.clone())
        .unwrap_or_else(|| sapphillon_core::permission::Permissions {
            permissions: vec![],
        });

    match check_permission(&allowed_perms, &required) {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "PermissionDenied",
            perm.to_string(),
        )),
    }
}

// ============================================================================
// Operations Implementation
// ============================================================================

fn find_thunderbird_profile() -> anyhow::Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let profiles_dir = format!("{}/Library/Thunderbird/Profiles", home);
    
    let output = Command::new("ls")
        .arg(&profiles_dir)
        .output()?;
    
    if output.status.success() {
        let profiles = String::from_utf8_lossy(&output.stdout);
        let first_profile = profiles.lines().next().unwrap_or("").trim();
        if !first_profile.is_empty() {
            return Ok(first_profile.to_string());
        }
    }
    
    Err(anyhow::anyhow!("No Thunderbird profile found"))
}

fn get_identity_from_prefs(profile: &str) -> anyhow::Result<ThunderbirdIdentity> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let prefs_path = format!("{}/Library/Thunderbird/Profiles/{}/prefs.js", home, profile);
    
    // Read name
    let name_output = Command::new("sh")
        .arg("-c")
        .arg(format!("grep 'mail.identity.id1.fullName' '{}' | head -1", prefs_path))
        .output()?;
    
    let name_line = String::from_utf8_lossy(&name_output.stdout);
    let name = extract_quoted_value(&name_line).unwrap_or_default();
    
    // Read email
    let email_output = Command::new("sh")
        .arg("-c")
        .arg(format!("grep 'mail.identity.id1.useremail' '{}' | head -1", prefs_path))
        .output()?;
    
    let email_line = String::from_utf8_lossy(&email_output.stdout);
    let email = extract_quoted_value(&email_line).unwrap_or_default();
    
    Ok(ThunderbirdIdentity {
        name,
        email,
        profile: profile.to_string(),
    })
}

fn extract_quoted_value(line: &str) -> Option<String> {
    // Extract value from: user_pref("key", "value");
    let re = regex::Regex::new(r#""([^"]+)"[^"]*$"#).ok()?;
    re.captures(line).and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn get_calendar_events_from_db(profile: &str, days: i64) -> anyhow::Result<Vec<CalendarEvent>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let db_path = format!("{}/Library/Thunderbird/Profiles/{}/calendar-data/cache.sqlite", home, profile);
    let tmp_db = "/tmp/thunderbird_cal_plugin.sqlite";
    
    // Copy DB to avoid lock issues
    let _ = Command::new("cp").arg(&db_path).arg(tmp_db).output()?;
    
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64;
    
    let future_micros = now_micros + (days * 24 * 60 * 60 * 1_000_000);
    
    let query = format!(
        "SELECT title, datetime(event_start/1000000,'unixepoch','localtime'), \
         datetime(event_end/1000000,'unixepoch','localtime') FROM cal_events \
         WHERE event_start >= {} AND event_start <= {} ORDER BY event_start;",
        now_micros, future_micros
    );
    
    let output = Command::new("sqlite3")
        .arg("-separator")
        .arg("|")
        .arg(tmp_db)
        .arg(&query)
        .output()?;
    
    // Clean up
    let _ = Command::new("rm").arg("-f").arg(tmp_db).output();
    
    let result = String::from_utf8_lossy(&output.stdout);
    let mut events = Vec::new();
    
    for line in result.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let start_time = parts[1].to_string();
            let date = start_time.split(' ').next().unwrap_or("").to_string();
            events.push(CalendarEvent {
                title: parts[0].to_string(),
                start_time,
                end_time: parts[2].to_string(),
                date,
            });
        }
    }
    
    Ok(events)
}

// ============================================================================
// Op2 Functions
// ============================================================================

#[op2]
#[string]
fn op2_thunderbird_get_identity(
    state: &mut OpState,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(state)?;
    
    let profile = find_thunderbird_profile()
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    let identity = get_identity_from_prefs(&profile)
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    serde_json::to_string(&identity)
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2]
#[string]
fn op2_thunderbird_get_calendar_events(
    state: &mut OpState,
    #[bigint] days: i64,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(state)?;
    
    let profile = find_thunderbird_profile()
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    let events = get_calendar_events_from_db(&profile, if days > 0 { days } else { 14 })
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    serde_json::to_string(&events)
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2]
#[string]
fn op2_thunderbird_get_profile(
    state: &mut OpState,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(state)?;
    
    find_thunderbird_profile()
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_profile() {
        // This test will only pass on machines with Thunderbird installed
        let result = find_thunderbird_profile();
        // Just check it doesn't panic
        println!("Profile result: {:?}", result);
    }

    #[test]
    fn test_extract_quoted_value() {
        let line = r#"user_pref("mail.identity.id1.fullName", "Test User");"#;
        let value = extract_quoted_value(line);
        assert_eq!(value, Some("Test User".to_string()));
    }
}
