//! Probar introspection hooks for ECS debugging and testing.
//!
//! This module is only compiled when the `probar` feature is enabled.
//! It provides read-only access to ECS internals for the Probar test runner.
//!
//! Per spec Section 14.2: "Add `#[cfg(feature = "jugar-probar")]` hooks in `jugar-core`
//! to expose ECS tables to the Probar test runner."
//!
//! # Safety
//!
//! All introspection APIs are read-only and cannot mutate game state.
//! This ensures that test instrumentation doesn't affect game behavior.

use core::any::TypeId;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Entity, GameLoop, World};

/// Introspection data for a single entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// The entity ID
    pub id: u64,
    /// Component type names attached to this entity
    pub components: Vec<String>,
}

/// Introspection data for the entire world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Total entity count
    pub entity_count: usize,
    /// Number of different component types registered
    pub component_type_count: usize,
    /// All entity snapshots
    pub entities: Vec<EntitySnapshot>,
    /// Frame number when snapshot was taken
    pub frame: u64,
}

/// Introspection data for game loop state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoopSnapshot {
    /// Current game state
    pub state: String,
    /// Current frame number
    pub frame: u64,
    /// Fixed timestep in seconds
    pub fixed_dt: f32,
    /// Accumulated time for physics
    pub accumulator: f32,
}

/// Component type registry for runtime type introspection
#[derive(Debug, Clone, Default)]
pub struct ComponentRegistry {
    /// Map from `TypeId` to human-readable name
    names: HashMap<TypeId, &'static str>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a component type with its name
    pub fn register<T: 'static>(&mut self, name: &'static str) {
        let _ = self.names.insert(TypeId::of::<T>(), name);
    }

    /// Get the name of a component type
    #[must_use]
    pub fn name_of(&self, type_id: TypeId) -> Option<&'static str> {
        self.names.get(&type_id).copied()
    }

    /// Get all registered type names
    #[must_use]
    pub fn all_names(&self) -> Vec<&'static str> {
        self.names.values().copied().collect()
    }
}

/// Trait for types that can be introspected by Probar
pub trait ProbarIntrospect {
    /// Take a snapshot of the current state
    fn snapshot(&self) -> WorldSnapshot;
}

impl ProbarIntrospect for World {
    fn snapshot(&self) -> WorldSnapshot {
        let entities: Vec<EntitySnapshot> = self
            .entities()
            .map(|entity| EntitySnapshot {
                id: entity.id(),
                components: Vec::new(), // Component names require registry
            })
            .collect();

        WorldSnapshot {
            entity_count: self.entity_count(),
            component_type_count: self.component_type_count(),
            entities,
            frame: 0, // Frame tracking requires external state
        }
    }
}

/// Extension trait for World to support probar introspection
pub trait WorldProbarExt {
    /// Get the number of registered component types
    fn component_type_count(&self) -> usize;

    /// Check if an entity has any components
    fn entity_component_count(&self, entity: Entity) -> usize;
}

impl WorldProbarExt for World {
    fn component_type_count(&self) -> usize {
        // This requires access to World's internal component HashMap
        // We'll add a method to World itself
        self.component_type_count_internal()
    }

    fn entity_component_count(&self, entity: Entity) -> usize {
        self.entity_component_count_internal(entity)
    }
}

/// Debug protocol message for WASM communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugMessage {
    /// Request a world snapshot
    RequestSnapshot,
    /// World snapshot response
    Snapshot(WorldSnapshot),
    /// Request game loop state
    RequestGameState,
    /// Game state response
    GameState(GameLoopSnapshot),
    /// Breakpoint hit notification
    BreakpointHit {
        /// Location identifier
        location: String,
        /// Frame number
        frame: u64,
    },
    /// Continue execution after breakpoint
    Continue,
    /// Step one frame
    StepFrame,
    /// Pause execution
    Pause,
    /// Resume execution
    Resume,
}

/// Debug controller for managing Probar introspection
#[derive(Debug, Clone, Default)]
pub struct DebugController {
    /// Whether debugging is currently paused
    pub paused: bool,
    /// Pending single-step request
    pub step_requested: bool,
    /// Breakpoint locations (by name/label)
    pub breakpoints: Vec<String>,
    /// Current frame number
    pub current_frame: u64,
}

impl DebugController {
    /// Create a new debug controller
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a debug message and return any response
    #[must_use]
    pub const fn process_message(&mut self, message: &DebugMessage) -> Option<DebugMessage> {
        match message {
            DebugMessage::Pause => {
                self.paused = true;
                None
            }
            DebugMessage::Resume => {
                self.paused = false;
                self.step_requested = false;
                None
            }
            DebugMessage::StepFrame => {
                self.step_requested = true;
                self.paused = true;
                None
            }
            DebugMessage::Continue => {
                self.paused = false;
                None
            }
            _ => None, // Other messages handled elsewhere
        }
    }

    /// Check if execution should proceed
    #[must_use]
    pub const fn should_execute(&mut self) -> bool {
        if self.step_requested {
            self.step_requested = false;
            return true;
        }
        !self.paused
    }

    /// Advance to next frame
    pub const fn advance_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Check if a breakpoint is set at the given location
    #[must_use]
    pub fn has_breakpoint(&self, location: &str) -> bool {
        self.breakpoints.iter().any(|bp| bp == location)
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, location: String) {
        if !self.breakpoints.contains(&location) {
            self.breakpoints.push(location);
        }
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, location: &str) {
        self.breakpoints.retain(|bp| bp != location);
    }
}

/// Take a snapshot of the game loop state
#[must_use]
pub fn snapshot_game_loop(game_loop: &GameLoop, frame: u64) -> GameLoopSnapshot {
    GameLoopSnapshot {
        state: format!("{:?}", game_loop.state()),
        frame,
        fixed_dt: game_loop.fixed_dt(),
        accumulator: game_loop.accumulator(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests for probar introspection hooks
    // ========================================================================

    mod component_registry_tests {
        use super::*;

        #[test]
        fn test_registry_creation() {
            let registry = ComponentRegistry::new();
            assert!(registry.all_names().is_empty());
        }

        #[test]
        fn test_registry_register_component() {
            let mut registry = ComponentRegistry::new();
            registry.register::<crate::Position>("Position");

            let type_id = TypeId::of::<crate::Position>();
            assert_eq!(registry.name_of(type_id), Some("Position"));
        }

        #[test]
        fn test_registry_all_names() {
            let mut registry = ComponentRegistry::new();
            registry.register::<crate::Position>("Position");
            registry.register::<crate::Velocity>("Velocity");

            let names = registry.all_names();
            assert_eq!(names.len(), 2);
            assert!(names.contains(&"Position"));
            assert!(names.contains(&"Velocity"));
        }

        #[test]
        fn test_registry_unknown_type() {
            let registry = ComponentRegistry::new();
            let type_id = TypeId::of::<String>();
            assert!(registry.name_of(type_id).is_none());
        }
    }

    mod debug_controller_tests {
        use super::*;

        #[test]
        fn test_controller_creation() {
            let controller = DebugController::new();
            assert!(!controller.paused);
            assert!(!controller.step_requested);
            assert!(controller.breakpoints.is_empty());
        }

        #[test]
        fn test_pause_resume() {
            let mut controller = DebugController::new();

            let _ = controller.process_message(&DebugMessage::Pause);
            assert!(controller.paused);
            assert!(!controller.should_execute());

            let _ = controller.process_message(&DebugMessage::Resume);
            assert!(!controller.paused);
            assert!(controller.should_execute());
        }

        #[test]
        fn test_step_frame() {
            let mut controller = DebugController::new();
            let _ = controller.process_message(&DebugMessage::Pause);

            let _ = controller.process_message(&DebugMessage::StepFrame);
            assert!(controller.should_execute()); // First call returns true
            assert!(!controller.should_execute()); // Second call returns false (still paused)
        }

        #[test]
        fn test_breakpoints() {
            let mut controller = DebugController::new();

            controller.add_breakpoint("game_loop_start".to_string());
            assert!(controller.has_breakpoint("game_loop_start"));
            assert!(!controller.has_breakpoint("physics_step"));

            controller.add_breakpoint("physics_step".to_string());
            assert!(controller.has_breakpoint("physics_step"));

            controller.remove_breakpoint("game_loop_start");
            assert!(!controller.has_breakpoint("game_loop_start"));
        }

        #[test]
        fn test_advance_frame() {
            let mut controller = DebugController::new();
            assert_eq!(controller.current_frame, 0);

            controller.advance_frame();
            assert_eq!(controller.current_frame, 1);

            controller.advance_frame();
            controller.advance_frame();
            assert_eq!(controller.current_frame, 3);
        }
    }

    mod world_snapshot_tests {
        use super::*;

        #[test]
        fn test_empty_world_snapshot() {
            let world = World::new();
            let snapshot = world.snapshot();

            assert_eq!(snapshot.entity_count, 0);
            assert!(snapshot.entities.is_empty());
        }

        #[test]
        fn test_world_snapshot_with_entities() {
            let mut world = World::new();
            let _ = world.spawn();
            let _ = world.spawn();
            let _ = world.spawn();

            let snapshot = world.snapshot();
            assert_eq!(snapshot.entity_count, 3);
            assert_eq!(snapshot.entities.len(), 3);
        }

        #[test]
        fn test_entity_snapshot_ids() {
            let mut world = World::new();
            let e1 = world.spawn();
            let e2 = world.spawn();

            let snapshot = world.snapshot();
            let ids: Vec<u64> = snapshot.entities.iter().map(|e| e.id).collect();

            assert!(ids.contains(&e1.id()));
            assert!(ids.contains(&e2.id()));
        }
    }

    mod debug_message_tests {
        use super::*;

        #[test]
        fn test_debug_message_serialization() {
            let msg = DebugMessage::RequestSnapshot;
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("RequestSnapshot"));
        }

        #[test]
        fn test_debug_message_deserialization() {
            let json = r#""RequestSnapshot""#;
            let msg: DebugMessage = serde_json::from_str(json).unwrap();
            assert!(matches!(msg, DebugMessage::RequestSnapshot));
        }

        #[test]
        fn test_breakpoint_message() {
            let msg = DebugMessage::BreakpointHit {
                location: "test_location".to_string(),
                frame: 42,
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("test_location"));
            assert!(json.contains("42"));
        }
    }

    mod game_loop_snapshot_tests {
        use super::*;

        #[test]
        fn test_game_loop_snapshot() {
            let game_loop = GameLoop::default();
            let snapshot = snapshot_game_loop(&game_loop, 100);

            assert_eq!(snapshot.frame, 100);
            assert!(snapshot.fixed_dt > 0.0);
            // GameLoopState is debug-printed, not GameState
            assert!(
                snapshot.state.contains("frame_count") || snapshot.state.contains("accumulator")
            );
        }
    }
}
