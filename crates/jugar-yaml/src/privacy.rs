//! COPPA compliance and privacy-first analytics.
//!
//! Per spec Section 9.2: Privacy-First Analytics - COPPA compliant data collection.
//! Per spec Section 14.1: Implements Differential Privacy for population-level
//! retention tracking without individual user tracking.
//!
//! Key principles:
//! - No personally identifiable information (PII) collected
//! - No tracking across sessions
//! - No third-party analytics
//! - All data stays on-device by default
//! - Differential privacy noise injection for aggregate statistics

use serde::{Deserialize, Serialize};

/// COPPA compliance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComplianceLevel {
    /// Full COPPA compliance (default) - no data collection
    #[default]
    Full,
    /// Minimal analytics - aggregate counts only, no identifiers
    MinimalAnalytics,
    /// Parental consent obtained - limited analytics allowed
    ParentalConsent,
}

impl ComplianceLevel {
    /// Check if any analytics are allowed
    #[must_use]
    pub const fn allows_analytics(&self) -> bool {
        !matches!(self, Self::Full)
    }

    /// Check if session tracking is allowed
    #[must_use]
    pub const fn allows_session_tracking(&self) -> bool {
        matches!(self, Self::ParentalConsent)
    }

    /// Get description for display
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Full => "No data collection (COPPA compliant)",
            Self::MinimalAnalytics => "Anonymous play counts only",
            Self::ParentalConsent => "Basic analytics with parental consent",
        }
    }
}

/// Privacy configuration for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Compliance level
    #[serde(default)]
    pub compliance_level: ComplianceLevelSerde,
    /// Whether to show privacy notice
    #[serde(default = "default_true")]
    pub show_privacy_notice: bool,
    /// Custom privacy notice text (optional)
    pub privacy_notice_text: Option<String>,
    /// Whether data export is enabled
    #[serde(default = "default_true")]
    pub data_export_enabled: bool,
    /// Whether data deletion is enabled
    #[serde(default = "default_true")]
    pub data_deletion_enabled: bool,
}

#[allow(clippy::missing_const_for_fn)] // serde default requires non-const fn
fn default_true() -> bool {
    true
}

/// Serde-compatible compliance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceLevelSerde {
    /// Full COPPA compliance (default)
    #[default]
    Full,
    /// Minimal analytics with aggregate counts only
    MinimalAnalytics,
    /// Analytics with parental consent
    ParentalConsent,
}

impl From<ComplianceLevelSerde> for ComplianceLevel {
    fn from(serde: ComplianceLevelSerde) -> Self {
        match serde {
            ComplianceLevelSerde::Full => Self::Full,
            ComplianceLevelSerde::MinimalAnalytics => Self::MinimalAnalytics,
            ComplianceLevelSerde::ParentalConsent => Self::ParentalConsent,
        }
    }
}

impl From<ComplianceLevel> for ComplianceLevelSerde {
    fn from(level: ComplianceLevel) -> Self {
        match level {
            ComplianceLevel::Full => Self::Full,
            ComplianceLevel::MinimalAnalytics => Self::MinimalAnalytics,
            ComplianceLevel::ParentalConsent => Self::ParentalConsent,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            compliance_level: ComplianceLevelSerde::Full,
            show_privacy_notice: true,
            privacy_notice_text: None,
            data_export_enabled: true,
            data_deletion_enabled: true,
        }
    }
}

impl PrivacyConfig {
    /// Create a new config with full COPPA compliance
    #[must_use]
    pub const fn new() -> Self {
        Self {
            compliance_level: ComplianceLevelSerde::Full,
            show_privacy_notice: true,
            privacy_notice_text: None,
            data_export_enabled: true,
            data_deletion_enabled: true,
        }
    }

    /// Get the compliance level
    #[must_use]
    pub const fn compliance(&self) -> ComplianceLevel {
        match self.compliance_level {
            ComplianceLevelSerde::Full => ComplianceLevel::Full,
            ComplianceLevelSerde::MinimalAnalytics => ComplianceLevel::MinimalAnalytics,
            ComplianceLevelSerde::ParentalConsent => ComplianceLevel::ParentalConsent,
        }
    }

    /// Validate that config meets COPPA requirements
    #[must_use]
    pub fn validate(&self) -> PrivacyValidationResult {
        let mut issues = Vec::new();

        // Data export must be enabled
        if !self.data_export_enabled {
            issues.push(PrivacyIssue::DataExportRequired);
        }

        // Data deletion must be enabled
        if !self.data_deletion_enabled {
            issues.push(PrivacyIssue::DataDeletionRequired);
        }

        // Privacy notice recommended
        if !self.show_privacy_notice {
            issues.push(PrivacyIssue::PrivacyNoticeRecommended);
        }

        PrivacyValidationResult {
            compliant: issues.iter().all(|i| !i.is_blocking()),
            issues,
        }
    }
}

/// Result of privacy validation
#[derive(Debug, Clone)]
pub struct PrivacyValidationResult {
    /// Whether the config is COPPA compliant
    pub compliant: bool,
    /// List of issues found
    pub issues: Vec<PrivacyIssue>,
}

impl PrivacyValidationResult {
    /// Check if validation passed
    #[must_use]
    pub const fn is_compliant(&self) -> bool {
        self.compliant
    }

    /// Get blocking issues
    #[must_use]
    pub fn blocking_issues(&self) -> Vec<&PrivacyIssue> {
        self.issues.iter().filter(|i| i.is_blocking()).collect()
    }
}

/// Privacy compliance issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyIssue {
    /// Data export must be enabled
    DataExportRequired,
    /// Data deletion must be enabled
    DataDeletionRequired,
    /// Privacy notice is recommended
    PrivacyNoticeRecommended,
    /// Session tracking requires parental consent
    SessionTrackingRequiresConsent,
    /// PII collection not allowed
    PiiCollectionNotAllowed,
}

impl PrivacyIssue {
    /// Check if this issue blocks compliance
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(
            self,
            Self::DataExportRequired | Self::DataDeletionRequired | Self::PiiCollectionNotAllowed
        )
    }

    /// Get human-readable description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::DataExportRequired => "Users must be able to export their data",
            Self::DataDeletionRequired => "Users must be able to delete their data",
            Self::PrivacyNoticeRecommended => "A privacy notice should be displayed",
            Self::SessionTrackingRequiresConsent => "Session tracking requires parental consent",
            Self::PiiCollectionNotAllowed => {
                "Personally identifiable information cannot be collected"
            }
        }
    }
}

/// Anonymous analytics event (COPPA compliant)
///
/// These events contain NO personally identifiable information.
/// All data is aggregated and cannot be traced to individuals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousEvent {
    /// Event type
    pub event_type: EventType,
    /// Timestamp (rounded to hour for anonymity)
    pub timestamp_hour: u64,
    /// Game identifier (from YAML)
    pub game_id: Option<String>,
    /// Anonymous session hash (not traceable)
    pub session_hash: Option<u32>,
}

/// Types of anonymous events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Game started
    GameStart,
    /// Game completed
    GameComplete,
    /// Level completed
    LevelComplete,
    /// Error occurred
    Error,
}

impl AnonymousEvent {
    /// Create a game start event
    #[must_use]
    pub const fn game_start(game_id: Option<String>) -> Self {
        Self {
            event_type: EventType::GameStart,
            timestamp_hour: current_hour_timestamp(),
            game_id,
            session_hash: None,
        }
    }

    /// Create a game complete event
    #[must_use]
    pub const fn game_complete(game_id: Option<String>) -> Self {
        Self {
            event_type: EventType::GameComplete,
            timestamp_hour: current_hour_timestamp(),
            game_id,
            session_hash: None,
        }
    }

    /// Create a level complete event
    #[must_use]
    pub const fn level_complete(game_id: Option<String>) -> Self {
        Self {
            event_type: EventType::LevelComplete,
            timestamp_hour: current_hour_timestamp(),
            game_id,
            session_hash: None,
        }
    }

    /// Create an error event
    #[must_use]
    pub const fn error(game_id: Option<String>) -> Self {
        Self {
            event_type: EventType::Error,
            timestamp_hour: current_hour_timestamp(),
            game_id,
            session_hash: None,
        }
    }
}

/// Get current timestamp rounded to hour (for anonymity)
const fn current_hour_timestamp() -> u64 {
    // In a real implementation, this would use actual time
    // Rounded to hour to prevent timing attacks
    0
}

/// On-device analytics storage (no cloud sync)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalAnalytics {
    /// Total play count
    pub play_count: u64,
    /// Total levels completed
    pub levels_completed: u64,
    /// Total play time in seconds (approximate)
    pub play_time_seconds: u64,
    /// Last play date (YYYY-MM-DD format, no time)
    pub last_play_date: Option<String>,
}

impl LocalAnalytics {
    /// Create new empty analytics
    #[must_use]
    pub const fn new() -> Self {
        Self {
            play_count: 0,
            levels_completed: 0,
            play_time_seconds: 0,
            last_play_date: None,
        }
    }

    /// Record a play session
    pub const fn record_play(&mut self, duration_seconds: u64) {
        self.play_count = self.play_count.saturating_add(1);
        self.play_time_seconds = self.play_time_seconds.saturating_add(duration_seconds);
    }

    /// Record a level completion
    pub const fn record_level_complete(&mut self) {
        self.levels_completed = self.levels_completed.saturating_add(1);
    }

    /// Export data as JSON (for COPPA compliance)
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Clear all analytics data (for COPPA compliance)
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Apply differential privacy noise to analytics for aggregate reporting.
    ///
    /// Per spec Section 14.1: Implements local noise injection before aggregation
    /// to allow population-level retention tracking without user tracking.
    #[must_use]
    pub fn with_differential_privacy(&self, config: &DifferentialPrivacyConfig) -> NoisyAnalytics {
        let noise_gen = DifferentialPrivacy::new(config.epsilon, config.sensitivity);

        NoisyAnalytics {
            play_count: noise_gen.add_laplace_noise_u64(self.play_count),
            levels_completed: noise_gen.add_laplace_noise_u64(self.levels_completed),
            play_time_seconds: noise_gen.add_laplace_noise_u64(self.play_time_seconds),
            epsilon: config.epsilon,
            sensitivity: config.sensitivity,
        }
    }
}

// ============================================================================
// DIFFERENTIAL PRIVACY IMPLEMENTATION
// Per spec Section 14.1: "Implement local noise injection before aggregation"
// ============================================================================

/// Configuration for differential privacy noise injection
#[derive(Debug, Clone, Copy)]
pub struct DifferentialPrivacyConfig {
    /// Privacy budget (epsilon). Lower = more privacy, more noise.
    /// - 0.1 = Very strong privacy (high noise)
    /// - 1.0 = Standard privacy (moderate noise)
    /// - 10.0 = Weak privacy (low noise)
    pub epsilon: f64,
    /// Sensitivity: maximum change one record can cause.
    /// For counting queries, this is typically 1.
    pub sensitivity: f64,
}

impl Default for DifferentialPrivacyConfig {
    fn default() -> Self {
        Self {
            epsilon: 1.0,     // Standard privacy level
            sensitivity: 1.0, // Single record impact
        }
    }
}

impl DifferentialPrivacyConfig {
    /// Create a new config with specified epsilon
    #[must_use]
    pub const fn new(epsilon: f64, sensitivity: f64) -> Self {
        Self {
            epsilon,
            sensitivity,
        }
    }

    /// Config for strong privacy (more noise)
    #[must_use]
    pub const fn strong_privacy() -> Self {
        Self {
            epsilon: 0.1,
            sensitivity: 1.0,
        }
    }

    /// Config for moderate privacy
    #[must_use]
    pub const fn moderate_privacy() -> Self {
        Self {
            epsilon: 1.0,
            sensitivity: 1.0,
        }
    }

    /// Config for weak privacy (less noise, better accuracy)
    #[must_use]
    pub const fn weak_privacy() -> Self {
        Self {
            epsilon: 10.0,
            sensitivity: 1.0,
        }
    }
}

/// Differential privacy noise generator using Laplace mechanism
///
/// The Laplace mechanism adds noise drawn from a Laplace distribution
/// centered at 0 with scale parameter b = sensitivity / epsilon.
///
/// Reference: d'Alessandro et al. (2017) - Privacy-Preserving Learning Analytics
#[derive(Debug, Clone, Copy)]
pub struct DifferentialPrivacy {
    /// Privacy parameter (stored for transparency/auditing)
    #[allow(dead_code)]
    epsilon: f64,
    /// Query sensitivity (stored for transparency/auditing)
    #[allow(dead_code)]
    sensitivity: f64,
    /// Scale parameter for Laplace distribution (b = sensitivity / epsilon)
    scale: f64,
}

impl DifferentialPrivacy {
    /// Create a new differential privacy noise generator
    #[must_use]
    pub fn new(epsilon: f64, sensitivity: f64) -> Self {
        let scale = sensitivity / epsilon.max(0.001); // Avoid division by zero
        Self {
            epsilon,
            sensitivity,
            scale,
        }
    }

    /// Get the scale parameter (b) for the Laplace distribution
    #[must_use]
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Generate Laplace noise using inverse CDF method
    ///
    /// Laplace(0, b) can be sampled as: -b * sign(u - 0.5) * ln(1 - 2|u - 0.5|)
    /// where u ~ Uniform(0, 1)
    #[must_use]
    pub fn sample_laplace(&self, random_u: f64) -> f64 {
        let u = random_u.clamp(0.0001, 0.9999); // Avoid log(0)
        let shifted = u - 0.5;
        let sign = if shifted >= 0.0 { 1.0 } else { -1.0 };
        -self.scale * sign * 2.0f64.mul_add(-shifted.abs(), 1.0).ln()
    }

    /// Add Laplace noise to a value
    #[must_use]
    pub fn add_laplace_noise(&self, value: f64, random_u: f64) -> f64 {
        value + self.sample_laplace(random_u)
    }

    /// Add Laplace noise to a u64 count using deterministic seed
    ///
    /// Uses a simple hash of the value as pseudo-random source for reproducibility
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn add_laplace_noise_u64(&self, value: u64) -> i64 {
        // Use value itself as seed for deterministic noise (for testing)
        // In production, this would use a secure random source
        let random_u = Self::pseudo_random_from_seed(value);
        let noisy = self.add_laplace_noise(value as f64, random_u);
        noisy.round() as i64
    }

    /// Simple pseudo-random generator from seed (for deterministic testing)
    #[allow(clippy::cast_precision_loss)]
    fn pseudo_random_from_seed(seed: u64) -> f64 {
        // Simple LCG-style mixing
        let mixed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let normalized = (mixed >> 33) as f64 / (1u64 << 31) as f64;
        normalized.clamp(0.0001, 0.9999)
    }

    /// Calculate the probability of a query result given true value
    /// (useful for privacy auditing)
    #[must_use]
    pub fn probability_density(&self, true_value: f64, noisy_value: f64) -> f64 {
        let diff = (noisy_value - true_value).abs();
        (1.0 / (2.0 * self.scale)) * (-diff / self.scale).exp()
    }

    /// Get the expected error (mean absolute error)
    #[must_use]
    pub const fn expected_error(&self) -> f64 {
        self.scale // For Laplace, E[|noise|] = b
    }

    /// Get the 95% confidence interval width
    #[must_use]
    pub fn confidence_interval_95(&self) -> f64 {
        // For Laplace, 95% CI is approximately ±3b
        3.0 * self.scale
    }
}

/// Analytics data with differential privacy noise applied
///
/// These values have had Laplace noise added and may be negative
/// or larger than the true values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoisyAnalytics {
    /// Play count with noise (may be negative)
    pub play_count: i64,
    /// Levels completed with noise
    pub levels_completed: i64,
    /// Play time in seconds with noise
    pub play_time_seconds: i64,
    /// Epsilon used for noise generation
    pub epsilon: f64,
    /// Sensitivity used for noise generation
    pub sensitivity: f64,
}

impl NoisyAnalytics {
    /// Check if values are plausible (for sanity checking)
    #[must_use]
    pub const fn is_plausible(&self) -> bool {
        // Negative play counts are possible with high noise but unlikely
        // This is a soft check for debugging
        self.play_count >= -100 && self.levels_completed >= -100
    }

    /// Get the expected error margin for these noisy values
    #[must_use]
    pub fn error_margin(&self) -> f64 {
        self.sensitivity / self.epsilon
    }

    /// Clamp values to non-negative for display purposes
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn clamped(&self) -> ClampedNoisyAnalytics {
        ClampedNoisyAnalytics {
            play_count: self.play_count.max(0) as u64,
            levels_completed: self.levels_completed.max(0) as u64,
            play_time_seconds: self.play_time_seconds.max(0) as u64,
            epsilon: self.epsilon,
        }
    }
}

/// Noisy analytics clamped to non-negative values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClampedNoisyAnalytics {
    /// Play count (clamped to 0 minimum)
    pub play_count: u64,
    /// Levels completed (clamped to 0 minimum)
    pub levels_completed: u64,
    /// Play time in seconds (clamped to 0 minimum)
    pub play_time_seconds: u64,
    /// Epsilon used (for transparency)
    pub epsilon: f64,
}

/// Aggregate retention metrics with differential privacy
///
/// Used for population-level Day-7 retention tracking per spec Section 14.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionMetrics {
    /// Day 1 retention rate (with noise)
    pub day1_retention: f64,
    /// Day 7 retention rate (with noise)
    pub day7_retention: f64,
    /// Sample size (with noise)
    pub sample_size: i64,
    /// Privacy parameters used
    pub epsilon: f64,
}

impl RetentionMetrics {
    /// Calculate retention metrics from cohort data with differential privacy
    ///
    /// # Arguments
    /// * `total_users` - Total users in cohort
    /// * `day1_active` - Users active on day 1
    /// * `day7_active` - Users active on day 7
    /// * `config` - Differential privacy configuration
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn from_cohort(
        total_users: u64,
        day1_active: u64,
        day7_active: u64,
        config: &DifferentialPrivacyConfig,
    ) -> Self {
        let noise_gen = DifferentialPrivacy::new(config.epsilon, config.sensitivity);

        // Add noise to counts
        let noisy_total = noise_gen.add_laplace_noise_u64(total_users).max(1) as f64;
        let noisy_day1 = noise_gen.add_laplace_noise_u64(day1_active) as f64;
        let noisy_day7 = noise_gen.add_laplace_noise_u64(day7_active) as f64;

        // Calculate noisy retention rates (clamped to 0-1)
        let day1_retention = (noisy_day1 / noisy_total).clamp(0.0, 1.0);
        let day7_retention = (noisy_day7 / noisy_total).clamp(0.0, 1.0);

        Self {
            day1_retention,
            day7_retention,
            sample_size: noisy_total as i64,
            epsilon: config.epsilon,
        }
    }
}

/// Privacy validator for YAML game definitions
#[derive(Debug, Clone, Default)]
pub struct PrivacyValidator {
    /// Whether to enforce strict COPPA compliance
    pub strict_mode: bool,
}

impl PrivacyValidator {
    /// Create a new validator
    #[must_use]
    pub const fn new() -> Self {
        Self { strict_mode: true }
    }

    /// Create a validator with strict mode disabled
    #[must_use]
    pub const fn permissive() -> Self {
        Self { strict_mode: false }
    }

    /// Validate a game YAML for privacy compliance
    #[must_use]
    pub fn validate_yaml(&self, yaml: &str) -> PrivacyValidationResult {
        let mut issues = Vec::new();

        // Check for PII-collecting keywords
        let pii_keywords = [
            "email",
            "name",
            "address",
            "phone",
            "birthday",
            "age",
            "location",
            "gps",
            "camera",
            "microphone",
            "contacts",
            "photo",
        ];

        let yaml_lower = yaml.to_lowercase();
        for keyword in &pii_keywords {
            if yaml_lower.contains(keyword) {
                issues.push(PrivacyIssue::PiiCollectionNotAllowed);
                break;
            }
        }

        // Check for tracking keywords
        let tracking_keywords = ["track", "analytics", "telemetry", "beacon"];
        for keyword in &tracking_keywords {
            if yaml_lower.contains(keyword) && self.strict_mode {
                issues.push(PrivacyIssue::SessionTrackingRequiresConsent);
                break;
            }
        }

        PrivacyValidationResult {
            compliant: issues.iter().all(|i| !i.is_blocking()),
            issues,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 9.2
    // ========================================================================

    mod compliance_level_tests {
        use super::*;

        #[test]
        fn test_default_is_full_compliance() {
            let level = ComplianceLevel::default();
            assert_eq!(level, ComplianceLevel::Full);
        }

        #[test]
        fn test_full_compliance_no_analytics() {
            let level = ComplianceLevel::Full;
            assert!(!level.allows_analytics());
            assert!(!level.allows_session_tracking());
        }

        #[test]
        fn test_minimal_analytics_allowed() {
            let level = ComplianceLevel::MinimalAnalytics;
            assert!(level.allows_analytics());
            assert!(!level.allows_session_tracking());
        }

        #[test]
        fn test_parental_consent_full_features() {
            let level = ComplianceLevel::ParentalConsent;
            assert!(level.allows_analytics());
            assert!(level.allows_session_tracking());
        }

        #[test]
        fn test_descriptions() {
            assert!(!ComplianceLevel::Full.description().is_empty());
            assert!(!ComplianceLevel::MinimalAnalytics.description().is_empty());
            assert!(!ComplianceLevel::ParentalConsent.description().is_empty());
        }
    }

    mod privacy_config_tests {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = PrivacyConfig::default();
            assert_eq!(config.compliance(), ComplianceLevel::Full);
            assert!(config.show_privacy_notice);
            assert!(config.data_export_enabled);
            assert!(config.data_deletion_enabled);
        }

        #[test]
        fn test_config_validates_compliant() {
            let config = PrivacyConfig::default();
            let result = config.validate();
            assert!(result.is_compliant());
        }

        #[test]
        fn test_config_requires_data_export() {
            let config = PrivacyConfig {
                data_export_enabled: false,
                ..Default::default()
            };
            let result = config.validate();
            assert!(!result.is_compliant());
            assert!(result.issues.contains(&PrivacyIssue::DataExportRequired));
        }

        #[test]
        fn test_config_requires_data_deletion() {
            let config = PrivacyConfig {
                data_deletion_enabled: false,
                ..Default::default()
            };
            let result = config.validate();
            assert!(!result.is_compliant());
            assert!(result.issues.contains(&PrivacyIssue::DataDeletionRequired));
        }

        #[test]
        fn test_config_recommends_privacy_notice() {
            let config = PrivacyConfig {
                show_privacy_notice: false,
                ..Default::default()
            };
            let result = config.validate();
            // Should still be compliant (not blocking)
            assert!(result.is_compliant());
            assert!(result
                .issues
                .contains(&PrivacyIssue::PrivacyNoticeRecommended));
        }
    }

    mod privacy_issue_tests {
        use super::*;

        #[test]
        fn test_blocking_issues() {
            assert!(PrivacyIssue::DataExportRequired.is_blocking());
            assert!(PrivacyIssue::DataDeletionRequired.is_blocking());
            assert!(PrivacyIssue::PiiCollectionNotAllowed.is_blocking());
            assert!(!PrivacyIssue::PrivacyNoticeRecommended.is_blocking());
            assert!(!PrivacyIssue::SessionTrackingRequiresConsent.is_blocking());
        }

        #[test]
        fn test_issue_descriptions() {
            for issue in [
                PrivacyIssue::DataExportRequired,
                PrivacyIssue::DataDeletionRequired,
                PrivacyIssue::PrivacyNoticeRecommended,
                PrivacyIssue::SessionTrackingRequiresConsent,
                PrivacyIssue::PiiCollectionNotAllowed,
            ] {
                assert!(!issue.description().is_empty());
            }
        }
    }

    mod anonymous_event_tests {
        use super::*;

        #[test]
        fn test_game_start_event() {
            let event = AnonymousEvent::game_start(Some("test-game".to_string()));
            assert_eq!(event.event_type, EventType::GameStart);
            assert_eq!(event.game_id, Some("test-game".to_string()));
        }

        #[test]
        fn test_game_complete_event() {
            let event = AnonymousEvent::game_complete(None);
            assert_eq!(event.event_type, EventType::GameComplete);
            assert!(event.game_id.is_none());
        }

        #[test]
        fn test_level_complete_event() {
            let event = AnonymousEvent::level_complete(Some("game".to_string()));
            assert_eq!(event.event_type, EventType::LevelComplete);
        }

        #[test]
        fn test_error_event() {
            let event = AnonymousEvent::error(None);
            assert_eq!(event.event_type, EventType::Error);
        }
    }

    mod local_analytics_tests {
        use super::*;

        #[test]
        fn test_new_analytics() {
            let analytics = LocalAnalytics::new();
            assert_eq!(analytics.play_count, 0);
            assert_eq!(analytics.levels_completed, 0);
            assert_eq!(analytics.play_time_seconds, 0);
        }

        #[test]
        fn test_record_play() {
            let mut analytics = LocalAnalytics::new();
            analytics.record_play(60);
            assert_eq!(analytics.play_count, 1);
            assert_eq!(analytics.play_time_seconds, 60);

            analytics.record_play(120);
            assert_eq!(analytics.play_count, 2);
            assert_eq!(analytics.play_time_seconds, 180);
        }

        #[test]
        fn test_record_level_complete() {
            let mut analytics = LocalAnalytics::new();
            analytics.record_level_complete();
            assert_eq!(analytics.levels_completed, 1);
        }

        #[test]
        fn test_export() {
            let analytics = LocalAnalytics {
                play_count: 5,
                levels_completed: 10,
                play_time_seconds: 3600,
                last_play_date: Some("2024-01-01".to_string()),
            };
            let json = analytics.export().unwrap();
            assert!(json.contains("\"play_count\": 5"));
            assert!(json.contains("\"levels_completed\": 10"));
        }

        #[test]
        fn test_clear() {
            let mut analytics = LocalAnalytics {
                play_count: 100,
                levels_completed: 50,
                play_time_seconds: 10000,
                last_play_date: Some("2024-01-01".to_string()),
            };
            analytics.clear();
            assert_eq!(analytics.play_count, 0);
            assert_eq!(analytics.levels_completed, 0);
            assert_eq!(analytics.play_time_seconds, 0);
            assert!(analytics.last_play_date.is_none());
        }

        #[test]
        fn test_saturating_add() {
            let mut analytics = LocalAnalytics {
                play_count: u64::MAX - 1,
                ..Default::default()
            };
            analytics.record_play(0);
            assert_eq!(analytics.play_count, u64::MAX);
            analytics.record_play(0); // Should not overflow
            assert_eq!(analytics.play_count, u64::MAX);
        }
    }

    mod privacy_validator_tests {
        use super::*;

        #[test]
        fn test_clean_yaml_passes() {
            let validator = PrivacyValidator::new();
            let yaml = r"
character: bunny
collect: stars
background: sky
";
            let result = validator.validate_yaml(yaml);
            assert!(result.is_compliant());
        }

        #[test]
        fn test_pii_yaml_fails() {
            let validator = PrivacyValidator::new();
            let yaml = r"
character: bunny
collect_email: true
";
            let result = validator.validate_yaml(yaml);
            assert!(!result.is_compliant());
            assert!(result
                .issues
                .contains(&PrivacyIssue::PiiCollectionNotAllowed));
        }

        #[test]
        fn test_tracking_yaml_warns() {
            let validator = PrivacyValidator::new();
            let yaml = r"
character: bunny
analytics: enabled
";
            let result = validator.validate_yaml(yaml);
            // Analytics warning is not blocking
            assert!(result.is_compliant());
            assert!(result
                .issues
                .contains(&PrivacyIssue::SessionTrackingRequiresConsent));
        }

        #[test]
        fn test_permissive_mode() {
            let validator = PrivacyValidator::permissive();
            let yaml = r"
character: bunny
analytics: enabled
";
            let result = validator.validate_yaml(yaml);
            assert!(result.is_compliant());
            // Should not contain tracking warning in permissive mode
            assert!(!result
                .issues
                .contains(&PrivacyIssue::SessionTrackingRequiresConsent));
        }

        #[test]
        fn test_pii_keywords() {
            let validator = PrivacyValidator::new();

            for keyword in ["email", "phone", "birthday", "camera", "gps"] {
                let yaml = format!("collect_{keyword}: true");
                let result = validator.validate_yaml(&yaml);
                assert!(!result.is_compliant(), "Should fail for keyword: {keyword}");
            }
        }
    }

    mod serde_tests {
        use super::*;

        #[test]
        fn test_compliance_level_serde() {
            let config = PrivacyConfig::default();
            let json = serde_json::to_string(&config).unwrap();
            assert!(json.contains("\"compliance_level\":\"full\""));
        }

        #[test]
        fn test_event_type_serde() {
            let event = AnonymousEvent::game_start(None);
            let json = serde_json::to_string(&event).unwrap();
            assert!(json.contains("\"event_type\":\"game_start\""));
        }
    }

    // ========================================================================
    // DIFFERENTIAL PRIVACY TESTS
    // Per spec Section 14.1: Privacy-Preserving Learning Analytics
    // ========================================================================

    mod differential_privacy_config_tests {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = DifferentialPrivacyConfig::default();
            assert!((config.epsilon - 1.0).abs() < f64::EPSILON);
            assert!((config.sensitivity - 1.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_strong_privacy() {
            let config = DifferentialPrivacyConfig::strong_privacy();
            assert!((config.epsilon - 0.1).abs() < f64::EPSILON);
        }

        #[test]
        fn test_moderate_privacy() {
            let config = DifferentialPrivacyConfig::moderate_privacy();
            assert!((config.epsilon - 1.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_weak_privacy() {
            let config = DifferentialPrivacyConfig::weak_privacy();
            assert!((config.epsilon - 10.0).abs() < f64::EPSILON);
        }
    }

    mod differential_privacy_tests {
        use super::*;

        #[test]
        fn test_scale_calculation() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);
            assert!((dp.scale() - 1.0).abs() < f64::EPSILON);

            let dp2 = DifferentialPrivacy::new(0.1, 1.0);
            assert!((dp2.scale() - 10.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_laplace_sample_symmetry() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);

            // u = 0.5 should give 0 noise
            let noise_at_half = dp.sample_laplace(0.5);
            assert!(noise_at_half.abs() < 0.1, "Noise at u=0.5 should be near 0");

            // u < 0.5 and u > 0.5 should give opposite signs
            let noise_low = dp.sample_laplace(0.3);
            let noise_high = dp.sample_laplace(0.7);
            assert!(noise_low * noise_high < 0.0, "Noise should be symmetric");
        }

        #[test]
        fn test_expected_error() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);
            assert!((dp.expected_error() - 1.0).abs() < f64::EPSILON);

            let dp2 = DifferentialPrivacy::new(0.5, 1.0);
            assert!((dp2.expected_error() - 2.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_confidence_interval() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);
            let ci = dp.confidence_interval_95();
            assert!((ci - 3.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_probability_density() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);

            // Density at true value should be maximum
            let density_at_true = dp.probability_density(100.0, 100.0);
            let density_away = dp.probability_density(100.0, 101.0);
            assert!(density_at_true > density_away);
        }

        #[test]
        fn test_add_noise_deterministic() {
            let dp = DifferentialPrivacy::new(1.0, 1.0);

            // Same input should give same output (deterministic for testing)
            let result1 = dp.add_laplace_noise_u64(100);
            let result2 = dp.add_laplace_noise_u64(100);
            assert_eq!(result1, result2);
        }

        #[test]
        fn test_noise_magnitude_varies_with_epsilon() {
            // Lower epsilon = more noise
            let dp_strong = DifferentialPrivacy::new(0.1, 1.0);
            let dp_weak = DifferentialPrivacy::new(10.0, 1.0);

            assert!(dp_strong.scale() > dp_weak.scale());
        }
    }

    mod noisy_analytics_tests {
        use super::*;

        #[test]
        fn test_analytics_with_differential_privacy() {
            let analytics = LocalAnalytics {
                play_count: 100,
                levels_completed: 50,
                play_time_seconds: 3600,
                last_play_date: None,
            };

            let config = DifferentialPrivacyConfig::default();
            let noisy = analytics.with_differential_privacy(&config);

            // Noisy values should be different but plausible
            assert!(noisy.is_plausible());
        }

        #[test]
        fn test_noisy_analytics_clamped() {
            let noisy = NoisyAnalytics {
                play_count: -5,
                levels_completed: 10,
                play_time_seconds: -100,
                epsilon: 1.0,
                sensitivity: 1.0,
            };

            let clamped = noisy.clamped();
            assert_eq!(clamped.play_count, 0);
            assert_eq!(clamped.levels_completed, 10);
            assert_eq!(clamped.play_time_seconds, 0);
        }

        #[test]
        fn test_error_margin() {
            let noisy = NoisyAnalytics {
                play_count: 100,
                levels_completed: 50,
                play_time_seconds: 3600,
                epsilon: 0.5,
                sensitivity: 1.0,
            };

            let margin = noisy.error_margin();
            assert!((margin - 2.0).abs() < f64::EPSILON);
        }
    }

    mod retention_metrics_tests {
        use super::*;

        #[test]
        fn test_retention_metrics_calculation() {
            let config = DifferentialPrivacyConfig::weak_privacy(); // Less noise for predictable test

            let metrics = RetentionMetrics::from_cohort(
                1000, // total users
                800,  // day 1 active
                500,  // day 7 active
                &config,
            );

            // With weak privacy, values should be close to true values
            // Day 1 retention ~80%, Day 7 ~50%
            assert!(metrics.day1_retention >= 0.0 && metrics.day1_retention <= 1.0);
            assert!(metrics.day7_retention >= 0.0 && metrics.day7_retention <= 1.0);
            assert!(metrics.sample_size > 0);
        }

        #[test]
        fn test_retention_rates_clamped() {
            let config = DifferentialPrivacyConfig::default();

            let metrics = RetentionMetrics::from_cohort(100, 50, 25, &config);

            // Rates should always be between 0 and 1
            assert!(metrics.day1_retention >= 0.0 && metrics.day1_retention <= 1.0);
            assert!(metrics.day7_retention >= 0.0 && metrics.day7_retention <= 1.0);
        }
    }

    mod differential_privacy_integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow() {
            // Simulate a user's local analytics
            let mut analytics = LocalAnalytics::new();
            analytics.record_play(60);
            analytics.record_play(120);
            analytics.record_level_complete();
            analytics.record_level_complete();
            analytics.record_level_complete();

            assert_eq!(analytics.play_count, 2);
            assert_eq!(analytics.levels_completed, 3);
            assert_eq!(analytics.play_time_seconds, 180);

            // Apply differential privacy before sending to server
            let config = DifferentialPrivacyConfig::moderate_privacy();
            let noisy = analytics.with_differential_privacy(&config);

            // Noisy values are suitable for aggregate analysis
            assert!(noisy.is_plausible());

            // Can be clamped for display
            let clamped = noisy.clamped();
            assert!(clamped.play_count <= 100); // Reasonable range
        }

        #[test]
        fn test_privacy_preserving_retention_tracking() {
            // Per spec Section 14.1: Validate "Day-7 Retention" (H2)
            // without violating COPPA/Privacy
            let config = DifferentialPrivacyConfig::moderate_privacy();

            // Simulate cohort data from multiple users
            let metrics = RetentionMetrics::from_cohort(
                500, // 500 users in cohort
                400, // 400 returned day 1 (80%)
                200, // 200 returned day 7 (40%)
                &config,
            );

            // Can compute retention without knowing individual users
            println!("Day 1 retention: {:.1}%", metrics.day1_retention * 100.0);
            println!("Day 7 retention: {:.1}%", metrics.day7_retention * 100.0);

            // Privacy parameter is recorded for transparency
            assert!((metrics.epsilon - 1.0).abs() < f64::EPSILON);
        }
    }
}
