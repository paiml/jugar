//! Benchmarks for game loop and core engine operations.

#![allow(missing_docs, unused_results)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use jugar_core::{GameLoop, GameLoopConfig, Position, Velocity, World};

fn bench_game_loop_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_loop");

    for time in &[0.016, 0.033, 0.050] {
        group.bench_with_input(BenchmarkId::new("update", time), time, |b, &time| {
            let mut game_loop = GameLoop::new(GameLoopConfig::default());
            let mut t = 0.0f32;
            b.iter(|| {
                t += time;
                game_loop.update(black_box(t))
            });
        });
    }

    group.finish();
}

fn bench_ecs_spawn(c: &mut Criterion) {
    c.bench_function("ecs_spawn_entity", |b| {
        let mut world = World::new();
        b.iter(|| {
            let entity = world.spawn();
            black_box(entity)
        });
    });
}

fn bench_ecs_component_add(c: &mut Criterion) {
    c.bench_function("ecs_add_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        b.iter(|| {
            world.add_component(entity, Position::new(black_box(1.0), black_box(2.0)));
        });
    });
}

fn bench_ecs_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecs_iteration");

    for count in &[100, 1000, 10000] {
        let mut world = World::new();
        for i in 0..*count {
            let entity = world.spawn();
            world.add_component(entity, Position::new(i as f32, i as f32));
            world.add_component(entity, Velocity::new(1.0, 1.0));
        }

        group.bench_with_input(BenchmarkId::new("entities", count), count, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for entity in world.entities() {
                    if let Some(pos) = world.get_component::<Position>(entity) {
                        sum += pos.x + pos.y;
                    }
                }
                black_box(sum)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_game_loop_update,
    bench_ecs_spawn,
    bench_ecs_component_add,
    bench_ecs_iteration
);
criterion_main!(benches);
