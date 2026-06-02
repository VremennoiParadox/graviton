//! Load scenario TOML files and build [`SystemState`] in SI units.

use std::fs;
use std::path::Path;

use glam::DVec3;

use crate::error::{Result, ScenarioError};
use crate::physics::body::{Body, BodyClass};
use crate::physics::constants::{AU, DAY, M_EARTH, M_SUN};
use crate::physics::system::{PhysicsSettings, SystemState};
use crate::physics::units::days_to_seconds;
use crate::scenario::schema::ScenarioFile;
use crate::scenario::validate::validate;

/// Loaded scenario with simulation state and metadata.
pub struct LoadedScenario {
    pub system: SystemState,
    #[allow(dead_code)]
    /// Scenario description from TOML (HUD/docs in later phases).
    pub description: Option<String>,
    #[allow(dead_code)]
    /// Scenario author from TOML.
    pub author: Option<String>,
}

/// Read, validate, and convert a scenario file to SI [`SystemState`].
pub fn load(path: &Path) -> Result<LoadedScenario> {
    let text = fs::read_to_string(path).map_err(ScenarioError::Read)?;
    let raw: ScenarioFile = toml::from_str(&text).map_err(ScenarioError::Parse)?;
    validate(&raw)?;

    let distance_scale = distance_scale_m(&raw)?;
    let mass_scale = mass_scale_kg(&raw)?;
    let time_scale = time_scale_s(&raw)?;
    let velocity_scale = distance_scale / time_scale;

    let bodies = raw
        .bodies
        .iter()
        .map(|spec| {
            Ok(Body {
                id: spec.id.clone(),
                name: spec.name.clone(),
                mass_kg: spec.mass * mass_scale,
                radius_m: spec.radius * distance_scale,
                position_m: DVec3::from_array(spec.position) * distance_scale,
                velocity_mps: DVec3::from_array(spec.velocity) * velocity_scale,
                class: parse_class(&spec.class)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let dt_s = dt_seconds(&raw, time_scale)?;
    let settings = PhysicsSettings {
        dt_s,
        softening_m: raw.physics.softening_m,
        use_barnes_hut: false,
        barnes_hut_theta: 0.7,
    };

    let system = SystemState::new(bodies, settings, raw.name.clone());

    Ok(LoadedScenario {
        system,
        description: raw.description,
        author: raw.author,
    })
}

fn distance_scale_m(raw: &ScenarioFile) -> Result<f64> {
    match raw.units.distance.as_str() {
        "m" => Ok(1.0),
        "au" => Ok(AU),
        "custom" => Ok(raw
            .units
            .distance_scale_m
            .ok_or(ScenarioError::MissingUnitScale {
                field: "distance_scale_m".into(),
            })?),
        other => Err(ScenarioError::UnknownUnit {
            kind: "distance".into(),
            value: other.into(),
        }
        .into()),
    }
}

fn mass_scale_kg(raw: &ScenarioFile) -> Result<f64> {
    match raw.units.mass.as_str() {
        "kg" => Ok(1.0),
        "solar_mass" => Ok(M_SUN),
        "earth_mass" => Ok(M_EARTH),
        "custom" => Ok(raw
            .units
            .mass_scale_kg
            .ok_or(ScenarioError::MissingUnitScale {
                field: "mass_scale_kg".into(),
            })?),
        other => Err(ScenarioError::UnknownUnit {
            kind: "mass".into(),
            value: other.into(),
        }
        .into()),
    }
}

fn time_scale_s(raw: &ScenarioFile) -> Result<f64> {
    match raw.units.time.as_str() {
        "s" => Ok(1.0),
        "day" => Ok(DAY),
        "custom" => Ok(raw
            .units
            .time_scale_s
            .ok_or(ScenarioError::MissingUnitScale {
                field: "time_scale_s".into(),
            })?),
        other => Err(ScenarioError::UnknownUnit {
            kind: "time".into(),
            value: other.into(),
        }
        .into()),
    }
}

fn dt_seconds(raw: &ScenarioFile, time_scale_s: f64) -> Result<f64> {
    let unit = raw.physics.dt_unit.as_deref().unwrap_or("s");
    let dt = match unit {
        "s" => raw.physics.dt,
        "day" => days_to_seconds(raw.physics.dt),
        "time" => raw.physics.dt * time_scale_s,
        other => {
            return Err(ScenarioError::UnknownUnit {
                kind: "dt_unit".into(),
                value: other.into(),
            }
            .into())
        }
    };
    if dt <= 0.0 || !dt.is_finite() {
        return Err(ScenarioError::InvalidTimeStep.into());
    }
    Ok(dt)
}

fn parse_class(s: &str) -> Result<BodyClass> {
    let class = match s {
        "star" => BodyClass::Star,
        "planet" => BodyClass::Planet,
        "moon" => BodyClass::Moon,
        "dwarf_planet" => BodyClass::DwarfPlanet,
        "asteroid" => BodyClass::Asteroid,
        "comet" => BodyClass::Comet,
        "spacecraft" => BodyClass::Spacecraft,
        "custom" => BodyClass::Custom,
        other => {
            return Err(ScenarioError::UnknownBodyClass {
                class: other.into(),
            }
            .into())
        }
    };
    Ok(class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scenario_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scenarios")
            .join(name)
    }

    #[test]
    fn loads_earth_moon_in_si() {
        use crate::physics::constants::{EARTH_MOON_DISTANCE, M_EARTH, M_MOON};

        let loaded = load(&scenario_path("earth-moon.toml")).expect("earth-moon loads");
        assert_eq!(loaded.system.bodies.len(), 2);
        assert!((loaded.system.settings.dt_s - 300.0).abs() < f64::EPSILON);
        let earth = loaded
            .system
            .bodies
            .iter()
            .find(|b| b.id == "earth")
            .expect("earth");
        let moon = loaded
            .system
            .bodies
            .iter()
            .find(|b| b.id == "moon")
            .expect("moon");
        assert!((earth.mass_kg - M_EARTH).abs() / M_EARTH < 1e-6);
        assert!((moon.mass_kg - M_MOON).abs() / M_MOON < 1e-4);
        let separation = (moon.position_m - earth.position_m).length();
        assert!((separation - EARTH_MOON_DISTANCE).abs() / EARTH_MOON_DISTANCE < 1e-4);
    }

    #[test]
    fn loads_figure_eight_with_custom_units() {
        let loaded = load(&scenario_path("figure-eight.toml")).expect("figure-eight loads");
        assert_eq!(loaded.system.bodies.len(), 3);
        assert!(loaded.system.settings.dt_s > 0.0);
        let a = &loaded.system.bodies[0];
        assert!((a.mass_kg - 1.0e24).abs() < 1.0e15);
    }
}
