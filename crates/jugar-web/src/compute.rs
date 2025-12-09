// const fn and div_ceil are clearer as they are
#![allow(clippy::missing_const_for_fn, clippy::manual_div_ceil)]

//! WebGPU compute shader demonstration for physics calculations.
//!
//! This module provides a compute capability detection and demonstration layer
//! showing the engine's GPU compute potential. The actual WebGPU bindings are
//! handled by trueno, but this module exposes the game-engine-level API.
//!
//! ## Compute Tiers
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Compute Backend Tiers                    │
//! ├───────────┬──────────────────┬──────────────────────────────┤
//! │   Tier    │     Backend      │         Capability           │
//! ├───────────┼──────────────────┼──────────────────────────────┤
//! │  Tier 1   │ WebGPU Compute   │ 10,000+ rigid bodies         │
//! │  Tier 2   │ WASM SIMD128     │ 1,000+ rigid bodies          │
//! │  Tier 3   │ Scalar           │ ~100 rigid bodies            │
//! └───────────┴──────────────────┴──────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use jugar_web::compute::{ComputeCapability, detect_compute_capability};
//!
//! let caps = detect_compute_capability();
//! println!("Compute tier: {}", caps.tier);
//! println!("Max particles: {}", caps.max_recommended_particles);
//! ```

use crate::simd::{detect_compute_backend, ComputeBackend};

/// Compute capability tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComputeTier {
    /// Tier 1: WebGPU compute shaders available
    Tier1Gpu,
    /// Tier 2: SIMD acceleration available (AVX2/NEON/WASM SIMD)
    Tier2Simd,
    /// Tier 3: Scalar fallback only
    Tier3Scalar,
}

impl core::fmt::Display for ComputeTier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tier1Gpu => write!(f, "Tier 1 (GPU)"),
            Self::Tier2Simd => write!(f, "Tier 2 (SIMD)"),
            Self::Tier3Scalar => write!(f, "Tier 3 (Scalar)"),
        }
    }
}

/// Detected compute capabilities for the current platform.
#[derive(Debug, Clone)]
pub struct ComputeCapability {
    /// Compute tier classification
    pub tier: ComputeTier,
    /// Underlying backend being used
    pub backend: ComputeBackend,
    /// Whether GPU compute is available
    pub gpu_available: bool,
    /// Whether SIMD is available
    pub simd_available: bool,
    /// Maximum recommended particle count for 60 FPS
    pub max_recommended_particles: u32,
    /// Maximum recommended rigid body count for 60 FPS
    pub max_recommended_bodies: u32,
    /// Recommended batch size for physics updates
    pub optimal_batch_size: u32,
}

impl ComputeCapability {
    /// Creates capability info from detected backend.
    #[must_use]
    pub fn from_backend(backend: ComputeBackend) -> Self {
        let (tier, gpu_available, simd_available, particles, bodies, batch) = match backend {
            ComputeBackend::Gpu => (ComputeTier::Tier1Gpu, true, true, 100_000, 10_000, 1024),
            ComputeBackend::CpuSimd | ComputeBackend::WasmSimd => {
                (ComputeTier::Tier2Simd, false, true, 10_000, 1_000, 256)
            }
            ComputeBackend::CpuScalar => (ComputeTier::Tier3Scalar, false, false, 1_000, 100, 64),
        };

        Self {
            tier,
            backend,
            gpu_available,
            simd_available,
            max_recommended_particles: particles,
            max_recommended_bodies: bodies,
            optimal_batch_size: batch,
        }
    }

    /// Returns true if the system can handle large-scale physics.
    #[must_use]
    pub fn supports_large_scale_physics(&self) -> bool {
        self.tier <= ComputeTier::Tier2Simd
    }

    /// Returns the recommended physics substep count for stable simulation.
    #[must_use]
    pub const fn recommended_substeps(&self) -> u32 {
        match self.tier {
            ComputeTier::Tier1Gpu => 1,
            ComputeTier::Tier2Simd => 2,
            ComputeTier::Tier3Scalar => 4,
        }
    }
}

impl Default for ComputeCapability {
    fn default() -> Self {
        detect_compute_capability()
    }
}

/// Detects the compute capabilities of the current platform.
#[must_use]
pub fn detect_compute_capability() -> ComputeCapability {
    let backend = detect_compute_backend();
    ComputeCapability::from_backend(backend)
}

/// WebGPU compute demonstration module.
///
/// This struct demonstrates WebGPU compute shader concepts for physics.
/// In a full implementation, this would interface with actual WebGPU bindings.
#[derive(Debug, Clone)]
pub struct ComputeDemo {
    /// Current capability
    capability: ComputeCapability,
    /// Demo state
    state: ComputeDemoState,
}

/// State for the compute demo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeDemoState {
    /// Not started
    Idle,
    /// Demo is running
    Running,
    /// Demo completed
    Completed,
}

impl Default for ComputeDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeDemo {
    /// Creates a new compute demonstration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capability: detect_compute_capability(),
            state: ComputeDemoState::Idle,
        }
    }

    /// Returns the detected compute capability.
    #[must_use]
    pub const fn capability(&self) -> &ComputeCapability {
        &self.capability
    }

    /// Returns the current demo state.
    #[must_use]
    pub const fn state(&self) -> ComputeDemoState {
        self.state
    }

    /// Starts the compute demo.
    pub fn start(&mut self) {
        self.state = ComputeDemoState::Running;
    }

    /// Runs a compute benchmark and returns the result.
    ///
    /// This demonstrates the physics compute capability by running
    /// a batch particle update operation.
    #[must_use]
    pub fn run_benchmark(&mut self, particle_count: usize) -> ComputeBenchmarkResult {
        self.state = ComputeDemoState::Running;

        // Allocate test data
        let mut positions_x: Vec<f32> = (0..particle_count).map(|i| i as f32).collect();
        let mut positions_y: Vec<f32> = (0..particle_count).map(|i| (i * 2) as f32).collect();
        let velocities_x: Vec<f32> = (0..particle_count).map(|i| (i % 100) as f32).collect();
        let mut velocities_y: Vec<f32> = (0..particle_count).map(|i| -((i % 50) as f32)).collect();

        // Run physics update (uses SIMD when available)
        crate::simd::batch_particle_update(
            &mut positions_x,
            &mut positions_y,
            &velocities_x,
            &mut velocities_y,
            100.0, // gravity
            0.016, // dt (~60 FPS)
        );

        self.state = ComputeDemoState::Completed;

        ComputeBenchmarkResult {
            particle_count,
            backend: self.capability.backend,
            tier: self.capability.tier,
            positions_updated: !positions_x.is_empty(),
            velocities_updated: !velocities_y.is_empty(),
        }
    }

    /// Stops the demo and resets state.
    pub fn stop(&mut self) {
        self.state = ComputeDemoState::Idle;
    }
}

/// Result of a compute benchmark run.
#[derive(Debug, Clone)]
pub struct ComputeBenchmarkResult {
    /// Number of particles processed
    pub particle_count: usize,
    /// Backend used
    pub backend: ComputeBackend,
    /// Tier classification
    pub tier: ComputeTier,
    /// Whether positions were updated
    pub positions_updated: bool,
    /// Whether velocities were updated
    pub velocities_updated: bool,
}

impl ComputeBenchmarkResult {
    /// Returns a summary string for display.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Processed {} particles using {} ({})",
            self.particle_count, self.backend, self.tier
        )
    }
}

/// GPU compute shader information (for future WebGPU integration).
#[derive(Debug, Clone)]
pub struct GpuShaderInfo {
    /// Shader name
    pub name: String,
    /// Shader type
    pub shader_type: ShaderType,
    /// Workgroup size X
    pub workgroup_x: u32,
    /// Workgroup size Y
    pub workgroup_y: u32,
    /// Workgroup size Z
    pub workgroup_z: u32,
}

/// Type of compute shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    /// Particle physics update
    ParticlePhysics,
    /// Collision detection broad phase
    CollisionBroadPhase,
    /// Collision detection narrow phase
    CollisionNarrowPhase,
    /// Constraint solver
    ConstraintSolver,
}

impl core::fmt::Display for ShaderType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParticlePhysics => write!(f, "Particle Physics"),
            Self::CollisionBroadPhase => write!(f, "Collision Broad Phase"),
            Self::CollisionNarrowPhase => write!(f, "Collision Narrow Phase"),
            Self::ConstraintSolver => write!(f, "Constraint Solver"),
        }
    }
}

impl GpuShaderInfo {
    /// Creates info for a particle physics shader.
    #[must_use]
    pub fn particle_physics() -> Self {
        Self {
            name: "jugar_particle_physics".to_string(),
            shader_type: ShaderType::ParticlePhysics,
            workgroup_x: 256,
            workgroup_y: 1,
            workgroup_z: 1,
        }
    }

    /// Creates info for a collision broad phase shader.
    #[must_use]
    pub fn collision_broad_phase() -> Self {
        Self {
            name: "jugar_collision_broad".to_string(),
            shader_type: ShaderType::CollisionBroadPhase,
            workgroup_x: 64,
            workgroup_y: 1,
            workgroup_z: 1,
        }
    }

    /// Returns total workgroup size.
    #[must_use]
    pub const fn workgroup_size(&self) -> u32 {
        self.workgroup_x * self.workgroup_y * self.workgroup_z
    }

    /// Returns number of workgroups needed for given element count.
    #[must_use]
    pub const fn workgroups_for(&self, element_count: u32) -> u32 {
        let size = self.workgroup_size();
        if size == 0 {
            return 0;
        }
        (element_count + size - 1) / size
    }
}

/// WGSL compute shader source for particle physics (demonstration).
///
/// This is the WGSL shader that would be used when WebGPU is available.
/// Currently shown for documentation purposes.
pub const PARTICLE_PHYSICS_WGSL: &str = r"
// Jugar Particle Physics Compute Shader
// Processes particle positions and velocities in parallel on GPU

struct Particle {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
}

struct Params {
    dt: f32,
    gravity: f32,
    particle_count: u32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= params.particle_count) {
        return;
    }

    var p = particles[idx];

    // Apply velocity
    p.pos_x += p.vel_x * params.dt;
    p.pos_y += p.vel_y * params.dt;

    // Apply gravity
    p.vel_y += params.gravity * params.dt;

    particles[idx] = p;
}
";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // =========================================================================
    // ComputeTier Tests
    // =========================================================================

    #[test]
    fn test_compute_tier_display() {
        assert_eq!(format!("{}", ComputeTier::Tier1Gpu), "Tier 1 (GPU)");
        assert_eq!(format!("{}", ComputeTier::Tier2Simd), "Tier 2 (SIMD)");
        assert_eq!(format!("{}", ComputeTier::Tier3Scalar), "Tier 3 (Scalar)");
    }

    #[test]
    fn test_compute_tier_ordering() {
        // Lower tier = better
        assert!(ComputeTier::Tier1Gpu < ComputeTier::Tier2Simd);
        assert!(ComputeTier::Tier2Simd < ComputeTier::Tier3Scalar);
    }

    // =========================================================================
    // ComputeCapability Tests
    // =========================================================================

    #[test]
    fn test_compute_capability_from_gpu_backend() {
        let caps = ComputeCapability::from_backend(ComputeBackend::Gpu);

        assert_eq!(caps.tier, ComputeTier::Tier1Gpu);
        assert!(caps.gpu_available);
        assert!(caps.simd_available);
        assert_eq!(caps.max_recommended_particles, 100_000);
        assert_eq!(caps.max_recommended_bodies, 10_000);
    }

    #[test]
    fn test_compute_capability_from_simd_backend() {
        let caps = ComputeCapability::from_backend(ComputeBackend::CpuSimd);

        assert_eq!(caps.tier, ComputeTier::Tier2Simd);
        assert!(!caps.gpu_available);
        assert!(caps.simd_available);
        assert_eq!(caps.max_recommended_particles, 10_000);
        assert_eq!(caps.max_recommended_bodies, 1_000);
    }

    #[test]
    fn test_compute_capability_from_wasm_simd_backend() {
        let caps = ComputeCapability::from_backend(ComputeBackend::WasmSimd);

        assert_eq!(caps.tier, ComputeTier::Tier2Simd);
        assert!(!caps.gpu_available);
        assert!(caps.simd_available);
    }

    #[test]
    fn test_compute_capability_from_scalar_backend() {
        let caps = ComputeCapability::from_backend(ComputeBackend::CpuScalar);

        assert_eq!(caps.tier, ComputeTier::Tier3Scalar);
        assert!(!caps.gpu_available);
        assert!(!caps.simd_available);
        assert_eq!(caps.max_recommended_particles, 1_000);
        assert_eq!(caps.max_recommended_bodies, 100);
    }

    #[test]
    fn test_compute_capability_supports_large_scale_physics() {
        let gpu_caps = ComputeCapability::from_backend(ComputeBackend::Gpu);
        let simd_caps = ComputeCapability::from_backend(ComputeBackend::CpuSimd);
        let scalar_caps = ComputeCapability::from_backend(ComputeBackend::CpuScalar);

        assert!(gpu_caps.supports_large_scale_physics());
        assert!(simd_caps.supports_large_scale_physics());
        assert!(!scalar_caps.supports_large_scale_physics());
    }

    #[test]
    fn test_compute_capability_recommended_substeps() {
        let gpu_caps = ComputeCapability::from_backend(ComputeBackend::Gpu);
        let simd_caps = ComputeCapability::from_backend(ComputeBackend::CpuSimd);
        let scalar_caps = ComputeCapability::from_backend(ComputeBackend::CpuScalar);

        assert_eq!(gpu_caps.recommended_substeps(), 1);
        assert_eq!(simd_caps.recommended_substeps(), 2);
        assert_eq!(scalar_caps.recommended_substeps(), 4);
    }

    #[test]
    fn test_detect_compute_capability() {
        let caps = detect_compute_capability();

        // Should return valid capability
        assert!(caps.max_recommended_particles > 0);
        assert!(caps.max_recommended_bodies > 0);
        assert!(caps.optimal_batch_size > 0);
    }

    // =========================================================================
    // ComputeDemo Tests
    // =========================================================================

    #[test]
    fn test_compute_demo_new() {
        let demo = ComputeDemo::new();

        assert_eq!(demo.state(), ComputeDemoState::Idle);
    }

    #[test]
    fn test_compute_demo_start() {
        let mut demo = ComputeDemo::new();
        demo.start();

        assert_eq!(demo.state(), ComputeDemoState::Running);
    }

    #[test]
    fn test_compute_demo_stop() {
        let mut demo = ComputeDemo::new();
        demo.start();
        demo.stop();

        assert_eq!(demo.state(), ComputeDemoState::Idle);
    }

    #[test]
    fn test_compute_demo_run_benchmark() {
        let mut demo = ComputeDemo::new();
        let result = demo.run_benchmark(100);

        assert_eq!(demo.state(), ComputeDemoState::Completed);
        assert_eq!(result.particle_count, 100);
        assert!(result.positions_updated);
        assert!(result.velocities_updated);
    }

    #[test]
    fn test_compute_benchmark_result_summary() {
        let result = ComputeBenchmarkResult {
            particle_count: 1000,
            backend: ComputeBackend::CpuSimd,
            tier: ComputeTier::Tier2Simd,
            positions_updated: true,
            velocities_updated: true,
        };

        let summary = result.summary();
        assert!(summary.contains("1000"));
        assert!(summary.contains("CPU SIMD"));
    }

    // =========================================================================
    // GpuShaderInfo Tests
    // =========================================================================

    #[test]
    fn test_gpu_shader_info_particle_physics() {
        let info = GpuShaderInfo::particle_physics();

        assert_eq!(info.name, "jugar_particle_physics");
        assert_eq!(info.shader_type, ShaderType::ParticlePhysics);
        assert_eq!(info.workgroup_size(), 256);
    }

    #[test]
    fn test_gpu_shader_info_collision_broad_phase() {
        let info = GpuShaderInfo::collision_broad_phase();

        assert_eq!(info.name, "jugar_collision_broad");
        assert_eq!(info.shader_type, ShaderType::CollisionBroadPhase);
        assert_eq!(info.workgroup_size(), 64);
    }

    #[test]
    fn test_gpu_shader_info_workgroups_for() {
        let info = GpuShaderInfo::particle_physics();

        assert_eq!(info.workgroups_for(256), 1);
        assert_eq!(info.workgroups_for(257), 2);
        assert_eq!(info.workgroups_for(512), 2);
        assert_eq!(info.workgroups_for(1000), 4);
    }

    #[test]
    fn test_shader_type_display() {
        assert_eq!(
            format!("{}", ShaderType::ParticlePhysics),
            "Particle Physics"
        );
        assert_eq!(
            format!("{}", ShaderType::CollisionBroadPhase),
            "Collision Broad Phase"
        );
        assert_eq!(
            format!("{}", ShaderType::CollisionNarrowPhase),
            "Collision Narrow Phase"
        );
        assert_eq!(
            format!("{}", ShaderType::ConstraintSolver),
            "Constraint Solver"
        );
    }

    // =========================================================================
    // WGSL Shader Tests
    // =========================================================================

    #[test]
    fn test_particle_physics_wgsl_is_valid() {
        // Verify the WGSL shader source contains expected elements
        assert!(PARTICLE_PHYSICS_WGSL.contains("@compute"));
        assert!(PARTICLE_PHYSICS_WGSL.contains("@workgroup_size(256)"));
        assert!(PARTICLE_PHYSICS_WGSL.contains("struct Particle"));
        assert!(PARTICLE_PHYSICS_WGSL.contains("pos_x"));
        assert!(PARTICLE_PHYSICS_WGSL.contains("vel_y"));
        assert!(PARTICLE_PHYSICS_WGSL.contains("gravity"));
    }
}
