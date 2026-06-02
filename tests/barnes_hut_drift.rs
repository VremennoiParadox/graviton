//! Compare short-term energy drift: direct vs Barnes–Hut on Earth–Moon.

use graviton::physics::diagnostics::compute;
use graviton::physics::integrator::{Integrator, Rk4Integrator};
use graviton::physics::system::SystemState;
use graviton::scenario::load;

#[test]
fn barnes_hut_energy_drift_close_to_direct_on_earth_moon() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios/earth-moon.toml");
    let mut direct_loaded = load(&path).expect("load");
    let mut bh_loaded = load(&path).expect("load");

    direct_loaded.system.settings.use_barnes_hut = false;
    bh_loaded.system.settings.use_barnes_hut = true;
    bh_loaded.system.settings.barnes_hut_theta = 0.5;

    let steps = 200u64;
    let direct_drift = run_drift(&mut direct_loaded.system, steps);
    let bh_drift = run_drift(&mut bh_loaded.system, steps);

    let diff = (direct_drift - bh_drift).abs();
    assert!(
        diff < 1e-4,
        "energy drift fraction should agree: direct={direct_drift:.6e} bh={bh_drift:.6e} diff={diff:.6e}"
    );
}

fn run_drift(system: &mut SystemState, steps: u64) -> f64 {
    let integrator = Rk4Integrator;
    let initial = compute(system);
    for _ in 0..steps {
        integrator.step(system).expect("step");
    }
    let final_diag = compute(system);
    final_diag.energy_drift_fraction(initial.total_energy_j).abs()
}
