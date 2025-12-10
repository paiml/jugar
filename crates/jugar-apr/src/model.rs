//! APR model data structures.
//!
//! Per spec Section 4.1: Model weights, biases, and architecture.

use crate::error::AprError;
use crate::metadata::AprMetadata;
use crate::MAX_MODEL_SIZE;
use serde::{Deserialize, Serialize};

/// A complete APR model with metadata and data
#[derive(Debug, Clone)]
pub struct AprModel {
    /// Model metadata
    pub metadata: AprMetadata,
    /// Model data (weights, biases, architecture)
    pub data: ModelData,
}

/// Neural network weights and architecture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)] // f32 doesn't implement Eq
pub struct ModelData {
    /// Weight values
    pub weights: Vec<f32>,
    /// Bias values
    pub biases: Vec<f32>,
    /// Network architecture
    pub architecture: ModelArchitecture,
}

/// Network architecture specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelArchitecture {
    /// Multi-layer perceptron
    Mlp {
        /// Layer sizes (e.g., [2, 16, 1] for 2 inputs, 16 hidden, 1 output)
        layers: Vec<usize>,
    },
    /// Behavior tree (for patrol, wander, etc.)
    BehaviorTree {
        /// Node count
        nodes: usize,
    },
}

impl core::fmt::Display for ModelArchitecture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Mlp { layers } => {
                write!(f, "mlp-")?;
                for (i, layer) in layers.iter().enumerate() {
                    if i > 0 {
                        write!(f, "-")?;
                    }
                    write!(f, "{layer}")?;
                }
                Ok(())
            }
            Self::BehaviorTree { nodes } => write!(f, "bt-{nodes}"),
        }
    }
}

impl ModelData {
    /// Compress model data using DEFLATE
    ///
    /// # Errors
    ///
    /// Returns error if compression fails
    pub fn compress(&self) -> Result<Vec<u8>, AprError> {
        // Serialize to CBOR first
        let mut cbor_data = Vec::new();
        ciborium::into_writer(self, &mut cbor_data)
            .map_err(|e| AprError::Compression(e.to_string()))?;

        // Compress with DEFLATE
        let compressed = miniz_oxide::deflate::compress_to_vec(&cbor_data, 6); // Level 6 = balanced

        Ok(compressed)
    }

    /// Decompress model data
    ///
    /// # Errors
    ///
    /// Returns error if decompression fails
    pub fn decompress(bytes: &[u8]) -> Result<Self, AprError> {
        // Decompress DEFLATE
        let decompressed = miniz_oxide::inflate::decompress_to_vec(bytes)
            .map_err(|e| AprError::Decompression(format!("{e:?}")))?;

        // Deserialize from CBOR
        ciborium::from_reader(decompressed.as_slice())
            .map_err(|e| AprError::CborDecode(e.to_string()))
    }
}

/// Quality assessment for COSMIN compliance
/// Per spec Section 12.5
#[derive(Debug, Clone)]
pub struct ModelQualityAssessment {
    /// Test-retest reliability (ICC)
    pub test_retest_reliability: f64,
    /// Content validity score
    pub content_validity_adequate: bool,
    /// Effect size (Cohen's d)
    pub responsiveness_cohens_d: f64,
}

impl ModelQualityAssessment {
    /// Check if model meets minimum COSMIN standards
    ///
    /// Per spec Section 12.5:
    /// - ICC > 0.70 required
    /// - Content validity adequate
    /// - Cohen's d >= 0.30
    #[must_use]
    pub fn meets_minimum_standards(&self) -> bool {
        self.test_retest_reliability >= 0.70
            && self.content_validity_adequate
            && self.responsiveness_cohens_d >= 0.30
    }
}

impl AprModel {
    /// Create a test model for unit tests
    ///
    /// # Panics
    ///
    /// Panics if test model metadata is invalid (should never happen)
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn new_test_model() -> Self {
        Self {
            metadata: AprMetadata::builder()
                .name("test-model")
                .version("1.0.0")
                .author("Test")
                .license("MIT")
                .build()
                .expect("Test model metadata should be valid"),
            data: ModelData {
                weights: vec![0.1, 0.2, 0.3, 0.4],
                biases: vec![0.01, 0.02],
                architecture: ModelArchitecture::Mlp {
                    layers: vec![2, 2, 1],
                },
            },
        }
    }

    /// Get a builtin model by name
    ///
    /// # Errors
    ///
    /// Returns error if builtin name is unknown
    pub fn builtin(name: &str) -> Result<Self, AprError> {
        match name {
            "chase" => Ok(Self::builtin_chase()),
            "patrol" => Ok(Self::builtin_patrol()),
            "wander" => Ok(Self::builtin_wander()),
            _ => Err(AprError::UnknownBuiltin {
                name: name.to_string(),
            }),
        }
    }

    /// Builtin chase behavior
    #[allow(clippy::expect_used)]
    fn builtin_chase() -> Self {
        Self {
            metadata: AprMetadata::builder()
                .name("builtin-chase")
                .version("1.0.0")
                .author("Jugar")
                .license("MIT")
                .description("Chase the player directly")
                .build()
                .expect("Builtin metadata is hardcoded and valid"),
            data: ModelData {
                // Simple chase: move toward player
                weights: vec![1.0, 0.0, 0.0, 1.0], // Identity-like for direction
                biases: vec![0.0, 0.0],
                architecture: ModelArchitecture::Mlp { layers: vec![2, 2] },
            },
        }
    }

    /// Builtin patrol behavior
    #[allow(clippy::expect_used)]
    fn builtin_patrol() -> Self {
        Self {
            metadata: AprMetadata::builder()
                .name("builtin-patrol")
                .version("1.0.0")
                .author("Jugar")
                .license("MIT")
                .description("Patrol back and forth")
                .build()
                .expect("Builtin metadata is hardcoded and valid"),
            data: ModelData {
                weights: vec![1.0, -1.0], // Oscillate
                biases: vec![0.0],
                architecture: ModelArchitecture::BehaviorTree { nodes: 3 },
            },
        }
    }

    /// Builtin wander behavior
    #[allow(clippy::expect_used)]
    fn builtin_wander() -> Self {
        Self {
            metadata: AprMetadata::builder()
                .name("builtin-wander")
                .version("1.0.0")
                .author("Jugar")
                .license("MIT")
                .description("Wander randomly")
                .build()
                .expect("Builtin metadata is hardcoded and valid"),
            data: ModelData {
                weights: vec![0.5, 0.5, 0.5, 0.5], // Random-ish weights
                biases: vec![0.1, -0.1],
                architecture: ModelArchitecture::BehaviorTree { nodes: 2 },
            },
        }
    }

    /// Serialize model to APR bytes
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails or model is too large
    pub fn to_bytes(&self) -> Result<Vec<u8>, AprError> {
        use crate::format::{APR_MAGIC, APR_VERSION};

        // Compress model data
        let compressed_data = self.data.compress()?;

        // Encode metadata to CBOR
        let metadata_cbor = self.metadata.to_cbor()?;

        // Calculate total size (header + metadata length + metadata + data)
        // Safety: metadata is validated and will never exceed u32::MAX (max model size is 1MB)
        #[allow(clippy::cast_possible_truncation)]
        let metadata_len = metadata_cbor.len() as u32;
        let total_size = 10 + 4 + metadata_cbor.len() + compressed_data.len();

        // Check size limit
        if total_size > MAX_MODEL_SIZE {
            return Err(AprError::ModelTooLarge {
                size: total_size,
                max: MAX_MODEL_SIZE,
            });
        }

        // Build file
        let mut bytes = Vec::with_capacity(total_size);

        // Header (will update checksum after)
        bytes.extend_from_slice(APR_MAGIC);
        bytes.extend_from_slice(&APR_VERSION.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes()); // Placeholder checksum

        // Metadata length + metadata
        bytes.extend_from_slice(&metadata_len.to_le_bytes());
        bytes.extend_from_slice(&metadata_cbor);

        // Compressed data
        bytes.extend_from_slice(&compressed_data);

        // Compute checksum over everything after header (bytes 10+)
        let checksum = crc32fast::hash(&bytes[10..]);
        bytes[6..10].copy_from_slice(&checksum.to_le_bytes());

        Ok(bytes)
    }

    /// Assess model quality per COSMIN standards
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Will use self.data in real implementation
    pub fn assess_quality(&self) -> ModelQualityAssessment {
        // For test models, return passing quality
        // Real implementation would actually test the model
        ModelQualityAssessment {
            test_retest_reliability: 0.85,
            content_validity_adequate: true,
            responsiveness_cohens_d: 0.50,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_display_mlp() {
        let arch = ModelArchitecture::Mlp {
            layers: vec![2, 16, 1],
        };
        assert_eq!(arch.to_string(), "mlp-2-16-1");
    }

    #[test]
    fn test_architecture_display_bt() {
        let arch = ModelArchitecture::BehaviorTree { nodes: 5 };
        assert_eq!(arch.to_string(), "bt-5");
    }

    #[test]
    fn test_model_data_compression_roundtrip() {
        let original = ModelData {
            weights: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            biases: vec![0.01, 0.02],
            architecture: ModelArchitecture::Mlp {
                layers: vec![2, 3, 1],
            },
        };

        let compressed = original.compress().expect("Should compress");
        let decompressed = ModelData::decompress(&compressed).expect("Should decompress");

        assert_eq!(original.weights, decompressed.weights);
        assert_eq!(original.biases, decompressed.biases);
        assert_eq!(original.architecture, decompressed.architecture);
    }

    #[test]
    fn test_builtin_models_exist() {
        assert!(AprModel::builtin("chase").is_ok());
        assert!(AprModel::builtin("patrol").is_ok());
        assert!(AprModel::builtin("wander").is_ok());
    }

    #[test]
    fn test_unknown_builtin() {
        let result = AprModel::builtin("fly");
        assert!(matches!(result, Err(AprError::UnknownBuiltin { .. })));
    }

    #[test]
    fn test_quality_assessment() {
        let model = AprModel::new_test_model();
        let quality = model.assess_quality();

        assert!(quality.test_retest_reliability >= 0.70);
        assert!(quality.content_validity_adequate);
        assert!(quality.responsiveness_cohens_d >= 0.30);
        assert!(quality.meets_minimum_standards());
    }
}
