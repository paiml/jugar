//! APR model metadata.
//!
//! Per spec Section 4.1: CBOR-encoded metadata including name, version,
//! author, license, difficulty levels, and schemas.

use crate::error::AprError;
use serde::{Deserialize, Serialize};

/// Metadata for an APR model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AprMetadata {
    /// Model name (3-50 chars, alphanumeric + hyphen)
    pub name: String,

    /// Semantic version
    pub version: semver::Version,

    /// Author name or organization
    pub author: String,

    /// License identifier (e.g., "MIT", "Apache-2.0")
    pub license: String,

    /// Optional description
    #[serde(default)]
    pub description: String,

    /// Number of difficulty levels (1-10 typically)
    #[serde(default)]
    pub difficulty_levels: Option<u8>,

    /// Input schema description
    #[serde(default)]
    pub input_schema: Option<Schema>,

    /// Output schema description
    #[serde(default)]
    pub output_schema: Option<Schema>,

    /// File size in bytes (computed on save)
    #[serde(default)]
    pub file_size: u64,

    /// Creation timestamp (ISO 8601)
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Schema description for model inputs/outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Schema {
    /// Field definitions
    pub fields: Vec<SchemaField>,
}

/// A single field in a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    /// Field type (f32, i32, bool, etc.)
    pub field_type: String,
    /// Optional description
    #[serde(default)]
    pub description: String,
}

/// Builder for `AprMetadata`
#[derive(Debug, Default)]
pub struct AprMetadataBuilder {
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    license: Option<String>,
    description: Option<String>,
    difficulty_levels: Option<u8>,
    input_schema: Option<Schema>,
    output_schema: Option<Schema>,
}

impl AprMetadataBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the model name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the version string (semver)
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the author
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the license
    #[must_use]
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set difficulty levels
    #[must_use]
    pub const fn difficulty_levels(mut self, levels: u8) -> Self {
        self.difficulty_levels = Some(levels);
        self
    }

    /// Set input schema
    #[must_use]
    pub fn input_schema(mut self, schema: Schema) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Set output schema
    #[must_use]
    pub fn output_schema(mut self, schema: Schema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Build the metadata, validating all fields
    ///
    /// # Errors
    ///
    /// Returns error if required fields are missing or invalid
    pub fn build(self) -> Result<AprMetadata, AprError> {
        let name = self.name.ok_or(AprError::MissingField { field: "name" })?;
        let version_str = self
            .version
            .ok_or(AprError::MissingField { field: "version" })?;
        let author = self
            .author
            .ok_or(AprError::MissingField { field: "author" })?;
        let license = self
            .license
            .ok_or(AprError::MissingField { field: "license" })?;

        // Validate name (3-50 chars, alphanumeric + hyphen)
        if name.len() < 3 || name.len() > 50 {
            return Err(AprError::InvalidName { name });
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AprError::InvalidName { name });
        }

        // Parse version
        let version =
            semver::Version::parse(&version_str).map_err(|_| AprError::InvalidVersion {
                version: version_str,
            })?;

        Ok(AprMetadata {
            name,
            version,
            author,
            license,
            description: self.description.unwrap_or_default(),
            difficulty_levels: self.difficulty_levels,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            file_size: 0,
            created_at: Some(chrono::Utc::now()),
        })
    }
}

impl AprMetadata {
    /// Create a new metadata builder
    #[must_use]
    pub fn builder() -> AprMetadataBuilder {
        AprMetadataBuilder::new()
    }

    /// Encode metadata to CBOR
    ///
    /// # Errors
    ///
    /// Returns error if CBOR encoding fails
    pub fn to_cbor(&self) -> Result<Vec<u8>, AprError> {
        let mut buffer = Vec::new();
        ciborium::into_writer(self, &mut buffer)
            .map_err(|e| AprError::CborEncode(e.to_string()))?;
        Ok(buffer)
    }

    /// Decode metadata from CBOR
    ///
    /// # Errors
    ///
    /// Returns error if CBOR decoding fails
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, AprError> {
        ciborium::from_reader(bytes).map_err(|e| AprError::CborDecode(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_all_required() {
        let result = AprMetadata::builder()
            .name("test")
            .version("1.0.0")
            .author("Author")
            .license("MIT")
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_missing_name() {
        let result = AprMetadata::builder()
            .version("1.0.0")
            .author("Author")
            .license("MIT")
            .build();

        assert!(matches!(
            result,
            Err(AprError::MissingField { field: "name" })
        ));
    }

    #[test]
    fn test_name_too_short() {
        let result = AprMetadata::builder()
            .name("ab")
            .version("1.0.0")
            .author("Author")
            .license("MIT")
            .build();

        assert!(matches!(result, Err(AprError::InvalidName { .. })));
    }

    #[test]
    fn test_name_too_long() {
        let long_name = "a".repeat(51);
        let result = AprMetadata::builder()
            .name(long_name)
            .version("1.0.0")
            .author("Author")
            .license("MIT")
            .build();

        assert!(matches!(result, Err(AprError::InvalidName { .. })));
    }

    #[test]
    fn test_name_invalid_chars() {
        let result = AprMetadata::builder()
            .name("test model!") // Space and ! invalid
            .version("1.0.0")
            .author("Author")
            .license("MIT")
            .build();

        assert!(matches!(result, Err(AprError::InvalidName { .. })));
    }

    #[test]
    fn test_invalid_version() {
        let result = AprMetadata::builder()
            .name("test")
            .version("not.a.version")
            .author("Author")
            .license("MIT")
            .build();

        assert!(matches!(result, Err(AprError::InvalidVersion { .. })));
    }

    #[test]
    fn test_cbor_roundtrip() {
        let original = AprMetadata::builder()
            .name("test-model")
            .version("1.2.3")
            .author("Test Author")
            .license("MIT")
            .description("A test description")
            .difficulty_levels(5)
            .build()
            .expect("Should build");

        let encoded = original.to_cbor().expect("Should encode");
        let decoded = AprMetadata::from_cbor(&encoded).expect("Should decode");

        assert_eq!(original.name, decoded.name);
        assert_eq!(original.version, decoded.version);
        assert_eq!(original.author, decoded.author);
        assert_eq!(original.license, decoded.license);
        assert_eq!(original.description, decoded.description);
        assert_eq!(original.difficulty_levels, decoded.difficulty_levels);
    }
}
