//! Energy, momentum, and center-of-mass diagnostics.

use glam::DVec3;

use crate::physics::constants::G;
use crate::physics::system::SystemState;

/// Snapshot of conserved quantities and drift metrics.
#[derive(Debug, Clone, Copy)]
pub struct Diagnostics {
    pub kinetic_j: f64,
    pub potential_j: f64,
    pub total_energy_j: f64,
    pub linear_momentum_kg_mps: DVec3,
    pub center_of_mass_m: DVec3,
    pub center_of_mass_velocity_mps: DVec3,
}

impl Diagnostics {
    /// Fractional energy drift relative to an initial total energy.
    #[must_use]
    pub fn energy_drift_fraction(&self, initial_total_energy_j: f64) -> f64 {
        if initial_total_energy_j.abs() < f64::EPSILON {
            return self.total_energy_j - initial_total_energy_j;
        }
        (self.total_energy_j - initial_total_energy_j) / initial_total_energy_j.abs()
    }
}

/// Compute diagnostics for the current system state.
#[must_use]
pub fn compute(system: &SystemState) -> Diagnostics {
    let softening = system.settings.softening_m;
    let kinetic_j: f64 = system.bodies.iter().map(|b| b.kinetic_energy_j()).sum();
    let potential_j = potential_energy(&system.bodies, softening);
    let total_mass: f64 = system.bodies.iter().map(|b| b.mass_kg).sum();

    let mut momentum = DVec3::ZERO;
    let mut com = DVec3::ZERO;
    let mut com_vel = DVec3::ZERO;

    for body in &system.bodies {
        momentum += body.velocity_mps * body.mass_kg;
        com += body.position_m * body.mass_kg;
        com_vel += body.velocity_mps * body.mass_kg;
    }

    if total_mass > 0.0 {
        com /= total_mass;
        com_vel /= total_mass;
    }

    Diagnostics {
        kinetic_j,
        potential_j,
        total_energy_j: kinetic_j + potential_j,
        linear_momentum_kg_mps: momentum,
        center_of_mass_m: com,
        center_of_mass_velocity_mps: com_vel,
    }
}

fn potential_energy(bodies: &[crate::physics::body::Body], softening_m: f64) -> f64 {
    let eps_sq = softening_m * softening_m;
    let mut u = 0.0;
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let r = bodies[j].position_m - bodies[i].position_m;
            let dist = (r.length_squared() + eps_sq).sqrt();
            u -= G * bodies[i].mass_kg * bodies[j].mass_kg / dist;
        }
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::{Body, BodyClass};
    use crate::physics::constants::{G, M_EARTH};
    use crate::physics::integrator::{Integrator, Rk4Integrator};
    use crate::physics::system::{PhysicsSettings, SystemState};
    use glam::DVec3;

    fn circular_two_body_system() -> SystemState {
        let r = 4.0e8;
        let m_central = M_EARTH;
        let m_orbit = 1.0e22;
        let mu = G * (m_central + m_orbit);
        let v_orbit = (mu / r).sqrt() * m_central / (m_central + m_orbit);
        let v_central = -v_orbit * m_orbit / m_central;

        let bodies = vec![
            Body {
                id: "central".into(),
                name: "central".into(),
                mass_kg: m_central,
                radius_m: 1.0,
                position_m: DVec3::ZERO,
                velocity_mps: DVec3::new(0.0, v_central, 0.0),
                class: BodyClass::Planet,
                color_rgb: None,
            },
            Body {
                id: "orbit".into(),
                name: "orbit".into(),
                mass_kg: m_orbit,
                radius_m: 1.0,
                position_m: DVec3::new(r, 0.0, 0.0),
                velocity_mps: DVec3::new(0.0, v_orbit, 0.0),
                class: BodyClass::Moon,
                color_rgb: None,
            },
        ];

        SystemState::new(
            bodies,
            PhysicsSettings {
                dt_s: 60.0,
                softening_m: 1.0,
                use_barnes_hut: false,
                barnes_hut_theta: 0.7,
            },
            "test".into(),
        )
    }

    #[test]
    fn center_of_mass_velocity_stays_near_zero() {
        let mut system = circular_two_body_system();
        let initial = compute(&system);
        assert!(initial.center_of_mass_velocity_mps.length() < 1.0);

        let integrator = Rk4Integrator;
        for _ in 0..500 {
            integrator.step(&mut system).unwrap();
        }

        let final_diag = compute(&system);
        assert!(
            final_diag.center_of_mass_velocity_mps.length() < 50.0,
            "COM velocity drifted to {}",
            final_diag.center_of_mass_velocity_mps.length()
        );
    }

    #[test]
    fn circular_orbit_radius_stays_approximately_constant() {
        let mut system = circular_two_body_system();
        let r0 = (system.bodies[1].position_m - system.bodies[0].position_m).length();

        let integrator = Rk4Integrator;
        for _ in 0..200 {
            integrator.step(&mut system).unwrap();
        }

        let r1 = (system.bodies[1].position_m - system.bodies[0].position_m).length();
        let drift = (r1 - r0).abs() / r0;
        assert!(drift < 0.01, "relative separation drift {drift} exceeds 1%");
    }

    #[test]
    fn energy_drift_is_measurable_over_many_steps() {
        let mut system = circular_two_body_system();
        let e0 = compute(&system).total_energy_j;

        let integrator = Rk4Integrator;
        for _ in 0..2000 {
            integrator.step(&mut system).unwrap();
        }

        let e1 = compute(&system).total_energy_j;
        let drift = (e1 - e0).abs() / e0.abs();
        assert!(
            drift > 0.0,
            "energy should drift measurably for non-symplectic RK4"
        );
        assert!(drift < 0.1, "energy drift unexpectedly large: {drift}");
    }
}
