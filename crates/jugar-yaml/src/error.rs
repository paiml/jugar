//! Kid-friendly error handling for YAML game creation.
//!
//! Every error is a learning opportunity, not a failure.
//! Following Nintendo Quality Standards: Error prevention and helpful guidance.

use thiserror::Error;

/// Errors that can occur when parsing or compiling YAML games
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum YamlError {
    /// YAML syntax error
    #[error("YAML syntax error: {message}")]
    SyntaxError {
        /// Error message
        message: String,
        /// Line number (1-indexed)
        line: Option<usize>,
        /// Column number (1-indexed)
        column: Option<usize>,
    },

    /// Unknown keyword in the vocabulary
    #[error("Unknown word: '{word}'")]
    UnknownWord {
        /// The unknown word
        word: String,
        /// Suggested alternatives
        suggestions: Vec<String>,
        /// Location in source
        line: Option<usize>,
    },

    /// Nesting too deep for the schema level
    #[error("Nesting too deep: found {found} levels, max is {max}")]
    NestingTooDeep {
        /// Maximum allowed nesting
        max: u8,
        /// Found nesting depth
        found: u8,
    },

    /// Missing required field
    #[error("Missing required field: '{field}'")]
    MissingRequired {
        /// Name of the missing field
        field: String,
        /// Example value
        example: String,
    },

    /// Value out of valid range
    #[error("Value out of range: {field} must be between {min} and {max}, got {value}")]
    OutOfRange {
        /// Field name
        field: String,
        /// Minimum value
        min: i64,
        /// Maximum value
        max: i64,
        /// Actual value
        value: i64,
    },

    /// Invalid enum value
    #[error("Invalid value '{value}' for {field}")]
    InvalidEnumValue {
        /// Field name
        field: String,
        /// The invalid value
        value: String,
        /// Valid options
        valid_options: Vec<String>,
    },

    /// File not found (for asset references)
    #[error("File not found: '{path}'")]
    FileNotFound {
        /// Path to the missing file
        path: String,
    },

    /// Incompatible AI model
    #[error("AI model '{model}' is incompatible: {reason}")]
    IncompatibleModel {
        /// Model name
        model: String,
        /// Reason for incompatibility
        reason: String,
    },

    /// General validation error
    #[error("Validation error: {message}")]
    ValidationError {
        /// Error message
        message: String,
    },
}

impl YamlError {
    /// Convert to a kid-friendly error message
    #[must_use]
    pub fn to_kid_friendly(&self) -> KidFriendlyError {
        match self {
            Self::SyntaxError {
                message,
                line,
                column,
            } => KidFriendlyError {
                headline: "Oops, something's not quite right!".to_string(),
                explanation: format!(
                    "I had trouble reading your game. {}",
                    simplify_syntax_error(message)
                ),
                location: line.map(|l| ErrorLocation {
                    line: l,
                    column: *column,
                }),
                suggestions: vec![
                    "Check that each line is indented correctly".to_string(),
                    "Make sure colons (:) have a space after them".to_string(),
                ],
                helper: HelperCharacter::Robot,
            },

            Self::UnknownWord {
                word,
                suggestions,
                line,
            } => KidFriendlyError {
                headline: "I don't know that word!".to_string(),
                explanation: format!("Hmm, I don't know the word '{word}'."),
                location: line.map(|l| ErrorLocation {
                    line: l,
                    column: None,
                }),
                suggestions: if suggestions.is_empty() {
                    vec!["Check the spelling and try again".to_string()]
                } else {
                    suggestions
                        .iter()
                        .take(5)
                        .map(|s| format!("Did you mean '{s}'?"))
                        .collect()
                },
                helper: HelperCharacter::Owl,
            },

            Self::NestingTooDeep { max, found } => KidFriendlyError {
                headline: "That's too complicated for me!".to_string(),
                explanation: format!(
                    "You have {found} levels of nesting, but I can only handle {max}."
                ),
                location: None,
                suggestions: vec![
                    "Try keeping things simpler".to_string(),
                    "Move some parts to the top level".to_string(),
                ],
                helper: HelperCharacter::Dragon,
            },

            Self::MissingRequired { field, example } => KidFriendlyError {
                headline: "You forgot to tell me something!".to_string(),
                explanation: format!("Every game needs a '{field}' but I couldn't find one."),
                location: None,
                suggestions: vec![format!("Try adding: {field}: {example}")],
                helper: HelperCharacter::Bunny,
            },

            Self::OutOfRange {
                field,
                min,
                max,
                value,
            } => KidFriendlyError {
                headline: "That number is too big or too small!".to_string(),
                explanation: format!(
                    "The '{field}' should be between {min} and {max}, but you wrote {value}."
                ),
                location: None,
                suggestions: vec![format!("Try a number between {min} and {max}")],
                helper: HelperCharacter::Robot,
            },

            Self::InvalidEnumValue {
                field,
                value,
                valid_options,
            } => KidFriendlyError {
                headline: format!("I don't know that {field}!"),
                explanation: format!("'{value}' isn't a {field} I know about."),
                location: None,
                suggestions: valid_options
                    .iter()
                    .take(5)
                    .map(|opt| format!("Try: {field}: {opt}"))
                    .collect(),
                helper: HelperCharacter::Owl,
            },

            Self::FileNotFound { path } => KidFriendlyError {
                headline: "I can't find that file!".to_string(),
                explanation: format!("I looked for '{path}' but couldn't find it."),
                location: None,
                suggestions: vec![
                    "Check that the file name is spelled correctly".to_string(),
                    "Make sure the file is in the right folder".to_string(),
                ],
                helper: HelperCharacter::Bunny,
            },

            Self::IncompatibleModel { model, reason } => KidFriendlyError {
                headline: "That AI model doesn't fit!".to_string(),
                explanation: format!("The model '{model}' can't be used here: {reason}"),
                location: None,
                suggestions: vec![
                    "Try a different AI model".to_string(),
                    "Check that the model is the right type".to_string(),
                ],
                helper: HelperCharacter::Dragon,
            },

            Self::ValidationError { message } => KidFriendlyError {
                headline: "Something isn't quite right!".to_string(),
                explanation: message.clone(),
                location: None,
                suggestions: vec!["Check the requirements and try again".to_string()],
                helper: HelperCharacter::Owl,
            },
        }
    }
}

/// A kid-friendly error message with helpful guidance
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KidFriendlyError {
    /// Short headline (fits on one line)
    pub headline: String,
    /// Friendly explanation
    pub explanation: String,
    /// Visual pointer to the problem
    pub location: Option<ErrorLocation>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Helper character for personality
    pub helper: HelperCharacter,
}

impl KidFriendlyError {
    /// Render the error as a formatted string
    #[must_use]
    pub fn render(&self) -> String {
        use core::fmt::Write;
        let mut output = String::new();

        // Header with helper
        let _ = writeln!(output, "{} {}", self.helper.emoji(), self.headline);
        let _ = writeln!(output, "{}", "-".repeat(50));

        // Explanation
        let _ = writeln!(output, "{}\n", self.explanation);

        // Location if available
        if let Some(loc) = &self.location {
            let _ = write!(output, "Line {}", loc.line);
            if let Some(col) = loc.column {
                let _ = write!(output, ", column {col}");
            }
            output.push('\n');
        }

        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\nTry this instead:\n");
            for suggestion in &self.suggestions {
                let _ = writeln!(output, "  - {suggestion}");
            }
        }

        output
    }
}

/// Location in the source YAML
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed), if available
    pub column: Option<usize>,
}

/// Helper characters that provide friendly error guidance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperCharacter {
    /// Wise owl for unknown words and suggestions
    Owl,
    /// Helpful robot for technical/syntax issues
    Robot,
    /// Encouraging bunny for missing things
    Bunny,
    /// Brave dragon for complex problems
    Dragon,
}

impl HelperCharacter {
    /// Get the emoji for this helper character
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Owl => "🦉",
            Self::Robot => "🤖",
            Self::Bunny => "🐰",
            Self::Dragon => "🐉",
        }
    }

    /// Get a friendly phrase from this character
    #[must_use]
    pub const fn phrase(&self) -> &'static str {
        match self {
            Self::Owl => "Whooo made this mistake? Let me help!",
            Self::Robot => "BEEP BOOP! I found something to fix!",
            Self::Bunny => "Hop hop! Almost got it!",
            Self::Dragon => "Rarr! Don't worry, we'll figure it out!",
        }
    }
}

/// Simplify a technical syntax error message for kids
fn simplify_syntax_error(message: &str) -> String {
    // Map common YAML errors to simpler explanations
    let msg_lower = message.to_lowercase();

    if msg_lower.contains("expected") && msg_lower.contains("found") {
        return "Something looks out of place.".to_string();
    }
    if msg_lower.contains("mapping") {
        return "Check your indentation - each section should line up.".to_string();
    }
    if msg_lower.contains("scalar") {
        return "There might be a problem with a value.".to_string();
    }
    if msg_lower.contains("duplicate") {
        return "You used the same name twice.".to_string();
    }

    // Default: just show a simplified version
    "Something in the formatting isn't quite right.".to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_character_emoji() {
        assert_eq!(HelperCharacter::Owl.emoji(), "🦉");
        assert_eq!(HelperCharacter::Robot.emoji(), "🤖");
        assert_eq!(HelperCharacter::Bunny.emoji(), "🐰");
        assert_eq!(HelperCharacter::Dragon.emoji(), "🐉");
    }

    #[test]
    fn test_helper_character_phrase() {
        assert!(!HelperCharacter::Owl.phrase().is_empty());
        assert!(!HelperCharacter::Robot.phrase().is_empty());
        assert!(!HelperCharacter::Bunny.phrase().is_empty());
        assert!(!HelperCharacter::Dragon.phrase().is_empty());
    }

    #[test]
    fn test_kid_friendly_error_render_with_location() {
        let err = KidFriendlyError {
            headline: "Test headline".to_string(),
            explanation: "Test explanation".to_string(),
            location: Some(ErrorLocation {
                line: 5,
                column: Some(10),
            }),
            suggestions: vec!["Try this".to_string()],
            helper: HelperCharacter::Owl,
        };

        let rendered = err.render();
        assert!(rendered.contains("🦉"));
        assert!(rendered.contains("Test headline"));
        assert!(rendered.contains("Line 5"));
        assert!(rendered.contains("column 10"));
        assert!(rendered.contains("Try this"));
    }

    #[test]
    fn test_kid_friendly_error_render_without_column() {
        let err = KidFriendlyError {
            headline: "Test".to_string(),
            explanation: "Explanation".to_string(),
            location: Some(ErrorLocation {
                line: 3,
                column: None,
            }),
            suggestions: vec!["Fix".to_string()],
            helper: HelperCharacter::Robot,
        };

        let rendered = err.render();
        assert!(rendered.contains("Line 3"));
        assert!(!rendered.contains("column"));
    }

    #[test]
    fn test_kid_friendly_error_render_without_location() {
        let err = KidFriendlyError {
            headline: "Test".to_string(),
            explanation: "Explanation".to_string(),
            location: None,
            suggestions: vec![],
            helper: HelperCharacter::Bunny,
        };

        let rendered = err.render();
        assert!(!rendered.contains("Line"));
        assert!(!rendered.contains("Try this instead"));
    }

    #[test]
    fn test_syntax_error_to_kid_friendly() {
        let err = YamlError::SyntaxError {
            message: "expected scalar".to_string(),
            line: Some(10),
            column: Some(5),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("something's not quite right"));
        assert_eq!(kid_err.helper, HelperCharacter::Robot);
        assert!(kid_err.location.is_some());
        assert_eq!(kid_err.location.unwrap().line, 10);
    }

    #[test]
    fn test_unknown_word_to_kid_friendly() {
        let err = YamlError::UnknownWord {
            word: "dinosaur".to_string(),
            suggestions: vec!["dragon".to_string(), "dog".to_string()],
            line: Some(3),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("don't know"));
        assert!(kid_err.explanation.contains("dinosaur"));
        assert_eq!(kid_err.helper, HelperCharacter::Owl);
        assert!(kid_err.suggestions.iter().any(|s| s.contains("dragon")));
    }

    #[test]
    fn test_unknown_word_no_suggestions() {
        let err = YamlError::UnknownWord {
            word: "xyz".to_string(),
            suggestions: vec![],
            line: None,
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.suggestions.iter().any(|s| s.contains("spelling")));
    }

    #[test]
    fn test_nesting_too_deep_to_kid_friendly() {
        let err = YamlError::NestingTooDeep { max: 2, found: 5 };
        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("complicated"));
        assert!(kid_err.explanation.contains('5'));
        assert!(kid_err.explanation.contains('2'));
        assert_eq!(kid_err.helper, HelperCharacter::Dragon);
    }

    #[test]
    fn test_missing_required_to_kid_friendly() {
        let err = YamlError::MissingRequired {
            field: "name".to_string(),
            example: "My Game".to_string(),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("forgot"));
        assert!(kid_err.explanation.contains("name"));
        assert_eq!(kid_err.helper, HelperCharacter::Bunny);
        assert!(kid_err
            .suggestions
            .iter()
            .any(|s| s.contains("name: My Game")));
    }

    #[test]
    fn test_out_of_range_to_kid_friendly() {
        let err = YamlError::OutOfRange {
            field: "speed".to_string(),
            min: 1,
            max: 100,
            value: 200,
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("too big or too small"));
        assert!(kid_err.explanation.contains("speed"));
        assert!(kid_err.explanation.contains("200"));
        assert_eq!(kid_err.helper, HelperCharacter::Robot);
    }

    #[test]
    fn test_invalid_enum_value_to_kid_friendly() {
        let err = YamlError::InvalidEnumValue {
            field: "color".to_string(),
            value: "purple".to_string(),
            valid_options: vec!["red".to_string(), "blue".to_string()],
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("color"));
        assert!(kid_err.explanation.contains("purple"));
        assert_eq!(kid_err.helper, HelperCharacter::Owl);
        assert!(kid_err.suggestions.iter().any(|s| s.contains("red")));
    }

    #[test]
    fn test_file_not_found_to_kid_friendly() {
        let err = YamlError::FileNotFound {
            path: "sprite.png".to_string(),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("can't find"));
        assert!(kid_err.explanation.contains("sprite.png"));
        assert_eq!(kid_err.helper, HelperCharacter::Bunny);
    }

    #[test]
    fn test_incompatible_model_to_kid_friendly() {
        let err = YamlError::IncompatibleModel {
            model: "ai_model_v2".to_string(),
            reason: "too complex".to_string(),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("doesn't fit"));
        assert!(kid_err.explanation.contains("ai_model_v2"));
        assert_eq!(kid_err.helper, HelperCharacter::Dragon);
    }

    #[test]
    fn test_validation_error_to_kid_friendly() {
        let err = YamlError::ValidationError {
            message: "Custom validation failed".to_string(),
        };

        let kid_err = err.to_kid_friendly();
        assert!(kid_err.headline.contains("isn't quite right"));
        assert!(kid_err.explanation.contains("Custom validation failed"));
        assert_eq!(kid_err.helper, HelperCharacter::Owl);
    }

    #[test]
    fn test_simplify_syntax_error_expected_found() {
        let result = simplify_syntax_error("expected scalar, found mapping");
        assert!(result.contains("out of place"));
    }

    #[test]
    fn test_simplify_syntax_error_mapping() {
        let result = simplify_syntax_error("invalid mapping");
        assert!(result.contains("indentation"));
    }

    #[test]
    fn test_simplify_syntax_error_scalar() {
        let result = simplify_syntax_error("invalid scalar value");
        assert!(result.contains("value"));
    }

    #[test]
    fn test_simplify_syntax_error_duplicate() {
        let result = simplify_syntax_error("duplicate key");
        assert!(result.contains("twice"));
    }

    #[test]
    fn test_simplify_syntax_error_unknown() {
        let result = simplify_syntax_error("some random error");
        assert!(result.contains("formatting"));
    }

    #[test]
    fn test_yaml_error_display() {
        let err = YamlError::SyntaxError {
            message: "test".to_string(),
            line: Some(1),
            column: None,
        };
        assert!(err.to_string().contains("YAML syntax error"));

        let err = YamlError::UnknownWord {
            word: "foo".to_string(),
            suggestions: vec![],
            line: None,
        };
        assert!(err.to_string().contains("foo"));

        let err = YamlError::NestingTooDeep { max: 2, found: 5 };
        assert!(err.to_string().contains('5'));

        let err = YamlError::OutOfRange {
            field: "x".to_string(),
            min: 0,
            max: 100,
            value: 150,
        };
        assert!(err.to_string().contains("150"));

        let err = YamlError::FileNotFound {
            path: "test.png".to_string(),
        };
        assert!(err.to_string().contains("test.png"));
    }
}
