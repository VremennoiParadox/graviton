//! Scenario validation before conversion to simulation state.

use std::collections::HashSet;

use crate::error::{Result, ScenarioError};
use crate::scenario::schema::{ScenarioFile, CURRENT_SCHEMA_VERSION};

/// Collect all validation issues; returns the first batch as error strings.
pub fn validate(raw: &ScenarioFile) -> Result<()> {
    let mut errors = Vec::new();

    if raw.schema_version != CURRENT_SCHEMA_VERSION {
        errors.push(
            ScenarioError::UnsupportedSchema {
                found: raw.schema_version,
                expected: CURRENT_SCHEMA_VERSION,
            }
            .to_string(),
        );
    }

    if raw.physics.dt <= 0.0 || !raw.physics.dt.is_finite() {
        errors.push(ScenarioError::InvalidTimeStep.to_string());
    }

    if raw.physics.softening_m < 0.0 || !raw.physics.softening_m.is_finite() {
        errors.push("physics.softening_m must be finite and non-negative".into());
    }

    if raw.physics.integrator != "rk4" {
        errors.push(format!(
            "unsupported integrator `{}` (only rk4 is supported)",
            raw.physics.integrator
        ));
    }

    validate_units(raw, &mut errors);

    if raw.bodies.is_empty() {
        errors.push("scenario must contain at least one body".into());
    }

    let mut ids = HashSet::new();
    for body in &raw.bodies {
        if !ids.insert(&body.id) {
            errors.push(
                ScenarioError::DuplicateId {
                    id: body.id.clone(),
                }
                .to_string(),
            );
        }
        if body.mass <= 0.0 || !body.mass.is_finite() {
            errors.push(
                ScenarioError::NonPositiveMass {
                    id: body.id.clone(),
                }
                .to_string(),
            );
        }
        if body.radius < 0.0 || !body.radius.is_finite() {
            errors.push(format!(
                "body `{}` radius must be finite and non-negative",
                body.id
            ));
        }
        if !is_finite_vec3(&body.position) {
            errors.push(format!("body `{}` position must be finite", body.id));
        }
        if !is_finite_vec3(&body.velocity) {
            errors.push(format!("body `{}` velocity must be finite", body.id));
        }
    }

    if !errors.is_empty() {
        return Err(ScenarioError::Validation(errors).into());
    }

    Ok(())
}

fn validate_units(raw: &ScenarioFile, errors: &mut Vec<String>) {
    if raw.units.distance == "custom" && raw.units.distance_scale_m.is_none() {
        errors.push("units.distance = \"custom\" requires units.distance_scale_m".into());
    }
    if raw.units.mass == "custom" && raw.units.mass_scale_kg.is_none() {
        errors.push("units.mass = \"custom\" requires units.mass_scale_kg".into());
    }
    if raw.units.time == "custom" && raw.units.time_scale_s.is_none() {
        errors.push("units.time = \"custom\" requires units.time_scale_s".into());
    }
}

fn is_finite_vec3(v: &[f64; 3]) -> bool {
    v.iter().all(|c| c.is_finite())
}
