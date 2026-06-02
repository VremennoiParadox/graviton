//! NASA JPL HORIZONS HTTP client and URL construction.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{HorizonsError, Result};

pub const HORIZONS_API: &str = "https://ssd.jpl.nasa.gov/api/horizons.api";

/// JSON wrapper returned by `horizons.api` with `format=json`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HorizonsResponse {
    pub result: String,
}

/// Build a GET URL for barycentric state vectors at a single epoch.
#[must_use]
pub fn vector_query_url(
    command_id: &str,
    center: &str,
    start_time: &str,
    stop_time: &str,
) -> String {
    format!(
        "{HORIZONS_API}?format=json&EPHEM_TYPE=VECTORS&COMMAND='{command_id}'&CENTER='{center}'\
         &START_TIME='{start_time}'&STOP_TIME='{stop_time}'&STEP_SIZE='1%20d'&VEC_TABLE='2'\
         &CSV_FORMAT='YES'&OBJ_DATA='YES'"
    )
}

/// Fetch raw HORIZONS JSON for one body.
pub async fn fetch_vectors(
    client: &Client,
    command_id: &str,
    center: &str,
    start_time: &str,
    stop_time: &str,
) -> Result<HorizonsResponse> {
    let url = vector_query_url(command_id, center, start_time, stop_time);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| HorizonsError::Http(e.to_string()))?;

    if !response.status().is_success() {
        return Err(
            HorizonsError::Http(format!("status {} for {}", response.status(), url)).into(),
        );
    }

    let text = response
        .text()
        .await
        .map_err(|e| HorizonsError::Http(e.to_string()))?;

    if text.contains("No ephemeris") || (text.contains("ERROR") && !text.contains("\"result\"")) {
        return Err(HorizonsError::Http(format!(
            "HORIZONS returned an error for command {command_id}"
        ))
        .into());
    }

    serde_json::from_str(&text).map_err(|e| {
        HorizonsError::Http(format!("invalid JSON from HORIZONS for {command_id}: {e}")).into()
    })
}
