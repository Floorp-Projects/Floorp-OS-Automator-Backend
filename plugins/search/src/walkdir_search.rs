// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Walkdir-based file search implementation.
//!
//! This is a cross-platform fallback searcher that works on all operating systems
//! by traversing the filesystem directly.

use crate::searcher::FileSearcher;
use deno_error::JsErrorBox;
use walkdir::WalkDir;

/// Searcher using walkdir for filesystem traversal.
///
/// This is a cross-platform fallback that works everywhere but may be slow
/// for large directory trees since it doesn't use any indexing.
pub struct WalkdirSearcher;

impl WalkdirSearcher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WalkdirSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for WalkdirSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        let root = if root_path.is_empty() { "/" } else { root_path };

        let results: Vec<String> = WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(query))
            .take(1000) // Limit results to prevent memory issues
            .map(|e| e.path().to_string_lossy().into_owned())
            .collect();

        Ok(results)
    }

    fn is_available(&self) -> bool {
        // Walkdir is always available as a fallback
        true
    }

    fn name(&self) -> &'static str {
        "Walkdir"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_walkdir_search() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap().to_string();

        // Create some files (avoiding "file" in directory name to prevent false matches)
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("doc1.txt"), "hello").unwrap();
        fs::write(dir.path().join("subdir/doc2.log"), "world").unwrap();
        fs::write(dir.path().join("another.data"), "test").unwrap();

        let searcher = WalkdirSearcher::new();

        // Test searching for existing file
        let results = searcher.search(&dir_path, "doc1").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("doc1.txt"));

        // Test searching for non-existing file
        let results = searcher.search(&dir_path, "nonexistent").unwrap();
        assert_eq!(results.len(), 0);

        // Test searching for multiple files
        let results = searcher.search(&dir_path, "doc").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_walkdir_is_always_available() {
        let searcher = WalkdirSearcher::new();
        assert!(searcher.is_available());
    }
}
