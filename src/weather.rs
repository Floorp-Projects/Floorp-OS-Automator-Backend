use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WeatherApiResponse {
    pub hourly: HourlyData,
}

#[derive(Deserialize, Debug)]
pub struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
}

pub type WeatherData = WeatherApiResponse;

pub async fn fetch_weather_data(client: &Client, lat: f64, lon: f64) -> Result<WeatherApiResponse> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&past_days=1",
        lat, lon
    );

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to Open-Meteo API")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        anyhow::bail!(
            "Open-Meteo API request failed with status: {}. Body: {}",
            status,
            text
        );
    }

    let weather_data = response
        .json::<WeatherApiResponse>()
        .await
        .context("Failed to parse JSON response from Open-Meteo API")?;

    Ok(weather_data)
}
