use clap::Parser;
use std::io;
use std::path::{Path, PathBuf};
mod envelope;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};

/// Simple fetcher with output directory
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Directory to write output files
    #[arg(long, value_name = "DIR", default_value = ".")]
    out_dir: PathBuf,
}

fn go_fetch(path: &Path, use_bearer: bool, url: &str) -> io::Result<()> {
    let client = Client::new();

    let mut req = client.get(url);

    if use_bearer {
        let token = "0766b30cd4f2dd614f2e39817a179173d6ab5f74b80533631b4ae37a512802d9";
        let value = HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        req = req.header(AUTHORIZATION, value);
    }

    let resp = req.send();

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

    println!("Wrote to {:?}", path);

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Weather
    go_fetch(
        &args.out_dir.join("weather.json"),
        false,
        "https://api.open-meteo.com/v1/forecast?latitude=40.5508556&longitude=-80.0655996&daily=weather_code,temperature_2m_max,temperature_2m_min,apparent_temperature_max,apparent_temperature_min,precipitation_probability_max&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,weather_code&current=is_day,temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m&timezone=America%2FNew_York&wind_speed_unit=mph&temperature_unit=fahrenheit&precipitation_unit=inch",
    )?;

    // Cleaning
    go_fetch(
        &args.out_dir.join("cleaning.json"),
        true,
        "https://api.zq4.org/cleaning",
    )?;

    // Names
    go_fetch(
        &args.out_dir.join("names.json"),
        true,
        "https://api.zq4.org/people",
    )?;

    // Balances
    go_fetch(
        &args.out_dir.join("balances.json"),
        true,
        "https://api.zq4.org/allowance/balances",
    )?;

    // Upcoming payouts
    go_fetch(
        &args.out_dir.join("upcoming_payouts.json"),
        true,
        "https://api.zq4.org/allowance/upcoming_payouts",
    )?;

    go_fetch(
        &args.out_dir.join("dates.json"),
        true,
        "https://api.zq4.org/dates",
    )?;


    Ok(())
}
