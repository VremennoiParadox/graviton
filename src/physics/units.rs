//! Unit conversions between SI (internal) and display/scenario units.
#![allow(dead_code)] // full conversion API used as scenarios and HORIZONS grow

use super::constants::{AU, DAY};

/// Convert astronomical units to meters.
#[must_use]
pub fn au_to_meters(au: f64) -> f64 {
    au * AU
}

/// Convert meters to astronomical units.
#[must_use]
pub fn meters_to_au(m: f64) -> f64 {
    m / AU
}

/// Convert days to seconds.
#[must_use]
pub fn days_to_seconds(days: f64) -> f64 {
    days * DAY
}

/// Convert seconds to days.
#[must_use]
pub fn seconds_to_days(s: f64) -> f64 {
    s / DAY
}

/// Convert AU/day to m/s (common HORIZONS velocity units).
#[must_use]
pub fn au_per_day_to_m_per_s(v: f64) -> f64 {
    au_to_meters(v) / DAY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn au_round_trip() {
        let au = 1.0;
        let m = au_to_meters(au);
        assert!((meters_to_au(m) - au).abs() < 1e-9);
    }
}
