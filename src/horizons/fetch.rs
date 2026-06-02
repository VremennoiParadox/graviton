//! Fetch presets and write scenario TOML files.

use std::fs;
use std::path::PathBuf;

use reqwest::Client;

use crate::error::{GravitonError, HorizonsError, Result};
use crate::horizons::cache::{raw_cache_path, read_raw_cache, write_raw_cache};
use crate::horizons::client;
use crate::horizons::ids::{self, HorizonsBody};
use crate::horizons::parser::{self, ParsedVector};

/// Options for `graviton fetch`.
pub struct FetchOptions {
    pub preset: String,
    pub date: Option<String>,
    pub center: String,
    pub output: Option<PathBuf>,
    pub offline: bool,
    pub force: bool,
}

/// Run fetch for the requested preset.
pub fn run_fetch(options: FetchOptions) -> Result<PathBuf> {
    match options.preset.as_str() {
        "solar-system" => fetch_solar_system(options),
        other => Err(GravitonError::Other(format!(
            "unknown fetch preset `{other}` (only `solar-system` is supported)"
        ))),
    }
}

fn fetch_solar_system(options: FetchOptions) -> Result<PathBuf> {
    let date_iso = options.date.unwrap_or_else(default_date_iso);
    let horizons_start = iso_to_horizons_date(&date_iso)?;
    // HORIZONS requires STOP_TIME strictly after START_TIME (see PLANNING.md).
    let horizons_stop = iso_to_horizons_date(&add_one_day_iso(&date_iso)?)?;

    let output = options
        .output
        .unwrap_or_else(|| PathBuf::from("scenarios/solar-system.toml"));

    let rt = tokio::runtime::Runtime::new().map_err(|e| GravitonError::Other(e.to_string()))?;
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| HorizonsError::Http(e.to_string()))?;

    let mut body_vectors = Vec::new();

    for body in ids::solar_system_default() {
        let vector = rt.block_on(fetch_body_vector(
            &client,
            body,
            &horizons_start,
            &horizons_stop,
            &options.center,
            options.offline,
            options.force,
        ))?;
        body_vectors.push((body, vector));
    }

    let toml = build_solar_system_toml(&body_vectors, &date_iso, &options.center)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| GravitonError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    fs::write(&output, toml).map_err(|e| GravitonError::Io {
        path: output.clone(),
        source: e,
    })?;

    println!(
        "Wrote {} ({} bodies, HORIZONS date {}, center {})",
        output.display(),
        body_vectors.len(),
        horizons_start,
        options.center
    );

    Ok(output)
}

async fn fetch_body_vector(
    client: &Client,
    body: &HorizonsBody,
    start: &str,
    stop: &str,
    center: &str,
    offline: bool,
    force: bool,
) -> Result<ParsedVector> {
    let cache_path = raw_cache_path(body.horizons_id, start, center);

    let response = if cache_path.exists() && (offline || !force) {
        if offline {
            eprintln!(
                "note: using cached HORIZONS data for {} from {}",
                body.name,
                cache_path.display()
            );
        }
        read_raw_cache(&cache_path)?
    } else if offline {
        return Err(GravitonError::Other(format!(
            "offline mode: no cache for {} ({}) at {}",
            body.name,
            body.horizons_id,
            cache_path.display()
        )));
    } else {
        let fetched = client::fetch_vectors(client, body.horizons_id, center, start, stop).await?;
        write_raw_cache(&cache_path, &fetched)?;
        fetched
    };

    parser::parse_first_vector(&response).map_err(|e| {
        GravitonError::Other(format!(
            "failed to parse HORIZONS vectors for {} ({}): {e}",
            body.name, body.horizons_id
        ))
    })
}

fn build_solar_system_toml(
    bodies: &[(&HorizonsBody, ParsedVector)],
    date_iso: &str,
    center: &str,
) -> Result<String> {
    let mut out = String::new();
    out.push_str("schema_version = 1\n");
    out.push_str("name = \"Solar System (HORIZONS)\"\n");
    out.push_str(&format!(
        "description = \"Barycentric state vectors from NASA JPL HORIZONS ({date_iso}, center {center}).\"\n"
    ));
    out.push_str("author = \"graviton (graviton fetch)\"\n\n");
    out.push_str("[units]\n");
    out.push_str("distance = \"au\"\n");
    out.push_str("mass = \"kg\"\n");
    out.push_str("time = \"day\"\n\n");
    out.push_str("[physics]\n");
    out.push_str("integrator = \"rk4\"\n");
    out.push_str("dt = 3600.0\n");
    out.push_str("dt_unit = \"s\"\n");
    out.push_str("softening_m = 1000000.0\n\n");
    out.push_str("[render]\n");
    out.push_str("meters_per_cell = 5000000000.0\n");
    out.push_str("trail_points = 2048\n");
    out.push_str("follow_center_of_mass = true\n\n");

    for (body, vector) in bodies {
        out.push_str("[[bodies]]\n");
        out.push_str(&format!("id = \"{}\"\n", body.id));
        out.push_str(&format!("name = \"{}\"\n", body.name));
        out.push_str(&format!("class = \"{}\"\n", body.class));
        out.push_str(&format!("mass = {:.12e}\n", body.mass_kg));
        out.push_str(&format!("radius = {:.6e}\n", body.radius_m));
        out.push_str(&format!(
            "position = [{}, {}, {}]\n",
            vector.position_au[0], vector.position_au[1], vector.position_au[2]
        ));
        out.push_str(&format!(
            "velocity = [{}, {}, {}]\n",
            vector.velocity_au_per_day[0],
            vector.velocity_au_per_day[1],
            vector.velocity_au_per_day[2]
        ));
        out.push_str(&format!("color = \"{}\"\n", body.color));
        out.push_str(&format!("horizons_id = \"{}\"\n\n", body.horizons_id));
    }

    Ok(out)
}

fn default_date_iso() -> String {
    "2026-06-01".to_string()
}

fn add_one_day_iso(iso: &str) -> Result<String> {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 {
        return Err(GravitonError::Other(format!(
            "invalid date `{iso}` (expected YYYY-MM-DD)"
        )));
    }
    let mut year: i32 = parts[0]
        .parse()
        .map_err(|_| GravitonError::Other(format!("invalid year in date `{iso}`")))?;
    let mut month: u32 = parts[1]
        .parse()
        .map_err(|_| GravitonError::Other(format!("invalid month in date `{iso}`")))?;
    let mut day: u32 = parts[2]
        .parse()
        .map_err(|_| GravitonError::Other(format!("invalid day in date `{iso}`")))?;

    day += 1;
    let days_in_month = days_in_month(year, month);
    if day > days_in_month {
        day = 1;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }

    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn iso_to_horizons_date(iso: &str) -> Result<String> {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 {
        return Err(GravitonError::Other(format!(
            "invalid date `{iso}` (expected YYYY-MM-DD)"
        )));
    }
    let month = match parts[1] {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        other => {
            return Err(GravitonError::Other(format!(
                "invalid month `{other}` in date `{iso}`"
            )));
        }
    };
    let day: u8 = parts[2]
        .parse()
        .map_err(|_| GravitonError::Other(format!("invalid day in date `{iso}`")))?;
    Ok(format!("{}-{}-{:02}", parts[0], month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_one_day_iso_handles_month_boundary() {
        assert_eq!(add_one_day_iso("2026-06-01").unwrap(), "2026-06-02");
        assert_eq!(add_one_day_iso("2026-01-31").unwrap(), "2026-02-01");
    }
}
