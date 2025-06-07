mod args;
mod grpc_server;

use anyhow::Result;
use clap::Parser;

use args::{Args, Command};
use grpc_server::start_grpc_server;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Start => {
            // Start the gRPC server and demonstrate client communication
            println!("Starting gRPC server...");

            // Start server in a background task
            let server_handle = tokio::spawn(async {
                if let Err(e) = start_grpc_server().await {
                    eprintln!("Server error: {}", e);
                }
            });

            // Wait a moment for server to start
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Keep server running
            println!("Server running on [::1]:50051. Press Ctrl+C to stop.");
            server_handle.await?;
        }
    }

    Ok(())
}
