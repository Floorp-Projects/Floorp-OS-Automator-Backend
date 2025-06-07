use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the gRPC server
    Start,
    /// Get weather information
    Weather {
        /// Latitude for the weather forecast
        #[arg(short, long)]
        latitude: f64,

        /// Longitude for the weather forecast
        #[arg(short = 'L', long)] // Changed short option to 'L'
        longitude: f64,
    },
}
