mod args;
mod server;
mod services;

use anyhow::Result;
use clap::Parser;
use log::{info, error};

use args::{Args, Command};
use server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with default level set to info
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    let args = Args::parse();

    match args.command {
        Command::Start => {
            // Start the gRPC server and demonstrate client communication
            info!("Starting gRPC server...");

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
