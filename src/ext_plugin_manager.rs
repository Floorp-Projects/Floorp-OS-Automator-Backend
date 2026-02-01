// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! External plugin management module.
//!
//! This module provides functions for installing and uninstalling external
//! plugin packages. It manages both the filesystem storage and database
//! registration of plugins.

use anyhow::{Context, Result};
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::fs;
use std::path::Path;use std::sync::Arc;
/// Installs an external plugin package.
///
/// Creates the directory structure `{save_dir}/{author_id}/{package_id}/{version}/`
/// and writes the `package.js` file. Also registers the plugin in the database.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `save_dir` - Base directory for saving plugins
/// * `author_id` - Plugin author identifier
/// * `package_id` - Plugin package identifier
/// * `version` - Plugin version
/// * `package_js_content` - The JavaScript content to save
/// * `metadata_content` - Optional metadata.json content to save
///
/// # Returns
///
/// Returns the full plugin package ID (`author_id/package_id/version`) on success.
pub async fn install_ext_plugin(
    db: &DatabaseConnection,
    save_dir: &str,
    author_id: &str,
    package_id: &str,
    version: &str,
    package_js_content: &[u8],
    metadata_content: Option<&[u8]>,
) -> Result<String> {
    use database::ext_plugin::{create_ext_plugin_package, get_ext_plugin_package};

    let plugin_package_id = format!("{author_id}/{package_id}/{version}");
    let install_dir = format!("{save_dir}/{author_id}/{package_id}/{version}");
    let package_js_path = format!("{install_dir}/package.js");
    let metadata_path = format!("{install_dir}/metadata.json");

    // Check if plugin already exists
    let existing = get_ext_plugin_package(db, &plugin_package_id).await?;
    if existing.is_some() {
        anyhow::bail!("External plugin already installed: {plugin_package_id}");
    }

    // Create directory structure
    fs::create_dir_all(&install_dir)
        .with_context(|| format!("Failed to create plugin directory: {install_dir}"))?;

    // Write package.js file
    fs::write(&package_js_path, package_js_content)
        .with_context(|| format!("Failed to write package.js: {package_js_path}"))?;
    
    // Write metadata.json file if provided
    if let Some(metadata) = metadata_content {
        fs::write(&metadata_path, metadata)
            .with_context(|| format!("Failed to write metadata.json: {metadata_path}"))?;
    }

    // Register in database
    create_ext_plugin_package(db, plugin_package_id.clone(), install_dir)
        .await
        .with_context(|| format!("Failed to register plugin in database: {plugin_package_id}"))?;

    log::info!("Installed external plugin: {plugin_package_id}");

    Ok(plugin_package_id)
}

/// Uninstalls an external plugin package.
///
/// Removes the plugin files from the filesystem and deletes the database record.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `plugin_package_id` - Full plugin ID (author_id/package_id/version)
///
/// # Returns
///
/// Returns `Ok(())` on success.
pub async fn uninstall_ext_plugin(db: &DatabaseConnection, plugin_package_id: &str) -> Result<()> {
    use database::ext_plugin::{delete_ext_plugin_package, get_ext_plugin_package};

    // Get the plugin record
    let plugin = get_ext_plugin_package(db, plugin_package_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Plugin not found: {plugin_package_id}"))?;

    // Remove files from filesystem
    let install_path = Path::new(&plugin.install_dir);
    if install_path.exists() {
        fs::remove_dir_all(install_path).with_context(|| {
            format!("Failed to remove plugin directory: {}", plugin.install_dir)
        })?;

        // Try to clean up empty parent directories
        cleanup_empty_parent_dirs(install_path);
    }

    // Remove from database
    delete_ext_plugin_package(db, plugin_package_id)
        .await
        .with_context(|| format!("Failed to delete plugin from database: {plugin_package_id}"))?;

    log::info!("Uninstalled external plugin: {plugin_package_id}");

    Ok(())
}

/// Attempts to remove empty parent directories up to 3 levels.
fn cleanup_empty_parent_dirs(path: &Path) {
    let mut current = path.parent();
    for _ in 0..3 {
        if let Some(parent) = current {
            if parent.exists()
                && parent
                    .read_dir()
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false)
            {
                if fs::remove_dir(parent).is_err() {
                    break;
                }
            } else {
                break;
            }
            current = parent.parent();
        } else {
            break;
        }
    }
}

/// Scans a directory for installed external plugins.
///
/// Traverses the directory structure `{save_dir}/{author_id}/{package_id}/{version}/`
/// and returns plugin IDs for directories containing `package.js`.
///
/// # Arguments
///
/// * `save_dir` - Base directory to scan
///
/// # Returns
///
/// Returns a set of plugin package IDs found on the filesystem.
pub fn scan_ext_plugin_dir(save_dir: &str) -> Result<HashSet<String>> {
    let mut plugin_ids = HashSet::new();
    let base_path = Path::new(save_dir);

    if !base_path.exists() {
        return Ok(plugin_ids);
    }

    // Traverse: author-id/package-id/ver/package.js
    for author_entry in
        fs::read_dir(base_path).with_context(|| format!("Failed to read directory: {save_dir}"))?
    {
        let author_entry = author_entry?;
        if !author_entry.file_type()?.is_dir() {
            continue;
        }
        let author_id = author_entry.file_name().to_string_lossy().to_string();

        for package_entry in fs::read_dir(author_entry.path())? {
            let package_entry = package_entry?;
            if !package_entry.file_type()?.is_dir() {
                continue;
            }
            let package_id = package_entry.file_name().to_string_lossy().to_string();

            for version_entry in fs::read_dir(package_entry.path())? {
                let version_entry = version_entry?;
                if !version_entry.file_type()?.is_dir() {
                    continue;
                }
                let version = version_entry.file_name().to_string_lossy().to_string();

                // Check if package.js exists
                let package_js = version_entry.path().join("package.js");
                if package_js.exists() {
                    let plugin_id = format!("{author_id}/{package_id}/{version}");
                    plugin_ids.insert(plugin_id);
                }
            }
        }
    }

    Ok(plugin_ids)
}

/// Extracts function names from package.js content.
///
/// Looks for the pattern `functions: { funcName: { ... }, ... }` in the Sapphillon package format.
/// Returns a vector of function names found in the package.
///
/// # Arguments
///
/// * `package_js` - The JavaScript content of the package.js file
///
/// # Returns
///
/// A vector of function names extracted from the package.
fn extract_function_names(package_js: &str) -> Vec<String> {
    use regex::Regex;
    
    let mut function_names = Vec::new();
    
    // Look for function definitions in the Sapphillon.Package.functions object
    // Pattern: functions: { getIdentity: { ... }, getCalendarEvents: { ... } }
    // We need to find keys after "functions:" or "functions :"
    
    // First, try to find the functions block
    let functions_pattern = Regex::new(r"functions\s*:\s*\{").ok();
    
    if let Some(pattern) = functions_pattern {
        if let Some(mat) = pattern.find(package_js) {
            let start_pos = mat.end();
            let content_after_functions = &package_js[start_pos..];
            
            // Find function names: identifiers followed by ":"
            // Pattern: \b([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*\{
            let func_name_pattern = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*\{").ok();
            let next_key_pattern = Regex::new(r",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*\{").ok();
            
            // Extract the first function name
            if let Some(ref pattern) = func_name_pattern {
                if let Some(caps) = pattern.captures(content_after_functions) {
                    if let Some(name) = caps.get(1) {
                        function_names.push(name.as_str().to_string());
                        log::debug!("Found function: {}", name.as_str());
                    }
                }
            }
            
            // Extract subsequent function names
            if let Some(ref pattern) = next_key_pattern {
                for caps in pattern.captures_iter(content_after_functions) {
                    if let Some(name) = caps.get(1) {
                        let func_name = name.as_str().to_string();
                        if !function_names.contains(&func_name) {
                            function_names.push(func_name.clone());
                            log::debug!("Found function: {}", func_name);
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: also look for handler: async patterns as a backup
    if function_names.is_empty() {
        let handler_pattern = Regex::new(r"(\w+)\s*:\s*\{\s*(?:[^{}]*\bhandler\s*:)").ok();
        if let Some(pattern) = handler_pattern {
            for caps in pattern.captures_iter(package_js) {
                if let Some(name) = caps.get(1) {
                    let func_name = name.as_str().to_string();
                    // Skip meta and other non-function keys
                    if func_name != "meta" && func_name != "functions" && func_name != "Package" {
                        if !function_names.contains(&func_name) {
                            function_names.push(func_name);
                        }
                    }
                }
            }
        }
    }
    
    log::info!("Extracted {} function names from package.js", function_names.len());
    function_names
}

/// Loads all external plugins from database and merges them with internal plugins.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a vector of `Arc<dyn PluginPackageTrait>` containing only external plugins.
/// The caller is responsible for merging with internal plugins if needed.
pub async fn load_external_plugins(
    db: &DatabaseConnection,
) -> Result<Vec<Arc<dyn sapphillon_core::plugin::PluginPackageTrait>>> {
    use database::ext_plugin::list_ext_plugin_packages;
    use sapphillon_core::plugin::{CorePluginExternalFunction, CorePluginExternalPackage, PluginPackageTrait};
    use std::fs;
    use std::path::Path;

    log::info!("Starting to load external plugins from database...");

    let ext_plugins_db = list_ext_plugin_packages(db).await?;
    log::info!("Found {} external plugins in database", ext_plugins_db.len());

    let mut external_plugins = Vec::new();

    for ext_plugin in ext_plugins_db {
        log::info!("Processing external plugin: {}", ext_plugin.plugin_package_id);

        if ext_plugin.missing {
            log::warn!("Skipping missing plugin: {}", ext_plugin.plugin_package_id);
            continue; // Skip missing plugins
        }

        let install_dir = &ext_plugin.install_dir;
        let package_js_path = format!("{install_dir}/package.js");

        if !Path::new(&package_js_path).exists() {
            log::warn!("External plugin file not found: {package_js_path}");
            continue;
        }

        // Read package.js content
        let package_js = fs::read_to_string(&package_js_path)
            .with_context(|| format!("Failed to read package.js: {package_js_path}"))?;

        // Parse plugin_package_id to extract parts (format: author/package/version)
        let parts: Vec<&str> = ext_plugin.plugin_package_id.split('/').collect();
        if parts.len() < 3 {
            log::warn!("Invalid plugin_package_id format: {}", ext_plugin.plugin_package_id);
            continue;
        }

        let author_id = parts[0];
        let package_id = parts[1];
        let version = parts.get(2).unwrap_or(&"1.0.0");

        log::info!("Creating CorePluginExternalPackage for: {} (author={}, package={}, version={})",
            ext_plugin.plugin_package_id, author_id, package_id, version);

        // Extract function names from package.js
        let function_names = extract_function_names(&package_js);
        
        if function_names.is_empty() {
            log::warn!("No functions found in package.js for plugin: {}", ext_plugin.plugin_package_id);
            // Create a default function as fallback
            let ext_function = CorePluginExternalFunction::new(
                format!("{}-{}-{}-default", author_id, package_id, version),
                "default".to_string(),
                format!("External plugin: {}", package_id),
                package_id.to_string(),
                package_js.clone(),
                author_id.to_string(),
            );

            // Package ID must match CorePluginExternalFunction::get_package_id()
            // which returns "{author_id}.{package_name}"
            let full_package_id = format!("{}.{}", author_id, package_id);
            log::info!("Creating package with id: {}", full_package_id);

            let ext_package = CorePluginExternalPackage::new(
                full_package_id, // Must match get_package_id() for rsjs_bridge_core lookup
                package_id.to_string(),
                vec![ext_function],
                package_js,
            );

            external_plugins.push(Arc::new(ext_package) as Arc<dyn PluginPackageTrait>);
        } else {
            log::info!("Found {} functions in package.js: {:?}", function_names.len(), function_names);
            
            // Create external functions for each found function
            let ext_functions: Vec<CorePluginExternalFunction> = function_names
                .iter()
                .map(|func_name| {
                    CorePluginExternalFunction::new(
                        format!("{}-{}-{}-{}", author_id, package_id, version, func_name),
                        func_name.clone(),
                        format!("External plugin function: {}.{}", package_id, func_name),
                        package_id.to_string(),
                        package_js.clone(),
                        author_id.to_string(),
                    )
                })
                .collect();

            log::info!("Created {} CorePluginExternalFunction instances", ext_functions.len());

            // Package ID must match CorePluginExternalFunction::get_package_id()
            // which returns "{author_id}.{package_name}"
            // This creates namespace: globalThis.sapphillon.thunderbird.getIdentity()
            let full_package_id = format!("{}.{}", author_id, package_id);
            log::info!("Creating package with id: {}", full_package_id);

            let ext_package = CorePluginExternalPackage::new(
                full_package_id, // Must match get_package_id() for rsjs_bridge_core lookup
                package_id.to_string(),
                ext_functions,
                package_js,
            );

            external_plugins.push(Arc::new(ext_package) as Arc<dyn PluginPackageTrait>);
        }

        log::info!("Loaded external plugin: {}", ext_plugin.plugin_package_id);
    }

    log::info!("Total external plugins loaded: {}", external_plugins.len());
    Ok(external_plugins)
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::MigratorTrait;
    use sea_orm::Database;
    use tempfile::TempDir;

    async fn setup_db() -> Result<DatabaseConnection, sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        migration::Migrator::up(&db, None).await?;
        Ok(db)
    }

    #[tokio::test]
    async fn test_install_and_uninstall_ext_plugin() -> Result<()> {
        let db = setup_db().await?;
        let temp_dir = TempDir::new()?;
        let save_dir = temp_dir.path().to_string_lossy().to_string();

        // Install
        let plugin_id = install_ext_plugin(
            &db,
            &save_dir,
            "test-author",
            "test-package",
            "1.0.0",
            b"console.log('hello');",
            None, // No metadata
        )
        .await?;

        assert_eq!(plugin_id, "test-author/test-package/1.0.0");

        // Verify file exists
        let package_js_path = temp_dir
            .path()
            .join("test-author/test-package/1.0.0/package.js");
        assert!(package_js_path.exists());

        // Verify database record
        let record = database::ext_plugin::get_ext_plugin_package(&db, &plugin_id).await?;
        assert!(record.is_some());

        // Uninstall
        uninstall_ext_plugin(&db, &plugin_id).await?;

        // Verify file removed
        assert!(!package_js_path.exists());

        // Verify database record removed
        let record = database::ext_plugin::get_ext_plugin_package(&db, &plugin_id).await?;
        assert!(record.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_install_already_exists() -> Result<()> {
        let db = setup_db().await?;
        let temp_dir = TempDir::new()?;
        let save_dir = temp_dir.path().to_string_lossy().to_string();

        // Install first time
        install_ext_plugin(&db, &save_dir, "author", "pkg", "1.0.0", b"content", None).await?;

        // Try to install again
        let result =
            install_ext_plugin(&db, &save_dir, "author", "pkg", "1.0.0", b"new content", None).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already installed")
        );

        Ok(())
    }

    #[test]
    fn test_scan_ext_plugin_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let save_dir = temp_dir.path().to_string_lossy().to_string();

        // Create mock plugin structures
        let plugin1_dir = temp_dir.path().join("author1/pkg1/1.0.0");
        let plugin2_dir = temp_dir.path().join("author2/pkg2/2.0.0");
        let incomplete_dir = temp_dir.path().join("author3/pkg3/3.0.0"); // No package.js

        fs::create_dir_all(&plugin1_dir)?;
        fs::create_dir_all(&plugin2_dir)?;
        fs::create_dir_all(&incomplete_dir)?;

        fs::write(plugin1_dir.join("package.js"), b"content1")?;
        fs::write(plugin2_dir.join("package.js"), b"content2")?;

        let found = scan_ext_plugin_dir(&save_dir)?;

        assert_eq!(found.len(), 2);
        assert!(found.contains("author1/pkg1/1.0.0"));
        assert!(found.contains("author2/pkg2/2.0.0"));
        assert!(!found.contains("author3/pkg3/3.0.0"));

        Ok(())
    }

    #[test]
    fn test_scan_empty_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let found = scan_ext_plugin_dir(&temp_dir.path().to_string_lossy())?;
        assert!(found.is_empty());
        Ok(())
    }

    #[test]
    fn test_scan_nonexistent_dir() -> Result<()> {
        let found = scan_ext_plugin_dir("/nonexistent/path/12345")?;
        assert!(found.is_empty());
        Ok(())
    }
}
