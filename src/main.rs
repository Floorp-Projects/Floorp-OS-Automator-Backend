mod args;
mod grpc_client;
mod grpc_server;
mod utils;
mod weather;

use anyhow::Result;
use chrono::{Duration, Local, NaiveDate, TimeZone};
use clap::Parser;
use reqwest::Client;

use args::{Args, Command};
use grpc_client::send_hello_request;
use grpc_server::start_grpc_server;
use utils::calculate_average;
use weather::fetch_weather_data;

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

            // Send a test request
            println!("Sending test hello world request...");
            if let Err(e) = send_hello_request().await {
                eprintln!("Client error: {}", e);
            }

            // Keep server running
            println!("Server running on [::1]:50051. Press Ctrl+C to stop.");
            server_handle.await?;
        }
        Command::Weather {
            latitude,
            longitude,
        } => {
            let client = Client::new();

            println!(
                "Fetching weather data for latitude: {}, longitude: {}...",
                latitude, longitude
            );

            let weather_data = fetch_weather_data(&client, latitude, longitude).await?;

            let now = Local::now().date_naive();
            let yesterday = now - Duration::days(1);

            let (today_temps, yesterday_temps) =
                extract_temperatures(&weather_data, now, yesterday);

            println!("\n--- Hourly Weather Data (Every 6 Hours) ---");
            print_hourly_weather(&weather_data, now, yesterday);

            println!("\n--- Average Temperatures ---");
            print_average_temperature("Today's", &today_temps);
            print_average_temperature("Yesterday's", &yesterday_temps);
        }
    }

    Ok(())
}

fn extract_temperatures(
    weather_data: &weather::WeatherData,
    now: NaiveDate,
    yesterday: NaiveDate,
) -> (Vec<f64>, Vec<f64>) {
    let mut today_temps = Vec::new();
    let mut yesterday_temps = Vec::new();

    for (i, time_str) in weather_data.hourly.time.iter().enumerate() {
        if let Ok(date) = NaiveDate::parse_from_str(&time_str[0..10], "%Y-%m-%d") {
            if date == now {
                today_temps.push(weather_data.hourly.temperature_2m[i]);
            } else if date == yesterday {
                yesterday_temps.push(weather_data.hourly.temperature_2m[i]);
            }
        } else {
            eprintln!(
                "Warning: Could not parse date for average calculation from timestamp: {}",
                time_str
            );
        }
    }

    (today_temps, yesterday_temps)
}

fn print_hourly_weather(weather_data: &weather::WeatherData, now: NaiveDate, yesterday: NaiveDate) {
    for (i, (time_str, temp)) in weather_data
        .hourly
        .time
        .iter()
        .zip(weather_data.hourly.temperature_2m.iter())
        .enumerate()
    {
        if i % 6 != 0 {
            continue;
        }

        if let Ok(naive_datetime) =
            chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M")
        {
            match Local.from_local_datetime(&naive_datetime) {
                chrono::LocalResult::Single(local_datetime) => {
                    let date = local_datetime.date_naive();
                    if date == now || date == yesterday {
                        let display_time = local_datetime.format("%Y-%m-%d %H:%M");
                        println!("{}: {:.1}°C", display_time, temp);
                    }
                }
                chrono::LocalResult::Ambiguous(_, _) => {
                    let date = naive_datetime.date();
                    if date == now || date == yesterday {
                        println!("{}: {:.1}°C (Ambiguous Local Time)", time_str, temp);
                    }
                }
                chrono::LocalResult::None => {
                    let date = naive_datetime.date();
                    if date == now || date == yesterday {
                        println!("{}: {:.1}°C (Invalid Local Time)", time_str, temp);
                    }
                }
            }
        } else if let Ok(datetime) =
            chrono::DateTime::parse_from_rfc3339(&(time_str.replace("Z", "+00:00")))
        {
            let date = datetime.date_naive();
            if date == now || date == yesterday {
                let display_time = datetime.with_timezone(&Local).format("%Y-%m-%d %H:%M");
                println!("{}: {:.1}°C", display_time, temp);
            }
        } else if let Ok(date) = NaiveDate::parse_from_str(&time_str[0..10], "%Y-%m-%d") {
            if date == now || date == yesterday {
                eprintln!(
                    "Warning: Could not parse timestamp for hourly display: {}",
                    time_str
                );
                println!("{}: {:.1}°C", time_str, temp);
            }
        }
    }
}

fn print_average_temperature(label: &str, temperatures: &[f64]) {
    match calculate_average(temperatures) {
        Some(avg) => println!("{} average temperature: {:.2}°C", label, avg),
        None => println!(
            "Could not calculate {} average temperature (no data).",
            label
        ),
    }
}
