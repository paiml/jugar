//! Accessibility validation for games.
//!
//! Per spec Section 6.3 and 11.3: WCAG 2.1 AA compliance validation.

use crate::error::YamlError;

/// Accessibility validation results
#[derive(Debug, Clone, Default)]
pub struct AccessibilityReport {
    /// List of issues found
    pub issues: Vec<AccessibilityIssue>,
    /// Warnings (non-blocking)
    pub warnings: Vec<AccessibilityWarning>,
    /// Whether the game passes minimum accessibility standards
    pub passes_minimum: bool,
}

impl AccessibilityReport {
    /// Create a passing report
    #[must_use]
    pub const fn pass() -> Self {
        Self {
            issues: Vec::new(),
            warnings: Vec::new(),
            passes_minimum: true,
        }
    }

    /// Add an issue
    pub fn add_issue(&mut self, issue: AccessibilityIssue) {
        self.issues.push(issue);
        self.passes_minimum = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: AccessibilityWarning) {
        self.warnings.push(warning);
    }

    /// Check if report is clean (no issues or warnings)
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty() && self.warnings.is_empty()
    }

    /// Get total issue count
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

/// An accessibility issue (blocking)
#[derive(Debug, Clone)]
pub struct AccessibilityIssue {
    /// Issue code
    pub code: AccessibilityCode,
    /// Human-readable description
    pub description: String,
    /// WCAG guideline reference
    pub wcag_ref: Option<String>,
    /// Suggested fix
    pub suggestion: String,
}

/// An accessibility warning (non-blocking)
#[derive(Debug, Clone)]
pub struct AccessibilityWarning {
    /// Warning code
    pub code: AccessibilityCode,
    /// Human-readable description
    pub description: String,
    /// Suggested improvement
    pub suggestion: String,
}

/// Accessibility issue codes per spec Section 11.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityCode {
    /// Missing keyboard navigation
    MissingKeyboardNav,
    /// No screen reader support
    NoScreenReader,
    /// Color contrast too low
    LowContrast,
    /// Missing audio cues
    MissingAudioCues,
    /// No pause functionality
    NoPause,
    /// Time limits without extension
    TimeLimit,
    /// Flashing content
    FlashingContent,
    /// Missing text alternatives
    MissingAltText,
    /// Inconsistent navigation
    InconsistentNav,
    /// Small touch targets
    SmallTouchTargets,
}

impl AccessibilityCode {
    /// Get WCAG reference for this code
    #[must_use]
    pub const fn wcag_ref(&self) -> &'static str {
        match self {
            Self::MissingKeyboardNav => "WCAG 2.1.1",
            Self::NoScreenReader => "WCAG 4.1.2",
            Self::LowContrast => "WCAG 1.4.3",
            Self::MissingAudioCues => "WCAG 1.4.1",
            Self::NoPause => "WCAG 2.2.2",
            Self::TimeLimit => "WCAG 2.2.1",
            Self::FlashingContent => "WCAG 2.3.1",
            Self::MissingAltText => "WCAG 1.1.1",
            Self::InconsistentNav => "WCAG 3.2.3",
            Self::SmallTouchTargets => "WCAG 2.5.5",
        }
    }
}

/// Accessibility validator
#[derive(Debug, Clone, Default)]
pub struct AccessibilityValidator {
    /// Whether to check for screen reader support
    pub check_screen_reader: bool,
    /// Minimum contrast ratio (default 4.5:1 for AA)
    pub min_contrast_ratio: f32,
    /// Minimum touch target size in pixels
    pub min_touch_target: u32,
}

impl AccessibilityValidator {
    /// Create a new validator with defaults
    #[must_use]
    pub const fn new() -> Self {
        Self {
            check_screen_reader: true,
            min_contrast_ratio: 4.5, // WCAG AA
            min_touch_target: 44,    // WCAG 2.5.5
        }
    }

    /// Validate a parsed game configuration
    #[must_use]
    pub fn validate(&self, game: &GameAccessibilityInfo) -> AccessibilityReport {
        let mut report = AccessibilityReport::pass();

        // Check keyboard navigation
        if !game.has_keyboard_nav {
            report.add_issue(AccessibilityIssue {
                code: AccessibilityCode::MissingKeyboardNav,
                description: "Game requires keyboard navigation support".to_string(),
                wcag_ref: Some("WCAG 2.1.1".to_string()),
                suggestion: "Add 'move: arrows' or 'move: wasd' to enable keyboard control"
                    .to_string(),
            });
        }

        // Check pause functionality
        if !game.has_pause {
            report.add_warning(AccessibilityWarning {
                code: AccessibilityCode::NoPause,
                description: "Game should have pause functionality".to_string(),
                suggestion: "Consider adding pause support for accessibility".to_string(),
            });
        }

        // Check touch targets
        if game.min_touch_target < self.min_touch_target {
            report.add_warning(AccessibilityWarning {
                code: AccessibilityCode::SmallTouchTargets,
                description: format!(
                    "Touch targets should be at least {}px (found {}px)",
                    self.min_touch_target, game.min_touch_target
                ),
                suggestion: "Increase interactive element sizes for better accessibility"
                    .to_string(),
            });
        }

        // Check audio alternatives
        if game.has_audio_only_cues && !game.has_visual_alternatives {
            report.add_issue(AccessibilityIssue {
                code: AccessibilityCode::MissingAudioCues,
                description: "Audio cues need visual alternatives".to_string(),
                wcag_ref: Some("WCAG 1.4.1".to_string()),
                suggestion: "Add visual feedback for all audio cues".to_string(),
            });
        }

        // Check time limits
        if game.has_time_limit && !game.time_limit_extendable {
            report.add_warning(AccessibilityWarning {
                code: AccessibilityCode::TimeLimit,
                description: "Time limits should be adjustable".to_string(),
                suggestion: "Add option to extend or disable time limits".to_string(),
            });
        }

        report
    }

    /// Quick check if a YAML game definition is accessible
    ///
    /// # Errors
    ///
    /// Returns error if YAML is invalid
    pub fn check_yaml(&self, yaml: &str) -> Result<AccessibilityReport, YamlError> {
        // Parse YAML to extract accessibility info
        let info = GameAccessibilityInfo::from_yaml(yaml)?;
        Ok(self.validate(&info))
    }
}

/// Accessibility information extracted from a game
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // These are all distinct accessibility flags
pub struct GameAccessibilityInfo {
    /// Has keyboard navigation
    pub has_keyboard_nav: bool,
    /// Has touch input
    pub has_touch_input: bool,
    /// Has pause functionality
    pub has_pause: bool,
    /// Minimum touch target size
    pub min_touch_target: u32,
    /// Has audio-only cues
    pub has_audio_only_cues: bool,
    /// Has visual alternatives for audio
    pub has_visual_alternatives: bool,
    /// Has time limit
    pub has_time_limit: bool,
    /// Time limit is extendable
    pub time_limit_extendable: bool,
    /// Color contrast ratio (lowest found)
    pub min_contrast_ratio: f32,
}

impl Default for GameAccessibilityInfo {
    fn default() -> Self {
        Self {
            has_keyboard_nav: false,
            has_touch_input: false,
            has_pause: false,
            min_touch_target: 44, // WCAG 2.5.5 default
            has_audio_only_cues: false,
            has_visual_alternatives: false,
            has_time_limit: false,
            time_limit_extendable: false,
            min_contrast_ratio: 0.0,
        }
    }
}

impl GameAccessibilityInfo {
    /// Extract accessibility info from YAML
    ///
    /// # Errors
    ///
    /// Returns error if YAML is invalid
    pub fn from_yaml(yaml: &str) -> Result<Self, YamlError> {
        let doc: serde_yaml::Value =
            serde_yaml::from_str(yaml).map_err(|e| YamlError::SyntaxError {
                message: e.to_string(),
                line: e.location().map(|l| l.line()),
                column: e.location().map(|l| l.column()),
            })?;

        let mut info = Self::default();

        // Check for keyboard navigation
        if let Some(move_type) = doc.get("move").and_then(|v| v.as_str()) {
            info.has_keyboard_nav = matches!(move_type, "arrows" | "wasd" | "keyboard");
        }

        // Check for touch input
        if doc.get("when_touch").is_some() {
            info.has_touch_input = true;
        }

        // Check for audio
        if doc.get("music").is_some() || doc.get("sound").is_some() {
            info.has_audio_only_cues = true;
            // For now, assume visual alternatives exist if there's a background
            info.has_visual_alternatives = doc.get("background").is_some();
        }

        // Check characters for movement
        if let Some(chars) = doc.get("characters") {
            if let Some(mapping) = chars.as_mapping() {
                for (_key, char_def) in mapping {
                    if let Some(move_type) = char_def.get("move").and_then(|v| v.as_str()) {
                        if matches!(move_type, "arrows" | "wasd" | "keyboard") {
                            info.has_keyboard_nav = true;
                        }
                    }
                }
            }
        }

        // Level 1 games are always accessible by default
        if doc.get("character").is_some() && doc.get("characters").is_none() {
            info.has_keyboard_nav = true; // Level 1 defaults to arrows
            info.has_pause = true; // Engine provides pause
        }

        Ok(info)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 6.3 and 11.3
    // ========================================================================

    mod accessibility_code_tests {
        use super::*;

        #[test]
        fn test_wcag_references() {
            assert_eq!(
                AccessibilityCode::MissingKeyboardNav.wcag_ref(),
                "WCAG 2.1.1"
            );
            assert_eq!(AccessibilityCode::LowContrast.wcag_ref(), "WCAG 1.4.3");
            assert_eq!(AccessibilityCode::FlashingContent.wcag_ref(), "WCAG 2.3.1");
        }
    }

    mod report_tests {
        use super::*;

        #[test]
        fn test_pass_report() {
            let report = AccessibilityReport::pass();
            assert!(report.passes_minimum);
            assert!(report.is_clean());
        }

        #[test]
        fn test_add_issue() {
            let mut report = AccessibilityReport::pass();
            report.add_issue(AccessibilityIssue {
                code: AccessibilityCode::MissingKeyboardNav,
                description: "test".to_string(),
                wcag_ref: None,
                suggestion: "fix".to_string(),
            });
            assert!(!report.passes_minimum);
            assert_eq!(report.issue_count(), 1);
        }

        #[test]
        fn test_add_warning() {
            let mut report = AccessibilityReport::pass();
            report.add_warning(AccessibilityWarning {
                code: AccessibilityCode::NoPause,
                description: "test".to_string(),
                suggestion: "fix".to_string(),
            });
            assert!(report.passes_minimum); // Warnings don't fail
            assert!(!report.is_clean());
        }
    }

    mod validator_tests {
        use super::*;

        #[test]
        fn test_validator_defaults() {
            let validator = AccessibilityValidator::new();
            assert!(validator.check_screen_reader);
            assert!((validator.min_contrast_ratio - 4.5).abs() < 0.1);
            assert_eq!(validator.min_touch_target, 44);
        }

        #[test]
        fn test_validates_keyboard_nav() {
            let validator = AccessibilityValidator::new();
            let info = GameAccessibilityInfo {
                has_keyboard_nav: false,
                ..Default::default()
            };
            let report = validator.validate(&info);
            assert!(!report.passes_minimum);
        }

        #[test]
        fn test_passes_with_keyboard() {
            let validator = AccessibilityValidator::new();
            let info = GameAccessibilityInfo {
                has_keyboard_nav: true,
                has_visual_alternatives: true,
                min_touch_target: 44,
                ..Default::default()
            };
            let report = validator.validate(&info);
            assert!(report.passes_minimum);
        }

        #[test]
        fn test_warns_small_touch_targets() {
            let validator = AccessibilityValidator::new();
            let info = GameAccessibilityInfo {
                has_keyboard_nav: true,
                min_touch_target: 20, // Too small
                ..Default::default()
            };
            let report = validator.validate(&info);
            assert!(report
                .warnings
                .iter()
                .any(|w| w.code == AccessibilityCode::SmallTouchTargets));
        }
    }

    mod yaml_extraction_tests {
        use super::*;

        #[test]
        fn test_extract_from_level1() {
            let yaml = r"
character: bunny
move: arrows
background: sky
";
            let info = GameAccessibilityInfo::from_yaml(yaml).unwrap();
            assert!(info.has_keyboard_nav);
        }

        #[test]
        fn test_extract_from_level2() {
            let yaml = r"
characters:
  player:
    type: bunny
    move: wasd
";
            let info = GameAccessibilityInfo::from_yaml(yaml).unwrap();
            assert!(info.has_keyboard_nav);
        }

        #[test]
        fn test_detect_touch_input() {
            let yaml = r"
character: bunny
when_touch:
  target: star
";
            let info = GameAccessibilityInfo::from_yaml(yaml).unwrap();
            assert!(info.has_touch_input);
        }

        #[test]
        fn test_level1_defaults_accessible() {
            let yaml = "character: bunny";
            let info = GameAccessibilityInfo::from_yaml(yaml).unwrap();
            // Level 1 games default to accessible
            assert!(info.has_keyboard_nav);
            assert!(info.has_pause);
        }
    }

    mod full_validation_tests {
        use super::*;

        #[test]
        fn test_check_yaml_valid() {
            let validator = AccessibilityValidator::new();
            let yaml = r"
character: bunny
move: arrows
background: sky
";
            let report = validator.check_yaml(yaml).unwrap();
            assert!(report.passes_minimum);
        }

        #[test]
        fn test_check_yaml_invalid() {
            let validator = AccessibilityValidator::new();
            let yaml = "{ invalid yaml";
            let result = validator.check_yaml(yaml);
            assert!(result.is_err());
        }
    }
}
