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
use std::path::Path;

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
) -> Result<String> {
    use database::ext_plugin::{create_ext_plugin_package, get_ext_plugin_package};

    let plugin_package_id = format!("{author_id}/{package_id}/{version}");
    let install_dir = format!("{save_dir}/{author_id}/{package_id}/{version}");
    let package_js_path = format!("{install_dir}/package.js");

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
        install_ext_plugin(&db, &save_dir, "author", "pkg", "1.0.0", b"content").await?;

        // Try to install again
        let result =
            install_ext_plugin(&db, &save_dir, "author", "pkg", "1.0.0", b"new content").await;

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
