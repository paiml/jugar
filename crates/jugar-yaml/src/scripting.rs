//! Escape Hatch Scripting for Level 4 (Ages 11+)
//!
//! Per spec Section 14.1: "Add 'Escape Hatch': Embed Rhai/Lua scripting block
//! within YAML for custom logic (Level 4)."
//!
//! Per Resnick (2009): "Wide Walls require multiple pathways to complexity."
//!
//! # Design Philosophy
//!
//! - **Sandboxed**: Scripts run in a restricted environment
//! - **Kid-Safe**: No file system, network, or dangerous operations
//! - **Educational**: Provides clear error messages and learning hints
//! - **Progressive**: Bridges declarative YAML to imperative programming

use serde::{Deserialize, Serialize};

/// Script language supported by the escape hatch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLanguage {
    /// Rhai scripting language (default, Rust-native)
    #[default]
    Rhai,
    /// Lua scripting language (widely known)
    Lua,
    /// Simple expression language (for beginners)
    Expression,
}

impl ScriptLanguage {
    /// Get the file extension for this language
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Rhai => "rhai",
            Self::Lua => "lua",
            Self::Expression => "expr",
        }
    }

    /// Get a kid-friendly description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Rhai => "Rhai - A simple scripting language that's easy to learn!",
            Self::Lua => "Lua - A popular language used in many games!",
            Self::Expression => "Expressions - Simple math and logic, great for beginners!",
        }
    }

    /// Get the recommended age
    #[must_use]
    pub const fn recommended_age(&self) -> u8 {
        match self {
            Self::Expression => 8, // Simple enough for younger kids
            Self::Lua => 10,       // More familiar syntax
            Self::Rhai => 11,      // Most powerful
        }
    }
}

/// A script block embedded in YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptBlock {
    /// The script language
    #[serde(default)]
    pub language: ScriptLanguage,
    /// The script source code
    pub code: String,
    /// Optional name/description
    #[serde(default)]
    pub name: Option<String>,
    /// Whether this script is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[allow(clippy::missing_const_for_fn)]
fn default_true() -> bool {
    true
}

impl ScriptBlock {
    /// Create a new Rhai script block
    #[must_use]
    pub fn rhai(code: impl Into<String>) -> Self {
        Self {
            language: ScriptLanguage::Rhai,
            code: code.into(),
            name: None,
            enabled: true,
        }
    }

    /// Create a new Lua script block
    #[must_use]
    pub fn lua(code: impl Into<String>) -> Self {
        Self {
            language: ScriptLanguage::Lua,
            code: code.into(),
            name: None,
            enabled: true,
        }
    }

    /// Create a new Expression script block
    #[must_use]
    pub fn expression(code: impl Into<String>) -> Self {
        Self {
            language: ScriptLanguage::Expression,
            code: code.into(),
            name: None,
            enabled: true,
        }
    }

    /// Add a name to the script block
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Disable the script block
    #[must_use]
    pub const fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Result of script validation
#[derive(Debug, Clone)]
pub struct ScriptValidationResult {
    /// Whether the script is valid
    pub valid: bool,
    /// Errors found during validation
    pub errors: Vec<ScriptError>,
    /// Warnings (non-blocking issues)
    pub warnings: Vec<ScriptWarning>,
    /// Estimated complexity score (1-10)
    pub complexity: u8,
}

impl ScriptValidationResult {
    /// Create a successful validation result
    #[must_use]
    pub const fn valid(complexity: u8) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            complexity,
        }
    }

    /// Create a failed validation result
    #[must_use]
    pub const fn invalid(errors: Vec<ScriptError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            complexity: 0,
        }
    }
}

/// Script validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    /// Error message
    pub message: String,
    /// Line number (if known)
    pub line: Option<u32>,
    /// Column number (if known)
    pub column: Option<u32>,
    /// Kid-friendly explanation
    pub kid_message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ScriptError {
    /// Create a new script error
    #[must_use]
    pub fn new(message: impl Into<String>, kid_message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
            kid_message: kid_message.into(),
            suggestion: None,
        }
    }

    /// Add location information
    #[must_use]
    pub const fn at(mut self, line: u32, column: u32) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add a suggestion
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Script validation warning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptWarning {
    /// Warning message
    pub message: String,
    /// Line number (if known)
    pub line: Option<u32>,
    /// Kid-friendly explanation
    pub kid_message: String,
}

impl ScriptWarning {
    /// Create a new script warning
    #[must_use]
    pub fn new(message: impl Into<String>, kid_message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            kid_message: kid_message.into(),
        }
    }
}

/// Script sandbox configuration
#[derive(Debug, Clone)]
pub struct ScriptSandbox {
    /// Maximum execution time in milliseconds
    pub max_execution_ms: u64,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum recursion depth
    pub max_recursion_depth: u32,
    /// Maximum operations per execution
    pub max_operations: u64,
    /// Allowed API functions
    pub allowed_apis: Vec<String>,
    /// Blocked patterns (for safety)
    pub blocked_patterns: Vec<String>,
}

impl Default for ScriptSandbox {
    fn default() -> Self {
        Self {
            max_execution_ms: 100,         // 100ms max
            max_memory_bytes: 1024 * 1024, // 1MB max
            max_recursion_depth: 32,       // Reasonable depth
            max_operations: 10_000,        // Prevent infinite loops
            allowed_apis: Self::default_apis(),
            blocked_patterns: Self::default_blocked(),
        }
    }
}

impl ScriptSandbox {
    /// Create a new sandbox with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restricted sandbox for younger kids
    #[must_use]
    pub fn restricted() -> Self {
        Self {
            max_execution_ms: 50,
            max_memory_bytes: 512 * 1024,
            max_recursion_depth: 16,
            max_operations: 1000,
            allowed_apis: vec![
                "print".to_string(),
                "math".to_string(),
                "string".to_string(),
            ],
            blocked_patterns: Self::default_blocked(),
        }
    }

    /// Create a permissive sandbox for advanced users
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            max_execution_ms: 1000,
            max_memory_bytes: 10 * 1024 * 1024,
            max_recursion_depth: 64,
            max_operations: 100_000,
            allowed_apis: Self::extended_apis(),
            blocked_patterns: Self::default_blocked(),
        }
    }

    fn default_apis() -> Vec<String> {
        vec![
            "print".to_string(),
            "math".to_string(),
            "string".to_string(),
            "array".to_string(),
            "object".to_string(),
            "game".to_string(),
            "entity".to_string(),
            "input".to_string(),
            "audio".to_string(),
        ]
    }

    fn extended_apis() -> Vec<String> {
        let mut apis = Self::default_apis();
        apis.extend([
            "physics".to_string(),
            "ai".to_string(),
            "timer".to_string(),
            "random".to_string(),
            "debug".to_string(),
        ]);
        apis
    }

    fn default_blocked() -> Vec<String> {
        vec![
            "eval".to_string(),
            "exec".to_string(),
            "system".to_string(),
            "file".to_string(),
            "network".to_string(),
            "http".to_string(),
            "socket".to_string(),
            "process".to_string(),
            "os".to_string(),
            "import".to_string(),
            "require".to_string(),
        ]
    }
}

/// Script validator for safety checking
#[derive(Debug, Clone, Default)]
pub struct ScriptValidator {
    /// Sandbox configuration
    sandbox: ScriptSandbox,
}

impl ScriptValidator {
    /// Create a new validator with default sandbox
    #[must_use]
    pub fn new() -> Self {
        Self {
            sandbox: ScriptSandbox::default(),
        }
    }

    /// Create a validator with custom sandbox
    #[must_use]
    pub const fn with_sandbox(sandbox: ScriptSandbox) -> Self {
        Self { sandbox }
    }

    /// Validate a script block
    #[must_use]
    pub fn validate(&self, script: &ScriptBlock) -> ScriptValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for empty script
        if script.code.trim().is_empty() {
            errors.push(ScriptError::new(
                "Empty script",
                "Your script is empty! Try adding some code.",
            ));
            return ScriptValidationResult::invalid(errors);
        }

        // Check for blocked patterns
        let code_lower = script.code.to_lowercase();
        for blocked in &self.sandbox.blocked_patterns {
            if code_lower.contains(&blocked.to_lowercase()) {
                errors.push(
                    ScriptError::new(
                        format!("Blocked pattern: {blocked}"),
                        format!("Oops! You can't use '{blocked}' in your script - it's not safe!"),
                    )
                    .with_suggestion("Try using the game's built-in functions instead!"),
                );
            }
        }

        // Basic syntax validation based on language
        match script.language {
            ScriptLanguage::Rhai => {
                Self::validate_rhai_syntax(&script.code, &mut errors, &mut warnings);
            }
            ScriptLanguage::Lua => {
                Self::validate_lua_syntax(&script.code, &mut errors, &mut warnings);
            }
            ScriptLanguage::Expression => {
                Self::validate_expression_syntax(&script.code, &mut errors, &mut warnings);
            }
        }

        // Calculate complexity
        let complexity = Self::calculate_complexity(&script.code);

        if errors.is_empty() {
            ScriptValidationResult {
                valid: true,
                errors,
                warnings,
                complexity,
            }
        } else {
            ScriptValidationResult {
                valid: false,
                errors,
                warnings,
                complexity,
            }
        }
    }

    fn validate_rhai_syntax(
        code: &str,
        errors: &mut Vec<ScriptError>,
        warnings: &mut Vec<ScriptWarning>,
    ) {
        // Check for common Rhai issues
        let mut brace_count = 0i32;
        let mut paren_count = 0i32;

        for (line_num, line) in code.lines().enumerate() {
            for ch in line.chars() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    '(' => paren_count += 1,
                    ')' => paren_count -= 1,
                    _ => {}
                }
            }

            // Check for common mistakes
            if line.contains(";;") {
                warnings.push(ScriptWarning::new(
                    "Double semicolon",
                    "You have two semicolons in a row - you only need one!",
                ));
            }

            // Warn about infinite loops
            if line.contains("loop") && !line.contains("break") {
                warnings.push(ScriptWarning::new(
                    format!("Potential infinite loop at line {}", line_num + 1),
                    "Careful! This loop might run forever. Make sure to add a 'break' somewhere!",
                ));
            }
        }

        if brace_count != 0 {
            errors.push(ScriptError::new(
                "Unbalanced braces",
                "You have mismatched curly brackets {}. Make sure every { has a matching }!",
            ));
        }

        if paren_count != 0 {
            errors.push(ScriptError::new(
                "Unbalanced parentheses",
                "You have mismatched parentheses (). Make sure every ( has a matching )!",
            ));
        }
    }

    fn validate_lua_syntax(
        code: &str,
        errors: &mut Vec<ScriptError>,
        warnings: &mut Vec<ScriptWarning>,
    ) {
        // Check for common Lua issues
        let mut block_depth = 0i32;

        for (line_num, line) in code.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Count block depth
            if line_lower.contains("function")
                || line_lower.contains("if")
                || line_lower.contains("for")
                || line_lower.contains("while")
            {
                block_depth += 1;
            }
            if line_lower.contains("end") {
                block_depth -= 1;
            }

            // Check for common mistakes
            if line.contains("==")
                && line.contains('=')
                && !line.contains("==")
                && !line.contains("~=")
            {
                warnings.push(ScriptWarning::new(
                    format!("Possible assignment in condition at line {}", line_num + 1),
                    "Did you mean '==' for comparison? A single '=' assigns a value!",
                ));
            }

            // Warn about infinite loops
            if line_lower.contains("while true") && !code.contains("break") {
                warnings.push(ScriptWarning::new(
                    "Potential infinite loop",
                    "Careful! 'while true' runs forever. Add a 'break' to stop it!",
                ));
            }
        }

        if block_depth > 0 {
            errors.push(ScriptError::new(
                format!("Missing 'end' keyword ({block_depth} block(s) not closed)"),
                "You forgot to close some blocks with 'end'. Every 'function', 'if', 'for', and 'while' needs an 'end'!",
            ));
        } else if block_depth < 0 {
            errors.push(ScriptError::new(
                "Too many 'end' keywords",
                "You have too many 'end' keywords. Remove the extra ones!",
            ));
        }
    }

    fn validate_expression_syntax(
        code: &str,
        errors: &mut Vec<ScriptError>,
        warnings: &mut Vec<ScriptWarning>,
    ) {
        // Simple expression validation
        let mut paren_count = 0i32;

        for ch in code.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                _ => {}
            }
        }

        if paren_count != 0 {
            errors.push(ScriptError::new(
                "Unbalanced parentheses",
                "Check your parentheses! Every ( needs a matching ).",
            ));
        }

        // Check for invalid characters in expressions
        let allowed_chars =
            "0123456789+-*/%().,<>=!&|? abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        for ch in code.chars() {
            if !allowed_chars.contains(ch) && !ch.is_whitespace() {
                warnings.push(ScriptWarning::new(
                    format!("Unusual character: {ch}"),
                    format!("The character '{ch}' looks unusual in an expression. Is it correct?"),
                ));
                break;
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_complexity(code: &str) -> u8 {
        let mut complexity = 1u8;

        // Count control flow structures
        let code_lower = code.to_lowercase();
        complexity =
            complexity.saturating_add((code_lower.matches("if").count() as u8).saturating_mul(2));
        complexity =
            complexity.saturating_add((code_lower.matches("for").count() as u8).saturating_mul(2));
        complexity = complexity
            .saturating_add((code_lower.matches("while").count() as u8).saturating_mul(2));
        complexity = complexity
            .saturating_add((code_lower.matches("function").count() as u8).saturating_mul(3));
        complexity =
            complexity.saturating_add((code_lower.matches("loop").count() as u8).saturating_mul(2));

        // Count lines of code
        let lines = code.lines().filter(|l| !l.trim().is_empty()).count();
        complexity = complexity.saturating_add((lines / 10) as u8);

        complexity.min(10)
    }
}

/// Level 4 game definition with scripting support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level4Game {
    /// Game name
    pub game: Option<String>,
    /// Schema version
    pub version: Option<u32>,
    /// Custom scripts
    #[serde(default)]
    pub scripts: Vec<ScriptBlock>,
    /// On-start script
    #[serde(default)]
    pub on_start: Option<ScriptBlock>,
    /// On-update script (runs every frame)
    #[serde(default)]
    pub on_update: Option<ScriptBlock>,
    /// Event handlers
    #[serde(default)]
    pub handlers: std::collections::HashMap<String, ScriptBlock>,
}

impl Level4Game {
    /// Create a new empty Level 4 game
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            game: Some(name.into()),
            version: Some(4),
            scripts: Vec::new(),
            on_start: None,
            on_update: None,
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Add a script to the game
    pub fn add_script(&mut self, script: ScriptBlock) {
        self.scripts.push(script);
    }

    /// Set the on-start script
    pub fn set_on_start(&mut self, script: ScriptBlock) {
        self.on_start = Some(script);
    }

    /// Set the on-update script
    pub fn set_on_update(&mut self, script: ScriptBlock) {
        self.on_update = Some(script);
    }

    /// Add an event handler
    pub fn add_handler(&mut self, event: impl Into<String>, script: ScriptBlock) {
        let _ = self.handlers.insert(event.into(), script);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests for Escape Hatch Scripting
    // Per spec Section 14.1: "Wide Walls require multiple pathways to complexity"
    // ========================================================================

    mod script_language_tests {
        use super::*;

        #[test]
        fn test_default_language() {
            let lang = ScriptLanguage::default();
            assert_eq!(lang, ScriptLanguage::Rhai);
        }

        #[test]
        fn test_language_extensions() {
            assert_eq!(ScriptLanguage::Rhai.extension(), "rhai");
            assert_eq!(ScriptLanguage::Lua.extension(), "lua");
            assert_eq!(ScriptLanguage::Expression.extension(), "expr");
        }

        #[test]
        fn test_language_descriptions() {
            assert!(!ScriptLanguage::Rhai.description().is_empty());
            assert!(!ScriptLanguage::Lua.description().is_empty());
            assert!(!ScriptLanguage::Expression.description().is_empty());
        }

        #[test]
        fn test_recommended_ages() {
            assert!(
                ScriptLanguage::Expression.recommended_age()
                    < ScriptLanguage::Lua.recommended_age()
            );
            assert!(
                ScriptLanguage::Lua.recommended_age() <= ScriptLanguage::Rhai.recommended_age()
            );
        }
    }

    mod script_block_tests {
        use super::*;

        #[test]
        fn test_rhai_script() {
            let script = ScriptBlock::rhai("print(\"Hello!\");");
            assert_eq!(script.language, ScriptLanguage::Rhai);
            assert!(script.enabled);
        }

        #[test]
        fn test_lua_script() {
            let script = ScriptBlock::lua("print(\"Hello!\")");
            assert_eq!(script.language, ScriptLanguage::Lua);
        }

        #[test]
        fn test_expression_script() {
            let script = ScriptBlock::expression("score + 10");
            assert_eq!(script.language, ScriptLanguage::Expression);
        }

        #[test]
        fn test_script_with_name() {
            let script = ScriptBlock::rhai("x = 1").with_name("Initialize");
            assert_eq!(script.name, Some("Initialize".to_string()));
        }

        #[test]
        fn test_disabled_script() {
            let script = ScriptBlock::rhai("x = 1").disabled();
            assert!(!script.enabled);
        }
    }

    mod script_sandbox_tests {
        use super::*;

        #[test]
        fn test_default_sandbox() {
            let sandbox = ScriptSandbox::default();
            assert!(sandbox.max_execution_ms > 0);
            assert!(sandbox.max_memory_bytes > 0);
            assert!(!sandbox.blocked_patterns.is_empty());
        }

        #[test]
        fn test_restricted_sandbox() {
            let restricted = ScriptSandbox::restricted();
            let default = ScriptSandbox::default();
            assert!(restricted.max_execution_ms < default.max_execution_ms);
            assert!(restricted.max_operations < default.max_operations);
        }

        #[test]
        fn test_permissive_sandbox() {
            let permissive = ScriptSandbox::permissive();
            let default = ScriptSandbox::default();
            assert!(permissive.max_execution_ms > default.max_execution_ms);
            assert!(permissive.max_operations > default.max_operations);
        }

        #[test]
        fn test_blocked_patterns() {
            let sandbox = ScriptSandbox::default();
            assert!(sandbox.blocked_patterns.contains(&"eval".to_string()));
            assert!(sandbox.blocked_patterns.contains(&"file".to_string()));
            assert!(sandbox.blocked_patterns.contains(&"network".to_string()));
        }
    }

    mod script_validator_tests {
        use super::*;

        #[test]
        fn test_valid_rhai_script() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::rhai(
                r"
                let score = 0;
                score = score + 10;
                print(score);
            ",
            );
            let result = validator.validate(&script);
            assert!(result.valid);
        }

        #[test]
        fn test_empty_script_invalid() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::rhai("");
            let result = validator.validate(&script);
            assert!(!result.valid);
            assert!(!result.errors.is_empty());
        }

        #[test]
        fn test_blocked_pattern_detected() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::rhai("eval('malicious')");
            let result = validator.validate(&script);
            assert!(!result.valid);
            assert!(result.errors.iter().any(|e| e.message.contains("Blocked")));
        }

        #[test]
        fn test_unbalanced_braces_rhai() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::rhai("if x { y }");
            let result = validator.validate(&script);
            // This is actually balanced, so should be valid
            assert!(result.valid);

            let script2 = ScriptBlock::rhai("if x { y");
            let result2 = validator.validate(&script2);
            assert!(!result2.valid);
        }

        #[test]
        fn test_valid_lua_script() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::lua(
                r"
                local score = 0
                score = score + 10
                print(score)
            ",
            );
            let result = validator.validate(&script);
            assert!(result.valid);
        }

        #[test]
        fn test_lua_missing_end() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::lua(
                r#"
                function test()
                    print("hello")
            "#,
            );
            let result = validator.validate(&script);
            assert!(!result.valid);
            assert!(result.errors.iter().any(|e| e.message.contains("end")));
        }

        #[test]
        fn test_valid_expression() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::expression("score + 10 * bonus");
            let result = validator.validate(&script);
            assert!(result.valid);
        }

        #[test]
        fn test_expression_unbalanced_parens() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::expression("(score + 10");
            let result = validator.validate(&script);
            assert!(!result.valid);
        }

        #[test]
        fn test_complexity_calculation() {
            let validator = ScriptValidator::new();

            let simple = ScriptBlock::rhai("let x = 1;");
            let simple_result = validator.validate(&simple);

            let complex = ScriptBlock::rhai(
                r"
                function calculate(x) {
                    if x > 10 {
                        for i in range(0, x) {
                            if i % 2 == 0 {
                                print(i);
                            }
                        }
                    }
                }
            ",
            );
            let complex_result = validator.validate(&complex);

            assert!(complex_result.complexity > simple_result.complexity);
        }

        #[test]
        fn test_infinite_loop_warning() {
            let validator = ScriptValidator::new();
            let script = ScriptBlock::rhai("loop { x = x + 1; }");
            let result = validator.validate(&script);
            // Should warn about potential infinite loop
            assert!(result
                .warnings
                .iter()
                .any(|w| w.message.contains("infinite") || w.message.contains("loop")));
        }
    }

    mod script_error_tests {
        use super::*;

        #[test]
        fn test_error_with_location() {
            let error = ScriptError::new("Test error", "Oops!").at(5, 10);
            assert_eq!(error.line, Some(5));
            assert_eq!(error.column, Some(10));
        }

        #[test]
        fn test_error_with_suggestion() {
            let error = ScriptError::new("Test error", "Oops!").with_suggestion("Try this instead");
            assert_eq!(error.suggestion, Some("Try this instead".to_string()));
        }
    }

    mod level4_game_tests {
        use super::*;

        #[test]
        fn test_create_level4_game() {
            let game = Level4Game::new("my-scripted-game");
            assert_eq!(game.game, Some("my-scripted-game".to_string()));
            assert_eq!(game.version, Some(4));
        }

        #[test]
        fn test_add_scripts() {
            let mut game = Level4Game::new("test");
            game.add_script(ScriptBlock::rhai("let x = 1;"));
            game.add_script(ScriptBlock::rhai("let y = 2;"));
            assert_eq!(game.scripts.len(), 2);
        }

        #[test]
        fn test_set_lifecycle_scripts() {
            let mut game = Level4Game::new("test");
            game.set_on_start(ScriptBlock::rhai("init();"));
            game.set_on_update(ScriptBlock::rhai("update();"));
            assert!(game.on_start.is_some());
            assert!(game.on_update.is_some());
        }

        #[test]
        fn test_add_handlers() {
            let mut game = Level4Game::new("test");
            game.add_handler("on_touch", ScriptBlock::rhai("handle_touch();"));
            game.add_handler("on_collision", ScriptBlock::rhai("handle_collision();"));
            assert_eq!(game.handlers.len(), 2);
            assert!(game.handlers.contains_key("on_touch"));
        }
    }

    mod serde_tests {
        use super::*;

        #[test]
        fn test_script_language_serde() {
            let script = ScriptBlock::lua("print('test')");
            let json = serde_json::to_string(&script).unwrap();
            assert!(json.contains("\"language\":\"lua\""));
        }

        #[test]
        fn test_level4_game_serde() {
            let mut game = Level4Game::new("test");
            game.add_script(ScriptBlock::rhai("let x = 1;"));

            let json = serde_json::to_string(&game).unwrap();
            assert!(json.contains("\"game\":\"test\""));
            assert!(json.contains("\"version\":4"));
        }
    }
}
