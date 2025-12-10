//! Intelligent scaffolding engine for error recovery and learning.
//!
//! Per spec Section 14 (v1.3.0): Implements "Scaffolding Engine" that uses AST analysis
//! to rewrite user intent, not just report syntax errors.
//!
//! Based on Vygotsky's Zone of Proximal Development (ZPD):
//! - Identify what the child was trying to do
//! - Provide the minimum help needed to succeed
//! - Generate working examples as scaffolds

use crate::error::{KidFriendlyError, YamlError};
use crate::schema::SchemaLevel;
use crate::vocabulary::Vocabulary;

/// Scaffolding engine that provides intelligent error recovery
#[derive(Debug, Clone)]
pub struct ScaffoldingEngine {
    /// Current schema level for context-aware suggestions
    level: SchemaLevel,
    /// Vocabulary for the current level
    vocabulary: Vocabulary,
}

/// A scaffold suggestion that helps the child fix their code
#[derive(Debug, Clone, PartialEq)]
pub struct Scaffold {
    /// What we think the child was trying to do
    pub detected_intent: Intent,
    /// Working code example that achieves the intent
    pub working_example: String,
    /// Explanation of why the original didn't work
    pub learning_hint: String,
    /// Specific line-by-line corrections
    pub corrections: Vec<Correction>,
    /// Confidence score (0.0-1.0) for the detected intent
    pub confidence: f32,
}

/// Detected user intent based on semantic analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Trying to create a character with the given name
    CreateCharacter {
        /// The attempted character name
        name: String,
    },
    /// Trying to set movement behavior
    SetMovement {
        /// The attempted movement style
        style: String,
    },
    /// Trying to define a touch/collision event
    DefineEvent {
        /// The type of event being defined
        event_type: String,
    },
    /// Trying to add sound or music
    AddAudio {
        /// The type of audio being added
        audio_type: String,
    },
    /// Trying to set visual appearance
    SetVisuals {
        /// The visual element being set
        element: String,
    },
    /// Trying to define game rules
    DefineRules,
    /// Could not determine intent
    Unknown,
}

/// A specific correction with before/after
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    /// Line number (1-indexed)
    pub line: usize,
    /// Original text
    pub original: String,
    /// Suggested replacement
    pub replacement: String,
    /// Why this change is needed
    pub reason: String,
}

impl ScaffoldingEngine {
    /// Create a new scaffolding engine for the given schema level
    #[must_use]
    pub fn new(level: SchemaLevel) -> Self {
        let vocabulary = match level {
            SchemaLevel::Level1 => Vocabulary::level1(),
            SchemaLevel::Level2 => Vocabulary::level2(),
            SchemaLevel::Level3 => Vocabulary::level3(),
        };
        Self { level, vocabulary }
    }

    /// Analyze an error and generate intelligent scaffolding
    #[must_use]
    pub fn scaffold(&self, yaml: &str, error: &YamlError) -> ScaffoldedError {
        let scaffold = self.analyze_and_scaffold(yaml, error);
        let base_error = error.to_kid_friendly();

        ScaffoldedError {
            base: base_error,
            scaffold: Some(scaffold),
        }
    }

    /// Analyze YAML and error to create a scaffold
    fn analyze_and_scaffold(&self, yaml: &str, error: &YamlError) -> Scaffold {
        let intent = Self::detect_intent(yaml, error);
        let working_example = Self::generate_working_example(&intent);
        let learning_hint = self.generate_learning_hint(error, &intent);
        let corrections = self.generate_corrections(yaml, error, &intent);
        let confidence = Self::calculate_confidence(&intent, &corrections);

        Scaffold {
            detected_intent: intent,
            working_example,
            learning_hint,
            corrections,
            confidence,
        }
    }

    /// Detect what the user was trying to accomplish
    fn detect_intent(yaml: &str, error: &YamlError) -> Intent {
        let lines: Vec<&str> = yaml.lines().collect();

        // Check error type for context
        match error {
            YamlError::UnknownWord { word, .. } => Self::intent_from_unknown_word(word, &lines),
            YamlError::MissingRequired { field, .. } => Self::intent_from_missing_field(field),
            YamlError::InvalidEnumValue { field, value, .. } => {
                Self::intent_from_invalid_value(field, value)
            }
            YamlError::SyntaxError { line, .. } => {
                Self::intent_from_syntax_context(line.unwrap_or(1), &lines)
            }
            _ => {
                // Try to infer from overall structure
                Self::intent_from_structure(&lines)
            }
        }
    }

    /// Detect intent from an unknown word error
    fn intent_from_unknown_word(word: &str, lines: &[&str]) -> Intent {
        let word_lower = word.to_lowercase();

        // Character-related words
        if Self::is_character_like(&word_lower) {
            return Intent::CreateCharacter {
                name: word.to_string(),
            };
        }

        // Movement-related words
        if Self::is_movement_like(&word_lower) {
            return Intent::SetMovement {
                style: word.to_string(),
            };
        }

        // Sound/music-related words
        if Self::is_audio_like(&word_lower) {
            return Intent::AddAudio {
                audio_type: word.to_string(),
            };
        }

        // Event-related words
        if Self::is_event_like(&word_lower) {
            return Intent::DefineEvent {
                event_type: word.to_string(),
            };
        }

        // Check context from surrounding lines
        Self::intent_from_structure(lines)
    }

    /// Detect intent from a missing required field
    fn intent_from_missing_field(field: &str) -> Intent {
        match field {
            "character" => Intent::CreateCharacter {
                name: String::new(),
            },
            "move" | "movement" => Intent::SetMovement {
                style: String::new(),
            },
            "sound" | "music" => Intent::AddAudio {
                audio_type: String::new(),
            },
            "when_touch" | "when" | "event" => Intent::DefineEvent {
                event_type: String::new(),
            },
            "rules" => Intent::DefineRules,
            _ => Intent::Unknown,
        }
    }

    /// Detect intent from an invalid enum value
    fn intent_from_invalid_value(field: &str, value: &str) -> Intent {
        match field {
            "character" | "type" => Intent::CreateCharacter {
                name: value.to_string(),
            },
            "move" | "pattern" => Intent::SetMovement {
                style: value.to_string(),
            },
            "sound" | "music" => Intent::AddAudio {
                audio_type: value.to_string(),
            },
            "background" | "color" | "colour" => Intent::SetVisuals {
                element: value.to_string(),
            },
            _ => Intent::Unknown,
        }
    }

    /// Detect intent from syntax error context
    fn intent_from_syntax_context(line_num: usize, lines: &[&str]) -> Intent {
        if line_num == 0 || line_num > lines.len() {
            return Intent::Unknown;
        }

        let line = lines.get(line_num - 1).unwrap_or(&"");
        let line_lower = line.to_lowercase();

        // Look for keywords in the problematic line
        if line_lower.contains("character")
            || line_lower.contains("bunny")
            || line_lower.contains("cat")
            || line_lower.contains("dog")
        {
            return Intent::CreateCharacter {
                name: String::new(),
            };
        }

        if line_lower.contains("when") || line_lower.contains("touch") {
            return Intent::DefineEvent {
                event_type: "touch".to_string(),
            };
        }

        if line_lower.contains("move") || line_lower.contains("arrow") {
            return Intent::SetMovement {
                style: String::new(),
            };
        }

        Intent::Unknown
    }

    /// Detect intent from overall YAML structure
    fn intent_from_structure(lines: &[&str]) -> Intent {
        for line in lines {
            let trimmed = line.trim().to_lowercase();

            if trimmed.starts_with("character") {
                return Intent::CreateCharacter {
                    name: String::new(),
                };
            }
            if trimmed.starts_with("when") {
                return Intent::DefineEvent {
                    event_type: "touch".to_string(),
                };
            }
            if trimmed.starts_with("move") {
                return Intent::SetMovement {
                    style: String::new(),
                };
            }
            if trimmed.starts_with("rules") {
                return Intent::DefineRules;
            }
        }

        Intent::Unknown
    }

    /// Check if a word looks like it's trying to be a character
    fn is_character_like(word: &str) -> bool {
        // Common animal/creature suffixes and patterns
        let character_patterns = [
            "bunny", "cat", "dog", "bird", "fish", "bear", "fox", "dragon", "unicorn", "robot",
            "monster", "creature", "animal", "pet", "hero", "player",
            // Common kid variations
            "puppy", "kitty", "doggy", "birdie", "fishy",
        ];

        for pattern in &character_patterns {
            // Exact match or contains
            if word == *pattern || word.contains(pattern) {
                return true;
            }
            // Fuzzy match - use stricter threshold for short words
            let threshold = if word.len() <= 4 { 1 } else { 2 };
            if Self::levenshtein(word, pattern) <= threshold {
                return true;
            }
        }

        false
    }

    /// Check if a word looks like it's trying to be movement-related
    fn is_movement_like(word: &str) -> bool {
        let movement_patterns = [
            "move", "walk", "run", "jump", "fly", "swim", "arrow", "touch", "auto", "control",
            "steer", "zigzag", "patrol", "chase", "wander",
        ];

        for pattern in &movement_patterns {
            if word.contains(pattern) || Self::levenshtein(word, pattern) <= 2 {
                return true;
            }
        }

        false
    }

    /// Check if a word looks like it's trying to be audio-related
    fn is_audio_like(word: &str) -> bool {
        let audio_patterns = [
            "sound", "music", "audio", "noise", "beep", "boop", "pop", "ding", "whoosh", "splash",
            "boing", "buzz", "click", "twinkle", "tone", "chime", "ring",
        ];

        for pattern in &audio_patterns {
            if word.contains(pattern) || word == *pattern || Self::levenshtein(word, pattern) <= 2 {
                return true;
            }
        }

        false
    }

    /// Check if a word looks like it's trying to be event-related
    fn is_event_like(word: &str) -> bool {
        let event_patterns = [
            "when", "if", "touch", "hit", "collect", "reach", "score", "timer", "event", "trigger",
        ];

        for pattern in &event_patterns {
            if word.contains(pattern) || Self::levenshtein(word, pattern) <= 2 {
                return true;
            }
        }

        false
    }

    /// Generate a working code example for the detected intent
    fn generate_working_example(intent: &Intent) -> String {
        match intent {
            Intent::CreateCharacter { name } => {
                let suggested_char = Self::suggest_valid_character(name);
                format!(
                    "# Here's how to create your character:\n\
                     character: {suggested_char}\n\
                     move: arrows  # Use arrow keys to move"
                )
            }
            Intent::SetMovement { style } => {
                let suggested_move = Self::suggest_valid_movement(style);
                format!(
                    "# Here's how to set movement:\n\
                     move: {suggested_move}"
                )
            }
            Intent::DefineEvent { event_type: _ } => {
                "# Here's how to make things happen when you touch something:\n\
                 when_touch:\n  \
                   target: star\n  \
                   sound: ding\n  \
                   score: 1"
                    .to_string()
            }
            Intent::AddAudio { audio_type } => {
                let suggested = Self::suggest_valid_sound(audio_type);
                format!(
                    "# Here's how to add sounds:\n\
                     music: happy\n\n\
                     # Or in a touch event:\n\
                     when_touch:\n  \
                       target: star\n  \
                       sound: {suggested}"
                )
            }
            Intent::SetVisuals { element } => {
                let suggested = Self::suggest_valid_background(element);
                format!(
                    "# Here's how to set the background:\n\
                     background: {suggested}"
                )
            }
            Intent::DefineRules => "# Here's how to create rules (Level 2):\n\
                 rules:\n  \
                   - when: \"player touches star\"\n    \
                     then:\n      \
                       - add_score: 10"
                .to_string(),
            Intent::Unknown => "# Here's a complete example game:\n\
                 character: bunny\n\
                 move: arrows\n\
                 background: grass\n\n\
                 when_touch:\n  \
                   target: star\n  \
                   sound: ding\n  \
                   score: 1"
                .to_string(),
        }
    }

    /// Generate a learning hint explaining why the error occurred
    fn generate_learning_hint(&self, error: &YamlError, intent: &Intent) -> String {
        let intent_context = match intent {
            Intent::CreateCharacter { name } if !name.is_empty() => {
                format!("It looks like you want to create a character called '{name}'.")
            }
            Intent::SetMovement { style } if !style.is_empty() => {
                format!("It looks like you want to set movement to '{style}'.")
            }
            _ => String::new(),
        };

        // Level-aware vocabulary hint
        let level_hint = match self.level {
            SchemaLevel::Level1 => " At Level 1, we use simple words for beginners.",
            SchemaLevel::Level2 => " At Level 2, you have more words to choose from!",
            SchemaLevel::Level3 => " At Level 3, you can use advanced features.",
        };

        let error_explanation = match error {
            YamlError::UnknownWord { word, .. } => {
                format!(
                    "The word '{word}' isn't in my vocabulary yet.{level_hint} \
                     But don't worry - I can help you find the right word!"
                )
            }
            YamlError::SyntaxError { .. } => {
                "The way the code is written is a bit confusing for me. \
                 In YAML, spaces and colons are very important!"
                    .to_string()
            }
            YamlError::MissingRequired { field, .. } => {
                format!(
                    "Every game needs a '{field}'. It's like a recipe - \
                     some ingredients are required!"
                )
            }
            YamlError::InvalidEnumValue {
                value,
                valid_options,
                ..
            } => {
                let options_preview: Vec<_> = valid_options.iter().take(3).collect();
                format!(
                    "'{value}' isn't one of the choices I know. \
                     Some options are: {}",
                    options_preview
                        .iter()
                        .map(|s| format!("'{s}'"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            _ => "Something didn't quite work, but we can fix it together!".to_string(),
        };

        if intent_context.is_empty() {
            error_explanation
        } else {
            format!("{intent_context}\n\n{error_explanation}")
        }
    }

    /// Generate specific line-by-line corrections
    fn generate_corrections(
        &self,
        yaml: &str,
        error: &YamlError,
        intent: &Intent,
    ) -> Vec<Correction> {
        let mut corrections = Vec::new();
        let lines: Vec<&str> = yaml.lines().collect();

        match error {
            YamlError::UnknownWord { word, line, .. } => {
                let line_num = line.unwrap_or(1);
                if let Some(original_line) = lines.get(line_num.saturating_sub(1)) {
                    let replacement = self.suggest_replacement_for_word(word, intent);
                    let new_line = original_line.replace(word, &replacement);

                    corrections.push(Correction {
                        line: line_num,
                        original: (*original_line).to_string(),
                        replacement: new_line,
                        reason: format!("Replace '{word}' with '{replacement}'"),
                    });
                }
            }
            YamlError::SyntaxError { line, .. } => {
                let line_num = line.unwrap_or(1);
                if let Some(original_line) = lines.get(line_num.saturating_sub(1)) {
                    if let Some(fixed) = Self::fix_common_syntax_issues(original_line) {
                        corrections.push(Correction {
                            line: line_num,
                            original: (*original_line).to_string(),
                            replacement: fixed,
                            reason: "Fixed formatting".to_string(),
                        });
                    }
                }
            }
            YamlError::MissingRequired { field, example } => {
                // Suggest adding at the end
                let line_num = lines.len() + 1;
                corrections.push(Correction {
                    line: line_num,
                    original: String::new(),
                    replacement: format!("{field}: {example}"),
                    reason: format!("Add the required '{field}' field"),
                });
            }
            _ => {}
        }

        corrections
    }

    /// Calculate confidence score for the detected intent
    fn calculate_confidence(intent: &Intent, corrections: &[Correction]) -> f32 {
        let base_confidence = match intent {
            Intent::Unknown => 0.2,
            Intent::CreateCharacter { name } if name.is_empty() => 0.5,
            Intent::SetMovement { style } if style.is_empty() => 0.5,
            Intent::CreateCharacter { .. } | Intent::SetMovement { .. } => 0.8,
            Intent::DefineEvent { .. } | Intent::AddAudio { .. } | Intent::SetVisuals { .. } => 0.7,
            Intent::DefineRules => 0.6,
        };

        // Boost confidence if we have corrections
        let correction_boost: f32 = if corrections.is_empty() { 0.0 } else { 0.1 };

        f32::min(base_confidence + correction_boost, 1.0)
    }

    /// Suggest a valid character name
    fn suggest_valid_character(attempted: &str) -> &'static str {
        let attempted_lower = attempted.to_lowercase();

        // Map common attempts to valid characters
        if attempted_lower.contains("dog") || attempted_lower.contains("puppy") {
            return "dog";
        }
        if attempted_lower.contains("cat") || attempted_lower.contains("kitty") {
            return "cat";
        }
        if attempted_lower.contains("bird") {
            return "bird";
        }
        if attempted_lower.contains("fish") {
            return "fish";
        }

        // Default suggestion
        "bunny"
    }

    /// Suggest a valid movement type
    fn suggest_valid_movement(attempted: &str) -> &'static str {
        let attempted_lower = attempted.to_lowercase();

        if attempted_lower.contains("key") || attempted_lower.contains("arrow") {
            return "arrows";
        }
        if attempted_lower.contains("touch") || attempted_lower.contains("tap") {
            return "touch";
        }
        if attempted_lower.contains("auto") || attempted_lower.contains("self") {
            return "auto";
        }

        "arrows"
    }

    /// Suggest a valid sound
    fn suggest_valid_sound(attempted: &str) -> &'static str {
        let attempted_lower = attempted.to_lowercase();

        if attempted_lower.contains("pop") {
            return "pop";
        }
        if attempted_lower.contains("ding") || attempted_lower.contains("bell") {
            return "ding";
        }
        if attempted_lower.contains("splash") || attempted_lower.contains("water") {
            return "splash";
        }

        "ding"
    }

    /// Suggest a valid background
    fn suggest_valid_background(attempted: &str) -> &'static str {
        let attempted_lower = attempted.to_lowercase();

        if attempted_lower.contains("sky") || attempted_lower.contains("cloud") {
            return "sky";
        }
        if attempted_lower.contains("grass") || attempted_lower.contains("green") {
            return "grass";
        }
        if attempted_lower.contains("water") || attempted_lower.contains("ocean") {
            return "water";
        }
        if attempted_lower.contains("space") || attempted_lower.contains("star") {
            return "space";
        }

        "grass"
    }

    /// Suggest a replacement word based on intent
    fn suggest_replacement_for_word(&self, word: &str, intent: &Intent) -> String {
        match intent {
            Intent::CreateCharacter { .. } => Self::suggest_valid_character(word).to_string(),
            Intent::SetMovement { .. } => Self::suggest_valid_movement(word).to_string(),
            Intent::AddAudio { .. } => Self::suggest_valid_sound(word).to_string(),
            Intent::SetVisuals { .. } => Self::suggest_valid_background(word).to_string(),
            _ => {
                // Try to find closest match in vocabulary
                self.vocabulary
                    .suggest_similar(word, 3)
                    .first()
                    .cloned()
                    .unwrap_or_else(|| word.to_string())
            }
        }
    }

    /// Fix common syntax issues in a YAML line
    fn fix_common_syntax_issues(line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Missing space after colon
        if trimmed.contains(':') && !trimmed.contains(": ") && !trimmed.ends_with(':') {
            let fixed = trimmed.replace(':', ": ");
            return Some(fixed);
        }

        // Tab characters instead of spaces
        if line.contains('\t') {
            return Some(line.replace('\t', "  "));
        }

        // Missing colon
        if !trimmed.is_empty() && !trimmed.contains(':') && !trimmed.starts_with('#') {
            // Check if it looks like a key
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            if words.len() == 2 {
                return Some(format!("{}: {}", words[0], words[1]));
            }
        }

        None
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

        for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
            row[0] = i;
        }
        for (j, cell) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
            *cell = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }
}

/// An error enhanced with scaffolding information
#[derive(Debug, Clone)]
pub struct ScaffoldedError {
    /// The base kid-friendly error
    pub base: KidFriendlyError,
    /// Optional scaffold with working example and corrections
    pub scaffold: Option<Scaffold>,
}

impl ScaffoldedError {
    /// Render the scaffolded error as a formatted string
    #[must_use]
    pub fn render(&self) -> String {
        use core::fmt::Write;
        let mut output = self.base.render();

        if let Some(scaffold) = &self.scaffold {
            // Add the learning hint
            let _ = writeln!(output, "\n💡 Learning moment:");
            let _ = writeln!(output, "{}\n", scaffold.learning_hint);

            // Add the working example
            let _ = writeln!(output, "✨ Here's how to do it:");
            let _ = writeln!(output, "```yaml");
            let _ = writeln!(output, "{}", scaffold.working_example);
            let _ = writeln!(output, "```");

            // Add specific corrections if any
            if !scaffold.corrections.is_empty() {
                let _ = writeln!(output, "\n🔧 Specific fixes:");
                for correction in &scaffold.corrections {
                    let _ = writeln!(output, "  Line {}: {}", correction.line, correction.reason);
                    if !correction.original.is_empty() {
                        let _ = writeln!(output, "    Before: {}", correction.original.trim());
                    }
                    let _ = writeln!(output, "    After:  {}", correction.replacement.trim());
                }
            }
        }

        output
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 14
    // ========================================================================

    mod scaffolding_engine_tests {
        use super::*;

        #[test]
        fn test_engine_creation() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            assert_eq!(engine.level, SchemaLevel::Level1);
        }

        #[test]
        fn test_engine_for_each_level() {
            let _l1 = ScaffoldingEngine::new(SchemaLevel::Level1);
            let _l2 = ScaffoldingEngine::new(SchemaLevel::Level2);
            let _l3 = ScaffoldingEngine::new(SchemaLevel::Level3);
        }
    }

    mod intent_detection_tests {
        use super::*;

        #[test]
        fn test_detect_character_intent_from_unknown_word() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "character: puppy";
            let error = YamlError::UnknownWord {
                word: "puppy".to_string(),
                suggestions: vec![],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            assert!(matches!(
                scaffolded.scaffold.as_ref().unwrap().detected_intent,
                Intent::CreateCharacter { .. }
            ));
        }

        #[test]
        fn test_detect_movement_intent() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "move: keyboard";
            let error = YamlError::UnknownWord {
                word: "keyboard".to_string(),
                suggestions: vec![],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            assert!(matches!(
                scaffolded.scaffold.as_ref().unwrap().detected_intent,
                Intent::SetMovement { .. }
            ));
        }

        #[test]
        fn test_detect_audio_intent() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "sound: beep";
            let error = YamlError::UnknownWord {
                word: "beep".to_string(),
                suggestions: vec![],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            assert!(matches!(
                scaffolded.scaffold.as_ref().unwrap().detected_intent,
                Intent::AddAudio { .. }
            ));
        }

        #[test]
        fn test_detect_event_intent_from_missing_field() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "character: bunny";
            let error = YamlError::MissingRequired {
                field: "when_touch".to_string(),
                example: "target: star".to_string(),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            assert!(matches!(
                scaffolded.scaffold.as_ref().unwrap().detected_intent,
                Intent::DefineEvent { .. }
            ));
        }

        #[test]
        fn test_detect_rules_intent() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level2);
            let yaml = "rules:\n  - when: test";
            let error = YamlError::MissingRequired {
                field: "rules".to_string(),
                example: "[]".to_string(),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            assert!(matches!(
                scaffolded.scaffold.as_ref().unwrap().detected_intent,
                Intent::DefineRules
            ));
        }
    }

    mod working_example_tests {
        use super::*;

        #[test]
        fn test_working_example_for_character() {
            let intent = Intent::CreateCharacter {
                name: "puppy".to_string(),
            };
            let example = ScaffoldingEngine::generate_working_example(&intent);

            assert!(example.contains("character:"));
            assert!(example.contains("dog") || example.contains("bunny"));
        }

        #[test]
        fn test_working_example_for_movement() {
            let intent = Intent::SetMovement {
                style: "keyboard".to_string(),
            };
            let example = ScaffoldingEngine::generate_working_example(&intent);

            assert!(example.contains("move:"));
            assert!(example.contains("arrows") || example.contains("touch"));
        }

        #[test]
        fn test_working_example_for_event() {
            let intent = Intent::DefineEvent {
                event_type: "touch".to_string(),
            };
            let example = ScaffoldingEngine::generate_working_example(&intent);

            assert!(example.contains("when_touch:"));
            assert!(example.contains("target:"));
            assert!(example.contains("sound:"));
        }

        #[test]
        fn test_working_example_for_unknown_is_complete_game() {
            let intent = Intent::Unknown;
            let example = ScaffoldingEngine::generate_working_example(&intent);

            // Should provide a complete working game
            assert!(example.contains("character:"));
            assert!(example.contains("move:"));
            assert!(example.contains("when_touch:"));
        }
    }

    mod correction_tests {
        use super::*;

        #[test]
        fn test_correction_for_unknown_word() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "character: puppy";
            let error = YamlError::UnknownWord {
                word: "puppy".to_string(),
                suggestions: vec![],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            let corrections = &scaffolded.scaffold.unwrap().corrections;

            assert!(!corrections.is_empty());
            assert_eq!(corrections[0].line, 1);
            assert!(corrections[0].replacement.contains("dog"));
        }

        #[test]
        fn test_correction_for_missing_field() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "background: sky";
            let error = YamlError::MissingRequired {
                field: "character".to_string(),
                example: "bunny".to_string(),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            let corrections = &scaffolded.scaffold.unwrap().corrections;

            assert!(!corrections.is_empty());
            assert!(corrections[0].replacement.contains("character:"));
        }
    }

    mod syntax_fix_tests {
        use super::*;

        #[test]
        fn test_fix_missing_space_after_colon() {
            let fixed = ScaffoldingEngine::fix_common_syntax_issues("character:bunny");
            assert_eq!(fixed, Some("character: bunny".to_string()));
        }

        #[test]
        fn test_fix_tab_to_spaces() {
            let fixed = ScaffoldingEngine::fix_common_syntax_issues("\tcharacter: bunny");
            assert_eq!(fixed, Some("  character: bunny".to_string()));
        }

        #[test]
        fn test_fix_missing_colon() {
            let fixed = ScaffoldingEngine::fix_common_syntax_issues("character bunny");
            assert_eq!(fixed, Some("character: bunny".to_string()));
        }

        #[test]
        fn test_no_fix_for_valid_line() {
            let fixed = ScaffoldingEngine::fix_common_syntax_issues("character: bunny");
            assert!(fixed.is_none());
        }
    }

    mod confidence_tests {
        use super::*;

        #[test]
        fn test_confidence_high_for_known_intent() {
            let intent = Intent::CreateCharacter {
                name: "puppy".to_string(),
            };
            let corrections = vec![Correction {
                line: 1,
                original: "character: puppy".to_string(),
                replacement: "character: dog".to_string(),
                reason: "test".to_string(),
            }];

            let confidence = ScaffoldingEngine::calculate_confidence(&intent, &corrections);
            assert!(confidence >= 0.8);
        }

        #[test]
        fn test_confidence_low_for_unknown_intent() {
            let intent = Intent::Unknown;
            let corrections = vec![];

            let confidence = ScaffoldingEngine::calculate_confidence(&intent, &corrections);
            assert!(confidence <= 0.3);
        }
    }

    mod learning_hint_tests {
        use super::*;

        #[test]
        fn test_learning_hint_includes_context() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let error = YamlError::UnknownWord {
                word: "puppy".to_string(),
                suggestions: vec![],
                line: Some(1),
            };
            let intent = Intent::CreateCharacter {
                name: "puppy".to_string(),
            };

            let hint = engine.generate_learning_hint(&error, &intent);
            assert!(hint.contains("puppy"));
            assert!(hint.contains("character") || hint.contains("vocabulary"));
        }

        #[test]
        fn test_learning_hint_for_syntax_error() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let error = YamlError::SyntaxError {
                message: "test".to_string(),
                line: Some(1),
                column: Some(1),
            };
            let intent = Intent::Unknown;

            let hint = engine.generate_learning_hint(&error, &intent);
            assert!(
                hint.contains("spaces") || hint.contains("colon") || hint.contains("formatting")
            );
        }
    }

    mod render_tests {
        use super::*;

        #[test]
        fn test_scaffolded_error_render() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "character: puppy";
            let error = YamlError::UnknownWord {
                word: "puppy".to_string(),
                suggestions: vec!["bunny".to_string()],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            let rendered = scaffolded.render();

            // Should include all sections
            assert!(rendered.contains("Learning moment"));
            assert!(rendered.contains("Here's how to do it"));
            assert!(rendered.contains("```yaml"));
        }

        #[test]
        fn test_render_includes_corrections() {
            let engine = ScaffoldingEngine::new(SchemaLevel::Level1);
            let yaml = "character: puppy";
            let error = YamlError::UnknownWord {
                word: "puppy".to_string(),
                suggestions: vec![],
                line: Some(1),
            };

            let scaffolded = engine.scaffold(yaml, &error);
            let rendered = scaffolded.render();

            assert!(rendered.contains("fixes") || rendered.contains("Line 1"));
        }
    }

    mod levenshtein_tests {
        use super::*;

        #[test]
        fn test_levenshtein_identical() {
            assert_eq!(ScaffoldingEngine::levenshtein("hello", "hello"), 0);
        }

        #[test]
        fn test_levenshtein_one_char_diff() {
            assert_eq!(ScaffoldingEngine::levenshtein("hello", "hallo"), 1);
        }

        #[test]
        fn test_levenshtein_empty() {
            assert_eq!(ScaffoldingEngine::levenshtein("", "hello"), 5);
            assert_eq!(ScaffoldingEngine::levenshtein("hello", ""), 5);
        }
    }

    mod character_detection_tests {
        use super::*;

        #[test]
        fn test_detects_dog_variations() {
            assert!(ScaffoldingEngine::is_character_like("puppy"));
            assert!(ScaffoldingEngine::is_character_like("doggy"));
            assert!(ScaffoldingEngine::is_character_like("dog"));
        }

        #[test]
        fn test_detects_cat_variations() {
            assert!(ScaffoldingEngine::is_character_like("kitty"));
            assert!(ScaffoldingEngine::is_character_like("cat"));
        }
    }
}
