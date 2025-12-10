//! YAML Schema definitions for age-tiered game creation.
//!
//! Three schema levels matching cognitive development stages:
//! - Level 1 (Ages 5-7): Single-level nesting, 50-word vocabulary
//! - Level 2 (Ages 8-10): Two-level nesting, 150 words, conditionals
//! - Level 3 (Ages 11+): Full power with .apr models

use crate::error::YamlError;
use crate::vocabulary::Vocabulary;
use serde::{Deserialize, Serialize};

/// Schema level for the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchemaLevel {
    /// Ages 5-7: Single-level nesting, basic vocabulary
    #[default]
    Level1,
    /// Ages 8-10: Two-level nesting, conditionals
    Level2,
    /// Ages 11+: Full power with .apr models
    Level3,
}

impl SchemaLevel {
    /// Get maximum allowed nesting depth for this schema level
    ///
    /// Per spec Section 9.1:
    /// - Level 1: 3 (root + 2 levels of user content)
    /// - Level 2: 5 (root + 4 levels of user content)
    /// - Level 3: 6 (root + 5 levels of user content)
    #[must_use]
    pub const fn max_nesting_depth(self) -> u8 {
        match self {
            Self::Level1 => 3,
            Self::Level2 => 5,
            Self::Level3 => 6,
        }
    }

    /// Get the age range for this level
    #[must_use]
    pub const fn age_range(self) -> &'static str {
        match self {
            Self::Level1 => "5-7",
            Self::Level2 => "8-10",
            Self::Level3 => "11+",
        }
    }

    /// Get vocabulary size for this level
    #[must_use]
    pub const fn vocabulary_size(self) -> usize {
        match self {
            Self::Level1 => 50,
            Self::Level2 => 150,
            Self::Level3 => 500, // Extended vocabulary
        }
    }
}

/// Detect the appropriate schema level for a YAML document
///
/// # Errors
///
/// Returns error if YAML is malformed
pub fn detect_level(yaml: &str) -> Result<SchemaLevel, YamlError> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| YamlError::SyntaxError {
            message: e.to_string(),
            line: e.location().map(|l| l.line()),
            column: e.location().map(|l| l.column()),
        })?;

    // Check for Level 3 indicators
    if has_level3_features(&value) {
        return Ok(SchemaLevel::Level3);
    }

    // Check for Level 2 indicators
    if has_level2_features(&value) {
        return Ok(SchemaLevel::Level2);
    }

    Ok(SchemaLevel::Level1)
}

fn has_level3_features(value: &serde_yaml::Value) -> bool {
    if let serde_yaml::Value::Mapping(map) = value {
        // Level 3 indicators: assets, entities, world, version
        return map.contains_key("assets")
            || map.contains_key("entities")
            || map.contains_key("world")
            || map.contains_key("version");
    }
    false
}

fn has_level2_features(value: &serde_yaml::Value) -> bool {
    if let serde_yaml::Value::Mapping(map) = value {
        // Level 2 indicators: characters (plural), rules, lives
        return map.contains_key("characters")
            || map.contains_key("rules")
            || map.contains_key("lives");
    }
    false
}

/// Level 1 Game Schema (Ages 5-7)
///
/// Single-level nesting maximum. Vocabulary from children's picture books.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Level1Game {
    /// Game name (3-20 chars, alphanumeric + hyphen)
    #[serde(default)]
    pub game: Option<String>,

    /// Main character type
    pub character: String,

    /// Movement type: arrows, touch, auto
    #[serde(default, rename = "move")]
    pub move_type: Option<String>,

    /// Background setting
    #[serde(default)]
    pub background: Option<String>,

    /// Background music
    #[serde(default)]
    pub music: Option<String>,

    /// Touch event configuration
    #[serde(default)]
    pub when_touch: Option<Level1TouchEvent>,

    /// Color (American spelling)
    #[serde(default)]
    pub color: Option<String>,

    /// Colour (British spelling alias)
    #[serde(default)]
    pub colour: Option<String>,
}

/// Touch event for Level 1
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level1TouchEvent {
    /// Target to touch (star, coin, gem, heart, apple)
    pub target: String,

    /// Sound to play
    #[serde(default)]
    pub sound: Option<String>,

    /// Score change (-9 to +9)
    #[serde(default)]
    pub score: Option<i8>,

    /// Target action (`new_place`, `disappear`)
    #[serde(default)]
    pub target_action: Option<String>,
}

/// Level 2 Game Schema (Ages 8-10)
///
/// Two-level nesting with conditionals and multiple characters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Level2Game {
    /// Game name
    pub game: Option<String>,

    /// Multiple character definitions (Level 2 feature)
    pub characters: Option<std::collections::HashMap<String, Level2Character>>,

    /// Single character (fallback to Level 1 style)
    pub character: Option<String>,

    /// Game rules with when/then structure (Level 2 feature)
    pub rules: Option<Vec<Level2Rule>>,

    /// Number of lives (1-9 for Level 2)
    pub lives: Option<u8>,

    /// Score goal to win
    pub score_goal: Option<u32>,

    /// Background setting from vocabulary
    pub background: Option<String>,

    /// Background music from vocabulary
    pub music: Option<String>,

    /// Touch event (Level 1 compatibility)
    pub when_touch: Option<Level1TouchEvent>,

    /// Movement type (Level 1 compatibility)
    #[serde(rename = "move")]
    pub move_type: Option<String>,
}

/// Character definition for Level 2
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level2Character {
    /// Character type (bunny, rocket, etc.)
    #[serde(rename = "type")]
    pub char_type: String,

    /// Movement type
    #[serde(default, rename = "move")]
    pub move_type: Option<String>,

    /// Movement speed (slow, normal, fast)
    #[serde(default)]
    pub speed: Option<String>,

    /// Movement pattern for AI
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Rule for Level 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level2Rule {
    /// Trigger condition ("player touches star", "score reaches 100")
    pub when: String,

    /// Actions to execute
    pub then: Vec<Level2Action>,
}

/// Action for Level 2 rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Level2Action {
    /// Add score with the amount to add
    AddScore {
        /// Amount to add to score
        add_score: i32,
    },
    /// Lose life with the amount to lose
    LoseLife {
        /// Number of lives to lose
        lose_life: i32,
    },
    /// Play sound
    Play {
        /// Sound to play
        play: String,
    },
    /// Show message/screen
    Show {
        /// Message or screen to show
        show: String,
    },
    /// Entity action (respawn, blink, etc.)
    EntityAction {
        /// Target entity name
        entity: String,
        /// Action to perform
        action: String,
    },
    /// Generic string action
    Simple(String),
}

/// Level 3 Game Schema (Ages 11+)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Game {
    /// Game name
    #[serde(default)]
    pub game: Option<String>,

    /// Schema version
    #[serde(default)]
    pub version: Option<u32>,

    /// Asset definitions
    #[serde(default)]
    pub assets: Option<Level3Assets>,

    /// World configuration
    #[serde(default)]
    pub world: Option<Level3World>,

    /// Entity definitions
    #[serde(default)]
    pub entities: Option<std::collections::HashMap<String, Level3Entity>>,

    /// Physics configuration
    #[serde(default)]
    pub physics: Option<Level3Physics>,

    /// Camera configuration
    #[serde(default)]
    pub camera: Option<Level3Camera>,

    /// UI configuration
    #[serde(default)]
    pub ui: Option<std::collections::HashMap<String, Level3UiElement>>,

    /// Level 2 compatibility: character definitions
    #[serde(default)]
    pub characters: Option<std::collections::HashMap<String, Level2Character>>,
    /// Level 2 compatibility: game rules
    #[serde(default)]
    pub rules: Option<Vec<Level2Rule>>,
    /// Level 2 compatibility: number of lives
    #[serde(default)]
    pub lives: Option<u8>,
    /// Level 2 compatibility: background setting
    #[serde(default)]
    pub background: Option<String>,
    /// Level 2 compatibility: background music
    #[serde(default)]
    pub music: Option<String>,
}

/// Asset definitions for Level 3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Assets {
    /// Sprite paths
    #[serde(default)]
    pub sprites: Option<std::collections::HashMap<String, String>>,

    /// Sound paths
    #[serde(default)]
    pub sounds: Option<std::collections::HashMap<String, String>>,

    /// AI model paths (.apr files)
    #[serde(default)]
    pub models: Option<std::collections::HashMap<String, String>>,
}

/// World configuration for Level 3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3World {
    /// World type (static, procedural)
    #[serde(rename = "type")]
    pub world_type: Option<String>,

    /// Generation algorithm (grid, wfc, noise)
    #[serde(default)]
    pub algorithm: Option<String>,

    /// Random seed
    #[serde(default)]
    pub seed: Option<SeedValue>,

    /// World size [width, height]
    #[serde(default)]
    pub size: Option<[u32; 2]>,

    /// Tile distribution
    #[serde(default)]
    pub tiles: Option<std::collections::HashMap<String, f32>>,
}

/// Seed value can be "auto" or a number
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SeedValue {
    /// Automatic seed
    Auto(String),
    /// Specific seed
    Number(u64),
}

/// Entity definition for Level 3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Entity {
    /// Sprite reference
    #[serde(default)]
    pub sprite: Option<String>,

    /// AI model reference
    #[serde(default)]
    pub ai: Option<String>,

    /// Entity components
    #[serde(default)]
    pub components: Option<Level3Components>,

    /// Control scheme
    #[serde(default)]
    pub controls: Option<Level3Controls>,
}

/// Component definitions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Components {
    /// Position [x, y]
    #[serde(default)]
    pub position: Option<[f32; 2]>,

    /// Health points
    #[serde(default)]
    pub health: Option<i32>,

    /// Damage dealt
    #[serde(default)]
    pub damage: Option<i32>,

    /// Movement speed
    #[serde(default)]
    pub speed: Option<f32>,

    /// Inventory
    #[serde(default)]
    pub inventory: Option<Vec<String>>,
}

/// Control scheme
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Controls {
    /// Movement keys
    #[serde(default, rename = "move")]
    pub move_keys: Option<String>,

    /// Attack key
    #[serde(default)]
    pub attack: Option<String>,
}

/// Physics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Physics {
    /// Physics type (grid, continuous)
    #[serde(rename = "type")]
    pub physics_type: Option<String>,

    /// Collision type (`tile_based`, `aabb`, `circle`)
    #[serde(default)]
    pub collision: Option<String>,
}

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3Camera {
    /// Entity to follow
    #[serde(default)]
    pub follow: Option<String>,

    /// Zoom level
    #[serde(default)]
    pub zoom: Option<f32>,
}

/// UI element configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Level3UiElement {
    /// Anchor position
    #[serde(default)]
    pub anchor: Option<String>,

    /// Data binding
    #[serde(default)]
    pub bind: Option<String>,
}

/// Validate a Level 1 game
///
/// # Errors
///
/// Returns validation errors
pub fn validate_level1(game: &Level1Game) -> Result<(), YamlError> {
    let vocab = Vocabulary::level1();

    // Validate character
    if !vocab.is_valid_for_category(&game.character, "characters") {
        let _suggestions = vocab.suggest_similar(&game.character, 5);
        return Err(YamlError::InvalidEnumValue {
            field: "character".to_string(),
            value: game.character.clone(),
            valid_options: vocab.words_in_category("characters"),
        });
    }

    // Validate background if present
    if let Some(bg) = &game.background {
        if !vocab.is_valid_for_category(bg, "backgrounds") {
            return Err(YamlError::InvalidEnumValue {
                field: "background".to_string(),
                value: bg.clone(),
                valid_options: vocab.words_in_category("backgrounds"),
            });
        }
    }

    // Validate music if present
    if let Some(music) = &game.music {
        if !vocab.is_valid_for_category(music, "music") {
            return Err(YamlError::InvalidEnumValue {
                field: "music".to_string(),
                value: music.clone(),
                valid_options: vocab.words_in_category("music"),
            });
        }
    }

    // Validate touch event
    if let Some(touch) = &game.when_touch {
        if !vocab.is_valid_for_category(&touch.target, "targets") {
            return Err(YamlError::InvalidEnumValue {
                field: "target".to_string(),
                value: touch.target.clone(),
                valid_options: vocab.words_in_category("targets"),
            });
        }

        if let Some(sound) = &touch.sound {
            if !vocab.is_valid_for_category(sound, "sounds") {
                return Err(YamlError::InvalidEnumValue {
                    field: "sound".to_string(),
                    value: sound.clone(),
                    valid_options: vocab.words_in_category("sounds"),
                });
            }
        }

        // Validate score range (-9 to +9 for Level 1)
        if let Some(score) = touch.score {
            if !(-9..=9).contains(&score) {
                return Err(YamlError::OutOfRange {
                    field: "score".to_string(),
                    min: -9,
                    max: 9,
                    value: i64::from(score),
                });
            }
        }
    }

    Ok(())
}

/// Validate a Level 2 game
///
/// # Errors
///
/// Returns validation errors
pub fn validate_level2(game: &Level2Game) -> Result<(), YamlError> {
    let vocab = Vocabulary::level2();

    // Validate characters
    if let Some(characters) = &game.characters {
        for (name, char_def) in characters {
            // Validate character type
            let char_type = &char_def.char_type;
            if !vocab.is_valid_for_category(char_type, "characters")
                && !vocab.is_valid_for_category(char_type, "characters_l2")
            {
                return Err(YamlError::InvalidEnumValue {
                    field: format!("characters.{name}.type"),
                    value: char_type.clone(),
                    valid_options: [
                        vocab.words_in_category("characters"),
                        vocab.words_in_category("characters_l2"),
                    ]
                    .concat(),
                });
            }

            // Validate pattern if present
            if let Some(pattern) = &char_def.pattern {
                if !vocab.is_valid_for_category(pattern, "patterns") {
                    return Err(YamlError::InvalidEnumValue {
                        field: format!("characters.{name}.pattern"),
                        value: pattern.clone(),
                        valid_options: vocab.words_in_category("patterns"),
                    });
                }
            }

            // Validate speed if present
            if let Some(speed) = &char_def.speed {
                if !vocab.is_valid_for_category(speed, "speed") {
                    return Err(YamlError::InvalidEnumValue {
                        field: format!("characters.{name}.speed"),
                        value: speed.clone(),
                        valid_options: vocab.words_in_category("speed"),
                    });
                }
            }
        }
    }

    // Validate lives range (1-9 for Level 2)
    if let Some(lives) = game.lives {
        if !(1..=9).contains(&lives) {
            return Err(YamlError::OutOfRange {
                field: "lives".to_string(),
                min: 1,
                max: 9,
                value: i64::from(lives),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Schema Level Tests
    #[test]
    fn test_schema_level_default() {
        assert_eq!(SchemaLevel::default(), SchemaLevel::Level1);
    }

    #[test]
    fn test_schema_level_max_nesting_depth() {
        assert_eq!(SchemaLevel::Level1.max_nesting_depth(), 3);
        assert_eq!(SchemaLevel::Level2.max_nesting_depth(), 5);
        assert_eq!(SchemaLevel::Level3.max_nesting_depth(), 6);
    }

    #[test]
    fn test_schema_level_age_range() {
        assert_eq!(SchemaLevel::Level1.age_range(), "5-7");
        assert_eq!(SchemaLevel::Level2.age_range(), "8-10");
        assert_eq!(SchemaLevel::Level3.age_range(), "11+");
    }

    #[test]
    fn test_schema_level_vocabulary_size() {
        assert_eq!(SchemaLevel::Level1.vocabulary_size(), 50);
        assert_eq!(SchemaLevel::Level2.vocabulary_size(), 150);
        assert_eq!(SchemaLevel::Level3.vocabulary_size(), 500);
    }

    // Detect Level Tests
    #[test]
    fn test_detect_level1() {
        let yaml = "character: bunny";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level1);
    }

    #[test]
    fn test_detect_level2_characters() {
        let yaml = "characters:\n  player:\n    type: bunny";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level2);
    }

    #[test]
    fn test_detect_level2_rules() {
        let yaml = "rules:\n  - when: test\n    then: []";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level2);
    }

    #[test]
    fn test_detect_level2_lives() {
        let yaml = "lives: 3";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level2);
    }

    #[test]
    fn test_detect_level3_assets() {
        let yaml = "assets:\n  models:\n    ai: test.apr";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level3);
    }

    #[test]
    fn test_detect_level3_entities() {
        let yaml = "entities:\n  player:\n    sprite: hero";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level3);
    }

    #[test]
    fn test_detect_level3_world() {
        let yaml = "world:\n  type: static";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level3);
    }

    #[test]
    fn test_detect_level3_version() {
        let yaml = "version: 1";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level3);
    }

    #[test]
    fn test_detect_level_invalid_yaml() {
        let yaml = "invalid: [yaml: error";
        assert!(detect_level(yaml).is_err());
    }

    #[test]
    fn test_detect_level_non_mapping() {
        let yaml = "- item1\n- item2";
        assert_eq!(detect_level(yaml).unwrap(), SchemaLevel::Level1);
    }

    // Validation Tests
    #[test]
    fn test_validate_level1_valid() {
        let game = Level1Game {
            character: "bunny".to_string(),
            background: Some("sky".to_string()),
            music: Some("gentle".to_string()),
            ..Default::default()
        };
        assert!(validate_level1(&game).is_ok());
    }

    #[test]
    fn test_validate_level1_invalid_character() {
        let game = Level1Game {
            character: "dinosaur".to_string(),
            ..Default::default()
        };
        assert!(validate_level1(&game).is_err());
    }

    #[test]
    fn test_validate_level1_invalid_background() {
        let game = Level1Game {
            character: "bunny".to_string(),
            background: Some("invalid_background".to_string()),
            ..Default::default()
        };
        let err = validate_level1(&game).unwrap_err();
        assert!(matches!(err, YamlError::InvalidEnumValue { field, .. } if field == "background"));
    }

    #[test]
    fn test_validate_level1_invalid_music() {
        let game = Level1Game {
            character: "bunny".to_string(),
            music: Some("invalid_music".to_string()),
            ..Default::default()
        };
        let err = validate_level1(&game).unwrap_err();
        assert!(matches!(err, YamlError::InvalidEnumValue { field, .. } if field == "music"));
    }

    #[test]
    fn test_validate_level1_invalid_touch_target() {
        let game = Level1Game {
            character: "bunny".to_string(),
            when_touch: Some(Level1TouchEvent {
                target: "invalid_target".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = validate_level1(&game).unwrap_err();
        assert!(matches!(err, YamlError::InvalidEnumValue { field, .. } if field == "target"));
    }

    #[test]
    fn test_validate_level1_invalid_sound() {
        let game = Level1Game {
            character: "bunny".to_string(),
            when_touch: Some(Level1TouchEvent {
                target: "star".to_string(),
                sound: Some("invalid_sound".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = validate_level1(&game).unwrap_err();
        assert!(matches!(err, YamlError::InvalidEnumValue { field, .. } if field == "sound"));
    }

    #[test]
    fn test_validate_level1_score_range() {
        let game = Level1Game {
            character: "bunny".to_string(),
            when_touch: Some(Level1TouchEvent {
                target: "star".to_string(),
                score: Some(100),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = validate_level1(&game).unwrap_err();
        assert!(matches!(err, YamlError::OutOfRange { .. }));
    }

    #[test]
    fn test_validate_level1_valid_score() {
        let game = Level1Game {
            character: "bunny".to_string(),
            when_touch: Some(Level1TouchEvent {
                target: "star".to_string(),
                score: Some(5),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(validate_level1(&game).is_ok());
    }

    // Level 2 Validation Tests
    #[test]
    fn test_validate_level2_valid() {
        let game = Level2Game::default();
        assert!(validate_level2(&game).is_ok());
    }

    #[test]
    fn test_validate_level2_invalid_lives_high() {
        let game = Level2Game {
            lives: Some(99),
            ..Default::default()
        };
        let err = validate_level2(&game).unwrap_err();
        assert!(matches!(err, YamlError::OutOfRange { .. }));
    }

    #[test]
    fn test_validate_level2_invalid_lives_zero() {
        let game = Level2Game {
            lives: Some(0),
            ..Default::default()
        };
        let err = validate_level2(&game).unwrap_err();
        assert!(matches!(err, YamlError::OutOfRange { .. }));
    }

    #[test]
    fn test_validate_level2_valid_lives() {
        let game = Level2Game {
            lives: Some(3),
            ..Default::default()
        };
        assert!(validate_level2(&game).is_ok());
    }

    #[test]
    fn test_validate_level2_invalid_character_type() {
        let mut characters = std::collections::HashMap::new();
        let _ = characters.insert(
            "player".to_string(),
            Level2Character {
                char_type: "invalid_type".to_string(),
                ..Default::default()
            },
        );
        let game = Level2Game {
            characters: Some(characters),
            ..Default::default()
        };
        assert!(validate_level2(&game).is_err());
    }

    #[test]
    fn test_validate_level2_invalid_pattern() {
        let mut characters = std::collections::HashMap::new();
        let _ = characters.insert(
            "player".to_string(),
            Level2Character {
                char_type: "bunny".to_string(),
                pattern: Some("invalid_pattern".to_string()),
                ..Default::default()
            },
        );
        let game = Level2Game {
            characters: Some(characters),
            ..Default::default()
        };
        assert!(validate_level2(&game).is_err());
    }

    #[test]
    fn test_validate_level2_invalid_speed() {
        let mut characters = std::collections::HashMap::new();
        let _ = characters.insert(
            "player".to_string(),
            Level2Character {
                char_type: "bunny".to_string(),
                speed: Some("invalid_speed".to_string()),
                ..Default::default()
            },
        );
        let game = Level2Game {
            characters: Some(characters),
            ..Default::default()
        };
        assert!(validate_level2(&game).is_err());
    }

    // Parsing Tests
    #[test]
    fn test_parse_level1() {
        let yaml = r"
character: bunny
background: sky
";
        let game: Level1Game = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(game.character, "bunny");
        assert_eq!(game.background, Some("sky".to_string()));
    }

    #[test]
    fn test_parse_level1_with_touch() {
        let yaml = r"
character: bunny
when_touch:
  target: star
  sound: happy
  score: 1
";
        let game: Level1Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.when_touch.is_some());
        let touch = game.when_touch.unwrap();
        assert_eq!(touch.target, "star");
        assert_eq!(touch.score, Some(1));
    }

    #[test]
    fn test_parse_level2() {
        let yaml = r"
game: test
characters:
  player:
    type: bunny
    speed: fast
lives: 3
";
        let game: Level2Game = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(game.lives, Some(3));
        assert!(game.characters.is_some());
    }

    #[test]
    fn test_parse_level2_with_rules() {
        let yaml = r"
rules:
  - when: player touches star
    then:
      - add_score: 10
";
        let game: Level2Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.rules.is_some());
        let rules = game.rules.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].when, "player touches star");
    }

    #[test]
    fn test_parse_level3() {
        let yaml = r"
game: dungeon
version: 1
assets:
  models:
    enemy_ai: models/goblin.apr
entities:
  player:
    sprite: hero
    ai: enemy_ai
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(game.version, Some(1));
        assert!(game.assets.is_some());
        assert!(game.entities.is_some());
    }

    #[test]
    fn test_parse_level3_with_world() {
        let yaml = r"
world:
  type: procedural
  algorithm: wfc
  seed: 12345
  size: [100, 100]
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.world.is_some());
        let world = game.world.unwrap();
        assert_eq!(world.world_type, Some("procedural".to_string()));
        assert_eq!(world.algorithm, Some("wfc".to_string()));
    }

    #[test]
    fn test_parse_level3_seed_auto() {
        let yaml = r"
world:
  seed: auto
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.world.is_some());
        let world = game.world.unwrap();
        assert!(matches!(world.seed, Some(SeedValue::Auto(_))));
    }

    #[test]
    fn test_parse_level3_with_physics() {
        let yaml = r"
physics:
  type: continuous
  collision: aabb
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.physics.is_some());
    }

    #[test]
    fn test_parse_level3_with_camera() {
        let yaml = r"
camera:
  follow: player
  zoom: 2.0
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.camera.is_some());
        let camera = game.camera.unwrap();
        assert_eq!(camera.follow, Some("player".to_string()));
    }

    #[test]
    fn test_parse_level3_with_ui() {
        let yaml = r"
ui:
  health_bar:
    anchor: top-left
    bind: player.health
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.ui.is_some());
    }

    #[test]
    fn test_parse_level3_entity_with_components() {
        let yaml = r"
entities:
  player:
    sprite: hero
    components:
      position: [100, 200]
      health: 100
      speed: 5.0
    controls:
      move: wasd
      attack: space
";
        let game: Level3Game = serde_yaml::from_str(yaml).unwrap();
        assert!(game.entities.is_some());
        let entities = game.entities.unwrap();
        let player = entities.get("player").unwrap();
        assert!(player.components.is_some());
        assert!(player.controls.is_some());
    }
}
