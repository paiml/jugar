//! Load Testing and Performance Validation
//!
//! Implements Section 11 of the Renacer-Based Game Event Tracing specification.
//!
//! ## Key Components
//!
//! - [`ChaosConfig`] / [`ChaosScenario`]: Chaos engineering configuration
//! - [`ChaosRunner`]: Executes chaos scenarios and collects results
//! - [`FrameTimeStats`]: Percentile-based frame time analysis
//! - [`DriftDetector`]: Z-score based anomaly detection
//!
//! ## Research Foundation
//!
//! - Basiri et al. (2016): Chaos Engineering principles from Netflix
//! - Claessen & Hughes (2000): Property-based testing (QuickCheck)
//! - Dean & Barroso (2013): Tail latency and p99 analysis
//!
//! ## Usage
//!
//! ```ignore
//! use jugar_web::loadtest::{ChaosConfig, ChaosRunner};
//!
//! // Run input flood chaos scenario
//! let config = ChaosConfig::input_flood();
//! let mut runner = ChaosRunner::new(platform, config);
//! let results = runner.run();
//! assert!(!results.nan_detected);
//! ```

use std::collections::VecDeque;

// =============================================================================
// Chaos Engineering Types
// =============================================================================

/// Chaos scenario types for stress testing game systems.
///
/// Each scenario targets specific failure modes:
/// - `EntityStorm`: Memory limits, physics stability
/// - `InputFlood`: Input buffer overflow, dropped inputs
/// - `TimeWarp`: Physics explosion, NaN propagation
/// - `ResizeBlitz`: Layout thrashing, coordinate drift
/// - `ConfigSweep`: Edge case configurations
/// - `RngTorture`: Determinism validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChaosScenario {
    /// Spawn entities at maximum rate to test memory and physics stability.
    EntityStorm {
        /// Maximum number of entities to spawn.
        max_entities: usize,
    },
    /// Flood input buffer with events to test throughput limits.
    InputFlood {
        /// Number of input events to generate per frame.
        events_per_frame: usize,
    },
    /// Vary delta time extremely to test physics stability.
    TimeWarp {
        /// Minimum delta time in seconds.
        min_dt: f32,
        /// Maximum delta time in seconds.
        max_dt: f32,
    },
    /// Rapid resize events to test layout system.
    ResizeBlitz {
        /// Frequency of resize events (per N frames).
        frequency: u32,
    },
    /// Test all configuration permutations.
    ConfigSweep,
    /// Adversarial RNG seeds for determinism validation.
    RngTorture {
        /// Number of seed iterations to test.
        iterations: usize,
    },
}

/// Configuration for chaos testing scenarios.
///
/// # Example
///
/// ```
/// use jugar_web::loadtest::ChaosConfig;
///
/// let config = ChaosConfig::input_flood();
/// assert_eq!(config.duration_frames, 300);
/// ```
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Scenario type to execute.
    pub scenario: ChaosScenario,
    /// Duration in frames.
    pub duration_frames: u64,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Intensity level (0.0 - 1.0).
    pub intensity: f32,
}

impl ChaosConfig {
    /// Create a new chaos configuration.
    #[must_use]
    pub const fn new(scenario: ChaosScenario, duration_frames: u64, seed: u64) -> Self {
        Self {
            scenario,
            duration_frames,
            seed,
            intensity: 1.0,
        }
    }

    /// Standard entity storm for stress testing.
    ///
    /// Spawns up to 1000 entities over 600 frames (10 seconds at 60 FPS).
    #[must_use]
    pub const fn entity_storm() -> Self {
        Self {
            scenario: ChaosScenario::EntityStorm { max_entities: 1000 },
            duration_frames: 600,
            seed: 0xDEAD_BEEF,
            intensity: 1.0,
        }
    }

    /// Input flood to test buffer limits.
    ///
    /// Generates 100 input events per frame for 300 frames.
    #[must_use]
    pub const fn input_flood() -> Self {
        Self {
            scenario: ChaosScenario::InputFlood {
                events_per_frame: 100,
            },
            duration_frames: 300,
            seed: 0xCAFE_BABE,
            intensity: 1.0,
        }
    }

    /// Time warp scenario for physics stability testing.
    ///
    /// Varies delta time from 0.1ms to 1000ms.
    #[must_use]
    pub const fn time_warp() -> Self {
        Self {
            scenario: ChaosScenario::TimeWarp {
                min_dt: 0.000_1,
                max_dt: 1.0,
            },
            duration_frames: 600,
            seed: 0xBAD_F00D,
            intensity: 1.0,
        }
    }

    /// Resize blitz for layout testing.
    ///
    /// Triggers resize every 5 frames for 300 frames.
    #[must_use]
    pub const fn resize_blitz() -> Self {
        Self {
            scenario: ChaosScenario::ResizeBlitz { frequency: 5 },
            duration_frames: 300,
            seed: 0xFEED_FACE,
            intensity: 1.0,
        }
    }

    /// RNG torture test for determinism validation.
    ///
    /// Tests 1000 different RNG seeds.
    #[must_use]
    pub const fn rng_torture() -> Self {
        Self {
            scenario: ChaosScenario::RngTorture { iterations: 1000 },
            duration_frames: 100,
            seed: 0x1234_5678,
            intensity: 1.0,
        }
    }

    /// Configuration sweep for edge case testing.
    #[must_use]
    pub const fn config_sweep() -> Self {
        Self {
            scenario: ChaosScenario::ConfigSweep,
            duration_frames: 60,
            seed: 0xABCD_EF01,
            intensity: 1.0,
        }
    }

    /// Set the intensity level (0.0 - 1.0).
    #[must_use]
    pub const fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
}

/// Results from chaos scenario execution.
#[derive(Debug, Clone, Default)]
pub struct ChaosResults {
    /// Total frames executed.
    pub frames_executed: u64,
    /// Frames that exceeded target time (16.67ms).
    pub slow_frames: u64,
    /// Maximum frame time observed (ms).
    pub max_frame_time_ms: f64,
    /// Minimum frame time observed (ms).
    pub min_frame_time_ms: f64,
    /// Average frame time (ms).
    pub avg_frame_time_ms: f64,
    /// Any panics caught (in catch_unwind context).
    pub panics: Vec<String>,
    /// NaN/Inf values detected in game state.
    pub nan_detected: bool,
    /// Infinity values detected in game state.
    pub inf_detected: bool,
    /// Memory high-water mark (bytes), if tracked.
    pub peak_memory_bytes: Option<usize>,
    /// Inputs dropped (if any).
    pub inputs_dropped: u64,
    /// Scenario that was executed.
    pub scenario: Option<ChaosScenario>,
}

impl ChaosResults {
    /// Create new empty results.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_frame_time_ms: f64::INFINITY,
            ..Default::default()
        }
    }

    /// Check if the chaos test passed (no critical issues).
    #[must_use]
    pub fn passed(&self) -> bool {
        self.panics.is_empty() && !self.nan_detected && !self.inf_detected
    }

    /// Record a frame time sample.
    pub fn record_frame_time(&mut self, frame_time_ms: f64) {
        self.frames_executed += 1;
        if frame_time_ms > 16.67 {
            self.slow_frames += 1;
        }
        if frame_time_ms > self.max_frame_time_ms {
            self.max_frame_time_ms = frame_time_ms;
        }
        if frame_time_ms < self.min_frame_time_ms {
            self.min_frame_time_ms = frame_time_ms;
        }
        // Running average
        let n = self.frames_executed as f64;
        self.avg_frame_time_ms = self.avg_frame_time_ms * (n - 1.0) / n + frame_time_ms / n;
    }

    /// Record a NaN detection.
    pub fn record_nan(&mut self) {
        self.nan_detected = true;
    }

    /// Record an Inf detection.
    pub fn record_inf(&mut self) {
        self.inf_detected = true;
    }

    /// Record a panic message.
    pub fn record_panic(&mut self, message: String) {
        self.panics.push(message);
    }

    /// Record dropped inputs.
    pub fn record_dropped_inputs(&mut self, count: u64) {
        self.inputs_dropped += count;
    }
}

// =============================================================================
// Frame Time Statistics
// =============================================================================

/// Frame time statistics with percentile analysis.
///
/// Based on Dean & Barroso (2013): "The Tail at Scale" - p99 latency
/// is critical for interactive systems like games.
///
/// # Example
///
/// ```
/// use jugar_web::loadtest::FrameTimeStats;
///
/// let mut stats = FrameTimeStats::new();
/// for i in 0..100 {
///     stats.record(i as f64 * 0.1 + 5.0);
/// }
/// let report = stats.report();
/// assert!(report.p50 < report.p99);
/// ```
#[derive(Debug, Clone)]
pub struct FrameTimeStats {
    /// All frame times (ms).
    samples: Vec<f64>,
}

impl Default for FrameTimeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameTimeStats {
    /// Create new empty statistics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Create with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
        }
    }

    /// Record a frame time sample (in milliseconds).
    pub fn record(&mut self, frame_time_ms: f64) {
        self.samples.push(frame_time_ms);
    }

    /// Get the number of samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get percentile value (0-100).
    ///
    /// Common percentiles: p50 (median), p90, p95, p99.
    #[must_use]
    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[idx]
    }

    /// Get minimum value.
    #[must_use]
    pub fn min(&self) -> f64 {
        self.samples.iter().copied().fold(f64::INFINITY, f64::min)
    }

    /// Get maximum value.
    #[must_use]
    pub fn max(&self) -> f64 {
        self.samples
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get mean value.
    #[must_use]
    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    /// Get standard deviation.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self.samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / (self.samples.len() - 1) as f64;
        variance.sqrt()
    }

    /// Generate a summary report.
    #[must_use]
    pub fn report(&self) -> FrameTimeReport {
        FrameTimeReport {
            count: self.samples.len(),
            min: self.min(),
            max: self.max(),
            mean: self.mean(),
            std_dev: self.std_dev(),
            p50: self.percentile(50.0),
            p90: self.percentile(90.0),
            p95: self.percentile(95.0),
            p99: self.percentile(99.0),
        }
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Summary report of frame time statistics.
#[derive(Debug, Clone)]
pub struct FrameTimeReport {
    /// Number of samples.
    pub count: usize,
    /// Minimum frame time (ms).
    pub min: f64,
    /// Maximum frame time (ms).
    pub max: f64,
    /// Mean frame time (ms).
    pub mean: f64,
    /// Standard deviation (ms).
    pub std_dev: f64,
    /// 50th percentile / median (ms).
    pub p50: f64,
    /// 90th percentile (ms).
    pub p90: f64,
    /// 95th percentile (ms).
    pub p95: f64,
    /// 99th percentile (ms).
    pub p99: f64,
}

impl FrameTimeReport {
    /// Check if performance meets 60 FPS target (16.67ms budget).
    #[must_use]
    pub fn meets_60fps(&self) -> bool {
        self.p99 < 16.67
    }

    /// Check if performance meets 120 FPS target (8.33ms budget).
    #[must_use]
    pub fn meets_120fps(&self) -> bool {
        self.p99 < 8.33
    }

    /// Check if performance meets 30 FPS target (33.33ms budget).
    #[must_use]
    pub fn meets_30fps(&self) -> bool {
        self.p99 < 33.33
    }

    /// Get the jitter (max - min).
    #[must_use]
    pub fn jitter(&self) -> f64 {
        self.max - self.min
    }
}

// =============================================================================
// Drift Detection (Anomaly Detection)
// =============================================================================

/// Result of anomaly detection.
#[derive(Debug, Clone)]
pub enum AnomalyResult {
    /// Frame time is within normal range.
    Normal,
    /// Frame time is anomalous.
    Anomaly {
        /// The anomalous value (ms).
        value: f64,
        /// Z-score of the anomaly.
        z_score: f64,
        /// Expected range (min, max) in ms.
        expected_range: (f64, f64),
    },
}

impl AnomalyResult {
    /// Check if this is an anomaly.
    #[must_use]
    pub fn is_anomaly(&self) -> bool {
        matches!(self, Self::Anomaly { .. })
    }

    /// Check if this is normal.
    #[must_use]
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }
}

/// Report of sustained performance drift.
#[derive(Debug, Clone)]
pub struct DriftReport {
    /// Baseline mean frame time (ms).
    pub baseline_mean: f64,
    /// Current window mean frame time (ms).
    pub current_mean: f64,
    /// Drift as a percentage of baseline.
    pub drift_percent: f64,
}

impl DriftReport {
    /// Check if drift is a regression (slower).
    #[must_use]
    pub fn is_regression(&self) -> bool {
        self.drift_percent > 0.0
    }

    /// Check if drift is an improvement (faster).
    #[must_use]
    pub fn is_improvement(&self) -> bool {
        self.drift_percent < 0.0
    }
}

/// Drift detector using Z-score analysis.
///
/// Detects performance regressions using sliding window baseline comparison.
/// Based on statistical process control principles.
///
/// # Example
///
/// ```
/// use jugar_web::loadtest::DriftDetector;
///
/// let mut detector = DriftDetector::new(10, 2.0);
///
/// // Calibrate with baseline samples
/// detector.calibrate(&[1.0, 1.1, 0.9, 1.0, 1.05, 0.95, 1.0, 1.0, 0.98, 1.02]);
///
/// // Normal observation
/// let result = detector.observe(1.0);
/// assert!(result.is_normal());
///
/// // Anomalous observation (way outside normal range)
/// let result = detector.observe(10.0);
/// assert!(result.is_anomaly());
/// ```
#[derive(Debug)]
pub struct DriftDetector {
    /// Sliding window of recent frame times.
    window: VecDeque<f64>,
    /// Window size.
    window_size: usize,
    /// Z-score threshold for anomaly detection.
    z_threshold: f64,
    /// Baseline mean (from calibration).
    baseline_mean: f64,
    /// Baseline standard deviation (from calibration).
    baseline_std: f64,
    /// Drift threshold percentage (default 10%).
    drift_threshold_percent: f64,
}

impl DriftDetector {
    /// Create a new drift detector.
    ///
    /// # Arguments
    ///
    /// * `window_size` - Number of samples in the sliding window
    /// * `z_threshold` - Z-score threshold for anomaly detection (typical: 2.0-3.0)
    #[must_use]
    pub fn new(window_size: usize, z_threshold: f64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            z_threshold,
            baseline_mean: 0.0,
            baseline_std: 1.0,
            drift_threshold_percent: 10.0,
        }
    }

    /// Set the drift threshold percentage.
    #[must_use]
    pub fn with_drift_threshold(mut self, percent: f64) -> Self {
        self.drift_threshold_percent = percent;
        self
    }

    /// Calibrate the detector with baseline samples.
    ///
    /// This establishes the "normal" baseline for comparison.
    pub fn calibrate(&mut self, samples: &[f64]) {
        if samples.is_empty() {
            return;
        }

        self.baseline_mean = samples.iter().sum::<f64>() / samples.len() as f64;

        let variance: f64 = samples
            .iter()
            .map(|x| (x - self.baseline_mean).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        // Avoid division by zero
        self.baseline_std = variance.sqrt().max(0.001);
    }

    /// Check if the detector has been calibrated.
    #[must_use]
    #[allow(clippy::float_cmp)] // Comparing to initial default values is intentional
    pub fn is_calibrated(&self) -> bool {
        self.baseline_mean != 0.0 || self.baseline_std != 1.0
    }

    /// Add a sample and check for anomaly.
    #[allow(clippy::suboptimal_flops)] // Explicit mul-add is clearer here
    pub fn observe(&mut self, frame_time_ms: f64) -> AnomalyResult {
        // Add to sliding window
        self.window.push_back(frame_time_ms);
        if self.window.len() > self.window_size {
            let _ = self.window.pop_front();
        }

        // Calculate Z-score
        let z_score = (frame_time_ms - self.baseline_mean) / self.baseline_std;

        if z_score.abs() > self.z_threshold {
            AnomalyResult::Anomaly {
                value: frame_time_ms,
                z_score,
                expected_range: (
                    self.baseline_mean - self.z_threshold * self.baseline_std,
                    self.baseline_mean + self.z_threshold * self.baseline_std,
                ),
            }
        } else {
            AnomalyResult::Normal
        }
    }

    /// Detect sustained drift (window mean significantly different from baseline).
    #[must_use]
    pub fn detect_drift(&self) -> Option<DriftReport> {
        if self.window.len() < self.window_size {
            return None;
        }

        let window_mean: f64 = self.window.iter().sum::<f64>() / self.window.len() as f64;

        // Avoid division by zero
        if self.baseline_mean.abs() < 0.001 {
            return None;
        }

        let drift = (window_mean - self.baseline_mean) / self.baseline_mean * 100.0;

        if drift.abs() > self.drift_threshold_percent {
            Some(DriftReport {
                baseline_mean: self.baseline_mean,
                current_mean: window_mean,
                drift_percent: drift,
            })
        } else {
            None
        }
    }

    /// Get current window statistics.
    #[must_use]
    pub fn window_stats(&self) -> Option<(f64, f64)> {
        if self.window.is_empty() {
            return None;
        }

        let mean = self.window.iter().sum::<f64>() / self.window.len() as f64;
        let variance: f64 =
            self.window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.window.len() as f64;

        Some((mean, variance.sqrt()))
    }

    /// Reset the detector (clear window, keep calibration).
    pub fn reset(&mut self) {
        self.window.clear();
    }

    /// Full reset (clear window and calibration).
    pub fn full_reset(&mut self) {
        self.window.clear();
        self.baseline_mean = 0.0;
        self.baseline_std = 1.0;
    }
}

// =============================================================================
// Load Test Configuration
// =============================================================================

/// Configuration for a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Name of the test.
    pub name: String,
    /// Chaos configuration (optional).
    pub chaos: Option<ChaosConfig>,
    /// Run property-based tests.
    pub property_tests: bool,
    /// Run benchmarks.
    pub benchmark: bool,
    /// Duration in frames.
    pub duration_frames: u64,
    /// Tier level (1, 2, or 3).
    pub tier: u8,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            chaos: None,
            property_tests: false,
            benchmark: false,
            duration_frames: 600,
            tier: 1,
        }
    }
}

impl LoadTestConfig {
    /// Create a new load test configuration.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create Tier 1 config (on-save, < 5 seconds).
    #[must_use]
    pub fn tier1(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chaos: None,
            property_tests: false,
            benchmark: false,
            duration_frames: 60,
            tier: 1,
        }
    }

    /// Create Tier 2 config (on-commit, < 30 seconds).
    #[must_use]
    pub fn tier2(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chaos: Some(ChaosConfig::input_flood()),
            property_tests: true,
            benchmark: false,
            duration_frames: 300,
            tier: 2,
        }
    }

    /// Create Tier 3 config (on-merge, < 5 minutes).
    #[must_use]
    pub fn tier3(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chaos: Some(ChaosConfig::entity_storm()),
            property_tests: true,
            benchmark: true,
            duration_frames: 3600,
            tier: 3,
        }
    }

    /// Add chaos configuration.
    #[must_use]
    pub fn with_chaos(mut self, chaos: ChaosConfig) -> Self {
        self.chaos = Some(chaos);
        self
    }

    /// Enable property tests.
    #[must_use]
    pub fn with_property_tests(mut self) -> Self {
        self.property_tests = true;
        self
    }

    /// Enable benchmarks.
    #[must_use]
    pub fn with_benchmarks(mut self) -> Self {
        self.benchmark = true;
        self
    }

    /// Set duration in frames.
    #[must_use]
    pub fn with_duration(mut self, frames: u64) -> Self {
        self.duration_frames = frames;
        self
    }
}

/// Result from a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// Name of the test.
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Chaos results (if chaos was run).
    pub chaos_results: Option<ChaosResults>,
    /// Frame time statistics.
    pub frame_stats: FrameTimeReport,
    /// Anomalies detected.
    pub anomaly_count: usize,
    /// Property test failures (if any).
    pub property_failures: Vec<String>,
    /// Error message (if failed).
    pub error: Option<String>,
}

impl LoadTestResult {
    /// Create a passing result.
    #[must_use]
    pub fn pass(name: impl Into<String>, frame_stats: FrameTimeReport) -> Self {
        Self {
            name: name.into(),
            passed: true,
            chaos_results: None,
            frame_stats,
            anomaly_count: 0,
            property_failures: Vec::new(),
            error: None,
        }
    }

    /// Create a failing result.
    #[must_use]
    pub fn fail(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            chaos_results: None,
            frame_stats: FrameTimeReport {
                count: 0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                std_dev: 0.0,
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
            },
            anomaly_count: 0,
            property_failures: Vec::new(),
            error: Some(error.into()),
        }
    }
}

/// Summary of all load test results.
#[derive(Debug, Clone)]
pub struct LoadTestSummary {
    /// Total number of tests.
    pub total: usize,
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Individual results.
    pub results: Vec<LoadTestResult>,
}

impl LoadTestSummary {
    /// Create a new empty summary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            results: Vec::new(),
        }
    }

    /// Add a result to the summary.
    pub fn add(&mut self, result: LoadTestResult) {
        self.total += 1;
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }

    /// Check if all tests passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

impl Default for LoadTestSummary {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless, clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ChaosConfig tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_chaos_config_entity_storm() {
        let config = ChaosConfig::entity_storm();
        assert_eq!(config.duration_frames, 600);
        assert!(matches!(
            config.scenario,
            ChaosScenario::EntityStorm { max_entities: 1000 }
        ));
    }

    #[test]
    fn test_chaos_config_input_flood() {
        let config = ChaosConfig::input_flood();
        assert_eq!(config.duration_frames, 300);
        assert!(matches!(
            config.scenario,
            ChaosScenario::InputFlood {
                events_per_frame: 100
            }
        ));
    }

    #[test]
    fn test_chaos_config_time_warp() {
        let config = ChaosConfig::time_warp();
        assert!(matches!(
            config.scenario,
            ChaosScenario::TimeWarp { min_dt, max_dt } if min_dt < max_dt
        ));
    }

    #[test]
    fn test_chaos_config_with_intensity() {
        let config = ChaosConfig::entity_storm().with_intensity(0.5);
        assert!((config.intensity - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // ChaosResults tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_chaos_results_new() {
        let results = ChaosResults::new();
        assert_eq!(results.frames_executed, 0);
        assert!(results.passed());
    }

    #[test]
    fn test_chaos_results_record_frame_time() {
        let mut results = ChaosResults::new();
        results.record_frame_time(10.0);
        results.record_frame_time(20.0);
        assert_eq!(results.frames_executed, 2);
        assert!((results.max_frame_time_ms - 20.0).abs() < f64::EPSILON);
        assert!((results.min_frame_time_ms - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_chaos_results_slow_frames() {
        let mut results = ChaosResults::new();
        results.record_frame_time(10.0); // Fast
        results.record_frame_time(20.0); // Slow (> 16.67ms)
        assert_eq!(results.slow_frames, 1);
    }

    #[test]
    fn test_chaos_results_nan_detection() {
        let mut results = ChaosResults::new();
        assert!(results.passed());
        results.record_nan();
        assert!(!results.passed());
    }

    #[test]
    fn test_chaos_results_inf_detection() {
        let mut results = ChaosResults::new();
        assert!(results.passed());
        results.record_inf();
        assert!(!results.passed());
    }

    #[test]
    fn test_chaos_results_panic_recording() {
        let mut results = ChaosResults::new();
        assert!(results.passed());
        results.record_panic("test panic".to_string());
        assert!(!results.passed());
    }

    // -------------------------------------------------------------------------
    // FrameTimeStats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_frame_time_stats_empty() {
        let stats = FrameTimeStats::new();
        assert!(stats.is_empty());
        assert_eq!(stats.len(), 0);
        assert!((stats.percentile(50.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_time_stats_record() {
        let mut stats = FrameTimeStats::new();
        stats.record(10.0);
        stats.record(20.0);
        stats.record(30.0);
        assert_eq!(stats.len(), 3);
        assert!((stats.mean() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_time_stats_percentiles() {
        let mut stats = FrameTimeStats::new();
        for i in 1..=100 {
            stats.record(i as f64);
        }
        assert!((stats.percentile(50.0) - 50.0).abs() < 1.0);
        assert!((stats.percentile(90.0) - 90.0).abs() < 1.0);
        assert!((stats.percentile(99.0) - 99.0).abs() < 1.0);
    }

    #[test]
    fn test_frame_time_stats_min_max() {
        let mut stats = FrameTimeStats::new();
        stats.record(5.0);
        stats.record(10.0);
        stats.record(3.0);
        assert!((stats.min() - 3.0).abs() < f64::EPSILON);
        assert!((stats.max() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_time_stats_std_dev() {
        let mut stats = FrameTimeStats::new();
        // Standard deviation of [2, 4, 4, 4, 5, 5, 7, 9] is 2.0
        for v in [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0] {
            stats.record(v);
        }
        assert!((stats.std_dev() - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_frame_time_report_fps_targets() {
        let mut stats = FrameTimeStats::new();
        for _ in 0..100 {
            stats.record(5.0); // 5ms = 200 FPS
        }
        let report = stats.report();
        assert!(report.meets_120fps());
        assert!(report.meets_60fps());
        assert!(report.meets_30fps());
    }

    #[test]
    fn test_frame_time_report_fps_fail() {
        let mut stats = FrameTimeStats::new();
        for _ in 0..100 {
            stats.record(50.0); // 50ms = 20 FPS
        }
        let report = stats.report();
        assert!(!report.meets_120fps());
        assert!(!report.meets_60fps());
        assert!(!report.meets_30fps());
    }

    // -------------------------------------------------------------------------
    // DriftDetector tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drift_detector_new() {
        let detector = DriftDetector::new(10, 2.0);
        assert!(!detector.is_calibrated());
    }

    #[test]
    fn test_drift_detector_calibrate() {
        let mut detector = DriftDetector::new(10, 2.0);
        detector.calibrate(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        assert!(detector.is_calibrated());
    }

    #[test]
    fn test_drift_detector_observe_normal() {
        let mut detector = DriftDetector::new(10, 2.0);
        detector.calibrate(&[1.0, 1.1, 0.9, 1.0, 1.05]);
        let result = detector.observe(1.0);
        assert!(result.is_normal());
    }

    #[test]
    fn test_drift_detector_observe_anomaly() {
        let mut detector = DriftDetector::new(10, 2.0);
        detector.calibrate(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        let result = detector.observe(10.0); // Way outside normal
        assert!(result.is_anomaly());
    }

    #[test]
    fn test_drift_detector_detect_drift() {
        let mut detector = DriftDetector::new(5, 2.0).with_drift_threshold(10.0);
        detector.calibrate(&[1.0, 1.0, 1.0, 1.0, 1.0]);

        // Fill window with higher values
        for _ in 0..5 {
            let _ = detector.observe(1.5); // 50% higher
        }

        let drift = detector.detect_drift();
        assert!(drift.is_some());
        assert!(drift.unwrap().is_regression());
    }

    #[test]
    fn test_drift_detector_no_drift() {
        let mut detector = DriftDetector::new(5, 2.0).with_drift_threshold(10.0);
        detector.calibrate(&[1.0, 1.0, 1.0, 1.0, 1.0]);

        // Fill window with similar values
        for _ in 0..5 {
            let _ = detector.observe(1.05); // 5% higher (below threshold)
        }

        let drift = detector.detect_drift();
        assert!(drift.is_none());
    }

    #[test]
    fn test_drift_detector_reset() {
        let mut detector = DriftDetector::new(5, 2.0);
        detector.calibrate(&[1.0, 1.0, 1.0]);
        let _ = detector.observe(1.0);

        detector.reset();
        assert!(detector.window_stats().is_none());
        assert!(detector.is_calibrated()); // Calibration preserved
    }

    #[test]
    fn test_drift_detector_full_reset() {
        let mut detector = DriftDetector::new(5, 2.0);
        detector.calibrate(&[1.0, 1.0, 1.0]);
        let _ = detector.observe(1.0);

        detector.full_reset();
        assert!(detector.window_stats().is_none());
        assert!(!detector.is_calibrated()); // Calibration cleared
    }

    // -------------------------------------------------------------------------
    // LoadTestConfig tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_load_test_config_tier1() {
        let config = LoadTestConfig::tier1("fast_test");
        assert_eq!(config.tier, 1);
        assert_eq!(config.duration_frames, 60);
        assert!(!config.property_tests);
        assert!(!config.benchmark);
    }

    #[test]
    fn test_load_test_config_tier2() {
        let config = LoadTestConfig::tier2("medium_test");
        assert_eq!(config.tier, 2);
        assert!(config.property_tests);
        assert!(config.chaos.is_some());
    }

    #[test]
    fn test_load_test_config_tier3() {
        let config = LoadTestConfig::tier3("full_test");
        assert_eq!(config.tier, 3);
        assert!(config.property_tests);
        assert!(config.benchmark);
        assert!(config.chaos.is_some());
    }

    #[test]
    fn test_load_test_config_builder() {
        let config = LoadTestConfig::new("custom")
            .with_chaos(ChaosConfig::time_warp())
            .with_property_tests()
            .with_benchmarks()
            .with_duration(1000);

        assert_eq!(config.name, "custom");
        assert!(config.chaos.is_some());
        assert!(config.property_tests);
        assert!(config.benchmark);
        assert_eq!(config.duration_frames, 1000);
    }

    // -------------------------------------------------------------------------
    // LoadTestResult tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_load_test_result_pass() {
        let stats = FrameTimeStats::new();
        let result = LoadTestResult::pass("test", stats.report());
        assert!(result.passed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_load_test_result_fail() {
        let result = LoadTestResult::fail("test", "something went wrong");
        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    // -------------------------------------------------------------------------
    // LoadTestSummary tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_load_test_summary_add() {
        let mut summary = LoadTestSummary::new();
        assert_eq!(summary.total, 0);

        let stats = FrameTimeStats::new();
        summary.add(LoadTestResult::pass("test1", stats.report()));
        summary.add(LoadTestResult::fail("test2", "error"));

        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.all_passed());
    }

    #[test]
    fn test_load_test_summary_all_passed() {
        let mut summary = LoadTestSummary::new();
        let stats = FrameTimeStats::new();
        summary.add(LoadTestResult::pass("test1", stats.report()));
        summary.add(LoadTestResult::pass("test2", stats.report()));

        assert!(summary.all_passed());
    }
}
