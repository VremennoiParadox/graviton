//! Simulation bodies: mass, position, velocity in SI units.

use glam::DVec3;
use serde::{Deserialize, Serialize};

/// Stable identifier for a body within a scenario.
pub type BodyId = String;

/// Body classification for rendering and HUD (Phase 2+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyClass {
    Star,
    Planet,
    Moon,
    DwarfPlanet,
    Asteroid,
    Comet,
    Spacecraft,
    Custom,
}

/// A gravitating point mass in the simulation.
#[derive(Debug, Clone)]
pub struct Body {
    pub id: BodyId,
    pub name: String,
    pub mass_kg: f64,
    /// Display radius in meters (rendering in Phase 2+).
    #[allow(dead_code)]
    pub radius_m: f64,
    pub position_m: DVec3,
    pub velocity_mps: DVec3,
    /// Body class for colors and HUD (Phase 2+).
    #[allow(dead_code)]
    pub class: BodyClass,
}

impl Body {
    #[must_use]
    pub fn kinetic_energy_j(&self) -> f64 {
        0.5 * self.mass_kg * self.velocity_mps.length_squared()
    }
}
