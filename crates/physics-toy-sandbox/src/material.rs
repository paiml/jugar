//! Material properties for physics objects
//!
//! This module implements Poka-Yoke (error-proofing) through:
//! - `NonZeroU32` for density (prevents division-by-zero in physics solver)
//! - Clamped bounciness coefficient [0.0, 1.0]
//! - Validated friction coefficients

use core::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::{Result, SandboxError};

/// Material preset for quick configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MaterialPreset {
    /// Wood: moderate bounce, moderate friction
    Wood,
    /// Metal: low bounce, low friction
    Metal,
    /// Rubber: high bounce, high friction
    #[default]
    Rubber,
    /// Ice: low bounce, very low friction
    Ice,
    /// Custom: user-defined properties
    Custom,
}

/// POKA-YOKE: Material properties with compile-time safety guarantees
///
/// Key safety features:
/// - `density` uses `NonZeroU32` to prevent division-by-zero in mass calculations
/// - Density stored as milli-kg/m³ for precision without floating point
/// - Bounciness clamped to [0.0, 1.0] by setter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Coefficient of restitution (0.0 = inelastic, 1.0 = perfectly elastic)
    /// POKA-YOKE: Clamped to [0.0, 1.0] by setter
    bounciness: f32,

    /// Static friction coefficient (typically 0.0 to 1.0+)
    friction_static: f32,

    /// Dynamic friction coefficient (typically less than static)
    friction_dynamic: f32,

    /// POKA-YOKE: Density in milli-kg/m³ (`NonZeroU32` prevents zero mass)
    /// To get kg/m³: divide by 1000.0
    density_milli: NonZeroU32,

    /// Visual preset for rendering
    pub preset: MaterialPreset,
}

/// Helper to create compile-time `NonZeroU32` constants
///
/// # Panics
/// Panics at compile-time if given zero (caught by const evaluation).
/// This is acceptable because:
/// 1. All uses are const-evaluated at compile time
/// 2. Zero values will cause a compile error, not runtime panic
#[allow(clippy::panic)]
const fn non_zero_u32(value: u32) -> NonZeroU32 {
    assert!(value != 0, "value must be non-zero");
    match NonZeroU32::new(value) {
        Some(v) => v,
        None => panic!("value must be non-zero"),
    }
}

/// Default density in milli-kg/m³ (1200.0 kg/m³)
const DEFAULT_DENSITY_MILLI: NonZeroU32 = non_zero_u32(1_200_000);

impl Default for MaterialProperties {
    fn default() -> Self {
        // Rubber-like default (good for chain reactions)
        Self {
            bounciness: 0.7,
            friction_static: 0.6,
            friction_dynamic: 0.4,
            density_milli: DEFAULT_DENSITY_MILLI,
            preset: MaterialPreset::Rubber,
        }
    }
}

/// Wood density: 700 kg/m³ as milli-kg/m³
const WOOD_DENSITY_MILLI: NonZeroU32 = non_zero_u32(700_000);

/// Metal density: 7850 kg/m³ as milli-kg/m³
const METAL_DENSITY_MILLI: NonZeroU32 = non_zero_u32(7_850_000);

/// Ice density: 917 kg/m³ as milli-kg/m³
const ICE_DENSITY_MILLI: NonZeroU32 = non_zero_u32(917_000);

impl MaterialProperties {
    /// Create new material with custom density (in kg/m³)
    ///
    /// # Errors
    /// Returns error if density is zero or negative
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn new(density_kg_m3: f32) -> Result<Self> {
        if density_kg_m3 <= 0.0 {
            return Err(SandboxError::InvalidMaterial {
                reason: "Density must be positive".to_string(),
            });
        }

        let density_milli = (density_kg_m3 * 1000.0) as u32;

        Ok(Self {
            density_milli: NonZeroU32::new(density_milli).ok_or_else(|| {
                SandboxError::InvalidMaterial {
                    reason: "Density too small".to_string(),
                }
            })?,
            ..Self::default()
        })
    }

    /// Create material from preset
    #[must_use]
    pub fn from_preset(preset: MaterialPreset) -> Self {
        match preset {
            MaterialPreset::Wood => Self {
                bounciness: 0.3,
                friction_static: 0.5,
                friction_dynamic: 0.3,
                density_milli: WOOD_DENSITY_MILLI,
                preset,
            },
            MaterialPreset::Metal => Self {
                bounciness: 0.2,
                friction_static: 0.4,
                friction_dynamic: 0.3,
                density_milli: METAL_DENSITY_MILLI,
                preset,
            },
            MaterialPreset::Ice => Self {
                bounciness: 0.1,
                friction_static: 0.05,
                friction_dynamic: 0.03,
                density_milli: ICE_DENSITY_MILLI,
                preset,
            },
            MaterialPreset::Rubber | MaterialPreset::Custom => Self::default(),
        }
    }

    /// Get bounciness (coefficient of restitution)
    #[must_use]
    pub const fn bounciness(&self) -> f32 {
        self.bounciness
    }

    /// Set bounciness with clamping (Poka-Yoke)
    pub fn set_bounciness(&mut self, value: f32) {
        self.bounciness = value.clamp(0.0, 1.0);
        self.preset = MaterialPreset::Custom;
    }

    /// Get static friction coefficient
    #[must_use]
    pub const fn friction_static(&self) -> f32 {
        self.friction_static
    }

    /// Set static friction (clamped to non-negative)
    pub fn set_friction_static(&mut self, value: f32) {
        self.friction_static = value.max(0.0);
        self.preset = MaterialPreset::Custom;
    }

    /// Get dynamic friction coefficient
    #[must_use]
    pub const fn friction_dynamic(&self) -> f32 {
        self.friction_dynamic
    }

    /// Set dynamic friction (clamped to non-negative)
    pub fn set_friction_dynamic(&mut self, value: f32) {
        self.friction_dynamic = value.max(0.0);
        self.preset = MaterialPreset::Custom;
    }

    /// Get density in kg/m³
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn density(&self) -> f32 {
        self.density_milli.get() as f32 / 1000.0
    }

    /// Set density in kg/m³
    ///
    /// # Errors
    /// Returns error if density is zero or negative
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn set_density(&mut self, density_kg_m3: f32) -> Result<()> {
        if density_kg_m3 <= 0.0 {
            return Err(SandboxError::InvalidMaterial {
                reason: "Density must be positive".to_string(),
            });
        }

        let density_milli = (density_kg_m3 * 1000.0) as u32;
        self.density_milli =
            NonZeroU32::new(density_milli).ok_or_else(|| SandboxError::InvalidMaterial {
                reason: "Density too small".to_string(),
            })?;
        self.preset = MaterialPreset::Custom;
        Ok(())
    }

    /// Get raw density in milli-kg/m³ (for serialization)
    #[must_use]
    pub const fn density_raw(&self) -> NonZeroU32 {
        self.density_milli
    }

    /// Calculate mass for a given volume (in m³)
    #[must_use]
    pub fn mass_for_volume(&self, volume_m3: f32) -> f32 {
        // POKA-YOKE: density_milli is NonZeroU32, so mass is always positive
        self.density() * volume_m3
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // =========================================================================
    // EXTREME TDD: Poka-Yoke validation tests
    // =========================================================================

    mod poka_yoke_tests {
        use super::*;

        #[test]
        fn test_density_cannot_be_zero() {
            let result = MaterialProperties::new(0.0);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("positive"));
        }

        #[test]
        fn test_density_cannot_be_negative() {
            let result = MaterialProperties::new(-100.0);
            assert!(result.is_err());
        }

        #[test]
        fn test_set_density_rejects_zero() {
            let mut mat = MaterialProperties::default();
            let result = mat.set_density(0.0);
            assert!(result.is_err());
        }

        #[test]
        fn test_bounciness_clamped_to_max() {
            let mut mat = MaterialProperties::default();
            mat.set_bounciness(1.5);
            assert!((mat.bounciness() - 1.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_bounciness_clamped_to_min() {
            let mut mat = MaterialProperties::default();
            mat.set_bounciness(-0.5);
            assert!((mat.bounciness() - 0.0).abs() < f32::EPSILON);
        }

        #[test]
        fn test_friction_cannot_be_negative() {
            let mut mat = MaterialProperties::default();
            mat.set_friction_static(-1.0);
            assert!(mat.friction_static() >= 0.0);

            mat.set_friction_dynamic(-1.0);
            assert!(mat.friction_dynamic() >= 0.0);
        }

        #[test]
        fn test_mass_always_positive() {
            let mat = MaterialProperties::default();
            let mass = mat.mass_for_volume(1.0);
            assert!(mass > 0.0);
        }

        #[test]
        fn test_mass_for_zero_volume() {
            let mat = MaterialProperties::default();
            let mass = mat.mass_for_volume(0.0);
            assert!((mass - 0.0).abs() < f32::EPSILON);
        }
    }

    mod preset_tests {
        use super::*;

        #[test]
        fn test_wood_preset() {
            let mat = MaterialProperties::from_preset(MaterialPreset::Wood);
            assert!(mat.bounciness() < 0.5); // Wood doesn't bounce much
            assert!(mat.density() > 500.0 && mat.density() < 1000.0);
        }

        #[test]
        fn test_metal_preset() {
            let mat = MaterialProperties::from_preset(MaterialPreset::Metal);
            assert!(mat.density() > 5000.0); // Metal is heavy
        }

        #[test]
        fn test_rubber_preset() {
            let mat = MaterialProperties::from_preset(MaterialPreset::Rubber);
            assert!(mat.bounciness() > 0.5); // Rubber bounces
        }

        #[test]
        fn test_ice_preset() {
            let mat = MaterialProperties::from_preset(MaterialPreset::Ice);
            assert!(mat.friction_static() < 0.1); // Ice is slippery
            assert!(mat.friction_dynamic() < 0.1);
        }

        #[test]
        fn test_default_is_rubber() {
            let mat = MaterialProperties::default();
            assert_eq!(mat.preset, MaterialPreset::Rubber);
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_material_roundtrip() {
            let mat = MaterialProperties::from_preset(MaterialPreset::Metal);
            let json = serde_json::to_string(&mat).unwrap();
            let restored: MaterialProperties = serde_json::from_str(&json).unwrap();
            assert_eq!(mat, restored);
        }

        #[test]
        fn test_custom_material_roundtrip() {
            let mut mat = MaterialProperties::new(500.0).unwrap();
            mat.set_bounciness(0.42);
            mat.set_friction_static(0.33);

            let json = serde_json::to_string(&mat).unwrap();
            let restored: MaterialProperties = serde_json::from_str(&json).unwrap();

            assert!((mat.bounciness() - restored.bounciness()).abs() < f32::EPSILON);
            assert!((mat.density() - restored.density()).abs() < 0.001);
        }
    }

    mod setter_marks_custom_tests {
        use super::*;

        #[test]
        fn test_set_bounciness_marks_custom() {
            let mut mat = MaterialProperties::from_preset(MaterialPreset::Wood);
            mat.set_bounciness(0.5);
            assert_eq!(mat.preset, MaterialPreset::Custom);
        }

        #[test]
        fn test_set_density_marks_custom() {
            let mut mat = MaterialProperties::from_preset(MaterialPreset::Wood);
            mat.set_density(100.0).unwrap();
            assert_eq!(mat.preset, MaterialPreset::Custom);
        }

        #[test]
        fn test_set_friction_marks_custom() {
            let mut mat = MaterialProperties::from_preset(MaterialPreset::Wood);
            mat.set_friction_static(0.1);
            assert_eq!(mat.preset, MaterialPreset::Custom);
        }
    }
}
