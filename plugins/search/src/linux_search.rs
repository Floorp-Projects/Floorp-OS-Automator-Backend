// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Linux native file search implementations.
//!
//! This module provides searchers for multiple Linux desktop environments:
//! 1. `TrackerSearcher` - GNOME Tracker (GNOME/GTK environments)
//! 2. `BalooSearcher` - KDE Baloo (KDE Plasma environments)
//! 3. `LocateSearcher` - mlocate/plocate (command-line, available everywhere)

use crate::searcher::FileSearcher;
use deno_error::JsErrorBox;
use std::sync::OnceLock;

/// Searcher using GNOME Tracker via D-Bus.
///
/// Tracker is GNOME's file indexing and search framework.
/// It uses SPARQL for queries and provides rich metadata.
pub struct TrackerSearcher;

impl TrackerSearcher {
    pub fn new() -> Self {
        Self
    }

    /// Check if Tracker is available via D-Bus.
    fn check_availability() -> bool {
        use zbus::blocking::Connection;

        // Try to connect to the session bus and check if Tracker service exists
        Connection::session()
            .and_then(|conn| {
                // Check if the Tracker Miner FS service is available
                let proxy = zbus::blocking::fdo::DBusProxy::new(&conn)?;
                let names = proxy.list_names()?;
                Ok(names.iter().any(|name| {
                    name.as_str().contains("Tracker") || name.as_str().contains("tracker")
                }))
            })
            .unwrap_or(false)
    }
}

impl Default for TrackerSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for TrackerSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        use zbus::blocking::Connection;

        let conn = Connection::session()
            .map_err(|e| JsErrorBox::new("SearchError", format!("D-Bus connection failed: {e}")))?;

        // Build SPARQL query for Tracker
        // Tracker 3.x uses org.freedesktop.Tracker3.Endpoint
        let sparql = if root_path.is_empty() || root_path == "/" {
            format!(
                r#"
                SELECT ?url WHERE {{
                    ?file a nfo:FileDataObject ;
                          nie:url ?url ;
                          nfo:fileName ?name .
                    FILTER(CONTAINS(LCASE(?name), LCASE("{}")))
                }}
                LIMIT 1000
                "#,
                query.replace('"', "\\\"")
            )
        } else {
            format!(
                r#"
                SELECT ?url WHERE {{
                    ?file a nfo:FileDataObject ;
                          nie:url ?url ;
                          nfo:fileName ?name .
                    FILTER(CONTAINS(LCASE(?name), LCASE("{}")))
                    FILTER(STRSTARTS(?url, "file://{}"))
                }}
                LIMIT 1000
                "#,
                query.replace('"', "\\\""),
                root_path
            )
        };

        // Try Tracker 3.x first, then fall back to Tracker 2.x
        Self::query_tracker3(&conn, &sparql).or_else(|_| Self::query_tracker2(&conn, &sparql))
    }

    fn is_available(&self) -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(Self::check_availability)
    }

    fn name(&self) -> &'static str {
        "Tracker"
    }
}

impl TrackerSearcher {
    /// Query Tracker 3.x via D-Bus
    fn query_tracker3(
        conn: &zbus::blocking::Connection,
        sparql: &str,
    ) -> Result<Vec<String>, JsErrorBox> {
        // Call the Tracker3 endpoint
        let message = conn
            .call_method(
                Some("org.freedesktop.Tracker3.Miner.Files"),
                "/org/freedesktop/Tracker3/Endpoint",
                Some("org.freedesktop.Tracker3.Endpoint"),
                "Query",
                &(sparql,),
            )
            .map_err(|e| JsErrorBox::new("SearchError", format!("Tracker3 query failed: {e}")))?;

        let body = message.body();
        let results: Vec<Vec<String>> = body
            .deserialize()
            .map_err(|e| JsErrorBox::new("SearchError", format!("Failed to parse results: {e}")))?;

        // Extract URLs and convert to paths
        let paths: Vec<String> = results
            .into_iter()
            .filter_map(|row| row.first().cloned())
            .filter_map(|url| {
                // Convert file:// URL to path
                url.strip_prefix("file://")
                    .map(|s| urlencoding::decode(s).unwrap_or_default().into_owned())
            })
            .collect();

        Ok(paths)
    }

    /// Query Tracker 2.x via D-Bus (legacy)
    fn query_tracker2(
        conn: &zbus::blocking::Connection,
        sparql: &str,
    ) -> Result<Vec<String>, JsErrorBox> {
        // Tracker 2.x uses org.freedesktop.Tracker1.Resources
        let message = conn
            .call_method(
                Some("org.freedesktop.Tracker1"),
                "/org/freedesktop/Tracker1/Resources",
                Some("org.freedesktop.Tracker1.Resources"),
                "SparqlQuery",
                &(sparql,),
            )
            .map_err(|e| JsErrorBox::new("SearchError", format!("Tracker2 query failed: {e}")))?;

        let body = message.body();
        let results: Vec<Vec<String>> = body
            .deserialize()
            .map_err(|e| JsErrorBox::new("SearchError", format!("Failed to parse results: {e}")))?;

        let paths: Vec<String> = results
            .into_iter()
            .filter_map(|row| row.first().cloned())
            .filter_map(|url| {
                url.strip_prefix("file://")
                    .map(|s| urlencoding::decode(s).unwrap_or_default().into_owned())
            })
            .collect();

        Ok(paths)
    }
}

/// Searcher using KDE Baloo via D-Bus.
///
/// Baloo is KDE's file indexing and search framework.
pub struct BalooSearcher;

impl BalooSearcher {
    pub fn new() -> Self {
        Self
    }

    /// Check if Baloo is available via D-Bus.
    fn check_availability() -> bool {
        use zbus::blocking::Connection;

        Connection::session()
            .and_then(|conn| {
                let proxy = zbus::blocking::fdo::DBusProxy::new(&conn)?;
                let names = proxy.list_names()?;
                Ok(names
                    .iter()
                    .any(|name| name.as_str().contains("baloo") || name.as_str().contains("Baloo")))
            })
            .unwrap_or(false)
    }
}

impl Default for BalooSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for BalooSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        // Baloo uses the baloosearch command or baloo6/baloo5 D-Bus interface
        // The D-Bus interface varies between KDE versions, so we'll use the CLI tool
        // which provides a stable interface

        let mut cmd = std::process::Command::new("baloosearch");
        cmd.arg(query);

        if !root_path.is_empty() && root_path != "/" {
            cmd.current_dir(root_path);
        }

        let output = cmd.output().map_err(|e| {
            JsErrorBox::new("SearchError", format!("Failed to run baloosearch: {e}"))
        })?;

        if !output.status.success() {
            return Err(JsErrorBox::new(
                "SearchError",
                format!(
                    "baloosearch failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| {
                // Filter by root_path if specified
                if root_path.is_empty() || root_path == "/" {
                    true
                } else {
                    line.starts_with(root_path)
                }
            })
            .take(1000)
            .map(|s| s.to_string())
            .collect();

        Ok(results)
    }

    fn is_available(&self) -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(|| {
            // Check if baloosearch command exists
            std::process::Command::new("which")
                .arg("baloosearch")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
                && Self::check_availability()
        })
    }

    fn name(&self) -> &'static str {
        "Baloo"
    }
}

/// Searcher using locate/mlocate/plocate command.
///
/// This is a fallback for systems without Tracker or Baloo.
/// Note: The locate database may not be up-to-date.
pub struct LocateSearcher;

impl LocateSearcher {
    pub fn new() -> Self {
        Self
    }

    /// Find which locate variant is available.
    fn find_locate_command() -> Option<&'static str> {
        ["plocate", "mlocate", "locate"].into_iter().find(|cmd| {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }
}

impl Default for LocateSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for LocateSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        let locate_cmd = Self::find_locate_command()
            .ok_or_else(|| JsErrorBox::new("SearchError", "No locate command found"))?;

        let mut cmd = std::process::Command::new(locate_cmd);
        cmd.arg("-i"); // Case insensitive
        cmd.arg("-l").arg("1000"); // Limit results
        cmd.arg(query);

        let output = cmd.output().map_err(|e| {
            JsErrorBox::new("SearchError", format!("Failed to run {locate_cmd}: {e}"))
        })?;

        // locate returns non-zero if no matches found, which is not an error for us
        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| {
                if root_path.is_empty() || root_path == "/" {
                    true
                } else {
                    line.starts_with(root_path)
                }
            })
            .map(|s| s.to_string())
            .collect();

        Ok(results)
    }

    fn is_available(&self) -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(|| Self::find_locate_command().is_some())
    }

    fn name(&self) -> &'static str {
        "Locate"
    }
}

/// Get the best available Linux searcher.
///
/// Priority order:
/// 1. GNOME Tracker (if available)
/// 2. KDE Baloo (if available)
/// 3. locate/mlocate/plocate (if available)
/// 4. None (will fall back to walkdir)
pub fn get_linux_searcher() -> Option<Box<dyn FileSearcher>> {
    // Check Tracker first (GNOME)
    let tracker = TrackerSearcher::new();
    if tracker.is_available() {
        return Some(Box::new(tracker));
    }

    // Check Baloo (KDE)
    let baloo = BalooSearcher::new();
    if baloo.is_available() {
        return Some(Box::new(baloo));
    }

    // Fall back to locate
    let locate = LocateSearcher::new();
    if locate.is_available() {
        return Some(Box::new(locate));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_availability_check() {
        let searcher = TrackerSearcher::new();
        let _ = searcher.is_available();
    }

    #[test]
    fn test_baloo_availability_check() {
        let searcher = BalooSearcher::new();
        let _ = searcher.is_available();
    }

    #[test]
    fn test_locate_availability_check() {
        let searcher = LocateSearcher::new();
        let _ = searcher.is_available();
    }
}
