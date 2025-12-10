//! AI opponent module for Pong game.
//!
//! This module provides an adaptive AI opponent that uses a pre-trained
//! `.apr` model file for difficulty profiles. The AI implements Dynamic
//! Difficulty Adjustment (DDA) based on Flow Theory (Csikszentmihalyi).
//!
//! ## Research Foundation
//!
//! - **Flow Theory**: Three-channel model (boredom ↔ flow ↔ anxiety)
//! - **DDA**: [Zohaib 2018](https://onlinelibrary.wiley.com/doi/10.1155/2018/5681652)
//! - **Reproducibility**: Deterministic RNG with seed stored in model
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    PongAIModel (.apr)                        │
//! │  - Metadata (name, version, author, license)                │
//! │  - Determinism config (seed, algorithm)                     │
//! │  - Flow Theory params (thresholds, adaptation rate)         │
//! │  - 10 difficulty profiles (0-9)                             │
//! └─────────────────────────────┬───────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      PongAI                                  │
//! │  - Loads model from .apr bytes                              │
//! │  - Tracks player performance (FlowState)                    │
//! │  - Detects flow channel (boredom/flow/anxiety)              │
//! │  - Adapts difficulty to maintain flow state                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Model Metadata (for .apr showcase)
// ============================================================================

/// Model metadata for the `.apr` file format.
///
/// This showcases aprender's portable model format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Model description
    pub description: String,
    /// Author/organization
    pub author: String,
    /// License (e.g., "MIT")
    pub license: String,
    /// Creation timestamp (ISO 8601)
    pub created: String,
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            name: "Pong AI v1".to_string(),
            version: "1.0.0".to_string(),
            description: "Flow Theory-based adaptive Pong opponent".to_string(),
            author: "PAIML".to_string(),
            license: "MIT".to_string(),
            created: "2025-01-01T00:00:00Z".to_string(),
        }
    }
}

// ============================================================================
// Determinism Configuration (for reproducibility)
// ============================================================================

/// Determinism configuration for reproducible AI behavior.
///
/// Based on [arXiv Reproducibility (2022)](https://arxiv.org/abs/2203.01075).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Random seed for reproducibility
    pub seed: u64,
    /// RNG algorithm name
    pub rng_algorithm: String,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            rng_algorithm: "xorshift64".to_string(),
        }
    }
}

// ============================================================================
// Flow Theory Configuration
// ============================================================================

/// Flow Theory parameters for Dynamic Difficulty Adjustment.
///
/// Based on Csikszentmihalyi's three-channel model:
/// - **Boredom**: Challenge too low for skill level
/// - **Flow**: Challenge matches skill (optimal engagement)
/// - **Anxiety**: Challenge too high for skill level
///
/// Reference: [Think Game Design](https://thinkgamedesign.com/flow-theory-game-design/)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowTheoryConfig {
    /// Number of recent points to consider for skill estimation
    pub skill_window_size: usize,
    /// Rate at which AI adapts difficulty (0.0-1.0)
    pub adaptation_rate: f32,
    /// Win rate threshold above which player is "bored" (e.g., 0.7)
    pub boredom_threshold: f32,
    /// Win rate threshold below which player is "anxious" (e.g., 0.3)
    pub anxiety_threshold: f32,
    /// Target win rate for optimal flow (typically 0.5)
    pub target_win_rate: f32,
}

impl Default for FlowTheoryConfig {
    fn default() -> Self {
        Self {
            skill_window_size: 10,
            adaptation_rate: 0.15,
            boredom_threshold: 0.7,
            anxiety_threshold: 0.3,
            target_win_rate: 0.5,
        }
    }
}

// ============================================================================
// Difficulty Profile
// ============================================================================

/// A single difficulty profile defining AI behavior.
///
/// Difficulty curve formula (from design spec):
/// - `reaction = 500 * (1-t)² + 50` where `t = level/9`
/// - `accuracy = 0.30 + 0.65 * t`
/// - `speed = 200 + 400 * t`
/// - `error = 50 * (1-t)² + 5`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyProfile {
    /// Difficulty level (0-9)
    pub level: u8,
    /// Human-readable name for this level
    pub name: String,
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
            name: "Challenging".to_string(),
            reaction_delay_ms: 180.0,
            prediction_accuracy: 0.66,
            max_paddle_speed: 422.0,
            error_magnitude: 24.0,
            aggression: 0.55,
        }
    }
}

// ============================================================================
// Pong AI Model (.apr format)
// ============================================================================

/// Pong AI Model - the complete `.apr` file structure.
///
/// This is the downloadable model that showcases aprender's format.
/// Users can inspect, modify, and reload this JSON model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongAIModel {
    /// Schema version for forward compatibility
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,

    /// Model metadata
    pub metadata: ModelMetadata,

    /// Model type identifier
    pub model_type: String,

    /// Determinism configuration for reproducibility
    pub determinism: DeterminismConfig,

    /// Flow Theory parameters for DDA
    pub flow_theory: FlowTheoryConfig,

    /// Difficulty profiles (0-9)
    pub difficulty_profiles: Vec<DifficultyProfile>,
}

fn default_schema() -> String {
    "https://paiml.com/schemas/apr/v1".to_string()
}

impl Default for PongAIModel {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            metadata: ModelMetadata::default(),
            model_type: "behavior_profile".to_string(),
            determinism: DeterminismConfig::default(),
            flow_theory: FlowTheoryConfig::default(),
            difficulty_profiles: Self::generate_default_profiles(),
        }
    }
}

impl PongAIModel {
    /// Creates a new AI model with custom metadata.
    #[must_use]
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            metadata: ModelMetadata {
                name: name.to_string(),
                description: description.to_string(),
                ..ModelMetadata::default()
            },
            ..Default::default()
        }
    }

    /// Generates the default 10-level difficulty curve.
    ///
    /// Based on game design research:
    /// - Level 0: "Training wheels" (500ms reaction, 30% accuracy)
    /// - Level 5: "Challenging" (180ms reaction, 66% accuracy)
    /// - Level 9: "Perfect" (0ms reaction, 100% accuracy, 0 error - UNBEATABLE)
    #[must_use]
    #[allow(clippy::suboptimal_flops)] // mul_add is less readable here
    pub fn generate_default_profiles() -> Vec<DifficultyProfile> {
        const LEVEL_NAMES: [&str; 10] = [
            "Training Wheels",
            "Beginner",
            "Easy",
            "Casual",
            "Normal",
            "Challenging",
            "Hard",
            "Very Hard",
            "Expert",
            "Perfect",
        ];

        (0..10)
            .map(|level| {
                let t = f32::from(level) / 9.0; // 0.0 to 1.0

                // Level 9 is PERFECT - unbeatable AI
                if level == 9 {
                    return DifficultyProfile {
                        level,
                        name: LEVEL_NAMES[level as usize].to_string(),
                        reaction_delay_ms: 0.0,   // Instant reaction
                        prediction_accuracy: 1.0, // Perfect prediction
                        max_paddle_speed: 1000.0, // Very fast movement
                        error_magnitude: 0.0,     // Zero error
                        aggression: 1.0,          // Maximum aggression
                    };
                }

                DifficultyProfile {
                    level,
                    name: LEVEL_NAMES[level as usize].to_string(),
                    // Reaction delay: 500ms -> 50ms (exponential decay)
                    reaction_delay_ms: 500.0 * (1.0 - t).powi(2) + 50.0,
                    // Prediction accuracy: 30% -> 95% (linear, except level 9)
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

    /// Exports the model as a pretty-printed JSON string (APR format).
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Exports the model as compact JSON (for size measurement).
    #[must_use]
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Returns the approximate size of the model when serialized.
    #[must_use]
    pub fn serialized_size(&self) -> usize {
        self.to_json_compact().len()
    }

    /// Loads a model from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed to parse model: {e}"))
    }
}

// ============================================================================
// Flow State (Player Engagement Tracking)
// ============================================================================

/// Flow state based on Csikszentmihalyi's three-channel model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowChannel {
    /// Player winning too easily - increase difficulty
    Boredom,
    /// Player optimally challenged - maintain difficulty
    #[default]
    Flow,
    /// Player struggling - decrease difficulty
    Anxiety,
}

impl FlowChannel {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Boredom => "Bored (too easy)",
            Self::Flow => "In Flow (optimal)",
            Self::Anxiety => "Anxious (too hard)",
        }
    }
}

/// Player performance metrics for flow state detection.
#[derive(Debug, Clone, Default)]
pub struct PlayerMetrics {
    /// Recent point outcomes (true = player won)
    pub point_history: Vec<bool>,
    /// Total points won by player
    pub player_points: u32,
    /// Total points won by AI
    pub ai_points: u32,
    /// Current rally length
    pub current_rally: u32,
    /// Rally lengths history
    pub rally_history: Vec<u32>,
    /// Total player paddle hits
    pub player_hits: u32,
    /// Total player misses
    pub player_misses: u32,
}

impl PlayerMetrics {
    /// Creates new empty metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a player hit (successfully returned the ball).
    #[allow(clippy::missing_const_for_fn)]
    pub fn record_hit(&mut self) {
        self.player_hits += 1;
        self.current_rally += 1;
    }

    /// Records that the player scored a point.
    pub fn record_player_scored(&mut self, window_size: usize) {
        self.player_points += 1;
        self.point_history.push(true);
        self.finalize_rally(window_size);
    }

    /// Records that the AI scored a point (player missed).
    pub fn record_ai_scored(&mut self, window_size: usize) {
        self.ai_points += 1;
        self.player_misses += 1;
        self.point_history.push(false);
        self.finalize_rally(window_size);
    }

    /// Finalizes current rally and trims history.
    fn finalize_rally(&mut self, window_size: usize) {
        self.rally_history.push(self.current_rally);
        self.current_rally = 0;

        // Trim histories to window size
        while self.point_history.len() > window_size {
            let _ = self.point_history.remove(0);
        }
        while self.rally_history.len() > window_size {
            let _ = self.rally_history.remove(0);
        }
    }

    /// Calculates player win rate from recent history.
    #[must_use]
    pub fn recent_win_rate(&self) -> f32 {
        if self.point_history.is_empty() {
            return 0.5; // Unknown, assume balanced
        }
        let wins = self.point_history.iter().filter(|&&w| w).count() as f32;
        wins / self.point_history.len() as f32
    }

    /// Calculates average rally length.
    #[must_use]
    pub fn average_rally(&self) -> f32 {
        if self.rally_history.is_empty() {
            return 5.0; // Default assumption
        }
        self.rally_history.iter().sum::<u32>() as f32 / self.rally_history.len() as f32
    }

    /// Estimates player skill (0.0-1.0).
    #[must_use]
    #[allow(clippy::suboptimal_flops)]
    pub fn estimate_skill(&self) -> f32 {
        if self.player_hits + self.player_misses == 0 {
            return 0.5; // Unknown
        }

        let hit_rate = self.player_hits as f32 / (self.player_hits + self.player_misses) as f32;
        let rally_factor = (self.average_rally() / 20.0).min(1.0);

        // Weighted combination
        (hit_rate * 0.6 + rally_factor * 0.4).clamp(0.0, 1.0)
    }

    /// Detects the current flow channel based on recent performance.
    #[must_use]
    pub fn detect_flow_channel(&self, config: &FlowTheoryConfig) -> FlowChannel {
        let win_rate = self.recent_win_rate();

        if win_rate >= config.boredom_threshold {
            FlowChannel::Boredom
        } else if win_rate <= config.anxiety_threshold {
            FlowChannel::Anxiety
        } else {
            FlowChannel::Flow
        }
    }

    /// Resets all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ============================================================================
// SHAP-style Decision Explanation
// ============================================================================

/// AI decision state for explainability visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DecisionState {
    /// Ball moving away from AI - returning to center
    #[default]
    Idle,
    /// Ball approaching but within reaction delay window
    Reacting,
    /// Actively tracking and moving toward predicted position
    Tracking,
    /// At target position, waiting for ball
    Ready,
}

impl DecisionState {
    /// Returns a human-readable label for the decision state.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle (centering)",
            Self::Reacting => "Reacting (delay)",
            Self::Tracking => "Tracking ball",
            Self::Ready => "Ready (at target)",
        }
    }

    /// Returns a short code for compact display.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Reacting => "REACT",
            Self::Tracking => "TRACK",
            Self::Ready => "READY",
        }
    }
}

/// SHAP-style feature contribution for AI decision explainability.
///
/// Based on Lundberg & Lee (2017) SHAP values - shows how each input
/// feature contributed to the final decision (paddle velocity).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureContribution {
    /// Feature name
    pub name: String,
    /// Raw feature value
    pub value: f32,
    /// Contribution to output (positive = move down, negative = move up)
    pub contribution: f32,
    /// Normalized importance (0.0 - 1.0) for bar visualization
    pub importance: f32,
}

/// Complete AI decision explanation for real-time visualization.
///
/// This provides SHAP-style explainability for the AI's decisions,
/// showing which factors from the `.apr` model influenced the move.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionExplanation {
    /// Current decision state
    pub state: DecisionState,
    /// Output velocity (positive = down, negative = up)
    pub output_velocity: f32,
    /// Target Y position the AI is moving toward
    pub target_y: f32,
    /// Current paddle Y position
    pub paddle_y: f32,
    /// Distance to target
    pub distance_to_target: f32,

    // === .apr Model Parameters (from profile) ===
    /// Current difficulty level (0-9)
    pub difficulty_level: u8,
    /// Difficulty name from model
    pub difficulty_name: String,
    /// Reaction delay from model (ms)
    pub reaction_delay_ms: f32,
    /// Time spent reacting so far (ms)
    pub reaction_elapsed_ms: f32,
    /// Prediction accuracy from model (0-1)
    pub prediction_accuracy: f32,
    /// Max paddle speed from model (px/s)
    pub max_paddle_speed: f32,
    /// Error magnitude from model (px)
    pub error_magnitude: f32,
    /// Applied error this frame (px)
    pub applied_error: f32,

    // === Input Features ===
    /// Ball X position (0 = left, canvas_width = right)
    pub ball_x: f32,
    /// Ball Y position
    pub ball_y: f32,
    /// Ball X velocity (positive = moving right toward AI)
    pub ball_vx: f32,
    /// Ball Y velocity
    pub ball_vy: f32,
    /// Is ball approaching AI side?
    pub ball_approaching: bool,
    /// Is ball on AI's half of court?
    pub ball_on_ai_side: bool,

    // === SHAP-style Feature Contributions ===
    /// Ordered list of feature contributions (highest importance first)
    pub contributions: Vec<FeatureContribution>,

    // === Decision Rationale ===
    /// Human-readable explanation of the decision
    pub rationale: String,
}

impl DecisionExplanation {
    /// Creates a new empty explanation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Exports explanation as JSON for the widget.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Computes SHAP-style feature contributions based on current state.
    pub fn compute_contributions(&mut self) {
        self.contributions.clear();

        // Compute absolute contributions for normalization
        let mut raw_contributions = vec![
            (
                "Ball Direction",
                self.ball_vx,
                if self.ball_approaching { 1.0 } else { -1.0 },
            ),
            (
                "Reaction Delay",
                self.reaction_delay_ms,
                if self.state == DecisionState::Reacting {
                    1.0
                } else {
                    0.0
                },
            ),
            (
                "Distance to Target",
                self.distance_to_target,
                self.distance_to_target.abs() / 300.0,
            ),
            (
                "Prediction Acc",
                self.prediction_accuracy,
                self.prediction_accuracy,
            ),
            (
                "Max Speed",
                self.max_paddle_speed,
                self.max_paddle_speed / 1000.0,
            ),
            (
                "Applied Error",
                self.applied_error,
                self.applied_error.abs() / 50.0,
            ),
        ];

        // Find max for normalization
        let max_contrib = raw_contributions
            .iter()
            .map(|(_, _, c)| c.abs())
            .fold(0.0f32, f32::max)
            .max(0.001);

        // Sort by absolute contribution
        raw_contributions.sort_by(|a, b| {
            b.2.abs()
                .partial_cmp(&a.2.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Convert to FeatureContribution
        for (name, value, contrib) in raw_contributions {
            self.contributions.push(FeatureContribution {
                name: name.to_string(),
                value,
                contribution: contrib,
                importance: (contrib.abs() / max_contrib).min(1.0),
            });
        }
    }

    /// Generates human-readable rationale for the decision.
    pub fn generate_rationale(&mut self) {
        self.rationale = match self.state {
            DecisionState::Idle => {
                format!(
                    "Ball moving away (vx={:.0}). Returning to center.",
                    self.ball_vx
                )
            }
            DecisionState::Reacting => {
                let remaining = (self.reaction_delay_ms - self.reaction_elapsed_ms).max(0.0);
                format!(
                    "Reaction delay: {:.0}ms remaining of {:.0}ms (Level {} profile)",
                    remaining, self.reaction_delay_ms, self.difficulty_level
                )
            }
            DecisionState::Tracking => {
                let direction = if self.output_velocity > 0.0 {
                    "DOWN"
                } else {
                    "UP"
                };
                format!(
                    "Tracking ball → target Y={:.0} (acc={:.0}%, speed={:.0}px/s). Moving {}.",
                    self.target_y,
                    self.prediction_accuracy * 100.0,
                    self.max_paddle_speed,
                    direction
                )
            }
            DecisionState::Ready => {
                format!(
                    "At target Y={:.0} (within 5px). Waiting for ball.",
                    self.target_y
                )
            }
        };
    }
}

// ============================================================================
// Pong AI (Runtime State)
// ============================================================================

/// AI opponent with Flow Theory-based Dynamic Difficulty Adjustment.
#[derive(Debug, Clone)]
pub struct PongAI {
    /// The loaded AI model
    model: PongAIModel,
    /// Current difficulty level (0-9)
    difficulty: u8,
    /// Player performance metrics
    metrics: PlayerMetrics,
    /// Current flow channel
    flow_channel: FlowChannel,
    /// Time since ball crossed to AI side (for reaction delay)
    time_since_ball_visible: f32,
    /// Current target Y position for paddle
    target_y: f32,
    /// Whether AI has "seen" the ball this rally
    ball_acquired: bool,
    /// Deterministic RNG state
    rng_state: u64,
    /// Last applied error for explainability
    last_error: f32,
    /// Last decision explanation for SHAP-style widget
    last_explanation: DecisionExplanation,
}

impl Default for PongAI {
    fn default() -> Self {
        let model = PongAIModel::default();
        let seed = model.determinism.seed;
        Self::new(model, 5, seed)
    }
}

impl PongAI {
    /// Creates a new AI with the given model, initial difficulty, and seed.
    #[must_use]
    pub fn new(model: PongAIModel, difficulty: u8, seed: u64) -> Self {
        Self {
            model,
            difficulty: difficulty.min(9),
            metrics: PlayerMetrics::new(),
            flow_channel: FlowChannel::Flow,
            time_since_ball_visible: 0.0,
            target_y: 300.0,
            ball_acquired: false,
            rng_state: seed,
            last_error: 0.0,
            last_explanation: DecisionExplanation::new(),
        }
    }

    /// Creates AI with default model and specified difficulty.
    #[must_use]
    pub fn with_difficulty(difficulty: u8) -> Self {
        let model = PongAIModel::default();
        let seed = model.determinism.seed;
        Self::new(model, difficulty, seed)
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

    /// Gets the current difficulty name.
    #[must_use]
    pub fn difficulty_name(&self) -> &str {
        &self.model.get_profile(self.difficulty).name
    }

    /// Gets the current difficulty profile.
    #[must_use]
    pub fn profile(&self) -> &DifficultyProfile {
        self.model.get_profile(self.difficulty)
    }

    /// Gets the current flow channel.
    #[must_use]
    pub const fn flow_channel(&self) -> FlowChannel {
        self.flow_channel
    }

    /// Returns a reference to player metrics.
    #[must_use]
    pub const fn metrics(&self) -> &PlayerMetrics {
        &self.metrics
    }

    /// Returns a mutable reference to player metrics.
    #[allow(clippy::missing_const_for_fn)]
    pub fn metrics_mut(&mut self) -> &mut PlayerMetrics {
        &mut self.metrics
    }

    /// Returns the underlying model.
    #[must_use]
    pub const fn model(&self) -> &PongAIModel {
        &self.model
    }

    /// Exports the model as JSON for download.
    #[must_use]
    pub fn export_model(&self) -> String {
        self.model.to_json()
    }

    /// Returns the last decision explanation for SHAP-style visualization.
    #[must_use]
    pub fn explanation(&self) -> &DecisionExplanation {
        &self.last_explanation
    }

    /// Exports the last decision explanation as JSON for the widget.
    #[must_use]
    pub fn export_explanation(&self) -> String {
        self.last_explanation.to_json()
    }

    /// Deterministic pseudo-random number generator (xorshift64).
    fn next_random(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        self.rng_state as f32 / u64::MAX as f32
    }

    /// Updates AI state and returns the desired paddle velocity.
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
        let profile = self.model.get_profile(self.difficulty).clone();

        // Check if ball is moving toward AI (right side)
        let ball_approaching = ball_vx > 0.0;
        let ball_on_ai_side = ball_x > canvas_width * 0.5;

        if ball_approaching && ball_on_ai_side {
            if !self.ball_acquired {
                self.ball_acquired = true;
                self.time_since_ball_visible = 0.0;

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
            self.ball_acquired = false;
            self.time_since_ball_visible = 0.0;
            self.target_y = canvas_height / 2.0;
        }

        // Check reaction delay
        let reaction_delay_sec = profile.reaction_delay_ms / 1000.0;
        let in_reaction_delay =
            self.time_since_ball_visible < reaction_delay_sec && self.ball_acquired;

        // Move toward target
        let diff = self.target_y - paddle_y;
        let max_speed = profile.max_paddle_speed;

        let output_velocity = if in_reaction_delay || diff.abs() < 5.0 {
            0.0
        } else if diff > 0.0 {
            max_speed.min(diff / dt)
        } else {
            (-max_speed).max(diff / dt)
        };

        // Determine decision state for explainability
        let state = if !ball_approaching || !ball_on_ai_side {
            DecisionState::Idle
        } else if in_reaction_delay {
            DecisionState::Reacting
        } else if diff.abs() < 5.0 {
            DecisionState::Ready
        } else {
            DecisionState::Tracking
        };

        // Update explanation for SHAP widget
        self.last_explanation = DecisionExplanation {
            state,
            output_velocity,
            target_y: self.target_y,
            paddle_y,
            distance_to_target: diff,
            difficulty_level: self.difficulty,
            difficulty_name: profile.name.clone(),
            reaction_delay_ms: profile.reaction_delay_ms,
            reaction_elapsed_ms: self.time_since_ball_visible * 1000.0,
            prediction_accuracy: profile.prediction_accuracy,
            max_paddle_speed: profile.max_paddle_speed,
            error_magnitude: profile.error_magnitude,
            applied_error: self.last_error,
            ball_x,
            ball_y,
            ball_vx,
            ball_vy,
            ball_approaching,
            ball_on_ai_side,
            contributions: Vec::new(),
            rationale: String::new(),
        };
        self.last_explanation.compute_contributions();
        self.last_explanation.generate_rationale();

        output_velocity
    }

    /// Calculates the target Y position for the paddle.
    #[allow(clippy::too_many_arguments, clippy::suboptimal_flops)]
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
        let paddle_x = canvas_width - 35.0;
        let time_to_paddle = if ball_vx > 0.0 {
            (paddle_x - ball_x) / ball_vx
        } else {
            0.0
        };

        let mut predicted_y = ball_y + ball_vy * time_to_paddle * profile.prediction_accuracy;
        predicted_y = predicted_y.clamp(50.0, canvas_height - 50.0);

        // Add deterministic random error (save for explainability)
        self.last_error = (self.next_random() - 0.5) * 2.0 * profile.error_magnitude;
        predicted_y += self.last_error;

        predicted_y.clamp(50.0, canvas_height - 50.0)
    }

    /// Records that the player successfully hit the ball.
    pub fn record_player_hit(&mut self) {
        self.metrics.record_hit();
    }

    /// Records that the player scored (AI missed).
    pub fn record_player_scored(&mut self) {
        let window = self.model.flow_theory.skill_window_size;
        self.metrics.record_player_scored(window);
    }

    /// Records that the player missed (AI scored).
    pub fn record_player_miss(&mut self) {
        let window = self.model.flow_theory.skill_window_size;
        self.metrics.record_ai_scored(window);
    }

    /// Adapts difficulty based on Flow Theory.
    ///
    /// This is the core DDA algorithm based on Csikszentmihalyi's model.
    pub fn adapt_difficulty(&mut self) {
        // Detect current flow channel
        self.flow_channel = self.metrics.detect_flow_channel(&self.model.flow_theory);

        // Calculate target difficulty adjustment
        let adjustment: f32 = match self.flow_channel {
            FlowChannel::Boredom => 1.0,  // Increase difficulty
            FlowChannel::Flow => 0.0,     // Maintain
            FlowChannel::Anxiety => -1.0, // Decrease difficulty
        };

        if adjustment.abs() < 0.01 {
            return; // No adjustment needed
        }

        // Apply adaptation rate
        let rate = self.model.flow_theory.adaptation_rate;
        let current = f32::from(self.difficulty);
        let new_difficulty = (adjustment * rate).mul_add(9.0, current);

        self.difficulty = (new_difficulty.round() as u8).clamp(0, 9);
    }

    /// Resets AI state for a new game.
    pub fn reset(&mut self) {
        self.metrics.reset();
        self.flow_channel = FlowChannel::Flow;
        self.time_since_ball_visible = 0.0;
        self.target_y = 300.0;
        self.ball_acquired = false;
        // Reset RNG to model seed for reproducibility
        self.rng_state = self.model.determinism.seed;
    }

    /// Gets model info as JSON string for display.
    #[must_use]
    pub fn model_info_json(&self) -> String {
        serde_json::json!({
            "name": self.model.metadata.name,
            "version": self.model.metadata.version,
            "description": self.model.metadata.description,
            "author": self.model.metadata.author,
            "license": self.model.metadata.license,
            "difficulty_levels": self.model.difficulty_profiles.len(),
            "current_difficulty": self.difficulty,
            "current_difficulty_name": self.profile().name,
            "flow_channel": self.flow_channel.label(),
            "player_win_rate": self.metrics.recent_win_rate(),
            "model_size_bytes": self.model.serialized_size(),
        })
        .to_string()
    }

    /// Loads a model from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error string if the model fails to load.
    pub fn load_model_from_json(&mut self, json: &str) -> Result<(), String> {
        let model = PongAIModel::from_json(json)?;
        self.rng_state = model.determinism.seed;
        self.model = model;
        Ok(())
    }

    /// Loads a model from bytes (for .apr file loading via aprender).
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
                self.rng_state = model.determinism.seed;
                self.model = model;
                Ok(())
            }
            Err(e) => Err(format!("Failed to load AI model: {e}")),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::unwrap_used)]
mod tests {
    use super::*;

    // ==================== Model Metadata Tests ====================

    #[test]
    fn test_model_metadata_default() {
        let meta = ModelMetadata::default();
        assert_eq!(meta.name, "Pong AI v1");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.author, "PAIML");
        assert_eq!(meta.license, "MIT");
    }

    // ==================== DifficultyProfile Tests ====================

    #[test]
    fn test_difficulty_profile_default() {
        let profile = DifficultyProfile::default();
        assert_eq!(profile.level, 5);
        assert_eq!(profile.name, "Challenging");
        assert!((profile.reaction_delay_ms - 180.0).abs() < 1.0);
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

    // ==================== PongAIModel Tests ====================

    #[test]
    fn test_pong_ai_model_default() {
        let model = PongAIModel::default();
        assert_eq!(model.metadata.version, "1.0.0");
        assert_eq!(model.difficulty_profiles.len(), 10);
        assert_eq!(model.model_type, "behavior_profile");
    }

    #[test]
    fn test_pong_ai_model_new() {
        let model = PongAIModel::new("Test AI", "A test AI model");
        assert_eq!(model.metadata.name, "Test AI");
        assert_eq!(model.metadata.description, "A test AI model");
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

        // Level 0 should be easiest (slowest, least accurate)
        assert!(profiles[0].reaction_delay_ms > profiles[9].reaction_delay_ms);
        assert!(profiles[0].prediction_accuracy < profiles[9].prediction_accuracy);
        assert!(profiles[0].max_paddle_speed < profiles[9].max_paddle_speed);
        assert!(profiles[0].error_magnitude > profiles[9].error_magnitude);
    }

    #[test]
    fn test_generate_default_profiles_names() {
        let profiles = PongAIModel::generate_default_profiles();
        assert_eq!(profiles[0].name, "Training Wheels");
        assert_eq!(profiles[5].name, "Challenging");
        assert_eq!(profiles[9].name, "Perfect");
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
        assert_eq!(profile.level, 9);
    }

    #[test]
    fn test_model_serialization_roundtrip() {
        let model = PongAIModel::default();
        let json = model.to_json();
        let parsed = PongAIModel::from_json(&json).unwrap();

        assert_eq!(parsed.metadata.name, model.metadata.name);
        assert_eq!(
            parsed.difficulty_profiles.len(),
            model.difficulty_profiles.len()
        );
        assert_eq!(parsed.determinism.seed, model.determinism.seed);
    }

    #[test]
    fn test_model_size_reasonable() {
        let model = PongAIModel::default();
        let size = model.serialized_size();
        // Model should be reasonably small (under 5KB)
        assert!(size < 5000, "Model size {size} bytes exceeds 5KB");
    }

    // ==================== FlowChannel Tests ====================

    #[test]
    fn test_flow_channel_labels() {
        assert_eq!(FlowChannel::Boredom.label(), "Bored (too easy)");
        assert_eq!(FlowChannel::Flow.label(), "In Flow (optimal)");
        assert_eq!(FlowChannel::Anxiety.label(), "Anxious (too hard)");
    }

    // ==================== PlayerMetrics Tests ====================

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
    fn test_player_metrics_record_player_scored() {
        let mut metrics = PlayerMetrics::new();
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_player_scored(10);

        assert_eq!(metrics.player_points, 1);
        assert_eq!(metrics.current_rally, 0);
        assert_eq!(metrics.rally_history.len(), 1);
        assert_eq!(metrics.rally_history[0], 2);
        assert!(metrics.point_history[0]); // Player won
    }

    #[test]
    fn test_player_metrics_record_ai_scored() {
        let mut metrics = PlayerMetrics::new();
        metrics.record_hit();
        metrics.record_ai_scored(10);

        assert_eq!(metrics.ai_points, 1);
        assert_eq!(metrics.player_misses, 1);
        assert!(!metrics.point_history[0]); // Player lost
    }

    #[test]
    fn test_player_metrics_window_size() {
        let mut metrics = PlayerMetrics::new();

        for _ in 0..15 {
            metrics.record_player_scored(5);
        }

        assert_eq!(metrics.point_history.len(), 5);
    }

    #[test]
    fn test_player_metrics_recent_win_rate_unknown() {
        let metrics = PlayerMetrics::new();
        let rate = metrics.recent_win_rate();
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_player_metrics_recent_win_rate_all_wins() {
        let mut metrics = PlayerMetrics::new();
        for _ in 0..5 {
            metrics.record_player_scored(10);
        }
        let rate = metrics.recent_win_rate();
        assert!((rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_player_metrics_recent_win_rate_all_losses() {
        let mut metrics = PlayerMetrics::new();
        for _ in 0..5 {
            metrics.record_ai_scored(10);
        }
        let rate = metrics.recent_win_rate();
        assert!((rate - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_player_metrics_detect_flow_channel_boredom() {
        let mut metrics = PlayerMetrics::new();
        // Win 8 out of 10 points
        for _ in 0..8 {
            metrics.record_player_scored(10);
        }
        for _ in 0..2 {
            metrics.record_ai_scored(10);
        }

        let config = FlowTheoryConfig::default();
        let channel = metrics.detect_flow_channel(&config);
        assert_eq!(channel, FlowChannel::Boredom);
    }

    #[test]
    fn test_player_metrics_detect_flow_channel_anxiety() {
        let mut metrics = PlayerMetrics::new();
        // Win only 2 out of 10 points
        for _ in 0..2 {
            metrics.record_player_scored(10);
        }
        for _ in 0..8 {
            metrics.record_ai_scored(10);
        }

        let config = FlowTheoryConfig::default();
        let channel = metrics.detect_flow_channel(&config);
        assert_eq!(channel, FlowChannel::Anxiety);
    }

    #[test]
    fn test_player_metrics_detect_flow_channel_flow() {
        let mut metrics = PlayerMetrics::new();
        // Win 5 out of 10 points (balanced)
        for _ in 0..5 {
            metrics.record_player_scored(10);
        }
        for _ in 0..5 {
            metrics.record_ai_scored(10);
        }

        let config = FlowTheoryConfig::default();
        let channel = metrics.detect_flow_channel(&config);
        assert_eq!(channel, FlowChannel::Flow);
    }

    #[test]
    fn test_player_metrics_estimate_skill_unknown() {
        let metrics = PlayerMetrics::new();
        let skill = metrics.estimate_skill();
        assert!((skill - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_player_metrics_estimate_skill_good_player() {
        let mut metrics = PlayerMetrics::new();
        // Good player: high hit rate, long rallies
        for _ in 0..10 {
            metrics.record_hit();
        }
        metrics.record_ai_scored(10);

        let skill = metrics.estimate_skill();
        assert!(skill > 0.7);
    }

    // ==================== PongAI Tests ====================

    #[test]
    fn test_pong_ai_default() {
        let ai = PongAI::default();
        assert_eq!(ai.difficulty(), 5);
        assert_eq!(ai.flow_channel(), FlowChannel::Flow);
    }

    #[test]
    fn test_pong_ai_with_difficulty() {
        let ai = PongAI::with_difficulty(3);
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
        assert_eq!(profile.name, "Challenging");
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

    #[test]
    fn test_pong_ai_update_ball_not_approaching() {
        let mut ai = PongAI::default();

        let velocity = ai.update(
            400.0, 300.0, // ball position
            -200.0, 100.0, // ball velocity (moving left)
            300.0, 100.0, // paddle position and height
            800.0, 600.0, // canvas size
            0.016, // dt
        );

        assert!(velocity.abs() < ai.profile().max_paddle_speed + 1.0);
    }

    #[test]
    fn test_pong_ai_adapt_difficulty_on_boredom() {
        let mut ai = PongAI::with_difficulty(5);

        // Simulate player winning too much
        for _ in 0..10 {
            ai.record_player_scored();
        }

        let initial = ai.difficulty();
        ai.adapt_difficulty();

        // Difficulty should increase
        assert!(ai.difficulty() >= initial);
        assert_eq!(ai.flow_channel(), FlowChannel::Boredom);
    }

    #[test]
    fn test_pong_ai_adapt_difficulty_on_anxiety() {
        let mut ai = PongAI::with_difficulty(5);

        // Simulate player losing too much
        for _ in 0..10 {
            ai.record_player_miss();
        }

        let initial = ai.difficulty();
        ai.adapt_difficulty();

        // Difficulty should decrease
        assert!(ai.difficulty() <= initial);
        assert_eq!(ai.flow_channel(), FlowChannel::Anxiety);
    }

    #[test]
    fn test_pong_ai_reset() {
        let mut ai = PongAI::default();
        ai.record_player_hit();
        ai.set_difficulty(9);
        ai.time_since_ball_visible = 1.0;

        ai.reset();

        assert_eq!(ai.metrics().player_hits, 0);
        assert!((ai.time_since_ball_visible - 0.0).abs() < 0.01);
        assert_eq!(ai.flow_channel(), FlowChannel::Flow);
    }

    #[test]
    fn test_pong_ai_export_model() {
        let ai = PongAI::default();
        let json = ai.export_model();

        assert!(json.contains("Pong AI v1"));
        assert!(json.contains("difficulty_profiles"));
        assert!(json.contains("flow_theory"));
    }

    #[test]
    fn test_pong_ai_model_info_json() {
        let ai = PongAI::default();
        let json = ai.model_info_json();

        assert!(json.contains("name"));
        assert!(json.contains("version"));
        assert!(json.contains("current_difficulty"));
        assert!(json.contains("flow_channel"));
    }

    #[test]
    fn test_pong_ai_load_model_from_json() {
        let mut ai = PongAI::default();
        let model = PongAIModel::new("Custom AI", "Custom description");
        let json = model.to_json();

        ai.load_model_from_json(&json).unwrap();

        assert_eq!(ai.model().metadata.name, "Custom AI");
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_rally_simulation() {
        let mut ai = PongAI::with_difficulty(5);

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

            paddle_y += velocity * (1.0 / 60.0);
            paddle_y = paddle_y.clamp(50.0, 550.0);

            ball_x += ball_vx * (1.0 / 60.0);
            ball_y += ball_vy * (1.0 / 60.0);
        }

        // AI should have moved toward the ball
        assert!(paddle_y != 300.0 || ball_x < 500.0);
    }

    #[test]
    fn test_difficulty_affects_behavior() {
        let mut ai_easy = PongAI::with_difficulty(0);
        let mut ai_hard = PongAI::with_difficulty(9);

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

    #[test]
    fn test_flow_theory_dda_keeps_player_engaged() {
        let mut ai = PongAI::with_difficulty(5);

        // Simulate alternating wins/losses (balanced play)
        for i in 0..20 {
            if i % 2 == 0 {
                ai.record_player_scored();
            } else {
                ai.record_player_miss();
            }
            ai.adapt_difficulty();
        }

        // Should stay in flow (difficulty should hover around middle)
        assert!(ai.difficulty() >= 3 && ai.difficulty() <= 7);
        assert_eq!(ai.flow_channel(), FlowChannel::Flow);
    }

    // ==================== Coverage Gap Tests ====================

    #[test]
    fn test_load_model_from_bytes_error_path() {
        let mut ai = PongAI::default();

        // Invalid bytes should return an error
        let invalid_bytes: &[u8] = &[0, 1, 2, 3, 4, 5];
        let result = ai.load_model_from_bytes(invalid_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load AI model"));
    }

    #[test]
    fn test_load_model_from_bytes_empty() {
        let mut ai = PongAI::default();

        // Empty bytes should return an error
        let empty_bytes: &[u8] = &[];
        let result = ai.load_model_from_bytes(empty_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_rng_determinism_with_seed() {
        // Create two AIs with the same seed
        let mut ai1 = PongAI::default();
        let mut ai2 = PongAI::default();

        // Set the same RNG state
        ai1.rng_state = 12345;
        ai2.rng_state = 12345;

        // Run several updates - they should produce identical results
        for _ in 0..10 {
            let v1 = ai1.update(
                400.0, 300.0, 200.0, 100.0, 300.0, 100.0, 800.0, 600.0, 0.016,
            );
            let v2 = ai2.update(
                400.0, 300.0, 200.0, 100.0, 300.0, 100.0, 800.0, 600.0, 0.016,
            );
            assert_eq!(v1, v2);
        }
    }

    #[test]
    fn test_finalize_rally_window_trimming() {
        let mut ai = PongAI::default();

        // Record many rallies to trigger window trimming
        for _ in 0..100 {
            ai.record_player_scored();
        }

        // The history should be limited to window size (30 for DDA)
        // This tests the finalize_rally path indirectly via metrics
        assert!(ai.metrics.rally_history.len() <= 30);
    }

    // ==================== High-Priority Coverage Tests ====================

    #[test]
    fn test_pong_ai_model_new_with_metadata() {
        let model = PongAIModel::new("TestAI", "A test AI model");
        assert_eq!(model.metadata.name, "TestAI");
        assert_eq!(model.metadata.description, "A test AI model");
        // Inherits default profiles
        assert_eq!(model.difficulty_profiles.len(), 10);
    }

    #[test]
    fn test_pong_ai_model_generate_default_profiles() {
        let profiles = PongAIModel::generate_default_profiles();
        assert_eq!(profiles.len(), 10);

        // Level 0 - Training Wheels
        assert_eq!(profiles[0].level, 0);
        assert_eq!(profiles[0].name, "Training Wheels");
        assert!(profiles[0].reaction_delay_ms > 400.0); // High delay

        // Level 9 - Perfect (UNBEATABLE)
        let perfect = &profiles[9];
        assert_eq!(perfect.level, 9);
        assert_eq!(perfect.name, "Perfect");
        assert!((perfect.reaction_delay_ms - 0.0).abs() < 0.001); // Instant
        assert!((perfect.prediction_accuracy - 1.0).abs() < 0.001); // Perfect
        assert!((perfect.error_magnitude - 0.0).abs() < 0.001); // Zero error
    }

    #[test]
    fn test_pong_ai_model_to_json_compact() {
        let model = PongAIModel::default();
        let json = model.to_json_compact();
        assert!(!json.is_empty());
        assert!(json.len() < model.to_json().len()); // Compact is smaller
    }

    #[test]
    fn test_pong_ai_model_from_json() {
        let model = PongAIModel::default();
        let json = model.to_json();
        let loaded = PongAIModel::from_json(&json);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.metadata.name, model.metadata.name);
        assert_eq!(loaded.difficulty_profiles.len(), 10);
    }

    #[test]
    fn test_pong_ai_model_from_json_invalid() {
        let result = PongAIModel::from_json("invalid json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to parse model"));
    }

    #[test]
    fn test_player_metrics_average_rally() {
        let mut metrics = PlayerMetrics::default();
        // Empty history returns default
        assert!((metrics.average_rally() - 5.0).abs() < 0.001);

        // Add some rallies
        metrics.rally_history.push(10);
        metrics.rally_history.push(20);
        metrics.rally_history.push(30);
        assert!((metrics.average_rally() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_decision_explanation_compute_contributions() {
        let mut explanation = DecisionExplanation {
            ball_approaching: true,
            ball_vx: 200.0,
            reaction_delay_ms: 100.0,
            distance_to_target: 50.0,
            prediction_accuracy: 0.8,
            max_paddle_speed: 400.0,
            applied_error: 10.0,
            state: DecisionState::Tracking,
            ..Default::default()
        };

        explanation.compute_contributions();
        assert!(!explanation.contributions.is_empty());
        assert_eq!(explanation.contributions.len(), 6);
        // Each contribution should have a name and importance
        for contrib in &explanation.contributions {
            assert!(!contrib.name.is_empty());
            assert!(contrib.importance >= 0.0 && contrib.importance <= 1.0);
        }
    }

    #[test]
    fn test_decision_explanation_generate_rationale() {
        // Test Idle state
        let mut idle = DecisionExplanation {
            state: DecisionState::Idle,
            ball_vx: -100.0,
            ..Default::default()
        };
        idle.generate_rationale();
        assert!(idle.rationale.contains("Ball moving away"));

        // Test Reacting state
        let mut reacting = DecisionExplanation {
            state: DecisionState::Reacting,
            reaction_delay_ms: 150.0,
            reaction_elapsed_ms: 50.0,
            difficulty_level: 3,
            ..Default::default()
        };
        reacting.generate_rationale();
        assert!(reacting.rationale.contains("Reaction delay"));
        assert!(reacting.rationale.contains("Level 3"));

        // Test Tracking state
        let mut tracking = DecisionExplanation {
            state: DecisionState::Tracking,
            target_y: 300.0,
            prediction_accuracy: 0.75,
            max_paddle_speed: 400.0,
            output_velocity: -50.0, // Moving UP
            ..Default::default()
        };
        tracking.generate_rationale();
        assert!(tracking.rationale.contains("Tracking ball"));
        assert!(tracking.rationale.contains("UP"));

        // Test Ready state
        let mut ready = DecisionExplanation {
            state: DecisionState::Ready,
            target_y: 350.0,
            ..Default::default()
        };
        ready.generate_rationale();
        assert!(ready.rationale.contains("At target"));
    }

    #[test]
    fn test_pong_ai_with_difficulty_constructor() {
        let ai = PongAI::with_difficulty(7);
        assert_eq!(ai.difficulty(), 7);
        assert_eq!(ai.difficulty_name(), "Very Hard");
    }

    #[test]
    fn test_pong_ai_difficulty_name_levels() {
        let ai = PongAI::with_difficulty(0);
        assert_eq!(ai.difficulty_name(), "Training Wheels");

        let ai = PongAI::with_difficulty(5);
        assert_eq!(ai.difficulty_name(), "Challenging");
    }

    #[test]
    fn test_pong_ai_metrics_mut() {
        let mut ai = PongAI::default();
        let metrics = ai.metrics_mut();
        metrics.player_hits = 100;
        assert_eq!(ai.metrics().player_hits, 100);
    }

    #[test]
    fn test_pong_ai_explanation_accessor() {
        let ai = PongAI::default();
        let explanation = ai.explanation();
        assert!(matches!(explanation.state, DecisionState::Idle));
    }

    #[test]
    fn test_pong_ai_export_explanation() {
        let ai = PongAI::default();
        let json = ai.export_explanation();
        assert!(json.contains("state"));
        assert!(json.contains("target_y"));
    }

    #[test]
    fn test_decision_explanation_compute_contributions_reacting() {
        let mut explanation = DecisionExplanation {
            ball_approaching: true,
            ball_vx: 200.0,
            reaction_delay_ms: 200.0,
            distance_to_target: 0.0,
            prediction_accuracy: 0.5,
            max_paddle_speed: 300.0,
            applied_error: 0.0,
            state: DecisionState::Reacting, // Reacting state gives different contribution
            ..Default::default()
        };

        explanation.compute_contributions();
        // Find reaction delay contribution
        let reaction_contrib = explanation
            .contributions
            .iter()
            .find(|c| c.name == "Reaction Delay");
        assert!(reaction_contrib.is_some());
        assert!(reaction_contrib.unwrap().contribution > 0.0); // Should be positive when reacting
    }
}
