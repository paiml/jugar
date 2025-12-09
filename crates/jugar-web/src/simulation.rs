//! Game Simulation Testing Framework
//!
//! This module implements the Monte Carlo simulation testing framework
//! as specified in `docs/qa/game-replay-testing.md`.
//!
//! ## Tiered Testing Strategy
//!
//! - **Tier 1 (Smoke)**: 1 seed, deterministic, < 10 seconds
//! - **Tier 2 (Regression)**: 50 seeds, 95% confidence, < 5 minutes
//! - **Tier 3 (Full)**: 1000 seeds, 99% confidence, < 30 minutes

// Allow relaxed clippy lints for simulation testing framework
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::match_same_arms)]

use serde::{Deserialize, Serialize};
use std::env;

/// Test tier for Monte Carlo simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestTier {
    /// Smoke: 1 seed, deterministic (pre-commit)
    #[default]
    Smoke,
    /// Regression: 50 seeds, 95% confidence (on-merge)
    Regression,
    /// Full: 1000 seeds, 99% confidence (nightly)
    Full,
}

impl TestTier {
    /// Parse tier from environment variable TEST_TIER
    pub fn from_env() -> Self {
        match env::var("TEST_TIER").as_deref() {
            Ok("smoke") => Self::Smoke,
            Ok("regression") => Self::Regression,
            Ok("full") => Self::Full,
            _ => Self::Smoke, // Default to smoke for fast feedback
        }
    }
}

/// Monte Carlo test configuration
///
/// Configures how many random seeds and frames to run in simulation tests.
///
/// # Example
///
/// ```
/// use jugar_web::simulation::{MonteCarloConfig, TestTier};
///
/// let config = MonteCarloConfig::smoke();
/// assert_eq!(config.tier, TestTier::Smoke);
/// assert_eq!(config.iterations(), 1);
///
/// let regression = MonteCarloConfig::regression();
/// assert_eq!(regression.iterations(), 50);
/// ```
#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    /// Test tier (determines iteration count)
    pub tier: TestTier,
    /// RNG seed range start
    pub seed_start: u64,
    /// RNG seed range end
    pub seed_end: u64,
    /// Timeout per batch (seconds)
    pub batch_timeout: u64,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Frames per simulation
    pub frames_per_sim: usize,
}

impl MonteCarloConfig {
    /// Smoke test configuration (Tier 1)
    pub fn smoke() -> Self {
        Self {
            tier: TestTier::Smoke,
            seed_start: 42,
            seed_end: 42,
            batch_timeout: 10,
            confidence: 0.95,
            frames_per_sim: 600, // 10 seconds @ 60fps
        }
    }

    /// Regression test configuration (Tier 2)
    pub fn regression() -> Self {
        Self {
            tier: TestTier::Regression,
            seed_start: 0,
            seed_end: 49,
            batch_timeout: 60,
            confidence: 0.95,
            frames_per_sim: 1800, // 30 seconds @ 60fps
        }
    }

    /// Full Monte Carlo configuration (Tier 3)
    pub fn full() -> Self {
        Self {
            tier: TestTier::Full,
            seed_start: 0,
            seed_end: 999,
            batch_timeout: 300,
            confidence: 0.99,
            frames_per_sim: 3600, // 1 minute @ 60fps
        }
    }

    /// Get configuration from environment
    pub fn from_env() -> Self {
        match TestTier::from_env() {
            TestTier::Smoke => Self::smoke(),
            TestTier::Regression => Self::regression(),
            TestTier::Full => Self::full(),
        }
    }

    /// Number of iterations (seeds)
    pub fn iterations(&self) -> usize {
        (self.seed_end - self.seed_start + 1) as usize
    }
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self::smoke()
    }
}

/// Game state snapshot for failure replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    /// Ball X position
    pub ball_x: f64,
    /// Ball Y position
    pub ball_y: f64,
    /// Ball X velocity
    pub ball_vx: f64,
    /// Ball Y velocity
    pub ball_vy: f64,
    /// Left paddle Y position
    pub left_paddle_y: f64,
    /// Right paddle Y position
    pub right_paddle_y: f64,
    /// Left player score
    pub score_left: u32,
    /// Right player score
    pub score_right: u32,
    /// Current rally count
    pub rally: u32,
    /// Game state (Menu/Playing/Paused/GameOver)
    pub game_state: String,
    /// Game mode (Demo/SinglePlayer/TwoPlayer)
    pub game_mode: String,
}

/// Input event for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedInput {
    /// Frame number
    pub frame: usize,
    /// Timestamp in milliseconds
    pub timestamp_ms: f64,
    /// Input event JSON
    pub event: String,
}

/// Failure replay artifact for deterministic reproduction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureReplay {
    /// Test action ID (1-100)
    pub action_id: u32,
    /// Test name
    pub action_name: String,
    /// Random seed that caused failure
    pub seed: u64,
    /// Monte Carlo config used
    pub tier: String,
    /// Complete input trace
    pub input_trace: Vec<TimestampedInput>,
    /// Frame at which assertion failed
    pub failure_frame: usize,
    /// Assertion that failed
    pub assertion: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
    /// Full game state at failure
    pub state_snapshot: GameStateSnapshot,
    /// Timestamp
    pub timestamp: String,
}

impl FailureReplay {
    /// Save failure replay to file
    pub fn save(&self) -> std::io::Result<String> {
        let filename = format!(
            "target/test-failures/failure-{}-action-{:03}-seed-{}.json",
            chrono_lite_timestamp(),
            self.action_id,
            self.seed
        );

        // Ensure directory exists
        std::fs::create_dir_all("target/test-failures")?;

        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&filename, json)?;

        Ok(filename)
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Test result for Monte Carlo harness
#[derive(Debug)]
pub enum TestResult {
    /// Test passed
    Pass,
    /// Test failed with details
    Fail {
        /// Assertion that failed
        assertion: String,
        /// Expected value
        expected: String,
        /// Actual value
        actual: String,
        /// Game state at failure
        state: GameStateSnapshot,
    },
}

impl TestResult {
    /// Create a failure result
    pub fn fail<T: std::fmt::Display>(
        assertion: &str,
        expected: &str,
        actual: T,
        state: GameStateSnapshot,
    ) -> Self {
        Self::Fail {
            assertion: assertion.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            state,
        }
    }

    /// Check if result is pass
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Invariant violation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    /// Ball position is NaN
    BallPositionNaN,
    /// Ball position is infinite
    BallPositionInfinite,
    /// Paddle is out of bounds
    PaddleOutOfBounds,
    /// Score overflow
    ScoreOverflow,
    /// Rally overflow
    RallyOverflow,
    /// Velocity is NaN
    VelocityNaN,
    /// Velocity is infinite
    VelocityInfinite,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BallPositionNaN => write!(f, "Ball position is NaN"),
            Self::BallPositionInfinite => write!(f, "Ball position is infinite"),
            Self::PaddleOutOfBounds => write!(f, "Paddle is out of bounds"),
            Self::ScoreOverflow => write!(f, "Score overflow"),
            Self::RallyOverflow => write!(f, "Rally overflow"),
            Self::VelocityNaN => write!(f, "Velocity is NaN"),
            Self::VelocityInfinite => write!(f, "Velocity is infinite"),
        }
    }
}

impl std::error::Error for InvariantViolation {}

/// Check game state invariants
///
/// Validates that the game state is in a valid, non-corrupted state.
///
/// # Arguments
///
/// * `snapshot` - The game state to validate
/// * `max_y` - Maximum Y coordinate (screen height)
///
/// # Errors
///
/// Returns [`InvariantViolation`] if any game state invariant is violated:
/// - Ball position is NaN or infinite
/// - Paddle position is out of bounds
/// - Score or rally count overflow
/// - Velocity is NaN or infinite
///
/// # Example
///
/// ```
/// use jugar_web::simulation::{check_invariants, GameStateSnapshot};
///
/// let valid_state = GameStateSnapshot {
///     ball_x: 400.0,
///     ball_y: 300.0,
///     ball_vx: 200.0,
///     ball_vy: 150.0,
///     left_paddle_y: 300.0,
///     right_paddle_y: 300.0,
///     score_left: 5,
///     score_right: 3,
///     rally: 10,
///     game_state: "Playing".to_string(),
///     game_mode: "Demo".to_string(),
/// };
///
/// assert!(check_invariants(&valid_state, 600.0).is_ok());
/// ```
pub fn check_invariants(
    snapshot: &GameStateSnapshot,
    max_y: f64,
) -> Result<(), InvariantViolation> {
    // Ball must never be NaN
    if snapshot.ball_x.is_nan() || snapshot.ball_y.is_nan() {
        return Err(InvariantViolation::BallPositionNaN);
    }

    // Ball must never be Infinity
    if snapshot.ball_x.is_infinite() || snapshot.ball_y.is_infinite() {
        return Err(InvariantViolation::BallPositionInfinite);
    }

    // Velocity must never be NaN
    if snapshot.ball_vx.is_nan() || snapshot.ball_vy.is_nan() {
        return Err(InvariantViolation::VelocityNaN);
    }

    // Velocity must never be Infinity
    if snapshot.ball_vx.is_infinite() || snapshot.ball_vy.is_infinite() {
        return Err(InvariantViolation::VelocityInfinite);
    }

    // Paddles must be within bounds (with some tolerance for edge cases)
    if snapshot.left_paddle_y < -10.0 || snapshot.left_paddle_y > max_y + 10.0 {
        return Err(InvariantViolation::PaddleOutOfBounds);
    }
    if snapshot.right_paddle_y < -10.0 || snapshot.right_paddle_y > max_y + 10.0 {
        return Err(InvariantViolation::PaddleOutOfBounds);
    }

    // Scores must be reasonable
    if snapshot.score_left > 1000 || snapshot.score_right > 1000 {
        return Err(InvariantViolation::ScoreOverflow);
    }

    // Rally must be reasonable
    if snapshot.rally > 100_000 {
        return Err(InvariantViolation::RallyOverflow);
    }

    Ok(())
}

/// Hostile input generator for boundary testing (fuzzing)
#[derive(Debug)]
pub struct FuzzGenerator;

impl FuzzGenerator {
    /// Category 1: Numeric Extremes
    #[must_use]
    pub fn numeric_extremes() -> Vec<f64> {
        vec![
            0.0,
            -0.0,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            f64::EPSILON,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            1e-300,
            1e300,
        ]
    }

    /// Category 2: Delta Time Extremes
    #[must_use]
    pub fn dt_extremes() -> Vec<f64> {
        vec![
            0.0,     // Freeze frame
            0.001,   // 1000 FPS
            16.667,  // Normal 60 FPS
            33.333,  // 30 FPS
            100.0,   // 10 FPS (lag spike)
            1000.0,  // 1 FPS (severe lag)
            5000.0,  // Tab backgrounded
            10000.0, // Extreme lag
        ]
    }

    /// Category 3: Position Extremes
    #[must_use]
    pub fn position_extremes(width: f64, height: f64) -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (width, height),
            (-1.0, -1.0),
            (width + 1.0, height + 1.0),
            (width / 2.0, height / 2.0),
            (-1000.0, -1000.0),
            (width + 1000.0, height + 1000.0),
        ]
    }

    /// Category 4: Velocity Extremes
    #[must_use]
    pub fn velocity_extremes() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (1000.0, 0.0),
            (0.0, 1000.0),
            (-1000.0, -1000.0),
            (10000.0, 10000.0),
            (-10000.0, -10000.0),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_config_smoke() {
        let config = MonteCarloConfig::smoke();
        assert_eq!(config.tier, TestTier::Smoke);
        assert_eq!(config.iterations(), 1);
    }

    #[test]
    fn test_monte_carlo_config_regression() {
        let config = MonteCarloConfig::regression();
        assert_eq!(config.tier, TestTier::Regression);
        assert_eq!(config.iterations(), 50);
    }

    #[test]
    fn test_monte_carlo_config_full() {
        let config = MonteCarloConfig::full();
        assert_eq!(config.tier, TestTier::Full);
        assert_eq!(config.iterations(), 1000);
    }

    #[test]
    fn test_invariant_check_valid() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 5,
            score_right: 3,
            rally: 10,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert!(check_invariants(&snapshot, 600.0).is_ok());
    }

    #[test]
    fn test_invariant_check_nan_ball() {
        let snapshot = GameStateSnapshot {
            ball_x: f64::NAN,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::BallPositionNaN)
        );
    }

    #[test]
    fn test_invariant_check_infinite_velocity() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: f64::INFINITY,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::VelocityInfinite)
        );
    }

    #[test]
    fn test_invariant_check_score_overflow() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 9999,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::ScoreOverflow)
        );
    }

    #[test]
    fn test_fuzz_generator_dt_extremes() {
        let extremes = FuzzGenerator::dt_extremes();
        assert!(extremes.contains(&0.0));
        assert!(extremes.contains(&16.667));
        assert!(extremes.len() >= 5);
    }

    #[test]
    fn test_fuzz_generator_numeric_extremes() {
        let extremes = FuzzGenerator::numeric_extremes();
        assert!(extremes.iter().any(|x| x.is_nan()));
        assert!(extremes.iter().any(|x| x.is_infinite()));
        assert!(extremes.contains(&0.0));
    }

    #[test]
    fn test_test_result_pass() {
        let result = TestResult::Pass;
        assert!(result.is_pass());
    }

    #[test]
    fn test_test_result_fail() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        let result = TestResult::fail("v > 0", "> 0", -5.0, snapshot);
        assert!(!result.is_pass());
    }

    #[test]
    fn test_test_tier_default() {
        assert_eq!(TestTier::default(), TestTier::Smoke);
    }

    #[test]
    fn test_monte_carlo_config_default() {
        let config = MonteCarloConfig::default();
        assert_eq!(config.tier, TestTier::Smoke);
    }

    #[test]
    fn test_fuzz_generator_position_extremes() {
        let extremes = FuzzGenerator::position_extremes(800.0, 600.0);
        assert!(extremes.contains(&(0.0, 0.0)));
        assert!(extremes.contains(&(800.0, 600.0)));
        assert!(extremes.contains(&(400.0, 300.0))); // center
        assert!(extremes.len() >= 5);
    }

    #[test]
    fn test_fuzz_generator_velocity_extremes() {
        let extremes = FuzzGenerator::velocity_extremes();
        assert!(extremes.contains(&(0.0, 0.0)));
        assert!(extremes.contains(&(1000.0, 0.0)));
        assert!(extremes.len() >= 4);
    }

    #[test]
    fn test_invariant_check_infinite_ball() {
        let snapshot = GameStateSnapshot {
            ball_x: f64::INFINITY,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::BallPositionInfinite)
        );
    }

    #[test]
    fn test_invariant_check_nan_velocity() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: f64::NAN,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::VelocityNaN)
        );
    }

    #[test]
    fn test_invariant_check_paddle_out_of_bounds_left() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: -100.0, // Way out of bounds
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::PaddleOutOfBounds)
        );
    }

    #[test]
    fn test_invariant_check_paddle_out_of_bounds_right() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 800.0, // Way out of bounds for max_y=600
            score_left: 0,
            score_right: 0,
            rally: 0,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::PaddleOutOfBounds)
        );
    }

    #[test]
    fn test_invariant_check_rally_overflow() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 0,
            score_right: 0,
            rally: 200_000, // Overflow
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        assert_eq!(
            check_invariants(&snapshot, 600.0),
            Err(InvariantViolation::RallyOverflow)
        );
    }

    #[test]
    fn test_invariant_violation_display() {
        assert_eq!(
            format!("{}", InvariantViolation::BallPositionNaN),
            "Ball position is NaN"
        );
        assert_eq!(
            format!("{}", InvariantViolation::BallPositionInfinite),
            "Ball position is infinite"
        );
        assert_eq!(
            format!("{}", InvariantViolation::PaddleOutOfBounds),
            "Paddle is out of bounds"
        );
        assert_eq!(
            format!("{}", InvariantViolation::ScoreOverflow),
            "Score overflow"
        );
        assert_eq!(
            format!("{}", InvariantViolation::RallyOverflow),
            "Rally overflow"
        );
        assert_eq!(
            format!("{}", InvariantViolation::VelocityNaN),
            "Velocity is NaN"
        );
        assert_eq!(
            format!("{}", InvariantViolation::VelocityInfinite),
            "Velocity is infinite"
        );
    }

    #[test]
    fn test_invariant_violation_error_trait() {
        let violation: Box<dyn std::error::Error> = Box::new(InvariantViolation::BallPositionNaN);
        assert!(violation.to_string().contains("NaN"));
    }

    #[test]
    fn test_failure_replay_save() {
        let replay = FailureReplay {
            action_id: 1,
            action_name: "test_action".to_string(),
            seed: 42,
            tier: "smoke".to_string(),
            input_trace: vec![TimestampedInput {
                frame: 0,
                timestamp_ms: 0.0,
                event: r#"{"type":"KeyDown","key":"KeyW"}"#.to_string(),
            }],
            failure_frame: 100,
            assertion: "ball_x > 0".to_string(),
            expected: "> 0".to_string(),
            actual: "-5".to_string(),
            state_snapshot: GameStateSnapshot {
                ball_x: -5.0,
                ball_y: 300.0,
                ball_vx: 200.0,
                ball_vy: 150.0,
                left_paddle_y: 300.0,
                right_paddle_y: 300.0,
                score_left: 0,
                score_right: 0,
                rally: 0,
                game_state: "Playing".to_string(),
                game_mode: "Demo".to_string(),
            },
            timestamp: "test".to_string(),
        };

        let result = replay.save();
        assert!(result.is_ok());
        let filename = result.unwrap();
        assert!(filename.contains("failure-"));
        assert!(filename.contains("-action-001-seed-42.json"));

        // Clean up
        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_game_state_snapshot_serialization() {
        let snapshot = GameStateSnapshot {
            ball_x: 400.0,
            ball_y: 300.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            score_left: 5,
            score_right: 3,
            rally: 10,
            game_state: "Playing".to_string(),
            game_mode: "Demo".to_string(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("ball_x"));
        assert!(json.contains("400"));

        let deserialized: GameStateSnapshot = serde_json::from_str(&json).unwrap();
        assert!((deserialized.ball_x - 400.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timestamped_input_serialization() {
        let input = TimestampedInput {
            frame: 42,
            timestamp_ms: 1234.5,
            event: r#"{"type":"KeyDown"}"#.to_string(),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("42"));

        let deserialized: TimestampedInput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.frame, 42);
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let ts1 = chrono_lite_timestamp();
        let ts2 = chrono_lite_timestamp();
        // Should be the same or very close
        let n1: u64 = ts1.parse().unwrap();
        let n2: u64 = ts2.parse().unwrap();
        assert!(n2 >= n1);
        assert!(n2 - n1 <= 1); // Within 1 second
    }
}
