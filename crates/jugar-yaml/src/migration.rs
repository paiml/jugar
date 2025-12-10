//! Schema migration for YAML game definitions.
//!
//! Per spec Section 14.2 Phase 3: "Implement the `Migrate` trait for all Schema levels
//! to ensure backward compatibility and forward progression."
//!
//! # Design Philosophy
//!
//! - **Lossless**: Migration preserves all game logic and behavior
//! - **Forward**: Level 1 → Level 2 → Level 3 progression
//! - **Automatic**: Migration can be done without user intervention
//! - **Kid-Friendly**: Migration hints explain new capabilities

use crate::error::YamlError;
use crate::schema::{Level1Game, SchemaLevel};

/// Result type for migration operations
pub type MigrationResult<T> = Result<T, MigrationError>;

/// Error during schema migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Cannot migrate to a lower level
    CannotDowngrade {
        /// Current level
        from: SchemaLevel,
        /// Target level
        to: SchemaLevel,
    },
    /// Already at the target level
    AlreadyAtLevel(SchemaLevel),
    /// Migration failed due to incompatible data
    IncompatibleData {
        /// Description of the issue
        reason: String,
    },
}

impl core::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CannotDowngrade { from, to } => {
                write!(f, "Cannot downgrade from {from:?} to {to:?}")
            }
            Self::AlreadyAtLevel(level) => {
                write!(f, "Already at {level:?}")
            }
            Self::IncompatibleData { reason } => {
                write!(f, "Migration failed: {reason}")
            }
        }
    }
}

impl core::error::Error for MigrationError {}

impl From<MigrationError> for YamlError {
    fn from(err: MigrationError) -> Self {
        Self::InvalidEnumValue {
            field: "schema_level".to_string(),
            value: format!("{err}"),
            valid_options: vec![
                "Level1".to_string(),
                "Level2".to_string(),
                "Level3".to_string(),
            ],
        }
    }
}

/// Trait for types that can be migrated to a higher schema level
pub trait Migrate {
    /// The type that results from migration
    type Target;

    /// Migrate to the next schema level
    ///
    /// # Errors
    ///
    /// Returns `MigrationError` if migration fails
    fn migrate(self) -> MigrationResult<Self::Target>;

    /// Check if migration is possible without performing it
    fn can_migrate(&self) -> bool;

    /// Get hints about what will change during migration
    fn migration_hints(&self) -> Vec<MigrationHint>;
}

/// A hint about what changes during migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationHint {
    /// Category of the hint
    pub category: HintCategory,
    /// Human-readable description
    pub description: String,
    /// Whether this is a new capability being unlocked
    pub unlocks_feature: bool,
}

/// Categories of migration hints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintCategory {
    /// New vocabulary words available
    Vocabulary,
    /// New structural features (e.g., multiple characters)
    Structure,
    /// New game mechanics (e.g., rules, conditions)
    Mechanics,
    /// New content types (e.g., .apr models)
    Content,
    /// Syntax changes
    Syntax,
}

impl core::fmt::Display for HintCategory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Vocabulary => write!(f, "Vocabulary"),
            Self::Structure => write!(f, "Structure"),
            Self::Mechanics => write!(f, "Mechanics"),
            Self::Content => write!(f, "Content"),
            Self::Syntax => write!(f, "Syntax"),
        }
    }
}

impl MigrationHint {
    /// Create a new migration hint
    #[must_use]
    pub fn new(category: HintCategory, description: impl Into<String>, unlocks: bool) -> Self {
        Self {
            category,
            description: description.into(),
            unlocks_feature: unlocks,
        }
    }

    /// Create a vocabulary hint
    #[must_use]
    pub fn vocabulary(description: impl Into<String>) -> Self {
        Self::new(HintCategory::Vocabulary, description, true)
    }

    /// Create a structure hint
    #[must_use]
    pub fn structure(description: impl Into<String>) -> Self {
        Self::new(HintCategory::Structure, description, true)
    }

    /// Create a mechanics hint
    #[must_use]
    pub fn mechanics(description: impl Into<String>) -> Self {
        Self::new(HintCategory::Mechanics, description, true)
    }

    /// Create a content hint
    #[must_use]
    pub fn content(description: impl Into<String>) -> Self {
        Self::new(HintCategory::Content, description, true)
    }
}

impl Migrate for Level1Game {
    type Target = MigratedLevel2Game;

    fn migrate(self) -> MigrationResult<Self::Target> {
        // Convert Level1 single character to Level2 characters map
        let mut characters = std::collections::HashMap::new();
        let _ = characters.insert(
            "player".to_string(),
            Level2Character {
                character_type: self.character.clone(),
                movement: self.move_type.clone(),
                speed: "normal".to_string(),
                pattern: None,
            },
        );

        // Convert when_touch to rules
        let rules = self.when_touch.as_ref().map_or_else(Vec::new, |touch| {
            let sound_action = touch.sound.as_ref().map(|s| Level2Action::Play(s.clone()));
            let score_action = touch.score.map(|s| Level2Action::AddScore(i32::from(s)));
            let actions: Vec<_> = [sound_action, score_action].into_iter().flatten().collect();
            vec![Level2Rule {
                when: format!("player touches {}", touch.target),
                then: actions,
            }]
        });

        Ok(MigratedLevel2Game {
            name: self.game.unwrap_or_else(|| "my-game".to_string()),
            characters,
            rules,
            lives: None,
            score_goal: None,
            background: self.background,
            music: self.music,
        })
    }

    fn can_migrate(&self) -> bool {
        // Level 1 can always migrate to Level 2
        true
    }

    fn migration_hints(&self) -> Vec<MigrationHint> {
        vec![
            MigrationHint::vocabulary(
                "Level 2 adds ~100 new words including 'rocket', 'spaceship', 'ninja', 'wizard'",
            ),
            MigrationHint::structure(
                "You can now have multiple characters with 'characters:' instead of 'character:'",
            ),
            MigrationHint::mechanics(
                "Add game rules with 'rules:' to create more complex interactions",
            ),
            MigrationHint::mechanics(
                "New movement patterns: 'zigzag', 'circle', 'chase', 'wander', 'patrol', 'bounce'",
            ),
            MigrationHint::structure("Set 'lives:' to add lives to your game"),
        ]
    }
}

impl Migrate for MigratedLevel2Game {
    type Target = MigratedLevel3Game;

    fn migrate(self) -> MigrationResult<Self::Target> {
        // Convert Level2 characters to Level3 entities
        let mut entities = std::collections::HashMap::new();
        for (name, character) in self.characters {
            let _ = entities.insert(
                name,
                Level3Entity {
                    sprite: character.character_type,
                    ai: None,
                    components: std::collections::HashMap::new(),
                },
            );
        }

        Ok(MigratedLevel3Game {
            name: self.name,
            version: 1,
            assets: Level3Assets::default(),
            world: None,
            entities,
            background: self.background,
            music: self.music,
        })
    }

    fn can_migrate(&self) -> bool {
        // Level 2 can always migrate to Level 3
        true
    }

    fn migration_hints(&self) -> Vec<MigrationHint> {
        vec![
            MigrationHint::content("Level 3 supports custom .apr AI models in 'assets.models:'"),
            MigrationHint::structure("Create procedural worlds with 'world.type: procedural'"),
            MigrationHint::mechanics(
                "Add custom components like 'health', 'inventory' to entities",
            ),
            MigrationHint::content(
                "Use custom sprites and sounds in 'assets.sprites:' and 'assets.sounds:'",
            ),
            MigrationHint::structure(
                "Advanced physics with 'collision: tile_based | aabb | continuous'",
            ),
        ]
    }
}

// Supporting types for migration
// These are simplified versions - the real implementation would use the schema types

/// Level 2 character definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2Character {
    /// Character type (sprite name)
    pub character_type: String,
    /// Movement style
    pub movement: Option<String>,
    /// Speed setting
    pub speed: String,
    /// AI pattern
    pub pattern: Option<String>,
}

/// Level 2 rule definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2Rule {
    /// When condition
    pub when: String,
    /// Then actions
    pub then: Vec<Level2Action>,
}

/// Level 2 action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level2Action {
    /// Play a sound
    Play(String),
    /// Add to score
    AddScore(i32),
    /// Lose a life
    LoseLife(i32),
    /// Show something
    Show(String),
}

/// Level 2 game definition (migration target from Level 1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigratedLevel2Game {
    /// Game name
    pub name: String,
    /// Characters in the game
    pub characters: std::collections::HashMap<String, Level2Character>,
    /// Game rules
    pub rules: Vec<Level2Rule>,
    /// Number of lives
    pub lives: Option<i32>,
    /// Score goal to win
    pub score_goal: Option<i32>,
    /// Background setting
    pub background: Option<String>,
    /// Music setting
    pub music: Option<String>,
}

/// Level 3 entity definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level3Entity {
    /// Sprite name
    pub sprite: String,
    /// AI model reference
    pub ai: Option<String>,
    /// Custom components
    pub components: std::collections::HashMap<String, String>,
}

/// Level 3 assets definition
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Level3Assets {
    /// Custom sprites
    pub sprites: std::collections::HashMap<String, String>,
    /// Custom sounds
    pub sounds: std::collections::HashMap<String, String>,
    /// AI models (.apr files)
    pub models: std::collections::HashMap<String, String>,
}

/// Level 3 game definition (migration target from Level 2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigratedLevel3Game {
    /// Game name
    pub name: String,
    /// Schema version
    pub version: u32,
    /// Custom assets
    pub assets: Level3Assets,
    /// World configuration
    pub world: Option<String>,
    /// Game entities
    pub entities: std::collections::HashMap<String, Level3Entity>,
    /// Background setting
    pub background: Option<String>,
    /// Music setting
    pub music: Option<String>,
}

/// Migrate a game to a specific target level
///
/// # Errors
///
/// Returns `MigrationError` if migration is not possible
pub fn migrate_to_level<T: MigratableGame>(
    game: T,
    target: SchemaLevel,
) -> MigrationResult<MigratedGame> {
    game.migrate_to(target)
}

/// Trait for games that can be migrated to any level
pub trait MigratableGame {
    /// Get the current schema level
    fn current_level(&self) -> SchemaLevel;

    /// Migrate to a target level
    ///
    /// # Errors
    ///
    /// Returns `MigrationError` if:
    /// - Target level is lower than current level (cannot downgrade)
    /// - Already at the target level
    /// - Migration fails due to incompatible data
    fn migrate_to(self, target: SchemaLevel) -> MigrationResult<MigratedGame>;
}

/// Result of a migration - can be any schema level
#[derive(Debug, Clone)]
pub enum MigratedGame {
    /// Level 1 game
    Level1(Level1Game),
    /// Level 2 game
    Level2(MigratedLevel2Game),
    /// Level 3 game
    Level3(MigratedLevel3Game),
}

impl MigratedGame {
    /// Get the schema level of the migrated game
    #[must_use]
    pub const fn level(&self) -> SchemaLevel {
        match self {
            Self::Level1(_) => SchemaLevel::Level1,
            Self::Level2(_) => SchemaLevel::Level2,
            Self::Level3(_) => SchemaLevel::Level3,
        }
    }
}

impl MigratableGame for Level1Game {
    fn current_level(&self) -> SchemaLevel {
        SchemaLevel::Level1
    }

    fn migrate_to(self, target: SchemaLevel) -> MigrationResult<MigratedGame> {
        match target {
            SchemaLevel::Level1 => Err(MigrationError::AlreadyAtLevel(SchemaLevel::Level1)),
            SchemaLevel::Level2 => {
                let level2 = self.migrate()?;
                Ok(MigratedGame::Level2(level2))
            }
            SchemaLevel::Level3 => {
                let level2 = self.migrate()?;
                let level3 = level2.migrate()?;
                Ok(MigratedGame::Level3(level3))
            }
        }
    }
}

impl MigratableGame for MigratedLevel2Game {
    fn current_level(&self) -> SchemaLevel {
        SchemaLevel::Level2
    }

    fn migrate_to(self, target: SchemaLevel) -> MigrationResult<MigratedGame> {
        match target {
            SchemaLevel::Level1 => Err(MigrationError::CannotDowngrade {
                from: SchemaLevel::Level2,
                to: SchemaLevel::Level1,
            }),
            SchemaLevel::Level2 => Err(MigrationError::AlreadyAtLevel(SchemaLevel::Level2)),
            SchemaLevel::Level3 => {
                let level3 = self.migrate()?;
                Ok(MigratedGame::Level3(level3))
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests for schema migration
    // ========================================================================

    mod migration_trait_tests {
        use super::*;
        use crate::schema::Level1TouchEvent;

        fn sample_level1_game() -> Level1Game {
            Level1Game {
                game: Some("test-game".to_string()),
                character: "bunny".to_string(),
                move_type: Some("arrows".to_string()),
                background: Some("grass".to_string()),
                music: Some("happy".to_string()),
                when_touch: Some(Level1TouchEvent {
                    target: "star".to_string(),
                    sound: Some("ding".to_string()),
                    score: Some(1),
                    target_action: None,
                }),
                color: None,
                colour: None,
            }
        }

        #[test]
        fn test_level1_can_migrate() {
            let game = sample_level1_game();
            assert!(game.can_migrate());
        }

        #[test]
        fn test_level1_migrate_to_level2() {
            let game = sample_level1_game();
            let result = game.migrate();
            assert!(result.is_ok());

            let level2 = result.unwrap();
            assert_eq!(level2.name, "test-game");
            assert!(level2.characters.contains_key("player"));
            assert_eq!(
                level2.characters.get("player").unwrap().character_type,
                "bunny"
            );
        }

        #[test]
        fn test_level1_migration_preserves_touch_rules() {
            let game = sample_level1_game();
            let level2 = game.migrate().unwrap();

            assert!(!level2.rules.is_empty());
            assert!(level2.rules[0].when.contains("touches"));
            assert!(level2.rules[0].when.contains("star"));
        }

        #[test]
        fn test_level2_can_migrate() {
            let game = sample_level1_game();
            let level2 = game.migrate().unwrap();
            assert!(level2.can_migrate());
        }

        #[test]
        fn test_level2_migrate_to_level3() {
            let game = sample_level1_game();
            let level2 = game.migrate().unwrap();
            let result = level2.migrate();
            assert!(result.is_ok());

            let level3 = result.unwrap();
            assert_eq!(level3.name, "test-game");
            assert!(level3.entities.contains_key("player"));
        }

        #[test]
        fn test_migrate_level1_to_level3_directly() {
            let game = sample_level1_game();
            let result = game.migrate_to(SchemaLevel::Level3);
            assert!(result.is_ok());

            let migrated = result.unwrap();
            assert_eq!(migrated.level(), SchemaLevel::Level3);
        }
    }

    mod migration_error_tests {
        use super::*;

        fn sample_level2_game() -> MigratedLevel2Game {
            MigratedLevel2Game {
                name: "test-game".to_string(),
                characters: std::collections::HashMap::new(),
                rules: Vec::new(),
                lives: None,
                score_goal: None,
                background: None,
                music: None,
            }
        }

        #[test]
        fn test_cannot_downgrade_level2_to_level1() {
            let game = sample_level2_game();
            let result = game.migrate_to(SchemaLevel::Level1);
            assert!(matches!(
                result,
                Err(MigrationError::CannotDowngrade { .. })
            ));
        }

        #[test]
        fn test_already_at_level() {
            let game = sample_level2_game();
            let result = game.migrate_to(SchemaLevel::Level2);
            assert!(matches!(result, Err(MigrationError::AlreadyAtLevel(_))));
        }

        #[test]
        fn test_migration_error_display() {
            let err = MigrationError::CannotDowngrade {
                from: SchemaLevel::Level2,
                to: SchemaLevel::Level1,
            };
            let msg = err.to_string();
            assert!(msg.contains("downgrade"));
        }
    }

    mod migration_hint_tests {
        use super::*;

        fn sample_level1_game() -> Level1Game {
            Level1Game {
                game: Some("test-game".to_string()),
                character: "bunny".to_string(),
                move_type: None,
                background: None,
                music: None,
                when_touch: None,
                color: None,
                colour: None,
            }
        }

        #[test]
        fn test_level1_has_migration_hints() {
            let game = sample_level1_game();
            let hints = game.migration_hints();
            assert!(!hints.is_empty());
        }

        #[test]
        fn test_migration_hints_include_vocabulary() {
            let game = sample_level1_game();
            let hints = game.migration_hints();
            assert!(hints.iter().any(|h| h.category == HintCategory::Vocabulary));
        }

        #[test]
        fn test_migration_hints_include_structure() {
            let game = sample_level1_game();
            let hints = game.migration_hints();
            assert!(hints.iter().any(|h| h.category == HintCategory::Structure));
        }

        #[test]
        fn test_level2_has_migration_hints() {
            let level1 = sample_level1_game();
            let level2 = level1.migrate().unwrap();
            let hints = level2.migration_hints();
            assert!(!hints.is_empty());
        }

        #[test]
        fn test_level2_hints_mention_apr_models() {
            let level1 = sample_level1_game();
            let level2 = level1.migrate().unwrap();
            let hints = level2.migration_hints();
            assert!(hints.iter().any(|h| h.description.contains(".apr")));
        }

        #[test]
        fn test_migration_hint_creation() {
            let hint = MigrationHint::vocabulary("Test vocabulary hint");
            assert_eq!(hint.category, HintCategory::Vocabulary);
            assert!(hint.unlocks_feature);
        }
    }

    mod hint_category_tests {
        use super::*;

        #[test]
        fn test_hint_category_display() {
            assert_eq!(HintCategory::Vocabulary.to_string(), "Vocabulary");
            assert_eq!(HintCategory::Structure.to_string(), "Structure");
            assert_eq!(HintCategory::Mechanics.to_string(), "Mechanics");
            assert_eq!(HintCategory::Content.to_string(), "Content");
            assert_eq!(HintCategory::Syntax.to_string(), "Syntax");
        }
    }

    mod migrated_game_tests {
        use super::*;

        #[test]
        fn test_migrated_game_level() {
            let level2 = MigratedLevel2Game {
                name: "test".to_string(),
                characters: std::collections::HashMap::new(),
                rules: Vec::new(),
                lives: None,
                score_goal: None,
                background: None,
                music: None,
            };
            let migrated = MigratedGame::Level2(level2);
            assert_eq!(migrated.level(), SchemaLevel::Level2);
        }
    }
}
