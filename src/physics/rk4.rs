//! Fourth-order Runge-Kutta integrator for Newtonian N-body systems.

use glam::DVec3;

use crate::error::{PhysicsError, Result};
use crate::physics::gravity::accelerations;
use crate::physics::system::SystemState;

/// Per-body derivative: dr/dt = v, dv/dt = a.
#[derive(Debug, Clone, Copy)]
struct Derivative {
    dr: DVec3,
    dv: DVec3,
}

/// Advance the system by one fixed time step using RK4.
pub fn step(system: &mut SystemState) -> Result<()> {
    let dt = system.settings.dt_s;
    let softening = system.settings.softening_m;
    let n = system.bodies.len();

    let positions: Vec<DVec3> = system.bodies.iter().map(|b| b.position_m).collect();
    let velocities: Vec<DVec3> = system.bodies.iter().map(|b| b.velocity_mps).collect();
    let masses: Vec<f64> = system.bodies.iter().map(|b| b.mass_kg).collect();

    let k1 = evaluate(&system.bodies, &positions, &velocities, softening);
    let (pos2, vel2) = advance_state(&positions, &velocities, &k1, dt * 0.5);
    let k2 = evaluate(&system.bodies, &pos2, &vel2, softening);

    let (pos3, vel3) = advance_state(&positions, &velocities, &k2, dt * 0.5);
    let k3 = evaluate(&system.bodies, &pos3, &vel3, softening);

    let (pos4, vel4) = advance_state(&positions, &velocities, &k3, dt);
    let k4 = evaluate(&system.bodies, &pos4, &vel4, softening);

    for i in 0..n {
        let dr = (k1[i].dr + 2.0 * k2[i].dr + 2.0 * k3[i].dr + k4[i].dr) * (dt / 6.0);
        let dv = (k1[i].dv + 2.0 * k2[i].dv + 2.0 * k3[i].dv + k4[i].dv) * (dt / 6.0);
        system.bodies[i].position_m = positions[i] + dr;
        system.bodies[i].velocity_mps = velocities[i] + dv;
        let _ = masses[i];
    }

    if !system.is_finite() {
        return Err(PhysicsError::NonFiniteState.into());
    }

    Ok(())
}

fn evaluate(
    bodies: &[crate::physics::body::Body],
    positions: &[DVec3],
    velocities: &[DVec3],
    softening_m: f64,
) -> Vec<Derivative> {
    let acc = accelerations(bodies, positions, softening_m);
    velocities
        .iter()
        .zip(acc)
        .map(|(&v, a)| Derivative { dr: v, dv: a })
        .collect()
}

fn advance_state(
    positions: &[DVec3],
    velocities: &[DVec3],
    k: &[Derivative],
    dt: f64,
) -> (Vec<DVec3>, Vec<DVec3>) {
    let pos = positions
        .iter()
        .zip(k)
        .map(|(r, ki)| *r + ki.dr * dt)
        .collect();
    let vel = velocities
        .iter()
        .zip(k)
        .map(|(v, ki)| *v + ki.dv * dt)
        .collect();
    (pos, vel)
}
