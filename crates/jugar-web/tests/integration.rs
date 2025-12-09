//! Integration tests for jugar-web.
//!
//! These tests verify the complete game lifecycle from input to render output.

#![allow(clippy::unwrap_used, clippy::field_reassign_with_default)]

use jugar_web::{GameState, WebConfig, WebPlatform};

/// Test that a full game frame cycle works end-to-end.
#[test]
fn test_full_frame_cycle() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    // Start the game with Space key
    let start_event = r#"[{"event_type":"KeyDown","timestamp":100.0,"data":{"key":"Space"}}]"#;
    let output_json = platform.frame(100.0, start_event);

    // Should return valid JSON with commands
    assert!(output_json.starts_with('{'), "Output should be JSON object");
    assert!(
        output_json.contains("commands"),
        "Should have commands field"
    );

    // Game should be playing now
    assert_eq!(platform.pong().state(), GameState::Playing);
}

/// Test that input affects game state over time.
#[test]
fn test_input_over_time() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    // Start game
    let _ = platform.frame(
        100.0,
        r#"[{"event_type":"KeyDown","timestamp":100.0,"data":{"key":"Space"}}]"#,
    );

    // Run several frames with W key pressed (move up)
    for i in 0..10 {
        let ts = f64::from(i).mul_add(16.667, 116.667);
        let _ = platform.frame(
            ts,
            r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}}]"#,
        );
    }

    // Game should still be playing
    assert_eq!(platform.pong().state(), GameState::Playing);
}

/// Test that render commands produce valid JSON.
#[test]
fn test_render_commands_valid_json() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    let output_json = platform.frame(100.0, "[]");

    // Should parse as valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output_json);
    assert!(parsed.is_ok(), "Output should be valid JSON: {output_json}");

    let value = parsed.unwrap();
    assert!(
        value.get("commands").is_some(),
        "Should have commands field"
    );
}

/// Test game state transitions.
#[test]
fn test_game_state_transitions() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    // Initially in Playing (Demo mode attract)
    assert_eq!(platform.pong().state(), GameState::Playing);

    // Release Space first (to clear the "was pressed" state)
    let _ = platform.frame(
        108.333,
        r#"[{"event_type":"KeyUp","timestamp":108.333,"data":{"key":"Space"}}]"#,
    );

    // Press Escape to pause
    let _ = platform.frame(
        116.667,
        r#"[{"event_type":"KeyDown","timestamp":116.667,"data":{"key":"Escape"}}]"#,
    );
    assert_eq!(platform.pong().state(), GameState::Paused);

    // Release Escape first (to allow "just pressed" detection on next press)
    let _ = platform.frame(
        125.0,
        r#"[{"event_type":"KeyUp","timestamp":125.0,"data":{"key":"Escape"}}]"#,
    );

    // Press Escape again to resume
    let _ = platform.frame(
        133.333,
        r#"[{"event_type":"KeyDown","timestamp":133.333,"data":{"key":"Escape"}}]"#,
    );
    assert_eq!(platform.pong().state(), GameState::Playing);
}

/// Test resize handling.
#[test]
fn test_resize_updates_config() {
    let config = WebConfig::new(800, 600);
    let mut platform = WebPlatform::new_for_test(config);

    assert_eq!(platform.config().width, 800);
    assert_eq!(platform.config().height, 600);

    platform.resize(1920, 1080);

    assert_eq!(platform.config().width, 1920);
    assert_eq!(platform.config().height, 1080);
}

/// Test multiple frames without input.
#[test]
fn test_idle_frames() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    // Run 60 frames without any input (Demo mode, playing)
    for i in 0..60 {
        let ts = f64::from(i) * 16.667;
        let output = platform.frame(ts, "[]");
        assert!(!output.is_empty(), "Frame {i} should produce output");
    }

    // Should still be playing (Demo mode)
    assert_eq!(platform.pong().state(), GameState::Playing);
}

/// Test that game tracer records frames.
#[test]
fn test_tracer_records_frames() {
    let config = WebConfig::default();
    let mut platform = WebPlatform::new_for_test(config);

    // Run 10 frames
    for i in 0..10 {
        let ts = f64::from(i) * 16.667;
        let _ = platform.frame(ts, "[]");
    }

    // Tracer should have recorded 10 frames
    let stats = platform.tracer().stats();
    assert_eq!(stats.frame, 10, "Tracer should have recorded 10 frames");
    assert!(
        stats.buffer_len <= 10,
        "Buffer should contain up to 10 frames"
    );
}

/// Test that tracer statistics appear in debug output.
#[test]
fn test_tracer_debug_info() {
    let mut config = WebConfig::default();
    config.debug = true;
    let mut platform = WebPlatform::new_for_test(config);

    // Run a few frames
    for i in 0..5 {
        let ts = f64::from(i) * 16.667;
        let _ = platform.frame(ts, "[]");
    }

    // Get the output with debug info
    let output = platform.frame(100.0, "[]");

    // Debug output should contain trace buffer usage
    assert!(
        output.contains("trace_buffer_usage"),
        "Debug output should contain trace_buffer_usage: {output}"
    );
}
