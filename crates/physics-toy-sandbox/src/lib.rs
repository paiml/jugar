//! # Physics Toy Sandbox
//!
//! A remixable physics playground - Rube Goldberg machine builder for Jugar.
//!
//! This crate implements the Physics Toy Sandbox specification with Toyota Way principles:
//!
//! - **Kaizen**: Every remix is continuous improvement
//! - **Poka-Yoke**: Type-safe material properties (`NonZeroU32` for density)
//! - **Jidoka**: Engine versioning ensures replay compatibility
//! - **Mieruka**: Complexity Thermometer provides visual feedback
//! - **Muda Elimination**: No scalar fallback (SIMD support >99%)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    PHYSICS TOY SANDBOX                           │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Contraption ──► RemixGraph ──► Storage                         │
//! │       │                                                          │
//! │       ├── MaterialProperties (Poka-Yoke: NonZeroU32 density)    │
//! │       ├── PhysicsConfig (versioned)                             │
//! │       └── SerializedEntity[]                                    │
//! │                                                                  │
//! │  ComplexityThermometer (Mieruka) ──► UI Feedback                │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod contraption;
pub mod material;
pub mod remix;
pub mod thermometer;

pub use contraption::*;
pub use material::*;
pub use remix::*;
pub use thermometer::*;

/// Content-addressed ID for contraptions (SHA-256 based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContraptionId(pub Uuid);

impl ContraptionId {
    /// Create a new random contraption ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ContraptionId {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for ContraptionId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors that can occur in physics-toy-sandbox
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    /// Contraption exceeds object limit
    #[error("Contraption exceeds object limit: {count} > {limit}")]
    ObjectLimitExceeded {
        /// Current object count
        count: usize,
        /// Maximum allowed
        limit: usize,
    },

    /// Invalid material properties
    #[error("Invalid material: {reason}")]
    InvalidMaterial {
        /// Reason for invalidity
        reason: String,
    },

    /// Serialization error
    #[error("Serialization failed: {0}")]
    SerializationError(String),

    /// Deserialization error (malformed data)
    #[error("Deserialization failed: invalid or corrupt data")]
    DeserializationError,

    /// Engine version mismatch
    #[error("Engine version mismatch: contraption requires {required}, current is {current}")]
    VersionMismatch {
        /// Required engine version
        required: String,
        /// Current engine version
        current: String,
    },

    /// Contraption not found
    #[error("Contraption not found: {0}")]
    NotFound(ContraptionId),
}

/// Result type for sandbox operations
pub type Result<T> = core::result::Result<T, SandboxError>;

/// Maximum objects per contraption (performance budget)
pub const MAX_OBJECTS_PER_CONTRAPTION: usize = 500;

/// Current engine version for compatibility tracking (Jidoka)
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Physics backend selection (Muda: no scalar fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PhysicsBackend {
    /// WebGPU compute shaders (10,000+ bodies)
    #[default]
    WebGpu,
    /// WASM SIMD 128-bit (1,000+ bodies)
    WasmSimd,
}

/// 2D Transform for sandbox objects
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale (uniform or non-uniform)
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

/// Object types available in the sandbox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    /// Dynamic ball - rolls and bounces
    Ball,
    /// Dynamic domino - falls and triggers chain reactions
    Domino,
    /// Static ramp - redirects objects
    Ramp,
    /// Hinged lever - pivots around fulcrum
    Lever,
    /// Constraint pulley - transfers force
    Pulley,
    /// Constraint spring - stores elastic energy
    Spring,
    /// Force field fan - applies directional force
    Fan,
    /// Force field magnet - attracts/repels
    Magnet,
    /// Trigger bucket - detects objects (win condition)
    Bucket,
    /// Trigger sensor - detects proximity
    Sensor,
}

impl ObjectType {
    /// Is this object type dynamic (affected by physics)?
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Ball | Self::Domino)
    }

    /// Is this object type a trigger (detects but doesn't collide)?
    #[must_use]
    pub const fn is_trigger(&self) -> bool {
        matches!(self, Self::Bucket | Self::Sensor)
    }

    /// Is this object type a constraint (connects other objects)?
    #[must_use]
    pub const fn is_constraint(&self) -> bool {
        matches!(self, Self::Pulley | Self::Spring)
    }
}

/// Difficulty rating for contraptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Difficulty {
    /// Simple contraption (< 10 objects)
    Easy,
    /// Moderate complexity (10-50 objects)
    #[default]
    Medium,
    /// Complex contraption (50-200 objects)
    Hard,
    /// Expert level (200+ objects)
    Expert,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // =========================================================================
    // EXTREME TDD: Tests written FIRST per specification
    // =========================================================================

    mod contraption_id_tests {
        use super::*;

        #[test]
        fn test_contraption_id_uniqueness() {
            let id1 = ContraptionId::new();
            let id2 = ContraptionId::new();
            assert_ne!(id1, id2, "Each ID should be unique");
        }

        #[test]
        fn test_contraption_id_display() {
            let id = ContraptionId::new();
            let display = id.to_string();
            assert!(!display.is_empty());
            assert!(display.contains('-'), "UUID format should contain hyphens");
        }

        #[test]
        fn test_contraption_id_from_uuid() {
            let uuid = Uuid::new_v4();
            let id = ContraptionId::from_uuid(uuid);
            assert_eq!(id.0, uuid);
        }
    }

    mod object_type_tests {
        use super::*;

        #[test]
        fn test_ball_is_dynamic() {
            assert!(ObjectType::Ball.is_dynamic());
        }

        #[test]
        fn test_domino_is_dynamic() {
            assert!(ObjectType::Domino.is_dynamic());
        }

        #[test]
        fn test_ramp_is_not_dynamic() {
            assert!(!ObjectType::Ramp.is_dynamic());
        }

        #[test]
        fn test_bucket_is_trigger() {
            assert!(ObjectType::Bucket.is_trigger());
        }

        #[test]
        fn test_sensor_is_trigger() {
            assert!(ObjectType::Sensor.is_trigger());
        }

        #[test]
        fn test_ball_is_not_trigger() {
            assert!(!ObjectType::Ball.is_trigger());
        }

        #[test]
        fn test_spring_is_constraint() {
            assert!(ObjectType::Spring.is_constraint());
        }

        #[test]
        fn test_pulley_is_constraint() {
            assert!(ObjectType::Pulley.is_constraint());
        }

        #[test]
        fn test_ball_is_not_constraint() {
            assert!(!ObjectType::Ball.is_constraint());
        }
    }

    mod transform_tests {
        use super::*;

        #[test]
        fn test_transform_default() {
            let t = Transform2D::default();
            assert_eq!(t.position, Vec2::ZERO);
            assert!((t.rotation - 0.0).abs() < f32::EPSILON);
            assert_eq!(t.scale, Vec2::ONE);
        }

        #[test]
        fn test_transform_serialization() {
            let t = Transform2D {
                position: Vec2::new(100.0, 200.0),
                rotation: 1.5,
                scale: Vec2::new(2.0, 3.0),
            };
            let json = serde_json::to_string(&t).unwrap();
            let restored: Transform2D = serde_json::from_str(&json).unwrap();
            assert_eq!(t, restored);
        }
    }

    mod physics_backend_tests {
        use super::*;

        #[test]
        fn test_default_backend_is_webgpu() {
            assert_eq!(PhysicsBackend::default(), PhysicsBackend::WebGpu);
        }

        #[test]
        fn test_backend_serialization() {
            let backend = PhysicsBackend::WasmSimd;
            let json = serde_json::to_string(&backend).unwrap();
            let restored: PhysicsBackend = serde_json::from_str(&json).unwrap();
            assert_eq!(backend, restored);
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_object_limit_error_display() {
            let err = SandboxError::ObjectLimitExceeded {
                count: 600,
                limit: 500,
            };
            let msg = err.to_string();
            assert!(msg.contains("600"));
            assert!(msg.contains("500"));
        }

        #[test]
        fn test_version_mismatch_error() {
            let err = SandboxError::VersionMismatch {
                required: "1.0.0".to_string(),
                current: "0.1.0".to_string(),
            };
            let msg = err.to_string();
            assert!(msg.contains("1.0.0"));
            assert!(msg.contains("0.1.0"));
        }

        #[test]
        fn test_deserialization_error() {
            let err = SandboxError::DeserializationError;
            let msg = err.to_string();
            assert!(msg.contains("invalid") || msg.contains("corrupt"));
        }
    }

    mod difficulty_tests {
        use super::*;

        #[test]
        fn test_default_difficulty_is_medium() {
            assert_eq!(Difficulty::default(), Difficulty::Medium);
        }
    }
}
