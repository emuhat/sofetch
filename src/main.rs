use clap::Parser;
use std::io;
use std::path::{Path, PathBuf};
mod envelope;

/// Simple fetcher with output directory
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Directory to write output files
    #[arg(long, value_name = "DIR", default_value = ".")]
    out_dir: PathBuf,
}

fn go_fetch(path: &Path, url: &str) -> io::Result<()> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send();

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                let json: serde_json::Value = r
                    .json()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                envelope::update_success(path, json, None)?;
            } else {
                envelope::update_failure(path, format!("HTTP error: {}", r.status()))?;
            }
        }
        Err(e) => {
            envelope::update_failure(path, format!("Request failed: {}", e))?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Construct the full path in the out_dir
    let weather_path = args.out_dir.join("weather.json");

    go_fetch(
        &weather_path,
        "https://api.open-meteo.com/v1/forecast?latitude=40.5508556&longitude=-80.0655996&daily=weather_code,temperature_2m_max,temperature_2m_min,apparent_temperature_max,apparent_temperature_min,precipitation_probability_max&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,weather_code&current=is_day,temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m&timezone=America%2FNew_York&wind_speed_unit=mph&temperature_unit=fahrenheit&precipitation_unit=inch",
    )?;

    println!("Weather data written to {:?}", weather_path);
    Ok(())
}
