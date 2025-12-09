//! Game loop with fixed timestep (Heijunka principle)
//!
//! Implements a fixed timestep game loop that ensures physics consistency
//! across all frame rates (30fps mobile to 144Hz+ desktop).

use core::fmt;

use serde::{Deserialize, Serialize};

/// Configuration for the game loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameLoopConfig {
    /// Fixed timestep for physics updates (seconds)
    pub fixed_dt: f32,
    /// Maximum frame time to prevent spiral of death
    pub max_frame_time: f32,
    /// Target frames per second (0 = unlimited)
    pub target_fps: u32,
}

impl GameLoopConfig {
    /// Default configuration (60 FPS physics, 120 FPS cap)
    #[must_use]
    pub const fn default_60fps() -> Self {
        Self {
            fixed_dt: 1.0 / 60.0,
            max_frame_time: 0.25, // Max 4 physics steps per frame
            target_fps: 0,        // Unlimited (vsync controlled)
        }
    }

    /// Configuration for mobile (30 FPS physics to save battery)
    #[must_use]
    pub const fn mobile() -> Self {
        Self {
            fixed_dt: 1.0 / 30.0,
            max_frame_time: 0.1,
            target_fps: 60,
        }
    }

    /// Configuration for high refresh rate displays
    #[must_use]
    pub const fn high_refresh() -> Self {
        Self {
            fixed_dt: 1.0 / 120.0,
            max_frame_time: 0.25,
            target_fps: 0,
        }
    }
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self::default_60fps()
    }
}

/// State of the game loop accumulator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameLoopState {
    /// Accumulated time for fixed updates
    accumulator: f32,
    /// Time of last frame
    last_frame_time: f32,
    /// Total elapsed time
    total_time: f32,
    /// Frame counter
    frame_count: u64,
    /// Physics tick counter
    tick_count: u64,
}

impl GameLoopState {
    /// Creates a new game loop state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            accumulator: 0.0,
            last_frame_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            tick_count: 0,
        }
    }

    /// Returns the total elapsed time
    #[must_use]
    pub const fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Returns the frame count
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the physics tick count
    #[must_use]
    pub const fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Returns the interpolation alpha for rendering
    ///
    /// This value (0.0 to 1.0) represents how far between physics ticks
    /// the current render frame is, allowing smooth interpolation.
    #[must_use]
    pub fn alpha(&self, fixed_dt: f32) -> f32 {
        if fixed_dt <= 0.0 {
            return 0.0;
        }
        self.accumulator / fixed_dt
    }
}

impl Default for GameLoopState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a game loop update
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameResult {
    /// Number of physics ticks that should run this frame
    pub physics_ticks: u32,
    /// Whether rendering should occur
    pub should_render: bool,
}

impl FrameResult {
    /// Creates a result with the given tick count
    #[must_use]
    pub const fn new(physics_ticks: u32) -> Self {
        Self {
            physics_ticks,
            should_render: true,
        }
    }
}

/// Game loop manager implementing fixed timestep with interpolation
///
/// # Example
///
/// ```
/// use jugar_core::{GameLoop, GameLoopConfig};
///
/// let mut game_loop = GameLoop::new(GameLoopConfig::default_60fps());
///
/// // Game loop
/// let current_time = 0.1; // From platform
/// let result = game_loop.update(current_time);
///
/// // Run physics ticks
/// for _ in 0..result.physics_ticks {
///     // physics.step(game_loop.config().fixed_dt);
/// }
///
/// // Render with interpolation
/// if result.should_render {
///     let alpha = game_loop.alpha();
///     // render(alpha);
/// }
/// ```
pub struct GameLoop {
    config: GameLoopConfig,
    state: GameLoopState,
}

impl GameLoop {
    /// Creates a new game loop with the given configuration
    #[must_use]
    pub const fn new(config: GameLoopConfig) -> Self {
        Self {
            config,
            state: GameLoopState::new(),
        }
    }

    /// Returns the configuration
    #[must_use]
    pub const fn config(&self) -> &GameLoopConfig {
        &self.config
    }

    /// Returns the current state
    #[must_use]
    pub const fn state(&self) -> &GameLoopState {
        &self.state
    }

    /// Returns the interpolation alpha for rendering
    #[must_use]
    pub fn alpha(&self) -> f32 {
        self.state.alpha(self.config.fixed_dt)
    }

    /// Updates the game loop with the current time
    ///
    /// Returns the number of physics ticks to run.
    pub fn update(&mut self, current_time: f32) -> FrameResult {
        // Calculate frame time
        let mut frame_time = current_time - self.state.last_frame_time;
        self.state.last_frame_time = current_time;

        // First frame handling
        if self.state.frame_count == 0 {
            frame_time = self.config.fixed_dt;
        }

        // Clamp to prevent spiral of death
        if frame_time > self.config.max_frame_time {
            frame_time = self.config.max_frame_time;
        }

        // Update state
        self.state.total_time += frame_time;
        self.state.frame_count += 1;
        self.state.accumulator += frame_time;

        // Count physics ticks
        let mut ticks = 0u32;
        #[allow(clippy::while_float)]
        while self.state.accumulator >= self.config.fixed_dt {
            self.state.accumulator -= self.config.fixed_dt;
            self.state.tick_count += 1;
            ticks += 1;
        }

        FrameResult::new(ticks)
    }

    /// Resets the game loop state
    pub fn reset(&mut self) {
        self.state = GameLoopState::new();
    }

    /// Returns the fixed timestep
    #[must_use]
    pub const fn fixed_dt(&self) -> f32 {
        self.config.fixed_dt
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new(GameLoopConfig::default())
    }
}

impl fmt::Debug for GameLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameLoop")
            .field("fixed_dt", &self.config.fixed_dt)
            .field("frame_count", &self.state.frame_count)
            .field("tick_count", &self.state.tick_count)
            .finish()
    }
}

/// Game state machine for managing game modes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GameState {
    /// Initial loading state
    #[default]
    Loading,
    /// Main menu
    Menu,
    /// Active gameplay
    Playing,
    /// Game paused
    Paused,
    /// Game over screen
    GameOver,
}

impl GameState {
    /// Checks if this state allows gameplay updates
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Checks if this state should render the game world
    #[must_use]
    pub const fn should_render_world(&self) -> bool {
        matches!(self, Self::Playing | Self::Paused | Self::GameOver)
    }

    /// Attempts to transition to a new state
    ///
    /// Returns true if the transition is valid.
    #[must_use]
    pub const fn can_transition_to(&self, target: &Self) -> bool {
        matches!(
            (self, target),
            (Self::Loading, Self::Menu)
                | (Self::Menu, Self::Playing | Self::Loading)
                | (Self::Playing, Self::Paused | Self::GameOver | Self::Menu)
                | (Self::Paused | Self::GameOver, Self::Playing | Self::Menu)
        )
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loading => write!(f, "Loading"),
            Self::Menu => write!(f, "Menu"),
            Self::Playing => write!(f, "Playing"),
            Self::Paused => write!(f, "Paused"),
            Self::GameOver => write!(f, "GameOver"),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::let_underscore_must_use,
    clippy::cast_precision_loss
)]
mod tests {
    use super::*;

    // ==================== CONFIG TESTS ====================

    #[test]
    fn test_config_default_60fps() {
        let config = GameLoopConfig::default_60fps();
        assert!((config.fixed_dt - 1.0 / 60.0).abs() < 0.001);
        assert!((config.max_frame_time - 0.25).abs() < 0.001);
        assert_eq!(config.target_fps, 0);
    }

    #[test]
    fn test_config_mobile() {
        let config = GameLoopConfig::mobile();
        assert!((config.fixed_dt - 1.0 / 30.0).abs() < 0.001);
        assert!((config.max_frame_time - 0.1).abs() < 0.001);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_config_high_refresh() {
        let config = GameLoopConfig::high_refresh();
        assert!((config.fixed_dt - 1.0 / 120.0).abs() < 0.001);
        assert!((config.max_frame_time - 0.25).abs() < 0.001);
        assert_eq!(config.target_fps, 0);
    }

    #[test]
    fn test_config_default() {
        let config = GameLoopConfig::default();
        assert!((config.fixed_dt - 1.0 / 60.0).abs() < 0.001);
    }

    // ==================== GAME LOOP TESTS ====================

    #[test]
    fn test_game_loop_single_tick() {
        let mut game_loop = GameLoop::new(GameLoopConfig::default_60fps());

        // First frame at time 0
        let result = game_loop.update(0.0);
        assert_eq!(result.physics_ticks, 1); // First frame always runs one tick
        assert!(result.should_render);
    }

    #[test]
    fn test_game_loop_multiple_ticks() {
        let mut game_loop = GameLoop::new(GameLoopConfig::default_60fps());
        let fixed_dt = game_loop.fixed_dt();

        // First frame
        let _ = game_loop.update(0.0);

        // Frame at 2x fixed_dt - should run 2 physics ticks
        let result = game_loop.update(fixed_dt * 2.0);
        assert_eq!(result.physics_ticks, 2);
    }

    #[test]
    fn test_game_loop_accumulator() {
        let mut game_loop = GameLoop::new(GameLoopConfig::default_60fps());
        let fixed_dt = game_loop.fixed_dt();

        let _ = game_loop.update(0.0);
        let _ = game_loop.update(fixed_dt * 1.5);

        // Should have run 1 tick with 0.5*fixed_dt remaining
        let alpha = game_loop.alpha();
        assert!(
            alpha > 0.4 && alpha < 0.6,
            "Alpha should be ~0.5, got {alpha}"
        );
    }

    #[test]
    fn test_game_loop_max_frame_time() {
        let mut game_loop = GameLoop::new(GameLoopConfig {
            fixed_dt: 1.0 / 60.0,
            max_frame_time: 0.1, // Max 6 ticks at 60fps
            target_fps: 0,
        });

        let _ = game_loop.update(0.0);

        // Huge time jump should be clamped
        let result = game_loop.update(10.0);

        // Should be clamped to max_frame_time / fixed_dt ticks
        assert!(result.physics_ticks <= 6);
    }

    #[test]
    fn test_game_loop_reset() {
        let mut game_loop = GameLoop::default();
        let _ = game_loop.update(0.0);
        let _ = game_loop.update(1.0);

        game_loop.reset();

        assert_eq!(game_loop.state().frame_count(), 0);
        assert_eq!(game_loop.state().tick_count(), 0);
    }

    #[test]
    fn test_game_loop_frame_count_increments() {
        let mut game_loop = GameLoop::default();

        for i in 0..10 {
            let _ = game_loop.update(i as f32 * 0.016);
        }

        assert_eq!(game_loop.state().frame_count(), 10);
    }

    // ==================== GAME STATE TESTS ====================

    #[test]
    fn test_game_state_transitions() {
        assert!(GameState::Loading.can_transition_to(&GameState::Menu));
        assert!(GameState::Menu.can_transition_to(&GameState::Playing));
        assert!(GameState::Playing.can_transition_to(&GameState::Paused));
        assert!(GameState::Paused.can_transition_to(&GameState::Playing));
        assert!(GameState::Playing.can_transition_to(&GameState::GameOver));
        assert!(GameState::GameOver.can_transition_to(&GameState::Menu));
    }

    #[test]
    fn test_game_state_invalid_transitions() {
        assert!(!GameState::Loading.can_transition_to(&GameState::Playing));
        assert!(!GameState::Paused.can_transition_to(&GameState::GameOver));
    }

    #[test]
    fn test_game_state_is_active() {
        assert!(!GameState::Loading.is_active());
        assert!(!GameState::Menu.is_active());
        assert!(GameState::Playing.is_active());
        assert!(!GameState::Paused.is_active());
    }

    #[test]
    fn test_game_state_should_render_world() {
        assert!(!GameState::Loading.should_render_world());
        assert!(GameState::Playing.should_render_world());
        assert!(GameState::Paused.should_render_world());
        assert!(GameState::GameOver.should_render_world());
    }

    #[test]
    fn test_game_state_display() {
        assert_eq!(format!("{}", GameState::Loading), "Loading");
        assert_eq!(format!("{}", GameState::Menu), "Menu");
        assert_eq!(format!("{}", GameState::Playing), "Playing");
        assert_eq!(format!("{}", GameState::Paused), "Paused");
        assert_eq!(format!("{}", GameState::GameOver), "GameOver");
    }

    #[test]
    fn test_game_state_default() {
        let state = GameState::default();
        assert_eq!(state, GameState::Loading);
    }

    #[test]
    fn test_game_state_menu_should_not_render_world() {
        assert!(!GameState::Menu.should_render_world());
    }

    #[test]
    fn test_game_state_menu_not_active() {
        assert!(!GameState::Menu.is_active());
    }

    #[test]
    fn test_game_state_paused_transitions() {
        assert!(GameState::Paused.can_transition_to(&GameState::Menu));
        assert!(GameState::GameOver.can_transition_to(&GameState::Playing));
    }

    #[test]
    fn test_game_state_menu_to_loading() {
        assert!(GameState::Menu.can_transition_to(&GameState::Loading));
    }

    // ==================== GAME LOOP STATE TESTS ====================

    #[test]
    fn test_game_loop_state_new() {
        let state = GameLoopState::new();
        assert!((state.total_time() - 0.0).abs() < f32::EPSILON);
        assert_eq!(state.frame_count(), 0);
        assert_eq!(state.tick_count(), 0);
    }

    #[test]
    fn test_game_loop_state_default() {
        let state = GameLoopState::default();
        assert_eq!(state.frame_count(), 0);
    }

    #[test]
    fn test_game_loop_state_alpha_zero_dt() {
        let state = GameLoopState::new();
        let alpha = state.alpha(0.0);
        assert!((alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_game_loop_state_alpha_negative_dt() {
        let state = GameLoopState::new();
        let alpha = state.alpha(-1.0);
        assert!((alpha - 0.0).abs() < f32::EPSILON);
    }

    // ==================== FRAME RESULT TESTS ====================

    #[test]
    fn test_frame_result_new() {
        let result = FrameResult::new(5);
        assert_eq!(result.physics_ticks, 5);
        assert!(result.should_render);
    }

    // ==================== GAME LOOP DEBUG TEST ====================

    #[test]
    fn test_game_loop_debug() {
        let game_loop = GameLoop::default();
        let debug_str = format!("{game_loop:?}");
        assert!(debug_str.contains("GameLoop"));
        assert!(debug_str.contains("fixed_dt"));
    }

    #[test]
    fn test_game_loop_config_accessor() {
        let game_loop = GameLoop::default();
        let config = game_loop.config();
        assert!((config.fixed_dt - 1.0 / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_game_loop_total_time_increases() {
        let mut game_loop = GameLoop::default();
        let _ = game_loop.update(0.0);
        let _ = game_loop.update(1.0);
        assert!(game_loop.state().total_time() > 0.0);
    }

    // ==================== BEHAVIORAL TESTS (MUTATION-RESISTANT) ====================

    #[test]
    fn test_physics_actually_runs_correct_times() {
        let config = GameLoopConfig {
            fixed_dt: 0.1, // 10 Hz for easy math
            max_frame_time: 1.0,
            target_fps: 0,
        };
        let mut game_loop = GameLoop::new(config);

        // First frame
        let _ = game_loop.update(0.0);

        // After 0.35 seconds, should have 3 ticks (0.1, 0.2, 0.3) with 0.05 remaining
        let result = game_loop.update(0.35);
        assert_eq!(
            result.physics_ticks, 3,
            "Should run exactly 3 physics ticks for 0.35s at 0.1s timestep"
        );

        // Verify accumulator state
        let alpha = game_loop.alpha();
        assert!(
            (alpha - 0.5).abs() < 0.01,
            "Alpha should be 0.5 (0.05/0.1), got {alpha}"
        );
    }

    #[test]
    fn test_interpolation_actually_affects_rendering() {
        let config = GameLoopConfig {
            fixed_dt: 0.1,
            max_frame_time: 1.0,
            target_fps: 0,
        };
        let mut game_loop = GameLoop::new(config);

        let _ = game_loop.update(0.0);
        let _ = game_loop.update(0.15); // 1 tick, 0.05 remaining

        let alpha = game_loop.alpha();

        // Simulate position interpolation
        let prev_pos: f32 = 0.0;
        let curr_pos: f32 = 10.0;
        let interpolated = (curr_pos - prev_pos).mul_add(alpha, prev_pos);

        assert!(
            (interpolated - 5.0).abs() < 0.1,
            "Interpolated position should be ~5.0 at alpha=0.5, got {interpolated}"
        );
    }
}
