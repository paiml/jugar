//! # jugar-core
//!
//! Core ECS (Entity-Component-System), Game Loop, and State Management for Jugar.
//!
//! This crate provides the foundational data structures and systems for the Jugar
//! game engine. It follows Toyota Way principles:
//!
//! - **Poka-Yoke**: Type-safe entity/component relationships prevent invalid states
//! - **Heijunka**: Fixed timestep game loop ensures physics consistency
//! - **Mieruka**: Debug-friendly types with comprehensive Display/Debug impls
//!
//! ## Probar Integration
//!
//! When the `probar` feature is enabled, this crate exposes introspection hooks
//! for the Probar test runner. This allows debugging and testing of ECS state
//! without modifying game behavior.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;

pub mod components;
pub mod ecs;
pub mod game_loop;

/// Probar introspection hooks (only compiled with `probar` feature)
#[cfg(feature = "probar")]
pub mod introspection;

pub use components::*;
pub use ecs::*;
pub use game_loop::*;

#[cfg(feature = "probar")]
pub use introspection::*;

/// Errors that can occur in jugar-core
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Entity not found in the world
    #[error("Entity {0:?} not found")]
    EntityNotFound(Entity),

    /// Component not found on entity
    #[error("Component not found on entity {0:?}")]
    ComponentNotFound(Entity),

    /// Invalid game state transition
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        /// Current state
        from: String,
        /// Attempted target state
        to: String,
    },
}

/// Result type for jugar-core operations
pub type Result<T> = core::result::Result<T, CoreError>;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CoreError::EntityNotFound(Entity(42));
        assert!(err.to_string().contains("42"));
    }
}
