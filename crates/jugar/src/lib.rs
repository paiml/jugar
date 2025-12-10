//! # Jugar
//!
//! WASM-native game engine for mobile to ultrawide desktop experiences.
//!
//! Jugar provides a complete game development framework targeting `wasm32-unknown-unknown`
//! with pure WASM binary output (zero JavaScript).
//!
//! ## Features
//!
//! - **ECS Architecture**: High-performance Entity-Component-System
//! - **Responsive Design**: Scales from mobile to 32:9 ultrawide
//! - **Physics**: Tiered physics with WebGPU → WASM-SIMD → Scalar fallback
//! - **AI**: GOAP planner and Behavior Trees
//! - **Audio**: Spatial 2D audio system
//! - **Procedural Generation**: Noise, dungeons, and WFC
//!
//! ## Example
//!
//! ```rust,ignore
//! use jugar::prelude::*;
//!
//! fn main() {
//!     let mut engine = JugarEngine::new(JugarConfig::default());
//!     engine.run(|_| LoopControl::Exit);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export all crates
pub use jugar_ai as ai;
pub use jugar_audio as audio;
pub use jugar_core as game_core;
pub use jugar_input as input;
pub use jugar_physics as physics;
pub use jugar_procgen as procgen;
pub use jugar_render as render;
pub use jugar_ui as ui;

/// Prelude for common imports
pub mod prelude {
    pub use crate::{JugarConfig, JugarEngine, LoopControl};

    // Core types
    pub use jugar_core::{
        Anchor, Camera, Entity, FrameResult, GameLoop, GameLoopConfig, GameState, Position, Rect,
        ScaleMode, Sprite, UiElement, Velocity, World,
    };

    // Input
    pub use jugar_input::{
        ButtonState, GamepadButton, InputAction, InputState, KeyCode, MouseButton, TouchEvent,
        TouchPhase,
    };

    // Render
    pub use jugar_render::{
        calculate_anchored_position, AspectRatio, RenderCommand, RenderQueue, Viewport,
    };

    // UI
    pub use jugar_ui::{Button, ButtonState as UiButtonState, Label, UiContainer, WidgetId};

    // Physics
    pub use jugar_physics::{BodyHandle, PhysicsBackend, PhysicsWorld, RigidBody};

    // Audio
    pub use jugar_audio::{AudioChannel, AudioHandle, AudioListener, AudioSystem, SoundSource};

    // AI
    pub use jugar_ai::{
        Action, BehaviorNode, Goal, NodeStatus, Planner, Selector, Sequence, WorldState,
    };

    // Procgen
    pub use jugar_procgen::{
        Direction, Dungeon, DungeonGenerator, DungeonTile, Rng, Room, ValueNoise, Wfc,
    };

    // External
    pub use glam::Vec2;
}

/// Jugar engine errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum JugarError {
    /// Initialization error
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Result type for Jugar operations
pub type Result<T> = core::result::Result<T, JugarError>;

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JugarConfig {
    /// Window/canvas width
    pub width: u32,
    /// Window/canvas height
    pub height: u32,
    /// Target frames per second
    pub target_fps: u32,
    /// Fixed timestep for physics (in seconds)
    pub fixed_timestep: f32,
    /// Maximum delta time (to prevent spiral of death)
    pub max_delta: f32,
    /// Enable vsync
    pub vsync: bool,
    /// Application title
    pub title: String,
}

impl Default for JugarConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            target_fps: 60,
            fixed_timestep: 1.0 / 60.0,
            max_delta: 0.25,
            vsync: true,
            title: "Jugar Game".to_string(),
        }
    }
}

impl JugarConfig {
    /// Creates a new configuration
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Sets the title
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the target FPS
    #[must_use]
    pub const fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self.fixed_timestep = 1.0 / fps as f32;
        self
    }

    /// Mobile portrait preset
    #[must_use]
    pub fn mobile_portrait() -> Self {
        Self {
            width: 1080,
            height: 1920,
            ..Default::default()
        }
    }

    /// Mobile landscape preset
    #[must_use]
    pub fn mobile_landscape() -> Self {
        Self {
            width: 1920,
            height: 1080,
            ..Default::default()
        }
    }

    /// Ultrawide 21:9 preset
    #[must_use]
    pub fn ultrawide() -> Self {
        Self {
            width: 3440,
            height: 1440,
            ..Default::default()
        }
    }

    /// Super ultrawide 32:9 preset
    #[must_use]
    pub fn super_ultrawide() -> Self {
        Self {
            width: 5120,
            height: 1440,
            ..Default::default()
        }
    }
}

/// Loop control returned from update callback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopControl {
    /// Continue running
    Continue,
    /// Exit the game loop
    Exit,
}

/// Engine time information
#[derive(Debug, Clone, Copy, Default)]
pub struct Time {
    /// Total elapsed time since start (seconds)
    pub elapsed: f32,
    /// Delta time for this frame (seconds)
    pub delta: f32,
    /// Fixed timestep for physics
    pub fixed_delta: f32,
    /// Current frame number
    pub frame: u64,
}

/// The main Jugar game engine
pub struct JugarEngine {
    config: JugarConfig,
    time: Time,
    viewport: render::Viewport,
    input: input::InputState,
    audio: audio::AudioSystem,
    world: jugar_core::World,
    physics: physics::PhysicsWorld,
    ui: ui::UiContainer,
    game_loop: jugar_core::GameLoop,
    running: bool,
}

impl JugarEngine {
    /// Creates a new Jugar engine with the given configuration
    #[must_use]
    pub fn new(config: JugarConfig) -> Self {
        let viewport = render::Viewport::new(config.width, config.height);
        let ui_width = viewport.width as f32;
        let ui_height = viewport.height as f32;
        let loop_config = jugar_core::GameLoopConfig {
            fixed_dt: config.fixed_timestep,
            max_frame_time: config.max_delta,
            target_fps: config.target_fps,
        };
        let game_loop = jugar_core::GameLoop::new(loop_config);

        Self {
            config,
            time: Time::default(),
            viewport,
            input: input::InputState::new(),
            audio: audio::AudioSystem::new(),
            world: jugar_core::World::new(),
            physics: physics::PhysicsWorld::new(),
            ui: ui::UiContainer::new(ui_width, ui_height),
            game_loop,
            running: false,
        }
    }

    /// Gets the configuration
    #[must_use]
    pub const fn config(&self) -> &JugarConfig {
        &self.config
    }

    /// Gets the current time
    #[must_use]
    pub const fn time(&self) -> &Time {
        &self.time
    }

    /// Gets the viewport
    #[must_use]
    pub const fn viewport(&self) -> &render::Viewport {
        &self.viewport
    }

    /// Gets the viewport mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn viewport_mut(&mut self) -> &mut render::Viewport {
        &mut self.viewport
    }

    /// Gets the input state
    #[must_use]
    pub const fn input(&self) -> &input::InputState {
        &self.input
    }

    /// Gets the input state mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn input_mut(&mut self) -> &mut input::InputState {
        &mut self.input
    }

    /// Gets the audio system
    #[must_use]
    pub const fn audio(&self) -> &audio::AudioSystem {
        &self.audio
    }

    /// Gets the audio system mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn audio_mut(&mut self) -> &mut audio::AudioSystem {
        &mut self.audio
    }

    /// Gets the ECS world
    #[must_use]
    pub const fn world(&self) -> &jugar_core::World {
        &self.world
    }

    /// Gets the ECS world mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn world_mut(&mut self) -> &mut jugar_core::World {
        &mut self.world
    }

    /// Gets the physics world
    #[must_use]
    pub const fn physics(&self) -> &physics::PhysicsWorld {
        &self.physics
    }

    /// Gets the physics world mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn physics_mut(&mut self) -> &mut physics::PhysicsWorld {
        &mut self.physics
    }

    /// Gets the UI container
    #[must_use]
    pub const fn ui(&self) -> &ui::UiContainer {
        &self.ui
    }

    /// Gets the UI container mutably
    #[allow(clippy::missing_const_for_fn)]
    pub fn ui_mut(&mut self) -> &mut ui::UiContainer {
        &mut self.ui
    }

    /// Gets the game loop
    #[must_use]
    pub const fn game_loop(&self) -> &jugar_core::GameLoop {
        &self.game_loop
    }

    /// Resizes the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport.resize(width, height);
        self.ui.set_viewport_size(width as f32, height as f32);
    }

    /// Checks if the engine is running
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    /// Runs the game loop with a callback
    ///
    /// The callback receives a reference to the engine and returns `LoopControl`.
    pub fn run<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Self) -> LoopControl,
    {
        self.running = true;
        let start_time = std::time::Instant::now();

        while self.running {
            let elapsed = start_time.elapsed().as_secs_f32();

            // Update game loop and get physics ticks
            let frame_result = self.game_loop.update(elapsed);

            self.time.delta = elapsed - self.time.elapsed;
            self.time.elapsed = elapsed;
            self.time.fixed_delta = self.config.fixed_timestep;
            self.time.frame += 1;

            // Run physics for each tick
            for _ in 0..frame_result.physics_ticks {
                let _ = self.physics.step(self.config.fixed_timestep);
            }

            // Update audio
            self.audio.update(self.time.delta);

            // Call user callback
            if callback(self) == LoopControl::Exit {
                self.running = false;
            }

            // Advance input state
            self.input.advance_frame();
        }
    }

    /// Steps the engine for a single frame (useful for testing)
    pub fn step(&mut self, delta: f32) {
        self.time.delta = delta.min(self.config.max_delta);
        self.time.elapsed += self.time.delta;
        self.time.frame += 1;

        // Update game loop and get physics ticks
        let frame_result = self.game_loop.update(self.time.elapsed);

        // Run physics for each tick
        for _ in 0..frame_result.physics_ticks {
            let _ = self.physics.step(self.config.fixed_timestep);
        }

        self.audio.update(self.time.delta);
        self.input.advance_frame();
    }

    /// Stops the engine
    pub const fn stop(&mut self) {
        self.running = false;
    }
}

impl Default for JugarEngine {
    fn default() -> Self {
        Self::new(JugarConfig::default())
    }
}

impl fmt::Debug for JugarEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JugarEngine")
            .field("config", &self.config)
            .field("time", &self.time)
            .field("running", &self.running)
            .field("physics_backend", &self.physics.backend())
            .finish_non_exhaustive()
    }
}

/// Creates a simple game with default configuration
#[must_use]
pub fn create_game() -> JugarEngine {
    JugarEngine::default()
}

/// Creates a game with mobile configuration
#[must_use]
pub fn create_mobile_game() -> JugarEngine {
    JugarEngine::new(JugarConfig::mobile_landscape())
}

/// Creates a game with ultrawide configuration
#[must_use]
pub fn create_ultrawide_game() -> JugarEngine {
    JugarEngine::new(JugarConfig::super_ultrawide())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = JugarConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_config_mobile_portrait() {
        let config = JugarConfig::mobile_portrait();
        assert_eq!(config.width, 1080);
        assert_eq!(config.height, 1920);
    }

    #[test]
    fn test_config_super_ultrawide() {
        let config = JugarConfig::super_ultrawide();
        assert_eq!(config.width, 5120);
        assert_eq!(config.height, 1440);
    }

    #[test]
    fn test_config_builder() {
        let config = JugarConfig::new(800, 600)
            .with_title("Test Game")
            .with_target_fps(30);

        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.title, "Test Game");
        assert_eq!(config.target_fps, 30);
    }

    #[test]
    fn test_engine_creation() {
        let engine = JugarEngine::new(JugarConfig::default());
        assert!(!engine.is_running());
        assert_eq!(engine.time().frame, 0);
    }

    #[test]
    fn test_engine_default() {
        let engine = JugarEngine::default();
        assert_eq!(engine.config().width, 1920);
    }

    #[test]
    fn test_engine_resize() {
        let mut engine = JugarEngine::default();
        engine.resize(1280, 720);

        assert_eq!(engine.viewport().width, 1280);
        assert_eq!(engine.viewport().height, 720);
    }

    #[test]
    fn test_engine_step() {
        let mut engine = JugarEngine::default();
        engine.step(1.0 / 60.0);

        assert_eq!(engine.time().frame, 1);
        assert!(engine.time().elapsed > 0.0);
    }

    #[test]
    fn test_engine_step_multiple() {
        let mut engine = JugarEngine::default();

        for _ in 0..10 {
            engine.step(1.0 / 60.0);
        }

        assert_eq!(engine.time().frame, 10);
    }

    #[test]
    fn test_engine_run_exit() {
        let mut engine = JugarEngine::default();
        let mut count = 0;

        engine.run(|_| {
            count += 1;
            if count >= 5 {
                LoopControl::Exit
            } else {
                LoopControl::Continue
            }
        });

        assert_eq!(count, 5);
        assert!(!engine.is_running());
    }

    #[test]
    fn test_engine_stop() {
        let mut engine = JugarEngine::default();
        engine.running = true;
        engine.stop();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_create_game() {
        let engine = create_game();
        assert_eq!(engine.config().width, 1920);
    }

    #[test]
    fn test_create_mobile_game() {
        let engine = create_mobile_game();
        assert_eq!(engine.config().width, 1920);
        assert_eq!(engine.config().height, 1080);
    }

    #[test]
    fn test_create_ultrawide_game() {
        let engine = create_ultrawide_game();
        assert_eq!(engine.config().width, 5120);
    }

    #[test]
    fn test_engine_accessors() {
        let mut engine = JugarEngine::default();

        // Test all accessors compile and work
        let _ = engine.config();
        let _ = engine.time();
        let _ = engine.viewport();
        let _ = engine.viewport_mut();
        let _ = engine.input();
        let _ = engine.input_mut();
        let _ = engine.audio();
        let _ = engine.audio_mut();
        let _ = engine.world();
        let _ = engine.world_mut();
        let _ = engine.physics();
        let _ = engine.physics_mut();
        let _ = engine.ui();
        let _ = engine.ui_mut();
        let _ = engine.game_loop();
    }

    #[test]
    fn test_loop_control() {
        assert_eq!(LoopControl::Continue, LoopControl::Continue);
        assert_ne!(LoopControl::Continue, LoopControl::Exit);
    }

    #[test]
    fn test_time_default() {
        let time = Time::default();
        assert!(time.elapsed.abs() < f32::EPSILON);
        assert!(time.delta.abs() < f32::EPSILON);
        assert_eq!(time.frame, 0);
    }
}
