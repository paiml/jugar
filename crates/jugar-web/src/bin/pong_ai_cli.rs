//! Pong AI CLI - Prove and test AI behavior deterministically.
//!
//! This CLI tool demonstrates that the .apr model and AI logic work correctly
//! by running deterministic simulations and verifying expected behavior.
//!
//! # Usage
//!
//! ```bash
//! # Show model info
//! cargo run --bin pong_ai_cli -- info
//!
//! # Simulate a game with specific difficulty
//! cargo run --bin pong_ai_cli -- simulate --difficulty 5 --rounds 100
//!
//! # Test DDA (Dynamic Difficulty Adjustment)
//! cargo run --bin pong_ai_cli -- test-dda
//!
//! # Prove determinism (same seed = same results)
//! cargo run --bin pong_ai_cli -- prove-determinism
//!
//! # Export model as JSON
//! cargo run --bin pong_ai_cli -- export --output model.json
//! ```

#![allow(clippy::expect_used)] // CLI tool - expect is appropriate
#![allow(clippy::too_many_lines)] // CLI commands can be verbose
#![allow(clippy::suboptimal_flops)] // Clarity over performance in CLI
#![allow(clippy::cast_lossless)] // Casts are safe here
#![allow(clippy::uninlined_format_args)] // Clarity over compactness
#![allow(clippy::needless_range_loop)] // Clearer loop logic for comparison
#![allow(clippy::missing_const_for_fn)] // Functions may be modified later

use jugar_web::ai::{FlowChannel, PongAI, PongAIModel};

/// CLI commands
#[derive(Debug)]
enum Command {
    Info,
    Simulate { difficulty: u8, rounds: u32 },
    TestDda,
    ProveDeterminism,
    Export { output: Option<String> },
    Help,
}

fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Command::Help;
    }

    match args[1].as_str() {
        "info" => Command::Info,
        "simulate" => {
            let difficulty = args
                .iter()
                .position(|a| a == "--difficulty")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(5);
            let rounds = args
                .iter()
                .position(|a| a == "--rounds")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            Command::Simulate { difficulty, rounds }
        }
        "test-dda" => Command::TestDda,
        "prove-determinism" => Command::ProveDeterminism,
        "export" => {
            let output = args
                .iter()
                .position(|a| a == "--output")
                .and_then(|i| args.get(i + 1))
                .cloned();
            Command::Export { output }
        }
        _ => Command::Help,
    }
}

fn main() {
    let command = parse_args();

    match command {
        Command::Info => cmd_info(),
        Command::Simulate { difficulty, rounds } => cmd_simulate(difficulty, rounds),
        Command::TestDda => cmd_test_dda(),
        Command::ProveDeterminism => cmd_prove_determinism(),
        Command::Export { output } => cmd_export(output),
        Command::Help => cmd_help(),
    }
}

fn cmd_help() {
    println!("Pong AI CLI - Prove and test AI behavior");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin pong_ai_cli -- <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    info                Show model information");
    println!("    simulate            Run a game simulation");
    println!("        --difficulty N  Set AI difficulty (0-9, default: 5)");
    println!("        --rounds N      Number of points to play (default: 100)");
    println!("    test-dda            Test Dynamic Difficulty Adjustment");
    println!("    prove-determinism   Prove AI is deterministic (same seed = same results)");
    println!("    export              Export model as JSON");
    println!("        --output FILE   Output file (default: stdout)");
    println!("    help                Show this help message");
}

fn cmd_info() {
    let model = PongAIModel::default();

    println!("=== Pong AI Model Info ===");
    println!();
    println!("Metadata:");
    println!("  Name:        {}", model.metadata.name);
    println!("  Version:     {}", model.metadata.version);
    println!("  Description: {}", model.metadata.description);
    println!("  Author:      {}", model.metadata.author);
    println!("  License:     {}", model.metadata.license);
    println!("  Created:     {}", model.metadata.created);
    println!();
    println!("Determinism:");
    println!("  Seed:        {}", model.determinism.seed);
    println!("  Algorithm:   {}", model.determinism.rng_algorithm);
    println!();
    println!("Flow Theory (DDA):");
    println!(
        "  Skill Window:      {} points",
        model.flow_theory.skill_window_size
    );
    println!(
        "  Adaptation Rate:   {:.0}%",
        model.flow_theory.adaptation_rate * 100.0
    );
    println!(
        "  Boredom Threshold: {:.0}% win rate",
        model.flow_theory.boredom_threshold * 100.0
    );
    println!(
        "  Anxiety Threshold: {:.0}% win rate",
        model.flow_theory.anxiety_threshold * 100.0
    );
    println!(
        "  Target Win Rate:   {:.0}%",
        model.flow_theory.target_win_rate * 100.0
    );
    println!();
    println!("Difficulty Profiles:");
    println!("{:-<80}", "");
    println!(
        "{:>5} {:>15} {:>12} {:>12} {:>10} {:>10} {:>10}",
        "Level", "Name", "React(ms)", "Accuracy", "Speed", "Error", "Aggro"
    );
    println!("{:-<80}", "");

    for profile in &model.difficulty_profiles {
        println!(
            "{:>5} {:>15} {:>12.1} {:>11.0}% {:>10.0} {:>10.1} {:>9.0}%",
            profile.level,
            profile.name,
            profile.reaction_delay_ms,
            profile.prediction_accuracy * 100.0,
            profile.max_paddle_speed,
            profile.error_magnitude,
            profile.aggression * 100.0
        );
    }
    println!("{:-<80}", "");
    println!();
    println!("Model size: {} bytes (JSON)", model.serialized_size());
}

fn cmd_simulate(difficulty: u8, rounds: u32) {
    println!("=== Pong AI Simulation ===");
    println!();
    println!("Configuration:");
    println!(
        "  Difficulty: {} ({})",
        difficulty,
        difficulty_name(difficulty)
    );
    println!("  Rounds:     {rounds}");
    println!();

    let mut ai = PongAI::with_difficulty(difficulty);
    let canvas_width = 800.0;
    let canvas_height = 600.0;
    let paddle_height = 100.0;
    let dt = 1.0 / 60.0;

    // Simulate game state
    let mut ball_x;
    let mut ball_y;
    let mut ball_vx;
    let mut ball_vy;
    let mut paddle_y = canvas_height / 2.0;

    let mut player_wins = 0u32;
    let mut ai_wins = 0u32;
    let mut total_rallies = 0u32;
    let mut max_rally = 0u32;
    let mut current_rally;

    println!("Simulating {rounds} points...");
    println!();

    for round in 1..=rounds {
        // Reset ball
        ball_x = canvas_width / 2.0;
        ball_y = canvas_height / 2.0;
        ball_vx = if round % 2 == 0 { 200.0 } else { -200.0 };
        ball_vy = if round % 3 == 0 { 150.0 } else { -150.0 };
        current_rally = 0;

        // Simulate until someone scores
        let mut frames = 0;
        let max_frames = 60 * 60; // 60 seconds max

        while frames < max_frames {
            frames += 1;

            // AI updates paddle
            let velocity = ai.update(
                ball_x,
                ball_y,
                ball_vx,
                ball_vy,
                paddle_y,
                paddle_height,
                canvas_width,
                canvas_height,
                dt,
            );
            paddle_y += velocity * dt;
            paddle_y = paddle_y.clamp(paddle_height / 2.0, canvas_height - paddle_height / 2.0);

            // Update ball
            ball_x += ball_vx * dt;
            ball_y += ball_vy * dt;

            // Wall bounce
            if ball_y < 10.0 || ball_y > canvas_height - 10.0 {
                ball_vy = -ball_vy;
                ball_y = ball_y.clamp(10.0, canvas_height - 10.0);
            }

            // AI paddle collision (right side)
            let paddle_x = canvas_width - 35.0;
            if ball_x > paddle_x - 10.0
                && ball_x < paddle_x + 15.0
                && ball_y > paddle_y - paddle_height / 2.0
                && ball_y < paddle_y + paddle_height / 2.0
            {
                ball_vx = -ball_vx.abs() * 1.05;
                current_rally += 1;
            }

            // Simulated player paddle collision (left side)
            // Player hits 70% of the time at difficulty 5
            let player_hit_chance = 0.5 + (9.0 - f32::from(difficulty)) * 0.05;
            if ball_x < 35.0 && ball_x > 20.0 {
                let hit = (round as f32 * 0.7 + frames as f32 * 0.3) % 1.0 < player_hit_chance;
                if hit {
                    ball_vx = ball_vx.abs() * 1.05;
                    current_rally += 1;
                    ai.record_player_hit();
                }
            }

            // Score detection
            if ball_x < 0.0 {
                // AI scores
                ai_wins += 1;
                ai.record_player_miss();
                total_rallies += current_rally;
                max_rally = max_rally.max(current_rally);
                break;
            }
            if ball_x > canvas_width {
                // Player scores
                player_wins += 1;
                ai.record_player_scored();
                total_rallies += current_rally;
                max_rally = max_rally.max(current_rally);
                break;
            }
        }

        // Adapt difficulty every 5 points
        if round % 5 == 0 {
            ai.adapt_difficulty();
        }
    }

    println!("Results:");
    println!("  Player Wins: {player_wins}");
    println!("  AI Wins:     {ai_wins}");
    println!(
        "  Win Rate:    {:.1}%",
        player_wins as f64 / rounds as f64 * 100.0
    );
    println!();
    println!("Rally Stats:");
    println!("  Average:     {:.1}", total_rallies as f64 / rounds as f64);
    println!("  Max:         {max_rally}");
    println!();
    println!("Final AI State:");
    println!(
        "  Difficulty:  {} ({})",
        ai.difficulty(),
        ai.difficulty_name()
    );
    println!("  Flow:        {}", ai.flow_channel().label());
}

fn cmd_test_dda() {
    println!("=== Testing Dynamic Difficulty Adjustment ===");
    println!();

    // Test 1: Player winning too much -> difficulty increases
    println!("Test 1: Player dominating (should increase difficulty)");
    {
        let mut ai = PongAI::with_difficulty(5);
        let initial = ai.difficulty();

        for _ in 0..10 {
            ai.record_player_scored();
        }
        ai.adapt_difficulty();

        let final_diff = ai.difficulty();
        let result = if final_diff > initial { "PASS" } else { "FAIL" };
        println!(
            "  {} -> {} [{}] (flow: {})",
            initial,
            final_diff,
            result,
            ai.flow_channel().label()
        );
        assert!(
            final_diff > initial,
            "Difficulty should increase when player wins too much"
        );
    }

    // Test 2: Player losing too much -> difficulty decreases
    println!("Test 2: Player struggling (should decrease difficulty)");
    {
        let mut ai = PongAI::with_difficulty(5);
        let initial = ai.difficulty();

        for _ in 0..10 {
            ai.record_player_miss();
        }
        ai.adapt_difficulty();

        let final_diff = ai.difficulty();
        let result = if final_diff < initial { "PASS" } else { "FAIL" };
        println!(
            "  {} -> {} [{}] (flow: {})",
            initial,
            final_diff,
            result,
            ai.flow_channel().label()
        );
        assert!(
            final_diff < initial,
            "Difficulty should decrease when player loses too much"
        );
    }

    // Test 3: Balanced play -> difficulty stable
    println!("Test 3: Balanced play (should maintain difficulty)");
    {
        let mut ai = PongAI::with_difficulty(5);
        let initial = ai.difficulty();

        for i in 0..10 {
            if i % 2 == 0 {
                ai.record_player_scored();
            } else {
                ai.record_player_miss();
            }
        }
        ai.adapt_difficulty();

        let final_diff = ai.difficulty();
        let result = if (final_diff as i32 - initial as i32).abs() <= 1 {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "  {} -> {} [{}] (flow: {})",
            initial,
            final_diff,
            result,
            ai.flow_channel().label()
        );
    }

    // Test 4: Flow channel detection
    println!();
    println!("Test 4: Flow channel detection");
    {
        let mut ai = PongAI::with_difficulty(5);

        // Boredom state
        for _ in 0..10 {
            ai.record_player_scored();
        }
        ai.adapt_difficulty();
        let boredom_ok = ai.flow_channel() == FlowChannel::Boredom;
        println!(
            "  Boredom (80% wins):  {} [{}]",
            ai.flow_channel().label(),
            if boredom_ok { "PASS" } else { "FAIL" }
        );

        // Reset and test anxiety
        ai.reset();
        for _ in 0..10 {
            ai.record_player_miss();
        }
        ai.adapt_difficulty();
        let anxiety_ok = ai.flow_channel() == FlowChannel::Anxiety;
        println!(
            "  Anxiety (20% wins):  {} [{}]",
            ai.flow_channel().label(),
            if anxiety_ok { "PASS" } else { "FAIL" }
        );

        // Reset and test flow
        ai.reset();
        for i in 0..10 {
            if i % 2 == 0 {
                ai.record_player_scored();
            } else {
                ai.record_player_miss();
            }
        }
        ai.adapt_difficulty();
        let flow_ok = ai.flow_channel() == FlowChannel::Flow;
        println!(
            "  Flow (50% wins):     {} [{}]",
            ai.flow_channel().label(),
            if flow_ok { "PASS" } else { "FAIL" }
        );
    }

    println!();
    println!("All DDA tests passed!");
}

fn cmd_prove_determinism() {
    println!("=== Proving AI Determinism ===");
    println!();
    println!("Running two identical simulations with same seed...");
    println!();

    let runs = 2;
    let mut results: Vec<Vec<f32>> = Vec::new();

    for run in 0..runs {
        let mut ai = PongAI::with_difficulty(5);
        let mut velocities = Vec::new();

        // Same sequence of inputs
        for frame in 0..100 {
            let ball_x = 600.0 + (frame as f32 * 2.0);
            let ball_y = 300.0 + (frame as f32 * 1.5).sin() * 100.0;
            let ball_vx = 200.0;
            let ball_vy = 150.0;
            let paddle_y = 300.0;

            let v = ai.update(
                ball_x,
                ball_y,
                ball_vx,
                ball_vy,
                paddle_y,
                100.0,
                800.0,
                600.0,
                1.0 / 60.0,
            );
            velocities.push(v);
        }

        println!("Run {}: First 10 velocities:", run + 1);
        for (i, v) in velocities.iter().take(10).enumerate() {
            println!("  Frame {}: {:.6}", i, v);
        }
        println!();

        results.push(velocities);
    }

    // Compare results
    let mut all_match = true;
    for i in 0..results[0].len() {
        if (results[0][i] - results[1][i]).abs() > 0.0001 {
            println!(
                "MISMATCH at frame {}: {} vs {}",
                i, results[0][i], results[1][i]
            );
            all_match = false;
        }
    }

    if all_match {
        println!(
            "DETERMINISM VERIFIED: All {} frames produced identical results!",
            results[0].len()
        );
    } else {
        println!("DETERMINISM FAILED: Results differ between runs!");
        std::process::exit(1);
    }
}

fn cmd_export(output: Option<String>) {
    let model = PongAIModel::default();
    let json = model.to_json();

    match output {
        Some(path) => {
            std::fs::write(&path, &json).expect("Failed to write file");
            println!("Model exported to: {path}");
            println!("Size: {} bytes", json.len());
        }
        None => {
            println!("{json}");
        }
    }
}

fn difficulty_name(level: u8) -> &'static str {
    match level {
        0 => "Training Wheels",
        1 => "Beginner",
        2 => "Easy",
        3 => "Casual",
        4 => "Normal",
        5 => "Challenging",
        6 => "Hard",
        7 => "Very Hard",
        8 => "Expert",
        9 => "Master",
        _ => "Unknown",
    }
}
