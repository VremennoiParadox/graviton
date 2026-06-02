//! Direct O(n²) Newtonian gravity with Plummer softening.

use glam::DVec3;

use crate::physics::body::Body;
use crate::physics::constants::G;

/// Compute accelerations for all bodies at the given positions (meters, m/s²).
///
/// Uses softened distance: `|r|² + ε²` in the denominator.
pub fn accelerations(bodies: &[Body], positions: &[DVec3], softening_m: f64) -> Vec<DVec3> {
    let n = bodies.len();
    let eps_sq = softening_m * softening_m;
    let mut acc = vec![DVec3::ZERO; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let r = positions[j] - positions[i];
            let dist_sq = r.length_squared() + eps_sq;
            let inv_dist3 = 1.0 / (dist_sq * dist_sq.sqrt());
            acc[i] += G * bodies[j].mass_kg * r * inv_dist3;
        }
    }

    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::{Body, BodyClass};
    use crate::physics::constants::{G, M_EARTH};

    fn test_body(id: &str, mass: f64, pos: DVec3) -> Body {
        Body {
            id: id.into(),
            name: id.into(),
            mass_kg: mass,
            radius_m: 1.0,
            position_m: pos,
            velocity_mps: DVec3::ZERO,
            class: BodyClass::Planet,
            color_rgb: None,
        }
    }

    #[test]
    fn two_body_acceleration_symmetry() {
        let separation = 1.0e8;
        let softening = 1.0;
        let bodies = vec![
            test_body("a", M_EARTH, DVec3::ZERO),
            test_body("b", M_EARTH, DVec3::new(separation, 0.0, 0.0)),
        ];
        let positions: Vec<_> = bodies.iter().map(|b| b.position_m).collect();
        let acc = accelerations(&bodies, &positions, softening);

        // a on b from a should equal - (m_a/m_b) * a on a from b (equal masses -> opposite)
        let a_on_a = acc[0];
        let a_on_b = acc[1];
        assert!(a_on_a.length() > 0.0);
        assert!((a_on_a + a_on_b).length() < 1e-6 * a_on_a.length());
    }

    #[test]
    fn acceleration_scales_with_g_over_r_squared() {
        let m = 1.0e24;
        let r = 1.0e9;
        let softening = 0.0;
        let bodies = vec![
            test_body("central", m, DVec3::ZERO),
            test_body("probe", 1.0, DVec3::new(r, 0.0, 0.0)),
        ];
        let positions: Vec<_> = bodies.iter().map(|b| b.position_m).collect();
        let acc = accelerations(&bodies, &positions, softening);
        let expected = -G * m / (r * r);
        assert!((acc[1].x - expected).abs() / expected.abs() < 1e-10);
    }
}
