// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! macOS native file search implementation using Spotlight (MDQuery).

use crate::searcher::FileSearcher;
use crate::walkdir_search::WalkdirSearcher;
use deno_error::JsErrorBox;
use std::sync::OnceLock;

/// Searcher using macOS Spotlight (NSMetadataQuery/MDQuery).
///
/// Spotlight is Apple's built-in file indexing and search system.
/// It provides fast indexed search across the entire filesystem.
/// Falls back to walkdir for directories not indexed by Spotlight
/// (e.g., temporary directories).
pub struct SpotlightSearcher {
    walkdir_fallback: WalkdirSearcher,
}

impl SpotlightSearcher {
    pub fn new() -> Self {
        Self {
            walkdir_fallback: WalkdirSearcher::new(),
        }
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

    /// Check if a path is likely to be indexed by Spotlight.
    /// Spotlight typically doesn't index temporary directories.
    fn is_path_indexed(path: &str) -> bool {
        // Paths that are typically not indexed by Spotlight
        let non_indexed_prefixes = [
            "/private/var/folders/", // Temp directories
            "/var/folders/",         // Temp directories (symlink)
            "/tmp/",                 // System temp
            "/private/tmp/",         // System temp (full path)
        ];

        !non_indexed_prefixes
            .iter()
            .any(|prefix| path.starts_with(prefix))
    }

    /// Perform Spotlight search using MDQuery.
    fn spotlight_search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        use mdquery_rs::{MDQueryBuilder, MDQueryScope};

        // Determine search scope
        let scopes = if root_path.is_empty() || root_path == "/" {
            vec![MDQueryScope::Computer]
        } else {
            vec![MDQueryScope::from_path(root_path)]
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
}

impl Default for SpotlightSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for SpotlightSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        // If the path is not indexed by Spotlight, use walkdir fallback directly
        if !root_path.is_empty() && root_path != "/" && !Self::is_path_indexed(root_path) {
            return self.walkdir_fallback.search(root_path, query);
        }

        // Try Spotlight search first
        let spotlight_results = self.spotlight_search(root_path, query)?;

        // If Spotlight returns no results and we have a specific path,
        // fallback to walkdir (the path might not be indexed)
        if spotlight_results.is_empty() && !root_path.is_empty() && root_path != "/" {
            return self.walkdir_fallback.search(root_path, query);
        }

        Ok(spotlight_results)
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
