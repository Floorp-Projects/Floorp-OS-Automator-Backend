use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Latitude for the weather forecast
    #[arg(short, long)]
    pub latitude: f64,

    /// Longitude for the weather forecast
    #[arg(short = 'L', long)] // Changed short option to 'L'
    pub longitude: f64,
}
