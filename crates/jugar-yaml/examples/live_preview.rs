//! `LivePreview` Example - Real-time YAML Hot-Reload
//!
//! Demonstrates the `LivePreview` system for instant feedback when
//! editing YAML game definitions.
//!
//! # Running
//!
//! ```bash
//! cargo run --example live_preview -p jugar-yaml
//! ```
//!
//! # Features
//!
//! - Debounced compilation (150ms default)
//! - Instant validation feedback
//! - Kid-friendly error messages
//! - Performance statistics

#![allow(
    clippy::unwrap_used,
    clippy::uninlined_format_args,
    clippy::needless_raw_string_hashes,
    clippy::std_instead_of_core,
    clippy::single_match
)]

use jugar_yaml::{
    compile_game, detect_schema_level, Debouncer, LivePreview, PreviewResult, PreviewStats,
    SchemaLevel, DEFAULT_DEBOUNCE_MS,
};
use std::time::Duration;

fn main() {
    println!("=== LivePreview Demo ===\n");

    // Demo 1: Basic compilation workflow
    demo_basic_compilation();

    // Demo 2: Debouncer for rapid edits
    demo_debouncer();

    // Demo 3: LivePreview system
    demo_live_preview();

    // Demo 4: Error handling
    demo_error_handling();

    println!("\n=== LivePreview Demo Complete ===");
}

fn demo_basic_compilation() {
    println!("--- Demo 1: Basic Compilation ---\n");

    let yaml = r#"
game: catch-stars
character: bunny
move: arrows
background: space
when_touch:
  target: star
  sound: twinkle
  score: 1
"#;

    // Detect schema level first
    let level = detect_schema_level(yaml).unwrap();
    println!("Detected schema level: {:?}", level);
    assert_eq!(level, SchemaLevel::Level1);

    // Compile the game
    let game = compile_game(yaml).unwrap();
    println!("Game name: {}", game.name);
    println!("Entities: {}", game.entities.len());
    println!("Rules: {}", game.rules.len());
    println!();
}

fn demo_debouncer() {
    println!("--- Demo 2: Debouncer for Rapid Edits ---\n");

    let mut debouncer = Debouncer::new(Duration::from_millis(DEFAULT_DEBOUNCE_MS));

    // Simulate rapid keystrokes
    println!("Simulating rapid edits...");

    // First keystroke - triggers immediately (no previous)
    let result1 = debouncer.schedule();
    println!("  Edit 1: should execute = {}", result1);

    // Rapid edits within debounce window
    std::thread::sleep(Duration::from_millis(50));
    let result2 = debouncer.schedule();
    println!("  Edit 2 (50ms later): should execute = {}", result2);

    // Check if pending
    println!("  Has pending: {}", debouncer.is_pending());

    // Wait for debounce period
    std::thread::sleep(Duration::from_millis(160));
    let result3 = debouncer.schedule();
    println!("  Edit 3 (210ms total): should execute = {}", result3);

    // Reset and try again
    debouncer.reset();
    let result4 = debouncer.schedule();
    println!("  After reset: should execute = {}", result4);

    println!();
}

fn demo_live_preview() {
    println!("--- Demo 3: LivePreview System ---\n");

    let mut preview = LivePreview::new();

    // Step 1: Compile a simple valid YAML
    println!("Step 1: Compiling valid YAML...");
    let result = preview.on_yaml_change("character: bunny");
    match &result {
        PreviewResult::Success { game, compile_time } => {
            println!("  SUCCESS: Compiled '{}' in {:?}", game.name, compile_time);
        }
        PreviewResult::Error { errors } => {
            println!("  ERROR: {} errors", errors.len());
        }
        PreviewResult::Debounced => {
            println!("  DEBOUNCED: Waiting...");
        }
    }

    // Step 2: Try to compile again immediately (should be debounced)
    println!("\nStep 2: Immediate second call...");
    let result2 = preview.on_yaml_change("character: bunny\nbackground: space");
    println!(
        "  Result: {:?}",
        if result2.is_debounced() {
            "Debounced"
        } else {
            "Compiled"
        }
    );
    println!("  Has pending: {}", preview.has_pending());

    // Step 3: Force immediate compilation
    println!("\nStep 3: Force immediate compilation...");
    let result3 = preview.compile_now("character: cat\nbackground: forest");
    if result3.is_success() {
        println!("  SUCCESS: Force compiled");
    }

    // Step 4: Try invalid YAML
    println!("\nStep 4: Compile invalid YAML...");
    let result4 = preview.compile_now("character: dinosaur"); // Invalid character
    match &result4 {
        PreviewResult::Error { errors } => {
            println!("  ERROR: {} errors detected", errors.len());
            // Last valid game is preserved
            if let Some(last_game) = preview.last_valid_game() {
                println!("  Last valid game: '{}'", last_game.name);
            }
        }
        _ => {}
    }

    // Final stats
    let stats: PreviewStats = (&preview).into();
    println!("\nPreview Statistics:");
    println!("  Total compilations: {}", stats.total_compilations);
    println!("  Successful: {}", stats.successful_compilations);
    println!("  Success rate: {:.1}%", stats.success_rate * 100.0);
    println!();
}

fn demo_error_handling() {
    println!("--- Demo 4: Kid-Friendly Error Handling ---\n");

    let invalid_yaml = r#"character: dinosaur"#;

    match compile_game(invalid_yaml) {
        Ok(_) => println!("Unexpected success"),
        Err(err) => {
            let kid_err = err.to_kid_friendly();
            println!("Helper: {}", kid_err.helper.emoji());
            println!("Headline: {}", kid_err.headline);
            println!("Explanation: {}", kid_err.explanation);
            println!("Suggestions:");
            for suggestion in &kid_err.suggestions {
                println!("  - {}", suggestion);
            }
        }
    }
    println!();
}
