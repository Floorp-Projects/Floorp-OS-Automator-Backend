// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Windows native file search implementation using Windows Search API.
//!
//! This module provides `WindowsSearchApiSearcher` which uses the built-in
//! Windows Search indexer (Windows Index Search) available on all modern Windows versions.

use crate::searcher::FileSearcher;
use deno_error::JsErrorBox;
use std::sync::OnceLock;

/// Searcher using the Windows Search API (Windows Desktop Search).
///
/// This uses the built-in Windows Search indexer which is available on all modern Windows versions.
pub struct WindowsSearchApiSearcher;

impl WindowsSearchApiSearcher {
    pub fn new() -> Self {
        Self
    }

    /// Check if Windows Search service is available.
    fn check_availability() -> bool {
        use windows::core::BSTR;
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };
        use windows::Win32::System::Search::{CSearchManager, ISearchManager};

        unsafe {
            // Try to initialize COM
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            // Try to create search manager
            let result: Result<ISearchManager, _> =
                CoCreateInstance(&CSearchManager, None, CLSCTX_ALL);

            if let Ok(manager) = result {
                // Try to get the system catalog
                let catalog_name = BSTR::from("SystemIndex");
                manager.GetCatalog(&catalog_name).is_ok()
            } else {
                false
            }
        }
    }
}

impl Default for WindowsSearchApiSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearcher for WindowsSearchApiSearcher {
    fn search(&self, root_path: &str, query: &str) -> Result<Vec<String>, JsErrorBox> {
        use windows::core::BSTR;
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };
        use windows::Win32::System::Search::{CSearchManager, ISearchManager, ISearchQueryHelper};

        unsafe {
            // Initialize COM
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            // Create search manager
            let manager: ISearchManager = CoCreateInstance(&CSearchManager, None, CLSCTX_ALL)
                .map_err(|e| {
                    JsErrorBox::new(
                        "SearchError",
                        format!("Failed to create SearchManager: {}", e),
                    )
                })?;

            // Get the system catalog
            let catalog_name = BSTR::from("SystemIndex");
            let catalog = manager.GetCatalog(&catalog_name).map_err(|e| {
                JsErrorBox::new("SearchError", format!("Failed to get catalog: {}", e))
            })?;

            // Get query helper
            let query_helper: ISearchQueryHelper = catalog
                .GetCrawlScopeManager()
                .and_then(|_| catalog.GetQueryHelper())
                .map_err(|e| {
                    JsErrorBox::new("SearchError", format!("Failed to get query helper: {}", e))
                })?;

            // Set query properties
            query_helper
                .SetQueryContentLocale(0x0409) // English locale
                .ok();
            query_helper.SetQueryMaxResults(1000).ok();

            // Build the search query
            // Windows Search uses SQL-like syntax
            let scope = if root_path.is_empty() || root_path == "/" {
                String::new()
            } else {
                let normalized = root_path.replace('/', "\\");
                format!(" AND SCOPE='file:{}'", normalized)
            };

            let user_query = BSTR::from(format!("*{}*", query));
            let _sql_query = query_helper
                .GenerateSQLFromUserQuery(&user_query)
                .map_err(|e| {
                    JsErrorBox::new("SearchError", format!("Failed to generate SQL: {}", e))
                })?;

            // For now, we'll use a simpler approach with OLE DB
            // This is a simplified implementation - full implementation would use ICommand
            let mut results = Vec::new();

            // Build a direct SQL query
            let sql = format!(
                "SELECT System.ItemPathDisplay FROM SystemIndex WHERE System.FileName LIKE '%{}%'{}",
                query.replace('\'', "''"),
                scope
            );

            // Execute using ADO-style connection
            // Note: Full implementation would use ICommand/IRowset interfaces
            // For simplicity, we'll shell out to PowerShell as a fallback
            let output = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        r#"
                        $connection = New-Object -ComObject ADODB.Connection
                        $connection.Open("Provider=Search.CollatorDSO;Extended Properties='Application=Windows';")
                        $recordset = $connection.Execute("{}")
                        while (-not $recordset.EOF) {{
                            $recordset.Fields.Item("System.ItemPathDisplay").Value
                            $recordset.MoveNext()
                        }}
                        $recordset.Close()
                        $connection.Close()
                        "#,
                        sql.replace('"', "\"\"")
                    ),
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                results = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }

            Ok(results)
        }
    }

    fn is_available(&self) -> bool {
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(Self::check_availability)
    }

    fn name(&self) -> &'static str {
        "WindowsSearchAPI"
    }
}

/// Get the Windows searcher using Windows Search API.
pub fn get_windows_searcher() -> Option<Box<dyn FileSearcher>> {
    let windows_search = WindowsSearchApiSearcher::new();
    if windows_search.is_available() {
        return Some(Box::new(windows_search));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_search_api_availability_check() {
        // This test just checks that the availability check doesn't panic
        let searcher = WindowsSearchApiSearcher::new();
        let _ = searcher.is_available();
    }
}
