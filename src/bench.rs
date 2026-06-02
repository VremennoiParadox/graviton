//! Built-in physics benchmarks (`graviton bench`).

use std::time::Instant;

use glam::DVec3;

use crate::error::Result;
use crate::physics::barnes_hut;
use crate::physics::body::{Body, BodyClass};
use crate::physics::gravity::{accelerations, direct_accelerations};
use crate::physics::system::PhysicsSettings;

/// Run acceleration benchmarks and print timings to stdout.
pub fn run(filter: Option<&str>) -> Result<()> {
    let sizes = [10usize, 100, 500, 1000];
    println!("graviton acceleration benchmarks (single-threaded, best of 3)\n");

    for &n in &sizes {
        if let Some(f) = filter {
            if f != "all" && !f.contains(&n.to_string()) && f != "direct" && f != "barnes_hut" {
                continue;
            }
        }

        let (bodies, positions) = random_system(n, 7);
        let settings_direct = PhysicsSettings {
            dt_s: 1.0,
            softening_m: 1000.0,
            use_barnes_hut: false,
            barnes_hut_theta: 0.7,
        };
        let settings_bh = PhysicsSettings {
            use_barnes_hut: true,
            ..settings_direct
        };

        if filter.is_none() || filter == Some("direct") || filter == Some("all") {
            let t = bench(|| {
                let _ = direct_accelerations(&bodies, &positions, settings_direct.softening_m);
            });
            println!("direct O(n²)  n={n:4}  {t:.3} ms");
        }

        if filter.is_none() || filter == Some("barnes_hut") || filter == Some("all") {
            let t = bench(|| {
                let _ = barnes_hut::accelerations(
                    &bodies,
                    &positions,
                    settings_bh.softening_m,
                    settings_bh.barnes_hut_theta,
                );
            });
            println!("Barnes–Hut    n={n:4}  {t:.3} ms");
        }

        if filter.is_none() || filter == Some("all") {
            let t_direct = bench(|| {
                let _ = accelerations(&bodies, &positions, &settings_direct);
            });
            let t_bh = bench(|| {
                let _ = accelerations(&bodies, &positions, &settings_bh);
            });
            let speedup = t_direct / t_bh.max(1e-9);
            println!("  → speedup BH vs direct: {speedup:.2}x\n");
        }
    }

    println!("For Criterion reports: cargo bench --bench direct_vs_barnes_hut");
    Ok(())
}

fn bench(mut f: impl FnMut()) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64() * 1000.0);
    }
    best
}

fn random_system(n: usize, seed: u64) -> (Vec<Body>, Vec<DVec3>) {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 11) as f64 / u32::MAX as f64
    };

    let mut bodies = Vec::with_capacity(n);
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let x = (next() - 0.5) * 1.0e10;
        let y = (next() - 0.5) * 1.0e10;
        let z = (next() - 0.5) * 1.0e9;
        let mass = 1.0e20 + next() * 1.0e22;
        let pos = DVec3::new(x, y, z);
        bodies.push(Body {
            id: format!("b{i}"),
            name: format!("Body {i}"),
            mass_kg: mass,
            radius_m: 1.0e3,
            position_m: pos,
            velocity_mps: DVec3::ZERO,
            class: BodyClass::Asteroid,
            color_rgb: None,
        });
        positions.push(pos);
    }
    (bodies, positions)
}
