//! Error types for APR model handling.
//!
//! Per spec Section 4.1: Errors during model loading/saving.

use thiserror::Error;

/// Errors that can occur when working with APR models
#[derive(Debug, Error)]
pub enum AprError {
    /// Invalid magic number (not "APNR")
    #[error("Invalid APR file: expected magic 'APNR', got {found:?}")]
    InvalidMagic {
        /// The bytes found instead of "APNR"
        found: [u8; 4],
    },

    /// Unsupported version
    #[error("Unsupported APR version: {version} (supported: {min_supported}+)")]
    UnsupportedVersion {
        /// Version found in file
        version: u16,
        /// Minimum supported version
        min_supported: u16,
    },

    /// Checksum mismatch
    #[error("APR file corrupted: checksum mismatch (expected {expected:08x}, got {computed:08x})")]
    ChecksumMismatch {
        /// Expected checksum from header
        expected: u32,
        /// Computed checksum from data
        computed: u32,
    },

    /// Model too large
    #[error("Model too large: {size} bytes (max: {max} bytes)")]
    ModelTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// File too small
    #[error("APR file too small: {size} bytes (minimum header: 10 bytes)")]
    FileTooSmall {
        /// Actual size
        size: usize,
    },

    /// Invalid metadata name
    #[error("Invalid model name: '{name}' (must be 3-50 alphanumeric/hyphen chars)")]
    InvalidName {
        /// The invalid name
        name: String,
    },

    /// Invalid semver version
    #[error("Invalid version string: {version}")]
    InvalidVersion {
        /// The invalid version
        version: String,
    },

    /// CBOR encoding error
    #[error("CBOR encoding error: {0}")]
    CborEncode(String),

    /// CBOR decoding error
    #[error("CBOR decoding error: {0}")]
    CborDecode(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Unknown builtin model
    #[error("Unknown builtin model: '{name}' (available: chase, patrol, wander)")]
    UnknownBuiltin {
        /// The requested builtin name
        name: String,
    },

    /// Missing required field in metadata
    #[error("Missing required field: {field}")]
    MissingField {
        /// The missing field name
        field: &'static str,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AprError {
    /// Create an invalid magic error from bytes
    #[must_use]
    pub fn invalid_magic(bytes: &[u8]) -> Self {
        let mut found = [0u8; 4];
        let len = bytes.len().min(4);
        found[..len].copy_from_slice(&bytes[..len]);
        Self::InvalidMagic { found }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AprError::InvalidMagic { found: *b"BAAD" };
        assert!(err.to_string().contains("APNR"));
    }

    #[test]
    fn test_checksum_error_hex_format() {
        let err = AprError::ChecksumMismatch {
            expected: 0xDEAD_BEEF,
            computed: 0xCAFE_BABE,
        };
        let msg = err.to_string();
        assert!(msg.contains("deadbeef"));
        assert!(msg.contains("cafebabe"));
    }

    #[test]
    fn test_model_too_large_error() {
        let err = AprError::ModelTooLarge {
            size: 2_000_000,
            max: 1_048_576,
        };
        assert!(err.to_string().contains("2000000"));
        assert!(err.to_string().contains("1048576"));
    }
}
