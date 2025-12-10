//! Content sandboxing for safe game creation.
//!
//! Per spec Section 9.1: All uploaded content is sandboxed
//! to prevent malicious or inappropriate content.

use crate::error::YamlError;
use crate::schema::SchemaLevel;

/// Maximum YAML file size (64 KB per spec)
pub const MAX_YAML_SIZE: usize = 64 * 1024;

/// Maximum entities allowed in a game
pub const MAX_ENTITIES: usize = 1000;

/// Content sandbox configuration
#[derive(Debug, Clone)]
pub struct ContentSandbox {
    /// Maximum YAML size in bytes
    pub max_yaml_size: usize,
    /// Maximum number of entities
    pub max_entities: usize,
    /// Maximum nesting depth (depends on schema level)
    pub max_nesting_depth: u8,
    /// Content filter for inappropriate content
    pub content_filter: ContentFilter,
}

impl Default for ContentSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentSandbox {
    /// Create a new sandbox with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_yaml_size: MAX_YAML_SIZE,
            max_entities: MAX_ENTITIES,
            max_nesting_depth: 3, // Default to Level 1
            content_filter: ContentFilter::default(),
        }
    }

    /// Create sandbox configured for a specific schema level
    #[must_use]
    pub fn for_level(level: SchemaLevel) -> Self {
        Self {
            max_yaml_size: MAX_YAML_SIZE,
            max_entities: MAX_ENTITIES,
            max_nesting_depth: level.max_nesting_depth(),
            content_filter: ContentFilter::default(),
        }
    }

    /// Validate YAML content against sandbox rules
    ///
    /// # Errors
    ///
    /// Returns `SandboxError` if content violates sandbox rules
    pub fn validate(&self, yaml: &str) -> Result<(), SandboxError> {
        // Size check
        if yaml.len() > self.max_yaml_size {
            return Err(SandboxError::YamlTooLarge {
                size: yaml.len(),
                max: self.max_yaml_size,
            });
        }

        // Parse YAML to check structure
        let doc: serde_yaml::Value =
            serde_yaml::from_str(yaml).map_err(|e| SandboxError::ParseError(e.to_string()))?;

        // Depth check
        let depth = calculate_depth(&doc);
        if depth > self.max_nesting_depth {
            return Err(SandboxError::TooDeep {
                found: depth,
                max: self.max_nesting_depth,
            });
        }

        // Entity count check
        let entity_count = count_entities(&doc);
        if entity_count > self.max_entities {
            return Err(SandboxError::TooManyEntities {
                count: entity_count,
                max: self.max_entities,
            });
        }

        // Content filter
        let all_text = extract_all_strings(&doc);
        if let Some(violation) = self.content_filter.check(&all_text) {
            return Err(SandboxError::ContentViolation(violation));
        }

        Ok(())
    }

    /// Validate and convert to `YamlError` if failed
    ///
    /// # Errors
    ///
    /// Returns `YamlError` if content violates sandbox rules
    pub fn validate_yaml(&self, yaml: &str) -> Result<(), YamlError> {
        self.validate(yaml).map_err(SandboxError::into_yaml_error)
    }
}

/// Errors that can occur during sandbox validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    /// YAML file too large
    YamlTooLarge {
        /// Actual size in bytes
        size: usize,
        /// Maximum allowed size
        max: usize,
    },
    /// Nesting too deep
    TooDeep {
        /// Found depth
        found: u8,
        /// Maximum allowed depth
        max: u8,
    },
    /// Too many entities
    TooManyEntities {
        /// Found entity count
        count: usize,
        /// Maximum allowed count
        max: usize,
    },
    /// Inappropriate content detected
    ContentViolation(ContentViolation),
    /// YAML parse error
    ParseError(String),
}

impl SandboxError {
    /// Convert to a kid-friendly `YamlError`
    #[must_use]
    pub fn into_yaml_error(self) -> YamlError {
        match self {
            Self::YamlTooLarge { size, max } => YamlError::OutOfRange {
                field: "file size".to_string(),
                min: 0,
                max: i64::try_from(max).unwrap_or(i64::MAX),
                value: i64::try_from(size).unwrap_or(i64::MAX),
            },
            Self::TooDeep { found, max } => YamlError::NestingTooDeep { max, found },
            Self::TooManyEntities { count, max } => YamlError::OutOfRange {
                field: "number of characters".to_string(),
                min: 0,
                max: i64::try_from(max).unwrap_or(i64::MAX),
                value: i64::try_from(count).unwrap_or(i64::MAX),
            },
            Self::ContentViolation(violation) => YamlError::UnknownWord {
                word: violation.word,
                suggestions: vec!["Please use friendly words only!".to_string()],
                line: None,
            },
            Self::ParseError(msg) => YamlError::SyntaxError {
                message: msg,
                line: None,
                column: None,
            },
        }
    }
}

/// Content filter for inappropriate words
#[derive(Debug, Clone)]
pub struct ContentFilter {
    /// Blocked words (case-insensitive matching)
    blocked_words: Vec<String>,
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentFilter {
    /// Create a new content filter with default blocklist
    #[must_use]
    pub fn new() -> Self {
        Self {
            blocked_words: default_blocklist(),
        }
    }

    /// Check text for content violations
    #[must_use]
    pub fn check(&self, text: &str) -> Option<ContentViolation> {
        let text_lower = text.to_lowercase();

        for blocked in &self.blocked_words {
            if text_lower.contains(blocked) {
                return Some(ContentViolation {
                    word: blocked.clone(),
                    reason: "This word isn't allowed in games for kids".to_string(),
                });
            }
        }

        None
    }

    /// Add a word to the blocklist
    pub fn block_word(&mut self, word: impl Into<String>) {
        self.blocked_words.push(word.into().to_lowercase());
    }
}

/// A content violation detected by the filter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentViolation {
    /// The word that triggered the violation
    pub word: String,
    /// Reason for blocking
    pub reason: String,
}

/// Calculate the nesting depth of a YAML value
#[must_use]
pub fn calculate_depth(value: &serde_yaml::Value) -> u8 {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let max_child_depth = map.values().map(calculate_depth).max().unwrap_or(0);
            max_child_depth.saturating_add(1)
        }
        serde_yaml::Value::Sequence(seq) => {
            let max_child_depth = seq.iter().map(calculate_depth).max().unwrap_or(0);
            max_child_depth.saturating_add(1)
        }
        _ => 1, // Scalars count as depth 1
    }
}

/// Count the number of entities in a YAML document
fn count_entities(value: &serde_yaml::Value) -> usize {
    let mut count = 0;

    // Look for known entity keys
    if let serde_yaml::Value::Mapping(map) = value {
        let single_char_key = serde_yaml::Value::String("character".to_string());
        let multi_char_key = serde_yaml::Value::String("characters".to_string());
        let entities_key = serde_yaml::Value::String("entities".to_string());

        // Single character
        if map.contains_key(&single_char_key) {
            count += 1;
        }
        // Multiple characters
        if let Some(serde_yaml::Value::Mapping(char_defs)) = map.get(&multi_char_key) {
            count += char_defs.len();
        }
        // Entities (Level 3)
        if let Some(serde_yaml::Value::Mapping(entity_defs)) = map.get(&entities_key) {
            count += entity_defs.len();
        }
        // Recurse into nested structures
        for v in map.values() {
            count += count_nested_entities(v);
        }
    }

    count
}

/// Count entities in nested structures (items, enemies, etc.)
fn count_nested_entities(value: &serde_yaml::Value) -> usize {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let mut count = 0;
            // Check for items array
            if let Some(serde_yaml::Value::Sequence(items)) = map.get("items") {
                count += items.len();
            }
            // Check for enemies array
            if let Some(serde_yaml::Value::Sequence(enemies)) = map.get("enemies") {
                count += enemies.len();
            }
            count
        }
        serde_yaml::Value::Sequence(seq) => seq.len(),
        _ => 0,
    }
}

/// Extract all string values from a YAML document
fn extract_all_strings(value: &serde_yaml::Value) -> String {
    let mut result = String::new();
    collect_strings(value, &mut result);
    result
}

/// Recursively collect all strings
fn collect_strings(value: &serde_yaml::Value, output: &mut String) {
    match value {
        serde_yaml::Value::String(s) => {
            output.push_str(s);
            output.push(' ');
        }
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    output.push_str(key);
                    output.push(' ');
                }
                collect_strings(v, output);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                collect_strings(item, output);
            }
        }
        _ => {}
    }
}

/// Default blocklist for content filtering
/// This is a minimal list focused on child safety
fn default_blocklist() -> Vec<String> {
    // Minimal blocklist - in production this would be more comprehensive
    // but loaded from an external source for easy updates
    vec![
        "kill".to_string(),
        "death".to_string(),
        "blood".to_string(),
        "gore".to_string(),
        "violent".to_string(),
        "weapon".to_string(),
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 9.1
    // ========================================================================

    mod size_limit_tests {
        use super::*;

        #[test]
        fn test_max_yaml_size_is_64kb() {
            // Per spec Section 9.1: max_yaml_size: 64 KB
            assert_eq!(MAX_YAML_SIZE, 64 * 1024);
        }

        #[test]
        fn test_rejects_oversized_yaml() {
            let sandbox = ContentSandbox::new();
            let huge_yaml = "x".repeat(MAX_YAML_SIZE + 1);
            let result = sandbox.validate(&huge_yaml);
            assert!(matches!(result, Err(SandboxError::YamlTooLarge { .. })));
        }

        #[test]
        fn test_accepts_max_size_yaml() {
            let sandbox = ContentSandbox::new();
            // Create a valid YAML that's just under the limit
            let yaml = format!("character: bunny\n# {}", "x".repeat(MAX_YAML_SIZE - 50));
            // This should either pass or fail for other reasons, not size
            let result = sandbox.validate(&yaml);
            assert!(!matches!(result, Err(SandboxError::YamlTooLarge { .. })));
        }
    }

    mod depth_limit_tests {
        use super::*;

        #[test]
        fn test_level1_max_depth_is_3() {
            let sandbox = ContentSandbox::for_level(SchemaLevel::Level1);
            assert_eq!(sandbox.max_nesting_depth, 3);
        }

        #[test]
        fn test_level2_max_depth_is_5() {
            let sandbox = ContentSandbox::for_level(SchemaLevel::Level2);
            assert_eq!(sandbox.max_nesting_depth, 5);
        }

        #[test]
        fn test_level3_max_depth_is_6() {
            let sandbox = ContentSandbox::for_level(SchemaLevel::Level3);
            assert_eq!(sandbox.max_nesting_depth, 6);
        }

        #[test]
        fn test_rejects_too_deep_nesting() {
            let sandbox = ContentSandbox::for_level(SchemaLevel::Level1);
            let yaml = r"
level1:
  level2:
    level3:
      level4: too_deep
";
            let result = sandbox.validate(yaml);
            assert!(matches!(result, Err(SandboxError::TooDeep { .. })));
        }

        #[test]
        fn test_accepts_valid_depth() {
            let sandbox = ContentSandbox::for_level(SchemaLevel::Level1);
            let yaml = r"
character: bunny
when_touch:
  target: star
";
            let result = sandbox.validate(yaml);
            // Should not fail due to depth
            assert!(!matches!(result, Err(SandboxError::TooDeep { .. })));
        }
    }

    mod entity_limit_tests {
        use super::*;

        #[test]
        fn test_max_entities_is_1000() {
            // Per spec Section 9.1: max_entities: 1000
            assert_eq!(MAX_ENTITIES, 1000);
        }

        #[test]
        fn test_counts_single_character() {
            let yaml = "character: bunny";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(count_entities(&doc), 1);
        }

        #[test]
        fn test_counts_multiple_characters() {
            let yaml = r"
characters:
  player:
    type: bunny
  enemy1:
    type: fox
  enemy2:
    type: dragon
";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(count_entities(&doc), 3);
        }

        #[test]
        fn test_rejects_too_many_entities() {
            let mut sandbox = ContentSandbox::new();
            sandbox.max_entities = 2; // Lower for testing
            sandbox.max_nesting_depth = 10; // Allow deep nesting for this test

            let yaml = r"
characters:
  player:
    type: bunny
  enemy1:
    type: fox
  enemy2:
    type: dragon
";
            let result = sandbox.validate(yaml);
            assert!(
                matches!(result, Err(SandboxError::TooManyEntities { .. })),
                "Expected TooManyEntities, got {result:?}"
            );
        }
    }

    mod content_filter_tests {
        use super::*;

        #[test]
        fn test_filter_blocks_violent_words() {
            let filter = ContentFilter::new();
            let result = filter.check("I want to kill the monster");
            assert!(result.is_some());
            assert_eq!(result.unwrap().word, "kill");
        }

        #[test]
        fn test_filter_case_insensitive() {
            let filter = ContentFilter::new();
            let result = filter.check("BLOOD and gore");
            assert!(result.is_some());
        }

        #[test]
        fn test_filter_allows_friendly_content() {
            let filter = ContentFilter::new();
            let result = filter.check("bunny catches stars in the sky");
            assert!(result.is_none());
        }

        #[test]
        fn test_custom_blocked_word() {
            let mut filter = ContentFilter::new();
            filter.block_word("badword");
            let result = filter.check("this has a badword");
            assert!(result.is_some());
        }

        #[test]
        fn test_sandbox_validates_content() {
            let sandbox = ContentSandbox::new();
            let yaml = r"
game: violence-game
action: kill
";
            let result = sandbox.validate(yaml);
            assert!(matches!(result, Err(SandboxError::ContentViolation(_))));
        }
    }

    mod depth_calculation_tests {
        use super::*;

        #[test]
        fn test_scalar_depth_is_1() {
            let yaml = "value";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(calculate_depth(&doc), 1);
        }

        #[test]
        fn test_flat_map_depth_is_2() {
            let yaml = "key: value";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(calculate_depth(&doc), 2);
        }

        #[test]
        fn test_nested_depth_calculation() {
            let yaml = r"
level1:
  level2:
    level3: value
";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(calculate_depth(&doc), 4); // root + 3 levels
        }

        #[test]
        fn test_array_depth_calculation() {
            let yaml = r"
items:
  - one
  - two
";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(calculate_depth(&doc), 3); // root -> items -> array -> scalar
        }
    }

    mod string_extraction_tests {
        use super::*;

        #[test]
        fn test_extracts_all_strings() {
            let yaml = r"
game: my-game
character: bunny
";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            let text = extract_all_strings(&doc);
            assert!(text.contains("game"));
            assert!(text.contains("my-game"));
            assert!(text.contains("character"));
            assert!(text.contains("bunny"));
        }

        #[test]
        fn test_extracts_from_nested() {
            let yaml = r"
outer:
  inner: secret_word
";
            let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
            let text = extract_all_strings(&doc);
            assert!(text.contains("secret_word"));
        }
    }

    mod error_conversion_tests {
        use super::*;

        #[test]
        fn test_yaml_too_large_converts() {
            let err = SandboxError::YamlTooLarge {
                size: 100_000,
                max: 65_536,
            };
            let yaml_err = err.into_yaml_error();
            assert!(matches!(yaml_err, YamlError::OutOfRange { .. }));
        }

        #[test]
        fn test_too_deep_converts() {
            let err = SandboxError::TooDeep { found: 5, max: 3 };
            let yaml_err = err.into_yaml_error();
            assert!(matches!(yaml_err, YamlError::NestingTooDeep { .. }));
        }

        #[test]
        fn test_content_violation_converts() {
            let err = SandboxError::ContentViolation(ContentViolation {
                word: "badword".to_string(),
                reason: "blocked".to_string(),
            });
            let yaml_err = err.into_yaml_error();
            assert!(matches!(yaml_err, YamlError::UnknownWord { .. }));
        }
    }
}
