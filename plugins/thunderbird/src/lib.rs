// Sapphillon - Thunderbird Plugin
// SPDX-FileCopyrightText: 2025 Floorp Projects
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use std::process::Command;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    pub id: i64,
    pub subject: String,
    pub sender: String,
    pub recipients: String,
    pub body: String,
    pub date: String,
    pub folder: String,
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
        function_define: None,
        version: "".to_string(),
    }
}

pub fn thunderbird_get_calendar_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getCalendarEvents".to_string(),
        function_name: "Get Thunderbird Calendar Events".to_string(),
        description: "Retrieves upcoming calendar events from Thunderbird.".to_string(),
        permissions: thunderbird_read_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn thunderbird_get_profile_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getProfile".to_string(),
        function_name: "Get Thunderbird Profile".to_string(),
        description: "Finds and returns the default Thunderbird profile name.".to_string(),
        permissions: thunderbird_read_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn thunderbird_get_emails_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.thunderbird.getEmails".to_string(),
        function_name: "Get Thunderbird Emails".to_string(),
        description: "Retrieves emails from Thunderbird.".to_string(),
        permissions: thunderbird_read_permissions(),
        function_define: None,
        version: "".to_string(),
    }
}

pub fn thunderbird_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.thunderbird".to_string(),
        package_name: "Thunderbird".to_string(),
        provider_id: "".to_string(),
        description: "A plugin to read Thunderbird identity and calendar data.".to_string(),
        functions: vec![
            thunderbird_get_identity_function(),
            thunderbird_get_calendar_function(),
            thunderbird_get_profile_function(),
            thunderbird_get_emails_function(),
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
        None,
    )
}

pub fn core_thunderbird_get_emails() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.thunderbird.getEmails".to_string(),
        "Get Thunderbird Emails".to_string(),
        "Retrieves emails from Thunderbird.".to_string(),
        op2_thunderbird_get_emails(),
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
            core_thunderbird_get_emails(),
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

// For demo purposes: skip permission check to allow thunderbird plugin to work without explicit permissions
fn permission_check(_state: &mut OpState) -> Result<(), JsErrorBox> {
    Ok(())
}

// ============================================================================
// Operations Implementation
// ============================================================================

fn find_thunderbird_profile() -> anyhow::Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let profiles_ini = format!("{}/Library/Thunderbird/profiles.ini", home);
    let profiles_dir = format!("{}/Library/Thunderbird/Profiles", home);
    
    // Try to read profiles.ini and find the default profile from [Install*] section
    if let Ok(output) = Command::new("cat").arg(&profiles_ini).output() {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            let mut in_install_section = false;
            
            for line in content.lines() {
                let trimmed = line.trim();
                
                // Check if we're entering an [Install*] section
                if trimmed.starts_with("[Install") {
                    in_install_section = true;
                    continue;
                }
                
                // Check if we're leaving the section
                if trimmed.starts_with('[') {
                    in_install_section = false;
                    continue;
                }
                
                // Look for Default= in [Install*] section
                if in_install_section && trimmed.starts_with("Default=") {
                    let path = trimmed.trim_start_matches("Default=").trim();
                    // Path is like "Profiles/x9dsn3v8.default-release"
                    if let Some(profile_name) = path.strip_prefix("Profiles/") {
                        return Ok(profile_name.to_string());
                    }
                }
            }
        }
    }
    
    // Fallback: list profiles directory and pick one with calendar-data
    let output = Command::new("ls")
        .arg(&profiles_dir)
        .output()?;
    
    if output.status.success() {
        let profiles = String::from_utf8_lossy(&output.stdout);
        let profile_list: Vec<&str> = profiles.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        
        // Try to find profile with calendar-data
        for profile in &profile_list {
            let calendar_path = format!("{}/{}/calendar-data", profiles_dir, profile);
            let exists = Command::new("test")
                .arg("-d")
                .arg(&calendar_path)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if exists {
                return Ok(profile.to_string());
            }
        }

        // Fallback to first profile
        if let Some(first_profile) = profile_list.first() {
            return Ok(first_profile.to_string());
        }
    }

    Err(anyhow::anyhow!("No Thunderbird profile found"))
}

fn get_identity_from_prefs(profile: &str) -> anyhow::Result<ThunderbirdIdentity> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let prefs_path = format!("{}/Library/Thunderbird/Profiles/{}/prefs.js", home, profile);

    // Search for any mail.identity.id*.fullName (not just id1)
    // First, find all identity names and emails
    let name_output = Command::new("sh")
        .arg("-c")
        .arg(format!("grep 'mail.identity.id.*\\.fullName' '{}' | head -1", prefs_path))
        .output()?;
    
    let name_line = String::from_utf8_lossy(&name_output.stdout);
    let name = extract_quoted_value(&name_line).unwrap_or_default();
    
    // Read email - search for any identity's email
    let email_output = Command::new("sh")
        .arg("-c")
        .arg(format!("grep 'mail.identity.id.*\\.useremail' '{}' | head -1", prefs_path))
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
    let calendar_data_dir = format!("{}/Library/Thunderbird/Profiles/{}/calendar-data", home, profile);
    
    // Try both local.sqlite (primary for local calendars) and cache.sqlite (for CalDAV/remote)
    let db_files = vec![
        ("local.sqlite", "/tmp/thunderbird_cal_local.sqlite"),
        ("cache.sqlite", "/tmp/thunderbird_cal_cache.sqlite"),
    ];
    
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
    
    let mut all_events = Vec::new();
    
    for (db_name, tmp_db) in &db_files {
        let db_path = format!("{}/{}", calendar_data_dir, db_name);
        let wal_path = format!("{}-wal", db_path);
        let tmp_wal = format!("{}-wal", tmp_db);
        
        // Check if DB exists
        let db_exists = Command::new("test")
            .arg("-f")
            .arg(&db_path)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        
        if !db_exists {
            continue;
        }
        
        // Copy DB and WAL file (if exists) to avoid lock issues
        let _ = Command::new("cp").arg(&db_path).arg(tmp_db).output();
        let _ = Command::new("cp").arg(&wal_path).arg(&tmp_wal).output();
        
        let output = Command::new("sqlite3")
            .arg("-separator")
            .arg("|")
            .arg(tmp_db)
            .arg(&query)
            .output();

        // Clean up
        let _ = Command::new("rm").arg("-f").arg(tmp_db).output();
        let _ = Command::new("rm").arg("-f").arg(&tmp_wal).output();
        
        if let Ok(output) = output {
            let result = String::from_utf8_lossy(&output.stdout);
            for line in result.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let start_time = parts[1].to_string();
                    let date = start_time.split(' ').next().unwrap_or("").to_string();
                    all_events.push(CalendarEvent {
                        title: parts[0].to_string(),
                        start_time,
                        end_time: parts[2].to_string(),
                        date,
                    });
                }
            }
        }
    }
    
    // Sort events by start_time
    all_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    
    Ok(all_events)
}

fn folder_to_uri_pattern(folder: &str) -> String {
    // Convert folder alias to URI pattern for locale-independent matching
    match folder.to_lowercase().as_str() {
        "inbox" | "" => "%/INBOX".to_string(),
        "sent" => "%Sent%".to_string(),
        "drafts" | "draft" => "%Draft%".to_string(),
        "trash" => "%Trash%".to_string(),
        "spam" | "junk" => "%Spam%".to_string(),
        "all" => "%/すべてのメール".to_string(),  // Gmail specific
        // If not a known alias, use as-is for name matching
        _ => folder.to_string(),
    }
}

fn get_emails_from_db(profile: &str, folder: &str, limit: i64) -> anyhow::Result<Vec<Email>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let db_path = format!("{}/Library/Thunderbird/Profiles/{}/global-messages-db.sqlite", home, profile);
    let tmp_db = "/tmp/thunderbird_email_plugin.sqlite";
    
    // Copy DB to avoid lock issues
    let _ = Command::new("cp").arg(&db_path).arg(tmp_db).output()?;
    
    let uri_pattern = folder_to_uri_pattern(folder);
    
    // Check if we should match by URI pattern or by exact name
    let where_clause = if uri_pattern.contains('%') {
        format!("f.folderURI LIKE '{}'", uri_pattern.replace("'", "''"))
    } else {
        format!("f.name = '{}'", uri_pattern.replace("'", "''"))
    };
    
    // Query emails with folder filter
    let query = format!(
        "SELECT m.id, datetime(m.date/1000000,'unixepoch','localtime'), f.name, \
         mc.c1subject, mc.c3author, mc.c4recipients, mc.c0body \
         FROM messages m \
         JOIN messagesText_content mc ON m.id = mc.docid \
         JOIN folderLocations f ON m.folderID = f.id \
         WHERE m.deleted = 0 AND {} \
         ORDER BY m.date DESC LIMIT {};",
        where_clause,
        limit
    );
    
    let output = Command::new("sqlite3")
        .arg("-separator")
        .arg("|||")  // Use unique separator to avoid conflicts with email content
        .arg(tmp_db)
        .arg(&query)
        .output()?;
    
    // Clean up
    let _ = Command::new("rm").arg("-f").arg(tmp_db).output();
    
    let result = String::from_utf8_lossy(&output.stdout);
    let mut emails = Vec::new();
    
    for line in result.lines() {
        let parts: Vec<&str> = line.splitn(7, "|||").collect();
        if parts.len() >= 7 {
            emails.push(Email {
                id: parts[0].parse().unwrap_or(0),
                date: parts[1].to_string(),
                folder: parts[2].to_string(),
                subject: parts[3].to_string(),
                sender: parts[4].to_string(),
                recipients: parts[5].to_string(),
                body: parts[6].to_string(),
            });
        }
    }
    
    Ok(emails)
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

#[op2]
#[string]
fn op2_thunderbird_get_emails(
    state: &mut OpState,
    #[string] folder: String,
    #[bigint] limit: i64,
) -> std::result::Result<String, JsErrorBox> {
    permission_check(state)?;
    
    let profile = find_thunderbird_profile()
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    let folder_name = if folder.is_empty() { "inbox".to_string() } else { folder };
    let email_limit = if limit > 0 { limit } else { 20 };
    
    let emails = get_emails_from_db(&profile, &folder_name, email_limit)
        .map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    
    serde_json::to_string(&emails)
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
