//! Generate Pong AI model in .apr format.
//!
//! This binary creates the pre-trained AI model file that is loaded
//! at runtime by the Pong game.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin generate_pong_ai
//! ```
//!
//! This will create `assets/pong-ai-v1.apr` with the default difficulty
//! curve based on game design research.

#![allow(clippy::expect_used)] // CLI tool - expect is appropriate for error handling
#![allow(clippy::suboptimal_flops)] // mul_add less readable here

use aprender::format::{save, Compression, ModelType, SaveOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Pong AI Model - stored in .apr binary format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongAIModel {
    /// Model version string.
    pub version: String,
    /// Model name.
    pub name: String,
    /// Model description.
    pub description: String,
    /// Difficulty profiles for levels 0-9.
    pub difficulty_profiles: Vec<DifficultyProfile>,
    /// How quickly to adapt to player skill (0.0-1.0).
    pub skill_adaptation_rate: f32,
    /// Number of rallies to consider for skill estimation.
    pub performance_window_size: usize,
}

/// A single difficulty profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyProfile {
    /// Difficulty level (0-9).
    pub level: u8,
    /// Reaction delay in milliseconds.
    pub reaction_delay_ms: f32,
    /// Prediction accuracy (0.0-1.0).
    pub prediction_accuracy: f32,
    /// Maximum paddle speed in pixels/second.
    pub max_paddle_speed: f32,
    /// Error magnitude in pixels.
    pub error_magnitude: f32,
    /// Aggression factor (0.0-1.0).
    pub aggression: f32,
}

fn main() {
    println!("Generating Pong AI Model (.apr format)");
    println!("======================================");

    // Generate difficulty profiles based on research
    // Reference: Flow theory (Csikszentmihalyi), DDA research
    let profiles: Vec<DifficultyProfile> = (0..10)
        .map(|level| {
            let t = f32::from(level) / 9.0;

            DifficultyProfile {
                level,
                // Reaction delay: exponential decay from 500ms to 50ms
                // Research shows human reaction time ~150-300ms
                reaction_delay_ms: 500.0 * (1.0 - t).powi(2) + 50.0,

                // Prediction accuracy: linear 30% to 95%
                // Lower levels make more "mistakes" in predicting ball
                prediction_accuracy: 0.3 + 0.65 * t,

                // Max paddle speed: 200 to 600 px/s
                // Matches typical player skill progression
                max_paddle_speed: 200.0 + 400.0 * t,

                // Error magnitude: 50 to 5 pixels
                // Adds human-like imprecision
                error_magnitude: 50.0 * (1.0 - t).powi(2) + 5.0,

                // Aggression: 10% to 90%
                // Higher = more anticipation, lower = more reactive
                aggression: 0.1 + 0.8 * t,
            }
        })
        .collect();

    // Print profile summary
    println!("\nDifficulty Profiles:");
    println!("{:-<70}", "");
    println!(
        "{:>5} {:>12} {:>10} {:>10} {:>10} {:>10}",
        "Level", "Reaction(ms)", "Accuracy", "Speed", "Error", "Aggro"
    );
    println!("{:-<70}", "");

    for p in &profiles {
        println!(
            "{:>5} {:>12.1} {:>10.0}% {:>10.0} {:>10.1} {:>10.0}%",
            p.level,
            p.reaction_delay_ms,
            p.prediction_accuracy * 100.0,
            p.max_paddle_speed,
            p.error_magnitude,
            p.aggression * 100.0
        );
    }
    println!("{:-<70}", "");

    // Create the model
    let model = PongAIModel {
        version: "1.0.0".to_string(),
        name: "Pong Adaptive AI".to_string(),
        description: "Skill-matching AI opponent using Dynamic Difficulty Adjustment (DDA). \
                      Based on Csikszentmihalyi flow theory and game design research."
            .to_string(),
        difficulty_profiles: profiles,
        skill_adaptation_rate: 0.1,  // 10% adjustment per point
        performance_window_size: 10, // Consider last 10 rallies
    };

    // Determine output path
    let output_path = Path::new("assets/pong-ai-v1.apr");

    // Ensure assets directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create assets directory");
    }

    // Save with Zstd compression
    let options = SaveOptions::default().with_compression(Compression::ZstdDefault);

    match save(&model, ModelType::Custom, output_path, options) {
        Ok(()) => {
            // Get file size
            let metadata = std::fs::metadata(output_path).expect("Failed to read file metadata");
            let size_kb = metadata.len() as f64 / 1024.0;

            println!("\nModel saved successfully!");
            println!("  Path: {}", output_path.display());
            println!("  Size: {size_kb:.2} KB");

            // Verify by loading back
            println!("\nVerifying model...");
            match aprender::format::load::<PongAIModel>(output_path, ModelType::Custom) {
                Ok(loaded) => {
                    assert_eq!(loaded.version, model.version);
                    assert_eq!(loaded.difficulty_profiles.len(), 10);
                    println!("  Verification PASSED");
                    println!("  Name: {}", loaded.name);
                    println!("  Version: {}", loaded.version);
                    println!("  Profiles: {}", loaded.difficulty_profiles.len());
                }
                Err(e) => {
                    eprintln!("  Verification FAILED: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to save model: {e}");
            std::process::exit(1);
        }
    }

    println!("\nDone! Copy assets/pong-ai-v1.apr to examples/pong-web/assets/");
}
