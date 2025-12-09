//! End-to-end integration tests for the Jugar game engine.
//!
//! These tests verify cross-crate integration and the overall engine workflow.

use jugar::game_core::{GameLoop, GameLoopConfig, Position, Velocity, World};
use jugar::input::{InputState, KeyCode};
use jugar::prelude::*;

/// Test that all core crate types are accessible via jugar re-exports.
#[test]
fn test_core_types_reexported() {
    let _ = core::mem::size_of::<GameLoop>();
    let _ = core::mem::size_of::<World>();
    let _ = core::mem::size_of::<Position>();
    let _ = core::mem::size_of::<Velocity>();
}

/// Test basic ECS workflow through jugar facade.
#[test]
fn test_ecs_workflow() {
    let mut world = World::new();

    // Spawn an entity
    let entity = world.spawn();

    // Add components
    world.add_component(entity, Position::new(10.0, 20.0));
    world.add_component(entity, Velocity::new(1.0, 2.0));

    // Verify components are accessible
    let pos = world.get_component::<Position>(entity);
    assert!(pos.is_some());

    let vel = world.get_component::<Velocity>(entity);
    assert!(vel.is_some());
}

/// Test game loop integration through jugar facade.
#[test]
fn test_game_loop_integration() {
    let config = GameLoopConfig::default();
    let mut game_loop = GameLoop::new(config);

    // Simulate several updates
    for i in 1..=10 {
        let result = game_loop.update(i as f32 * 0.016);
        assert!(
            result.physics_ticks <= 10,
            "Physics ticks should be bounded"
        );
    }
}

/// Test input state management through jugar facade.
#[test]
fn test_input_state_management() {
    let mut input = InputState::new();

    // Initially no keys pressed
    assert!(!input.is_key_pressed(KeyCode::Space));

    // Press a key
    input.set_key_pressed(KeyCode::Space, true);
    assert!(input.is_key_pressed(KeyCode::Space));

    // Release the key
    input.set_key_pressed(KeyCode::Space, false);
    assert!(!input.is_key_pressed(KeyCode::Space));
}

/// Test full engine workflow.
#[test]
fn test_full_engine_workflow() {
    let mut engine = JugarEngine::default();

    // Step multiple times
    for _ in 0..10 {
        engine.step(1.0 / 60.0);
    }

    assert_eq!(engine.time().frame, 10);
    assert!(engine.time().elapsed > 0.0);
}

/// Test engine resize.
#[test]
fn test_engine_resize() {
    let mut engine = JugarEngine::default();
    engine.resize(1280, 720);

    assert_eq!(engine.viewport().width, 1280);
    assert_eq!(engine.viewport().height, 720);
}

/// Test engine run with immediate exit.
#[test]
fn test_engine_run_exit() {
    let mut engine = JugarEngine::default();
    let mut count = 0;

    engine.run(|_| {
        count += 1;
        if count >= 3 {
            LoopControl::Exit
        } else {
            LoopControl::Continue
        }
    });

    assert_eq!(count, 3);
    assert!(!engine.is_running());
}

/// Test config presets.
#[test]
fn test_config_presets() {
    let mobile = JugarConfig::mobile_portrait();
    assert_eq!(mobile.width, 1080);
    assert_eq!(mobile.height, 1920);

    let ultrawide = JugarConfig::super_ultrawide();
    assert_eq!(ultrawide.width, 5120);
}
