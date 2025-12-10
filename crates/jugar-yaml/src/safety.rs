//! Photosensitivity protection for safe game creation.
//!
//! Per spec Section 9.3: Protect against seizure-inducing content.
//! Based on WCAG 2.1 guidelines for photosensitive seizure prevention.

/// Maximum flash rate in Hz (WCAG 2.1 guideline)
pub const MAX_FLASH_RATE_HZ: f32 = 3.0;

/// Maximum red flash intensity threshold (0.0-1.0)
pub const MAX_RED_FLASH_THRESHOLD: f32 = 0.8;

/// Maximum screen area that can flash (25% per WCAG)
pub const MAX_FLASH_AREA_PERCENT: f32 = 0.25;

/// Photosensitivity guard for runtime frame validation
#[derive(Debug, Clone)]
pub struct PhotosensitivityGuard {
    /// Maximum flash rate in Hz
    pub max_flash_rate: f32,
    /// Maximum red flash intensity (0.0-1.0)
    pub max_red_flash: f32,
    /// Maximum screen area that can flash (0.0-1.0)
    pub max_area_flash: f32,
    /// Whether reduced motion is preferred
    pub reduced_motion: bool,
    /// Flash history for rate detection (timestamps in seconds)
    flash_history: Vec<f32>,
}

impl Default for PhotosensitivityGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PhotosensitivityGuard {
    /// Create a new guard with WCAG-compliant defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_flash_rate: MAX_FLASH_RATE_HZ,
            max_red_flash: MAX_RED_FLASH_THRESHOLD,
            max_area_flash: MAX_FLASH_AREA_PERCENT,
            reduced_motion: false,
            flash_history: Vec::with_capacity(10),
        }
    }

    /// Create a guard with reduced motion enabled
    #[must_use]
    pub const fn with_reduced_motion(mut self) -> Self {
        self.reduced_motion = true;
        self
    }

    /// Validate a frame transition for photosensitivity issues
    ///
    /// Analyzes brightness changes between frames to detect potentially
    /// harmful flashing patterns.
    #[must_use]
    pub fn validate_frame(&mut self, prev: &Frame, curr: &Frame, time: f32) -> SafetyResult {
        let flash_info = self.detect_flash(prev, curr);

        // Record flash if detected
        if flash_info.is_flash {
            self.flash_history.push(time);
            // Keep only recent history (last second)
            self.flash_history.retain(|&t| time - t < 1.0);
        }

        // Check flash rate (flashes per second)
        // Note: precision loss acceptable for this use case (few flashes per second)
        #[allow(clippy::cast_precision_loss)]
        let flash_rate = self.flash_history.len() as f32;
        if flash_rate > self.max_flash_rate {
            return SafetyResult::Warning(SafetyWarning {
                message: "Slowing down flashing to protect your eyes! 👀".to_string(),
                mitigation: Mitigation::SlowAnimation,
            });
        }

        // Check red flash intensity
        if flash_info.red_intensity > self.max_red_flash {
            return SafetyResult::Block(SafetyBlock {
                message: "Reducing red flashing for safety 🛡️".to_string(),
                mitigation: Mitigation::ReduceRedFlash,
            });
        }

        // Check flash area
        if flash_info.area_percent > self.max_area_flash {
            return SafetyResult::Warning(SafetyWarning {
                message: "Large area flash detected, dimming effect ✨".to_string(),
                mitigation: Mitigation::ReduceArea,
            });
        }

        SafetyResult::Ok
    }

    /// Detect flash characteristics between two frames
    #[must_use]
    pub fn detect_flash(&self, prev: &Frame, curr: &Frame) -> FlashInfo {
        // Calculate luminance change
        let prev_luminance = prev.average_luminance();
        let curr_luminance = curr.average_luminance();
        let luminance_change = (curr_luminance - prev_luminance).abs();

        // Flash threshold: >10% luminance change is considered a flash
        let is_flash = luminance_change > 0.1;

        // Calculate red intensity change
        let prev_red = prev.average_red();
        let curr_red = curr.average_red();
        let red_intensity = if is_flash {
            (curr_red - prev_red).abs().max(curr_red)
        } else {
            0.0
        };

        // Calculate affected area (simplified: assume full frame for now)
        let area_percent = if is_flash { 1.0 } else { 0.0 };

        FlashInfo {
            is_flash,
            luminance_change,
            red_intensity,
            area_percent,
            rate: 0.0, // Calculated separately from history
        }
    }

    /// Apply reduced motion settings to a game configuration
    pub const fn apply_reduced_motion(&self, config: &mut ReducedMotionConfig) {
        if self.reduced_motion {
            config.animation_scale = 0.3;
            config.screen_shake_enabled = false;
            config.particle_effects_enabled = false;
            config.flash_effects_enabled = false;
        }
    }

    /// Check if prefers-reduced-motion is set
    #[must_use]
    pub const fn prefers_reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// Clear flash history (call when game state resets)
    pub fn reset(&mut self) {
        self.flash_history.clear();
    }
}

/// Result of a safety validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyResult {
    /// Frame is safe
    Ok,
    /// Warning: should mitigate but can continue
    Warning(SafetyWarning),
    /// Block: must apply mitigation before continuing
    Block(SafetyBlock),
}

impl SafetyResult {
    /// Check if the result is ok
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if the result is a warning
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Check if the result is blocked
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

/// Safety warning with mitigation suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyWarning {
    /// Kid-friendly warning message
    pub message: String,
    /// Suggested mitigation action
    pub mitigation: Mitigation,
}

/// Safety block requiring immediate mitigation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyBlock {
    /// Kid-friendly block message
    pub message: String,
    /// Required mitigation action
    pub mitigation: Mitigation,
}

/// Mitigation actions for safety issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mitigation {
    /// Slow down animation speed
    SlowAnimation,
    /// Reduce red color intensity
    ReduceRedFlash,
    /// Reduce affected screen area
    ReduceArea,
    /// Disable all flash effects
    DisableFlash,
}

/// Information about a detected flash
#[derive(Debug, Clone, PartialEq)]
pub struct FlashInfo {
    /// Whether this qualifies as a flash
    pub is_flash: bool,
    /// Change in luminance (0.0-1.0)
    pub luminance_change: f32,
    /// Red channel intensity (0.0-1.0)
    pub red_intensity: f32,
    /// Percentage of screen area affected (0.0-1.0)
    pub area_percent: f32,
    /// Flash rate in Hz (calculated from history)
    pub rate: f32,
}

/// A single frame for analysis
#[derive(Debug, Clone, Default)]
pub struct Frame {
    /// Average red channel value (0.0-1.0)
    pub red: f32,
    /// Average green channel value (0.0-1.0)
    pub green: f32,
    /// Average blue channel value (0.0-1.0)
    pub blue: f32,
}

impl Frame {
    /// Create a new frame with RGB values
    #[must_use]
    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    /// Calculate average luminance using standard weights
    /// Per ITU-R BT.709: Y = 0.2126R + 0.7152G + 0.0722B
    #[must_use]
    pub fn average_luminance(&self) -> f32 {
        0.0722_f32.mul_add(self.blue, 0.2126_f32.mul_add(self.red, 0.7152 * self.green))
    }

    /// Get average red value
    #[must_use]
    pub const fn average_red(&self) -> f32 {
        self.red
    }

    /// Create a black frame
    #[must_use]
    pub const fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Create a white frame
    #[must_use]
    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Create a red frame
    #[must_use]
    pub const fn red() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
}

/// Configuration for reduced motion preferences
#[derive(Debug, Clone)]
pub struct ReducedMotionConfig {
    /// Animation speed multiplier (1.0 = normal, 0.3 = slow)
    pub animation_scale: f32,
    /// Whether screen shake is enabled
    pub screen_shake_enabled: bool,
    /// Whether particle effects are enabled
    pub particle_effects_enabled: bool,
    /// Whether flash effects are enabled
    pub flash_effects_enabled: bool,
}

impl Default for ReducedMotionConfig {
    fn default() -> Self {
        Self {
            animation_scale: 1.0,
            screen_shake_enabled: true,
            particle_effects_enabled: true,
            flash_effects_enabled: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 9.3
    // ========================================================================

    mod constants_tests {
        use super::*;

        #[test]
        fn test_max_flash_rate_is_3hz() {
            // Per WCAG 2.1: max 3 flashes per second
            assert!((MAX_FLASH_RATE_HZ - 3.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_max_flash_area_is_25_percent() {
            // Per spec: 25% of screen area max
            assert!((MAX_FLASH_AREA_PERCENT - 0.25).abs() < f32::EPSILON);
        }

        #[test]
        fn test_red_flash_threshold_exists() {
            // Red flashes are particularly dangerous
            // Use runtime check to validate constant is in valid range
            let threshold = MAX_RED_FLASH_THRESHOLD;
            assert!(threshold > 0.0);
            assert!(threshold <= 1.0);
        }
    }

    mod flash_detection_tests {
        use super::*;

        #[test]
        fn test_detects_black_to_white_flash() {
            let guard = PhotosensitivityGuard::new();
            let prev = Frame::black();
            let curr = Frame::white();

            let flash = guard.detect_flash(&prev, &curr);
            assert!(flash.is_flash);
            assert!(flash.luminance_change > 0.5);
        }

        #[test]
        fn test_no_flash_on_similar_frames() {
            let guard = PhotosensitivityGuard::new();
            let prev = Frame::new(0.5, 0.5, 0.5);
            let curr = Frame::new(0.52, 0.52, 0.52);

            let flash = guard.detect_flash(&prev, &curr);
            assert!(!flash.is_flash);
        }

        #[test]
        fn test_detects_red_flash_intensity() {
            let guard = PhotosensitivityGuard::new();
            let prev = Frame::black();
            let curr = Frame::red();

            let flash = guard.detect_flash(&prev, &curr);
            assert!(flash.is_flash);
            assert!(flash.red_intensity > 0.5);
        }

        #[test]
        fn test_luminance_calculation() {
            let white = Frame::white();
            let black = Frame::black();
            let gray = Frame::new(0.5, 0.5, 0.5);

            assert!(white.average_luminance() > 0.99);
            assert!(black.average_luminance() < 0.01);
            assert!((gray.average_luminance() - 0.5).abs() < 0.01);
        }
    }

    mod rate_limiting_tests {
        use super::*;

        #[test]
        fn test_warns_on_excessive_flash_rate() {
            let mut guard = PhotosensitivityGuard::new();
            let prev = Frame::black();
            let curr = Frame::white();

            // Simulate 4 flashes in 1 second (exceeds 3 Hz limit)
            // Time values: 0.0, 0.2, 0.4, 0.6 seconds
            let times = [0.0_f32, 0.2, 0.4, 0.6];
            for (i, &time) in times.iter().enumerate() {
                let result = guard.validate_frame(&prev, &curr, time);
                if i >= 3 {
                    assert!(
                        result.is_warning() || result.is_blocked(),
                        "Should warn/block on flash #{}",
                        i + 1
                    );
                }
            }
        }

        #[test]
        fn test_allows_normal_flash_rate() {
            let mut guard = PhotosensitivityGuard::new();
            // Increase area limit to test rate limiting specifically
            guard.max_area_flash = 1.0;

            // Use gray frames to avoid triggering red flash detection
            let prev = Frame::new(0.2, 0.2, 0.2);
            let curr = Frame::new(0.8, 0.8, 0.8);

            // Simulate 2 flashes in 1 second (under 3 Hz limit)
            // First flash is ok (only 1 flash)
            let result1 = guard.validate_frame(&prev, &curr, 0.0);
            // Second flash at 0.5s is also ok (2 flashes/sec < 3 Hz)
            let result2 = guard.validate_frame(&prev, &curr, 0.5);

            assert!(result1.is_ok(), "First flash should be ok: {result1:?}");
            assert!(result2.is_ok(), "Second flash should be ok: {result2:?}");
        }

        #[test]
        fn test_flash_history_clears_after_time() {
            let mut guard = PhotosensitivityGuard::new();
            let prev = Frame::black();
            let curr = Frame::white();

            // Flash at t=0
            let _ = guard.validate_frame(&prev, &curr, 0.0);
            assert_eq!(guard.flash_history.len(), 1);

            // Flash at t=2.0 (old one should be cleared)
            let _ = guard.validate_frame(&prev, &curr, 2.0);
            assert_eq!(guard.flash_history.len(), 1); // Only the new one
        }
    }

    mod red_flash_tests {
        use super::*;

        #[test]
        fn test_blocks_intense_red_flash() {
            let mut guard = PhotosensitivityGuard::new();
            guard.max_red_flash = 0.5; // Lower threshold for test

            let prev = Frame::black();
            let curr = Frame::red();

            let result = guard.validate_frame(&prev, &curr, 0.0);
            assert!(result.is_blocked());
        }

        #[test]
        fn test_allows_mild_red() {
            let mut guard = PhotosensitivityGuard::new();
            let prev = Frame::new(0.3, 0.0, 0.0);
            let curr = Frame::new(0.4, 0.0, 0.0);

            let result = guard.validate_frame(&prev, &curr, 0.0);
            assert!(result.is_ok());
        }
    }

    mod reduced_motion_tests {
        use super::*;

        #[test]
        fn test_reduced_motion_config_defaults() {
            let config = ReducedMotionConfig::default();
            assert!((config.animation_scale - 1.0).abs() < f32::EPSILON);
            assert!(config.screen_shake_enabled);
            assert!(config.particle_effects_enabled);
        }

        #[test]
        fn test_apply_reduced_motion() {
            let guard = PhotosensitivityGuard::new().with_reduced_motion();
            let mut config = ReducedMotionConfig::default();

            guard.apply_reduced_motion(&mut config);

            assert!((config.animation_scale - 0.3).abs() < f32::EPSILON);
            assert!(!config.screen_shake_enabled);
            assert!(!config.particle_effects_enabled);
            assert!(!config.flash_effects_enabled);
        }

        #[test]
        fn test_prefers_reduced_motion() {
            let guard = PhotosensitivityGuard::new();
            assert!(!guard.prefers_reduced_motion());

            let guard = PhotosensitivityGuard::new().with_reduced_motion();
            assert!(guard.prefers_reduced_motion());
        }
    }

    mod safety_result_tests {
        use super::*;

        #[test]
        fn test_safety_result_ok() {
            let result = SafetyResult::Ok;
            assert!(result.is_ok());
            assert!(!result.is_warning());
            assert!(!result.is_blocked());
        }

        #[test]
        fn test_safety_result_warning() {
            let result = SafetyResult::Warning(SafetyWarning {
                message: "test".to_string(),
                mitigation: Mitigation::SlowAnimation,
            });
            assert!(!result.is_ok());
            assert!(result.is_warning());
            assert!(!result.is_blocked());
        }

        #[test]
        fn test_safety_result_block() {
            let result = SafetyResult::Block(SafetyBlock {
                message: "test".to_string(),
                mitigation: Mitigation::DisableFlash,
            });
            assert!(!result.is_ok());
            assert!(!result.is_warning());
            assert!(result.is_blocked());
        }
    }

    mod frame_tests {
        use super::*;

        #[test]
        fn test_frame_constructors() {
            let black = Frame::black();
            assert!(black.red < 0.01 && black.green < 0.01 && black.blue < 0.01);

            let white = Frame::white();
            assert!(white.red > 0.99 && white.green > 0.99 && white.blue > 0.99);

            let red = Frame::red();
            assert!(red.red > 0.99 && red.green < 0.01 && red.blue < 0.01);
        }

        #[test]
        fn test_reset_clears_history() {
            let mut guard = PhotosensitivityGuard::new();
            guard.flash_history.push(0.5);
            guard.flash_history.push(1.0);

            guard.reset();
            assert!(guard.flash_history.is_empty());
        }
    }
}
