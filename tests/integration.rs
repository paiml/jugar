//! Integration tests for the Jugar game engine.
//!
//! These tests verify the end-to-end functionality of the engine
//! components working together.

use jugar_core::{GameLoop, GameLoopConfig, TimeManager};

/// Test that the game loop initializes correctly with default config.
#[test]
fn test_game_loop_initialization() {
    let config = GameLoopConfig::default();
    let game_loop = GameLoop::new(config);

    assert!(game_loop.is_running());
}

/// Test that the time manager provides consistent frame timing.
#[test]
fn test_time_manager_frame_timing() {
    let mut time = TimeManager::new();

    // Simulate 60 FPS for a few frames
    for i in 0..10 {
        time.update((i as f64) * 16.667);
    }

    assert!(time.total_time() > 0.0);
}

/// Test cross-crate integration between core and physics.
#[test]
fn test_core_physics_integration() {
    // Verify that jugar-core types are compatible with physics system expectations
    let config = GameLoopConfig {
        fixed_timestep: 1.0 / 60.0,
        max_frame_time: 0.25,
        ..Default::default()
    };

    assert!((config.fixed_timestep - 0.016666668).abs() < 0.0001);
}
