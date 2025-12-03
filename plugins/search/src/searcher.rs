// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Common trait and types for file searchers across platforms.

use deno_error::JsErrorBox;

/// A trait for file search implementations across different platforms.
pub trait FileSearcher: Send + Sync {
    /// Search for files matching the query within the given root path.
    ///
    /// # Arguments
    /// * `root_path` - The root directory to search in (may be ignored by indexed searchers)
    /// * `query` - The search query (file name pattern)
    ///
    /// # Returns
    /// A vector of file paths matching the query
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox>;

    /// Check if this searcher is available on the current system.
    fn is_available(&self) -> bool;

    /// Get the name of this searcher for debugging/logging purposes.
    fn name(&self) -> &'static str;
}
