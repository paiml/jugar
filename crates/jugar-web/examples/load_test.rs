//! Load Testing Example
//!
//! Demonstrates how to use the load testing infrastructure to validate
//! game performance and stability.
//!
//! # Running
//!
//! ```bash
//! cargo run --example load_test
//! ```
//!
//! # Features
//!
//! - Chaos engineering scenarios (input flood, time warp, resize blitz)
//! - Frame time statistics with percentile analysis
//! - Drift detection for performance regressions
//!
//! # Output
//!
//! Prints a report showing:
//! - Pass/fail status for each scenario
//! - Frame time statistics (min, max, mean, p50, p90, p95, p99)
//! - Whether 60 FPS / 120 FPS targets are met

#![allow(clippy::cast_lossless, clippy::uninlined_format_args)]

use jugar_web::{
    loadtest::{ChaosConfig, ChaosResults, DriftDetector, FrameTimeStats},
    WebConfig, WebPlatform,
};

fn main() {
    println!("=== Jugar Load Test Suite ===\n");

    // Run each load test tier
    run_tier1_tests();
    run_tier2_tests();
    run_tier3_tests();

    println!("\n=== Load Test Complete ===");
}

/// Tier 1: Quick smoke tests (< 5 seconds)
fn run_tier1_tests() {
    println!("--- Tier 1: Quick Smoke Tests ---\n");

    // Basic stability test
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut stats = FrameTimeStats::new();

    for frame in 0..60 {
        let start = std::time::Instant::now();
        let _ = platform.frame(frame as f64 * 16.667, "[]");
        stats.record(start.elapsed().as_secs_f64() * 1000.0);
    }

    let report = stats.report();
    println!("Basic Stability (60 frames):");
    println!("  Frames: {}", report.count);
    println!("  Mean:   {:.3}ms", report.mean);
    println!("  p99:    {:.3}ms", report.p99);
    println!(
        "  Status: {}",
        if report.meets_60fps() {
            "PASS (meets 60 FPS)"
        } else {
            "FAIL (exceeds 16.67ms budget)"
        }
    );
    println!();
}

/// Tier 2: Integration tests (< 30 seconds)
fn run_tier2_tests() {
    println!("--- Tier 2: Integration Tests ---\n");

    // Input flood test
    let config = ChaosConfig::input_flood();
    let (passed, stats) = run_chaos_scenario(&config, "Input Flood");
    print_chaos_results("Input Flood", passed, &stats);

    // Time warp test
    let config = ChaosConfig::time_warp();
    let (passed, stats) = run_chaos_scenario(&config, "Time Warp");
    print_chaos_results("Time Warp", passed, &stats);
}

/// Tier 3: Full validation (< 5 minutes)
fn run_tier3_tests() {
    println!("--- Tier 3: Full Validation ---\n");

    // Long session stability
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut stats = FrameTimeStats::with_capacity(3600);
    // Use z_threshold of 3.0 (99.7% confidence) instead of 2.0 (95%)
    // This reduces false positives from normal variance
    let mut drift_detector = DriftDetector::new(60, 3.0);

    // WARM-UP PHASE: Run 60 frames to stabilize JIT/cache (don't record)
    // This is a common pattern in benchmarking to avoid cold-start bias
    for frame in 0..60 {
        let _ = platform.frame(frame as f64 * 16.667, "[]");
    }

    // Calibrate with next 60 frames (after warm-up)
    let mut calibration_samples = Vec::with_capacity(60);
    for frame in 60..120 {
        let start = std::time::Instant::now();
        let _ = platform.frame(frame as f64 * 16.667, "[]");
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        calibration_samples.push(elapsed);
        stats.record(elapsed);
    }
    drift_detector.calibrate(&calibration_samples);

    // Run for 1 minute (3600 frames at 60fps)
    let mut anomaly_count = 0;
    for frame in 120..3720 {
        let start = std::time::Instant::now();
        let _ = platform.frame(frame as f64 * 16.667, "[]");
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        stats.record(elapsed);
        if drift_detector.observe(elapsed).is_anomaly() {
            anomaly_count += 1;
        }
    }

    let report = stats.report();
    println!("Long Session (3600 frames / 1 minute):");
    println!("  Min:       {:.3}ms", report.min);
    println!("  Max:       {:.3}ms", report.max);
    println!("  Mean:      {:.3}ms", report.mean);
    println!("  Std Dev:   {:.3}ms", report.std_dev);
    println!("  p50:       {:.3}ms", report.p50);
    println!("  p90:       {:.3}ms", report.p90);
    println!("  p95:       {:.3}ms", report.p95);
    println!("  p99:       {:.3}ms", report.p99);
    println!("  Jitter:    {:.3}ms", report.jitter());
    println!("  Anomalies: {}", anomaly_count);
    println!();

    // Check drift
    if let Some(drift) = drift_detector.detect_drift() {
        println!(
            "  WARNING: Performance drift detected! ({:.1}% {})",
            drift.drift_percent.abs(),
            if drift.is_regression() {
                "regression"
            } else {
                "improvement"
            }
        );
    }

    // Final verdict
    let meets_60fps = report.meets_60fps();
    let meets_120fps = report.meets_120fps();
    println!("  60 FPS:    {}", if meets_60fps { "PASS" } else { "FAIL" });
    println!(
        "  120 FPS:   {}",
        if meets_120fps { "PASS" } else { "FAIL" }
    );
    println!();
}

/// Run a chaos scenario and collect results
fn run_chaos_scenario(config: &ChaosConfig, _name: &str) -> (bool, FrameTimeStats) {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut results = ChaosResults::new();
    let mut stats = FrameTimeStats::with_capacity(config.duration_frames as usize);

    for frame in 0..config.duration_frames {
        let ts = frame as f64 * 16.667;

        // Generate input based on scenario
        let input = match &config.scenario {
            jugar_web::loadtest::ChaosScenario::InputFlood { events_per_frame } => {
                let events: Vec<String> = (0..*events_per_frame)
                    .map(|i| {
                        let key = match i % 4 {
                            0 => "KeyW",
                            1 => "KeyS",
                            2 => "ArrowUp",
                            _ => "ArrowDown",
                        };
                        format!(
                            r#"{{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"{key}"}}}}"#
                        )
                    })
                    .collect();
                format!("[{}]", events.join(","))
            }
            _ => "[]".to_string(),
        };

        let start = std::time::Instant::now();
        let output = platform.frame(ts, &input);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        results.record_frame_time(elapsed);
        stats.record(elapsed);

        // Check for NaN/Inf
        if output.contains("NaN") || output.contains("nan") {
            results.record_nan();
        }
        if output.contains("Infinity") || output.contains("inf") {
            results.record_inf();
        }
    }

    (results.passed(), stats)
}

/// Print chaos test results
fn print_chaos_results(name: &str, passed: bool, stats: &FrameTimeStats) {
    let report = stats.report();
    println!("{} Chaos Test:", name);
    println!("  Frames: {}", report.count);
    println!("  Mean:   {:.3}ms", report.mean);
    println!("  p99:    {:.3}ms", report.p99);
    println!(
        "  Status: {}",
        if passed {
            "PASS (no crashes/NaN)"
        } else {
            "FAIL (crash or NaN detected)"
        }
    );
    println!();
}
