//! Gravitational field magnitude for visualization (softened Newtonian).

use glam::DVec3;

use crate::physics::body::Body;
use crate::physics::constants::G;

/// Softened Newtonian acceleration at `point_m` from all bodies at `positions`.
#[must_use]
pub fn gravitational_acceleration(
    point_m: DVec3,
    bodies: &[Body],
    positions: &[DVec3],
    softening_m: f64,
) -> DVec3 {
    let eps_sq = softening_m * softening_m;
    let mut acc = DVec3::ZERO;

    for (body, pos) in bodies.iter().zip(positions) {
        let r = *pos - point_m;
        let dist_sq = r.length_squared() + eps_sq;
        let inv_dist3 = 1.0 / (dist_sq * dist_sq.sqrt());
        acc += G * body.mass_kg * r * inv_dist3;
    }

    acc
}

/// |g(point)| in m/s² — used for heatmap intensity.
#[must_use]
pub fn gravitational_field_magnitude(
    point_m: DVec3,
    bodies: &[Body],
    positions: &[DVec3],
    softening_m: f64,
) -> f64 {
    gravitational_acceleration(point_m, bodies, positions, softening_m).length()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::{Body, BodyClass};
    use crate::physics::constants::{G, M_EARTH};

    fn test_body(mass: f64, pos: DVec3) -> Body {
        Body {
            id: "m".into(),
            name: "m".into(),
            mass_kg: mass,
            radius_m: 1.0,
            position_m: pos,
            velocity_mps: DVec3::ZERO,
            class: BodyClass::Planet,
            color_rgb: None,
        }
    }

    #[test]
    fn field_magnitude_matches_point_mass_acceleration() {
        let m = M_EARTH;
        let r = 1.0e7;
        let softening = 0.0;
        let bodies = vec![test_body(m, DVec3::ZERO)];
        let positions = vec![DVec3::ZERO];
        let point = DVec3::new(r, 0.0, 0.0);
        let g = gravitational_field_magnitude(point, &bodies, &positions, softening);
        let expected = G * m / (r * r);
        assert!((g - expected).abs() / expected < 1e-10);
    }

    #[test]
    fn field_increases_near_mass() {
        let bodies = vec![test_body(1.0e24, DVec3::ZERO)];
        let positions = vec![DVec3::ZERO];
        let softening = 1.0;
        let far = gravitational_field_magnitude(
            DVec3::new(1.0e10, 0.0, 0.0),
            &bodies,
            &positions,
            softening,
        );
        let near = gravitational_field_magnitude(
            DVec3::new(1.0e8, 0.0, 0.0),
            &bodies,
            &positions,
            softening,
        );
        assert!(near > far);
    }
}
