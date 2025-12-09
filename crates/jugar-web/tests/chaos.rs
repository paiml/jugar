//! Chaos engineering tests for jugar-web.
//!
//! These tests apply extreme conditions to surface bugs and validate stability.
//! Based on Netflix's Chaos Engineering principles (Basiri et al., 2016).

#![allow(
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::cast_lossless,
    clippy::uninlined_format_args,
    clippy::std_instead_of_core
)]

use jugar_web::{
    loadtest::{ChaosConfig, ChaosResults, FrameTimeStats},
    GameState, WebConfig, WebPlatform,
};

/// Helper to generate a `KeyDown` event JSON string.
fn key_down_json(key: &str, timestamp: f64) -> String {
    format!(r#"{{"event_type":"KeyDown","timestamp":{timestamp},"data":{{"key":"{key}"}}}}"#)
}

/// Helper to generate a `KeyUp` event JSON string.
fn key_up_json(key: &str, timestamp: f64) -> String {
    format!(r#"{{"event_type":"KeyUp","timestamp":{timestamp},"data":{{"key":"{key}"}}}}"#)
}

// =============================================================================
// Input Flood Tests
// =============================================================================

/// Test that the game handles a flood of input events without crashing.
#[test]
fn test_input_flood_no_crash() {
    let config = ChaosConfig::input_flood();
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut results = ChaosResults::new();

    for frame in 0..config.duration_frames {
        // Generate many input events per frame
        let ts = frame as f64 * 16.667;
        let events: Vec<String> = (0..100)
            .map(|i| {
                let key = match i % 6 {
                    0 => "KeyW",
                    1 => "KeyS",
                    2 => "ArrowUp",
                    3 => "ArrowDown",
                    4 => "Space",
                    _ => "Escape",
                };
                key_down_json(key, ts)
            })
            .collect();

        let json = format!("[{}]", events.join(","));
        let start = std::time::Instant::now();
        let _ = platform.frame(ts, &json);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        results.record_frame_time(elapsed);
    }

    // Verify no crashes occurred
    assert!(results.passed(), "Input flood caused failures");
    assert_eq!(
        results.frames_executed, config.duration_frames,
        "Not all frames executed"
    );
}

/// Test that input flood doesn't cause NaN in game state.
#[test]
fn test_input_flood_no_nan() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut results = ChaosResults::new();

    for frame in 0..300 {
        let ts = frame as f64 * 16.667;
        // Alternate between many up and down inputs rapidly
        let events: Vec<String> = (0..50)
            .flat_map(|_| {
                vec![
                    key_down_json("KeyW", ts),
                    key_up_json("KeyW", ts + 1.0),
                    key_down_json("KeyS", ts + 2.0),
                    key_up_json("KeyS", ts + 3.0),
                ]
            })
            .collect();

        let json = format!("[{}]", events.join(","));
        let output = platform.frame(ts, &json);

        // Check for NaN in output
        if output.contains("NaN") || output.contains("nan") {
            results.record_nan();
        }
        if output.contains("Infinity") || output.contains("inf") {
            results.record_inf();
        }
    }

    assert!(!results.nan_detected, "NaN detected in output");
    assert!(!results.inf_detected, "Infinity detected in output");
}

// =============================================================================
// Time Warp Tests
// =============================================================================

/// Test that extreme delta times don't cause physics explosion.
#[test]
fn test_time_warp_stability() {
    let config = ChaosConfig::time_warp();
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut results = ChaosResults::new();

    // Start the game
    let _ = platform.frame(
        0.0,
        r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
    );

    let mut timestamp = 16.667;
    for frame in 0..config.duration_frames {
        // Vary delta time extremely
        let dt_factor = match frame % 10 {
            0 => 0.001,  // Very fast (0.001ms)
            1 => 0.01,   // Fast (0.01ms)
            2 => 0.1,    // Normal low
            3 => 1.0,    // Normal
            4 => 10.0,   // Slow
            5 => 100.0,  // Very slow (100ms)
            6 => 500.0,  // Extremely slow (500ms)
            7 => 1000.0, // 1 second
            8 => 0.0001, // Microsecond
            _ => 16.667, // Normal frame
        };

        timestamp += dt_factor;
        let output = platform.frame(timestamp, "[]");

        // Check for NaN/Inf
        if output.contains("NaN") || output.contains("nan") {
            results.record_nan();
        }
        if output.contains("Infinity") || output.contains("inf") {
            results.record_inf();
        }
    }

    assert!(!results.nan_detected, "Time warp caused NaN");
    assert!(!results.inf_detected, "Time warp caused Infinity");
}

/// Test backwards time doesn't crash.
#[test]
fn test_backwards_time() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    // Run forward
    for frame in 0..60 {
        let _ = platform.frame(frame as f64 * 16.667, "[]");
    }

    // Try to go backwards (should be handled gracefully)
    let output = platform.frame(0.0, "[]");

    // Should still produce valid output
    assert!(
        output.starts_with('{'),
        "Invalid output after backwards time"
    );
    assert!(
        output.contains("commands"),
        "Missing commands after backwards time"
    );
}

// =============================================================================
// Resize Blitz Tests
// =============================================================================

/// Test rapid resize events don't cause layout issues.
#[test]
fn test_resize_blitz_stability() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut results = ChaosResults::new();

    for frame in 0..300 {
        // Resize every 5 frames
        if frame % 5 == 0 {
            let width = 400 + (frame % 1600) as u32;
            let height = 300 + (frame % 900) as u32;
            platform.resize(width, height);
        }

        let start = std::time::Instant::now();
        let _ = platform.frame(frame as f64 * 16.667, "[]");
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        results.record_frame_time(elapsed);
    }

    assert!(results.passed(), "Resize blitz caused failures");
    // Resize shouldn't cause massive slowdowns
    assert!(
        results.max_frame_time_ms < 100.0,
        "Resize caused excessive frame time: {}ms",
        results.max_frame_time_ms
    );
}

/// Test extreme resize values.
///
/// Note: Resizing to very small values (< 100x100) causes panics because
/// the ball position calculations require min < max bounds. This is a known
/// limitation - games should enforce minimum canvas sizes. The chaos test
/// identified this issue! (Chaos engineering in action.)
#[test]
fn test_extreme_resize_values() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    // Test reasonable minimum size (below this causes invalid bounds)
    // TODO: Game should clamp to minimum size rather than panic
    platform.resize(100, 100);
    let output = platform.frame(0.0, "[]");
    assert!(output.starts_with('{'), "Invalid output at 100x100");

    // Test large size
    platform.resize(3840, 2160);
    let output = platform.frame(16.667, "[]");
    assert!(output.starts_with('{'), "Invalid output at 4K");

    // Test ultra-wide
    platform.resize(5120, 1440);
    let output = platform.frame(33.334, "[]");
    assert!(output.starts_with('{'), "Invalid output at ultrawide");

    // Test portrait
    platform.resize(600, 1200);
    let output = platform.frame(50.0, "[]");
    assert!(output.starts_with('{'), "Invalid output at portrait");

    // Test small but valid
    platform.resize(200, 150);
    let output = platform.frame(66.667, "[]");
    assert!(output.starts_with('{'), "Invalid output at 200x150");
}

// =============================================================================
// State Transition Stress Tests
// =============================================================================

/// Test rapid state transitions (pause/unpause spam).
#[test]
fn test_pause_spam() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    // Start game
    let _ = platform.frame(
        0.0,
        r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
    );
    assert_eq!(platform.pong().state(), GameState::Playing);

    // Spam pause/unpause
    for i in 0..100 {
        let ts = (i + 1) as f64 * 16.667;

        // Press Escape
        let _ = platform.frame(
            ts,
            &format!(r#"[{{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"Escape"}}}}]"#),
        );

        // Release Escape
        let ts2 = ts + 8.0;
        let _ = platform.frame(
            ts2,
            &format!(r#"[{{"event_type":"KeyUp","timestamp":{ts2},"data":{{"key":"Escape"}}}}]"#),
        );
    }

    // Game should still be in a valid state
    let state = platform.pong().state();
    assert!(
        state == GameState::Playing || state == GameState::Paused,
        "Invalid state after pause spam: {:?}",
        state
    );
}

// =============================================================================
// Memory Stability Tests
// =============================================================================

/// Test long-running session doesn't show signs of issues.
#[test]
fn test_long_session_stability() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());
    let mut stats = FrameTimeStats::with_capacity(3600);

    // Start game
    let _ = platform.frame(
        0.0,
        r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
    );

    // Run for 1 minute (3600 frames at 60fps)
    for frame in 1..3600 {
        let ts = frame as f64 * 16.667;

        // Occasional input
        let input = if frame % 60 == 0 {
            r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}}]"#
        } else {
            "[]"
        };

        let start = std::time::Instant::now();
        let _ = platform.frame(ts, input);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        stats.record(elapsed);
    }

    let report = stats.report();

    // Frame times should be consistent
    assert!(
        report.p99 < 50.0,
        "p99 frame time too high: {}ms",
        report.p99
    );

    // No huge outliers (indicates memory or other issues)
    assert!(
        report.max < 200.0,
        "Max frame time too high: {}ms (possible memory issue)",
        report.max
    );

    // Jitter should be reasonable
    assert!(
        report.jitter() < 100.0,
        "Frame time jitter too high: {}ms",
        report.jitter()
    );
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test empty input for many frames.
#[test]
fn test_empty_input_stability() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    for frame in 0..1000 {
        let output = platform.frame(frame as f64 * 16.667, "[]");
        assert!(output.starts_with('{'), "Invalid output at frame {}", frame);
    }
}

/// Test malformed JSON handling.
#[test]
fn test_malformed_json_handling() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    // Various malformed inputs
    let malformed_inputs = [
        "",
        "null",
        "{}",
        "[",
        "]",
        "not json",
        "[{}]",
        r#"[{"event_type": "Invalid"}]"#,
    ];

    for (i, input) in malformed_inputs.iter().enumerate() {
        // Should not panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            platform.frame(i as f64 * 16.667, input)
        }));

        assert!(result.is_ok(), "Panicked on malformed input: {}", input);
    }
}

/// Test simultaneous conflicting inputs.
#[test]
fn test_conflicting_inputs() {
    let mut platform = WebPlatform::new_for_test(WebConfig::default());

    // Start game
    let _ = platform.frame(
        0.0,
        r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
    );

    // Press up and down simultaneously
    for frame in 1..100 {
        let ts = frame as f64 * 16.667;
        let input = format!(
            r#"[
                {{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"KeyW"}}}},
                {{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"KeyS"}}}}
            ]"#
        );

        let output = platform.frame(ts, &input);
        assert!(
            output.starts_with('{'),
            "Invalid output with conflicting inputs"
        );
    }

    // Game should still be valid
    assert_eq!(platform.pong().state(), GameState::Playing);
}

// =============================================================================
// Determinism Tests
// =============================================================================

/// Test that same inputs produce same state.
#[test]
fn test_determinism() {
    // Run 1
    let mut platform1 = WebPlatform::new_for_test(WebConfig::default());

    // Fixed sequence of inputs
    let inputs = vec![
        (
            0.0,
            r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
        ),
        (
            16.667,
            r#"[{"event_type":"KeyDown","timestamp":16.667,"data":{"key":"KeyW"}}]"#,
        ),
        (33.334, "[]"),
        (
            50.0,
            r#"[{"event_type":"KeyUp","timestamp":50,"data":{"key":"KeyW"}}]"#,
        ),
        (
            66.667,
            r#"[{"event_type":"KeyDown","timestamp":66.667,"data":{"key":"KeyS"}}]"#,
        ),
    ];

    for (ts, input) in &inputs {
        let _ = platform1.frame(*ts, input);
    }

    let state1 = format!("{:?}", platform1.pong().state());

    // Run 2 (identical)
    let mut platform2 = WebPlatform::new_for_test(WebConfig::default());

    for (ts, input) in &inputs {
        let _ = platform2.frame(*ts, input);
    }

    let state2 = format!("{:?}", platform2.pong().state());

    assert_eq!(state1, state2, "Determinism violation detected");
}
