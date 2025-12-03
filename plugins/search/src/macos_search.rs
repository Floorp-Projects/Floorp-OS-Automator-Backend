// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! macOS native file search implementation using Spotlight (MDQuery).

use crate::searcher::FileSearcher;
use deno_error::JsErrorBox;
use std::sync::OnceLock;

/// Searcher using macOS Spotlight (NSMetadataQuery/MDQuery).
///
/// Spotlight is Apple's built-in file indexing and search system.
/// It provides fast indexed search across the entire filesystem.
pub struct SpotlightSearcher;

impl SpotlightSearcher {
    pub fn new() -> Self {
        Self
    }

    /// Check if Spotlight is available and enabled.
    fn check_availability() -> bool {
        // Check if mdutil command exists and Spotlight is enabled
        // We could also try to query the index directly
        std::process::Command::new("mdutil")
            .args(["-s", "/"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("Indexing enabled")
            })
            .unwrap_or(false)
    }
}

impl Default for SpotlightSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for SpotlightSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        use mdquery_rs::{MDQueryBuilder, MDQueryScope};

        // Determine search scope
        let scopes = if root_path.is_empty() || root_path == "/" {
            vec![MDQueryScope::Computer]
        } else {
            vec![MDQueryScope::Path(root_path.to_string())]
        };

        // Build the query
        // MDQuery uses NSPredicate-style queries
        let query_result = MDQueryBuilder::default()
            .name_like(query)
            .build(scopes, Some(1000));

        let mdquery = match query_result {
            Ok(q) => q,
            Err(e) => {
                return Err(JsErrorBox::new(
                    "SearchError",
                    format!("Failed to build MDQuery: {:?}", e),
                ));
            }
        };

        // Execute the query
        let results = match mdquery.execute() {
            Ok(items) => items,
            Err(e) => {
                return Err(JsErrorBox::new(
                    "SearchError",
                    format!("MDQuery execution failed: {:?}", e),
                ));
            }
        };

        // Extract paths from results
        let paths: Vec<String> = results
            .into_iter()
            .filter_map(|item| item.path().map(|p| p.to_string_lossy().into_owned()))
            .collect();

        Ok(paths)
    }

    fn is_available(&self) -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(Self::check_availability)
    }

    fn name(&self) -> &'static str {
        "Spotlight"
    }
}

/// Get the macOS searcher if available.
pub fn get_macos_searcher() -> Option<Box<dyn FileSearcher>> {
    let spotlight = SpotlightSearcher::new();
    if spotlight.is_available() {
        return Some(Box::new(spotlight));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spotlight_availability_check() {
        // This test just checks that the availability check doesn't panic
        let searcher = SpotlightSearcher::new();
        let _ = searcher.is_available();
    }
}
