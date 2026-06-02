//! Simulation state and physics settings.

use crate::physics::body::Body;

/// Newtonian N-body system state (SI units).
#[derive(Debug, Clone)]
pub struct SystemState {
    pub bodies: Vec<Body>,
    pub time_s: f64,
    pub settings: PhysicsSettings,
    pub scenario_name: String,
}

/// Integrator and force-model parameters.
#[derive(Debug, Clone)]
pub struct PhysicsSettings {
    pub dt_s: f64,
    pub softening_m: f64,
    pub use_barnes_hut: bool,
    pub barnes_hut_theta: f64,
}

impl SystemState {
    #[must_use]
    pub fn new(bodies: Vec<Body>, settings: PhysicsSettings, scenario_name: String) -> Self {
        Self {
            bodies,
            time_s: 0.0,
            settings,
            scenario_name,
        }
    }

    pub fn is_finite(&self) -> bool {
        self.bodies.iter().all(|b| {
            b.position_m.is_finite() && b.velocity_mps.is_finite() && b.mass_kg.is_finite()
        })
    }
}
