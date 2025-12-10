//! Game sharing and export system.
//!
//! Per spec Section 10.3: Sharing and Collaboration - enables kids to share their games.
//!
//! Bundle format (.jgb - Jugar Game Bundle):
//! - YAML game definition
//! - Referenced assets (sprites, sounds)
//! - Metadata (creator nickname, version)
//! - Integrity checksum

use crate::error::YamlError;
use crate::privacy::PrivacyValidator;
use base64::Engine;
use core::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Maximum bundle size (1 MB for sharing)
pub const MAX_BUNDLE_SIZE: usize = 1024 * 1024;

/// Bundle file magic number
pub const BUNDLE_MAGIC: &[u8; 4] = b"JGB1";

/// A shareable game bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBundle {
    /// Bundle version
    pub version: u8,
    /// Game definition (YAML)
    pub game_yaml: String,
    /// Bundle metadata
    pub metadata: BundleMetadata,
    /// Embedded assets (base64 encoded)
    pub assets: Vec<EmbeddedAsset>,
    /// CRC32 checksum of contents
    pub checksum: u32,
}

/// Metadata about the bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Creator's nickname (optional, no real names for COPPA)
    pub creator_nickname: Option<String>,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Bundle title
    pub title: String,
    /// Short description (max 140 chars)
    pub description: String,
    /// Schema level (1, 2, or 3)
    pub schema_level: u8,
    /// Tags for discovery
    pub tags: Vec<String>,
}

impl Default for BundleMetadata {
    fn default() -> Self {
        Self {
            creator_nickname: None,
            created_at: 0,
            title: String::new(),
            description: String::new(),
            schema_level: 1,
            tags: Vec::new(),
        }
    }
}

impl BundleMetadata {
    /// Create new metadata with required fields
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set description (truncated to 140 chars)
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        let desc = desc.into();
        self.description = if desc.len() > 140 {
            desc[..140].to_string()
        } else {
            desc
        };
        self
    }

    /// Set creator nickname
    #[must_use]
    pub fn with_creator(mut self, nickname: impl Into<String>) -> Self {
        self.creator_nickname = Some(nickname.into());
        self
    }

    /// Add a tag
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    /// Validate metadata for kid-safety
    #[must_use]
    pub fn validate(&self) -> MetadataValidationResult {
        let mut issues = Vec::new();

        // Title required
        if self.title.trim().is_empty() {
            issues.push(MetadataIssue::MissingTitle);
        }

        // Title length check
        if self.title.len() > 50 {
            issues.push(MetadataIssue::TitleTooLong);
        }

        // Check nickname for PII patterns
        if let Some(ref nickname) = self.creator_nickname {
            if looks_like_real_name(nickname) {
                issues.push(MetadataIssue::NicknameLooksLikeName);
            }
            if nickname.contains('@') {
                issues.push(MetadataIssue::NicknameContainsEmail);
            }
        }

        // Check description for inappropriate content
        let privacy_validator = PrivacyValidator::new();
        if !self.description.is_empty() {
            let result = privacy_validator.validate_yaml(&self.description);
            if !result.is_compliant() {
                issues.push(MetadataIssue::DescriptionContainsPii);
            }
        }

        MetadataValidationResult {
            valid: issues.is_empty(),
            issues,
        }
    }
}

/// Result of metadata validation
#[derive(Debug, Clone)]
pub struct MetadataValidationResult {
    /// Whether metadata is valid
    pub valid: bool,
    /// Issues found
    pub issues: Vec<MetadataIssue>,
}

/// Metadata validation issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataIssue {
    /// Title is missing
    MissingTitle,
    /// Title is too long (>50 chars)
    TitleTooLong,
    /// Nickname looks like a real name
    NicknameLooksLikeName,
    /// Nickname contains email address
    NicknameContainsEmail,
    /// Description contains PII
    DescriptionContainsPii,
}

impl MetadataIssue {
    /// Get kid-friendly message
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::MissingTitle => "Your game needs a title!",
            Self::TitleTooLong => "That title is too long - try making it shorter",
            Self::NicknameLooksLikeName => {
                "Use a fun nickname instead of your real name (for safety!)"
            }
            Self::NicknameContainsEmail => "Don't put your email in the nickname (for safety!)",
            Self::DescriptionContainsPii => "Don't put personal info in the description",
        }
    }
}

/// An embedded asset in the bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedAsset {
    /// Asset name/path
    pub name: String,
    /// Asset type
    pub asset_type: AssetType,
    /// Base64 encoded data
    pub data_base64: String,
    /// Original file size in bytes
    pub original_size: usize,
}

/// Types of embeddable assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// Sprite image (PNG)
    Sprite,
    /// Sound effect (WAV/OGG)
    Sound,
    /// Music track (OGG)
    Music,
    /// AI model (.apr)
    AiModel,
}

impl GameBundle {
    /// Create a new bundle from YAML
    ///
    /// # Errors
    ///
    /// Returns error if YAML is invalid or unsafe
    pub fn from_yaml(
        yaml: impl Into<String>,
        metadata: BundleMetadata,
    ) -> Result<Self, BundleError> {
        let game_yaml = yaml.into();

        // Validate YAML
        let _: serde_yaml::Value =
            serde_yaml::from_str(&game_yaml).map_err(|e| BundleError::InvalidYaml {
                message: e.to_string(),
            })?;

        // Check privacy compliance
        let privacy_validator = PrivacyValidator::new();
        let privacy_result = privacy_validator.validate_yaml(&game_yaml);
        if !privacy_result.is_compliant() {
            return Err(BundleError::PrivacyViolation {
                message: "Game contains potentially unsafe content".to_string(),
            });
        }

        // Validate metadata
        let meta_result = metadata.validate();
        if !meta_result.valid {
            let msg = meta_result
                .issues
                .first()
                .map_or("Invalid metadata", MetadataIssue::message);
            return Err(BundleError::InvalidMetadata {
                message: msg.to_string(),
            });
        }

        let mut bundle = Self {
            version: 1,
            game_yaml,
            metadata,
            assets: Vec::new(),
            checksum: 0,
        };

        // Calculate checksum
        bundle.checksum = bundle.calculate_checksum();

        Ok(bundle)
    }

    /// Add an asset to the bundle
    ///
    /// # Errors
    ///
    /// Returns error if bundle would exceed size limit
    pub fn add_asset(&mut self, asset: EmbeddedAsset) -> Result<(), BundleError> {
        // Check size
        if self.estimated_size() + asset.data_base64.len() > MAX_BUNDLE_SIZE {
            return Err(BundleError::BundleTooLarge {
                size: self.estimated_size() + asset.data_base64.len(),
                max: MAX_BUNDLE_SIZE,
            });
        }

        self.assets.push(asset);
        self.checksum = self.calculate_checksum();
        Ok(())
    }

    /// Calculate CRC32 checksum of bundle contents
    #[must_use]
    pub fn calculate_checksum(&self) -> u32 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        self.version.hash(&mut hasher);
        self.game_yaml.hash(&mut hasher);
        self.metadata.title.hash(&mut hasher);

        for asset in &self.assets {
            asset.name.hash(&mut hasher);
            asset.data_base64.hash(&mut hasher);
        }

        #[allow(clippy::cast_possible_truncation)]
        let hash = hasher.finish() as u32;
        hash
    }

    /// Verify bundle integrity
    #[must_use]
    pub fn verify(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }

    /// Estimate bundle size in bytes
    #[must_use]
    pub fn estimated_size(&self) -> usize {
        let mut size = self.game_yaml.len();
        size += self.metadata.title.len();
        size += self.metadata.description.len();

        for asset in &self.assets {
            size += asset.data_base64.len();
            size += asset.name.len();
        }

        size + 100 // overhead for JSON structure
    }

    /// Export bundle to JSON
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_json(&self) -> Result<String, BundleError> {
        serde_json::to_string_pretty(self).map_err(|e| BundleError::SerializationError {
            message: e.to_string(),
        })
    }

    /// Import bundle from JSON
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails or bundle is invalid
    pub fn from_json(json: &str) -> Result<Self, BundleError> {
        let bundle: Self =
            serde_json::from_str(json).map_err(|e| BundleError::DeserializationError {
                message: e.to_string(),
            })?;

        // Verify integrity
        if !bundle.verify() {
            return Err(BundleError::IntegrityError);
        }

        Ok(bundle)
    }

    /// Export bundle to base64 (for sharing via URL/QR code)
    ///
    /// # Errors
    ///
    /// Returns error if encoding fails
    pub fn to_base64(&self) -> Result<String, BundleError> {
        let json = self.to_json()?;
        Ok(base64::engine::general_purpose::URL_SAFE.encode(json))
    }

    /// Import bundle from base64
    ///
    /// # Errors
    ///
    /// Returns error if decoding fails
    pub fn from_base64(encoded: &str) -> Result<Self, BundleError> {
        let json = base64::engine::general_purpose::URL_SAFE
            .decode(encoded)
            .map_err(|e| BundleError::DeserializationError {
                message: e.to_string(),
            })?;

        let json_str = String::from_utf8(json).map_err(|e| BundleError::DeserializationError {
            message: e.to_string(),
        })?;

        Self::from_json(&json_str)
    }

    /// Get the game YAML
    #[must_use]
    pub fn yaml(&self) -> &str {
        &self.game_yaml
    }

    /// Get bundle metadata
    #[must_use]
    pub const fn metadata(&self) -> &BundleMetadata {
        &self.metadata
    }
}

/// Bundle-related errors
#[derive(Debug, Clone)]
pub enum BundleError {
    /// Invalid YAML content
    InvalidYaml {
        /// Error message
        message: String,
    },
    /// Privacy violation detected
    PrivacyViolation {
        /// Error message
        message: String,
    },
    /// Invalid metadata
    InvalidMetadata {
        /// Error message
        message: String,
    },
    /// Bundle exceeds size limit
    BundleTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed
        max: usize,
    },
    /// Serialization error
    SerializationError {
        /// Error message
        message: String,
    },
    /// Deserialization error
    DeserializationError {
        /// Error message
        message: String,
    },
    /// Bundle integrity check failed
    IntegrityError,
}

impl core::fmt::Display for BundleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidYaml { message } => write!(f, "Invalid game: {message}"),
            Self::PrivacyViolation { message } => write!(f, "Safety check failed: {message}"),
            Self::InvalidMetadata { message } => write!(f, "Invalid info: {message}"),
            Self::BundleTooLarge { size, max } => {
                write!(
                    f,
                    "Game too big to share ({} KB, max {} KB)",
                    size / 1024,
                    max / 1024
                )
            }
            Self::SerializationError { message } => write!(f, "Export failed: {message}"),
            Self::DeserializationError { message } => write!(f, "Import failed: {message}"),
            Self::IntegrityError => write!(f, "Game file is corrupted"),
        }
    }
}

impl core::error::Error for BundleError {}

impl From<BundleError> for YamlError {
    fn from(err: BundleError) -> Self {
        Self::ValidationError {
            message: err.to_string(),
        }
    }
}

/// Check if a string looks like a real name (simple heuristic)
fn looks_like_real_name(s: &str) -> bool {
    // Check for common name patterns: "First Last" or "First M. Last"
    let words: Vec<&str> = s.split_whitespace().collect();

    if words.len() >= 2 {
        // If both words are capitalized and look like names
        let first = words[0];
        let last = words[words.len() - 1];

        let first_is_name = first.len() >= 2
            && first.chars().next().is_some_and(char::is_uppercase)
            && first.chars().skip(1).all(char::is_lowercase);

        let last_is_name = last.len() >= 2
            && last.chars().next().is_some_and(char::is_uppercase)
            && last.chars().skip(1).all(char::is_lowercase);

        first_is_name && last_is_name
    } else {
        false
    }
}

/// Share link generator
#[derive(Debug, Clone)]
pub struct ShareLinkGenerator {
    /// Base URL for sharing
    pub base_url: String,
}

impl Default for ShareLinkGenerator {
    fn default() -> Self {
        Self {
            base_url: String::from("https://jugar.dev/play/"),
        }
    }
}

impl ShareLinkGenerator {
    /// Create a share link for a bundle
    ///
    /// # Errors
    ///
    /// Returns error if bundle encoding fails
    pub fn create_link(&self, bundle: &GameBundle) -> Result<String, BundleError> {
        let encoded = bundle.to_base64()?;
        Ok(format!("{}#{encoded}", self.base_url))
    }

    /// Extract bundle from share link
    ///
    /// # Errors
    ///
    /// Returns error if link is invalid
    pub fn extract_bundle(&self, link: &str) -> Result<GameBundle, BundleError> {
        let encoded = link
            .split('#')
            .nth(1)
            .ok_or_else(|| BundleError::DeserializationError {
                message: "Invalid share link format".to_string(),
            })?;

        GameBundle::from_base64(encoded)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 10.3
    // ========================================================================

    mod bundle_creation_tests {
        use super::*;

        #[test]
        fn test_create_bundle() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("My Game");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            assert_eq!(bundle.version, 1);
            assert_eq!(bundle.game_yaml, "character: bunny");
            assert_eq!(bundle.metadata.title, "My Game");
        }

        #[test]
        fn test_bundle_checksum() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            assert!(bundle.verify());
        }

        #[test]
        fn test_bundle_rejects_invalid_yaml() {
            let yaml = "{ invalid: yaml: : }";
            let metadata = BundleMetadata::new("Test");
            let result = GameBundle::from_yaml(yaml, metadata);

            assert!(result.is_err());
        }

        #[test]
        fn test_bundle_rejects_pii() {
            let yaml = "character: bunny\nemail: test@test.com";
            let metadata = BundleMetadata::new("Test");
            let result = GameBundle::from_yaml(yaml, metadata);

            assert!(result.is_err());
        }
    }

    mod metadata_tests {
        use super::*;

        #[test]
        fn test_metadata_creation() {
            let metadata = BundleMetadata::new("Cool Game")
                .with_description("A fun game!")
                .with_creator("CoolKid99")
                .with_tag("fun")
                .with_tag("easy");

            assert_eq!(metadata.title, "Cool Game");
            assert_eq!(metadata.description, "A fun game!");
            assert_eq!(metadata.creator_nickname, Some("CoolKid99".to_string()));
            assert_eq!(metadata.tags.len(), 2);
        }

        #[test]
        fn test_description_truncation() {
            let long_desc = "a".repeat(200);
            let metadata = BundleMetadata::new("Test").with_description(long_desc);

            assert_eq!(metadata.description.len(), 140);
        }

        #[test]
        fn test_metadata_validation_success() {
            let metadata = BundleMetadata::new("My Game");
            let result = metadata.validate();

            assert!(result.valid);
        }

        #[test]
        fn test_metadata_rejects_missing_title() {
            let metadata = BundleMetadata::new("");
            let result = metadata.validate();

            assert!(!result.valid);
            assert!(result.issues.contains(&MetadataIssue::MissingTitle));
        }

        #[test]
        fn test_metadata_rejects_real_names() {
            let metadata = BundleMetadata::new("Test").with_creator("John Smith");
            let result = metadata.validate();

            assert!(result
                .issues
                .contains(&MetadataIssue::NicknameLooksLikeName));
        }

        #[test]
        fn test_metadata_rejects_email_in_nickname() {
            let metadata = BundleMetadata::new("Test").with_creator("kid@email.com");
            let result = metadata.validate();

            assert!(result
                .issues
                .contains(&MetadataIssue::NicknameContainsEmail));
        }
    }

    mod asset_tests {
        use super::*;

        #[test]
        fn test_add_asset() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let mut bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let asset = EmbeddedAsset {
                name: "bunny.png".to_string(),
                asset_type: AssetType::Sprite,
                data_base64: "iVBORw0KGgo=".to_string(),
                original_size: 100,
            };

            bundle.add_asset(asset).unwrap();
            assert_eq!(bundle.assets.len(), 1);
        }

        #[test]
        fn test_asset_size_limit() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let mut bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            // Create a large asset (over 1MB)
            let large_data = "a".repeat(MAX_BUNDLE_SIZE + 1000);
            let asset = EmbeddedAsset {
                name: "huge.png".to_string(),
                asset_type: AssetType::Sprite,
                data_base64: large_data,
                original_size: MAX_BUNDLE_SIZE + 1000,
            };

            let result = bundle.add_asset(asset);
            assert!(result.is_err());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_to_json() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let json = bundle.to_json().unwrap();
            assert!(json.contains("character: bunny"));
            assert!(json.contains("\"title\": \"Test\""));
        }

        #[test]
        fn test_from_json() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let json = bundle.to_json().unwrap();
            let restored = GameBundle::from_json(&json).unwrap();

            assert_eq!(restored.game_yaml, "character: bunny");
            assert_eq!(restored.metadata.title, "Test");
            assert!(restored.verify());
        }

        #[test]
        fn test_base64_roundtrip() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let encoded = bundle.to_base64().unwrap();
            let restored = GameBundle::from_base64(&encoded).unwrap();

            assert_eq!(restored.game_yaml, bundle.game_yaml);
        }

        #[test]
        fn test_integrity_check_fails_on_tamper() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let mut bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            // Tamper with the bundle
            bundle.game_yaml = "character: cat".to_string();

            // Checksum should no longer match
            assert!(!bundle.verify());
        }
    }

    mod share_link_tests {
        use super::*;

        #[test]
        fn test_create_share_link() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let generator = ShareLinkGenerator::default();
            let link = generator.create_link(&bundle).unwrap();

            assert!(link.starts_with("https://jugar.dev/play/#"));
        }

        #[test]
        fn test_extract_bundle_from_link() {
            let yaml = "character: bunny";
            let metadata = BundleMetadata::new("Test");
            let bundle = GameBundle::from_yaml(yaml, metadata).unwrap();

            let generator = ShareLinkGenerator::default();
            let link = generator.create_link(&bundle).unwrap();

            let restored = generator.extract_bundle(&link).unwrap();
            assert_eq!(restored.game_yaml, "character: bunny");
        }
    }

    mod helper_function_tests {
        use super::*;

        #[test]
        fn test_looks_like_real_name() {
            assert!(looks_like_real_name("John Smith"));
            assert!(looks_like_real_name("Jane Doe"));
            assert!(!looks_like_real_name("CoolKid99"));
            assert!(!looks_like_real_name("ALLCAPS NAME"));
            assert!(!looks_like_real_name("single"));
        }
    }
}
