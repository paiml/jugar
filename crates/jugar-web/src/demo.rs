//! Demo mode and game mode management.
//!
//! This module implements the features from the Pong Improvements Demo Specification:
//! - A. Demo Mode (AI vs AI attract mode)
//! - B. Speed Toggle (1x to 1000x)
//! - C. Game Modes (Demo/1P/2P)
//! - Safety: Photosensitivity warning for high speeds

use serde::{Deserialize, Serialize};

/// Speed multiplier for physics simulation.
///
/// Higher speeds showcase SIMD/GPU acceleration by running multiple
/// physics updates per render frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpeedMultiplier {
    /// Normal speed: 60 physics updates/sec
    #[default]
    Normal = 1,
    /// 5x speed: 300 physics updates/sec
    Fast5x = 5,
    /// 10x speed: 600 physics updates/sec
    Fast10x = 10,
    /// 50x speed: 3,000 physics updates/sec
    Fast50x = 50,
    /// 100x speed: 6,000 physics updates/sec
    Fast100x = 100,
    /// 1000x speed: 60,000 physics updates/sec
    Fast1000x = 1000,
}

impl SpeedMultiplier {
    /// Returns the numeric multiplier value.
    #[must_use]
    pub const fn value(self) -> u32 {
        match self {
            Self::Normal => 1,
            Self::Fast5x => 5,
            Self::Fast10x => 10,
            Self::Fast50x => 50,
            Self::Fast100x => 100,
            Self::Fast1000x => 1000,
        }
    }

    /// Returns true if this speed requires a photosensitivity warning.
    ///
    /// Speeds above 10x may cause high-frequency visual flashing.
    #[must_use]
    pub const fn requires_warning(self) -> bool {
        matches!(self, Self::Fast50x | Self::Fast100x | Self::Fast1000x)
    }

    /// Returns the display label for this speed.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "1x",
            Self::Fast5x => "5x",
            Self::Fast10x => "10x",
            Self::Fast50x => "50x",
            Self::Fast100x => "100x",
            Self::Fast1000x => "1000x",
        }
    }

    /// Cycles to the next speed multiplier.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Normal => Self::Fast5x,
            Self::Fast5x => Self::Fast10x,
            Self::Fast10x => Self::Fast50x,
            Self::Fast50x => Self::Fast100x,
            Self::Fast100x => Self::Fast1000x,
            Self::Fast1000x => Self::Normal,
        }
    }

    /// Creates from keyboard shortcut (1-6).
    #[must_use]
    pub const fn from_key(key: u8) -> Option<Self> {
        match key {
            1 => Some(Self::Normal),
            2 => Some(Self::Fast5x),
            3 => Some(Self::Fast10x),
            4 => Some(Self::Fast50x),
            5 => Some(Self::Fast100x),
            6 => Some(Self::Fast1000x),
            _ => None,
        }
    }

    /// Returns all speed multipliers in order.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Normal,
            Self::Fast5x,
            Self::Fast10x,
            Self::Fast50x,
            Self::Fast100x,
            Self::Fast1000x,
        ]
    }
}

/// Game mode selection.
///
/// Determines which paddles are controlled by humans vs AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GameMode {
    /// Demo mode: AI vs AI (attract mode)
    Demo,
    /// Single player: Human (left, W/S keys) vs AI (right) - DEFAULT
    #[default]
    SinglePlayer,
    /// Two player: Human (left, W/S) vs Human (right, Up/Down arrows)
    TwoPlayer,
}

impl GameMode {
    /// Returns the enum variant name (Demo, SinglePlayer, TwoPlayer).
    /// Used for API/debug output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Demo => "Demo",
            Self::SinglePlayer => "SinglePlayer",
            Self::TwoPlayer => "TwoPlayer",
        }
    }

    /// Returns the display label for this mode.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Demo => "Demo",
            Self::SinglePlayer => "1 Player",
            Self::TwoPlayer => "2 Player",
        }
    }

    /// Returns the short label for buttons.
    #[must_use]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Demo => "Demo",
            Self::SinglePlayer => "1P",
            Self::TwoPlayer => "2P",
        }
    }

    /// Returns true if left paddle is AI-controlled.
    /// In Demo and SinglePlayer modes, the left paddle is AI.
    /// SinglePlayer: AI (left) vs Human (right/arrows)
    #[must_use]
    pub const fn left_is_ai(self) -> bool {
        matches!(self, Self::Demo | Self::SinglePlayer)
    }

    /// Returns true if right paddle is AI-controlled.
    /// Only in Demo mode is the right paddle AI.
    /// SinglePlayer: AI (left) vs Human (right/arrows)
    #[must_use]
    pub const fn right_is_ai(self) -> bool {
        matches!(self, Self::Demo)
    }

    /// Returns the label for the left paddle based on game mode.
    /// Shows controller type and keys if human-controlled.
    /// In 1P mode: AI on left, human on right
    /// In 2P mode: P2 on left (W/S), P1 on right (arrows)
    #[must_use]
    pub const fn left_paddle_label(self) -> &'static str {
        match self {
            Self::Demo | Self::SinglePlayer => "AI",
            Self::TwoPlayer => "P2 [W/S]",
        }
    }

    /// Returns the label for the right paddle based on game mode.
    /// Shows controller type and keys if human-controlled.
    /// In 1P mode: human on right (arrow keys)
    /// In 2P mode: P1 on right (arrow keys)
    #[must_use]
    pub const fn right_paddle_label(self) -> &'static str {
        match self {
            Self::Demo => "AI",
            Self::SinglePlayer | Self::TwoPlayer => "P1 [^/v]",
        }
    }

    /// Cycles to the next game mode.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Demo => Self::SinglePlayer,
            Self::SinglePlayer => Self::TwoPlayer,
            Self::TwoPlayer => Self::Demo,
        }
    }

    /// Returns all game modes in order.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Demo, Self::SinglePlayer, Self::TwoPlayer]
    }
}

/// Demo mode state tracking.
///
/// Manages auto-demo engagement and transition to player control.
#[derive(Debug, Clone)]
pub struct DemoState {
    /// Time since last user input (seconds)
    idle_time: f64,
    /// Threshold for auto-engage (seconds)
    auto_engage_threshold: f64,
    /// Whether demo was auto-engaged (vs manual)
    auto_engaged: bool,
    /// Current difficulty cycle time
    difficulty_cycle_time: f64,
    /// Difficulty cycle period (seconds)
    difficulty_cycle_period: f64,
    /// Current left AI difficulty (for demo mode)
    left_ai_difficulty: u8,
    /// Current right AI difficulty (for demo mode)
    right_ai_difficulty: u8,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            idle_time: 0.0,
            auto_engage_threshold: 10.0, // 10 seconds per spec
            auto_engaged: false,
            difficulty_cycle_time: 0.0,
            difficulty_cycle_period: 60.0, // 60 seconds per spec
            left_ai_difficulty: 7,         // Per spec: challenging
            right_ai_difficulty: 5,        // Per spec: slightly easier
        }
    }
}

impl DemoState {
    /// Creates a new demo state with custom thresholds.
    #[must_use]
    pub fn new(auto_engage_threshold: f64, difficulty_cycle_period: f64) -> Self {
        Self {
            auto_engage_threshold,
            difficulty_cycle_period,
            ..Default::default()
        }
    }

    /// Records user input, resetting idle timer.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable ref not yet stable
    pub fn record_input(&mut self) {
        self.idle_time = 0.0;
        self.auto_engaged = false;
    }

    /// Updates idle time and returns true if demo should auto-engage.
    pub fn update(&mut self, dt: f64, has_input: bool) -> bool {
        if has_input {
            self.record_input();
            return false;
        }

        self.idle_time += dt;

        // Check if we should auto-engage
        if !self.auto_engaged && self.idle_time >= self.auto_engage_threshold {
            self.auto_engaged = true;
            return true;
        }

        false
    }

    /// Updates difficulty cycling for demo mode.
    pub fn update_difficulty_cycle(&mut self, dt: f64) {
        self.difficulty_cycle_time += dt;

        // Cycle difficulties every period
        if self.difficulty_cycle_time >= self.difficulty_cycle_period {
            self.difficulty_cycle_time = 0.0;
            // Swap difficulties for variety
            core::mem::swap(&mut self.left_ai_difficulty, &mut self.right_ai_difficulty);
        }
    }

    /// Returns true if demo was auto-engaged (vs manually selected).
    #[must_use]
    pub const fn is_auto_engaged(&self) -> bool {
        self.auto_engaged
    }

    /// Returns the current left AI difficulty for demo mode.
    #[must_use]
    pub const fn left_difficulty(&self) -> u8 {
        self.left_ai_difficulty
    }

    /// Returns the current right AI difficulty for demo mode.
    #[must_use]
    pub const fn right_difficulty(&self) -> u8 {
        self.right_ai_difficulty
    }

    /// Returns idle time in seconds.
    #[must_use]
    pub const fn idle_time(&self) -> f64 {
        self.idle_time
    }

    /// Resets the demo state.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable ref not yet stable
    pub fn reset(&mut self) {
        self.idle_time = 0.0;
        self.auto_engaged = false;
        self.difficulty_cycle_time = 0.0;
    }
}

/// Performance statistics for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Current physics updates per second
    pub physics_updates_per_sec: u32,
    /// Current render FPS
    pub render_fps: f64,
    /// Physics backend name
    pub backend_name: String,
    /// Current speed multiplier
    pub speed_multiplier: u32,
}

impl PerformanceStats {
    /// Creates new stats with the given values.
    #[must_use]
    pub fn new(physics_ups: u32, render_fps: f64, backend: &str, speed: u32) -> Self {
        Self {
            physics_updates_per_sec: physics_ups,
            render_fps,
            backend_name: backend.to_string(),
            speed_multiplier: speed,
        }
    }

    /// Formats the stats for display.
    #[must_use]
    pub fn format_display(&self) -> String {
        format!(
            "Backend: {} | Physics: {}/s | Render: {:.0} FPS",
            self.backend_name, self.physics_updates_per_sec, self.render_fps
        )
    }
}

/// Attribution information for footer display.
#[derive(Debug, Clone)]
pub struct Attribution {
    /// Engine name and version
    pub engine_version: String,
    /// GitHub repository URL
    pub github_url: String,
    /// Organization website URL
    pub org_url: String,
    /// AI model filename
    pub model_filename: String,
    /// AI model size in bytes
    pub model_size: u32,
}

impl Default for Attribution {
    fn default() -> Self {
        Self {
            engine_version: "Jugar Engine v0.1.0".to_string(),
            github_url: "https://github.com/paiml/jugar".to_string(),
            org_url: "https://paiml.com".to_string(),
            model_filename: "pong-ai-v1.apr".to_string(),
            model_size: 491,
        }
    }
}

impl Attribution {
    /// Returns the GitHub link label.
    #[must_use]
    pub fn github_label(&self) -> String {
        "github.com/paiml/jugar".to_string()
    }

    /// Returns the organization link label.
    #[must_use]
    pub fn org_label(&self) -> String {
        "paiml.com".to_string()
    }

    /// Returns the model download label.
    #[must_use]
    pub fn model_label(&self) -> String {
        format!("{} ({} bytes)", self.model_filename, self.model_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SpeedMultiplier Tests ====================

    #[test]
    fn test_speed_multiplier_values() {
        assert_eq!(SpeedMultiplier::Normal.value(), 1);
        assert_eq!(SpeedMultiplier::Fast5x.value(), 5);
        assert_eq!(SpeedMultiplier::Fast10x.value(), 10);
        assert_eq!(SpeedMultiplier::Fast50x.value(), 50);
        assert_eq!(SpeedMultiplier::Fast100x.value(), 100);
        assert_eq!(SpeedMultiplier::Fast1000x.value(), 1000);
    }

    #[test]
    fn test_speed_multiplier_warning_threshold() {
        // Speeds <= 10x should NOT require warning
        assert!(!SpeedMultiplier::Normal.requires_warning());
        assert!(!SpeedMultiplier::Fast5x.requires_warning());
        assert!(!SpeedMultiplier::Fast10x.requires_warning());

        // Speeds > 10x MUST require warning (Safety Condition #1)
        assert!(SpeedMultiplier::Fast50x.requires_warning());
        assert!(SpeedMultiplier::Fast100x.requires_warning());
        assert!(SpeedMultiplier::Fast1000x.requires_warning());
    }

    #[test]
    fn test_speed_multiplier_labels() {
        assert_eq!(SpeedMultiplier::Normal.label(), "1x");
        assert_eq!(SpeedMultiplier::Fast5x.label(), "5x");
        assert_eq!(SpeedMultiplier::Fast1000x.label(), "1000x");
    }

    #[test]
    fn test_speed_multiplier_cycling() {
        let speed = SpeedMultiplier::Normal;
        assert_eq!(speed.next(), SpeedMultiplier::Fast5x);
        assert_eq!(speed.next().next(), SpeedMultiplier::Fast10x);
        // Full cycle back to normal
        assert_eq!(SpeedMultiplier::Fast1000x.next(), SpeedMultiplier::Normal);
    }

    #[test]
    fn test_speed_multiplier_from_key() {
        assert_eq!(SpeedMultiplier::from_key(1), Some(SpeedMultiplier::Normal));
        assert_eq!(SpeedMultiplier::from_key(2), Some(SpeedMultiplier::Fast5x));
        assert_eq!(
            SpeedMultiplier::from_key(6),
            Some(SpeedMultiplier::Fast1000x)
        );
        assert_eq!(SpeedMultiplier::from_key(0), None);
        assert_eq!(SpeedMultiplier::from_key(7), None);
    }

    #[test]
    fn test_speed_multiplier_all() {
        let all = SpeedMultiplier::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], SpeedMultiplier::Normal);
        assert_eq!(all[5], SpeedMultiplier::Fast1000x);
    }

    // ==================== GameMode Tests ====================

    #[test]
    fn test_game_mode_default_is_single_player() {
        // SinglePlayer is DEFAULT so users can play immediately with W/S keys
        assert_eq!(GameMode::default(), GameMode::SinglePlayer);
    }

    #[test]
    fn test_game_mode_labels() {
        assert_eq!(GameMode::Demo.label(), "Demo");
        assert_eq!(GameMode::SinglePlayer.label(), "1 Player");
        assert_eq!(GameMode::TwoPlayer.label(), "2 Player");
    }

    #[test]
    fn test_game_mode_short_labels() {
        assert_eq!(GameMode::Demo.short_label(), "Demo");
        assert_eq!(GameMode::SinglePlayer.short_label(), "1P");
        assert_eq!(GameMode::TwoPlayer.short_label(), "2P");
    }

    #[test]
    fn test_game_mode_ai_control() {
        // Demo: both AI
        assert!(GameMode::Demo.left_is_ai());
        assert!(GameMode::Demo.right_is_ai());

        // 1 Player: AI left, Human right (arrow keys)
        // Player controls RIGHT paddle with arrow keys
        assert!(GameMode::SinglePlayer.left_is_ai());
        assert!(!GameMode::SinglePlayer.right_is_ai());

        // 2 Player: both human (P2 left W/S, P1 right arrows)
        assert!(!GameMode::TwoPlayer.left_is_ai());
        assert!(!GameMode::TwoPlayer.right_is_ai());
    }

    #[test]
    fn test_game_mode_cycling() {
        assert_eq!(GameMode::Demo.next(), GameMode::SinglePlayer);
        assert_eq!(GameMode::SinglePlayer.next(), GameMode::TwoPlayer);
        assert_eq!(GameMode::TwoPlayer.next(), GameMode::Demo);
    }

    #[test]
    fn test_game_mode_paddle_labels() {
        // Demo: both AI
        assert_eq!(GameMode::Demo.left_paddle_label(), "AI");
        assert_eq!(GameMode::Demo.right_paddle_label(), "AI");

        // SinglePlayer: AI left, P1 right (human uses arrow keys)
        assert_eq!(GameMode::SinglePlayer.left_paddle_label(), "AI");
        assert_eq!(GameMode::SinglePlayer.right_paddle_label(), "P1 [^/v]");

        // TwoPlayer: P2 left (W/S), P1 right (arrows)
        assert_eq!(GameMode::TwoPlayer.left_paddle_label(), "P2 [W/S]");
        assert_eq!(GameMode::TwoPlayer.right_paddle_label(), "P1 [^/v]");
    }

    // ==================== DemoState Tests ====================

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_demo_state_default_threshold() {
        let state = DemoState::default();
        assert_eq!(state.auto_engage_threshold, 10.0); // 10 seconds per spec
    }

    #[test]
    fn test_demo_state_auto_engage_after_timeout() {
        let mut state = DemoState::default();

        // Simulate 9 seconds - should NOT auto-engage
        let should_engage = state.update(9.0, false);
        assert!(!should_engage);
        assert!(!state.is_auto_engaged());

        // Simulate 1 more second (total 10) - SHOULD auto-engage
        let should_engage = state.update(1.0, false);
        assert!(should_engage);
        assert!(state.is_auto_engaged());
    }

    #[test]
    fn test_demo_state_input_resets_idle() {
        let mut state = DemoState::default();

        // Simulate 8 seconds idle
        let _ = state.update(8.0, false);
        assert!(state.idle_time() > 7.0);

        // User input should reset
        let _ = state.update(0.016, true);
        assert!(state.idle_time() < 0.1);
        assert!(!state.is_auto_engaged());
    }

    #[test]
    fn test_demo_state_difficulty_defaults() {
        let state = DemoState::default();
        assert_eq!(state.left_difficulty(), 7); // Per spec: challenging
        assert_eq!(state.right_difficulty(), 5); // Per spec: slightly easier
    }

    #[test]
    fn test_demo_state_difficulty_cycling() {
        let mut state = DemoState::default();
        let initial_left = state.left_difficulty();
        let initial_right = state.right_difficulty();

        // Simulate 60 seconds
        state.update_difficulty_cycle(60.0);

        // Difficulties should swap
        assert_eq!(state.left_difficulty(), initial_right);
        assert_eq!(state.right_difficulty(), initial_left);
    }

    #[test]
    fn test_demo_state_reset() {
        let mut state = DemoState::default();
        let _ = state.update(15.0, false); // Auto-engage
        assert!(state.is_auto_engaged());

        state.reset();
        assert!(!state.is_auto_engaged());
        assert!(state.idle_time() < 0.1);
    }

    // ==================== PerformanceStats Tests ====================

    #[test]
    fn test_performance_stats_format() {
        let stats = PerformanceStats::new(60000, 60.0, "WASM-SIMD", 1000);
        let display = stats.format_display();

        assert!(display.contains("WASM-SIMD"));
        assert!(display.contains("60000/s"));
        assert!(display.contains("60 FPS"));
    }

    // ==================== Attribution Tests ====================

    #[test]
    fn test_attribution_defaults() {
        let attr = Attribution::default();
        assert!(attr.engine_version.contains("Jugar"));
        assert!(attr.github_url.contains("paiml/jugar"));
        assert!(attr.org_url.contains("paiml.com"));
        assert_eq!(attr.model_filename, "pong-ai-v1.apr");
        assert_eq!(attr.model_size, 491);
    }

    #[test]
    fn test_attribution_labels() {
        let attr = Attribution::default();
        assert!(attr.github_label().contains("github.com"));
        assert!(attr.org_label().contains("paiml.com"));
        assert!(attr.model_label().contains("491 bytes"));
    }

    // ==================== Coverage Gap Tests ====================

    #[test]
    fn test_demo_state_record_input_direct() {
        let mut state = DemoState {
            idle_time: 5.0,
            auto_engaged: true,
            ..Default::default()
        };

        state.record_input();

        assert!(state.idle_time() < 0.001);
        assert!(!state.is_auto_engaged());
    }

    #[test]
    fn test_demo_state_partial_difficulty_cycle() {
        let mut state = DemoState::default();
        let initial_left = state.left_difficulty();
        let initial_right = state.right_difficulty();

        // Update with less than one period (60s default)
        state.update_difficulty_cycle(30.0); // Half period

        // Should NOT swap yet
        assert_eq!(state.left_difficulty(), initial_left);
        assert_eq!(state.right_difficulty(), initial_right);

        // Update with more time to complete the period
        state.update_difficulty_cycle(30.0); // Complete the period

        // Now should swap
        assert_eq!(state.left_difficulty(), initial_right);
        assert_eq!(state.right_difficulty(), initial_left);
    }

    #[test]
    fn test_demo_state_new_accessors() {
        let state = DemoState::new(10.0, 30.0);

        assert!(state.idle_time() < 0.001);
        assert_eq!(state.left_difficulty(), 7); // Per spec: challenging
        assert_eq!(state.right_difficulty(), 5); // Per spec: slightly easier
    }
}
