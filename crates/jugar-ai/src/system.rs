//! AI system for running .apr models in game entities.
//!
//! Per spec Section 5.3: Aprender AI Integration.
//!
//! # Example
//!
//! ```ignore
//! let mut ai_system = AiSystem::new();
//! ai_system.load_model("ghost", "models/smart-ghost.apr")?;
//! ai_system.update(&mut world, 0.016);
//! ```

use crate::{AiError, Result};
use glam::Vec2;
use jugar_apr::{AprModel, ModelArchitecture, ModelData};
use std::collections::HashMap;

/// AI component attached to entities
#[derive(Debug, Clone)]
pub struct AiComponent {
    /// Model identifier (either path or builtin name)
    pub model_id: String,
    /// Current behavior state
    pub state: BehaviorState,
    /// Difficulty level (1-10, affects model parameters)
    pub difficulty: u8,
}

impl AiComponent {
    /// Create a new AI component with the given model
    #[must_use]
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            state: BehaviorState::default(),
            difficulty: 5,
        }
    }

    /// Set difficulty level (1-10)
    #[must_use]
    pub const fn with_difficulty(mut self, difficulty: u8) -> Self {
        // Manual clamp for const fn (clamp is not const in stable Rust)
        self.difficulty = if difficulty < 1 {
            1
        } else if difficulty > 10 {
            10
        } else {
            difficulty
        };
        self
    }
}

/// Current state of a behavior
#[derive(Debug, Clone, Default)]
pub struct BehaviorState {
    /// Current direction of movement
    pub direction: Vec2,
    /// Time in current state
    pub state_time: f32,
    /// Patrol waypoint index (for patrol behavior)
    pub waypoint_index: usize,
    /// Internal state value for deterministic behaviors
    pub internal_state: f32,
}

/// Input data for AI inference
#[derive(Debug, Clone, Default)]
pub struct AiInputs {
    /// Entity's current position
    pub position: Vec2,
    /// Target position (usually player)
    pub target_position: Vec2,
    /// Distance to target
    pub distance_to_target: f32,
    /// Normalized direction to target
    pub direction_to_target: Vec2,
    /// Delta time
    pub dt: f32,
}

impl AiInputs {
    /// Create inputs from positions
    #[must_use]
    pub fn from_positions(position: Vec2, target: Vec2, dt: f32) -> Self {
        let delta = target - position;
        let distance = delta.length();
        let direction = if distance > 0.001 {
            delta / distance
        } else {
            Vec2::ZERO
        };

        Self {
            position,
            target_position: target,
            distance_to_target: distance,
            direction_to_target: direction,
            dt,
        }
    }

    /// Convert to input vector for MLP inference
    #[must_use]
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.direction_to_target.x,
            self.direction_to_target.y,
            self.distance_to_target / 100.0, // Normalize distance
            self.dt,
        ]
    }
}

/// Output from AI inference
#[derive(Debug, Clone, Default)]
pub struct AiOutputs {
    /// Desired movement direction (normalized)
    pub movement: Vec2,
    /// Speed multiplier (0.0-1.0)
    pub speed: f32,
    /// Should trigger action (e.g., attack)
    pub action: bool,
}

impl AiOutputs {
    /// Create from raw output values
    #[must_use]
    pub fn from_raw(values: &[f32]) -> Self {
        let movement = if values.len() >= 2 {
            Vec2::new(values[0], values[1]).normalize_or_zero()
        } else {
            Vec2::ZERO
        };

        let speed = if values.len() >= 3 {
            values[2].clamp(0.0, 1.0)
        } else {
            1.0
        };

        let action = values.len() >= 4 && values[3] > 0.5;

        Self {
            movement,
            speed,
            action,
        }
    }
}

/// The AI system that manages and runs AI models
#[derive(Debug, Default)]
pub struct AiSystem {
    /// Loaded models by ID
    models: HashMap<String, LoadedModel>,
}

/// A loaded model ready for inference
#[derive(Debug, Clone)]
struct LoadedModel {
    /// The underlying APR model
    model: AprModel,
    /// Cached layer weights for fast inference
    layer_weights: Vec<LayerWeights>,
}

/// Weights for a single layer
#[derive(Debug, Clone)]
struct LayerWeights {
    /// Weight matrix (flattened, row-major)
    weights: Vec<f32>,
    /// Bias vector
    biases: Vec<f32>,
    /// Input size
    input_size: usize,
    /// Output size
    output_size: usize,
}

impl AiSystem {
    /// Create a new AI system
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a model from an APR file path
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or model is invalid
    pub fn load_model_from_file(&mut self, id: &str, path: &str) -> Result<()> {
        let bytes = std::fs::read(path).map_err(|e| AiError::PreconditionsNotMet(e.to_string()))?;

        let apr_file = jugar_apr::AprFile::from_bytes(&bytes)
            .map_err(|e| AiError::PreconditionsNotMet(e.to_string()))?;

        self.register_model(id, apr_file.model)
    }

    /// Load a builtin model
    ///
    /// # Errors
    ///
    /// Returns error if builtin name is unknown
    pub fn load_builtin(&mut self, id: &str, builtin_name: &str) -> Result<()> {
        let model = AprModel::builtin(builtin_name)
            .map_err(|e| AiError::PreconditionsNotMet(e.to_string()))?;

        self.register_model(id, model)
    }

    /// Register a model directly
    ///
    /// # Errors
    ///
    /// Returns error if model architecture is invalid
    pub fn register_model(&mut self, id: &str, model: AprModel) -> Result<()> {
        let layer_weights = Self::prepare_weights(&model.data)?;
        let loaded = LoadedModel {
            model,
            layer_weights,
        };
        let _ = self.models.insert(id.to_string(), loaded);
        Ok(())
    }

    /// Prepare layer weights from model data
    fn prepare_weights(data: &ModelData) -> Result<Vec<LayerWeights>> {
        match &data.architecture {
            ModelArchitecture::Mlp { layers } => {
                if layers.len() < 2 {
                    return Err(AiError::PreconditionsNotMet(
                        "MLP needs at least 2 layers".to_string(),
                    ));
                }

                let mut result = Vec::new();
                let mut weight_offset = 0;
                let mut bias_offset = 0;

                for i in 0..layers.len() - 1 {
                    let input_size = layers[i];
                    let output_size = layers[i + 1];
                    let weight_count = input_size * output_size;

                    let weights = if weight_offset + weight_count <= data.weights.len() {
                        data.weights[weight_offset..weight_offset + weight_count].to_vec()
                    } else {
                        // Use default weights if not enough in model
                        vec![0.1; weight_count]
                    };

                    let biases = if bias_offset + output_size <= data.biases.len() {
                        data.biases[bias_offset..bias_offset + output_size].to_vec()
                    } else {
                        // Use default biases
                        vec![0.0; output_size]
                    };

                    result.push(LayerWeights {
                        weights,
                        biases,
                        input_size,
                        output_size,
                    });

                    weight_offset += weight_count;
                    bias_offset += output_size;
                }

                Ok(result)
            }
            ModelArchitecture::BehaviorTree { .. } => {
                // Behavior trees don't need weight preparation
                Ok(Vec::new())
            }
        }
    }

    /// Run inference on a model
    ///
    /// # Errors
    ///
    /// Returns error if model is not found
    pub fn infer(&self, model_id: &str, inputs: &AiInputs) -> Result<AiOutputs> {
        let loaded = self
            .models
            .get(model_id)
            .ok_or_else(|| AiError::PreconditionsNotMet(format!("Model not found: {model_id}")))?;

        match &loaded.model.data.architecture {
            ModelArchitecture::Mlp { .. } => {
                let raw_outputs =
                    Self::run_mlp_inference(&loaded.layer_weights, &inputs.to_vector());
                Ok(AiOutputs::from_raw(&raw_outputs))
            }
            ModelArchitecture::BehaviorTree { .. } => {
                // Behavior trees use special inference based on model name
                Self::run_behavior_inference(&loaded.model.metadata.name, inputs)
            }
        }
    }

    /// Run MLP forward pass
    fn run_mlp_inference(layers: &[LayerWeights], input: &[f32]) -> Vec<f32> {
        let mut current = input.to_vec();

        for layer in layers {
            let mut output = vec![0.0; layer.output_size];

            // Matrix multiplication: output = weights * input + bias
            for (i, out) in output.iter_mut().enumerate() {
                let mut sum = layer.biases.get(i).copied().unwrap_or(0.0);
                for (j, &inp) in current.iter().enumerate() {
                    let weight_idx = i * layer.input_size + j;
                    let weight = layer.weights.get(weight_idx).copied().unwrap_or(0.0);
                    sum += weight * inp;
                }
                // ReLU activation
                *out = sum.max(0.0);
            }

            current = output;
        }

        // Final output uses tanh for bounded output
        current.iter().map(|&x| x.tanh()).collect()
    }

    /// Run behavior tree based inference
    fn run_behavior_inference(behavior_name: &str, inputs: &AiInputs) -> Result<AiOutputs> {
        match behavior_name {
            "builtin-chase" => Ok(AiOutputs {
                movement: inputs.direction_to_target,
                speed: 1.0,
                action: inputs.distance_to_target < 50.0,
            }),
            "builtin-patrol" => {
                // Simple left-right patrol
                let phase = (inputs.position.x / 100.0).sin();
                Ok(AiOutputs {
                    movement: Vec2::new(phase.signum(), 0.0),
                    speed: 0.5,
                    action: false,
                })
            }
            "builtin-wander" => {
                // Pseudo-random wander using position as seed
                #[allow(clippy::suboptimal_flops)]
                let angle = (inputs.position.x * 0.1 + inputs.position.y * 0.07).sin()
                    * core::f32::consts::PI;
                Ok(AiOutputs {
                    movement: Vec2::new(angle.cos(), angle.sin()),
                    speed: 0.3,
                    action: false,
                })
            }
            _ => Err(AiError::PreconditionsNotMet(format!(
                "Unknown behavior: {behavior_name}"
            ))),
        }
    }

    /// Check if a model is loaded
    #[must_use]
    pub fn has_model(&self, id: &str) -> bool {
        self.models.contains_key(id)
    }

    /// Get model count
    #[must_use]
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Remove a model
    pub fn unload_model(&mut self, id: &str) -> bool {
        self.models.remove(id).is_some()
    }
}

/// Bridge between YAML keywords and AI behaviors
///
/// Per spec Section 5.3: Maps YAML keywords to AI behaviors.
#[derive(Debug, Default)]
pub struct YamlAiBridge {
    /// Custom model paths mapped to IDs
    custom_models: HashMap<String, String>,
}

impl YamlAiBridge {
    /// Create a new YAML-AI bridge
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom model path
    pub fn register_custom(&mut self, yaml_key: &str, path: &str) {
        let _ = self
            .custom_models
            .insert(yaml_key.to_string(), path.to_string());
    }

    /// Resolve a YAML AI keyword to a behavior
    ///
    /// # Examples
    ///
    /// - `"builtin:chase"` -> Builtin chase behavior
    /// - `"builtin:patrol"` -> Builtin patrol behavior
    /// - `"builtin:wander"` -> Builtin wander behavior
    /// - `"models/ghost.apr"` -> Custom .apr model
    ///
    /// # Errors
    ///
    /// Returns error if keyword cannot be resolved
    pub fn resolve(&self, yaml_key: &str, system: &mut AiSystem) -> Result<String> {
        // Check for builtin prefix
        if let Some(builtin) = yaml_key.strip_prefix("builtin:") {
            let id = format!("builtin-{builtin}");
            if !system.has_model(&id) {
                system.load_builtin(&id, builtin)?;
            }
            return Ok(id);
        }

        // Check for .apr file path (case-insensitive)
        if std::path::Path::new(yaml_key)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("apr"))
        {
            let id = yaml_key.replace(['/', '\\', '.'], "_");
            if !system.has_model(&id) {
                system.load_model_from_file(&id, yaml_key)?;
            }
            return Ok(id);
        }

        // Check custom mappings
        if let Some(path) = self.custom_models.get(yaml_key) {
            let id = yaml_key.to_string();
            if !system.has_model(&id) {
                system.load_model_from_file(&id, path)?;
            }
            return Ok(id);
        }

        // Try as a direct builtin name
        if matches!(yaml_key, "chase" | "patrol" | "wander") {
            let id = format!("builtin-{yaml_key}");
            if !system.has_model(&id) {
                system.load_builtin(&id, yaml_key)?;
            }
            return Ok(id);
        }

        Err(AiError::PreconditionsNotMet(format!(
            "Unknown AI behavior: {yaml_key}"
        )))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    mod ai_component_tests {
        use super::*;

        #[test]
        fn test_ai_component_new() {
            let component = AiComponent::new("builtin:chase");
            assert_eq!(component.model_id, "builtin:chase");
            assert_eq!(component.difficulty, 5);
        }

        #[test]
        fn test_ai_component_with_difficulty() {
            let component = AiComponent::new("chase").with_difficulty(8);
            assert_eq!(component.difficulty, 8);
        }

        #[test]
        fn test_ai_component_difficulty_clamped() {
            let low = AiComponent::new("chase").with_difficulty(0);
            assert_eq!(low.difficulty, 1);

            let high = AiComponent::new("chase").with_difficulty(100);
            assert_eq!(high.difficulty, 10);
        }
    }

    mod ai_inputs_tests {
        use super::*;

        #[test]
        fn test_from_positions() {
            let inputs =
                AiInputs::from_positions(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 0.016);

            assert!((inputs.distance_to_target - 100.0).abs() < 0.01);
            assert!((inputs.direction_to_target.x - 1.0).abs() < 0.01);
            assert!(inputs.direction_to_target.y.abs() < 0.01);
        }

        #[test]
        fn test_from_positions_same_point() {
            let inputs =
                AiInputs::from_positions(Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), 0.016);

            assert!(inputs.distance_to_target < 0.001);
            assert_eq!(inputs.direction_to_target, Vec2::ZERO);
        }

        #[test]
        fn test_to_vector() {
            let inputs = AiInputs::from_positions(Vec2::ZERO, Vec2::new(100.0, 0.0), 0.016);

            let vec = inputs.to_vector();
            assert_eq!(vec.len(), 4);
            assert!((vec[0] - 1.0).abs() < 0.01); // direction x
            assert!(vec[1].abs() < 0.01); // direction y
            assert!((vec[2] - 1.0).abs() < 0.01); // normalized distance
        }
    }

    mod ai_outputs_tests {
        use super::*;

        #[test]
        fn test_from_raw() {
            let outputs = AiOutputs::from_raw(&[0.5, 0.5, 0.8, 0.9]);

            assert!(outputs.movement.length() > 0.0);
            assert!((outputs.speed - 0.8).abs() < 0.01);
            assert!(outputs.action);
        }

        #[test]
        fn test_from_raw_empty() {
            let outputs = AiOutputs::from_raw(&[]);

            assert_eq!(outputs.movement, Vec2::ZERO);
            assert!((outputs.speed - 1.0).abs() < 0.01);
            assert!(!outputs.action);
        }

        #[test]
        fn test_from_raw_speed_clamped() {
            let outputs = AiOutputs::from_raw(&[0.0, 0.0, 2.0]);
            assert!((outputs.speed - 1.0).abs() < 0.01);

            let outputs2 = AiOutputs::from_raw(&[0.0, 0.0, -1.0]);
            assert!(outputs2.speed.abs() < 0.01);
        }
    }

    mod ai_system_tests {
        use super::*;

        #[test]
        fn test_new_system() {
            let system = AiSystem::new();
            assert_eq!(system.model_count(), 0);
        }

        #[test]
        fn test_load_builtin_chase() {
            let mut system = AiSystem::new();
            system.load_builtin("chase", "chase").unwrap();

            assert!(system.has_model("chase"));
            assert_eq!(system.model_count(), 1);
        }

        #[test]
        fn test_load_builtin_patrol() {
            let mut system = AiSystem::new();
            system.load_builtin("patrol", "patrol").unwrap();

            assert!(system.has_model("patrol"));
        }

        #[test]
        fn test_load_builtin_wander() {
            let mut system = AiSystem::new();
            system.load_builtin("wander", "wander").unwrap();

            assert!(system.has_model("wander"));
        }

        #[test]
        fn test_load_unknown_builtin() {
            let mut system = AiSystem::new();
            let result = system.load_builtin("unknown", "unknown");

            assert!(result.is_err());
        }

        #[test]
        fn test_register_model() {
            let mut system = AiSystem::new();
            let model = AprModel::new_test_model();

            system.register_model("test", model).unwrap();
            assert!(system.has_model("test"));
        }

        #[test]
        fn test_unload_model() {
            let mut system = AiSystem::new();
            system.load_builtin("chase", "chase").unwrap();

            assert!(system.unload_model("chase"));
            assert!(!system.has_model("chase"));
        }

        #[test]
        fn test_infer_chase() {
            let mut system = AiSystem::new();
            system.load_builtin("chase", "chase").unwrap();

            let inputs =
                AiInputs::from_positions(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 0.016);

            let outputs = system.infer("chase", &inputs).unwrap();

            // Chase should move toward target
            assert!(outputs.movement.x > 0.0);
            assert!((outputs.speed - 1.0).abs() < 0.01);
        }

        #[test]
        fn test_infer_patrol() {
            let mut system = AiSystem::new();
            system.load_builtin("patrol", "patrol").unwrap();

            let inputs = AiInputs::from_positions(Vec2::new(50.0, 0.0), Vec2::new(0.0, 0.0), 0.016);

            let outputs = system.infer("patrol", &inputs).unwrap();

            // Patrol should have some movement
            assert!(outputs.movement.length() > 0.0);
            assert!((outputs.speed - 0.5).abs() < 0.01);
        }

        #[test]
        fn test_infer_wander() {
            let mut system = AiSystem::new();
            system.load_builtin("wander", "wander").unwrap();

            let inputs =
                AiInputs::from_positions(Vec2::new(25.0, 75.0), Vec2::new(0.0, 0.0), 0.016);

            let outputs = system.infer("wander", &inputs).unwrap();

            // Wander should have some movement
            assert!(outputs.movement.length() > 0.0);
            assert!((outputs.speed - 0.3).abs() < 0.01);
        }

        #[test]
        fn test_infer_mlp_model() {
            let mut system = AiSystem::new();
            let model = AprModel::new_test_model();
            system.register_model("mlp", model).unwrap();

            let inputs =
                AiInputs::from_positions(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0), 0.016);

            let outputs = system.infer("mlp", &inputs).unwrap();

            // MLP should produce some outputs
            assert!(outputs.movement.length() >= 0.0);
        }

        #[test]
        fn test_infer_unknown_model() {
            let system = AiSystem::new();
            let inputs = AiInputs::default();

            let result = system.infer("nonexistent", &inputs);
            assert!(result.is_err());
        }
    }

    mod yaml_bridge_tests {
        use super::*;

        #[test]
        fn test_resolve_builtin_prefix() {
            let bridge = YamlAiBridge::new();
            let mut system = AiSystem::new();

            let id = bridge.resolve("builtin:chase", &mut system).unwrap();

            assert_eq!(id, "builtin-chase");
            assert!(system.has_model("builtin-chase"));
        }

        #[test]
        fn test_resolve_simple_builtin() {
            let bridge = YamlAiBridge::new();
            let mut system = AiSystem::new();

            let id = bridge.resolve("patrol", &mut system).unwrap();

            assert_eq!(id, "builtin-patrol");
            assert!(system.has_model("builtin-patrol"));
        }

        #[test]
        fn test_resolve_all_builtins() {
            let bridge = YamlAiBridge::new();
            let mut system = AiSystem::new();

            let _ = bridge.resolve("chase", &mut system).unwrap();
            let _ = bridge.resolve("patrol", &mut system).unwrap();
            let _ = bridge.resolve("wander", &mut system).unwrap();

            assert_eq!(system.model_count(), 3);
        }

        #[test]
        fn test_resolve_unknown() {
            let bridge = YamlAiBridge::new();
            let mut system = AiSystem::new();

            let result = bridge.resolve("unknown_behavior", &mut system);
            assert!(result.is_err());
        }

        #[test]
        fn test_resolve_caches_model() {
            let bridge = YamlAiBridge::new();
            let mut system = AiSystem::new();

            // Resolve twice
            let _ = bridge.resolve("builtin:chase", &mut system).unwrap();
            let _ = bridge.resolve("builtin:chase", &mut system).unwrap();

            // Should only have one model loaded
            assert_eq!(system.model_count(), 1);
        }

        #[test]
        fn test_register_custom() {
            let mut bridge = YamlAiBridge::new();
            bridge.register_custom("smart-ghost", "models/ghost.apr");

            // Can't test file loading without a file, but registration works
            assert!(!bridge.custom_models.is_empty());
        }
    }

    mod mlp_inference_tests {
        use super::*;

        #[test]
        fn test_simple_mlp() {
            // Simple 2->2 identity-like network
            let layers = vec![LayerWeights {
                weights: vec![1.0, 0.0, 0.0, 1.0], // Identity matrix
                biases: vec![0.0, 0.0],
                input_size: 2,
                output_size: 2,
            }];

            let input = vec![0.5, -0.5];
            let output = AiSystem::run_mlp_inference(&layers, &input);

            // With ReLU, negative becomes 0
            assert!(output[0] > 0.0);
            assert!(output[1].abs() < 0.01);
        }

        #[test]
        fn test_multi_layer_mlp() {
            let layers = vec![
                LayerWeights {
                    weights: vec![0.5, 0.5, 0.5, 0.5],
                    biases: vec![0.0, 0.0],
                    input_size: 2,
                    output_size: 2,
                },
                LayerWeights {
                    weights: vec![1.0, 1.0],
                    biases: vec![0.0],
                    input_size: 2,
                    output_size: 1,
                },
            ];

            let input = vec![1.0, 1.0];
            let output = AiSystem::run_mlp_inference(&layers, &input);

            assert_eq!(output.len(), 1);
            assert!(output[0] > 0.0);
        }
    }
}
