//! # jugar-physics
//!
//! Physics engine for Jugar using Trueno backend with runtime capability detection.
//!
//! Supports three tiers:
//! - **Tier 1**: WebGPU compute shaders (10,000+ rigid bodies)
//! - **Tier 2**: WASM SIMD 128-bit
//! - **Tier 3**: Scalar fallback

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use std::time::Duration;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use jugar_core::{Position, Velocity};

/// Physics backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PhysicsBackend {
    /// WebGPU compute shaders (best performance)
    WebGpu,
    /// WASM SIMD 128-bit (good performance)
    #[default]
    WasmSimd,
    /// Scalar fallback (lowest performance)
    Scalar,
}

impl fmt::Display for PhysicsBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebGpu => write!(f, "WebGPU"),
            Self::WasmSimd => write!(f, "WASM-SIMD"),
            Self::Scalar => write!(f, "Scalar"),
        }
    }
}

/// Physics errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PhysicsError {
    /// Backend not available
    #[error("Physics backend {0} not available")]
    BackendNotAvailable(PhysicsBackend),
}

/// Result type for physics operations
pub type Result<T> = core::result::Result<T, PhysicsError>;

/// Rigid body for physics simulation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidBody {
    /// Position
    pub position: Position,
    /// Velocity
    pub velocity: Velocity,
    /// Mass (kg)
    pub mass: f32,
    /// Restitution (bounciness, 0-1)
    pub restitution: f32,
    /// Friction coefficient
    pub friction: f32,
    /// Whether the body is static (immovable)
    pub is_static: bool,
}

impl RigidBody {
    /// Creates a new dynamic rigid body
    #[must_use]
    pub fn new(position: Position) -> Self {
        Self {
            position,
            velocity: Velocity::zero(),
            mass: 1.0,
            restitution: 0.5,
            friction: 0.3,
            is_static: false,
        }
    }

    /// Creates a static rigid body
    #[must_use]
    pub fn new_static(position: Position) -> Self {
        Self {
            position,
            velocity: Velocity::zero(),
            mass: f32::INFINITY,
            restitution: 0.5,
            friction: 0.3,
            is_static: true,
        }
    }

    /// Sets the velocity
    #[must_use]
    pub const fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }

    /// Sets the mass
    #[must_use]
    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new(Position::zero())
    }
}

/// Handle to a body in the physics world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub u32);

/// Physics world containing all bodies
pub struct PhysicsWorld {
    backend: PhysicsBackend,
    bodies: Vec<RigidBody>,
    gravity: Vec2,
}

impl PhysicsWorld {
    /// Creates a new physics world with automatic backend detection
    #[must_use]
    pub fn new() -> Self {
        let backend = detect_best_backend();
        Self {
            backend,
            bodies: Vec::new(),
            gravity: Vec2::new(0.0, -9.81),
        }
    }

    /// Creates a physics world with a specific backend
    #[must_use]
    pub fn with_backend(backend: PhysicsBackend) -> Self {
        Self {
            backend,
            bodies: Vec::new(),
            gravity: Vec2::new(0.0, -9.81),
        }
    }

    /// Returns the current backend
    #[must_use]
    pub const fn backend(&self) -> PhysicsBackend {
        self.backend
    }

    /// Sets the gravity
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    /// Adds a body to the world
    pub fn add_body(&mut self, body: RigidBody) -> BodyHandle {
        let handle = BodyHandle(self.bodies.len() as u32);
        self.bodies.push(body);
        handle
    }

    /// Gets a reference to a body
    #[must_use]
    pub fn get_body(&self, handle: BodyHandle) -> Option<&RigidBody> {
        self.bodies.get(handle.0 as usize)
    }

    /// Gets a mutable reference to a body
    pub fn get_body_mut(&mut self, handle: BodyHandle) -> Option<&mut RigidBody> {
        self.bodies.get_mut(handle.0 as usize)
    }

    /// Returns the number of bodies
    #[must_use]
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Steps the physics simulation
    ///
    /// Returns the time taken for the step.
    pub fn step(&mut self, dt: f32) -> Duration {
        let start = std::time::Instant::now();

        // Apply gravity and integrate
        for body in &mut self.bodies {
            if body.is_static {
                continue;
            }

            // Apply gravity
            body.velocity.x += self.gravity.x * dt;
            body.velocity.y += self.gravity.y * dt;

            // Integrate position
            body.position.x += body.velocity.x * dt;
            body.position.y += body.velocity.y * dt;
        }

        start.elapsed()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PhysicsWorld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicsWorld")
            .field("backend", &self.backend)
            .field("body_count", &self.bodies.len())
            .finish()
    }
}

/// Detects the best available physics backend
#[must_use]
pub fn detect_best_backend() -> PhysicsBackend {
    // In WASM, we'd check for WebGPU availability
    // For now, default to SIMD
    PhysicsBackend::WasmSimd
}

/// Detects if WebGPU is available
#[must_use]
pub fn detect_webgpu() -> bool {
    // Would check navigator.gpu in WASM
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_creation() {
        let body = RigidBody::new(Position::new(10.0, 20.0));
        assert!((body.position.x - 10.0).abs() < f32::EPSILON);
        assert!(!body.is_static);
    }

    #[test]
    fn test_static_body() {
        let body = RigidBody::new_static(Position::zero());
        assert!(body.is_static);
        assert!(body.mass.is_infinite());
    }

    #[test]
    fn test_physics_world_add_body() {
        let mut world = PhysicsWorld::new();
        let handle = world.add_body(RigidBody::default());
        assert_eq!(world.body_count(), 1);
        assert!(world.get_body(handle).is_some());
    }

    #[test]
    fn test_physics_step_moves_body() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::ZERO); // No gravity for this test

        let body = RigidBody::new(Position::zero()).with_velocity(Velocity::new(10.0, 0.0));
        let handle = world.add_body(body);

        world.step(1.0);

        let body = world.get_body(handle).unwrap();
        assert!(
            (body.position.x - 10.0).abs() < f32::EPSILON,
            "Body should move by velocity"
        );
    }

    #[test]
    fn test_physics_gravity_affects_velocity() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -10.0));

        let handle = world.add_body(RigidBody::new(Position::new(0.0, 100.0)));

        world.step(1.0);

        let body = world.get_body(handle).unwrap();
        assert!(
            (body.velocity.y - (-10.0)).abs() < f32::EPSILON,
            "Gravity should affect velocity"
        );
    }

    #[test]
    fn test_static_body_not_affected_by_physics() {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -10.0));

        let handle = world.add_body(RigidBody::new_static(Position::new(0.0, 0.0)));

        world.step(1.0);

        let body = world.get_body(handle).unwrap();
        assert!(
            body.position.y.abs() < f32::EPSILON,
            "Static body should not move"
        );
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(format!("{}", PhysicsBackend::WebGpu), "WebGPU");
        assert_eq!(format!("{}", PhysicsBackend::WasmSimd), "WASM-SIMD");
    }
}
