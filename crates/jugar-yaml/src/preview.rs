//! Live Preview system for real-time YAML hot-reload.
//!
//! Per spec Section 8.2: "Hot-reload on every keystroke."
//!
//! This module provides the infrastructure for:
//! - Real-time YAML compilation
//! - Debouncing rapid changes
//! - Hot-reload game state
//! - Kid-friendly error display

use std::time::{Duration, Instant};

use crate::compiler::YamlCompiler;
use crate::{CompiledGame, YamlError};

/// Default debounce delay in milliseconds
pub const DEFAULT_DEBOUNCE_MS: u64 = 150;

/// Minimum debounce delay
pub const MIN_DEBOUNCE_MS: u64 = 50;

/// Maximum debounce delay
pub const MAX_DEBOUNCE_MS: u64 = 1000;

/// Debouncer for throttling rapid changes
///
/// Per spec: "Debounce rapid changes"
#[derive(Debug, Clone)]
pub struct Debouncer {
    /// Delay before executing
    delay: Duration,
    /// Last scheduled time
    last_scheduled: Option<Instant>,
    /// Whether a call is pending
    pending: bool,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(Duration::from_millis(DEFAULT_DEBOUNCE_MS))
    }
}

impl Debouncer {
    /// Create a new debouncer with the given delay
    #[must_use]
    pub const fn new(delay: Duration) -> Self {
        Self {
            delay,
            last_scheduled: None,
            pending: false,
        }
    }

    /// Create a debouncer with delay in milliseconds
    #[must_use]
    pub fn from_millis(ms: u64) -> Self {
        let clamped = ms.clamp(MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS);
        Self::new(Duration::from_millis(clamped))
    }

    /// Schedule a call, returns true if it should execute now
    pub fn schedule(&mut self) -> bool {
        let now = Instant::now();

        match self.last_scheduled {
            Some(last) if now.duration_since(last) < self.delay => {
                // Within debounce window, mark as pending
                self.pending = true;
                false
            }
            _ => {
                // Execute now
                self.last_scheduled = Some(now);
                self.pending = false;
                true
            }
        }
    }

    /// Check if there's a pending call that should execute now
    pub fn check_pending(&mut self) -> bool {
        if !self.pending {
            return false;
        }

        let now = Instant::now();
        if let Some(last) = self.last_scheduled {
            if now.duration_since(last) >= self.delay {
                self.last_scheduled = Some(now);
                self.pending = false;
                return true;
            }
        }

        false
    }

    /// Reset the debouncer
    pub fn reset(&mut self) {
        self.last_scheduled = None;
        self.pending = false;
    }

    /// Get the current delay
    #[must_use]
    pub const fn delay(&self) -> Duration {
        self.delay
    }

    /// Set a new delay
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    /// Check if there's a pending call
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending
    }
}

/// Result of a preview compilation attempt
#[derive(Debug, Clone)]
pub enum PreviewResult {
    /// Compilation succeeded
    Success {
        /// The compiled game
        game: CompiledGame,
        /// Compilation time
        compile_time: Duration,
    },
    /// Compilation failed
    Error {
        /// The errors that occurred
        errors: Vec<YamlError>,
    },
    /// Debounced - not compiled yet
    Debounced,
}

impl PreviewResult {
    /// Check if compilation succeeded
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if compilation failed
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if the call was debounced
    #[must_use]
    pub const fn is_debounced(&self) -> bool {
        matches!(self, Self::Debounced)
    }

    /// Get the compiled game if successful
    #[must_use]
    pub fn game(&self) -> Option<&CompiledGame> {
        match self {
            Self::Success { game, .. } => Some(game),
            _ => None,
        }
    }

    /// Get the errors if failed
    #[must_use]
    pub fn errors(&self) -> Option<&[YamlError]> {
        match self {
            Self::Error { errors } => Some(errors),
            _ => None,
        }
    }
}

/// Callback for preview events
pub trait PreviewCallback {
    /// Called when compilation succeeds
    fn on_success(&mut self, game: &CompiledGame, compile_time: Duration);

    /// Called when compilation fails
    fn on_error(&mut self, errors: &[YamlError]);
}

/// Live Preview system for real-time YAML hot-reload
///
/// Per spec Section 8.2: Handles hot-reload on every keystroke with debouncing.
#[derive(Debug)]
pub struct LivePreview {
    /// YAML compiler
    compiler: YamlCompiler,
    /// Debouncer for rapid changes
    debouncer: Debouncer,
    /// Last successfully compiled game
    last_valid_game: Option<CompiledGame>,
    /// Last compilation errors
    last_errors: Vec<YamlError>,
    /// Total compilations
    compilation_count: u64,
    /// Successful compilations
    success_count: u64,
}

impl Default for LivePreview {
    fn default() -> Self {
        Self::new()
    }
}

impl LivePreview {
    /// Create a new LivePreview with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            compiler: YamlCompiler::new(),
            debouncer: Debouncer::default(),
            last_valid_game: None,
            last_errors: Vec::new(),
            compilation_count: 0,
            success_count: 0,
        }
    }

    /// Create a LivePreview with custom debounce delay
    #[must_use]
    pub fn with_debounce(delay_ms: u64) -> Self {
        Self {
            compiler: YamlCompiler::new(),
            debouncer: Debouncer::from_millis(delay_ms),
            last_valid_game: None,
            last_errors: Vec::new(),
            compilation_count: 0,
            success_count: 0,
        }
    }

    /// Handle YAML change event
    ///
    /// Per spec: "Hot-reload on every keystroke"
    pub fn on_yaml_change(&mut self, yaml: &str) -> PreviewResult {
        // Check debounce
        if !self.debouncer.schedule() {
            return PreviewResult::Debounced;
        }

        self.compile_and_update(yaml)
    }

    /// Force immediate compilation (bypasses debounce)
    pub fn compile_now(&mut self, yaml: &str) -> PreviewResult {
        self.debouncer.reset();
        self.compile_and_update(yaml)
    }

    /// Check and handle any pending debounced calls
    pub fn check_pending(&mut self, yaml: &str) -> Option<PreviewResult> {
        if self.debouncer.check_pending() {
            Some(self.compile_and_update(yaml))
        } else {
            None
        }
    }

    /// Internal compilation logic
    fn compile_and_update(&mut self, yaml: &str) -> PreviewResult {
        let start = Instant::now();
        self.compilation_count += 1;

        match self.compiler.compile(yaml) {
            Ok(game) => {
                let compile_time = start.elapsed();
                self.last_valid_game = Some(game.clone());
                self.last_errors.clear();
                self.success_count += 1;

                PreviewResult::Success { game, compile_time }
            }
            Err(error) => {
                self.last_errors = vec![error];

                PreviewResult::Error {
                    errors: self.last_errors.clone(),
                }
            }
        }
    }

    /// Get the last successfully compiled game
    #[must_use]
    pub fn last_valid_game(&self) -> Option<&CompiledGame> {
        self.last_valid_game.as_ref()
    }

    /// Get the last compilation errors
    #[must_use]
    pub fn last_errors(&self) -> &[YamlError] {
        &self.last_errors
    }

    /// Get total compilation count
    #[must_use]
    pub const fn compilation_count(&self) -> u64 {
        self.compilation_count
    }

    /// Get successful compilation count
    #[must_use]
    pub const fn success_count(&self) -> u64 {
        self.success_count
    }

    /// Get success rate (0.0 to 1.0)
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.compilation_count == 0 {
            1.0
        } else {
            self.success_count as f64 / self.compilation_count as f64
        }
    }

    /// Reset preview state
    pub fn reset(&mut self) {
        self.debouncer.reset();
        self.last_valid_game = None;
        self.last_errors.clear();
        self.compilation_count = 0;
        self.success_count = 0;
    }

    /// Check if there are pending changes
    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.debouncer.is_pending()
    }

    /// Get the debounce delay
    #[must_use]
    pub fn debounce_delay(&self) -> Duration {
        self.debouncer.delay()
    }

    /// Set the debounce delay
    pub fn set_debounce_delay(&mut self, delay_ms: u64) {
        self.debouncer.set_delay(Duration::from_millis(
            delay_ms.clamp(MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS),
        ));
    }
}

/// Preview status for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewStatus {
    /// Ready, no pending changes
    Ready,
    /// Compiling
    Compiling,
    /// Has pending debounced changes
    Pending,
    /// Last compilation succeeded
    Success,
    /// Last compilation failed
    Error,
}

/// Preview statistics for metrics
#[derive(Debug, Clone, Copy)]
pub struct PreviewStats {
    /// Total compilations
    pub total_compilations: u64,
    /// Successful compilations
    pub successful_compilations: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average compile time (if tracked)
    pub avg_compile_time_ms: Option<f64>,
}

impl From<&LivePreview> for PreviewStats {
    fn from(preview: &LivePreview) -> Self {
        Self {
            total_compilations: preview.compilation_count(),
            successful_compilations: preview.success_count(),
            success_rate: preview.success_rate(),
            avg_compile_time_ms: None, // Would need tracking to implement
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::thread;

    // ========================================================================
    // EXTREME TDD: Tests for LivePreview per Section 8.2
    // ========================================================================

    mod debouncer_tests {
        use super::*;

        #[test]
        fn test_debouncer_default() {
            let debouncer = Debouncer::default();
            assert_eq!(
                debouncer.delay(),
                Duration::from_millis(DEFAULT_DEBOUNCE_MS)
            );
            assert!(!debouncer.is_pending());
        }

        #[test]
        fn test_debouncer_from_millis() {
            let debouncer = Debouncer::from_millis(200);
            assert_eq!(debouncer.delay(), Duration::from_millis(200));
        }

        #[test]
        fn test_debouncer_clamps_min() {
            let debouncer = Debouncer::from_millis(10); // Below min
            assert_eq!(debouncer.delay(), Duration::from_millis(MIN_DEBOUNCE_MS));
        }

        #[test]
        fn test_debouncer_clamps_max() {
            let debouncer = Debouncer::from_millis(5000); // Above max
            assert_eq!(debouncer.delay(), Duration::from_millis(MAX_DEBOUNCE_MS));
        }

        #[test]
        fn test_debouncer_first_call_executes() {
            let mut debouncer = Debouncer::default();
            assert!(debouncer.schedule());
        }

        #[test]
        fn test_debouncer_rapid_calls_debounced() {
            let mut debouncer = Debouncer::from_millis(100);
            assert!(debouncer.schedule()); // First executes
            assert!(!debouncer.schedule()); // Second debounced
            assert!(debouncer.is_pending());
        }

        #[test]
        fn test_debouncer_reset() {
            let mut debouncer = Debouncer::default();
            let _ = debouncer.schedule();
            let _ = debouncer.schedule();
            debouncer.reset();
            assert!(!debouncer.is_pending());
            assert!(debouncer.schedule()); // Should execute after reset
        }

        #[test]
        fn test_debouncer_after_delay() {
            let mut debouncer = Debouncer::from_millis(MIN_DEBOUNCE_MS);
            assert!(debouncer.schedule());

            // Wait for debounce delay
            thread::sleep(Duration::from_millis(MIN_DEBOUNCE_MS + 10));

            assert!(debouncer.schedule()); // Should execute after delay
        }
    }

    mod preview_result_tests {
        use super::*;
        use crate::SchemaLevel;

        fn mock_game() -> CompiledGame {
            CompiledGame {
                name: "test".to_string(),
                level: SchemaLevel::Level1,
                entities: Vec::new(),
                rules: Vec::new(),
                background: None,
                music: None,
            }
        }

        #[test]
        fn test_success_result() {
            let result = PreviewResult::Success {
                game: mock_game(),
                compile_time: Duration::from_millis(10),
            };
            assert!(result.is_success());
            assert!(!result.is_error());
            assert!(!result.is_debounced());
            assert!(result.game().is_some());
        }

        #[test]
        fn test_error_result() {
            let result = PreviewResult::Error {
                errors: vec![YamlError::SyntaxError {
                    message: "test error".to_string(),
                    line: Some(1),
                    column: Some(1),
                }],
            };
            assert!(!result.is_success());
            assert!(result.is_error());
            assert!(result.errors().is_some());
        }

        #[test]
        fn test_debounced_result() {
            let result = PreviewResult::Debounced;
            assert!(result.is_debounced());
            assert!(result.game().is_none());
            assert!(result.errors().is_none());
        }
    }

    mod live_preview_tests {
        use super::*;

        #[test]
        fn test_live_preview_new() {
            let preview = LivePreview::new();
            assert_eq!(preview.compilation_count(), 0);
            assert_eq!(preview.success_count(), 0);
            assert!(preview.last_valid_game().is_none());
        }

        #[test]
        fn test_live_preview_with_debounce() {
            let preview = LivePreview::with_debounce(200);
            assert_eq!(preview.debounce_delay(), Duration::from_millis(200));
        }

        #[test]
        fn test_on_yaml_change_success() {
            let mut preview = LivePreview::new();
            let yaml = "character: bunny";

            let result = preview.on_yaml_change(yaml);
            assert!(result.is_success());
            assert!(preview.last_valid_game().is_some());
            assert_eq!(preview.compilation_count(), 1);
            assert_eq!(preview.success_count(), 1);
        }

        #[test]
        fn test_on_yaml_change_error() {
            let mut preview = LivePreview::new();
            let yaml = "character: dinosaur"; // Invalid character

            let result = preview.on_yaml_change(yaml);
            assert!(result.is_error());
            assert!(!preview.last_errors().is_empty());
        }

        #[test]
        fn test_on_yaml_change_debounced() {
            let mut preview = LivePreview::with_debounce(100);
            let yaml = "character: bunny";

            let result1 = preview.on_yaml_change(yaml);
            assert!(result1.is_success());

            let result2 = preview.on_yaml_change(yaml);
            assert!(result2.is_debounced());

            // Compilation count should still be 1
            assert_eq!(preview.compilation_count(), 1);
        }

        #[test]
        fn test_compile_now_bypasses_debounce() {
            let mut preview = LivePreview::with_debounce(100);
            let yaml = "character: bunny";

            let _ = preview.on_yaml_change(yaml);
            let result = preview.compile_now(yaml);

            assert!(result.is_success());
            assert_eq!(preview.compilation_count(), 2);
        }

        #[test]
        fn test_keeps_last_valid_game_on_error() {
            let mut preview = LivePreview::new();

            // First, compile a valid game
            let _ = preview.compile_now("character: bunny");
            assert!(preview.last_valid_game().is_some());

            // Then, compile an invalid one
            let _ = preview.compile_now("character: dinosaur");

            // Should still have the last valid game
            assert!(preview.last_valid_game().is_some());
            assert_eq!(preview.last_valid_game().unwrap().entities.len(), 1);
        }

        #[test]
        fn test_success_rate() {
            let mut preview = LivePreview::new();

            let _ = preview.compile_now("character: bunny");
            let _ = preview.compile_now("character: cat");
            let _ = preview.compile_now("character: dinosaur"); // Error

            assert_eq!(preview.compilation_count(), 3);
            assert_eq!(preview.success_count(), 2);
            assert!((preview.success_rate() - 0.666).abs() < 0.01);
        }

        #[test]
        fn test_reset() {
            let mut preview = LivePreview::new();
            let _ = preview.compile_now("character: bunny");

            preview.reset();

            assert_eq!(preview.compilation_count(), 0);
            assert_eq!(preview.success_count(), 0);
            assert!(preview.last_valid_game().is_none());
            assert!(preview.last_errors().is_empty());
        }

        #[test]
        fn test_set_debounce_delay() {
            let mut preview = LivePreview::new();
            preview.set_debounce_delay(300);
            assert_eq!(preview.debounce_delay(), Duration::from_millis(300));
        }
    }

    mod preview_stats_tests {
        use super::*;

        #[test]
        fn test_preview_stats_from() {
            let mut preview = LivePreview::new();
            let _ = preview.compile_now("character: bunny");
            let _ = preview.compile_now("character: cat");

            let stats: PreviewStats = (&preview).into();

            assert_eq!(stats.total_compilations, 2);
            assert_eq!(stats.successful_compilations, 2);
            assert!((stats.success_rate - 1.0).abs() < f64::EPSILON);
        }
    }

    mod constants_tests {
        use super::*;

        #[test]
        fn test_default_debounce() {
            assert_eq!(DEFAULT_DEBOUNCE_MS, 150);
        }

        #[test]
        fn test_min_debounce() {
            assert_eq!(MIN_DEBOUNCE_MS, 50);
        }

        #[test]
        fn test_max_debounce() {
            assert_eq!(MAX_DEBOUNCE_MS, 1000);
        }
    }
}
