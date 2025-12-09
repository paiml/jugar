//! Property-based tests for game invariants.
//!
//! Uses proptest to generate random inputs and verify that game invariants
//! hold across thousands of test cases.
//!
//! Based on `QuickCheck` principles (Claessen & Hughes, 2000).

#![allow(clippy::unwrap_used, clippy::doc_markdown)]

use proptest::prelude::*;

use jugar_web::{GameState, WebConfig, WebPlatform};

// =============================================================================
// Strategies for generating game inputs
// =============================================================================

/// Generate a random key code from the set of valid game inputs.
fn key_code_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("KeyW"),
        Just("KeyS"),
        Just("ArrowUp"),
        Just("ArrowDown"),
        Just("Space"),
        Just("Escape"),
    ]
}

/// Generate a KeyDown event JSON string.
fn key_down_event(key: &str, timestamp: f64) -> String {
    format!(r#"{{"event_type":"KeyDown","timestamp":{timestamp},"data":{{"key":"{key}"}}}}"#)
}

/// Generate a KeyUp event JSON string.
fn key_up_event(key: &str, timestamp: f64) -> String {
    format!(r#"{{"event_type":"KeyUp","timestamp":{timestamp},"data":{{"key":"{key}"}}}}"#)
}

/// Strategy for generating a frame's worth of input events.
fn frame_events_strategy() -> impl Strategy<Value = Vec<(&'static str, bool)>> {
    prop::collection::vec(
        (key_code_strategy(), prop::bool::ANY),
        0..5, // 0-5 events per frame
    )
}

// =============================================================================
// Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 100,
        ..ProptestConfig::default()
    })]

    /// Game state is always valid after any sequence of inputs.
    #[test]
    fn state_always_valid(
        frame_events in prop::collection::vec(frame_events_strategy(), 0..100)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (frame, events) in frame_events.iter().enumerate() {
            let ts = frame as f64 * 16.667;
            let event_jsons: Vec<String> = events
                .iter()
                .map(|(key, is_down)| {
                    if *is_down {
                        key_down_event(key, ts)
                    } else {
                        key_up_event(key, ts)
                    }
                })
                .collect();

            let json = format!("[{}]", event_jsons.join(","));
            let output = platform.frame(ts, &json);

            // Output should always be valid JSON
            prop_assert!(
                output.starts_with('{'),
                "Invalid output at frame {}: {}",
                frame,
                &output[..output.len().min(100)]
            );

            // State should be one of the valid states
            let state = platform.pong().state();
            prop_assert!(
                matches!(state, GameState::Playing | GameState::Paused),
                "Invalid state: {:?}",
                state
            );
        }
    }

    /// Game remains in a valid state after random input sequences.
    #[test]
    fn state_remains_valid_after_inputs(
        frame_events in prop::collection::vec(frame_events_strategy(), 0..200)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (frame, events) in frame_events.iter().enumerate() {
            let ts = frame as f64 * 16.667;
            let event_jsons: Vec<String> = events
                .iter()
                .map(|(key, is_down)| {
                    if *is_down {
                        key_down_event(key, ts)
                    } else {
                        key_up_event(key, ts)
                    }
                })
                .collect();

            let json = format!("[{}]", event_jsons.join(","));
            let output = platform.frame(ts, &json);

            // Output must be valid JSON
            prop_assert!(
                output.starts_with('{'),
                "Invalid output at frame {}",
                frame
            );

            // State must be valid
            let state = platform.pong().state();
            prop_assert!(
                matches!(state, GameState::Playing | GameState::Paused),
                "Invalid state at frame {}: {:?}",
                frame,
                state
            );
        }
    }

    /// Output JSON is always valid and contains expected fields.
    #[test]
    fn output_json_valid(
        num_frames in 1usize..100,
        with_input in prop::bool::ANY,
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for frame in 0..num_frames {
            let ts = frame as f64 * 16.667;
            let input = if with_input && frame % 10 == 0 {
                format!(r#"[{{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"KeyW"}}}}]"#)
            } else {
                "[]".to_string()
            };

            let output = platform.frame(ts, &input);

            // Parse as JSON
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
            prop_assert!(
                parsed.is_ok(),
                "Invalid JSON at frame {}: {}",
                frame,
                &output[..output.len().min(200)]
            );

            let value = parsed.unwrap();

            // Must have commands field
            prop_assert!(
                value.get("commands").is_some(),
                "Missing 'commands' field at frame {}",
                frame
            );
        }
    }

    /// Frame rate variations don't cause physics instability.
    #[test]
    fn variable_frame_rate_stability(
        dt_factors in prop::collection::vec(0.5f64..2.0, 10..100)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        // Start game
        let _ = platform.frame(
            0.0,
            r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
        );

        let mut timestamp = 16.667;
        for (frame, dt_factor) in dt_factors.iter().enumerate() {
            let dt = 16.667 * dt_factor;
            timestamp += dt;

            let output = platform.frame(timestamp, "[]");

            // Should not contain NaN or Infinity
            prop_assert!(
                !output.contains("NaN") && !output.contains("nan"),
                "NaN detected at frame {} with dt_factor {}",
                frame,
                dt_factor
            );
            prop_assert!(
                !output.contains("Infinity") && !output.contains("inf"),
                "Infinity detected at frame {} with dt_factor {}",
                frame,
                dt_factor
            );
        }
    }

    /// Resize operations don't break the game (within valid bounds).
    #[test]
    fn resize_stability(
        sizes in prop::collection::vec((200u32..2000, 150u32..1500), 1..20)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (i, (width, height)) in sizes.iter().enumerate() {
            platform.resize(*width, *height);
            let output = platform.frame(i as f64 * 16.667, "[]");

            prop_assert!(
                output.starts_with('{'),
                "Invalid output after resize to {}x{}",
                width,
                height
            );

            // Config should be updated
            prop_assert_eq!(
                platform.config().width, *width,
                "Width not updated"
            );
            prop_assert_eq!(
                platform.config().height, *height,
                "Height not updated"
            );
        }
    }

    /// Tracer records correct frame count.
    #[test]
    fn tracer_frame_count(num_frames in 1usize..200) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for frame in 0..num_frames {
            let _ = platform.frame(frame as f64 * 16.667, "[]");
        }

        let stats = platform.tracer().stats();
        prop_assert_eq!(
            stats.frame as usize, num_frames,
            "Tracer frame count mismatch"
        );
    }

    /// Game handles rapid pause/unpause without corruption.
    #[test]
    fn rapid_pause_toggle(
        toggles in prop::collection::vec(prop::bool::ANY, 0..50)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        // Start game first
        let _ = platform.frame(
            0.0,
            r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#,
        );

        for (i, should_toggle) in toggles.iter().enumerate() {
            let ts = (i + 1) as f64 * 16.667;

            if *should_toggle {
                // Press and release Escape
                let _ = platform.frame(
                    ts,
                    &format!(r#"[{{"event_type":"KeyDown","timestamp":{ts},"data":{{"key":"Escape"}}}}]"#),
                );
                let _ = platform.frame(
                    ts + 8.0,
                    &format!(r#"[{{"event_type":"KeyUp","timestamp":{},"data":{{"key":"Escape"}}}}]"#, ts + 8.0),
                );
            } else {
                let _ = platform.frame(ts, "[]");
            }

            // State should always be valid
            let state = platform.pong().state();
            prop_assert!(
                matches!(state, GameState::Playing | GameState::Paused),
                "Invalid state after toggle {}: {:?}",
                i,
                state
            );
        }
    }

    /// Empty input frames don't cause drift or corruption.
    #[test]
    fn empty_input_stability(num_frames in 100usize..500) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for frame in 0..num_frames {
            let output = platform.frame(frame as f64 * 16.667, "[]");

            prop_assert!(
                output.contains("commands"),
                "Missing commands at frame {}",
                frame
            );
        }

        // Tracer should have recorded all frames
        let stats = platform.tracer().stats();
        prop_assert_eq!(
            stats.frame as usize, num_frames,
            "Frame count mismatch after {} frames",
            num_frames
        );
    }
}

// =============================================================================
// Determinism Tests (require exact reproducibility)
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 20, // Fewer cases since these run twice
        ..ProptestConfig::default()
    })]

    /// Same inputs produce identical final state (determinism check).
    #[test]
    fn deterministic_replay(
        frame_events in prop::collection::vec(frame_events_strategy(), 10..50)
    ) {
        // Run 1
        let mut platform1 = WebPlatform::new_for_test(WebConfig::default());

        for (frame, events) in frame_events.iter().enumerate() {
            let ts = frame as f64 * 16.667;
            let event_jsons: Vec<String> = events
                .iter()
                .map(|(key, is_down)| {
                    if *is_down {
                        key_down_event(key, ts)
                    } else {
                        key_up_event(key, ts)
                    }
                })
                .collect();

            let json = format!("[{}]", event_jsons.join(","));
            let _ = platform1.frame(ts, &json);
        }

        let state1 = format!("{:?}", platform1.pong().state());

        // Run 2 (identical)
        let mut platform2 = WebPlatform::new_for_test(WebConfig::default());

        for (frame, events) in frame_events.iter().enumerate() {
            let ts = frame as f64 * 16.667;
            let event_jsons: Vec<String> = events
                .iter()
                .map(|(key, is_down)| {
                    if *is_down {
                        key_down_event(key, ts)
                    } else {
                        key_up_event(key, ts)
                    }
                })
                .collect();

            let json = format!("[{}]", event_jsons.join(","));
            let _ = platform2.frame(ts, &json);
        }

        let state2 = format!("{:?}", platform2.pong().state());

        prop_assert_eq!(
            &state1, &state2,
            "State mismatch: {} vs {}",
            state1, state2
        );
    }
}
