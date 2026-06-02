//! Criterion benchmarks: direct O(n²) vs Barnes–Hut.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::DVec3;
use graviton::physics::body::{Body, BodyClass};
use graviton::physics::barnes_hut;
use graviton::physics::gravity::direct_accelerations;

fn random_system(n: usize) -> (Vec<Body>, Vec<DVec3>) {
    let mut bodies = Vec::with_capacity(n);
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 * 0.31;
        let pos = DVec3::new(
            1.0e9 * t.cos(),
            1.0e9 * t.sin(),
            (i as f64 - n as f64 / 2.0) * 1.0e7,
        );
        bodies.push(Body {
            id: format!("b{i}"),
            name: format!("b{i}"),
            mass_kg: 1.0e22,
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

fn bench_acceleration(c: &mut Criterion) {
    let mut group = c.benchmark_group("acceleration");
    for n in [10usize, 100, 500, 1000] {
        let (bodies, positions) = random_system(n);
        group.bench_with_input(BenchmarkId::new("direct", n), &n, |b, _| {
            b.iter(|| {
                black_box(direct_accelerations(
                    black_box(&bodies),
                    black_box(&positions),
                    1000.0,
                ));
            });
        });
        group.bench_with_input(BenchmarkId::new("barnes_hut", n), &n, |b, _| {
            b.iter(|| {
                black_box(barnes_hut::accelerations(
                    black_box(&bodies),
                    black_box(&positions),
                    1000.0,
                    0.7,
                ));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_acceleration);
criterion_main!(benches);
