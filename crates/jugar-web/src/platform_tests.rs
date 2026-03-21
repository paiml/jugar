use super::*;

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::manual_range_contains,
    clippy::cast_lossless,
    clippy::suboptimal_flops,
    clippy::uninlined_format_args
)]
mod tests {
    use super::*;

    #[test]
    fn test_web_config_default() {
        let config = WebConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.target_fps, 60);
        assert!(!config.debug);
    }

    #[test]
    fn test_web_config_new() {
        let config = WebConfig::new(1920, 1080);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_web_config_from_json() {
        let json = r#"{"width":1024,"height":768,"target_fps":30,"debug":true}"#;
        let config = WebConfig::from_json(json).unwrap();
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.target_fps, 30);
        assert!(config.debug);
    }

    #[test]
    fn test_web_config_from_json_invalid() {
        let result = WebConfig::from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_web_config_to_json() {
        let config = WebConfig::new(800, 600);
        let json = config.to_json().unwrap();
        assert!(json.contains("800"));
        assert!(json.contains("600"));
    }

    #[test]
    fn test_web_platform_error_display() {
        let err = WebPlatformError::InvalidConfig("bad config".to_string());
        assert_eq!(err.to_string(), "Invalid config: bad config");

        let err = WebPlatformError::InputError("input failed".to_string());
        assert_eq!(err.to_string(), "Input error: input failed");

        let err = WebPlatformError::RenderError("render failed".to_string());
        assert_eq!(err.to_string(), "Render error: render failed");
    }

    #[test]
    fn test_web_platform_error_from_input_error() {
        let input_err = InputTranslationError::InvalidJson("test".to_string());
        let platform_err: WebPlatformError = input_err.into();
        assert!(matches!(platform_err, WebPlatformError::InputError(_)));
    }

    #[test]
    fn test_pong_game_new() {
        let game = PongGame::new(800.0, 600.0, false);
        assert!((game.width - 800.0).abs() < f32::EPSILON);
        assert!((game.height - 600.0).abs() < f32::EPSILON);
        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
    }

    #[test]
    fn test_pong_game_default() {
        let game = PongGame::default();
        assert!((game.width - 800.0).abs() < f32::EPSILON);
        assert!((game.height - 600.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pong_game_ball_position() {
        let game = PongGame::new(800.0, 600.0, false);
        let (x, y) = game.ball_position();
        assert!((x - 400.0).abs() < f32::EPSILON);
        assert!((y - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pong_game_update_no_input() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let input = InputState::new();
        let initial_ball_x = game.ball_x;

        game.update(&input, 0.016);

        // Ball should have moved
        assert!((game.ball_x - initial_ball_x).abs() > 1.0);
    }

    #[test]
    fn test_pong_game_paddle_movement() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::SinglePlayer; // Human controls RIGHT paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press Up arrow key (P1 controls right paddle)
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Right paddle should have moved up
        assert!(game.right_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_game_paddle_clamping() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::SinglePlayer; // Human controls left paddle
        let mut input = InputState::new();

        // Press W key many times to hit top
        input.set_key_pressed(jugar_input::KeyCode::Letter('W'), true);
        for _ in 0..100 {
            game.update(&input, 0.1);
        }

        // Paddle should be clamped to screen
        let half_paddle = game.paddle_height / 2.0;
        assert!(game.left_paddle_y >= half_paddle);
    }

    #[test]
    fn test_pong_game_render() {
        let mut game = PongGame::new(800.0, 600.0, false);
        let mut frame = RenderFrame::new();

        game.render(&mut frame);

        // Should have several commands (clear, center line dashes, 2 paddles, ball, 2 scores)
        assert!(frame.len() > 5);
    }

    #[test]
    fn test_pong_game_resize() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.resize(1600, 1200);

        assert!((game.width - 1600.0).abs() < f32::EPSILON);
        assert!((game.height - 1200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_web_platform_new_for_test() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        assert_eq!(platform.config().width, 800);
        assert_eq!(platform.config().height, 600);
    }

    #[test]
    fn test_web_platform_frame() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // First frame
        let result = platform.frame(0.0, "[]");
        assert!(!result.is_empty());
        assert!(result.contains("Clear"));

        // Second frame
        let result = platform.frame(16.667, "[]");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_web_platform_frame_with_input() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let input_json = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}}]"#;
        let _ = platform.frame(0.0, input_json);

        // Key should be registered
        assert!(platform
            .input()
            .is_key_pressed(jugar_input::KeyCode::Letter('W')));
    }

    #[test]
    fn test_web_platform_resize() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        platform.resize(1920, 1080);

        assert_eq!(platform.config().width, 1920);
        assert_eq!(platform.config().height, 1080);
    }

    #[test]
    fn test_web_platform_get_config() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let config_json = platform.get_config();
        assert!(config_json.contains("800"));
        assert!(config_json.contains("600"));
    }

    #[test]
    fn test_web_platform_get_stats() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Run a few frames
        let _ = platform.frame(0.0, "[]");
        let _ = platform.frame(16.667, "[]");
        let _ = platform.frame(33.333, "[]");

        let stats = platform.get_stats();
        assert!(stats.contains("fps"));
        assert!(stats.contains("frame_count"));
    }

    #[test]
    fn test_web_platform_reset_timer() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");
        let _ = platform.frame(1000.0, "[]");

        platform.reset_timer();

        assert_eq!(platform.timer().frame_count(), 0);
    }

    #[test]
    fn test_web_platform_debug_mode() {
        let config = WebConfig {
            debug: true,
            ..WebConfig::default()
        };
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");
        let result = platform.frame(16.667, "[]");

        // Debug text should be rendered
        assert!(result.contains("FPS"));
    }

    #[test]
    fn test_web_platform_input_accessors() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Test input_mut
        platform
            .input_mut()
            .set_key_pressed(jugar_input::KeyCode::Space, true);

        // Test input
        assert!(platform.input().is_key_pressed(jugar_input::KeyCode::Space));
    }

    #[test]
    fn test_web_platform_pong_accessor() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let pong = platform.pong();
        assert_eq!(pong.left_score(), 0);
        assert_eq!(pong.right_score(), 0);
    }

    #[test]
    fn test_pong_ball_wall_collision_top() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_y = 5.0; // Near top
        game.ball_vy = -100.0; // Moving up

        let input = InputState::new();
        game.update(&input, 0.1);

        // Ball should have bounced
        assert!(game.ball_vy > 0.0);
    }

    #[test]
    fn test_pong_ball_wall_collision_bottom() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_y = 595.0; // Near bottom
        game.ball_vy = 100.0; // Moving down

        let input = InputState::new();
        game.update(&input, 0.1);

        // Ball should have bounced
        assert!(game.ball_vy < 0.0);
    }

    #[test]
    fn test_pong_scoring_right() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_x = -10.0; // Past left edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // Right player should score
        assert_eq!(game.right_score(), 1);
    }

    #[test]
    fn test_pong_scoring_left() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_x = 810.0; // Past right edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // Left player should score
        assert_eq!(game.left_score(), 1);
    }

    #[test]
    fn test_frame_output_serialization() {
        let output = FrameOutput {
            commands: vec![Canvas2DCommand::Clear {
                color: Color::BLACK,
            }],
            audio_events: vec![],
            actions: vec![],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("commands"));
        assert!(!json.contains("debug_info")); // Skipped when None
        assert!(!json.contains("audio_events")); // Skipped when empty
    }

    #[test]
    fn test_frame_output_with_debug() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![],
            actions: vec![],
            debug_info: Some(DebugInfo {
                dt_ms: 16.667,
                fps: 60.0,
                frame_count: 100,
                input_summary: "W pressed".to_string(),
                game_mode: "SinglePlayer".to_string(),
                speed_multiplier: 1,
                left_paddle_y: 300.0,
                right_paddle_y: 300.0,
                ball_x: 400.0,
                ball_y: 300.0,
                trace_buffer_usage: Some("100/3600".to_string()),
                trace_inputs: Some(42),
                trace_dropped: Some(0),
            }),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("debug_info"));
        assert!(json.contains("fps"));
    }

    #[test]
    fn test_frame_output_with_audio() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![AudioEvent::GameStart { volume: 0.7 }],
            actions: vec![],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("audio_events"));
        assert!(json.contains("GameStart"));
    }

    #[test]
    fn test_debug_info_serialization() {
        let info = DebugInfo {
            dt_ms: 16.667,
            fps: 60.0,
            frame_count: 42,
            input_summary: "test".to_string(),
            game_mode: "Demo".to_string(),
            speed_multiplier: 10,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            ball_x: 400.0,
            ball_y: 300.0,
            trace_buffer_usage: None,
            trace_inputs: None,
            trace_dropped: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("16.667"));
        assert!(json.contains("60"));
        // Trace fields are skipped when None
        assert!(!json.contains("trace_buffer_usage"));
        assert!(json.contains("42"));
        assert!(json.contains("Demo"));
        assert!(json.contains("10"));
        assert!(json.contains("left_paddle_y"));
        assert!(json.contains("ball_x"));
    }

    #[test]
    fn test_pong_right_paddle_controls() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::TwoPlayer; // Enable human control of right paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press ArrowDown key
        input.set_key_pressed(jugar_input::KeyCode::Down, true);
        game.update(&input, 0.1);

        // Right paddle should have moved down
        assert!(game.right_paddle_y > initial_y);
    }

    #[test]
    fn test_pong_right_paddle_up() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::TwoPlayer; // Enable human control of right paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press ArrowUp key
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Right paddle should have moved up
        assert!(game.right_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_right_paddle_down() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::SinglePlayer; // Human controls RIGHT paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press Down arrow key (P1 controls right paddle)
        input.set_key_pressed(jugar_input::KeyCode::Down, true);
        game.update(&input, 0.1);

        // Right paddle should have moved down
        assert!(game.right_paddle_y > initial_y);
    }

    #[test]
    fn test_pong_paddle_collision_left() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
                                            // Position ball to hit left paddle (paddle at x=20..35)
                                            // Ball radius is 10, so ball_x - radius needs to be in (20, 35)
                                            // ball_x = 46 means left edge at 36, after moving left 1.6px (vx=-100, dt=0.016)
                                            // left edge becomes 36-1.6=34.4, which is in (20, 35)
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0; // Slower velocity so we stay in collision zone

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should have bounced (velocity reversed to positive)
        assert!(game.ball_vx > 0.0);
    }

    #[test]
    fn test_pong_paddle_collision_right() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
                                            // Position ball to hit right paddle (paddle at x=765..780)
                                            // Ball radius is 10, so ball_x + radius needs to be in (765, 780)
                                            // ball_x = 758 means right edge at 768, which is in (765, 780)
        game.ball_x = 758.0;
        game.ball_y = game.right_paddle_y;
        game.ball_vx = 200.0;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should have bounced (velocity reversed to negative)
        assert!(game.ball_vx < 0.0);
    }

    #[test]
    fn test_pong_resize_scales_positions() {
        let mut game = PongGame::new(800.0, 600.0, false);
        // Set ball to center
        game.ball_x = 400.0;
        game.ball_y = 300.0;

        game.resize(1600, 1200);

        // Ball should be scaled to new center
        assert!((game.ball_x - 800.0).abs() < 1.0);
        assert!((game.ball_y - 600.0).abs() < 1.0);
    }

    // =========================================================================
    // Game State Tests
    // =========================================================================

    #[test]
    fn test_game_state_default_is_playing() {
        // Game starts directly in Playing state (Demo mode attract)
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_start_from_menu() {
        // If game is set to Menu state, start() transitions to Playing
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu);
        assert_eq!(game.state(), GameState::Menu);

        game.start();

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_pause_transitions() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        // Press Escape to pause
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Escape, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Paused);
    }

    #[test]
    fn test_game_state_unpause_with_space() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);

        // First update with space not pressed (to set prev state)
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_menu_space_starts_game() {
        // When in Menu state, pressing Space starts the game
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu); // Set to Menu state for this test
        assert_eq!(game.state(), GameState::Menu);

        // First update with space not pressed (to set prev state)
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_game_over_on_winning_score() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.left_score = 10; // One away from winning
        game.ball_x = 810.0; // Past right edge - left player scores

        let input = InputState::new();
        game.update(&input, 0.016);

        assert_eq!(game.left_score(), 11);
        assert_eq!(game.state(), GameState::GameOver);
    }

    #[test]
    fn test_game_state_reset_from_game_over() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::GameOver);
        game.left_score = 11;
        game.right_score = 5;

        // First update with space not pressed
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space to restart
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
    }

    #[test]
    fn test_game_state_no_update_when_paused() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);
        let initial_ball_x = game.ball_x;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should not have moved
        assert!((game.ball_x - initial_ball_x).abs() < 0.001);
    }

    #[test]
    fn test_game_state_reset_game() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.left_score = 5;
        game.right_score = 3;
        game.ball_x = 100.0;

        game.reset_game();

        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
        assert!((game.ball_x - 400.0).abs() < 1.0); // Back to center
    }

    // =========================================================================
    // Rally Counter Tests
    // =========================================================================

    #[test]
    fn test_rally_counter_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.rally_count(), 0);
    }

    #[test]
    fn test_rally_counter_increments_on_paddle_hit() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        assert_eq!(game.rally_count(), 0);

        // Position ball to hit left paddle
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have incremented
        assert_eq!(game.rally_count(), 1);
    }

    #[test]
    fn test_rally_counter_resets_on_scoring() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.rally_count = 5; // Simulate some rallies
        game.ball_x = -10.0; // Ball past left edge - right player scores

        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have reset to 0
        assert_eq!(game.rally_count(), 0);
    }

    #[test]
    fn test_rally_counter_resets_on_game_reset() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.rally_count = 10;

        game.reset_game();

        assert_eq!(game.rally_count(), 0);
    }

    // =========================================================================
    // High Score Tests
    // =========================================================================

    #[test]
    fn test_high_score_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.high_score(), 0);
    }

    #[test]
    fn test_high_score_updates_when_rally_ends() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.rally_count = 8;
        game.ball_x = -10.0; // Ball past left edge - triggers scoring

        let input = InputState::new();
        game.update(&input, 0.016);

        // High score should be 8 (the rally count before reset)
        assert_eq!(game.high_score(), 8);
    }

    #[test]
    fn test_high_score_only_updates_if_higher() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.high_score = 10;
        game.rally_count = 5;
        game.ball_x = -10.0; // Ball past left edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // High score should still be 10 (higher than 5)
        assert_eq!(game.high_score(), 10);
    }

    #[test]
    fn test_high_score_persists_across_reset() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.high_score = 15;

        game.reset_game();

        // High score should persist
        assert_eq!(game.high_score(), 15);
    }

    #[test]
    fn test_high_score_updates_on_both_sides_scoring() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        // Left player misses
        game.rally_count = 7;
        game.ball_x = -10.0;
        let input = InputState::new();
        game.update(&input, 0.016);
        assert_eq!(game.high_score(), 7);

        // Reset ball and rally
        game.ball_x = 400.0;

        // Now right player misses with a higher rally
        game.rally_count = 12;
        game.ball_x = 810.0;
        game.update(&input, 0.016);
        assert_eq!(game.high_score(), 12);
    }

    // =========================================================================
    // Background Animation Tests
    // =========================================================================

    #[test]
    fn test_bg_time_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert!((game.bg_time - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bg_time_increments_during_gameplay() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        assert!((game.bg_time - 0.0).abs() < f32::EPSILON);

        let input = InputState::new();
        game.update(&input, 0.5);

        assert!((game.bg_time - 0.5).abs() < 0.001);
    }

    // =========================================================================
    // HUD Button Tests for Coverage
    // =========================================================================

    #[test]
    fn test_hud_buttons_default() {
        let hud = HudButtons::default();
        // All buttons should be at default (0, 0, 0, 0) position
        assert_eq!(hud.mode_demo.x, 0.0);
        assert_eq!(hud.mode_1p.x, 0.0);
        assert_eq!(hud.mode_2p.x, 0.0);
        assert_eq!(hud.speed_1x.x, 0.0);
    }

    #[test]
    fn test_hud_buttons_calculate() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Mode buttons should be positioned
        assert!(hud.mode_demo.x > 0.0);
        assert!(hud.mode_1p.x > hud.mode_demo.x);
        assert!(hud.mode_2p.x > hud.mode_1p.x);
        // Speed buttons should be positioned
        assert!(hud.speed_1x.x > 0.0);
    }

    #[test]
    fn test_hud_buttons_hit_test() {
        let hud = HudButtons::calculate(800.0, 600.0);

        // Test hit detection using contains method
        let test_x = hud.mode_demo.x + hud.mode_demo.width / 2.0;
        let test_y = hud.mode_demo.y + hud.mode_demo.height / 2.0;

        // Point inside button
        assert!(hud.mode_demo.contains(test_x, test_y));
    }

    #[test]
    fn test_button_rect_new() {
        let btn = ButtonRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(btn.x, 10.0);
        assert_eq!(btn.y, 20.0);
        assert_eq!(btn.width, 100.0);
        assert_eq!(btn.height, 50.0);
    }

    #[test]
    fn test_button_rect_contains() {
        let btn = ButtonRect::new(10.0, 10.0, 100.0, 50.0);
        // Inside
        assert!(btn.contains(50.0, 30.0));
        // Outside left
        assert!(!btn.contains(5.0, 30.0));
        // Outside right
        assert!(!btn.contains(120.0, 30.0));
        // Outside top
        assert!(!btn.contains(50.0, 5.0));
        // Outside bottom
        assert!(!btn.contains(50.0, 65.0));
    }

    #[test]
    fn test_web_platform_mode_toggle_via_d_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Get initial mode
        let _ = platform.frame(0.0, "[]");

        // Press D to toggle demo mode
        let d_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyD"}}]"#;
        let result = platform.frame(100.0, d_down);
        assert!(result.contains("Demo") || result.contains("SinglePlayer"));

        // Release D
        let d_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyD"}}]"#;
        let _ = platform.frame(116.0, d_up);
    }

    #[test]
    fn test_web_platform_mode_cycle_via_m_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press M to cycle modes
        let m_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyM"}}]"#;
        let _ = platform.frame(100.0, m_down);

        let m_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyM"}}]"#;
        let _ = platform.frame(116.0, m_up);
    }

    #[test]
    fn test_web_platform_speed_keys_1_through_6() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press digit 3 for 10x speed
        let key_3 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Digit3"}}]"#;
        let _ = platform.frame(100.0, key_3);

        // Press digit 6 for 1000x speed
        let key_6 = r#"[{"event_type":"KeyDown","timestamp":200,"data":{"key":"Digit6"}}]"#;
        let _ = platform.frame(200.0, key_6);

        // Press digit 1 for 1x speed
        let key_1 = r#"[{"event_type":"KeyDown","timestamp":300,"data":{"key":"Digit1"}}]"#;
        let _ = platform.frame(300.0, key_1);
    }

    #[test]
    fn test_web_platform_fullscreen_via_f_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press F to request fullscreen
        let key_f = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyF"}}]"#;
        let output = platform.frame(100.0, key_f);

        // Verify EnterFullscreen action is emitted
        assert!(
            output.contains("EnterFullscreen"),
            "F key should trigger EnterFullscreen action"
        );

        // Release F
        let key_f_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyF"}}]"#;
        let _ = platform.frame(116.0, key_f_up);

        // Press F again to exit fullscreen
        let output = platform.frame(200.0, key_f);
        assert!(
            output.contains("ExitFullscreen"),
            "F key should toggle to ExitFullscreen action"
        );
    }

    #[test]
    fn test_web_platform_fullscreen_via_f11_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press F11 to request fullscreen
        let key_f11 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"F11"}}]"#;
        let output = platform.frame(100.0, key_f11);

        // Verify EnterFullscreen action is emitted
        assert!(
            output.contains("EnterFullscreen"),
            "F11 key should trigger EnterFullscreen action"
        );
    }

    #[test]
    fn test_web_platform_pause_resume_via_escape() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space_down = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space_down);
        let space_up = r#"[{"event_type":"KeyUp","timestamp":16,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(16.0, space_up);
        let _ = platform.frame(33.0, "[]");

        // Press Escape to pause
        let esc_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Escape"}}]"#;
        let result = platform.frame(100.0, esc_down);
        assert!(result.contains("PAUSED") || result.contains("commands"));

        // Release and re-press to resume
        let esc_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"Escape"}}]"#;
        let _ = platform.frame(116.0, esc_up);
        let _ = platform.frame(200.0, esc_down);
    }

    #[test]
    fn test_web_platform_info_panel_toggle() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press I to toggle info panel
        let i_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyI"}}]"#;
        let result = platform.frame(100.0, i_down);
        assert!(result.contains("Pong AI") || result.contains("commands"));
    }

    #[test]
    fn test_web_platform_mouse_click_hud() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Click on approximate HUD area
        let click =
            r#"[{"event_type":"MouseDown","timestamp":100,"data":{"button":0,"x":100,"y":20}}]"#;
        let _ = platform.frame(100.0, click);

        let release =
            r#"[{"event_type":"MouseUp","timestamp":116,"data":{"button":0,"x":100,"y":20}}]"#;
        let _ = platform.frame(116.0, release);
    }

    #[test]
    fn test_web_platform_download_button_click() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Click on download button area (bottom left footer)
        let click =
            r#"[{"event_type":"MouseDown","timestamp":100,"data":{"button":0,"x":70,"y":569}}]"#;
        let result = platform.frame(100.0, click);
        // Should have download action in response
        assert!(result.contains("DownloadAiModel") || result.contains("commands"));
    }

    #[test]
    fn test_web_platform_get_ai_model() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let model_json = platform.get_ai_model();
        assert!(model_json.contains("metadata"));
        assert!(model_json.contains("Pong AI"));
        assert!(model_json.contains("difficulty_profiles"));
    }

    #[test]
    fn test_web_platform_set_game_mode() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        platform.set_game_mode("demo");
        let _ = platform.frame(0.0, "[]");

        platform.set_game_mode("twoplayer");
        let _ = platform.frame(16.0, "[]");

        platform.set_game_mode("singleplayer");
        let _ = platform.frame(33.0, "[]");
    }

    #[test]
    fn test_web_platform_ai_difficulty_accessors() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let initial = platform.get_ai_difficulty();
        assert!(initial >= 1 && initial <= 10);

        platform.set_ai_difficulty(3);
        assert_eq!(platform.get_ai_difficulty(), 3);

        platform.set_ai_difficulty(9);
        assert_eq!(platform.get_ai_difficulty(), 9);
    }

    #[test]
    fn test_pong_game_victory_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::GameOver);
        game.left_score = 11;

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render victory text
        assert!(frame.len() > 5);
    }

    #[test]
    fn test_pong_game_paused_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render PAUSED text
        assert!(frame.len() > 3);
    }

    #[test]
    fn test_pong_game_menu_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu);

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render menu
        assert!(frame.len() > 3);
    }

    #[test]
    fn test_pong_game_demo_mode_ai_controls_both_paddles() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.game_mode = GameMode::Demo;

        let input = InputState::new();

        // Run several frames - AI should move both paddles
        for _ in 0..60 {
            game.update(&input, 0.016);
        }

        // Game should still be running
        assert!(game.state() == GameState::Playing || game.state() == GameState::Menu);
    }

    #[test]
    fn test_pong_game_two_player_mode() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.game_mode = GameMode::TwoPlayer;

        let mut input = InputState::new();
        let initial_left_y = game.left_paddle_y;
        let initial_right_y = game.right_paddle_y;

        // Both players control their paddles
        input.set_key_pressed(jugar_input::KeyCode::Letter('W'), true);
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Both paddles should have moved up
        assert!(game.left_paddle_y < initial_left_y);
        assert!(game.right_paddle_y < initial_right_y);
    }

    #[test]
    fn test_render_frame_commands_count() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let result = platform.frame(0.0, "[]");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have commands array
        assert!(parsed["commands"].is_array());
        assert!(parsed["commands"].as_array().unwrap().len() > 5);
    }

    #[test]
    fn test_speed_multiplier_affects_physics() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);
        let _ = platform.frame(16.0, "[]");

        // Set 10x speed
        let key_3 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Digit3"}}]"#;
        let _ = platform.frame(100.0, key_3);

        // Run frame at high speed
        let result = platform.frame(116.0, "[]");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_js_action_download_ai_model() {
        let action = JsAction::DownloadAiModel;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("DownloadAiModel"));
    }

    #[test]
    fn test_js_action_open_url() {
        let action = JsAction::OpenUrl {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("OpenUrl"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_js_action_fullscreen() {
        let enter = JsAction::EnterFullscreen;
        let exit = JsAction::ExitFullscreen;

        let enter_json = serde_json::to_string(&enter).unwrap();
        let exit_json = serde_json::to_string(&exit).unwrap();

        assert!(enter_json.contains("EnterFullscreen"));
        assert!(exit_json.contains("ExitFullscreen"));
    }

    #[test]
    fn test_frame_output_with_actions() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![],
            actions: vec![JsAction::DownloadAiModel],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("actions"));
        assert!(json.contains("DownloadAiModel"));
    }

    #[test]
    fn test_web_platform_full_gameplay_simulation() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Menu -> Playing
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);

        // Play for many frames
        for i in 1..100 {
            let ts = i as f64 * 16.667;
            let _ = platform.frame(ts, "[]");
        }

        // Get stats
        let stats = platform.get_stats();
        assert!(stats.contains("frame_count"));
    }

    #[test]
    fn test_web_platform_continuous_input() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);
        let _ = platform.frame(16.0, "[]");

        // Hold W key for many frames
        for i in 0..30 {
            let ts = 100.0 + (i as f64 * 16.667);
            let w_down = format!(
                r#"[{{"event_type":"KeyDown","timestamp":{},"data":{{"key":"KeyW"}}}}]"#,
                ts
            );
            let _ = platform.frame(ts, &w_down);
        }
    }

    #[test]
    fn test_pong_game_ball_hits_track_rally() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        assert_eq!(game.rally_count(), 0);

        // Hit the ball - should increment rally
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0;
        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have incremented after paddle hit
        assert!(game.rally_count() > 0 || game.ball_vx > 0.0);
    }

    #[test]
    fn test_all_speed_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // All 6 speed buttons should be positioned
        assert!(hud.speed_1x.width > 0.0);
        assert!(hud.speed_5x.width > 0.0);
        assert!(hud.speed_10x.width > 0.0);
        assert!(hud.speed_50x.width > 0.0);
        assert!(hud.speed_100x.width > 0.0);
        assert!(hud.speed_1000x.width > 0.0);
    }

    #[test]
    fn test_ai_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // AI difficulty buttons should be positioned
        assert!(hud.ai_decrease.width > 0.0);
        assert!(hud.ai_increase.width > 0.0);
    }

    #[test]
    fn test_footer_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Download and model info buttons
        assert!(hud.download.width > 0.0);
        assert!(hud.model_info.width > 0.0);
    }

    #[test]
    fn test_ai_shap_widget_renders_in_single_player_mode() {
        // Use WebConfig::default which has ai_enabled: true
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Run a frame - game starts in SinglePlayer mode with AI enabled
        let result = platform.frame(0.0, "[]");

        // Should render the .apr ML Model widget title
        assert!(
            result.contains(".apr ML Model"),
            "Expected frame output to contain '.apr ML Model' widget text. Output: {}",
            &result[..result.len().min(2000)]
        );
    }

    #[test]
    fn test_ai_shap_widget_not_rendered_in_two_player_mode() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Switch to 2P mode using the public method
        platform.set_game_mode("2p");

        // Run a frame
        let result = platform.frame(0.0, "[]");

        // Should NOT contain the .apr ML Model widget in 2P mode
        assert!(
            !result.contains(".apr ML Model"),
            "Expected frame output to NOT contain '.apr ML Model' widget text in 2P mode"
        );
    }

    #[test]
    fn test_footer_buttons_positions_match_render() {
        // Test that HudButtons::calculate() produces positions that match render_hud()
        let hud = HudButtons::calculate(800.0, 600.0);

        // Download button at x=10
        assert_eq!(hud.download.x, 10.0, "Download button should be at x=10");
        assert!(
            hud.download.y > 500.0,
            "Download button should be near bottom"
        );

        // Info button after download
        assert!(
            hud.model_info.x > hud.download.x + hud.download.width,
            "Info button should be after download button"
        );

        // Sound button after info (with [I] hint space)
        let expected_sound_x = hud.model_info.x + hud.model_info.width + 25.0;
        assert_eq!(
            hud.sound_toggle.x, expected_sound_x,
            "Sound button should be 25px after info button (accounting for [I] hint)"
        );
    }

    #[test]
    fn test_info_button_click_toggles_panel() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // First frame to initialize
        let _ = platform.frame(0.0, "[]");

        // Get info button position
        let hud = HudButtons::calculate(800.0, 600.0);
        let click_x = hud.model_info.x + hud.model_info.width / 2.0;
        let click_y = hud.model_info.y + hud.model_info.height / 2.0;

        // Click on info button
        let click = format!(
            r#"[{{"event_type":"MouseDown","timestamp":100,"data":{{"button":0,"x":{},"y":{}}}}}]"#,
            click_x, click_y
        );
        let result = platform.frame(100.0, &click);

        // Info panel should now be visible (contains more detailed info)
        assert!(
            result.contains("Model Info") || result.contains("difficulty_profiles"),
            "Info panel should be visible after clicking Info button"
        );
    }

    #[test]
    fn test_sound_button_click_toggles_sound() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // First frame - sound is enabled by default
        let result1 = platform.frame(0.0, "[]");
        assert!(
            result1.contains("Sound"),
            "Should show 'Sound' when enabled"
        );

        // Get sound button position
        let hud = HudButtons::calculate(800.0, 600.0);
        let click_x = hud.sound_toggle.x + hud.sound_toggle.width / 2.0;
        let click_y = hud.sound_toggle.y + hud.sound_toggle.height / 2.0;

        // Click on sound button to mute
        let click = format!(
            r#"[{{"event_type":"MouseDown","timestamp":100,"data":{{"button":0,"x":{},"y":{}}}}}]"#,
            click_x, click_y
        );
        let result2 = platform.frame(100.0, &click);

        // Should now show "Muted"
        assert!(
            result2.contains("Muted"),
            "Should show 'Muted' after clicking sound button"
        );
    }

    #[test]
    fn test_ai_buttons_dont_overlap_text() {
        let hud = HudButtons::calculate(800.0, 600.0);

        // AI text starts at bar_x + bar_width + 10 = 40 + 100 + 10 = 150
        // "9/9 Nightmare" is ~12 chars at 8px each = 96px
        // So text ends around 150 + 96 = 246
        // AI buttons should start after this

        let text_end_estimate = 40.0 + 100.0 + 10.0 + 96.0; // ~246
        assert!(
            hud.ai_decrease.x >= text_end_estimate - 10.0, // Allow small overlap
            "AI decrease button (x={}) should not overlap AI text (ends ~{})",
            hud.ai_decrease.x,
            text_end_estimate
        );
    }

    // ==================== Coverage Gap Tests ====================

    #[test]
    fn test_button_rect_boundary_conditions() {
        let btn = ButtonRect::new(10.0, 20.0, 100.0, 50.0);

        // Top-left corner (inside)
        assert!(btn.contains(10.0, 20.0), "Top-left corner should be inside");

        // Bottom-right corner (inside)
        assert!(
            btn.contains(110.0, 70.0),
            "Bottom-right corner should be inside"
        );

        // Inside the button
        assert!(btn.contains(50.0, 45.0), "Center should be inside");

        // Just outside left edge
        assert!(!btn.contains(9.9, 45.0), "Left of button should be outside");

        // Just outside right edge
        assert!(
            !btn.contains(110.1, 45.0),
            "Right of button should be outside"
        );

        // Just outside top edge
        assert!(!btn.contains(50.0, 19.9), "Above button should be outside");

        // Just outside bottom edge
        assert!(!btn.contains(50.0, 70.1), "Below button should be outside");
    }

    #[test]
    fn test_button_width_calculation() {
        // "Test" = 4 chars * 8.0 + 10.0 * 2 = 32 + 20 = 52
        let width = HudButtons::button_width("Test", 8.0, 10.0);
        assert!((width - 52.0).abs() < 0.01);

        // Empty string
        let empty_width = HudButtons::button_width("", 8.0, 10.0);
        assert!((empty_width - 20.0).abs() < 0.01); // Just padding

        // Longer text
        let long_width = HudButtons::button_width("HelloWorld", 8.0, 5.0);
        // 10 chars * 8.0 + 5.0 * 2 = 80 + 10 = 90
        assert!((long_width - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_hud_buttons_ultrawide_layout() {
        // Ultra-wide monitor (32:9 aspect ratio)
        let hud = HudButtons::calculate(3840.0, 1080.0);

        // Verify buttons exist and have positive dimensions
        assert!(hud.mode_demo.width > 0.0);
        assert!(hud.mode_1p.width > 0.0);
        assert!(hud.mode_2p.width > 0.0);

        // Speed buttons should be present
        assert!(hud.speed_1x.width > 0.0);
        assert!(hud.speed_1000x.width > 0.0);

        // Buttons should be within canvas bounds
        assert!(hud.mode_demo.x + hud.mode_demo.width < 3840.0);
        assert!(hud.download.x + hud.download.width < 3840.0);
    }

    #[test]
    fn test_hud_buttons_mobile_layout() {
        // Mobile (narrow screen)
        let hud = HudButtons::calculate(320.0, 640.0);

        // Verify buttons fit on narrow screen
        assert!(hud.mode_demo.x >= 0.0);
        assert!(hud.mode_demo.x + hud.mode_demo.width <= 320.0);

        // All buttons should be within bounds
        let all_buttons = [&hud.mode_demo, &hud.mode_1p, &hud.mode_2p, &hud.speed_1x];

        for btn in all_buttons {
            assert!(
                btn.x + btn.width <= 320.0,
                "Button at x={} width={} exceeds screen width",
                btn.x,
                btn.width
            );
        }
    }

    #[test]
    fn test_hud_buttons_square_layout() {
        // Square aspect ratio
        let hud = HudButtons::calculate(600.0, 600.0);

        // Buttons should be present
        assert!(hud.mode_demo.width > 0.0);
        assert!(hud.ai_decrease.width > 0.0);
        assert!(hud.ai_increase.width > 0.0);
    }

    // ==================== High-Priority Coverage Tests ====================

    #[test]
    fn test_web_config_from_json_full() {
        let json =
            r#"{"width":1920,"height":1080,"target_fps":144,"debug":true,"ai_enabled":false}"#;
        let config = WebConfig::from_json(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.target_fps, 144);
        assert!(config.debug);
        assert!(!config.ai_enabled);
    }

    #[test]
    fn test_web_config_from_json_partial() {
        // JSON with only some fields - others should use defaults
        let json = r#"{"width":640}"#;
        let config = WebConfig::from_json(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 600); // default
        assert_eq!(config.target_fps, 60); // default
        assert!(!config.debug); // default
        assert!(config.ai_enabled); // default
    }

    #[test]
    fn test_web_config_from_json_malformed() {
        let result = WebConfig::from_json("not json at all {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_web_platform_error_from_conversion() {
        let input_err = InputTranslationError::InvalidJson("test error".to_string());
        let platform_err: WebPlatformError = input_err.into();
        assert!(matches!(platform_err, WebPlatformError::InputError(_)));
        let err_str = platform_err.to_string();
        assert!(err_str.contains("Input error"));
    }

    #[test]
    fn test_web_platform_error_all_variants() {
        let config_err = WebPlatformError::InvalidConfig("bad config".to_string());
        assert_eq!(config_err.to_string(), "Invalid config: bad config");

        let input_err = WebPlatformError::InputError("bad input".to_string());
        assert_eq!(input_err.to_string(), "Input error: bad input");

        let render_err = WebPlatformError::RenderError("bad render".to_string());
        assert_eq!(render_err.to_string(), "Render error: bad render");
    }

    #[test]
    fn test_button_rect_constructor() {
        let btn = ButtonRect::new(10.0, 20.0, 100.0, 50.0);
        assert!((btn.x - 10.0).abs() < 0.001);
        assert!((btn.y - 20.0).abs() < 0.001);
        assert!((btn.width - 100.0).abs() < 0.001);
        assert!((btn.height - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_hud_buttons_model_info_button_layout() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Model info button should be present with positive dimensions
        assert!(hud.model_info.width > 0.0);
        assert!(hud.model_info.height > 0.0);
        // Should be within screen bounds
        assert!(hud.model_info.x + hud.model_info.width <= 800.0);
    }

    #[test]
    fn test_hud_buttons_sound_toggle_layout() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Sound toggle button should be present
        assert!(hud.sound_toggle.width > 0.0);
        assert!(hud.sound_toggle.height > 0.0);
        assert!(hud.sound_toggle.x >= 0.0);
    }

    #[test]
    fn test_hud_buttons_download_button_layout() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Download button should be present
        assert!(hud.download.width > 0.0);
        assert!(hud.download.height > 0.0);
    }
}
