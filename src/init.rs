// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use crate::GLOBAL_STATE;
use crate::args::Args;
use anyhow::{Error, Result};
use migration::MigratorTrait;

#[allow(unused)]
use log::{debug, error, info, warn};

pub async fn initialize_system(args: &Args) -> Result<()> {
    debug!("Initializing system...");
    debug!("Log level set to: {:?}", args.loglevel);

    // Init Database
    setup_database().await?;

    // Register Initial Plugins
    register_initial_plugins().await?;

    // Sync External Plugins with filesystem
    sync_ext_plugins().await?;

    // Register Initial Workflows
    register_initial_workflows().await?;

    debug!("Initializing Completed.");
    debug!("Global State: {:?}", &GLOBAL_STATE);
    Ok(())
}

async fn setup_database() -> Result<()> {
    // Run migrations immediately after setting DB URL so the schema
    // is ready before the server starts accepting requests.
    info!("Running database migrations...");

    let mut db_url = GLOBAL_STATE.async_get_db_url().await;

    if !db_url.starts_with("sqlite:") {
        error!("Database migrations are only supported for SQLite databases in this version.");
        return Err(anyhow::anyhow!("Unsupported database type for migrations"));
    }

    // Normalize the sqlite path portion for path checks while tolerating common forms
    // such as `sqlite::memory:`, `sqlite://:memory:`, and `sqlite:file::memory:?mode=memory&cache=shared`.
    let path_part = db_url
        .trim_start_matches("sqlite:")
        // Handle the optional `//` after the scheme (sqlite://<path>). We remove exactly
        // two slashes so absolute paths keep a single leading slash.
        .strip_prefix("//")
        .unwrap_or_else(|| db_url.trim_start_matches("sqlite:"));

    let is_memory = path_part.starts_with(":memory:")
        || path_part.starts_with("file::memory:")
        || db_url.contains("mode=memory");

    // Ensure all in-memory URLs use a shared cache so migrations and subsequent connections
    // see the same schema. This rewrites common short forms like `sqlite::memory:` into the
    // canonical shared-cache URL.
    if is_memory && !(db_url.contains("mode=memory") && db_url.contains("cache=shared")) {
        let normalized_memory_url = "sqlite:file:sapphillon?mode=memory&cache=shared".to_string();
        warn!(
            "Detected in-memory SQLite URL without shared cache; normalizing to {normalized_memory_url}"
        );
        db_url = normalized_memory_url.clone();
        GLOBAL_STATE.async_set_db_url(normalized_memory_url).await;
    }

    let path_part = db_url
        .trim_start_matches("sqlite:")
        .strip_prefix("//")
        .unwrap_or_else(|| db_url.trim_start_matches("sqlite:"));

    // If using a file-backed SQLite database, create the file when it does not yet exist.
    if !is_memory && !path_part.is_empty() && !std::path::Path::new(path_part).exists() {
        info!("Database file does not exist. Creating new database at: {path_part}");
        match std::fs::File::create(path_part) {
            Ok(_) => info!("Database file created successfully."),
            Err(e) => {
                error!("Failed to create database file: {e:#?}");
                return Err(Error::new(e));
            }
        }
    }

    let database_connection = sea_orm::Database::connect(db_url.as_str()).await;
    match database_connection {
        Ok(conn) => {
            // Attempt to run migrations from the `migration` crate.
            // If this fails, log the error and exit since the server
            // depends on a correct schema state.
            if let Err(e) = migration::Migrator::up(&conn, None).await {
                error!("Database migration failed: {e:#?}");
                // Ensure we don't continue in a bad state.
                std::process::exit(1);
            }

            // Mark DB as initialized so other tasks can proceed.
            GLOBAL_STATE.async_set_db_initialized(true).await;
            info!("Database migrations applied");
        }
        Err(e) => {
            error!("Failed to obtain DB connection for migrations: {e:#?}");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn register_initial_workflows() -> Result<()> {
    use database::workflow::{create_workflow, create_workflow_code};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let db = GLOBAL_STATE.get_db_connection().await?;
    let config = crate::sysconfig::sysconfig();

    for wf_def in config.initial_workflows {
        // Check if workflow with this display_name already exists
        let exists = entity::entity::workflow::Entity::find()
            .filter(entity::entity::workflow::Column::DisplayName.eq(&wf_def.display_name))
            .one(&db)
            .await?;

        if exists.is_none() {
            info!("Registering initial workflow: {}", wf_def.display_name);
            let wf = create_workflow(
                &db,
                wf_def.display_name.clone(),
                wf_def.description.clone(),
                0, // WORKFLOW_LANGUAGE_UNSPECIFIED
            )
            .await?;

            create_workflow_code(
                &db,
                wf_def.code.clone(),
                wf.id,
                vec![], // No initial plugins
                vec![],
            )
            .await?;
        }
    }
    Ok(())
}

async fn register_initial_plugins() -> Result<()> {
    use database::plugin::init_register_plugins;

    let database_connection = GLOBAL_STATE.get_db_connection().await?;

    let plugin_packages = crate::sysconfig::sysconfig().initial_plugins;

    init_register_plugins(&database_connection, plugin_packages).await?;

    Ok(())
}

/// Synchronizes external plugins between the filesystem and database.
///
/// - Plugins on filesystem but not in DB are registered
/// - Plugins in DB but not on filesystem are marked as `missing`
/// - Plugins in both locations have their `missing` flag cleared
async fn sync_ext_plugins() -> Result<()> {
    use crate::ext_plugin_manager::scan_ext_plugin_dir;
    use database::ext_plugin::{
        create_ext_plugin_package, list_ext_plugin_packages, mark_ext_plugin_missing,
    };

    let db = GLOBAL_STATE.get_db_connection().await?;
    let save_dir = GLOBAL_STATE.get_ext_plugin_save_dir().await;

    info!("Syncing external plugins from directory: {}", save_dir);

    // 1. Get all registered plugins from DB
    let db_plugins = list_ext_plugin_packages(&db).await?;

    // 2. Scan filesystem for installed plugins
    let fs_plugins = scan_ext_plugin_dir(&save_dir)?;

    let mut synced_count = 0;
    let mut new_count = 0;
    let mut missing_count = 0;

    // 3. Check each DB plugin against filesystem
    for db_plugin in &db_plugins {
        if fs_plugins.contains(&db_plugin.plugin_package_id) {
            // Plugin exists on filesystem - ensure not marked as missing
            if db_plugin.missing {
                mark_ext_plugin_missing(&db, &db_plugin.plugin_package_id, false).await?;
                info!("External plugin recovered: {}", db_plugin.plugin_package_id);
            }
            synced_count += 1;
        } else {
            // Plugin missing from filesystem
            if !db_plugin.missing {
                mark_ext_plugin_missing(&db, &db_plugin.plugin_package_id, true).await?;
                warn!(
                    "External plugin missing from filesystem: {}",
                    db_plugin.plugin_package_id
                );
            }
            missing_count += 1;
        }
    }

    // 4. Register new plugins found on filesystem but not in DB
    for fs_plugin_id in &fs_plugins {
        if !db_plugins
            .iter()
            .any(|p| &p.plugin_package_id == fs_plugin_id)
        {
            let install_dir = format!("{}/{}", save_dir, fs_plugin_id);
            create_ext_plugin_package(&db, fs_plugin_id.clone(), install_dir).await?;
            info!("Registered new external plugin: {}", fs_plugin_id);
            new_count += 1;
        }
    }

    info!(
        "External plugin sync complete: {} synced, {} new, {} missing",
        synced_count, new_count, missing_count
    );

    Ok(())
}
