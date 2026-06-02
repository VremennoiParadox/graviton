//! Physical and astronomical constants (SI internal storage).

/// Gravitational constant in m³ kg⁻¹ s⁻².
pub const G: f64 = 6.674_30e-11;

/// Astronomical unit in meters.
pub const AU: f64 = 149_597_870_700.0;

/// Seconds per day.
pub const DAY: f64 = 86_400.0;

/// Seconds per Julian year (365.25 days).
#[allow(dead_code)]
pub const YEAR: f64 = 31_557_600.0;

/// Solar mass in kilograms.
pub const M_SUN: f64 = 1.988_92e30;

/// Earth mass in kilograms.
pub const M_EARTH: f64 = 5.972_19e24;

/// Moon mass in kilograms.
#[allow(dead_code)]
pub const M_MOON: f64 = 7.342e22;

/// Mean Earth–Moon distance in meters.
#[allow(dead_code)]
pub const EARTH_MOON_DISTANCE: f64 = 384_400_000.0;
