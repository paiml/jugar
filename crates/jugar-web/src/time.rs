//! Time utilities for browser integration.
//!
//! This module handles conversion from JavaScript `DOMHighResTimeStamp` (milliseconds)
//! to the engine's internal time representation (seconds as f64).

/// Converts a `DOMHighResTimeStamp` (milliseconds from `performance.now()`) to seconds.
///
/// # Arguments
///
/// * `timestamp_ms` - Timestamp in milliseconds from `performance.now()`
///
/// # Returns
///
/// Time in seconds as f64
///
/// # Example
///
/// ```
/// use jugar_web::time::dom_timestamp_to_seconds;
///
/// let ms = 1234.567;
/// let seconds = dom_timestamp_to_seconds(ms);
/// assert!((seconds - 1.234567).abs() < 1e-9);
/// ```
#[must_use]
pub const fn dom_timestamp_to_seconds(timestamp_ms: f64) -> f64 {
    timestamp_ms / 1000.0
}

/// Converts seconds to milliseconds (for passing back to JavaScript if needed).
///
/// # Arguments
///
/// * `seconds` - Time in seconds
///
/// # Returns
///
/// Time in milliseconds as f64
#[must_use]
pub const fn seconds_to_dom_timestamp(seconds: f64) -> f64 {
    seconds * 1000.0
}

/// Calculates delta time from two consecutive `DOMHighResTimeStamp` values.
///
/// # Arguments
///
/// * `current_ms` - Current frame's timestamp in milliseconds
/// * `previous_ms` - Previous frame's timestamp in milliseconds
///
/// # Returns
///
/// Delta time in seconds
///
/// # Example
///
/// ```
/// use jugar_web::time::calculate_delta_time;
///
/// let prev = 1000.0;  // 1 second
/// let curr = 1016.67; // ~16.67ms later (60 FPS)
/// let dt = calculate_delta_time(curr, prev);
/// assert!((dt - 0.01667).abs() < 0.001);
/// ```
#[must_use]
pub const fn calculate_delta_time(current_ms: f64, previous_ms: f64) -> f64 {
    (current_ms - previous_ms) / 1000.0
}

/// Clamps delta time to prevent physics explosions from large time gaps.
///
/// This is important when the tab is backgrounded and then restored,
/// which can produce very large delta times.
///
/// # Arguments
///
/// * `dt` - Raw delta time in seconds
/// * `max_dt` - Maximum allowed delta time in seconds (typical: 0.1 = 100ms)
///
/// # Returns
///
/// Clamped delta time in seconds
#[must_use]
#[allow(clippy::missing_const_for_fn)] // f64::min/max are not const
pub fn clamp_delta_time(dt: f64, max_dt: f64) -> f64 {
    dt.min(max_dt).max(0.0)
}

/// Default maximum delta time (100ms = 10 FPS minimum).
///
/// When delta time exceeds this, the game will slow down rather than
/// trying to simulate a huge time gap.
pub const DEFAULT_MAX_DELTA_TIME: f64 = 0.1;

/// Target delta time for 60 FPS.
pub const TARGET_DT_60FPS: f64 = 1.0 / 60.0;

/// Target delta time for 30 FPS.
pub const TARGET_DT_30FPS: f64 = 1.0 / 30.0;

/// Target delta time for 120 FPS.
pub const TARGET_DT_120FPS: f64 = 1.0 / 120.0;

/// Frame time tracker for computing frame statistics.
#[derive(Debug, Clone)]
pub struct FrameTimer {
    /// Last frame's timestamp in milliseconds
    last_timestamp_ms: Option<f64>,
    /// Accumulated delta time for fixed timestep accumulation
    accumulator: f64,
    /// Fixed timestep interval in seconds
    fixed_dt: f64,
    /// Maximum delta time before clamping
    max_dt: f64,
    /// Total elapsed time since start in seconds
    total_time: f64,
    /// Frame count since start
    frame_count: u64,
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameTimer {
    /// Creates a new frame timer with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_timestamp_ms: None,
            accumulator: 0.0,
            fixed_dt: TARGET_DT_60FPS,
            max_dt: DEFAULT_MAX_DELTA_TIME,
            total_time: 0.0,
            frame_count: 0,
        }
    }

    /// Creates a frame timer with custom fixed timestep.
    #[must_use]
    pub const fn with_fixed_dt(fixed_dt: f64) -> Self {
        Self {
            last_timestamp_ms: None,
            accumulator: 0.0,
            fixed_dt,
            max_dt: DEFAULT_MAX_DELTA_TIME,
            total_time: 0.0,
            frame_count: 0,
        }
    }

    /// Sets the maximum allowed delta time.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn set_max_dt(&mut self, max_dt: f64) {
        self.max_dt = max_dt;
    }

    /// Sets the fixed timestep interval.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn set_fixed_dt(&mut self, fixed_dt: f64) {
        self.fixed_dt = fixed_dt;
    }

    /// Updates the timer with a new frame timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp_ms` - Current timestamp from `requestAnimationFrame` in milliseconds
    ///
    /// # Returns
    ///
    /// The clamped delta time in seconds
    pub fn update(&mut self, timestamp_ms: f64) -> f64 {
        let dt = self
            .last_timestamp_ms
            .map_or(0.0, |last| calculate_delta_time(timestamp_ms, last));

        self.last_timestamp_ms = Some(timestamp_ms);
        let clamped_dt = clamp_delta_time(dt, self.max_dt);
        self.total_time += clamped_dt;
        self.frame_count += 1;
        self.accumulator += clamped_dt;

        clamped_dt
    }

    /// Consumes accumulated time in fixed timestep chunks.
    ///
    /// Call this in a loop to run physics updates at fixed intervals:
    ///
    /// ```ignore
    /// while let Some(fixed_dt) = timer.consume_fixed_step() {
    ///     physics.step(fixed_dt);
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// `Some(fixed_dt)` if enough time has accumulated, `None` otherwise
    pub fn consume_fixed_step(&mut self) -> Option<f64> {
        if self.accumulator >= self.fixed_dt {
            self.accumulator -= self.fixed_dt;
            Some(self.fixed_dt)
        } else {
            None
        }
    }

    /// Returns the interpolation alpha for rendering between physics steps.
    ///
    /// This value (0.0 to 1.0) indicates how far between the last and next
    /// physics step we are, useful for interpolating visual positions.
    #[must_use]
    pub fn interpolation_alpha(&self) -> f64 {
        if self.fixed_dt > 0.0 {
            self.accumulator / self.fixed_dt
        } else {
            0.0
        }
    }

    /// Returns the total elapsed time since the timer was created.
    #[must_use]
    pub const fn total_time(&self) -> f64 {
        self.total_time
    }

    /// Returns the total number of frames processed.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the average frames per second.
    #[must_use]
    pub fn average_fps(&self) -> f64 {
        if self.total_time > 0.0 {
            self.frame_count as f64 / self.total_time
        } else {
            0.0
        }
    }

    /// Returns the remaining accumulator time.
    #[must_use]
    pub const fn accumulator(&self) -> f64 {
        self.accumulator
    }

    /// Returns the fixed timestep interval.
    #[must_use]
    pub const fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }

    /// Resets the timer to its initial state.
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn reset(&mut self) {
        self.last_timestamp_ms = None;
        self.accumulator = 0.0;
        self.total_time = 0.0;
        self.frame_count = 0;
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_lossless,
    clippy::manual_range_contains
)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_timestamp_to_seconds() {
        assert!((dom_timestamp_to_seconds(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((dom_timestamp_to_seconds(1000.0) - 1.0).abs() < f64::EPSILON);
        assert!((dom_timestamp_to_seconds(16.667) - 0.016_667).abs() < 1e-9);
        assert!((dom_timestamp_to_seconds(100_000.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_seconds_to_dom_timestamp() {
        assert!((seconds_to_dom_timestamp(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((seconds_to_dom_timestamp(1.0) - 1000.0).abs() < f64::EPSILON);
        assert!((seconds_to_dom_timestamp(0.016_667) - 16.667).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = 12345.678;
        let seconds = dom_timestamp_to_seconds(original);
        let back = seconds_to_dom_timestamp(seconds);
        assert!((back - original).abs() < 1e-9);
    }

    #[test]
    fn test_calculate_delta_time() {
        // 60 FPS frame
        let dt = calculate_delta_time(1016.667, 1000.0);
        assert!((dt - 0.016_667).abs() < 1e-6);

        // 30 FPS frame
        let dt = calculate_delta_time(1033.333, 1000.0);
        assert!((dt - 0.033_333).abs() < 1e-6);

        // Same timestamp
        let dt = calculate_delta_time(1000.0, 1000.0);
        assert!(dt.abs() < f64::EPSILON);
    }

    #[test]
    fn test_clamp_delta_time() {
        // Normal delta time passes through
        assert!((clamp_delta_time(0.016_667, 0.1) - 0.016_667).abs() < 1e-9);

        // Large delta time gets clamped
        assert!((clamp_delta_time(0.5, 0.1) - 0.1).abs() < f64::EPSILON);

        // Negative delta time becomes zero
        assert!((clamp_delta_time(-0.1, 0.1) - 0.0).abs() < f64::EPSILON);

        // Zero passes through
        assert!(clamp_delta_time(0.0, 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_constants() {
        assert!((DEFAULT_MAX_DELTA_TIME - 0.1).abs() < f64::EPSILON);
        assert!((TARGET_DT_60FPS - 1.0 / 60.0).abs() < 1e-9);
        assert!((TARGET_DT_30FPS - 1.0 / 30.0).abs() < 1e-9);
        assert!((TARGET_DT_120FPS - 1.0 / 120.0).abs() < 1e-9);
    }

    #[test]
    fn test_frame_timer_new() {
        let timer = FrameTimer::new();
        assert!(timer.last_timestamp_ms.is_none());
        assert!(timer.accumulator.abs() < f64::EPSILON);
        assert!((timer.fixed_dt - TARGET_DT_60FPS).abs() < f64::EPSILON);
        assert!((timer.max_dt - DEFAULT_MAX_DELTA_TIME).abs() < f64::EPSILON);
        assert!(timer.total_time.abs() < f64::EPSILON);
        assert_eq!(timer.frame_count, 0);
    }

    #[test]
    fn test_frame_timer_default() {
        let timer = FrameTimer::default();
        assert!(timer.last_timestamp_ms.is_none());
    }

    #[test]
    fn test_frame_timer_with_fixed_dt() {
        let timer = FrameTimer::with_fixed_dt(TARGET_DT_30FPS);
        assert!((timer.fixed_dt - TARGET_DT_30FPS).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_set_max_dt() {
        let mut timer = FrameTimer::new();
        timer.set_max_dt(0.05);
        assert!((timer.max_dt - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_set_fixed_dt() {
        let mut timer = FrameTimer::new();
        timer.set_fixed_dt(TARGET_DT_30FPS);
        assert!((timer.fixed_dt - TARGET_DT_30FPS).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_first_update() {
        let mut timer = FrameTimer::new();
        let dt = timer.update(1000.0);

        // First frame should return 0
        assert!(dt.abs() < f64::EPSILON);
        assert_eq!(timer.frame_count, 1);
        assert!(timer.last_timestamp_ms.is_some());
    }

    #[test]
    fn test_frame_timer_normal_updates() {
        let mut timer = FrameTimer::new();

        // First frame
        let _ = timer.update(0.0);

        // Simulate 60 FPS frames
        let dt1 = timer.update(16.667);
        assert!((dt1 - 0.016_667).abs() < 1e-6);

        let dt2 = timer.update(33.333);
        assert!((dt2 - 0.016_666).abs() < 1e-5);

        assert_eq!(timer.frame_count, 3);
        assert!(timer.total_time > 0.03);
    }

    #[test]
    fn test_frame_timer_clamps_large_dt() {
        let mut timer = FrameTimer::new();
        let _ = timer.update(0.0);

        // Simulate a 500ms gap (tab backgrounded)
        let dt = timer.update(500.0);

        // Should be clamped to max_dt
        assert!((dt - DEFAULT_MAX_DELTA_TIME).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_total_time() {
        let mut timer = FrameTimer::new();
        let _ = timer.update(0.0);
        let _ = timer.update(1000.0); // 1 second later

        assert!((timer.total_time() - 0.1).abs() < f64::EPSILON); // Clamped to 0.1
    }

    #[test]
    fn test_frame_timer_consume_fixed_step() {
        let mut timer = FrameTimer::with_fixed_dt(TARGET_DT_60FPS);
        let _ = timer.update(0.0);

        // Add ~34ms (slightly more than 2 fixed steps to account for float precision)
        let _ = timer.update(34.0);

        // Should be able to consume 2 steps
        let step1 = timer.consume_fixed_step();
        assert!(step1.is_some());
        assert!((step1.unwrap() - TARGET_DT_60FPS).abs() < f64::EPSILON);

        let step2 = timer.consume_fixed_step();
        assert!(step2.is_some());

        // Third step should fail (34ms = 0.034s, 2 steps = ~0.0333s, remainder ~0.0007s)
        let step3 = timer.consume_fixed_step();
        assert!(step3.is_none());
    }

    #[test]
    fn test_frame_timer_interpolation_alpha() {
        let mut timer = FrameTimer::with_fixed_dt(TARGET_DT_60FPS);
        let _ = timer.update(0.0);

        // Add half a fixed step
        timer.accumulator = TARGET_DT_60FPS / 2.0;

        let alpha = timer.interpolation_alpha();
        assert!((alpha - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_frame_timer_interpolation_alpha_zero_fixed_dt() {
        let mut timer = FrameTimer::new();
        timer.fixed_dt = 0.0;

        let alpha = timer.interpolation_alpha();
        assert!(alpha.abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_average_fps() {
        let mut timer = FrameTimer::new();
        let _ = timer.update(0.0);

        // Simulate 60 frames at 60 FPS (1 second total)
        for i in 1..=60 {
            let _ = timer.update((i as f64) * 16.667);
        }

        let fps = timer.average_fps();
        // Should be close to 61 FPS (60 frames + 1 initial in ~1 second)
        assert!(fps > 50.0 && fps < 70.0);
    }

    #[test]
    fn test_frame_timer_average_fps_no_time() {
        let timer = FrameTimer::new();
        let fps = timer.average_fps();
        assert!(fps.abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_accumulator() {
        let mut timer = FrameTimer::new();
        let _ = timer.update(0.0);
        let _ = timer.update(10.0); // 10ms

        assert!((timer.accumulator() - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_frame_timer_fixed_dt_getter() {
        let timer = FrameTimer::with_fixed_dt(0.02);
        assert!((timer.fixed_dt() - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_reset() {
        let mut timer = FrameTimer::new();
        let _ = timer.update(0.0);
        let _ = timer.update(1000.0);

        timer.reset();

        assert!(timer.last_timestamp_ms.is_none());
        assert!(timer.accumulator.abs() < f64::EPSILON);
        assert!(timer.total_time.abs() < f64::EPSILON);
        assert_eq!(timer.frame_count, 0);
        // fixed_dt and max_dt are preserved
        assert!((timer.fixed_dt - TARGET_DT_60FPS).abs() < f64::EPSILON);
    }

    #[test]
    fn test_frame_timer_game_loop_simulation() {
        let mut timer = FrameTimer::with_fixed_dt(TARGET_DT_60FPS);
        let mut physics_steps = 0;

        // Simulate game loop
        let timestamps: [f64; 5] = [0.0, 16.0, 32.0, 48.0, 64.0];

        for &ts in &timestamps {
            let _dt = timer.update(ts);

            // Run physics at fixed timestep
            while timer.consume_fixed_step().is_some() {
                physics_steps += 1;
            }
        }

        // Should have run roughly 4 physics steps for 64ms at 60 FPS
        assert!(physics_steps >= 3 && physics_steps <= 5);
    }
}

#[cfg(test)]
#[allow(clippy::manual_range_contains, clippy::unwrap_used)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Timestamp conversion is bijective (roundtrip)
        #[test]
        fn property_timestamp_roundtrip(ms in 0.0f64..1_000_000.0) {
            let seconds = dom_timestamp_to_seconds(ms);
            let back = seconds_to_dom_timestamp(seconds);
            prop_assert!((ms - back).abs() < 1e-9, "Roundtrip failed: {} -> {} -> {}", ms, seconds, back);
        }

        /// Property: Delta time is always non-negative after clamping
        #[test]
        fn property_clamped_dt_non_negative(dt in -10.0f64..10.0) {
            let clamped = clamp_delta_time(dt, DEFAULT_MAX_DELTA_TIME);
            prop_assert!(clamped >= 0.0, "Clamped dt should be non-negative: {}", clamped);
        }

        /// Property: Clamped delta time never exceeds max
        #[test]
        fn property_clamped_dt_bounded(dt in 0.0f64..1.0, max_dt in 0.001f64..0.5) {
            let clamped = clamp_delta_time(dt, max_dt);
            prop_assert!(clamped <= max_dt, "Clamped {} should be <= {}", clamped, max_dt);
        }

        /// Property: Frame timer total time is monotonically increasing
        #[test]
        fn property_total_time_monotonic(timestamps in proptest::collection::vec(0.0f64..10000.0, 2..20)) {
            let mut timer = FrameTimer::new();
            let mut prev_total = 0.0;

            // Sort timestamps to simulate realistic frame sequence
            let mut sorted = timestamps;
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for ts in sorted {
                let _ = timer.update(ts);
                prop_assert!(timer.total_time >= prev_total,
                    "Total time should be monotonic: {} -> {}", prev_total, timer.total_time);
                prev_total = timer.total_time;
            }
        }

        /// Property: Frame count always increments
        #[test]
        fn property_frame_count_increments(n in 1usize..100) {
            let mut timer = FrameTimer::new();

            for i in 0..n {
                let _ = timer.update(i as f64 * 16.667);
            }

            prop_assert_eq!(timer.frame_count, n as u64);
        }

        /// Property: Interpolation alpha is non-negative and bounded when accumulator < fixed_dt
        #[test]
        fn property_interpolation_alpha_bounded(fixed_dt in 0.001f64..0.1) {
            let mut timer = FrameTimer::new();
            // Accumulator should be less than fixed_dt for normal operation
            timer.accumulator = fixed_dt * 0.5;
            timer.fixed_dt = fixed_dt;

            let alpha = timer.interpolation_alpha();
            prop_assert!(alpha >= 0.0 && alpha <= 1.0,
                "Alpha {} should be in [0, 1] for acc={}, fixed_dt={}", alpha, timer.accumulator, fixed_dt);
        }

        /// Property: Interpolation alpha is always non-negative
        #[test]
        fn property_interpolation_alpha_non_negative(accumulator in 0.0f64..1.0, fixed_dt in 0.001f64..0.1) {
            let mut timer = FrameTimer::new();
            timer.accumulator = accumulator;
            timer.fixed_dt = fixed_dt;

            let alpha = timer.interpolation_alpha();
            prop_assert!(alpha >= 0.0, "Alpha {} should be non-negative", alpha);
        }
    }
}
