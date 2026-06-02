//! Parse HORIZONS vector tables from the JSON `result` text field.

use crate::error::{HorizonsError, Result};
use crate::horizons::client::HorizonsResponse;
use crate::physics::constants::DAY;
use crate::physics::units::meters_to_au;

const SOE: &str = "$$SOE";
const EOE: &str = "$$EOE";

/// Detected position/velocity units in a HORIZONS vector block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorUnits {
    AuAndAuPerDay,
    KmAndKmPerSecond,
}

/// Parsed state vector in astronomical units for scenario export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParsedVector {
    pub position_au: [f64; 3],
    pub velocity_au_per_day: [f64; 3],
}

/// Parse the first data row between `$$SOE` and `$$EOE`.
pub fn parse_first_vector(response: &HorizonsResponse) -> Result<ParsedVector> {
    ensure_vector_table(&response.result)?;
    let units = detect_units(&response.result)?;
    let table = extract_table(&response.result)?;
    let row = table
        .first()
        .ok_or(HorizonsError::MissingVectorTable)?;
    parse_row(row, units)
}

fn ensure_vector_table(result: &str) -> Result<()> {
    if result.contains(SOE) && result.contains(EOE) {
        return Ok(());
    }
    for line in result.lines().rev() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Err(HorizonsError::Http(format!(
                "HORIZONS response has no vector table: {trimmed}"
            ))
            .into());
        }
    }
    Err(HorizonsError::MissingVectorTable.into())
}

/// Detect units from the HORIZONS text header (do not assume silently).
pub fn detect_units(result: &str) -> Result<VectorUnits> {
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("output units") {
            let upper = trimmed.to_ascii_uppercase();
            if upper.contains("KM-S") || upper.contains("KM/S") {
                return Ok(VectorUnits::KmAndKmPerSecond);
            }
            if upper.contains("AU") && upper.contains("D") {
                return Ok(VectorUnits::AuAndAuPerDay);
            }
            if upper.contains("AU") {
                return Ok(VectorUnits::AuAndAuPerDay);
            }
            return Err(HorizonsError::UnknownUnits.into());
        }
    }

    Err(HorizonsError::UnknownUnits.into())
}

fn extract_table(result: &str) -> Result<Vec<Vec<String>>> {
    let start = result
        .find(SOE)
        .ok_or(HorizonsError::MissingVectorTable)?
        + SOE.len();
    let end = result
        .find(EOE)
        .ok_or(HorizonsError::MissingVectorTable)?;
    if end <= start {
        return Err(HorizonsError::MissingVectorTable.into());
    }

    let mut rows = Vec::new();
    for line in result[start..end].lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        if fields.len() >= 8 && fields[0].parse::<f64>().is_ok() {
            rows.push(fields);
        }
    }

    if rows.is_empty() {
        return Err(HorizonsError::MissingVectorTable.into());
    }
    Ok(rows)
}

fn parse_row(fields: &[String], units: VectorUnits) -> Result<ParsedVector> {
    // CSV: JD, calendar, X, Y, Z, VX, VY, VZ, ...
    let x: f64 = fields[2].parse().map_err(|_| HorizonsError::MissingVectorTable)?;
    let y: f64 = fields[3].parse().map_err(|_| HorizonsError::MissingVectorTable)?;
    let z: f64 = fields[4].parse().map_err(|_| HorizonsError::MissingVectorTable)?;
    let vx: f64 = fields[5].parse().map_err(|_| HorizonsError::MissingVectorTable)?;
    let vy: f64 = fields[6].parse().map_err(|_| HorizonsError::MissingVectorTable)?;
    let vz: f64 = fields[7].parse().map_err(|_| HorizonsError::MissingVectorTable)?;

    let (position_au, velocity_au_per_day) = match units {
        VectorUnits::AuAndAuPerDay => ([x, y, z], [vx, vy, vz]),
        VectorUnits::KmAndKmPerSecond => {
            const KM_TO_M: f64 = 1000.0;
            let pos_au = [
                meters_to_au(x * KM_TO_M),
                meters_to_au(y * KM_TO_M),
                meters_to_au(z * KM_TO_M),
            ];
            let velocity_au_per_day = [
                meters_to_au(vx * KM_TO_M) * DAY,
                meters_to_au(vy * KM_TO_M) * DAY,
                meters_to_au(vz * KM_TO_M) * DAY,
            ];
            (pos_au, velocity_au_per_day)
        }
    };

    if !position_au.iter().all(|v| v.is_finite()) || !velocity_au_per_day.iter().all(|v| v.is_finite()) {
        return Err(HorizonsError::MissingVectorTable.into());
    }

    Ok(ParsedVector {
        position_au,
        velocity_au_per_day,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_earth_fixture_km_units() {
        let text = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/horizons_earth.json"
        ))
        .expect("fixture");
        let response: HorizonsResponse = serde_json::from_str(&text).expect("json");
        assert_eq!(detect_units(&response.result).unwrap(), VectorUnits::KmAndKmPerSecond);
        let vector = parse_first_vector(&response).expect("parse");
        let r_au = (vector.position_au[0].powi(2)
            + vector.position_au[1].powi(2)
            + vector.position_au[2].powi(2))
        .sqrt();
        assert!(
            r_au > 0.9 && r_au < 1.1,
            "Earth distance from Solar System barycenter should be ~1 AU, got {r_au}"
        );
        assert!(vector.velocity_au_per_day.iter().all(|v| v.is_finite()));
    }
}
