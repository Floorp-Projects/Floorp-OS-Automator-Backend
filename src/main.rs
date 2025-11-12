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

mod args;
mod init;
mod server;
mod services;
mod workflow;

#[allow(unused)]
mod global;

/// System Configuration
#[allow(unused)]
mod sysconfig;

use anyhow::Result;
use clap::Parser;

#[allow(unused)]
use log::{debug, error, info, warn};

use args::{Args, Command};
use server::start_server; // bring `up`/`down` methods into scope

#[allow(unused)]
pub(crate) static GLOBAL_STATE: global::GlobalState = global::GlobalState::new();

/// Bootstraps the application, wiring logging, migrations, and the gRPC server lifecycle.
///
/// # Arguments
///
/// This entry point takes no arguments beyond the implicit command-line parsing performed by `clap`.
///
/// # Returns
///
/// Returns `Ok(())` when the server exits cleanly, or an error propagated from initialization tasks.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing/logging once (combine settings to avoid double init)
    let log_level_tracing: tracing::Level = args.loglevel.clone().into();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            args.loglevel.to_string(),
        ))
        // keep ORM and debug-related verbosity and useful thread info
        .with_max_level(log_level_tracing)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    // Display application information
    let app_info = sysconfig::sysconfig().app_info();
    for line in app_info.lines() {
        log::info!("{line}");
    }
    // END

    // Check db_url
    info!("Using database URL: {}", args.db_url);
    if args.db_url == "sqlite:memory:" {
        warn!("Using in-memory SQLite database. Data will not be persisted.");
    }
    // Initialize Database Connection

    GLOBAL_STATE.async_set_db_url(args.db_url.clone()).await;

    match args.command {
        Command::Start => {
            // Initialize system (migrations, etc.)

            init::initialize_system(&args).await?;

            // Start server in a background task
            let server_handle = tokio::spawn(async {
                if let Err(e) = start_server().await {
                    error!("Server error: {e}");
                }
            });

            // Wait a moment for server to start
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Keep server running
            info!("Server running on [::1]:50051. Press Ctrl+C to stop.");
            server_handle.await?;
        }
    }

    Ok(())
}
