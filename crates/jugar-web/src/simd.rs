//! SIMD-accelerated operations using trueno.
//!
//! This module provides SIMD-optimized computations for game physics and
//! particle systems using the trueno compute library.
//!
//! ## Backend Selection
//!
//! Trueno automatically selects the best available backend at runtime:
//! - **Tier 1**: WebGPU compute shaders (if available)
//! - **Tier 2**: WASM SIMD 128-bit
//! - **Tier 3**: Scalar fallback
//!
//! ## Usage
//!
//! ```rust,ignore
//! use jugar_web::simd::{SimdVec2, batch_distance_squared};
//!
//! // Create SIMD-optimized 2D vectors
//! let positions = vec![SimdVec2::new(10.0, 20.0), SimdVec2::new(30.0, 40.0)];
//! let target = SimdVec2::new(0.0, 0.0);
//!
//! // Batch compute distances (SIMD accelerated)
//! let distances = batch_distance_squared(&positions, target);
//! ```

// mul_add is less readable for dot product
#![allow(clippy::suboptimal_flops, clippy::missing_const_for_fn)]

use trueno::{Backend, Vector};

/// A SIMD-optimized 2D vector using trueno.
#[derive(Debug, Clone)]
pub struct SimdVec2 {
    data: Vector<f32>,
}

impl SimdVec2 {
    /// Creates a new 2D vector.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            data: Vector::from_slice(&[x, y]),
        }
    }

    /// Returns the X component.
    #[must_use]
    pub fn x(&self) -> f32 {
        self.data.as_slice().first().copied().unwrap_or(0.0)
    }

    /// Returns the Y component.
    #[must_use]
    pub fn y(&self) -> f32 {
        self.data.as_slice().get(1).copied().unwrap_or(0.0)
    }

    /// Computes the squared magnitude (avoids sqrt for performance).
    #[must_use]
    pub fn magnitude_squared(&self) -> f32 {
        let x = self.x();
        let y = self.y();
        x * x + y * y
    }

    /// Computes the magnitude (length).
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    /// Adds another vector using SIMD-accelerated operation.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        let result = self.data.add(&other.data).unwrap_or_else(|_| {
            // Fallback for mismatched sizes
            Vector::from_slice(&[self.x() + other.x(), self.y() + other.y()])
        });
        Self { data: result }
    }

    /// Subtracts another vector using SIMD-accelerated operation.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        let result = self.data.sub(&other.data).unwrap_or_else(|_| {
            // Fallback for mismatched sizes
            Vector::from_slice(&[self.x() - other.x(), self.y() - other.y()])
        });
        Self { data: result }
    }

    /// Multiplies by a scalar using SIMD-accelerated operation.
    #[must_use]
    pub fn scale(&self, scalar: f32) -> Self {
        let result = self.data.scale(scalar).unwrap_or_else(|_| {
            // Fallback
            Vector::from_slice(&[self.x() * scalar, self.y() * scalar])
        });
        Self { data: result }
    }

    /// Computes dot product with another vector.
    #[must_use]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x() * other.x() + self.y() * other.y()
    }
}

impl Default for SimdVec2 {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Batch-computes squared distances from a set of positions to a target.
///
/// This uses SIMD acceleration when available for improved performance
/// with large numbers of positions (e.g., particle systems).
///
/// # Arguments
///
/// * `positions` - Slice of position vectors
/// * `target` - Target position to measure distance from
///
/// # Returns
///
/// Vector of squared distances (one per position)
#[must_use]
pub fn batch_distance_squared(positions: &[SimdVec2], target: &SimdVec2) -> Vec<f32> {
    positions
        .iter()
        .map(|pos| {
            let diff = pos.sub(target);
            diff.magnitude_squared()
        })
        .collect()
}

/// SIMD-accelerated particle position update.
///
/// Updates particle positions based on velocities using batch operations.
///
/// # Arguments
///
/// * `positions` - Mutable slice of position X/Y pairs (interleaved)
/// * `velocities` - Slice of velocity X/Y pairs (interleaved)
/// * `dt` - Delta time in seconds
///
/// # Safety
///
/// Positions and velocities slices must have equal length.
pub fn batch_update_positions(positions: &mut [f32], velocities: &[f32], dt: f32) {
    if positions.len() != velocities.len() {
        return;
    }

    // Create trueno vectors for batch operation
    let pos_vec = Vector::from_slice(positions);
    let vel_vec = Vector::from_slice(velocities);

    // SIMD-accelerated: positions += velocities * dt
    if let Ok(scaled_vel) = vel_vec.scale(dt) {
        if let Ok(new_pos) = pos_vec.add(&scaled_vel) {
            // Copy results back
            let result_slice = new_pos.as_slice();
            for (i, val) in result_slice.iter().enumerate() {
                if i < positions.len() {
                    positions[i] = *val;
                }
            }
        }
    }
}

/// SIMD-accelerated batch particle physics update.
///
/// Updates positions, applies gravity, and returns updated velocities.
///
/// # Arguments
///
/// * `positions_x` - Particle X positions
/// * `positions_y` - Particle Y positions
/// * `velocities_x` - Particle X velocities
/// * `velocities_y` - Particle Y velocities
/// * `gravity` - Gravity acceleration
/// * `dt` - Delta time in seconds
pub fn batch_particle_update(
    positions_x: &mut [f32],
    positions_y: &mut [f32],
    velocities_x: &[f32],
    velocities_y: &mut [f32],
    gravity: f32,
    dt: f32,
) {
    let n = positions_x.len();
    if n == 0 || positions_y.len() != n || velocities_x.len() != n || velocities_y.len() != n {
        return;
    }

    // Create trueno vectors
    let pos_x = Vector::from_slice(positions_x);
    let pos_y = Vector::from_slice(positions_y);
    let vel_x = Vector::from_slice(velocities_x);
    let vel_y = Vector::from_slice(velocities_y);

    // SIMD-accelerated position update: pos += vel * dt
    if let Ok(scaled_vx) = vel_x.scale(dt) {
        if let Ok(new_pos_x) = pos_x.add(&scaled_vx) {
            for (i, &val) in new_pos_x.as_slice().iter().enumerate() {
                if i < positions_x.len() {
                    positions_x[i] = val;
                }
            }
        }
    }

    if let Ok(scaled_vy) = vel_y.scale(dt) {
        if let Ok(new_pos_y) = pos_y.add(&scaled_vy) {
            for (i, &val) in new_pos_y.as_slice().iter().enumerate() {
                if i < positions_y.len() {
                    positions_y[i] = val;
                }
            }
        }
    }

    // Apply gravity to Y velocities
    let gravity_delta = gravity * dt;
    for vy in velocities_y.iter_mut() {
        *vy += gravity_delta;
    }
}

/// Computes collision detection for a ball against multiple paddles.
///
/// Returns the index of the first paddle that collides, if any.
///
/// # Arguments
///
/// * `ball_x` - Ball X position
/// * `ball_y` - Ball Y position
/// * `ball_radius` - Ball radius
/// * `paddle_xs` - Paddle X positions
/// * `paddle_ys` - Paddle Y positions
/// * `paddle_heights` - Paddle heights
/// * `paddle_widths` - Paddle widths
///
/// # Returns
///
/// Index of colliding paddle, or None if no collision
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn check_paddle_collisions(
    ball_x: f32,
    ball_y: f32,
    ball_radius: f32,
    paddle_xs: &[f32],
    paddle_ys: &[f32],
    paddle_heights: &[f32],
    paddle_widths: &[f32],
) -> Option<usize> {
    let n = paddle_xs.len();
    if n == 0 || paddle_ys.len() != n || paddle_heights.len() != n || paddle_widths.len() != n {
        return None;
    }

    // Create trueno vector for batch ball X subtraction
    let ball_x_vec = Vector::from_slice(&vec![ball_x; n]);
    let paddle_x_vec = Vector::from_slice(paddle_xs);

    // SIMD-accelerated: compute all X distances at once
    let x_distances = ball_x_vec.sub(&paddle_x_vec).ok()?;
    let x_dist_slice = x_distances.as_slice();

    // Check each paddle for collision
    for i in 0..n {
        let x_dist = x_dist_slice.get(i).copied().unwrap_or(f32::MAX).abs();
        let half_width = paddle_widths.get(i).copied().unwrap_or(0.0) / 2.0;
        let half_height = paddle_heights.get(i).copied().unwrap_or(0.0) / 2.0;

        // X axis collision check
        if x_dist < half_width + ball_radius {
            // Y axis collision check
            let y_dist = (ball_y - paddle_ys.get(i).copied().unwrap_or(0.0)).abs();
            if y_dist < half_height + ball_radius {
                return Some(i);
            }
        }
    }

    None
}

/// Information about the compute backend being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeBackend {
    /// CPU scalar (no SIMD)
    CpuScalar,
    /// CPU SIMD (SSE/AVX/NEON)
    CpuSimd,
    /// WebAssembly SIMD128
    WasmSimd,
    /// GPU compute (WebGPU/Vulkan/Metal)
    Gpu,
}

impl core::fmt::Display for ComputeBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CpuScalar => write!(f, "CPU Scalar"),
            Self::CpuSimd => write!(f, "CPU SIMD"),
            Self::WasmSimd => write!(f, "WASM SIMD128"),
            Self::Gpu => write!(f, "GPU Compute"),
        }
    }
}

/// Converts trueno Backend to our ComputeBackend enum.
#[must_use]
pub fn trueno_backend_to_compute_backend(backend: Backend) -> ComputeBackend {
    match backend {
        Backend::Scalar => ComputeBackend::CpuScalar,
        Backend::SSE2 | Backend::AVX | Backend::AVX2 | Backend::AVX512 | Backend::NEON => {
            ComputeBackend::CpuSimd
        }
        Backend::WasmSIMD => ComputeBackend::WasmSimd,
        Backend::GPU => ComputeBackend::Gpu,
        Backend::Auto => ComputeBackend::CpuSimd, // Auto typically selects SIMD
    }
}

/// Detects the best available compute backend.
///
/// This queries trueno's runtime backend selection to determine
/// what SIMD capabilities are available.
#[must_use]
pub fn detect_compute_backend() -> ComputeBackend {
    let test_vec = Vector::<f32>::from_slice(&[1.0, 2.0, 3.0, 4.0]);
    trueno_backend_to_compute_backend(test_vec.backend())
}

/// Benchmark result for SIMD operations.
#[derive(Debug, Clone)]
pub struct SimdBenchmark {
    /// Operation name
    pub operation: String,
    /// Number of elements processed
    pub element_count: usize,
    /// Backend used
    pub backend: ComputeBackend,
    /// Whether SIMD acceleration was applied
    pub simd_accelerated: bool,
}

impl SimdBenchmark {
    /// Creates a new benchmark result.
    #[must_use]
    pub fn new(operation: &str, element_count: usize) -> Self {
        Self {
            operation: operation.to_string(),
            element_count,
            backend: detect_compute_backend(),
            simd_accelerated: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]
mod tests {
    use super::*;

    // =========================================================================
    // SimdVec2 Tests
    // =========================================================================

    #[test]
    fn test_simd_vec2_new() {
        let v = SimdVec2::new(3.0, 4.0);
        assert!((v.x() - 3.0).abs() < 0.001);
        assert!((v.y() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_default() {
        let v = SimdVec2::default();
        assert!((v.x()).abs() < 0.001);
        assert!((v.y()).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_magnitude_squared() {
        let v = SimdVec2::new(3.0, 4.0);
        assert!((v.magnitude_squared() - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_magnitude() {
        let v = SimdVec2::new(3.0, 4.0);
        assert!((v.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_add() {
        let a = SimdVec2::new(1.0, 2.0);
        let b = SimdVec2::new(3.0, 4.0);
        let c = a.add(&b);
        assert!((c.x() - 4.0).abs() < 0.001);
        assert!((c.y() - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_sub() {
        let a = SimdVec2::new(5.0, 7.0);
        let b = SimdVec2::new(2.0, 3.0);
        let c = a.sub(&b);
        assert!((c.x() - 3.0).abs() < 0.001);
        assert!((c.y() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_scale() {
        let v = SimdVec2::new(2.0, 3.0);
        let s = v.scale(2.0);
        assert!((s.x() - 4.0).abs() < 0.001);
        assert!((s.y() - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_vec2_dot() {
        let a = SimdVec2::new(1.0, 2.0);
        let b = SimdVec2::new(3.0, 4.0);
        assert!((a.dot(&b) - 11.0).abs() < 0.001);
    }

    // =========================================================================
    // Batch Operations Tests
    // =========================================================================

    #[test]
    fn test_batch_distance_squared() {
        let positions = vec![
            SimdVec2::new(3.0, 0.0),
            SimdVec2::new(0.0, 4.0),
            SimdVec2::new(3.0, 4.0),
        ];
        let target = SimdVec2::new(0.0, 0.0);

        let distances = batch_distance_squared(&positions, &target);

        assert_eq!(distances.len(), 3);
        assert!((distances[0] - 9.0).abs() < 0.001); // 3^2
        assert!((distances[1] - 16.0).abs() < 0.001); // 4^2
        assert!((distances[2] - 25.0).abs() < 0.001); // 3^2 + 4^2
    }

    #[test]
    fn test_batch_distance_squared_empty() {
        let positions: Vec<SimdVec2> = vec![];
        let target = SimdVec2::new(0.0, 0.0);

        let distances = batch_distance_squared(&positions, &target);

        assert!(distances.is_empty());
    }

    #[test]
    fn test_batch_update_positions() {
        let mut positions = vec![0.0, 0.0, 10.0, 10.0]; // Two 2D positions
        let velocities = vec![100.0, 200.0, -50.0, -100.0];
        let dt = 0.1;

        batch_update_positions(&mut positions, &velocities, dt);

        assert!((positions[0] - 10.0).abs() < 0.001);
        assert!((positions[1] - 20.0).abs() < 0.001);
        assert!((positions[2] - 5.0).abs() < 0.001);
        assert!((positions[3] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_batch_update_positions_mismatched_lengths() {
        let mut positions = vec![0.0, 0.0];
        let velocities = vec![100.0]; // Mismatched length

        // Should not panic, just skip
        batch_update_positions(&mut positions, &velocities, 0.1);

        // Positions unchanged
        assert_eq!(positions[0], 0.0);
    }

    #[test]
    fn test_batch_particle_update() {
        let mut pos_x = vec![0.0, 10.0];
        let mut pos_y = vec![100.0, 200.0];
        let vel_x = vec![50.0, -25.0];
        let mut vel_y = vec![0.0, 10.0];
        let gravity = 100.0;
        let dt = 0.1;

        batch_particle_update(&mut pos_x, &mut pos_y, &vel_x, &mut vel_y, gravity, dt);

        // Check X positions updated
        assert!((pos_x[0] - 5.0).abs() < 0.01); // 0 + 50 * 0.1
        assert!((pos_x[1] - 7.5).abs() < 0.01); // 10 + (-25) * 0.1

        // Check Y positions updated
        assert!((pos_y[0] - 100.0).abs() < 0.01); // 100 + 0 * 0.1
        assert!((pos_y[1] - 201.0).abs() < 0.01); // 200 + 10 * 0.1

        // Check gravity applied to velocities
        assert!((vel_y[0] - 10.0).abs() < 0.01); // 0 + 100 * 0.1
        assert!((vel_y[1] - 20.0).abs() < 0.01); // 10 + 100 * 0.1
    }

    // =========================================================================
    // Collision Detection Tests
    // =========================================================================

    #[test]
    fn test_check_paddle_collisions_hit() {
        let paddle_xs = vec![50.0, 750.0];
        let paddle_ys = vec![300.0, 300.0];
        let paddle_heights = vec![100.0, 100.0];
        let paddle_widths = vec![20.0, 20.0];

        // Ball near left paddle
        let result = check_paddle_collisions(
            55.0,
            300.0,
            10.0,
            &paddle_xs,
            &paddle_ys,
            &paddle_heights,
            &paddle_widths,
        );

        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_check_paddle_collisions_miss() {
        let paddle_xs = vec![50.0, 750.0];
        let paddle_ys = vec![300.0, 300.0];
        let paddle_heights = vec![100.0, 100.0];
        let paddle_widths = vec![20.0, 20.0];

        // Ball in center, no collision
        let result = check_paddle_collisions(
            400.0,
            300.0,
            10.0,
            &paddle_xs,
            &paddle_ys,
            &paddle_heights,
            &paddle_widths,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_check_paddle_collisions_empty() {
        let result = check_paddle_collisions(400.0, 300.0, 10.0, &[], &[], &[], &[]);

        assert!(result.is_none());
    }

    // =========================================================================
    // Backend Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_compute_backend() {
        let backend = detect_compute_backend();
        // Should be one of the valid backends
        assert!(matches!(
            backend,
            ComputeBackend::CpuScalar
                | ComputeBackend::CpuSimd
                | ComputeBackend::WasmSimd
                | ComputeBackend::Gpu
        ));
    }

    #[test]
    fn test_compute_backend_display() {
        assert_eq!(format!("{}", ComputeBackend::CpuScalar), "CPU Scalar");
        assert_eq!(format!("{}", ComputeBackend::CpuSimd), "CPU SIMD");
        assert_eq!(format!("{}", ComputeBackend::WasmSimd), "WASM SIMD128");
        assert_eq!(format!("{}", ComputeBackend::Gpu), "GPU Compute");
    }

    #[test]
    fn test_simd_benchmark_new() {
        let bench = SimdBenchmark::new("particle_update", 1000);

        assert_eq!(bench.operation, "particle_update");
        assert_eq!(bench.element_count, 1000);
        assert!(bench.simd_accelerated);
    }

    #[test]
    fn test_trueno_backend_to_compute_backend() {
        assert_eq!(
            trueno_backend_to_compute_backend(Backend::Scalar),
            ComputeBackend::CpuScalar
        );
        assert_eq!(
            trueno_backend_to_compute_backend(Backend::SSE2),
            ComputeBackend::CpuSimd
        );
        assert_eq!(
            trueno_backend_to_compute_backend(Backend::AVX2),
            ComputeBackend::CpuSimd
        );
        assert_eq!(
            trueno_backend_to_compute_backend(Backend::WasmSIMD),
            ComputeBackend::WasmSimd
        );
        assert_eq!(
            trueno_backend_to_compute_backend(Backend::GPU),
            ComputeBackend::Gpu
        );
    }
}
