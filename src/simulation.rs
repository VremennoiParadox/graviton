//! Headless simulation driver and diagnostic reporting.

use crate::error::{OrreryTuiError, Result};
use crate::physics::constants::DAY;
use crate::physics::diagnostics::{compute, Diagnostics};
use crate::physics::integrator::{Integrator, Rk4Integrator};
use crate::physics::system::SystemState;
use crate::scenario::LoadedScenario;

/// Run a fixed number of physics steps and return initial/final diagnostics.
pub fn run_headless(
    loaded: &mut LoadedScenario,
    steps: u64,
    dt_override: Option<f64>,
) -> Result<(Diagnostics, Diagnostics)> {
    if let Some(dt) = dt_override {
        if dt <= 0.0 || !dt.is_finite() {
            return Err(OrreryTuiError::Scenario(
                crate::error::ScenarioError::InvalidTimeStep,
            ));
        }
        loaded.system.settings.dt_s = dt;
    }

    let initial = compute(&loaded.system);

    let integrator = Rk4Integrator;
    for _ in 0..steps {
        integrator.step(&mut loaded.system)?;
    }

    let final_diag = compute(&loaded.system);
    Ok((initial, final_diag))
}

/// Print human-readable diagnostics to stdout.
pub fn print_diagnostics(
    system: &SystemState,
    initial: &Diagnostics,
    final_diag: &Diagnostics,
    steps: u64,
) {
    println!("Scenario: {}", system.scenario_name);
    if system.settings.use_barnes_hut {
        println!(
            "Integrator: RK4 (Barnes–Hut, θ = {:.2})",
            system.settings.barnes_hut_theta
        );
    } else {
        println!("Integrator: RK4 (direct O(n²) gravity)");
    }
    println!("Time step: {:.6} s", system.settings.dt_s);
    println!("Softening ε: {:.3} m", system.settings.softening_m);
    println!(
        "Simulation time: {:.3} s ({:.5} days)",
        system.time_s,
        system.time_s / DAY
    );
    println!("Steps completed: {steps}");
    println!();
    println!("Bodies:");
    for body in &system.bodies {
        let speed = body.velocity_mps.length();
        println!(
            "  {} ({}) — mass {:.6e} kg, |r| {:.6e} m, speed {:.3} m/s",
            body.name,
            body.id,
            body.mass_kg,
            body.position_m.length(),
            speed
        );
    }
    println!();
    println!("Energy (J):");
    println!("  initial kinetic:     {:.6e}", initial.kinetic_j);
    println!("  initial potential:   {:.6e}", initial.potential_j);
    println!("  initial total:       {:.6e}", initial.total_energy_j);
    println!("  final kinetic:       {:.6e}", final_diag.kinetic_j);
    println!("  final potential:     {:.6e}", final_diag.potential_j);
    println!("  final total:         {:.6e}", final_diag.total_energy_j);
    println!(
        "  drift:         {:.6e} ({:.4e} relative)",
        final_diag.total_energy_j - initial.total_energy_j,
        final_diag.energy_drift_fraction(initial.total_energy_j)
    );
    println!();
    println!("Center of mass:");
    println!(
        "  position: ({:.6e}, {:.6e}, {:.6e}) m",
        final_diag.center_of_mass_m.x, final_diag.center_of_mass_m.y, final_diag.center_of_mass_m.z
    );
    println!(
        "  velocity: ({:.6e}, {:.6e}, {:.6e}) m/s",
        final_diag.center_of_mass_velocity_mps.x,
        final_diag.center_of_mass_velocity_mps.y,
        final_diag.center_of_mass_velocity_mps.z
    );
    println!(
        "  |linear momentum|: {:.6e} kg·m/s",
        final_diag.linear_momentum_kg_mps.length()
    );
}
