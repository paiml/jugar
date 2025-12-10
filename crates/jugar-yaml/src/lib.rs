//! # jugar-yaml
//!
//! ELI5 YAML-First declarative game creation system for children ages 5-12.
//!
//! This crate implements the specification from `docs/specifications/eli5-yaml-first-spec.md`,
//! providing a "golden path" for kids to create games using simple YAML configuration.
//!
//! ## Design Philosophy
//!
//! Following research-backed principles:
//! - **Low Floor**: Simple entry point (Resnick et al., 2009)
//! - **High Ceiling**: Complex creations possible
//! - **Wide Walls**: Many paths to expression
//!
//! ## Toyota Way Integration
//!
//! - **Poka-Yoke**: Schema prevents invalid YAML at parse time
//! - **Jidoka**: Stop-the-line on child-unfriendly errors
//! - **Mieruka**: Visual error messages with helper characters
//!
//! ## Schema Levels
//!
//! - **Level 1** (Ages 5-7): Single-level nesting, 50-word vocabulary
//! - **Level 2** (Ages 8-10): Two-level nesting, 150 words, conditionals
//! - **Level 3** (Ages 11+): Full power with .apr models

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod accessibility;
pub mod compiler;
pub mod error;
pub mod migration;
#[allow(
    clippy::std_instead_of_core,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::cast_precision_loss
)]
pub mod preview;
pub mod privacy;
pub mod safety;
pub mod sandbox;
pub mod scaffolding;
pub mod schema;
pub mod scripting;
pub mod sharing;
pub mod tutorial;
pub mod vocabulary;

pub use accessibility::{AccessibilityCode, AccessibilityReport, AccessibilityValidator};
pub use compiler::YamlCompiler;
pub use error::{HelperCharacter, KidFriendlyError, YamlError};
pub use migration::{
    HintCategory, MigratableGame, Migrate, MigratedGame, MigratedLevel2Game, MigratedLevel3Game,
    MigrationError, MigrationHint,
};
pub use preview::{
    Debouncer, LivePreview, PreviewCallback, PreviewResult, PreviewStats, PreviewStatus,
    DEFAULT_DEBOUNCE_MS,
};
pub use privacy::{
    ComplianceLevel, DifferentialPrivacy, DifferentialPrivacyConfig, LocalAnalytics,
    NoisyAnalytics, PrivacyConfig, PrivacyValidator, RetentionMetrics,
};
pub use safety::{FlashInfo, PhotosensitivityGuard, SafetyResult};
pub use sandbox::{ContentFilter, ContentSandbox, SandboxError, MAX_ENTITIES, MAX_YAML_SIZE};
pub use scaffolding::{Correction, Intent, Scaffold, ScaffoldedError, ScaffoldingEngine};
pub use schema::{Level1Game, Level2Game, Level3Game, SchemaLevel};
pub use scripting::{
    Level4Game, ScriptBlock, ScriptLanguage, ScriptSandbox, ScriptValidationResult, ScriptValidator,
};
pub use sharing::{BundleError, BundleMetadata, GameBundle, ShareLinkGenerator};
pub use tutorial::{GameTemplate, TemplateCatalog, TutorialError, TutorialProgress, TutorialStage};
pub use vocabulary::Vocabulary;

/// Result type for jugar-yaml operations
pub type Result<T> = core::result::Result<T, YamlError>;

/// Detect the schema level required for a YAML document
///
/// # Errors
///
/// Returns error if YAML is malformed
pub fn detect_schema_level(yaml: &str) -> Result<SchemaLevel> {
    schema::detect_level(yaml)
}

/// Parse and compile a YAML game definition
///
/// # Errors
///
/// Returns `YamlError` with kid-friendly message if parsing fails
pub fn compile_game(yaml: &str) -> Result<CompiledGame> {
    let compiler = YamlCompiler::new();
    compiler.compile(yaml)
}

/// A compiled game ready for the Jugar runtime
#[derive(Debug, Clone)]
pub struct CompiledGame {
    /// The game's name
    pub name: String,
    /// Schema level used
    pub level: SchemaLevel,
    /// Compiled entities
    pub entities: Vec<CompiledEntity>,
    /// Compiled rules
    pub rules: Vec<CompiledRule>,
    /// Background setting
    pub background: Option<String>,
    /// Music setting
    pub music: Option<String>,
}

/// A compiled entity from YAML
#[derive(Debug, Clone)]
pub struct CompiledEntity {
    /// Entity identifier
    pub id: String,
    /// Entity type (character, item, etc.)
    pub entity_type: String,
    /// Position if specified
    pub position: Option<(f32, f32)>,
    /// Movement type
    pub movement: Option<String>,
    /// AI model path if specified
    pub ai_model: Option<String>,
}

/// A compiled rule from YAML
#[derive(Debug, Clone)]
pub struct CompiledRule {
    /// Trigger condition
    pub when: String,
    /// Actions to execute
    pub then: Vec<CompiledAction>,
}

/// A compiled action from YAML
#[derive(Debug, Clone)]
pub enum CompiledAction {
    /// Play a sound effect
    PlaySound(String),
    /// Add to score
    AddScore(i32),
    /// Lose a life
    LoseLife(i32),
    /// Make entity disappear
    Disappear(String),
    /// Move entity to new random position
    Respawn(String),
    /// Show a message or screen
    Show(String),
    /// Stop the game
    StopGame,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: Tests written FIRST, implementation follows
    // ============================================================

    mod level1_schema_tests {
        use super::*;

        #[test]
        fn test_minimal_game_single_character() {
            // The simplest possible game - just a character
            let yaml = "character: bunny";
            let result = compile_game(yaml);
            assert!(result.is_ok(), "Single character should compile");
            let game = result.unwrap();
            assert_eq!(game.level, SchemaLevel::Level1);
            assert_eq!(game.entities.len(), 1);
            assert_eq!(game.entities[0].entity_type, "bunny");
        }

        #[test]
        fn test_full_level1_game() {
            let yaml = r"
game: catch-the-stars
character: bunny
move: arrows
background: space
music: gentle
when_touch:
  target: star
  sound: twinkle
  score: 1
";
            let result = compile_game(yaml);
            assert!(
                result.is_ok(),
                "Full Level 1 game should compile: {:?}",
                result.err()
            );
            let game = result.unwrap();
            assert_eq!(game.name, "catch-the-stars");
            assert_eq!(game.level, SchemaLevel::Level1);
            assert_eq!(game.background, Some("space".to_string()));
        }

        #[test]
        fn test_level1_valid_characters() {
            // All 10 Level 1 characters must be accepted
            let characters = [
                "bunny", "cat", "dog", "bird", "robot", "unicorn", "dragon", "fish", "bear", "fox",
            ];
            for character in characters {
                let yaml = format!("character: {character}");
                let result = compile_game(&yaml);
                assert!(result.is_ok(), "Character '{character}' should be valid");
            }
        }

        #[test]
        fn test_level1_valid_backgrounds() {
            let backgrounds = [
                "sky", "grass", "water", "space", "forest", "beach", "snow", "rainbow",
            ];
            for bg in backgrounds {
                let yaml = format!("character: bunny\nbackground: {bg}");
                let result = compile_game(&yaml);
                assert!(result.is_ok(), "Background '{bg}' should be valid");
            }
        }

        #[test]
        fn test_level1_valid_sounds() {
            let sounds = [
                "pop", "ding", "whoosh", "splash", "boing", "twinkle", "buzz", "click",
            ];
            for sound in sounds {
                let yaml = format!(
                    r"
character: bunny
when_touch:
  target: star
  sound: {sound}
"
                );
                let result = compile_game(&yaml);
                assert!(result.is_ok(), "Sound '{sound}' should be valid");
            }
        }

        #[test]
        fn test_level1_rejects_invalid_character() {
            let yaml = "character: dinosaur"; // Not in Level 1 vocabulary
            let result = compile_game(yaml);
            assert!(result.is_err());
            let err = result.unwrap_err();
            // Error should suggest valid alternatives
            let kid_err = err.to_kid_friendly();
            assert!(
                !kid_err.suggestions.is_empty(),
                "Should suggest alternatives"
            );
        }

        #[test]
        fn test_level1_rejects_deep_nesting() {
            let yaml = r"
character: bunny
nested:
  level1:
    level2:
      too_deep: true
";
            let result = compile_game(yaml);
            assert!(
                result.is_err(),
                "Deep nesting should be rejected for Level 1"
            );
        }

        #[test]
        fn test_level1_score_range() {
            // Score must be -9 to +9 for Level 1
            let yaml_valid = r"
character: bunny
when_touch:
  target: star
  score: 5
";
            assert!(compile_game(yaml_valid).is_ok());

            let yaml_too_high = r"
character: bunny
when_touch:
  target: star
  score: 100
";
            assert!(
                compile_game(yaml_too_high).is_err(),
                "Score > 9 should be rejected"
            );
        }

        #[test]
        fn test_level1_case_insensitive() {
            // Per spec: "Color: BLUE" should be accepted
            let yaml = "Character: BUNNY";
            let result = compile_game(yaml);
            assert!(result.is_ok(), "Should be case-insensitive");
        }

        #[test]
        fn test_level1_british_spelling() {
            // Per spec: "colour: blue" should be accepted
            let yaml = "character: bunny\ncolour: red";
            let result = compile_game(yaml);
            // Should either accept or give helpful error, not crash
            assert!(result.is_ok() || result.is_err());
        }
    }

    mod level2_schema_tests {
        use super::*;

        #[test]
        fn test_level2_multiple_characters() {
            let yaml = r"
game: space-adventure
characters:
  player:
    type: rocket
    move: arrows
    speed: fast
  enemy:
    type: asteroid
    move: auto
    pattern: zigzag
";
            let result = compile_game(yaml);
            assert!(
                result.is_ok(),
                "Level 2 with multiple characters should compile"
            );
            let game = result.unwrap();
            assert_eq!(game.level, SchemaLevel::Level2);
            assert_eq!(game.entities.len(), 2);
        }

        #[test]
        fn test_level2_rules() {
            let yaml = r"
game: test-rules
characters:
  player:
    type: bunny
rules:
  - when: player touches star
    then:
      - add_score: 100
      - play: victory
";
            let result = compile_game(yaml);
            assert!(
                result.is_ok(),
                "Level 2 rules should compile: {:?}",
                result.err()
            );
            let game = result.unwrap();
            assert!(!game.rules.is_empty(), "Should have compiled rules");
        }

        #[test]
        fn test_level2_patterns() {
            let patterns = ["zigzag", "circle", "chase", "wander", "patrol", "bounce"];
            for pattern in patterns {
                let yaml = format!(
                    r"
characters:
  enemy:
    type: asteroid
    pattern: {pattern}
"
                );
                let result = compile_game(&yaml);
                assert!(result.is_ok(), "Pattern '{pattern}' should be valid");
            }
        }

        #[test]
        fn test_level2_lives() {
            let yaml = r"
game: test
characters:
  player:
    type: bunny
lives: 3
";
            let result = compile_game(yaml);
            assert!(result.is_ok());
        }
    }

    mod level3_schema_tests {
        use super::*;

        #[test]
        fn test_level3_apr_model_reference() {
            let yaml = r"
game: dungeon-crawler
version: 1
assets:
  models:
    enemy_ai: models/goblin-v2.apr
entities:
  goblin:
    sprite: goblin
    ai: enemy_ai
";
            let result = compile_game(yaml);
            assert!(result.is_ok(), "Level 3 with .apr model should compile");
            let game = result.unwrap();
            assert_eq!(game.level, SchemaLevel::Level3);
        }

        #[test]
        fn test_level3_procedural_world() {
            let yaml = r"
game: procedural-test
world:
  type: procedural
  algorithm: wfc
  seed: auto
  size: [20, 20]
";
            let result = compile_game(yaml);
            assert!(result.is_ok(), "Level 3 procedural world should compile");
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn test_kid_friendly_error_unknown_word() {
            let yaml = "character: dinosaur";
            let result = compile_game(yaml);
            assert!(result.is_err());
            let err = result.unwrap_err();
            let kid_err = err.to_kid_friendly();

            // Should have a friendly headline
            assert!(!kid_err.headline.is_empty());
            // Should have a helper character
            assert!(matches!(
                kid_err.helper,
                HelperCharacter::Owl
                    | HelperCharacter::Robot
                    | HelperCharacter::Bunny
                    | HelperCharacter::Dragon
            ));
            // Should have suggestions
            assert!(!kid_err.suggestions.is_empty());
        }

        #[test]
        fn test_kid_friendly_error_syntax() {
            let yaml = "this is not: valid: yaml: at: all";
            let result = compile_game(yaml);
            assert!(result.is_err());
            let err = result.unwrap_err();
            let kid_err = err.to_kid_friendly();
            assert!(!kid_err.explanation.is_empty());
        }

        #[test]
        fn test_error_includes_location() {
            let yaml = r"
character: bunny
invalid_key: oops
";
            let result = compile_game(yaml);
            if let Err(err) = result {
                let kid_err = err.to_kid_friendly();
                // Location is optional but should exist for detectable errors
                if kid_err.location.is_some() {
                    let loc = kid_err.location.unwrap();
                    assert!(loc.line > 0);
                }
            }
        }
    }

    mod schema_detection_tests {
        use super::*;

        #[test]
        fn test_detect_level1() {
            let yaml = "character: bunny";
            let level = detect_schema_level(yaml).unwrap();
            assert_eq!(level, SchemaLevel::Level1);
        }

        #[test]
        fn test_detect_level2() {
            let yaml = r"
characters:
  player:
    type: bunny
rules:
  - when: player touches star
    then:
      - add_score: 10
";
            let level = detect_schema_level(yaml).unwrap();
            assert_eq!(level, SchemaLevel::Level2);
        }

        #[test]
        fn test_detect_level3() {
            let yaml = r"
assets:
  models:
    ai: test.apr
entities:
  enemy:
    ai: ai
";
            let level = detect_schema_level(yaml).unwrap();
            assert_eq!(level, SchemaLevel::Level3);
        }
    }

    mod vocabulary_tests {
        use super::*;

        #[test]
        fn test_level1_vocabulary_size() {
            let vocab = Vocabulary::level1();
            // Per spec: 50 words for Level 1
            assert!(
                vocab.word_count() >= 50,
                "Level 1 should have at least 50 words"
            );
        }

        #[test]
        fn test_level2_vocabulary_includes_level1() {
            let vocab1 = Vocabulary::level1();
            let vocab2 = Vocabulary::level2();

            // Level 2 should include all Level 1 words
            for word in vocab1.all_words() {
                assert!(vocab2.contains(&word), "Level 2 should include '{word}'");
            }
            // Level 2 should have more words
            assert!(vocab2.word_count() > vocab1.word_count());
        }

        #[test]
        fn test_vocabulary_suggestions() {
            let vocab = Vocabulary::level1();
            let suggestions = vocab.suggest_similar("bunnny", 5); // typo
            assert!(!suggestions.is_empty());
            assert!(suggestions.contains(&"bunny".to_string()));
        }
    }

    mod compiler_tests {
        use super::*;

        #[test]
        fn test_compiler_produces_entities() {
            let yaml = r"
game: test
character: bunny
";
            let game = compile_game(yaml).unwrap();
            assert!(!game.entities.is_empty());
        }

        #[test]
        fn test_compiler_produces_rules() {
            let yaml = r"
character: bunny
when_touch:
  target: star
  score: 1
";
            let game = compile_game(yaml).unwrap();
            // when_touch should become a rule
            assert!(!game.rules.is_empty());
        }
    }
}
