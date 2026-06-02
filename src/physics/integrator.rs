//! Integrator trait for advancing [`SystemState`].

use crate::error::Result;
use crate::physics::system::SystemState;

/// Numerical integrator for N-body dynamics.
pub trait Integrator {
    fn step(&self, system: &mut SystemState) -> Result<()>;
}

/// Fixed-step RK4 integrator.
pub struct Rk4Integrator;

impl Integrator for Rk4Integrator {
    fn step(&self, system: &mut SystemState) -> Result<()> {
        crate::physics::rk4::step(system)?;
        system.time_s += system.settings.dt_s;
        Ok(())
    }
}
