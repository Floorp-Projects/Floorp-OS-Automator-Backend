// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! Plugin installer module.
//!
//! Handles downloading and installing external plugins from various URI schemes
//! (https, http, file).

use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::path::Path;

/// Result of a plugin installation operation.
#[derive(Debug)]
pub struct InstallResult {
    pub plugin_package_id: String,
    #[allow(dead_code)]
    pub install_dir: String,
}

/// Error types for plugin installation.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("URI is required")]
    EmptyUri,

    #[error("unsupported URI scheme: {0}. Only https, http, and file are allowed")]
    UnsupportedScheme(String),

    #[error("invalid URI format: {0}")]
    InvalidUriFormat(String),

    #[error("failed to download plugin: {0}")]
    DownloadFailed(String),

    #[error("failed to read file: {0}")]
    FileReadFailed(String),

    #[error("plugin already installed: {0}")]
    AlreadyInstalled(String),

    #[error("installation failed: {0}")]
    InstallFailed(String),
}

/// Supported URI schemes for plugin installation.
#[derive(Debug, Clone, PartialEq)]
pub enum UriScheme {
    Https,
    Http,
    File,
}

impl UriScheme {
    /// Parse a URI string and return the scheme.
    pub fn parse(uri: &str) -> Result<(Self, &str), InstallError> {
        if uri.starts_with("https://") {
            Ok((UriScheme::Https, uri.trim_start_matches("https://")))
        } else if uri.starts_with("http://") {
            Ok((UriScheme::Http, uri.trim_start_matches("http://")))
        } else if uri.starts_with("file://") {
            Ok((UriScheme::File, uri.trim_start_matches("file://")))
        } else {
            // Try to extract scheme for error message
            let scheme = uri.split("://").next().unwrap_or("unknown");
            Err(InstallError::UnsupportedScheme(scheme.to_string()))
        }
    }
}

/// Metadata extracted from a plugin URI.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub author_id: String,
    pub package_id: String,
    pub version: String,
}

impl PluginMetadata {
    /// Extract plugin metadata from URI path segments.
    ///
    /// Expected format: host/author/package/version/... (for http/https)
    /// or /path/to/author/package/version/... (for file)
    pub fn from_uri_path(path: &str, scheme: &UriScheme) -> Result<Self, InstallError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        match scheme {
            UriScheme::Https | UriScheme::Http => {
                // Format: host/author/package/version/...
                if parts.len() < 4 {
                    return Err(InstallError::InvalidUriFormat(
                        "URI must contain host/author/package/version path segments".to_string(),
                    ));
                }
                Ok(Self {
                    author_id: parts[1].to_string(),
                    package_id: parts[2].to_string(),
                    version: parts[3].to_string(),
                })
            }
            UriScheme::File => {
                // Format: /path/to/.../author/package/version/file.js
                // Take last 3 directories before filename
                if parts.len() < 4 {
                    return Err(InstallError::InvalidUriFormat(
                        "file path must contain author/package/version directories".to_string(),
                    ));
                }
                let len = parts.len();
                Ok(Self {
                    author_id: parts[len - 4].to_string(),
                    package_id: parts[len - 3].to_string(),
                    version: parts[len - 2].to_string(),
                })
            }
        }
    }
}

/// Fetch plugin content from a URI.
///
/// Supports https, http, and file schemes.
pub async fn fetch_plugin_content(uri: &str) -> Result<Vec<u8>, InstallError> {
    let (scheme, path) = UriScheme::parse(uri)?;

    match scheme {
        UriScheme::Https | UriScheme::Http => {
            let response = reqwest::get(uri)
                .await
                .map_err(|e| InstallError::DownloadFailed(e.to_string()))?;

            if !response.status().is_success() {
                return Err(InstallError::DownloadFailed(format!(
                    "HTTP status: {}",
                    response.status()
                )));
            }

            response
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| InstallError::DownloadFailed(e.to_string()))
        }
        UriScheme::File => {
            let file_path = Path::new(path);
            std::fs::read(file_path)
                .map_err(|e| InstallError::FileReadFailed(format!("{}: {}", path, e)))
        }
    }
}

/// Install a plugin from a URI.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `save_dir` - Base directory to save plugins
/// * `uri` - URI of the plugin (https://, http://, or file://)
///
/// # Returns
///
/// Returns the installation result including the plugin package ID.
pub async fn install_plugin_from_uri(
    db: &DatabaseConnection,
    save_dir: &str,
    uri: &str,
) -> Result<InstallResult, InstallError> {
    use crate::ext_plugin_manager::install_ext_plugin;

    // Validate URI
    let uri = uri.trim();
    if uri.is_empty() {
        return Err(InstallError::EmptyUri);
    }

    // Parse scheme and extract metadata
    let (scheme, path) = UriScheme::parse(uri)?;
    let metadata = PluginMetadata::from_uri_path(path, &scheme)?;

    // Fetch content
    let content = fetch_plugin_content(uri).await?;

    // Install
    let plugin_package_id = install_ext_plugin(
        db,
        save_dir,
        &metadata.author_id,
        &metadata.package_id,
        &metadata.version,
        &content,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("already installed") {
            InstallError::AlreadyInstalled(e.to_string())
        } else {
            InstallError::InstallFailed(e.to_string())
        }
    })?;

    let install_dir = format!(
        "{}/{}/{}/{}",
        save_dir, metadata.author_id, metadata.package_id, metadata.version
    );

    Ok(InstallResult {
        plugin_package_id,
        install_dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_scheme_parse_https() {
        let (scheme, path) =
            UriScheme::parse("https://example.com/author/pkg/1.0.0/file.js").unwrap();
        assert_eq!(scheme, UriScheme::Https);
        assert_eq!(path, "example.com/author/pkg/1.0.0/file.js");
    }

    #[test]
    fn test_uri_scheme_parse_http() {
        let (scheme, path) =
            UriScheme::parse("http://example.com/author/pkg/1.0.0/file.js").unwrap();
        assert_eq!(scheme, UriScheme::Http);
        assert_eq!(path, "example.com/author/pkg/1.0.0/file.js");
    }

    #[test]
    fn test_uri_scheme_parse_file() {
        let (scheme, path) =
            UriScheme::parse("file:///home/user/author/pkg/1.0.0/file.js").unwrap();
        assert_eq!(scheme, UriScheme::File);
        assert_eq!(path, "/home/user/author/pkg/1.0.0/file.js");
    }

    #[test]
    fn test_uri_scheme_parse_unsupported() {
        let result = UriScheme::parse("ftp://example.com/file.js");
        assert!(matches!(result, Err(InstallError::UnsupportedScheme(_))));
    }

    #[test]
    fn test_metadata_from_http_uri() {
        let meta = PluginMetadata::from_uri_path(
            "example.com/myauthor/mypkg/2.0.0/package.js",
            &UriScheme::Https,
        )
        .unwrap();
        assert_eq!(meta.author_id, "myauthor");
        assert_eq!(meta.package_id, "mypkg");
        assert_eq!(meta.version, "2.0.0");
    }

    #[test]
    fn test_metadata_from_file_uri() {
        let meta = PluginMetadata::from_uri_path(
            "/home/user/plugins/myauthor/mypkg/1.0.0/package.js",
            &UriScheme::File,
        )
        .unwrap();
        assert_eq!(meta.author_id, "myauthor");
        assert_eq!(meta.package_id, "mypkg");
        assert_eq!(meta.version, "1.0.0");
    }

    #[test]
    fn test_metadata_invalid_path() {
        let result = PluginMetadata::from_uri_path("example.com/short", &UriScheme::Https);
        assert!(matches!(result, Err(InstallError::InvalidUriFormat(_))));
    }
}
