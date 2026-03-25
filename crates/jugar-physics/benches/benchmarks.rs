//! Criterion benchmarks for jugar-physics engine.
//!
//! Benchmarks physics step, body creation, and world operations
//! which are the hot paths in the game simulation loop.

#![allow(missing_docs, unused_results, clippy::unwrap_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use jugar_core::{Position, Velocity};
use jugar_physics::{PhysicsBackend, PhysicsWorld, RigidBody};

fn bench_physics_step_10_bodies(c: &mut Criterion) {
    let mut world = PhysicsWorld::with_backend(PhysicsBackend::Scalar);
    for i in 0..10 {
        let body = RigidBody::new(Position::new(i as f32 * 10.0, 100.0))
            .with_velocity(Velocity::new(1.0, 0.0))
            .with_mass(1.0);
        world.add_body(body);
    }
    c.bench_function("physics_step_10_bodies", |b| {
        b.iter(|| {
            world.step(black_box(1.0 / 60.0));
        });
    });
}

fn bench_physics_step_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_step");
    for count in [10, 100, 500, 1000] {
        let mut world = PhysicsWorld::with_backend(PhysicsBackend::Scalar);
        for i in 0..count {
            let body = RigidBody::new(Position::new(i as f32, 0.0));
            world.add_body(body);
        }
        group.bench_with_input(
            BenchmarkId::new("bodies", count),
            &count,
            |b, _| {
                b.iter(|| {
                    world.step(black_box(1.0 / 60.0));
                });
            },
        );
    }
    group.finish();
}

fn bench_body_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_create", |b| {
        b.iter(|| {
            let body = RigidBody::new(Position::new(black_box(10.0), black_box(20.0)))
                .with_velocity(Velocity::new(1.0, -1.0))
                .with_mass(2.5);
            black_box(body);
        });
    });
}

fn bench_world_add_body(c: &mut Criterion) {
    c.bench_function("world_add_body", |b| {
        let mut world = PhysicsWorld::new();
        b.iter(|| {
            let body = RigidBody::new(Position::new(0.0, 0.0));
            world.add_body(body);
        });
    });
}

fn bench_world_get_body(c: &mut Criterion) {
    let mut world = PhysicsWorld::new();
    let mut handles = Vec::new();
    for i in 0..100 {
        let body = RigidBody::new(Position::new(i as f32, 0.0));
        handles.push(world.add_body(body));
    }
    c.bench_function("world_get_body_100", |b| {
        let mut idx = 0usize;
        b.iter(|| {
            let handle = handles[idx % handles.len()];
            black_box(world.get_body(handle));
            idx += 1;
        });
    });
}

criterion_group!(
    benches,
    bench_physics_step_10_bodies,
    bench_physics_step_scaling,
    bench_body_creation,
    bench_world_add_body,
    bench_world_get_body,
);
criterion_main!(benches);
