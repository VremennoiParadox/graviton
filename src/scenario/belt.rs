//! Procedural asteroid belt body generation for stress scenarios.

use crate::physics::constants::{AU, DAY, G, M_SUN};
use crate::scenario::schema::{AsteroidBeltSection, BodySpec};

/// Simple LCG for deterministic pseudo-random values in `[0, 1)`.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bits = (self.state >> 11) as u32;
        (f64::from(bits) / f64::from(u32::MAX)).clamp(0.0, 1.0 - f64::EPSILON)
    }
}

/// Generate `count` asteroid [`BodySpec`] entries in an annulus around the origin.
#[must_use]
pub fn generate_asteroids(belt: &AsteroidBeltSection) -> Vec<BodySpec> {
    let mut rng = Rng::new(belt.seed);
    let mut bodies = Vec::with_capacity(belt.count as usize);
    let mu = G * M_SUN;

    for i in 0..belt.count {
        let t = rng.next_f64();
        let r_au = belt.inner_au + (belt.outer_au - belt.inner_au) * t;
        let r_m = r_au * AU;
        let theta = rng.next_f64() * std::f64::consts::TAU;
        let z_au = (rng.next_f64() - 0.5) * 0.05;

        let v_circ = (mu / r_m).sqrt();
        let v_au_day = v_circ * DAY / AU;

        let mass = 1.0e15 + rng.next_f64() * 1.0e17;
        let id = format!("ast_{i:04}");

        bodies.push(BodySpec {
            id: id.clone(),
            name: format!("Asteroid {i}"),
            class: "asteroid".into(),
            mass,
            radius: 500.0 + rng.next_f64() * 2000.0,
            position: [r_au * theta.cos(), r_au * theta.sin(), z_au],
            velocity: [-v_au_day * theta.sin(), v_au_day * theta.cos(), 0.0],
            color: Some("#8c7f70".into()),
            horizons_id: None,
            primary: Some("sun".into()),
            trail_enabled: Some(false),
            notes: None,
        });
    }

    bodies
}
