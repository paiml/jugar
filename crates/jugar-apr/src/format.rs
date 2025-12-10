//! APR file format handling.
//!
//! Per spec Section 4.1: File structure with magic, version, checksum.

use crate::error::AprError;
use crate::metadata::AprMetadata;
use crate::model::{AprModel, ModelData};
use crate::{MAX_MODEL_SIZE, MIN_SUPPORTED_VERSION};

/// Magic number for APR files
pub const APR_MAGIC: &[u8; 4] = b"APNR";

/// Current APR version
pub const APR_VERSION: u16 = 1;

/// Minimum header size (magic + version + checksum)
const HEADER_SIZE: usize = 10;

/// Parsed APR file
#[derive(Debug)]
pub struct AprFile {
    /// File version
    pub version: u16,
    /// Loaded model
    pub model: AprModel,
}

impl AprFile {
    /// Check if bytes start with APR magic number
    #[must_use]
    pub fn has_magic(bytes: &[u8]) -> bool {
        bytes.len() >= 4 && &bytes[0..4] == APR_MAGIC
    }

    /// Parse an APR file from bytes
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File too small
    /// - Invalid magic number
    /// - Unsupported version
    /// - Checksum mismatch
    /// - Invalid metadata or model data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AprError> {
        // Check minimum size
        if bytes.len() < HEADER_SIZE {
            return Err(AprError::FileTooSmall { size: bytes.len() });
        }

        // Check magic
        if !Self::has_magic(bytes) {
            return Err(AprError::invalid_magic(bytes));
        }

        // Read version
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version < MIN_SUPPORTED_VERSION {
            return Err(AprError::UnsupportedVersion {
                version,
                min_supported: MIN_SUPPORTED_VERSION,
            });
        }

        // Read and verify checksum
        let stored_checksum = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        let computed_checksum = crc32fast::hash(&bytes[HEADER_SIZE..]);
        if stored_checksum != computed_checksum {
            return Err(AprError::ChecksumMismatch {
                expected: stored_checksum,
                computed: computed_checksum,
            });
        }

        // Check size limit
        if bytes.len() > MAX_MODEL_SIZE {
            return Err(AprError::ModelTooLarge {
                size: bytes.len(),
                max: MAX_MODEL_SIZE,
            });
        }

        // Read metadata length (4 bytes after header)
        if bytes.len() < HEADER_SIZE + 4 {
            return Err(AprError::FileTooSmall { size: bytes.len() });
        }
        let metadata_len = u32::from_le_bytes([
            bytes[HEADER_SIZE],
            bytes[HEADER_SIZE + 1],
            bytes[HEADER_SIZE + 2],
            bytes[HEADER_SIZE + 3],
        ]) as usize;

        // Validate metadata bounds
        let metadata_start = HEADER_SIZE + 4;
        let metadata_end = metadata_start + metadata_len;
        if metadata_end > bytes.len() {
            return Err(AprError::CborDecode(
                "Metadata length exceeds file size".to_string(),
            ));
        }

        // Parse metadata
        let metadata = AprMetadata::from_cbor(&bytes[metadata_start..metadata_end])?;

        // Parse compressed model data
        let data_start = metadata_end;
        let data = ModelData::decompress(&bytes[data_start..])?;

        Ok(Self {
            version,
            model: AprModel { metadata, data },
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::model::ModelArchitecture;

    #[test]
    fn test_has_magic_valid() {
        let valid = b"APNRsomedata";
        assert!(AprFile::has_magic(valid));
    }

    #[test]
    fn test_has_magic_invalid() {
        let invalid = b"WRONGdata";
        assert!(!AprFile::has_magic(invalid));
    }

    #[test]
    fn test_has_magic_too_short() {
        let short = b"APR";
        assert!(!AprFile::has_magic(short));
    }

    #[test]
    fn test_file_too_small() {
        let tiny = b"APNR";
        let result = AprFile::from_bytes(tiny);
        assert!(matches!(result, Err(AprError::FileTooSmall { .. })));
    }

    #[test]
    fn test_invalid_magic() {
        let bad = b"WRONG_____";
        let result = AprFile::from_bytes(bad);
        assert!(matches!(result, Err(AprError::InvalidMagic { .. })));
    }

    #[test]
    fn test_version_zero_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(APR_MAGIC);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());

        let result = AprFile::from_bytes(&bytes);
        assert!(matches!(result, Err(AprError::UnsupportedVersion { .. })));
    }

    #[test]
    fn test_full_roundtrip() {
        let model = AprModel {
            metadata: AprMetadata::builder()
                .name("roundtrip-test")
                .version("1.0.0")
                .author("Test")
                .license("MIT")
                .build()
                .expect("metadata"),
            data: ModelData {
                weights: vec![1.0, 2.0, 3.0],
                biases: vec![0.1],
                architecture: ModelArchitecture::Mlp {
                    layers: vec![1, 2, 1],
                },
            },
        };

        let bytes = model.to_bytes().expect("serialize");
        let loaded = AprFile::from_bytes(&bytes).expect("deserialize");

        assert_eq!(loaded.model.metadata.name, "roundtrip-test");
        assert_eq!(loaded.model.data.weights, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_checksum_corruption_detected() {
        let model = AprModel::new_test_model();
        let mut bytes = model.to_bytes().expect("serialize");

        // Corrupt data after header
        if bytes.len() > 20 {
            bytes[20] ^= 0xFF;
        }

        let result = AprFile::from_bytes(&bytes);
        assert!(matches!(result, Err(AprError::ChecksumMismatch { .. })));
    }
}
