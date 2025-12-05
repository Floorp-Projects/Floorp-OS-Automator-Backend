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

    // Register Internal Plugins
    register_internal_plugins().await?;

    debug!("Initializing Completed.");
    debug!("Global State: {:?}", &GLOBAL_STATE);
    Ok(())
}

async fn setup_database() -> Result<()> {
    // Run migrations immediately after setting DB URL so the schema
    // is ready before the server starts accepting requests.
    info!("Running database migrations...");

    let db_path = match GLOBAL_STATE.async_get_db_url().await {
        // String starts with "sqlite://"
        url if url.starts_with("sqlite://") => {
            // Extract the path after "sqlite://"
            Some(url.trim_start_matches("sqlite://").to_string())
        }
        url if url == "sqlite::memory:" => {
            // In-memory database
            Some(":memory:".to_string())
        }

        _ => {
            error!("Database migrations are only supported for SQLite databases in this version.");
            None
        }
    };

    if db_path.is_none() {
        return Err(anyhow::anyhow!("Unsupported database type for migrations"));
    }

    // If DB path is no db files, create the db file
    match db_path.as_deref() {
        Some(path) if path != ":memory:" && !std::path::Path::new(path).exists() => {
            info!("Database file does not exist. Creating new database at: {path}");
            match std::fs::File::create(path) {
                Ok(_) => info!("Database file created successfully."),
                Err(e) => {
                    error!("Failed to create database file: {e:#?}");
                    return Err(Error::new(e));
                }
            }
        }
        _ => {}
    }

    let database_connection =
        sea_orm::Database::connect(GLOBAL_STATE.async_get_db_url().await.as_str()).await;
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

async fn register_internal_plugins() -> Result<()> {
    use database::plugin::init_register_plugins;

    let database_connection = GLOBAL_STATE.get_db_connection().await?;

    let plugin_packages = crate::sysconfig::sysconfig().plugin_package;

    init_register_plugins(&database_connection, plugin_packages).await?;

    Ok(())
}
