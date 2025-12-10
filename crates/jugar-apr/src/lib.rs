//! Aprender Package Resource (.apr) model format.
//!
//! Per spec Section 4.1: The .apr format is a compact, portable container
//! for trained AI behaviors that can be hot-swapped like trading cards.
//!
//! # File Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    .APR File Structure                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Magic Number: "APNR" (4 bytes)                             │
//! │  Version: u16 (2 bytes)                                      │
//! │  Checksum: CRC32 (4 bytes)                                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Metadata (CBOR encoded):                                    │
//! │    - name, version, author, license                          │
//! │    - difficulty_levels, input_schema, output_schema          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Model Data (compressed):                                    │
//! │    - weights: [f32; N]                                       │
//! │    - biases: [f32; M]                                        │
//! │    - architecture: string                                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod format;
mod metadata;
mod model;

pub use error::AprError;
pub use format::{AprFile, APR_MAGIC, APR_VERSION};
pub use metadata::AprMetadata;
pub use model::{AprModel, ModelArchitecture, ModelData};

/// Maximum allowed model size (1 MB per spec Section 9.1)
pub const MAX_MODEL_SIZE: usize = 1024 * 1024;

/// Minimum model version supported
pub const MIN_SUPPORTED_VERSION: u16 = 1;

/// Current model version
pub const CURRENT_VERSION: u16 = 1;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec requirements
    // ========================================================================

    mod magic_number_tests {
        use super::*;

        #[test]
        fn test_apr_magic_is_apnr() {
            // Per spec Section 4.1: Magic Number: "APNR" (4 bytes)
            assert_eq!(APR_MAGIC, b"APNR");
            assert_eq!(APR_MAGIC.len(), 4);
        }

        #[test]
        fn test_apr_magic_detection() {
            let valid = b"APNRxxxxxx";
            let invalid = b"WRONGxxxxx";

            assert!(AprFile::has_magic(valid));
            assert!(!AprFile::has_magic(invalid));
        }

        #[test]
        fn test_rejects_wrong_magic() {
            let bad_magic = b"BAAD\x00\x01\x00\x00\x00\x00";
            let result = AprFile::from_bytes(bad_magic);
            assert!(matches!(result, Err(AprError::InvalidMagic { .. })));
        }
    }

    mod version_tests {
        use super::*;

        #[test]
        fn test_apr_version_is_u16() {
            // Per spec Section 4.1: Version: u16 (2 bytes)
            assert_eq!(APR_VERSION, 1_u16);
        }

        #[test]
        fn test_current_version_supported() {
            // Use runtime check to validate version constants
            let current = CURRENT_VERSION;
            let min = MIN_SUPPORTED_VERSION;
            assert!(current >= min);
        }

        #[test]
        fn test_rejects_unsupported_version() {
            // Version 0 should be rejected
            let mut bytes = Vec::new();
            bytes.extend_from_slice(APR_MAGIC);
            bytes.extend_from_slice(&0_u16.to_le_bytes()); // Version 0
            bytes.extend_from_slice(&0_u32.to_le_bytes()); // Checksum placeholder

            let result = AprFile::from_bytes(&bytes);
            assert!(matches!(result, Err(AprError::UnsupportedVersion { .. })));
        }
    }

    mod checksum_tests {
        use super::*;

        #[test]
        fn test_checksum_is_crc32() {
            // Per spec Section 4.1: Checksum: CRC32 (4 bytes)
            let data = b"test data";
            let checksum = crc32fast::hash(data);
            assert_eq!(core::mem::size_of_val(&checksum), 4);
        }

        #[test]
        fn test_checksum_verification() {
            // Create a minimal valid APR file
            let model = AprModel::new_test_model();
            let bytes = model.to_bytes().expect("Should serialize");

            // Verify it can be loaded back
            let loaded = AprFile::from_bytes(&bytes).expect("Should load");
            assert_eq!(loaded.model.metadata.name, model.metadata.name);
        }

        #[test]
        fn test_rejects_corrupted_checksum() {
            let model = AprModel::new_test_model();
            let mut bytes = model.to_bytes().expect("Should serialize");

            // Corrupt the checksum (bytes 6-9)
            if bytes.len() > 9 {
                bytes[6] ^= 0xFF;
            }

            let result = AprFile::from_bytes(&bytes);
            assert!(matches!(result, Err(AprError::ChecksumMismatch { .. })));
        }
    }

    mod metadata_tests {
        use super::*;

        #[test]
        fn test_metadata_has_required_fields() {
            let metadata = AprMetadata::builder()
                .name("test-model")
                .version("1.0.0")
                .author("Test Author")
                .license("MIT")
                .build()
                .expect("Should build metadata");

            assert_eq!(metadata.name, "test-model");
            assert_eq!(metadata.version.to_string(), "1.0.0");
            assert_eq!(metadata.author, "Test Author");
            assert_eq!(metadata.license, "MIT");
        }

        #[test]
        fn test_metadata_optional_difficulty_levels() {
            let metadata = AprMetadata::builder()
                .name("pong-ai")
                .version("1.0.0")
                .author("PAIML")
                .license("MIT")
                .difficulty_levels(10)
                .build()
                .expect("Should build");

            assert_eq!(metadata.difficulty_levels, Some(10));
        }

        #[test]
        fn test_metadata_validates_name_length() {
            // Per spec Section 3.1: 3-20 chars for game names
            let result = AprMetadata::builder()
                .name("ab") // Too short
                .version("1.0.0")
                .author("Test")
                .license("MIT")
                .build();

            assert!(result.is_err());
        }

        #[test]
        fn test_metadata_cbor_roundtrip() {
            let original = AprMetadata::builder()
                .name("test-model")
                .version("1.0.0")
                .author("Test")
                .license("MIT")
                .description("A test model")
                .build()
                .expect("Should build");

            let encoded = original.to_cbor().expect("Should encode");
            let decoded = AprMetadata::from_cbor(&encoded).expect("Should decode");

            assert_eq!(original.name, decoded.name);
            assert_eq!(original.description, decoded.description);
        }
    }

    mod model_data_tests {
        use super::*;

        #[test]
        fn test_model_data_weights_and_biases() {
            let data = ModelData {
                weights: vec![0.5, -0.3, 0.8, 0.1],
                biases: vec![0.0, 0.1],
                architecture: ModelArchitecture::Mlp {
                    layers: vec![2, 4, 1],
                },
            };

            assert_eq!(data.weights.len(), 4);
            assert_eq!(data.biases.len(), 2);
        }

        #[test]
        fn test_model_architecture_mlp() {
            // Per spec: "mlp-2-16-1"
            let arch = ModelArchitecture::Mlp {
                layers: vec![2, 16, 1],
            };

            assert_eq!(arch.to_string(), "mlp-2-16-1");
        }

        #[test]
        fn test_model_data_compression() {
            let data = ModelData {
                weights: vec![0.5; 1000], // Large weight array
                biases: vec![0.0; 100],
                architecture: ModelArchitecture::Mlp {
                    layers: vec![10, 100, 10],
                },
            };

            let compressed = data.compress().expect("Should compress");
            let decompressed = ModelData::decompress(&compressed).expect("Should decompress");

            assert_eq!(data.weights, decompressed.weights);
            assert_eq!(data.biases, decompressed.biases);
        }
    }

    mod size_limit_tests {
        use super::*;

        #[test]
        fn test_max_model_size_is_1mb() {
            // Per spec Section 9.1: max_model_size: 1 MB
            assert_eq!(MAX_MODEL_SIZE, 1024 * 1024);
        }

        #[test]
        #[allow(clippy::cast_precision_loss)]
        fn test_rejects_oversized_model() {
            // Create a model that would exceed 1MB even after compression
            // Use varied values to prevent good compression
            let huge_data = ModelData {
                weights: (0..MAX_MODEL_SIZE)
                    .map(|i| (i as f32) * 0.000_001)
                    .collect(),
                biases: vec![0.0],
                architecture: ModelArchitecture::Mlp { layers: vec![1] },
            };

            let model = AprModel {
                metadata: AprMetadata::builder()
                    .name("too-big")
                    .version("1.0.0")
                    .author("Test")
                    .license("MIT")
                    .build()
                    .expect("metadata"),
                data: huge_data,
            };

            let result = model.to_bytes();
            assert!(matches!(result, Err(AprError::ModelTooLarge { .. })));
        }
    }

    mod roundtrip_tests {
        use super::*;

        #[test]
        fn test_full_roundtrip() {
            let original = AprModel {
                metadata: AprMetadata::builder()
                    .name("pong-champion")
                    .version("2.0.0")
                    .author("PAIML")
                    .license("MIT")
                    .description("A really good pong player!")
                    .difficulty_levels(10)
                    .build()
                    .expect("metadata"),
                data: ModelData {
                    weights: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
                    biases: vec![0.01, 0.02],
                    architecture: ModelArchitecture::Mlp {
                        layers: vec![2, 4, 2],
                    },
                },
            };

            // Serialize
            let bytes = original.to_bytes().expect("Should serialize");

            // Verify header
            assert_eq!(&bytes[0..4], APR_MAGIC);

            // Deserialize
            let file = AprFile::from_bytes(&bytes).expect("Should deserialize");

            // Verify all fields
            assert_eq!(file.model.metadata.name, "pong-champion");
            assert_eq!(file.model.metadata.version.to_string(), "2.0.0");
            assert_eq!(file.model.metadata.author, "PAIML");
            assert_eq!(file.model.metadata.difficulty_levels, Some(10));
            assert_eq!(file.model.data.weights.len(), 8);
            assert_eq!(file.model.data.biases.len(), 2);
        }

        #[test]
        fn test_deterministic_serialization() {
            let model = AprModel::new_test_model();

            let bytes1 = model.to_bytes().expect("Should serialize");
            let bytes2 = model.to_bytes().expect("Should serialize again");

            assert_eq!(bytes1, bytes2, "Serialization should be deterministic");
        }
    }

    mod builtin_model_tests {
        use super::*;

        #[test]
        fn test_builtin_chase() {
            // Per spec: ai: builtin:chase
            let model = AprModel::builtin("chase").expect("Should have builtin chase");
            assert_eq!(model.metadata.name, "builtin-chase");
        }

        #[test]
        fn test_builtin_patrol() {
            // Per spec: ai: builtin:patrol
            let model = AprModel::builtin("patrol").expect("Should have builtin patrol");
            assert_eq!(model.metadata.name, "builtin-patrol");
        }

        #[test]
        fn test_builtin_wander() {
            // Per spec: ai: builtin:wander
            let model = AprModel::builtin("wander").expect("Should have builtin wander");
            assert_eq!(model.metadata.name, "builtin-wander");
        }

        #[test]
        fn test_unknown_builtin_fails() {
            let result = AprModel::builtin("nonexistent");
            assert!(matches!(result, Err(AprError::UnknownBuiltin { .. })));
        }
    }

    mod cosmin_quality_tests {
        use super::*;

        // Per spec Section 12.5: COSMIN-aligned model validation
        #[test]
        fn test_model_reliability_score() {
            let model = AprModel::new_test_model();
            let quality = model.assess_quality();

            // ICC > 0.70 required per spec
            assert!(
                quality.test_retest_reliability >= 0.70,
                "Reliability should be >= 0.70"
            );
        }

        #[test]
        fn test_model_meets_minimum_standards() {
            let model = AprModel::new_test_model();
            let quality = model.assess_quality();

            assert!(
                quality.meets_minimum_standards(),
                "Test model should meet minimum standards"
            );
        }
    }
}
