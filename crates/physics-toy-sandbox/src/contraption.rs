//! Contraption: A complete physics scene that can be forked and remixed
//!
//! This module implements the core data model for the remix system:
//! - Content-addressable storage (SHA-256 based IDs)
//! - Engine versioning for replay compatibility (Jidoka)
//! - Serialization for sharing

use glam::Vec2;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{
    ContraptionId, Difficulty, MaterialProperties, ObjectType, PhysicsBackend, Result,
    SandboxError, Transform2D, ENGINE_VERSION, MAX_OBJECTS_PER_CONTRAPTION,
};

/// Physics world configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity vector (default: (0, -9.8) for Earth-like)
    pub gravity: Vec2,

    /// Physics substeps per frame (higher = more accurate, slower)
    pub substeps: u32,

    /// Physics backend to use
    pub backend: PhysicsBackend,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, -9.8),
            substeps: 4,
            backend: PhysicsBackend::default(),
        }
    }
}

/// Visual properties for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VisualProperties {
    /// Color as RGB (0-255)
    pub color: [u8; 3],

    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: f32,

    /// Custom sprite asset path (if any)
    pub sprite: Option<String>,
}

impl VisualProperties {
    /// Create with a color
    #[must_use]
    pub const fn with_color(r: u8, g: u8, b: u8) -> Self {
        Self {
            color: [r, g, b],
            opacity: 1.0,
            sprite: None,
        }
    }
}

/// Optional behavior script for advanced contraptions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BehaviorScript {
    /// Script type identifier
    pub script_type: String,

    /// Script parameters as JSON
    pub params: String,
}

/// A serialized entity within a contraption
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// Type of physics object
    pub entity_type: ObjectType,

    /// Position, rotation, scale
    pub transform: Transform2D,

    /// Material properties (optional for triggers)
    pub material: Option<MaterialProperties>,

    /// Visual properties
    pub visual: VisualProperties,

    /// Optional behavior script
    pub behavior: Option<BehaviorScript>,
}

impl SerializedEntity {
    /// Create a new entity with defaults
    #[must_use]
    pub fn new(entity_type: ObjectType, transform: Transform2D) -> Self {
        let material = if entity_type.is_trigger() {
            None
        } else {
            Some(MaterialProperties::default())
        };

        Self {
            entity_type,
            transform,
            material,
            visual: VisualProperties::default(),
            behavior: None,
        }
    }

    /// Set material properties
    #[must_use]
    pub const fn with_material(mut self, material: MaterialProperties) -> Self {
        self.material = Some(material);
        self
    }

    /// Set visual properties
    #[must_use]
    pub fn with_visual(mut self, visual: VisualProperties) -> Self {
        self.visual = visual;
        self
    }
}

/// Metadata for a contraption (human-readable info)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContraptionMetadata {
    /// Display name
    pub name: String,

    /// Author identifier (username or anonymous)
    pub author: String,

    /// Description of what this contraption does
    pub description: String,

    /// Tags for search/filtering
    pub tags: Vec<String>,

    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,

    /// Play count (incremented on each playback)
    pub play_count: u32,

    /// Number of times this has been remixed
    pub remix_count: u32,

    /// Difficulty rating
    pub difficulty: Difficulty,
}

impl Default for ContraptionMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            author: "Anonymous".to_string(),
            description: String::new(),
            tags: Vec::new(),
            created_at: 0,
            play_count: 0,
            remix_count: 0,
            difficulty: Difficulty::default(),
        }
    }
}

impl ContraptionMetadata {
    /// Create with a name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Set author
    #[must_use]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a tag
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// A complete physics scene that can be forked and remixed
///
/// JIDOKA: The `engine_version` field ensures replay compatibility.
/// When loading a contraption, the runtime should warn if the current
/// engine version differs significantly from when it was created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contraption {
    /// Unique identifier (content-addressable)
    pub id: ContraptionId,

    /// JIDOKA: Engine version at creation time
    /// Ensures replayability even after engine updates
    pub engine_version: Version,

    /// Human-readable metadata
    pub metadata: ContraptionMetadata,

    /// All entities in the scene
    pub entities: Vec<SerializedEntity>,

    /// Physics world configuration
    pub physics_config: PhysicsConfig,

    /// Parent contraption (for remix tracking)
    pub forked_from: Option<ContraptionId>,

    /// Deterministic replay seed
    pub initial_seed: u64,
}

impl Contraption {
    /// Create a new empty contraption
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ContraptionId::new(),
            engine_version: Version::parse(ENGINE_VERSION)
                .unwrap_or_else(|_| Version::new(0, 1, 0)),
            metadata: ContraptionMetadata::new(name),
            entities: Vec::new(),
            physics_config: PhysicsConfig::default(),
            forked_from: None,
            initial_seed: 0,
        }
    }

    /// Add an entity to the contraption
    ///
    /// # Errors
    /// Returns error if object limit is exceeded
    pub fn add_entity(&mut self, entity: SerializedEntity) -> Result<()> {
        if self.entities.len() >= MAX_OBJECTS_PER_CONTRAPTION {
            return Err(SandboxError::ObjectLimitExceeded {
                count: self.entities.len() + 1,
                limit: MAX_OBJECTS_PER_CONTRAPTION,
            });
        }
        self.entities.push(entity);
        Ok(())
    }

    /// Add an object with transform (convenience method)
    ///
    /// # Errors
    /// Returns error if object limit is exceeded
    pub fn add_object(&mut self, object_type: ObjectType, transform: Transform2D) -> Result<()> {
        self.add_entity(SerializedEntity::new(object_type, transform))
    }

    /// Fork this contraption (create a remix)
    #[must_use]
    pub fn fork(&self, new_name: impl Into<String>) -> Self {
        let mut forked = self.clone();
        forked.id = ContraptionId::new();
        forked.forked_from = Some(self.id);
        forked.metadata.name = new_name.into();
        forked.metadata.play_count = 0;
        forked.metadata.remix_count = 0;
        forked.metadata.created_at = 0; // Should be set by caller
        forked
    }

    /// Validate the contraption
    ///
    /// # Errors
    /// Returns error if validation fails
    pub fn validate(&self) -> Result<()> {
        if self.entities.len() > MAX_OBJECTS_PER_CONTRAPTION {
            return Err(SandboxError::ObjectLimitExceeded {
                count: self.entities.len(),
                limit: MAX_OBJECTS_PER_CONTRAPTION,
            });
        }
        Ok(())
    }

    /// Serialize to bytes (for storage/sharing)
    ///
    /// # Errors
    /// Returns error if serialization fails
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| SandboxError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    ///
    /// # Errors
    /// Returns error if deserialization fails
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|_| SandboxError::DeserializationError)
    }

    /// Compute content hash for deduplication
    #[must_use]
    pub fn content_hash(&self) -> u32 {
        // Use CRC32 of serialized content (excluding mutable metadata)
        let mut hasher = crc32fast::Hasher::new();

        // Hash entities
        for entity in &self.entities {
            if let Ok(bytes) = bincode::serialize(entity) {
                hasher.update(&bytes);
            }
        }

        // Hash physics config
        if let Ok(bytes) = bincode::serialize(&self.physics_config) {
            hasher.update(&bytes);
        }

        hasher.finalize()
    }

    /// Get object count
    #[must_use]
    pub fn object_count(&self) -> usize {
        self.entities.len()
    }

    /// Check if this is a fork of another contraption
    #[must_use]
    pub const fn is_fork(&self) -> bool {
        self.forked_from.is_some()
    }

    /// Check engine version compatibility
    ///
    /// Returns `true` if the contraption was created with a compatible engine version
    #[must_use]
    pub fn is_version_compatible(&self) -> bool {
        let current = Version::parse(ENGINE_VERSION).unwrap_or_else(|_| Version::new(0, 1, 0));

        // Compatible if major version matches and current >= contraption version
        self.engine_version.major == current.major && current >= self.engine_version
    }
}

/// Builder for creating contraptions fluently
#[derive(Debug, Default)]
pub struct ContraptionBuilder {
    name: String,
    author: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    entities: Vec<SerializedEntity>,
    physics_config: Option<PhysicsConfig>,
    seed: Option<u64>,
}

impl ContraptionBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Set author
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add an object
    #[must_use]
    pub fn with_object(mut self, object_type: ObjectType, transform: Transform2D) -> Self {
        self.entities
            .push(SerializedEntity::new(object_type, transform));
        self
    }

    /// Add an entity
    #[must_use]
    pub fn with_entity(mut self, entity: SerializedEntity) -> Self {
        self.entities.push(entity);
        self
    }

    /// Set physics config
    #[must_use]
    pub const fn with_physics_config(mut self, config: PhysicsConfig) -> Self {
        self.physics_config = Some(config);
        self
    }

    /// Set random seed
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Build the contraption
    ///
    /// # Errors
    /// Returns error if validation fails
    pub fn build(self) -> Result<Contraption> {
        let mut contraption = Contraption::new(self.name);

        if let Some(author) = self.author {
            contraption.metadata.author = author;
        }

        if let Some(desc) = self.description {
            contraption.metadata.description = desc;
        }

        contraption.metadata.tags = self.tags;

        if let Some(config) = self.physics_config {
            contraption.physics_config = config;
        }

        if let Some(seed) = self.seed {
            contraption.initial_seed = seed;
        }

        for entity in self.entities {
            contraption.add_entity(entity)?;
        }

        contraption.validate()?;
        Ok(contraption)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_precision_loss,
    unused_results
)]
mod tests {
    use super::*;

    // =========================================================================
    // EXTREME TDD: Contraption tests from specification
    // =========================================================================

    mod creation_tests {
        use super::*;

        #[test]
        fn test_new_contraption_has_unique_id() {
            let c1 = Contraption::new("Test 1");
            let c2 = Contraption::new("Test 2");
            assert_ne!(c1.id, c2.id);
        }

        #[test]
        fn test_new_contraption_has_engine_version() {
            let c = Contraption::new("Test");
            assert!(!c.engine_version.to_string().is_empty());
        }

        #[test]
        fn test_new_contraption_has_no_parent() {
            let c = Contraption::new("Test");
            assert!(c.forked_from.is_none());
        }

        #[test]
        fn test_new_contraption_is_empty() {
            let c = Contraption::new("Test");
            assert_eq!(c.object_count(), 0);
        }
    }

    mod fork_tests {
        use super::*;

        #[test]
        fn test_fork_creates_independent_copy() {
            let original = ContraptionBuilder::new("Original")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            let forked = original.fork("Forked Copy");

            // IDs should differ
            assert_ne!(original.id, forked.id);
        }

        #[test]
        fn test_fork_references_original() {
            let original = Contraption::new("Original");
            let forked = original.fork("Forked");

            assert_eq!(forked.forked_from, Some(original.id));
        }

        #[test]
        fn test_fork_is_independent() {
            let mut original = Contraption::new("Original");
            original
                .add_object(ObjectType::Ball, Transform2D::default())
                .unwrap();

            let mut forked = original.fork("Forked");
            forked
                .add_object(ObjectType::Ramp, Transform2D::default())
                .unwrap();

            assert_eq!(original.object_count(), 1);
            assert_eq!(forked.object_count(), 2);
        }

        #[test]
        fn test_fork_resets_counts() {
            let mut original = Contraption::new("Original");
            original.metadata.play_count = 100;
            original.metadata.remix_count = 50;

            let forked = original.fork("Forked");

            assert_eq!(forked.metadata.play_count, 0);
            assert_eq!(forked.metadata.remix_count, 0);
        }

        #[test]
        fn test_is_fork() {
            let original = Contraption::new("Original");
            let forked = original.fork("Forked");

            assert!(!original.is_fork());
            assert!(forked.is_fork());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_serialization_roundtrip() {
            let original = ContraptionBuilder::new("Test Scene")
                .with_object(
                    ObjectType::Ball,
                    Transform2D {
                        position: Vec2::new(123.456, 789.012),
                        rotation: 1.23456,
                        scale: Vec2::new(20.0, 20.0),
                    },
                )
                .with_physics_config(PhysicsConfig {
                    gravity: Vec2::new(0.0, -9.8),
                    substeps: 4,
                    backend: PhysicsBackend::WasmSimd,
                })
                .build()
                .unwrap();

            let bytes = original.serialize().unwrap();
            let restored = Contraption::deserialize(&bytes).unwrap();

            assert_eq!(original.id, restored.id);
            assert_eq!(original.entities.len(), restored.entities.len());
        }

        #[test]
        fn test_invalid_data_rejected() {
            let garbage = vec![0xFF, 0xFE, 0x00, 0x01, 0x02, 0x03];
            let result = Contraption::deserialize(&garbage);
            assert!(result.is_err());
        }
    }

    mod validation_tests {
        use super::*;

        #[test]
        fn test_object_limit_enforced() {
            let mut builder = ContraptionBuilder::new("Too Large");

            for i in 0..MAX_OBJECTS_PER_CONTRAPTION {
                builder = builder.with_object(
                    ObjectType::Ball,
                    Transform2D {
                        position: Vec2::new(i as f32, 0.0),
                        ..Transform2D::default()
                    },
                );
            }

            // Should succeed at limit
            let contraption = builder.build().unwrap();
            assert_eq!(contraption.object_count(), MAX_OBJECTS_PER_CONTRAPTION);

            // Should fail when adding one more
            let mut full = contraption;
            let result = full.add_object(ObjectType::Ball, Transform2D::default());
            assert!(result.is_err());

            if let Err(SandboxError::ObjectLimitExceeded { count, limit }) = result {
                assert_eq!(count, MAX_OBJECTS_PER_CONTRAPTION + 1);
                assert_eq!(limit, MAX_OBJECTS_PER_CONTRAPTION);
            } else {
                panic!("Expected ObjectLimitExceeded error");
            }
        }
    }

    mod content_hash_tests {
        use super::*;

        #[test]
        fn test_same_content_same_hash() {
            let scene1 = ContraptionBuilder::new("Scene A")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            let scene2 = ContraptionBuilder::new("Scene A")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            assert_eq!(scene1.content_hash(), scene2.content_hash());
        }

        #[test]
        fn test_different_content_different_hash() {
            let scene1 = ContraptionBuilder::new("Scene A")
                .with_object(ObjectType::Ball, Transform2D::default())
                .build()
                .unwrap();

            let scene2 = ContraptionBuilder::new("Scene B")
                .with_object(ObjectType::Ramp, Transform2D::default())
                .build()
                .unwrap();

            assert_ne!(scene1.content_hash(), scene2.content_hash());
        }
    }

    mod builder_tests {
        use super::*;

        #[test]
        fn test_builder_fluent_api() {
            let contraption = ContraptionBuilder::new("My Machine")
                .author("TestUser")
                .description("A test contraption")
                .tag("test")
                .tag("physics")
                .with_object(ObjectType::Ball, Transform2D::default())
                .with_seed(42)
                .build()
                .unwrap();

            assert_eq!(contraption.metadata.name, "My Machine");
            assert_eq!(contraption.metadata.author, "TestUser");
            assert_eq!(contraption.metadata.description, "A test contraption");
            assert_eq!(contraption.metadata.tags.len(), 2);
            assert_eq!(contraption.initial_seed, 42);
            assert_eq!(contraption.object_count(), 1);
        }
    }

    mod version_compatibility_tests {
        use super::*;

        #[test]
        fn test_current_version_is_compatible() {
            let c = Contraption::new("Test");
            assert!(c.is_version_compatible());
        }
    }
}
