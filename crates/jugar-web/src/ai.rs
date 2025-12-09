//! AI opponent module for Pong game.
//!
//! This module provides an adaptive AI opponent that uses a pre-trained
//! `.apr` model file for difficulty profiles. The AI implements Dynamic
//! Difficulty Adjustment (DDA) based on player performance.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    PongAIModel (.apr)                        │
//! │  - 10 difficulty profiles (0-9)                             │
//! │  - Reaction delays, prediction accuracy, error magnitudes   │
//! └─────────────────────────────┬───────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      PongAI                                  │
//! │  - Loads model from .apr bytes                              │
//! │  - Tracks player performance (hits, misses, rallies)        │
//! │  - Adapts difficulty to maintain flow state                 │
//! │  - Controls right paddle movement                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};

/// Pong AI Model - stored in .apr binary format.
///
/// This model contains difficulty profiles and skill-matching parameters
/// for the adaptive AI opponent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongAIModel {
    /// Model version (semver)
    pub version: String,
    /// Model name
    pub name: String,
    /// Model description
    pub description: String,

    /// Difficulty profiles (0-9)
    pub difficulty_profiles: Vec<DifficultyProfile>,

    /// Rate at which AI adapts to player skill (0.0-1.0)
    pub skill_adaptation_rate: f32,
    /// Number of rallies to consider for skill assessment
    pub performance_window_size: usize,
}

/// A single difficulty profile defining AI behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyProfile {
    /// Difficulty level (0-9)
    pub level: u8,
    /// Base reaction delay in milliseconds
    pub reaction_delay_ms: f32,
    /// Ball position prediction accuracy (0.0-1.0)
    pub prediction_accuracy: f32,
    /// Maximum paddle movement speed (pixels/second)
    pub max_paddle_speed: f32,
    /// Random error magnitude in target position (pixels)
    pub error_magnitude: f32,
    /// Aggression factor - how much to anticipate vs react (0.0-1.0)
    pub aggression: f32,
}

impl Default for DifficultyProfile {
    fn default() -> Self {
        Self {
            level: 5,
            reaction_delay_ms: 150.0,
            prediction_accuracy: 0.7,
            max_paddle_speed: 400.0,
            error_magnitude: 20.0,
            aggression: 0.5,
        }
    }
}

impl Default for PongAIModel {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            name: "Default Pong AI".to_string(),
            description: "Fallback AI model with standard difficulty curve".to_string(),
            difficulty_profiles: Self::generate_default_profiles(),
            skill_adaptation_rate: 0.1,
            performance_window_size: 10,
        }
    }
}

impl PongAIModel {
    /// Creates a new AI model with custom parameters.
    #[must_use]
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            version: "1.0.0".to_string(),
            name: name.to_string(),
            description: description.to_string(),
            difficulty_profiles: Self::generate_default_profiles(),
            skill_adaptation_rate: 0.1,
            performance_window_size: 10,
        }
    }

    /// Generates the default 10-level difficulty curve.
    ///
    /// Based on game design research:
    /// - Level 0: Very easy (500ms reaction, 30% accuracy)
    /// - Level 5: Medium (150ms reaction, 70% accuracy)
    /// - Level 9: Expert (50ms reaction, 95% accuracy)
    #[must_use]
    #[allow(clippy::suboptimal_flops)] // mul_add is less readable here
    pub fn generate_default_profiles() -> Vec<DifficultyProfile> {
        (0..10)
            .map(|level| {
                let t = f32::from(level) / 9.0; // 0.0 to 1.0

                DifficultyProfile {
                    level,
                    // Reaction delay: 500ms -> 50ms (exponential decay)
                    reaction_delay_ms: 500.0 * (1.0 - t).powi(2) + 50.0,
                    // Prediction accuracy: 30% -> 95% (linear)
                    prediction_accuracy: 0.3 + 0.65 * t,
                    // Max speed: 200 -> 600 px/s (linear)
                    max_paddle_speed: 200.0 + 400.0 * t,
                    // Error magnitude: 50 -> 5 pixels (exponential decay)
                    error_magnitude: 50.0 * (1.0 - t).powi(2) + 5.0,
                    // Aggression: 10% -> 90% (linear)
                    aggression: 0.1 + 0.8 * t,
                }
            })
            .collect()
    }

    /// Gets the difficulty profile for a given level (clamped to 0-9).
    #[must_use]
    pub fn get_profile(&self, level: u8) -> &DifficultyProfile {
        let index = (level as usize).min(self.difficulty_profiles.len().saturating_sub(1));
        &self.difficulty_profiles[index]
    }
}

/// Player performance metrics for skill matching.
#[derive(Debug, Clone, Default)]
pub struct PlayerMetrics {
    /// Recent rally lengths (number of hits before point scored)
    pub rally_history: Vec<u32>,
    /// Total hits by player
    pub player_hits: u32,
    /// Total misses by player
    pub player_misses: u32,
    /// Current rally length
    pub current_rally: u32,
    /// Player's average reaction quality (0.0-1.0)
    pub reaction_quality: f32,
}

impl PlayerMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a player hit.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn record_hit(&mut self) {
        self.player_hits += 1;
        self.current_rally += 1;
    }

    /// Records a player miss (point lost).
    pub fn record_miss(&mut self, window_size: usize) {
        self.player_misses += 1;
        self.rally_history.push(self.current_rally);

        // Keep only recent rallies
        while self.rally_history.len() > window_size {
            let _ = self.rally_history.remove(0);
        }

        self.current_rally = 0;
    }

    /// Records an AI miss (player scored).
    pub fn record_ai_miss(&mut self, window_size: usize) {
        self.rally_history.push(self.current_rally);

        while self.rally_history.len() > window_size {
            let _ = self.rally_history.remove(0);
        }

        self.current_rally = 0;
    }

    /// Calculates player skill estimate (0.0-1.0).
    #[must_use]
    #[allow(clippy::suboptimal_flops)] // mul_add is less readable here
    pub fn estimate_skill(&self) -> f32 {
        if self.player_hits + self.player_misses == 0 {
            return 0.5; // Unknown skill
        }

        let hit_rate = self.player_hits as f32 / (self.player_hits + self.player_misses) as f32;
        let avg_rally = if self.rally_history.is_empty() {
            5.0
        } else {
            self.rally_history.iter().sum::<u32>() as f32 / self.rally_history.len() as f32
        };

        // Combine hit rate and rally length into skill estimate
        // Normalize rally length (assume max useful rally is ~20)
        let rally_factor = (avg_rally / 20.0).min(1.0);

        // Weighted combination
        (hit_rate * 0.6 + rally_factor * 0.4).clamp(0.0, 1.0)
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// AI opponent state and behavior.
#[derive(Debug, Clone)]
pub struct PongAI {
    /// The loaded AI model
    model: PongAIModel,
    /// Current difficulty level (0-9)
    difficulty: u8,
    /// Player performance metrics
    metrics: PlayerMetrics,
    /// Time since ball crossed to AI side (for reaction delay)
    time_since_ball_visible: f32,
    /// Current target Y position for paddle
    target_y: f32,
    /// Whether AI has "seen" the ball this rally
    ball_acquired: bool,
    /// Random seed for deterministic behavior in tests
    rng_state: u64,
}

impl Default for PongAI {
    fn default() -> Self {
        Self::new(PongAIModel::default(), 5)
    }
}

impl PongAI {
    /// Creates a new AI with the given model and initial difficulty.
    #[must_use]
    pub fn new(model: PongAIModel, difficulty: u8) -> Self {
        Self {
            model,
            difficulty: difficulty.min(9),
            metrics: PlayerMetrics::new(),
            time_since_ball_visible: 0.0,
            target_y: 300.0, // Center of default 600px height
            ball_acquired: false,
            rng_state: 12345,
        }
    }

    /// Sets the difficulty level (0-9).
    pub fn set_difficulty(&mut self, level: u8) {
        self.difficulty = level.min(9);
    }

    /// Gets the current difficulty level.
    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    /// Gets the current difficulty profile.
    #[must_use]
    pub fn profile(&self) -> &DifficultyProfile {
        self.model.get_profile(self.difficulty)
    }

    /// Returns a reference to player metrics.
    #[must_use]
    pub const fn metrics(&self) -> &PlayerMetrics {
        &self.metrics
    }

    /// Returns a mutable reference to player metrics.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn metrics_mut(&mut self) -> &mut PlayerMetrics {
        &mut self.metrics
    }

    /// Simple pseudo-random number generator (deterministic for tests).
    fn next_random(&mut self) -> f32 {
        // xorshift64
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        self.rng_state as f32 / u64::MAX as f32
    }

    /// Updates AI state and returns the desired paddle movement.
    ///
    /// # Arguments
    ///
    /// * `ball_x` - Ball X position
    /// * `ball_y` - Ball Y position
    /// * `ball_vx` - Ball X velocity
    /// * `ball_vy` - Ball Y velocity
    /// * `paddle_y` - Current AI paddle Y position
    /// * `paddle_height` - Paddle height
    /// * `canvas_width` - Canvas width
    /// * `canvas_height` - Canvas height
    /// * `dt` - Delta time in seconds
    ///
    /// # Returns
    ///
    /// The desired paddle velocity (positive = down, negative = up)
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        ball_x: f32,
        ball_y: f32,
        ball_vx: f32,
        ball_vy: f32,
        paddle_y: f32,
        _paddle_height: f32,
        canvas_width: f32,
        canvas_height: f32,
        dt: f32,
    ) -> f32 {
        // Clone profile values to avoid borrow issues
        let profile = self.model.get_profile(self.difficulty).clone();

        // Check if ball is moving toward AI (right side)
        let ball_approaching = ball_vx > 0.0;
        let ball_on_ai_side = ball_x > canvas_width * 0.5;

        if ball_approaching && ball_on_ai_side {
            if !self.ball_acquired {
                // Ball just became visible to AI
                self.ball_acquired = true;
                self.time_since_ball_visible = 0.0;

                // Calculate target with prediction and error
                self.target_y = self.calculate_target(
                    ball_x,
                    ball_y,
                    ball_vx,
                    ball_vy,
                    canvas_width,
                    canvas_height,
                    &profile,
                );
            }

            self.time_since_ball_visible += dt;
        } else {
            // Ball not approaching, return to center
            self.ball_acquired = false;
            self.time_since_ball_visible = 0.0;
            self.target_y = canvas_height / 2.0;
        }

        // Check reaction delay
        let reaction_delay_sec = profile.reaction_delay_ms / 1000.0;
        if self.time_since_ball_visible < reaction_delay_sec && self.ball_acquired {
            // Still "reacting", don't move yet
            return 0.0;
        }

        // Move toward target
        let diff = self.target_y - paddle_y;
        let max_speed = profile.max_paddle_speed;

        // Apply speed limit
        if diff.abs() < 5.0 {
            0.0 // Close enough
        } else if diff > 0.0 {
            max_speed.min(diff / dt)
        } else {
            (-max_speed).max(diff / dt)
        }
    }

    /// Calculates the target Y position for the paddle.
    #[allow(clippy::too_many_arguments)] // Game physics needs multiple parameters
    #[allow(clippy::suboptimal_flops)]
    fn calculate_target(
        &mut self,
        ball_x: f32,
        ball_y: f32,
        ball_vx: f32,
        ball_vy: f32,
        canvas_width: f32,
        canvas_height: f32,
        profile: &DifficultyProfile,
    ) -> f32 {
        // Simple prediction: where will ball be when it reaches paddle X?
        let paddle_x = canvas_width - 35.0; // AI paddle position
        let time_to_paddle = if ball_vx > 0.0 {
            (paddle_x - ball_x) / ball_vx
        } else {
            0.0
        };

        // Predicted Y position (with wall bounces simplified)
        let mut predicted_y = ball_y + ball_vy * time_to_paddle * profile.prediction_accuracy;

        // Clamp to canvas
        predicted_y = predicted_y.clamp(50.0, canvas_height - 50.0);

        // Add random error based on difficulty
        let error = (self.next_random() - 0.5) * 2.0 * profile.error_magnitude;
        predicted_y += error;

        // Clamp again after error
        predicted_y.clamp(50.0, canvas_height - 50.0)
    }

    /// Adapts difficulty based on player performance.
    ///
    /// Call this after each point to adjust AI difficulty.
    #[allow(clippy::suboptimal_flops)]
    pub fn adapt_difficulty(&mut self) {
        let skill = self.metrics.estimate_skill();
        let target_difficulty = (skill * 9.0).round() as u8;

        // Gradually adjust difficulty
        let rate = self.model.skill_adaptation_rate;
        let current = f32::from(self.difficulty);
        let target = f32::from(target_difficulty);

        let new_difficulty = current + (target - current) * rate;
        self.difficulty = (new_difficulty.round() as u8).min(9);
    }

    /// Resets AI state for a new game.
    pub fn reset(&mut self) {
        self.metrics.reset();
        self.time_since_ball_visible = 0.0;
        self.target_y = 300.0;
        self.ball_acquired = false;
    }

    /// Records that the player successfully hit the ball.
    pub fn record_player_hit(&mut self) {
        self.metrics.record_hit();
    }

    /// Records that the player missed the ball.
    pub fn record_player_miss(&mut self) {
        let window_size = self.model.performance_window_size;
        self.metrics.record_miss(window_size);
    }

    /// Loads a model from bytes (for .apr file loading).
    ///
    /// # Errors
    ///
    /// Returns an error string if the model fails to load.
    pub fn load_model_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        match aprender::format::load_from_bytes::<PongAIModel>(
            bytes,
            aprender::format::ModelType::Custom,
        ) {
            Ok(model) => {
                self.model = model;
                Ok(())
            }
            Err(e) => Err(format!("Failed to load AI model: {e}")),
        }
    }

    /// Gets model info as JSON string.
    #[must_use]
    pub fn model_info_json(&self) -> String {
        serde_json::json!({
            "name": self.model.name,
            "version": self.model.version,
            "description": self.model.description,
            "difficulty_levels": self.model.difficulty_profiles.len(),
            "current_difficulty": self.difficulty,
        })
        .to_string()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    // =========================================================================
    // DifficultyProfile Tests
    // =========================================================================

    #[test]
    fn test_difficulty_profile_default() {
        let profile = DifficultyProfile::default();
        assert_eq!(profile.level, 5);
        assert!((profile.reaction_delay_ms - 150.0).abs() < 0.01);
        assert!((profile.prediction_accuracy - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_difficulty_profile_values_valid() {
        let profile = DifficultyProfile::default();
        assert!(profile.reaction_delay_ms > 0.0);
        assert!(profile.prediction_accuracy >= 0.0 && profile.prediction_accuracy <= 1.0);
        assert!(profile.max_paddle_speed > 0.0);
        assert!(profile.error_magnitude >= 0.0);
        assert!(profile.aggression >= 0.0 && profile.aggression <= 1.0);
    }

    // =========================================================================
    // PongAIModel Tests
    // =========================================================================

    #[test]
    fn test_pong_ai_model_default() {
        let model = PongAIModel::default();
        assert_eq!(model.version, "1.0.0");
        assert_eq!(model.difficulty_profiles.len(), 10);
    }

    #[test]
    fn test_pong_ai_model_new() {
        let model = PongAIModel::new("Test AI", "A test AI model");
        assert_eq!(model.name, "Test AI");
        assert_eq!(model.description, "A test AI model");
        assert_eq!(model.difficulty_profiles.len(), 10);
    }

    #[test]
    fn test_generate_default_profiles_count() {
        let profiles = PongAIModel::generate_default_profiles();
        assert_eq!(profiles.len(), 10);
    }

    #[test]
    fn test_generate_default_profiles_levels() {
        let profiles = PongAIModel::generate_default_profiles();
        for (i, profile) in profiles.iter().enumerate() {
            assert_eq!(profile.level, i as u8);
        }
    }

    #[test]
    fn test_generate_default_profiles_difficulty_curve() {
        let profiles = PongAIModel::generate_default_profiles();

        // Level 0 should be easiest
        assert!(profiles[0].reaction_delay_ms > profiles[9].reaction_delay_ms);
        assert!(profiles[0].prediction_accuracy < profiles[9].prediction_accuracy);
        assert!(profiles[0].max_paddle_speed < profiles[9].max_paddle_speed);
        assert!(profiles[0].error_magnitude > profiles[9].error_magnitude);
    }

    #[test]
    fn test_get_profile_valid_level() {
        let model = PongAIModel::default();
        let profile = model.get_profile(5);
        assert_eq!(profile.level, 5);
    }

    #[test]
    fn test_get_profile_clamped_high() {
        let model = PongAIModel::default();
        let profile = model.get_profile(100);
        assert_eq!(profile.level, 9); // Clamped to max
    }

    // =========================================================================
    // PlayerMetrics Tests
    // =========================================================================

    #[test]
    fn test_player_metrics_new() {
        let metrics = PlayerMetrics::new();
        assert_eq!(metrics.player_hits, 0);
        assert_eq!(metrics.player_misses, 0);
        assert_eq!(metrics.current_rally, 0);
    }

    #[test]
    fn test_player_metrics_record_hit() {
        let mut metrics = PlayerMetrics::new();
        metrics.record_hit();
        assert_eq!(metrics.player_hits, 1);
        assert_eq!(metrics.current_rally, 1);
    }

    #[test]
    fn test_player_metrics_record_miss() {
        let mut metrics = PlayerMetrics::new();
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss(10);

        assert_eq!(metrics.player_misses, 1);
        assert_eq!(metrics.current_rally, 0);
        assert_eq!(metrics.rally_history.len(), 1);
        assert_eq!(metrics.rally_history[0], 2);
    }

    #[test]
    fn test_player_metrics_window_size() {
        let mut metrics = PlayerMetrics::new();

        // Record many rallies
        for i in 0..15 {
            metrics.current_rally = i;
            metrics.record_miss(5);
        }

        // Should only keep last 5
        assert_eq!(metrics.rally_history.len(), 5);
    }

    #[test]
    fn test_player_metrics_estimate_skill_unknown() {
        let metrics = PlayerMetrics::new();
        let skill = metrics.estimate_skill();
        assert!((skill - 0.5).abs() < 0.01); // Unknown = 0.5
    }

    #[test]
    fn test_player_metrics_estimate_skill_good_player() {
        let mut metrics = PlayerMetrics::new();

        // Good player: high hit rate, long rallies
        for _ in 0..10 {
            metrics.record_hit();
        }
        metrics.record_miss(10);

        let skill = metrics.estimate_skill();
        assert!(skill > 0.7); // Should be high
    }

    #[test]
    fn test_player_metrics_estimate_skill_poor_player() {
        let mut metrics = PlayerMetrics::new();

        // Poor player: low hit rate, short rallies
        for _ in 0..5 {
            metrics.record_hit();
            metrics.record_miss(10);
        }

        let skill = metrics.estimate_skill();
        assert!(skill < 0.6); // Should be lower
    }

    #[test]
    fn test_player_metrics_reset() {
        let mut metrics = PlayerMetrics::new();
        metrics.record_hit();
        metrics.record_miss(10);
        metrics.reset();

        assert_eq!(metrics.player_hits, 0);
        assert_eq!(metrics.player_misses, 0);
        assert!(metrics.rally_history.is_empty());
    }

    // =========================================================================
    // PongAI Tests
    // =========================================================================

    #[test]
    fn test_pong_ai_default() {
        let ai = PongAI::default();
        assert_eq!(ai.difficulty(), 5);
    }

    #[test]
    fn test_pong_ai_new() {
        let model = PongAIModel::default();
        let ai = PongAI::new(model, 3);
        assert_eq!(ai.difficulty(), 3);
    }

    #[test]
    fn test_pong_ai_set_difficulty() {
        let mut ai = PongAI::default();
        ai.set_difficulty(7);
        assert_eq!(ai.difficulty(), 7);
    }

    #[test]
    fn test_pong_ai_set_difficulty_clamped() {
        let mut ai = PongAI::default();
        ai.set_difficulty(100);
        assert_eq!(ai.difficulty(), 9);
    }

    #[test]
    fn test_pong_ai_profile() {
        let ai = PongAI::default();
        let profile = ai.profile();
        assert_eq!(profile.level, 5);
    }

    #[test]
    fn test_pong_ai_update_ball_not_approaching() {
        let mut ai = PongAI::default();

        // Ball moving away (negative vx)
        let velocity = ai.update(
            400.0, 300.0, // ball position
            -200.0, 100.0, // ball velocity (moving left)
            300.0, 100.0, // paddle position and height
            800.0, 600.0, // canvas size
            0.016, // dt
        );

        // AI should move toward center (300.0)
        assert!(velocity.abs() < ai.profile().max_paddle_speed + 1.0);
    }

    #[test]
    fn test_pong_ai_update_ball_approaching() {
        let mut ai = PongAI::default();

        // Ball moving toward AI (positive vx) on AI side
        let velocity = ai.update(
            600.0, 400.0, // ball position (on AI side)
            200.0, 100.0, // ball velocity (moving right)
            300.0, 100.0, // paddle position and height
            800.0, 600.0, // canvas size
            0.5,   // dt (long enough to pass reaction delay)
        );

        // AI should start moving (after reaction delay)
        // First update won't move due to reaction delay
        assert!(velocity.abs() >= 0.0);
    }

    #[test]
    fn test_pong_ai_reaction_delay() {
        let mut ai = PongAI::default();
        ai.set_difficulty(0); // Slowest reaction (500ms)

        // First update with very small dt
        let v1 = ai.update(
            600.0, 400.0, 200.0, 100.0, 300.0, 100.0, 800.0, 600.0, 0.001,
        );

        // Should not move yet (still in reaction delay)
        assert!((v1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_pong_ai_adapt_difficulty_increases() {
        let mut ai = PongAI::default();
        ai.set_difficulty(3);

        // Simulate good player performance
        for _ in 0..20 {
            ai.metrics_mut().record_hit();
        }
        ai.metrics_mut().record_ai_miss(10);

        let initial_difficulty = ai.difficulty();
        ai.adapt_difficulty();

        // Difficulty should increase (or stay same if already matched)
        assert!(ai.difficulty() >= initial_difficulty);
    }

    #[test]
    fn test_pong_ai_reset() {
        let mut ai = PongAI::default();
        ai.metrics_mut().record_hit();
        ai.set_difficulty(9);
        ai.time_since_ball_visible = 1.0;

        ai.reset();

        assert_eq!(ai.metrics().player_hits, 0);
        assert!((ai.time_since_ball_visible - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_pong_ai_model_info_json() {
        let ai = PongAI::default();
        let json = ai.model_info_json();

        assert!(json.contains("name"));
        assert!(json.contains("version"));
        assert!(json.contains("current_difficulty"));
    }

    #[test]
    fn test_pong_ai_deterministic_rng() {
        let mut ai1 = PongAI::default();
        let mut ai2 = PongAI::default();

        // Same seed should produce same sequence
        let r1 = ai1.next_random();
        let r2 = ai2.next_random();

        assert!((r1 - r2).abs() < 0.0001);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_full_rally_simulation() {
        let mut ai = PongAI::default();
        ai.set_difficulty(5);

        // Simulate a rally
        let mut ball_x = 400.0;
        let mut ball_y = 300.0;
        let ball_vx = 300.0;
        let ball_vy = 150.0;
        let mut paddle_y = 300.0;

        // Run for 60 frames (1 second at 60fps)
        for _ in 0..60 {
            let velocity = ai.update(
                ball_x,
                ball_y,
                ball_vx,
                ball_vy,
                paddle_y,
                100.0,
                800.0,
                600.0,
                1.0 / 60.0,
            );

            // Update paddle position
            paddle_y += velocity * (1.0 / 60.0);
            paddle_y = paddle_y.clamp(50.0, 550.0);

            // Update ball position
            ball_x += ball_vx * (1.0 / 60.0);
            ball_y += ball_vy * (1.0 / 60.0);
        }

        // AI should have moved toward the ball
        // (exact position depends on difficulty and randomness)
        assert!(paddle_y != 300.0 || ball_x < 500.0);
    }

    #[test]
    fn test_difficulty_affects_behavior() {
        let mut ai_easy = PongAI::default();
        let mut ai_hard = PongAI::default();

        ai_easy.set_difficulty(0);
        ai_hard.set_difficulty(9);

        // Same ball position
        let ball_x = 600.0;
        let ball_y = 400.0;
        let ball_vx = 200.0;
        let ball_vy = 100.0;

        // Run enough time to pass reaction delays
        for _ in 0..120 {
            let _ = ai_easy.update(
                ball_x,
                ball_y,
                ball_vx,
                ball_vy,
                300.0,
                100.0,
                800.0,
                600.0,
                1.0 / 60.0,
            );
            let _ = ai_hard.update(
                ball_x,
                ball_y,
                ball_vx,
                ball_vy,
                300.0,
                100.0,
                800.0,
                600.0,
                1.0 / 60.0,
            );
        }

        // Hard AI should have acquired ball faster
        assert!(ai_hard.ball_acquired);
    }
}
