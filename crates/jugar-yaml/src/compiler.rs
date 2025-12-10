//! YAML-to-Game compiler for ELI5 game creation.
//!
//! Transforms validated YAML into a `CompiledGame` ready for the Jugar runtime.

use crate::error::YamlError;
use crate::schema::{
    self, validate_level1, validate_level2, Level1Game, Level2Game, Level3Game, SchemaLevel,
};
use crate::vocabulary::Vocabulary;
use crate::{CompiledAction, CompiledEntity, CompiledGame, CompiledRule};

/// YAML game compiler
#[derive(Debug, Default)]
pub struct YamlCompiler {
    /// Strict mode rejects any unknown fields (reserved for future use)
    #[allow(dead_code)]
    strict: bool,
}

impl YamlCompiler {
    /// Create a new compiler with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self { strict: false }
    }

    /// Create a strict compiler that rejects unknown fields
    #[must_use]
    pub const fn strict() -> Self {
        Self { strict: true }
    }

    /// Compile a YAML string into a game
    ///
    /// # Errors
    ///
    /// Returns `YamlError` with kid-friendly message if compilation fails
    pub fn compile(&self, yaml: &str) -> Result<CompiledGame, YamlError> {
        // Normalize YAML (case-insensitive keys)
        let normalized = normalize_yaml(yaml)?;

        // Detect schema level
        let level = schema::detect_level(&normalized)?;

        // Check nesting depth - per spec Section 9.1:
        // Level 1: max 2 levels (single-level nesting)
        // Level 2: max 3 levels (two-level nesting)
        // Level 3: max 4 levels (full power)
        // But YAML structures like rules arrays add depth, so we allow +1 for internal structures
        let max_depth = match level {
            SchemaLevel::Level1 => 3, // 2 + 1 for YAML structure
            SchemaLevel::Level2 => 5, // 3 + 2 for rules with then arrays
            SchemaLevel::Level3 => 6, // 4 + 2 for complex entities
        };
        check_nesting_depth(&normalized, max_depth)?;

        // Parse and validate based on level
        match level {
            SchemaLevel::Level1 => self.compile_level1(&normalized),
            SchemaLevel::Level2 => self.compile_level2(&normalized),
            SchemaLevel::Level3 => self.compile_level3(&normalized),
        }
    }

    #[allow(clippy::unused_self)]
    fn compile_level1(&self, yaml: &str) -> Result<CompiledGame, YamlError> {
        let game: Level1Game = parse_yaml(yaml)?;

        // Validate
        validate_level1(&game)?;

        // Build compiled game
        let mut entities = Vec::new();
        let mut rules = Vec::new();

        // Main character entity
        entities.push(CompiledEntity {
            id: "player".to_string(),
            entity_type: game.character.clone(),
            position: None,
            movement: game.move_type.clone(),
            ai_model: None,
        });

        // Convert when_touch to a rule
        if let Some(touch) = &game.when_touch {
            let mut actions = Vec::new();

            if let Some(sound) = &touch.sound {
                actions.push(CompiledAction::PlaySound(sound.clone()));
            }

            if let Some(score) = touch.score {
                actions.push(CompiledAction::AddScore(i32::from(score)));
            }

            if let Some(action) = &touch.target_action {
                match action.as_str() {
                    "disappear" => actions.push(CompiledAction::Disappear(touch.target.clone())),
                    "new_place" => actions.push(CompiledAction::Respawn(touch.target.clone())),
                    _ => {}
                }
            }

            // Add target entity
            entities.push(CompiledEntity {
                id: touch.target.clone(),
                entity_type: touch.target.clone(),
                position: None,
                movement: None,
                ai_model: None,
            });

            rules.push(CompiledRule {
                when: format!("player touches {}", touch.target),
                then: actions,
            });
        }

        Ok(CompiledGame {
            name: game.game.unwrap_or_else(|| "my-game".to_string()),
            level: SchemaLevel::Level1,
            entities,
            rules,
            background: game.background,
            music: game.music,
        })
    }

    #[allow(clippy::unused_self)]
    fn compile_level2(&self, yaml: &str) -> Result<CompiledGame, YamlError> {
        let game: Level2Game = parse_yaml(yaml)?;

        // Validate
        validate_level2(&game)?;

        let mut entities = Vec::new();
        let mut rules = Vec::new();

        // Compile characters
        if let Some(characters) = &game.characters {
            for (name, char_def) in characters {
                entities.push(CompiledEntity {
                    id: name.clone(),
                    entity_type: char_def.char_type.clone(),
                    position: None,
                    movement: char_def.move_type.clone(),
                    ai_model: char_def.pattern.as_ref().map(|p| format!("builtin:{p}")),
                });
            }
        }

        // Fallback to single character (Level 1 compatibility)
        if entities.is_empty() {
            if let Some(character) = &game.character {
                entities.push(CompiledEntity {
                    id: "player".to_string(),
                    entity_type: character.clone(),
                    position: None,
                    movement: game.move_type.clone(),
                    ai_model: None,
                });
            }
        }

        // Compile rules
        if let Some(yaml_rules) = &game.rules {
            for rule in yaml_rules {
                let actions = compile_level2_actions(&rule.then);
                rules.push(CompiledRule {
                    when: rule.when.clone(),
                    then: actions,
                });
            }
        }

        // Level 1 compatibility: when_touch
        if let Some(touch) = &game.when_touch {
            let mut actions = Vec::new();
            if let Some(sound) = &touch.sound {
                actions.push(CompiledAction::PlaySound(sound.clone()));
            }
            if let Some(score) = touch.score {
                actions.push(CompiledAction::AddScore(i32::from(score)));
            }

            entities.push(CompiledEntity {
                id: touch.target.clone(),
                entity_type: touch.target.clone(),
                position: None,
                movement: None,
                ai_model: None,
            });

            rules.push(CompiledRule {
                when: format!("player touches {}", touch.target),
                then: actions,
            });
        }

        Ok(CompiledGame {
            name: game.game.unwrap_or_else(|| "my-game".to_string()),
            level: SchemaLevel::Level2,
            entities,
            rules,
            background: game.background,
            music: game.music,
        })
    }

    #[allow(clippy::unused_self)]
    fn compile_level3(&self, yaml: &str) -> Result<CompiledGame, YamlError> {
        let game: Level3Game = parse_yaml(yaml)?;

        let mut entities = Vec::new();
        let mut rules = Vec::new();

        // Compile entities
        if let Some(entity_defs) = &game.entities {
            for (name, entity_def) in entity_defs {
                entities.push(CompiledEntity {
                    id: name.clone(),
                    entity_type: entity_def.sprite.clone().unwrap_or_default(),
                    position: entity_def
                        .components
                        .as_ref()
                        .and_then(|c| c.position)
                        .map(Into::into),
                    movement: entity_def
                        .controls
                        .as_ref()
                        .and_then(|c| c.move_keys.clone()),
                    ai_model: entity_def.ai.clone(),
                });
            }
        }

        // Level 2 compatibility: characters
        if let Some(characters) = &game.characters {
            for (name, char_def) in characters {
                entities.push(CompiledEntity {
                    id: name.clone(),
                    entity_type: char_def.char_type.clone(),
                    position: None,
                    movement: char_def.move_type.clone(),
                    ai_model: char_def.pattern.as_ref().map(|p| format!("builtin:{p}")),
                });
            }
        }

        // Compile rules
        if let Some(yaml_rules) = &game.rules {
            for rule in yaml_rules {
                let actions = compile_level2_actions(&rule.then);
                rules.push(CompiledRule {
                    when: rule.when.clone(),
                    then: actions,
                });
            }
        }

        Ok(CompiledGame {
            name: game.game.unwrap_or_else(|| "my-game".to_string()),
            level: SchemaLevel::Level3,
            entities,
            rules,
            background: game.background,
            music: game.music,
        })
    }
}

/// Parse YAML with friendly error handling
fn parse_yaml<T: serde::de::DeserializeOwned>(yaml: &str) -> Result<T, YamlError> {
    serde_yaml::from_str(yaml).map_err(|e| {
        // Try to extract a meaningful field name from the error
        let message = e.to_string();

        // Check for unknown field errors
        if message.contains("unknown field") {
            if let Some(field) = extract_unknown_field(&message) {
                let vocab = Vocabulary::level1();
                let suggestions = vocab.suggest_similar(&field, 5);
                return YamlError::UnknownWord {
                    word: field,
                    suggestions,
                    line: e.location().map(|l| l.line()),
                };
            }
        }

        YamlError::SyntaxError {
            message,
            line: e.location().map(|l| l.line()),
            column: e.location().map(|l| l.column()),
        }
    })
}

/// Extract unknown field name from serde error message
fn extract_unknown_field(message: &str) -> Option<String> {
    // Pattern: "unknown field `fieldname`"
    if let Some(start) = message.find("unknown field `") {
        let after = &message[start + 15..];
        if let Some(end) = after.find('`') {
            return Some(after[..end].to_string());
        }
    }
    None
}

/// Normalize YAML for case-insensitive parsing
fn normalize_yaml(yaml: &str) -> Result<String, YamlError> {
    // Parse as generic value
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| YamlError::SyntaxError {
            message: e.to_string(),
            line: e.location().map(|l| l.line()),
            column: e.location().map(|l| l.column()),
        })?;

    // Normalize keys to lowercase
    let normalized = normalize_value(value);

    // Convert back to YAML string
    serde_yaml::to_string(&normalized).map_err(|e| YamlError::SyntaxError {
        message: e.to_string(),
        line: None,
        column: None,
    })
}

/// Recursively normalize a YAML value
fn normalize_value(value: serde_yaml::Value) -> serde_yaml::Value {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let mut new_map = serde_yaml::Mapping::new();
            for (k, v) in map {
                let normalized_key = if let serde_yaml::Value::String(s) = k {
                    serde_yaml::Value::String(normalize_key(&s))
                } else {
                    k
                };
                let _ = new_map.insert(normalized_key, normalize_value(v));
            }
            serde_yaml::Value::Mapping(new_map)
        }
        serde_yaml::Value::Sequence(seq) => {
            serde_yaml::Value::Sequence(seq.into_iter().map(normalize_value).collect())
        }
        // Keep string values as-is to preserve identifiers
        // Only keys are normalized to lowercase
        other => other,
    }
}

/// Normalize a key name (case-insensitive, British spelling)
fn normalize_key(key: &str) -> String {
    let lower = key.to_lowercase();

    // Handle British spellings
    match lower.as_str() {
        "colour" => "color".to_string(),
        "behaviour" => "behavior".to_string(),
        _ => lower,
    }
}

/// Check nesting depth of YAML
fn check_nesting_depth(yaml: &str, max_depth: u8) -> Result<(), YamlError> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| YamlError::SyntaxError {
            message: e.to_string(),
            line: e.location().map(|l| l.line()),
            column: e.location().map(|l| l.column()),
        })?;

    let depth = calculate_depth(&value);
    if depth > max_depth {
        return Err(YamlError::NestingTooDeep {
            max: max_depth,
            found: depth,
        });
    }
    Ok(())
}

/// Calculate the nesting depth of a YAML value
fn calculate_depth(value: &serde_yaml::Value) -> u8 {
    match value {
        serde_yaml::Value::Mapping(map) => 1 + map.values().map(calculate_depth).max().unwrap_or(0),
        serde_yaml::Value::Sequence(seq) => 1 + seq.iter().map(calculate_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Compile Level 2 actions
fn compile_level2_actions(actions: &[schema::Level2Action]) -> Vec<CompiledAction> {
    actions
        .iter()
        .filter_map(|action| match action {
            schema::Level2Action::AddScore { add_score } => {
                Some(CompiledAction::AddScore(*add_score))
            }
            schema::Level2Action::LoseLife { lose_life } => {
                Some(CompiledAction::LoseLife(*lose_life))
            }
            schema::Level2Action::Play { play } => Some(CompiledAction::PlaySound(play.clone())),
            schema::Level2Action::Show { show } => Some(CompiledAction::Show(show.clone())),
            schema::Level2Action::EntityAction { entity, action } => match action.as_str() {
                "respawn" | "new_place" => Some(CompiledAction::Respawn(entity.clone())),
                "disappear" => Some(CompiledAction::Disappear(entity.clone())),
                _ => None,
            },
            schema::Level2Action::Simple(s) => {
                if s == "stop" || s == "stop_game" {
                    Some(CompiledAction::StopGame)
                } else {
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_minimal_game() {
        let compiler = YamlCompiler::new();
        let result = compiler.compile("character: bunny");
        assert!(
            result.is_ok(),
            "Minimal game should compile: {:?}",
            result.err()
        );
        let game = result.unwrap();
        assert_eq!(game.level, SchemaLevel::Level1);
        assert_eq!(game.entities.len(), 1);
    }

    #[test]
    fn test_compile_with_touch() {
        let compiler = YamlCompiler::new();
        let yaml = r"
character: bunny
when_touch:
  target: star
  sound: twinkle
  score: 1
";
        let result = compiler.compile(yaml);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert!(!game.rules.is_empty());
    }

    #[test]
    fn test_case_insensitive() {
        let compiler = YamlCompiler::new();
        let yaml = "Character: BUNNY";
        let result = compiler.compile(yaml);
        assert!(
            result.is_ok(),
            "Should be case-insensitive: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_british_spelling() {
        let compiler = YamlCompiler::new();
        let yaml = "character: bunny\ncolour: red";
        // Should parse without error (colour -> color normalization)
        let result = compiler.compile(yaml);
        // This may fail validation since color isn't a core Level1 field,
        // but shouldn't crash on parsing
        assert!(result.is_ok() || matches!(result.err(), Some(YamlError::UnknownWord { .. })));
    }

    #[test]
    fn test_nesting_depth_level1() {
        let compiler = YamlCompiler::new();
        let yaml = r"
character: bunny
nested:
  level1:
    level2:
      level3:
        too_deep: true
";
        let result = compiler.compile(yaml);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            YamlError::NestingTooDeep { .. }
        ));
    }

    #[test]
    fn test_calculate_depth() {
        let yaml = "a: 1";
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(calculate_depth(&value), 1);

        let yaml = "a:\n  b: 1";
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(calculate_depth(&value), 2);

        let yaml = "a:\n  b:\n    c: 1";
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(calculate_depth(&value), 3);
    }

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("Character"), "character");
        assert_eq!(normalize_key("BUNNY"), "bunny");
        assert_eq!(normalize_key("colour"), "color");
        assert_eq!(normalize_key("behaviour"), "behavior");
    }

    #[test]
    fn test_extract_unknown_field() {
        let msg = "unknown field `dinosaur`, expected one of";
        assert_eq!(extract_unknown_field(msg), Some("dinosaur".to_string()));
    }

    #[test]
    fn test_compile_level2() {
        let compiler = YamlCompiler::new();
        let yaml = r"
game: test
characters:
  player:
    type: bunny
    speed: fast
  enemy:
    type: robot
    pattern: zigzag
lives: 3
";
        let result = compiler.compile(yaml);
        assert!(result.is_ok(), "{:?}", result.err());
        let game = result.unwrap();
        assert_eq!(game.level, SchemaLevel::Level2);
        assert_eq!(game.entities.len(), 2);
    }

    #[test]
    fn test_compile_level3() {
        let compiler = YamlCompiler::new();
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
        let result = compiler.compile(yaml);
        assert!(result.is_ok(), "{:?}", result.err());
        let game = result.unwrap();
        assert_eq!(game.level, SchemaLevel::Level3);
    }
}
