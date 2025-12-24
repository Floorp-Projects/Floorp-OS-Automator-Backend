// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

mod args;
mod dummy_plugin;
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
