mod envelope;

use envelope::*;
use reqwest::blocking::Client;
use std::io;
use serde_json::Value;

fn go_fetch(path: &str, url: &str) -> io::Result<()> {
    let client = Client::new();
    let resp = client.get(url).send();

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                // Parse JSON
                let json: Value = r.json().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                // Update envelope
                update_success(path, json, None)?;
            } else {
                update_failure(path, format!("HTTP error: {}", r.status()))?;
            }
        }
        Err(e) => {
            update_failure(path, format!("Request failed: {}", e))?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    go_fetch("weather.json", "https://api.open-meteo.com/v1/forecast?latitude=40.5&longitude=-79.9&daily=weather_code,temperature_2m_max,temperature_2m_min,apparent_temperature_max,apparent_temperature_min,precipitation_probability_max&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,weather_code&current=is_day,temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m&timezone=America%2FNew_York&wind_speed_unit=mph&temperature_unit=fahrenheit&precipitation_unit=inch")?;
    Ok(())
}
