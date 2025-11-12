// Sapphillon
// Copyright 2025 Yuta Takahashi
//
// This file is part of Sapphillon
//
// Sapphillon is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::GLOBAL_STATE;
use crate::args::Args;
use anyhow::Result;
use migration::MigratorTrait;

#[allow(unused)]
use log::{debug, error, info, warn};

pub async fn initialize_system(args: &Args) -> Result<()> {
    debug!("Initializing system...");
    debug!("Log level set to: {:?}", args.loglevel);

    setup_database().await?;

    debug!("Initializing Completed.");
    debug!("Global State: {:?}", &GLOBAL_STATE);
    Ok(())
}

async fn setup_database() -> Result<()> {
    // Run migrations immediately after setting DB URL so the schema
    // is ready before the server starts accepting requests.
    info!("Running database migrations...");
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
