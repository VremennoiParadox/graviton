//! Serde types for scenario TOML files.

use serde::Deserialize;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct ScenarioFile {
    pub schema_version: u32,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub units: UnitsSection,
    pub physics: PhysicsSection,
    #[serde(default)]
    pub render: RenderSection,
    pub bodies: Vec<BodySpec>,
    #[serde(default)]
    pub asteroid_belt: Option<AsteroidBeltSection>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UnitsSection {
    #[serde(default = "default_distance_unit")]
    pub distance: String,
    #[serde(default)]
    pub distance_scale_m: Option<f64>,
    #[serde(default = "default_mass_unit")]
    pub mass: String,
    #[serde(default)]
    pub mass_scale_kg: Option<f64>,
    #[serde(default = "default_time_unit")]
    pub time: String,
    #[serde(default)]
    pub time_scale_s: Option<f64>,
}

fn default_distance_unit() -> String {
    "m".into()
}
fn default_mass_unit() -> String {
    "kg".into()
}
fn default_time_unit() -> String {
    "s".into()
}

#[derive(Debug, Deserialize)]
pub struct PhysicsSection {
    #[serde(default = "default_integrator")]
    pub integrator: String,
    pub dt: f64,
    #[serde(default)]
    pub dt_unit: Option<String>,
    #[serde(default = "default_softening")]
    pub softening_m: f64,
    #[serde(default)]
    pub barnes_hut: BarnesHutSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct BarnesHutSection {
    pub enabled: Option<bool>,
    pub theta: Option<f64>,
}

/// Procedural asteroid belt parameters (bodies generated at load time).
#[derive(Debug, Deserialize)]
pub struct AsteroidBeltSection {
    pub count: u32,
    pub inner_au: f64,
    pub outer_au: f64,
    #[serde(default = "default_belt_seed")]
    pub seed: u64,
}

fn default_belt_seed() -> u64 {
    42
}

fn default_integrator() -> String {
    "rk4".into()
}
fn default_softening() -> f64 {
    1000.0
}

#[derive(Debug, Default, Deserialize)]
pub struct RenderSection {
    pub meters_per_cell: Option<f64>,
    pub trail_points: Option<u32>,
    pub follow_center_of_mass: Option<bool>,
    pub heatmap_enabled: Option<bool>,
    pub heatmap_sample_divisor: Option<u32>,
    pub show_com_marker: Option<bool>,
    #[serde(default)]
    pub kitty: KittyRenderSection,
}

#[derive(Debug, Default, Deserialize)]
pub struct KittyRenderSection {
    pub enabled: Option<bool>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // optional metadata used by fetch/validate and future phases
pub struct BodySpec {
    pub id: String,
    pub name: String,
    pub class: String,
    pub mass: f64,
    pub radius: f64,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub color: Option<String>,
    pub horizons_id: Option<String>,
    pub primary: Option<String>,
    pub trail_enabled: Option<bool>,
    pub notes: Option<String>,
}
