mod args;
mod server;
mod services;

use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};

use args::{Args, Command};
use server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger with the log level from command line arguments
    env_logger::Builder::from_default_env()
        .filter_level(args.loglevel.clone().into())
        .init();

    match args.command {
        Command::Start => {
            // Start the gRPC server and demonstrate client communication
            info!("Starting gRPC server...");
            debug!("Log level set to: {:?}", args.loglevel);

            // Start server in a background task
            let server_handle = tokio::spawn(async {
                if let Err(e) = start_server().await {
                    error!("Server error: {}", e);
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
