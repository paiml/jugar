//! Complexity Thermometer - Mieruka (Visual Control)
//!
//! Provides visual feedback when scene complexity approaches performance limits,
//! enabling users to self-correct before the simulation degrades.
//!
//! > "The user must know immediately if they are exceeding the system's capacity
//! > to maintain real-time fidelity." — Respect for People principle

use serde::{Deserialize, Serialize};

/// Rolling average calculator for frame times
#[derive(Debug, Clone)]
pub struct RollingAverage<const N: usize> {
    values: [f32; N],
    index: usize,
    count: usize,
}

impl<const N: usize> Default for RollingAverage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> RollingAverage<N> {
    /// Create a new rolling average calculator
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: [0.0; N],
            index: 0,
            count: 0,
        }
    }

    /// Add a new value to the rolling average
    pub fn push(&mut self, value: f32) {
        self.values[self.index] = value;
        self.index = (self.index + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// Get the current average
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn average(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: f32 = self.values[..self.count].iter().sum();
        sum / self.count as f32
    }

    /// Reset the rolling average
    pub fn reset(&mut self) {
        self.values = [0.0; N];
        self.index = 0;
        self.count = 0;
    }

    /// Get the number of samples collected
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.count
    }
}

/// Per-subsystem performance breakdown
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PerformanceBreakdown {
    /// Physics step time in milliseconds
    pub physics_ms: f32,

    /// Render time in milliseconds
    pub render_ms: f32,

    /// UI update time in milliseconds
    pub ui_ms: f32,

    /// Other overhead in milliseconds
    pub other_ms: f32,
}

impl PerformanceBreakdown {
    /// Total frame time
    #[must_use]
    pub fn total(&self) -> f32 {
        self.physics_ms + self.render_ms + self.ui_ms + self.other_ms
    }

    /// Get the dominant subsystem
    #[must_use]
    pub fn dominant_subsystem(&self) -> &'static str {
        let max = self
            .physics_ms
            .max(self.render_ms)
            .max(self.ui_ms)
            .max(self.other_ms);

        if (max - self.physics_ms).abs() < f32::EPSILON {
            "physics"
        } else if (max - self.render_ms).abs() < f32::EPSILON {
            "render"
        } else if (max - self.ui_ms).abs() < f32::EPSILON {
            "ui"
        } else {
            "other"
        }
    }
}

/// Visual state for the thermometer UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermometerState {
    /// < 70% budget consumed - normal operation
    Green,
    /// 70-90% budget consumed - warning indicator
    Yellow,
    /// > 90% budget consumed - critical, additions blocked
    Red,
}

impl ThermometerState {
    /// Get CSS color for this state
    #[must_use]
    pub const fn css_color(&self) -> &'static str {
        match self {
            Self::Green => "#22c55e",  // Tailwind green-500
            Self::Yellow => "#eab308", // Tailwind yellow-500
            Self::Red => "#ef4444",    // Tailwind red-500
        }
    }

    /// Get description for this state
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Green => "Normal operation, all features enabled",
            Self::Yellow => "Warning: approaching performance limit",
            Self::Red => "Critical: object additions blocked",
        }
    }
}

/// Performance Mieruka: Real-time complexity visualization
///
/// Provides visual feedback to users about scene complexity,
/// implementing the Toyota Way "Respect for People" principle
/// by giving users agency to self-correct.
#[derive(Debug, Clone)]
pub struct ComplexityThermometer {
    /// Current load as ratio of budget consumed (0.0 - 1.0+)
    load: f32,

    /// Rolling average of frame times (60 samples = 1 second at 60fps)
    frame_time_avg: RollingAverage<60>,

    /// Per-subsystem breakdown for debugging
    breakdown: PerformanceBreakdown,

    /// Target frames per second
    target_fps: f32,

    /// Green threshold (below this is green)
    green_threshold: f32,

    /// Yellow threshold (below this is yellow, above is red)
    yellow_threshold: f32,
}

impl Default for ComplexityThermometer {
    fn default() -> Self {
        Self::new(60.0)
    }
}

impl ComplexityThermometer {
    /// Create a new thermometer with target FPS
    #[must_use]
    pub fn new(target_fps: f32) -> Self {
        Self {
            load: 0.0,
            frame_time_avg: RollingAverage::new(),
            breakdown: PerformanceBreakdown::default(),
            target_fps,
            green_threshold: 0.7,
            yellow_threshold: 0.9,
        }
    }

    /// Update with frame timing measurements
    pub fn update(&mut self, breakdown: PerformanceBreakdown) {
        self.breakdown = breakdown;
        self.frame_time_avg.push(breakdown.total());
        self.load = self.load_factor();
    }

    /// Calculate load factor: Load = `T_total` / `T_budget`
    #[must_use]
    pub fn load_factor(&self) -> f32 {
        let budget_ms = 1000.0 / self.target_fps;
        self.frame_time_avg.average() / budget_ms
    }

    /// Get current load (0.0 - 1.0+)
    #[must_use]
    pub const fn load(&self) -> f32 {
        self.load
    }

    /// Get current breakdown
    #[must_use]
    pub const fn breakdown(&self) -> &PerformanceBreakdown {
        &self.breakdown
    }

    /// Get target FPS
    #[must_use]
    pub const fn target_fps(&self) -> f32 {
        self.target_fps
    }

    /// Get budget in milliseconds
    #[must_use]
    pub fn budget_ms(&self) -> f32 {
        1000.0 / self.target_fps
    }

    /// Visual state for UI rendering
    #[must_use]
    pub fn visual_state(&self) -> ThermometerState {
        if self.load < self.green_threshold {
            ThermometerState::Green
        } else if self.load < self.yellow_threshold {
            ThermometerState::Yellow
        } else {
            ThermometerState::Red
        }
    }

    /// POKA-YOKE: Should the "Add Object" button be disabled?
    #[must_use]
    pub fn should_block_additions(&self) -> bool {
        self.load > self.yellow_threshold
    }

    /// Get load as percentage (0-100+)
    #[must_use]
    pub fn load_percent(&self) -> f32 {
        self.load * 100.0
    }

    /// Reset the thermometer
    pub fn reset(&mut self) {
        self.load = 0.0;
        self.frame_time_avg.reset();
        self.breakdown = PerformanceBreakdown::default();
    }

    /// Set custom thresholds
    pub fn set_thresholds(&mut self, green: f32, yellow: f32) {
        self.green_threshold = green.clamp(0.0, 1.0);
        self.yellow_threshold = yellow.clamp(self.green_threshold, 1.0);
    }

    /// Format for display
    #[must_use]
    pub fn format_display(&self) -> String {
        format!(
            "Load: {:.0}% | Physics: {:.1}ms | Render: {:.1}ms | UI: {:.1}ms | Budget: {:.1}ms",
            self.load_percent(),
            self.breakdown.physics_ms,
            self.breakdown.render_ms,
            self.breakdown.ui_ms,
            self.budget_ms()
        )
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default,
    unused_results
)]
mod tests {
    use super::*;

    // =========================================================================
    // EXTREME TDD: Complexity Thermometer tests from specification
    // =========================================================================

    mod rolling_average_tests {
        use super::*;

        #[test]
        fn test_empty_average_is_zero() {
            let avg = RollingAverage::<10>::new();
            assert!((avg.average() - 0.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_single_value() {
            let mut avg = RollingAverage::<10>::new();
            avg.push(5.0);
            assert!((avg.average() - 5.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_multiple_values() {
            let mut avg = RollingAverage::<10>::new();
            avg.push(1.0);
            avg.push(2.0);
            avg.push(3.0);
            assert!((avg.average() - 2.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_rolling_window() {
            let mut avg = RollingAverage::<3>::new();
            avg.push(1.0);
            avg.push(2.0);
            avg.push(3.0);
            // Window: [1, 2, 3] => avg = 2.0
            assert!((avg.average() - 2.0).abs() < f32::EPSILON);

            avg.push(6.0);
            // Window: [6, 2, 3] => avg = 11/3 ≈ 3.67
            assert!((avg.average() - 11.0 / 3.0).abs() < 0.01);
        }

        #[test]
        fn test_reset() {
            let mut avg = RollingAverage::<10>::new();
            avg.push(5.0);
            avg.push(10.0);
            avg.reset();
            assert!((avg.average() - 0.0).abs() < f32::EPSILON);
            assert_eq!(avg.sample_count(), 0);
        }
    }

    mod thermometer_state_tests {
        use super::*;

        #[test]
        fn test_green_below_70_percent() {
            let mut thermo = ComplexityThermometer::default();
            // Simulate 50% load (8.33ms of 16.67ms budget at 60fps)
            let breakdown = PerformanceBreakdown {
                physics_ms: 4.0,
                render_ms: 3.0,
                ui_ms: 1.33,
                other_ms: 0.0,
            };
            thermo.update(breakdown);
            assert_eq!(thermo.visual_state(), ThermometerState::Green);
        }

        #[test]
        fn test_yellow_between_70_and_90_percent() {
            let mut thermo = ComplexityThermometer::default();
            // Simulate 80% load (13.33ms of 16.67ms budget at 60fps)
            for _ in 0..10 {
                let breakdown = PerformanceBreakdown {
                    physics_ms: 6.0,
                    render_ms: 5.0,
                    ui_ms: 2.33,
                    other_ms: 0.0,
                };
                thermo.update(breakdown);
            }
            assert_eq!(thermo.visual_state(), ThermometerState::Yellow);
        }

        #[test]
        fn test_red_above_90_percent() {
            let mut thermo = ComplexityThermometer::default();
            // Simulate 95% load (15.83ms of 16.67ms budget at 60fps)
            for _ in 0..10 {
                let breakdown = PerformanceBreakdown {
                    physics_ms: 8.0,
                    render_ms: 6.0,
                    ui_ms: 1.83,
                    other_ms: 0.0,
                };
                thermo.update(breakdown);
            }
            assert_eq!(thermo.visual_state(), ThermometerState::Red);
        }
    }

    mod blocking_tests {
        use super::*;

        #[test]
        fn test_thermometer_blocks_at_90_percent() {
            let mut thermo = ComplexityThermometer::default();
            thermo.load = 0.85;
            assert!(!thermo.should_block_additions());

            thermo.load = 0.91;
            assert!(thermo.should_block_additions());
        }

        #[test]
        fn test_exactly_at_threshold() {
            let mut thermo = ComplexityThermometer::default();
            thermo.load = 0.9;
            assert!(!thermo.should_block_additions());

            thermo.load = 0.90001;
            assert!(thermo.should_block_additions());
        }
    }

    mod breakdown_tests {
        use super::*;

        #[test]
        fn test_breakdown_total() {
            let breakdown = PerformanceBreakdown {
                physics_ms: 4.0,
                render_ms: 8.0,
                ui_ms: 2.0,
                other_ms: 1.0,
            };
            assert!((breakdown.total() - 15.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_dominant_subsystem_physics() {
            let breakdown = PerformanceBreakdown {
                physics_ms: 10.0,
                render_ms: 5.0,
                ui_ms: 2.0,
                other_ms: 1.0,
            };
            assert_eq!(breakdown.dominant_subsystem(), "physics");
        }

        #[test]
        fn test_dominant_subsystem_render() {
            let breakdown = PerformanceBreakdown {
                physics_ms: 5.0,
                render_ms: 10.0,
                ui_ms: 2.0,
                other_ms: 1.0,
            };
            assert_eq!(breakdown.dominant_subsystem(), "render");
        }
    }

    mod display_tests {
        use super::*;

        #[test]
        fn test_format_display() {
            let mut thermo = ComplexityThermometer::new(60.0);
            let breakdown = PerformanceBreakdown {
                physics_ms: 4.0,
                render_ms: 6.0,
                ui_ms: 2.0,
                other_ms: 0.0,
            };
            thermo.update(breakdown);

            let display = thermo.format_display();
            assert!(display.contains("Physics:"));
            assert!(display.contains("Render:"));
            assert!(display.contains("UI:"));
            assert!(display.contains("Budget:"));
        }

        #[test]
        fn test_state_colors() {
            assert!(ThermometerState::Green.css_color().starts_with('#'));
            assert!(ThermometerState::Yellow.css_color().starts_with('#'));
            assert!(ThermometerState::Red.css_color().starts_with('#'));
        }

        #[test]
        fn test_state_descriptions() {
            assert!(!ThermometerState::Green.description().is_empty());
            assert!(!ThermometerState::Yellow.description().is_empty());
            assert!(!ThermometerState::Red.description().is_empty());
        }
    }

    mod configuration_tests {
        use super::*;

        #[test]
        fn test_custom_thresholds() {
            let mut thermo = ComplexityThermometer::default();
            thermo.set_thresholds(0.5, 0.8);
            thermo.load = 0.6;
            assert_eq!(thermo.visual_state(), ThermometerState::Yellow);
        }

        #[test]
        fn test_threshold_clamping() {
            let mut thermo = ComplexityThermometer::default();
            thermo.set_thresholds(1.5, 2.0); // Invalid values
            assert!(thermo.green_threshold <= 1.0);
            assert!(thermo.yellow_threshold <= 1.0);
        }

        #[test]
        fn test_30fps_target() {
            let thermo = ComplexityThermometer::new(30.0);
            assert!((thermo.budget_ms() - 33.33).abs() < 0.1);
        }

        #[test]
        fn test_reset_clears_state() {
            let mut thermo = ComplexityThermometer::default();
            let breakdown = PerformanceBreakdown {
                physics_ms: 10.0,
                render_ms: 10.0,
                ui_ms: 5.0,
                other_ms: 0.0,
            };
            thermo.update(breakdown);
            assert!(thermo.load() > 0.0);

            thermo.reset();
            assert!((thermo.load() - 0.0).abs() < f32::EPSILON);
        }
    }
}
