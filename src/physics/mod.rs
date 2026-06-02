//! Newtonian N-body physics: gravity, RK4 integration, diagnostics, Barnes-Hut.
//!
//! Internal units: meters, seconds, kilograms (SI). See [`units`] and [`constants`].

pub mod body;
pub mod constants;
pub mod diagnostics;
pub mod gravity;
pub mod integrator;
pub mod rk4;
pub mod system;
pub mod units;

// Phase 5
// pub mod barnes_hut;
