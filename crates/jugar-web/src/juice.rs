//! Game juice and visual feedback effects.
//!
//! This module implements "game feel" enhancements based on Steve Swink's framework:
//! - Screen shake on goals and impacts
//! - Ball trail effects
//! - Hit confirmation feedback (flash effects)
//! - Score popup animations
//!
//! Reference: Swink, S. (2008). "Game Feel: A Game Designer's Guide to Virtual Sensation"

// const fn with mutable references is not yet stable; mul_add less readable here
#![allow(clippy::missing_const_for_fn, clippy::suboptimal_flops)]

use serde::{Deserialize, Serialize};

/// Screen shake effect state.
#[derive(Debug, Clone, Default)]
pub struct ScreenShake {
    /// Current shake intensity (decays over time)
    intensity: f32,
    /// Shake duration remaining in seconds
    duration: f32,
    /// Current shake offset X
    offset_x: f32,
    /// Current shake offset Y
    offset_y: f32,
    /// Random seed for deterministic shake pattern
    seed: u64,
}

impl ScreenShake {
    /// Creates a new screen shake controller.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            seed: 12345,
        }
    }

    /// Triggers a screen shake effect.
    ///
    /// # Arguments
    ///
    /// * `intensity` - Maximum shake offset in pixels (e.g., 10.0 for goal, 3.0 for hit)
    /// * `duration` - How long the shake lasts in seconds
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        // Only override if new shake is stronger
        if intensity > self.intensity {
            self.intensity = intensity;
            self.duration = duration;
        }
    }

    /// Updates the shake effect and returns current offset.
    ///
    /// # Arguments
    ///
    /// * `dt` - Delta time in seconds
    ///
    /// # Returns
    ///
    /// Tuple of (offset_x, offset_y) to apply to camera/rendering
    pub fn update(&mut self, dt: f32) -> (f32, f32) {
        if self.duration <= 0.0 {
            self.offset_x = 0.0;
            self.offset_y = 0.0;
            return (0.0, 0.0);
        }

        self.duration -= dt;

        // Decay intensity over time (exponential decay)
        let decay = (self.duration * 10.0).min(1.0);
        let current_intensity = self.intensity * decay;

        // Generate pseudo-random shake using simple xorshift
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;

        let rand_x = (self.seed as f32 / u64::MAX as f32) * 2.0 - 1.0;

        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;

        let rand_y = (self.seed as f32 / u64::MAX as f32) * 2.0 - 1.0;

        self.offset_x = rand_x * current_intensity;
        self.offset_y = rand_y * current_intensity;

        (self.offset_x, self.offset_y)
    }

    /// Returns true if shake is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.duration > 0.0
    }

    /// Returns current shake offsets.
    #[must_use]
    pub const fn offset(&self) -> (f32, f32) {
        (self.offset_x, self.offset_y)
    }

    /// Resets the shake state.
    pub fn reset(&mut self) {
        self.intensity = 0.0;
        self.duration = 0.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
}

/// A single point in a ball trail.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrailPoint {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Age in seconds (0 = newest)
    pub age: f32,
    /// Whether this point is active
    pub active: bool,
}

/// Ball trail effect for motion visualization.
#[derive(Debug, Clone)]
pub struct BallTrail {
    /// Trail points (ring buffer)
    points: Vec<TrailPoint>,
    /// Current write index
    write_index: usize,
    /// Time since last point was added
    time_since_last: f32,
    /// Interval between trail points
    interval: f32,
    /// Maximum age before point fades
    max_age: f32,
}

impl Default for BallTrail {
    fn default() -> Self {
        Self::new(10, 0.016, 0.15)
    }
}

impl BallTrail {
    /// Creates a new ball trail.
    ///
    /// # Arguments
    ///
    /// * `max_points` - Maximum number of trail points
    /// * `interval` - Time between adding new points (seconds)
    /// * `max_age` - How long points last before fading (seconds)
    #[must_use]
    pub fn new(max_points: usize, interval: f32, max_age: f32) -> Self {
        Self {
            points: vec![TrailPoint::default(); max_points],
            write_index: 0,
            time_since_last: 0.0,
            interval,
            max_age,
        }
    }

    /// Updates the trail with the current ball position.
    ///
    /// # Arguments
    ///
    /// * `x` - Ball X position
    /// * `y` - Ball Y position
    /// * `dt` - Delta time in seconds
    pub fn update(&mut self, x: f32, y: f32, dt: f32) {
        // Age existing points
        for point in &mut self.points {
            if point.active {
                point.age += dt;
                if point.age > self.max_age {
                    point.active = false;
                }
            }
        }

        // Add new point at interval
        self.time_since_last += dt;
        if self.time_since_last >= self.interval {
            self.time_since_last = 0.0;

            self.points[self.write_index] = TrailPoint {
                x,
                y,
                age: 0.0,
                active: true,
            };

            self.write_index = (self.write_index + 1) % self.points.len();
        }
    }

    /// Returns active trail points for rendering.
    ///
    /// Points are returned with alpha values based on age (1.0 = new, 0.0 = old).
    #[must_use]
    pub fn get_points(&self) -> Vec<(f32, f32, f32)> {
        self.points
            .iter()
            .filter(|p| p.active)
            .map(|p| {
                let alpha = 1.0 - (p.age / self.max_age).min(1.0);
                (p.x, p.y, alpha)
            })
            .collect()
    }

    /// Clears all trail points.
    pub fn clear(&mut self) {
        for point in &mut self.points {
            point.active = false;
        }
        self.time_since_last = 0.0;
    }

    /// Returns the number of active points.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.points.iter().filter(|p| p.active).count()
    }
}

/// Hit flash effect for paddle collision feedback.
#[derive(Debug, Clone, Default)]
pub struct HitFlash {
    /// Flash duration remaining
    duration: f32,
    /// Flash intensity (1.0 = full white)
    intensity: f32,
    /// Which paddle flashed (true = right, false = left)
    right_paddle: bool,
}

impl HitFlash {
    /// Creates a new hit flash controller.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            duration: 0.0,
            intensity: 0.0,
            right_paddle: false,
        }
    }

    /// Triggers a hit flash on a paddle.
    ///
    /// # Arguments
    ///
    /// * `right_paddle` - True for right paddle, false for left
    /// * `intensity` - Flash brightness (0.0-1.0)
    /// * `duration` - How long the flash lasts
    pub fn trigger(&mut self, right_paddle: bool, intensity: f32, duration: f32) {
        self.right_paddle = right_paddle;
        self.intensity = intensity;
        self.duration = duration;
    }

    /// Updates the flash effect.
    ///
    /// # Arguments
    ///
    /// * `dt` - Delta time in seconds
    ///
    /// # Returns
    ///
    /// Current flash state: (is_active, is_right_paddle, intensity)
    pub fn update(&mut self, dt: f32) -> (bool, bool, f32) {
        if self.duration <= 0.0 {
            return (false, false, 0.0);
        }

        self.duration -= dt;

        // Linear decay
        let current_intensity = self.intensity * (self.duration * 20.0).min(1.0);

        (true, self.right_paddle, current_intensity)
    }

    /// Returns true if flash is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.duration > 0.0
    }

    /// Resets the flash state.
    pub fn reset(&mut self) {
        self.duration = 0.0;
        self.intensity = 0.0;
    }

    /// Returns flash state for rendering.
    ///
    /// # Returns
    ///
    /// Tuple of (left_flash_active, right_flash_active, intensity)
    #[must_use]
    pub fn flash_state(&self) -> (bool, bool, f32) {
        if self.duration <= 0.0 {
            return (false, false, 0.0);
        }
        let current_intensity = self.intensity * (self.duration * 20.0).min(1.0);
        if self.right_paddle {
            (false, true, current_intensity)
        } else {
            (true, false, current_intensity)
        }
    }
}

/// A single particle in the particle system.
#[derive(Debug, Clone, Copy, Default)]
pub struct Particle {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// X velocity
    pub vx: f32,
    /// Y velocity
    pub vy: f32,
    /// Lifetime remaining in seconds
    pub lifetime: f32,
    /// Initial lifetime (for alpha calculation)
    pub initial_lifetime: f32,
    /// Particle size (radius)
    pub size: f32,
    /// Color RGB (packed as u32: 0xRRGGBB)
    pub color: u32,
    /// Whether this particle is active
    pub active: bool,
}

impl Particle {
    /// Creates a new particle.
    #[must_use]
    pub const fn new(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        lifetime: f32,
        size: f32,
        color: u32,
    ) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            lifetime,
            initial_lifetime: lifetime,
            size,
            color,
            active: true,
        }
    }

    /// Updates the particle position and lifetime.
    ///
    /// # Returns
    ///
    /// True if particle is still alive
    pub fn update(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }

        self.lifetime -= dt;
        if self.lifetime <= 0.0 {
            self.active = false;
            return false;
        }

        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Apply gravity (subtle downward pull)
        self.vy += 200.0 * dt;

        // Shrink over time
        let life_ratio = self.lifetime / self.initial_lifetime;
        self.size *= 0.99 + 0.01 * life_ratio;

        true
    }

    /// Returns the current alpha value based on remaining lifetime.
    #[must_use]
    pub fn alpha(&self) -> f32 {
        if self.initial_lifetime <= 0.0 {
            return 0.0;
        }
        (self.lifetime / self.initial_lifetime).clamp(0.0, 1.0)
    }

    /// Returns the RGB color components.
    #[must_use]
    pub const fn rgb(&self) -> (f32, f32, f32) {
        let r = ((self.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((self.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (self.color & 0xFF) as f32 / 255.0;
        (r, g, b)
    }
}

/// Particle system for visual effects.
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    /// Pool of particles (pre-allocated)
    particles: Vec<Particle>,
    /// Next particle index to write
    write_index: usize,
    /// Random seed for variation
    seed: u64,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(200) // Default pool size
    }
}

impl ParticleSystem {
    /// Creates a new particle system with the given pool size.
    #[must_use]
    pub fn new(pool_size: usize) -> Self {
        Self {
            particles: vec![Particle::default(); pool_size],
            write_index: 0,
            seed: 42,
        }
    }

    /// Generates a pseudo-random f32 in [0, 1).
    fn random(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed as f32) / (u64::MAX as f32)
    }

    /// Generates a pseudo-random f32 in [-1, 1).
    fn random_signed(&mut self) -> f32 {
        self.random() * 2.0 - 1.0
    }

    /// Spawns particles at the given position.
    ///
    /// # Arguments
    ///
    /// * `x` - X position
    /// * `y` - Y position
    /// * `count` - Number of particles to spawn
    /// * `speed` - Base speed of particles
    /// * `lifetime` - Base lifetime in seconds
    /// * `size` - Base size (radius)
    /// * `color` - Color as 0xRRGGBB
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        x: f32,
        y: f32,
        count: usize,
        speed: f32,
        lifetime: f32,
        size: f32,
        color: u32,
    ) {
        for _ in 0..count {
            // Random direction
            let angle = self.random() * core::f32::consts::TAU;
            let speed_var = speed * (0.5 + self.random() * 0.5);
            let vx = angle.cos() * speed_var;
            let vy = angle.sin() * speed_var;

            // Random lifetime and size variation
            let life_var = lifetime * (0.7 + self.random() * 0.3);
            let size_var = size * (0.5 + self.random() * 0.5);

            self.particles[self.write_index] =
                Particle::new(x, y, vx, vy, life_var, size_var, color);
            self.write_index = (self.write_index + 1) % self.particles.len();
        }
    }

    /// Spawns particles in a directional burst.
    ///
    /// # Arguments
    ///
    /// * `x` - X position
    /// * `y` - Y position
    /// * `direction_x` - Direction X component
    /// * `direction_y` - Direction Y component
    /// * `spread` - Angular spread in radians
    /// * `count` - Number of particles
    /// * `speed` - Base speed
    /// * `lifetime` - Base lifetime
    /// * `size` - Base size
    /// * `color` - Color as 0xRRGGBB
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_directional(
        &mut self,
        x: f32,
        y: f32,
        direction_x: f32,
        direction_y: f32,
        spread: f32,
        count: usize,
        speed: f32,
        lifetime: f32,
        size: f32,
        color: u32,
    ) {
        let base_angle = direction_y.atan2(direction_x);

        for _ in 0..count {
            // Random angle within spread
            let angle = base_angle + self.random_signed() * spread;
            let speed_var = speed * (0.5 + self.random() * 0.5);
            let vx = angle.cos() * speed_var;
            let vy = angle.sin() * speed_var;

            // Random lifetime and size variation
            let life_var = lifetime * (0.7 + self.random() * 0.3);
            let size_var = size * (0.5 + self.random() * 0.5);

            self.particles[self.write_index] =
                Particle::new(x, y, vx, vy, life_var, size_var, color);
            self.write_index = (self.write_index + 1) % self.particles.len();
        }
    }

    /// Updates all particles.
    pub fn update(&mut self, dt: f32) {
        for particle in &mut self.particles {
            let _ = particle.update(dt);
        }
    }

    /// Returns all active particles for rendering.
    #[must_use]
    pub fn get_active(&self) -> Vec<&Particle> {
        self.particles.iter().filter(|p| p.active).collect()
    }

    /// Returns the number of active particles.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }

    /// Clears all particles.
    pub fn clear(&mut self) {
        for particle in &mut self.particles {
            particle.active = false;
        }
    }
}

/// Score popup animation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorePopup {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Text to display
    pub text: String,
    /// Time remaining
    pub duration: f32,
    /// Initial Y position (for float animation)
    pub start_y: f32,
}

impl ScorePopup {
    /// Creates a new score popup.
    #[must_use]
    pub fn new(x: f32, y: f32, text: &str, duration: f32) -> Self {
        Self {
            x,
            y,
            text: text.to_string(),
            duration,
            start_y: y,
        }
    }

    /// Updates the popup animation.
    ///
    /// # Returns
    ///
    /// True if popup is still active
    pub fn update(&mut self, dt: f32) -> bool {
        if self.duration <= 0.0 {
            return false;
        }

        self.duration -= dt;

        // Float upward
        self.y -= 50.0 * dt;

        true
    }

    /// Returns the current alpha (fades out over time).
    #[must_use]
    pub fn alpha(&self) -> f32 {
        (self.duration * 2.0).min(1.0)
    }
}

/// Combined juice effects manager.
#[derive(Debug, Clone)]
pub struct JuiceEffects {
    /// Screen shake effect
    pub screen_shake: ScreenShake,
    /// Ball trail effect
    pub ball_trail: BallTrail,
    /// Hit flash effect
    pub hit_flash: HitFlash,
    /// Active score popups
    pub score_popups: Vec<ScorePopup>,
    /// Particle system for collision effects
    pub particles: ParticleSystem,
}

impl Default for JuiceEffects {
    fn default() -> Self {
        Self::new()
    }
}

impl JuiceEffects {
    /// Creates a new juice effects manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            screen_shake: ScreenShake::new(),
            ball_trail: BallTrail::default(),
            hit_flash: HitFlash::new(),
            score_popups: Vec::new(),
            particles: ParticleSystem::default(),
        }
    }

    /// Updates all juice effects.
    ///
    /// # Arguments
    ///
    /// * `ball_x` - Current ball X position
    /// * `ball_y` - Current ball Y position
    /// * `dt` - Delta time in seconds
    pub fn update(&mut self, ball_x: f32, ball_y: f32, dt: f32) {
        let _ = self.screen_shake.update(dt);
        self.ball_trail.update(ball_x, ball_y, dt);
        let _ = self.hit_flash.update(dt);
        self.particles.update(dt);

        // Update and remove expired popups
        self.score_popups.retain_mut(|popup| popup.update(dt));
    }

    /// Triggers effects for a goal scored.
    ///
    /// # Arguments
    ///
    /// * `scorer_x` - X position where score occurred
    /// * `scorer_y` - Y position where score occurred
    /// * `points_text` - Text to show in popup (e.g., "+1")
    pub fn on_goal(&mut self, scorer_x: f32, scorer_y: f32, points_text: &str) {
        // Strong screen shake for goals
        self.screen_shake.trigger(8.0, 0.3);

        // Score popup
        self.score_popups
            .push(ScorePopup::new(scorer_x, scorer_y, points_text, 1.0));

        // Celebratory particle burst (gold/yellow)
        self.particles
            .spawn(scorer_x, scorer_y, 30, 200.0, 0.8, 4.0, 0x00FF_D700);

        // Clear trail on goal (ball resets)
        self.ball_trail.clear();
    }

    /// Triggers effects for a paddle hit.
    ///
    /// # Arguments
    ///
    /// * `right_paddle` - True if right paddle was hit
    pub fn on_paddle_hit(&mut self, right_paddle: bool) {
        // Light screen shake for hits
        self.screen_shake.trigger(3.0, 0.1);

        // Flash the paddle
        self.hit_flash.trigger(right_paddle, 0.8, 0.1);
    }

    /// Triggers effects for a paddle hit with ball position.
    ///
    /// # Arguments
    ///
    /// * `ball_x` - Ball X position
    /// * `ball_y` - Ball Y position
    /// * `right_paddle` - True if right paddle was hit
    pub fn on_paddle_hit_at(&mut self, ball_x: f32, ball_y: f32, right_paddle: bool) {
        // Light screen shake for hits
        self.screen_shake.trigger(3.0, 0.1);

        // Flash the paddle
        self.hit_flash.trigger(right_paddle, 0.8, 0.1);

        // Directional particle burst (white/cyan sparks)
        // Direction away from paddle
        let direction_x = if right_paddle { -1.0 } else { 1.0 };
        self.particles.spawn_directional(
            ball_x,
            ball_y,
            direction_x,
            0.0,
            0.5, // ~30 degree spread
            12,
            150.0,
            0.4,
            3.0,
            0x0000_FFFF, // Cyan
        );
    }

    /// Triggers effects for a wall bounce.
    pub fn on_wall_bounce(&mut self) {
        // Very light shake for wall bounces
        self.screen_shake.trigger(1.5, 0.05);
    }

    /// Resets all effects.
    pub fn reset(&mut self) {
        self.screen_shake.reset();
        self.ball_trail.clear();
        self.hit_flash.reset();
        self.score_popups.clear();
        self.particles.clear();
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::unreadable_literal)]
mod tests {
    use super::*;

    // =========================================================================
    // ScreenShake Tests
    // =========================================================================

    #[test]
    fn test_screen_shake_new() {
        let shake = ScreenShake::new();
        assert_eq!(shake.intensity, 0.0);
        assert_eq!(shake.duration, 0.0);
        assert!(!shake.is_active());
    }

    #[test]
    fn test_screen_shake_trigger() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0, 0.5);

        assert_eq!(shake.intensity, 10.0);
        assert_eq!(shake.duration, 0.5);
        assert!(shake.is_active());
    }

    #[test]
    fn test_screen_shake_stronger_override() {
        let mut shake = ScreenShake::new();
        shake.trigger(5.0, 0.5);
        shake.trigger(10.0, 0.3); // Stronger intensity

        assert_eq!(shake.intensity, 10.0);
    }

    #[test]
    fn test_screen_shake_weaker_no_override() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0, 0.5);
        shake.trigger(5.0, 0.3); // Weaker intensity

        assert_eq!(shake.intensity, 10.0); // Should keep original
    }

    #[test]
    fn test_screen_shake_update_decay() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0, 0.5);

        // Update several times
        for _ in 0..100 {
            let _ = shake.update(0.016);
        }

        // Should have decayed to zero
        assert!(!shake.is_active());
        let (x, y) = shake.offset();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_screen_shake_produces_offset() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0, 0.5);

        let (x, y) = shake.update(0.016);

        // Should produce some offset
        assert!(x != 0.0 || y != 0.0);
    }

    #[test]
    fn test_screen_shake_reset() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0, 0.5);
        let _ = shake.update(0.016);

        shake.reset();

        assert!(!shake.is_active());
        assert_eq!(shake.offset(), (0.0, 0.0));
    }

    // =========================================================================
    // BallTrail Tests
    // =========================================================================

    #[test]
    fn test_ball_trail_new() {
        let trail = BallTrail::new(10, 0.016, 0.15);
        assert_eq!(trail.points.len(), 10);
        assert_eq!(trail.active_count(), 0);
    }

    #[test]
    fn test_ball_trail_default() {
        let trail = BallTrail::default();
        assert_eq!(trail.points.len(), 10);
    }

    #[test]
    fn test_ball_trail_update_adds_points() {
        let mut trail = BallTrail::new(10, 0.016, 0.15);

        // Update with ball position
        trail.update(100.0, 200.0, 0.016);

        assert_eq!(trail.active_count(), 1);
    }

    #[test]
    fn test_ball_trail_points_age() {
        let mut trail = BallTrail::new(10, 0.016, 0.1);
        trail.update(100.0, 200.0, 0.016);

        // Age the point past max_age
        for _ in 0..20 {
            trail.update(100.0, 200.0, 0.016);
        }

        // Old points should have been removed due to aging
        let points = trail.get_points();
        for (_, _, alpha) in &points {
            assert!(*alpha > 0.0);
        }
    }

    #[test]
    fn test_ball_trail_get_points() {
        let mut trail = BallTrail::new(10, 0.016, 0.15);
        trail.update(100.0, 200.0, 0.016);

        let points = trail.get_points();
        assert_eq!(points.len(), 1);

        let (x, y, alpha) = points[0];
        assert_eq!(x, 100.0);
        assert_eq!(y, 200.0);
        assert!((alpha - 1.0).abs() < 0.1); // Nearly full alpha
    }

    #[test]
    fn test_ball_trail_clear() {
        let mut trail = BallTrail::new(10, 0.016, 0.15);
        trail.update(100.0, 200.0, 0.016);
        trail.update(100.0, 200.0, 0.016);

        trail.clear();

        assert_eq!(trail.active_count(), 0);
    }

    #[test]
    fn test_ball_trail_ring_buffer() {
        let mut trail = BallTrail::new(5, 0.001, 1.0); // Very short interval

        // Add more points than capacity
        for i in 0..10 {
            trail.update(i as f32 * 10.0, 0.0, 0.002);
        }

        // Should have at most max_points active
        assert!(trail.active_count() <= 5);
    }

    // =========================================================================
    // HitFlash Tests
    // =========================================================================

    #[test]
    fn test_hit_flash_new() {
        let flash = HitFlash::new();
        assert!(!flash.is_active());
    }

    #[test]
    fn test_hit_flash_trigger() {
        let mut flash = HitFlash::new();
        flash.trigger(true, 0.8, 0.1);

        assert!(flash.is_active());
    }

    #[test]
    fn test_hit_flash_update() {
        let mut flash = HitFlash::new();
        flash.trigger(false, 1.0, 0.1);

        let (active, right, intensity) = flash.update(0.016);

        assert!(active);
        assert!(!right);
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_hit_flash_decays() {
        let mut flash = HitFlash::new();
        flash.trigger(true, 1.0, 0.1);

        // Update past duration
        for _ in 0..20 {
            let _ = flash.update(0.016);
        }

        assert!(!flash.is_active());
    }

    #[test]
    fn test_hit_flash_reset() {
        let mut flash = HitFlash::new();
        flash.trigger(true, 1.0, 0.5);

        flash.reset();

        assert!(!flash.is_active());
    }

    #[test]
    fn test_hit_flash_state_inactive() {
        let flash = HitFlash::new();
        let (left, right, intensity) = flash.flash_state();

        assert!(!left);
        assert!(!right);
        assert_eq!(intensity, 0.0);
    }

    #[test]
    fn test_hit_flash_state_left_paddle() {
        let mut flash = HitFlash::new();
        flash.trigger(false, 1.0, 0.1);

        let (left, right, intensity) = flash.flash_state();

        assert!(left);
        assert!(!right);
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_hit_flash_state_right_paddle() {
        let mut flash = HitFlash::new();
        flash.trigger(true, 1.0, 0.1);

        let (left, right, intensity) = flash.flash_state();

        assert!(!left);
        assert!(right);
        assert!(intensity > 0.0);
    }

    // =========================================================================
    // ScorePopup Tests
    // =========================================================================

    #[test]
    fn test_score_popup_new() {
        let popup = ScorePopup::new(400.0, 300.0, "+1", 1.0);

        assert_eq!(popup.x, 400.0);
        assert_eq!(popup.y, 300.0);
        assert_eq!(popup.text, "+1");
        assert_eq!(popup.duration, 1.0);
    }

    #[test]
    fn test_score_popup_update() {
        let mut popup = ScorePopup::new(400.0, 300.0, "+1", 1.0);
        let initial_y = popup.y;

        let active = popup.update(0.1);

        assert!(active);
        assert!(popup.y < initial_y); // Should float up
    }

    #[test]
    fn test_score_popup_expires() {
        let mut popup = ScorePopup::new(400.0, 300.0, "+1", 0.1);

        // Update past duration
        for _ in 0..20 {
            let _ = popup.update(0.016);
        }

        let active = popup.update(0.016);
        assert!(!active);
    }

    #[test]
    fn test_score_popup_alpha() {
        let popup = ScorePopup::new(400.0, 300.0, "+1", 1.0);
        assert_eq!(popup.alpha(), 1.0);

        let mut popup2 = ScorePopup::new(400.0, 300.0, "+1", 0.3);
        let _ = popup2.update(0.2);
        assert!(popup2.alpha() < 1.0);
    }

    // =========================================================================
    // JuiceEffects Tests
    // =========================================================================

    #[test]
    fn test_juice_effects_new() {
        let juice = JuiceEffects::new();
        assert!(!juice.screen_shake.is_active());
        assert!(!juice.hit_flash.is_active());
        assert!(juice.score_popups.is_empty());
    }

    #[test]
    fn test_juice_effects_default() {
        let juice = JuiceEffects::default();
        assert!(!juice.screen_shake.is_active());
    }

    #[test]
    fn test_juice_effects_on_goal() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");

        assert!(juice.screen_shake.is_active());
        assert_eq!(juice.score_popups.len(), 1);
    }

    #[test]
    fn test_juice_effects_on_paddle_hit() {
        let mut juice = JuiceEffects::new();
        juice.on_paddle_hit(true);

        assert!(juice.screen_shake.is_active());
        assert!(juice.hit_flash.is_active());
    }

    #[test]
    fn test_juice_effects_on_wall_bounce() {
        let mut juice = JuiceEffects::new();
        juice.on_wall_bounce();

        assert!(juice.screen_shake.is_active());
    }

    #[test]
    fn test_juice_effects_update() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");

        // Update should process all effects
        juice.update(100.0, 200.0, 0.016);

        // Trail should have a point
        assert!(juice.ball_trail.active_count() > 0);
    }

    #[test]
    fn test_juice_effects_reset() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");
        juice.on_paddle_hit(false);
        juice.update(100.0, 200.0, 0.016);

        juice.reset();

        assert!(!juice.screen_shake.is_active());
        assert!(!juice.hit_flash.is_active());
        assert!(juice.score_popups.is_empty());
        assert_eq!(juice.ball_trail.active_count(), 0);
    }

    #[test]
    fn test_juice_effects_popup_cleanup() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");

        // Update many times to expire popup
        for _ in 0..100 {
            juice.update(100.0, 200.0, 0.016);
        }

        // Popup should have been removed
        assert!(juice.score_popups.is_empty());
    }

    // =========================================================================
    // Particle Tests
    // =========================================================================

    #[test]
    fn test_particle_new() {
        let particle = Particle::new(100.0, 200.0, 50.0, -30.0, 1.0, 5.0, 0xFF0000);

        assert_eq!(particle.x, 100.0);
        assert_eq!(particle.y, 200.0);
        assert_eq!(particle.vx, 50.0);
        assert_eq!(particle.vy, -30.0);
        assert_eq!(particle.lifetime, 1.0);
        assert_eq!(particle.initial_lifetime, 1.0);
        assert_eq!(particle.size, 5.0);
        assert_eq!(particle.color, 0xFF0000);
        assert!(particle.active);
    }

    #[test]
    fn test_particle_update_position() {
        let mut particle = Particle::new(100.0, 200.0, 50.0, -30.0, 1.0, 5.0, 0xFF0000);

        let alive = particle.update(0.1);

        assert!(alive);
        // Position should change based on velocity
        assert!((particle.x - 105.0).abs() < 0.01);
        // Y changes by velocity + gravity
        assert!(particle.y < 200.0 - 2.0); // Should have moved up somewhat
    }

    #[test]
    fn test_particle_expires() {
        let mut particle = Particle::new(100.0, 200.0, 50.0, -30.0, 0.1, 5.0, 0xFF0000);

        // Update past lifetime
        for _ in 0..20 {
            let _ = particle.update(0.016);
        }

        assert!(!particle.active);
        assert!(!particle.update(0.016)); // Should return false when dead
    }

    #[test]
    fn test_particle_alpha() {
        let mut particle = Particle::new(100.0, 200.0, 50.0, -30.0, 1.0, 5.0, 0xFF0000);

        // Initially full alpha
        assert!((particle.alpha() - 1.0).abs() < 0.01);

        // Half way through
        let _ = particle.update(0.5);
        assert!(particle.alpha() > 0.4 && particle.alpha() < 0.6);

        // Dead particle
        particle.lifetime = 0.0;
        particle.active = false;
        assert_eq!(particle.alpha(), 0.0);
    }

    #[test]
    fn test_particle_alpha_zero_initial_lifetime() {
        let mut particle = Particle::new(100.0, 200.0, 50.0, -30.0, 1.0, 5.0, 0xFF0000);
        particle.initial_lifetime = 0.0;

        assert_eq!(particle.alpha(), 0.0);
    }

    #[test]
    fn test_particle_rgb() {
        let particle = Particle::new(100.0, 200.0, 0.0, 0.0, 1.0, 5.0, 0xFF8000); // Orange

        let (r, g, b) = particle.rgb();

        assert!((r - 1.0).abs() < 0.01); // FF = 255 = 1.0
        assert!((g - 0.5).abs() < 0.02); // 80 = 128 ≈ 0.5
        assert!((b - 0.0).abs() < 0.01); // 00 = 0 = 0.0
    }

    #[test]
    fn test_particle_inactive_doesnt_update() {
        let mut particle = Particle::new(100.0, 200.0, 50.0, -30.0, 1.0, 5.0, 0xFF0000);
        particle.active = false;

        let alive = particle.update(0.1);

        assert!(!alive);
        // Position should not have changed
        assert_eq!(particle.x, 100.0);
    }

    // =========================================================================
    // ParticleSystem Tests
    // =========================================================================

    #[test]
    fn test_particle_system_new() {
        let system = ParticleSystem::new(100);

        assert_eq!(system.particles.len(), 100);
        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_particle_system_default() {
        let system = ParticleSystem::default();

        assert_eq!(system.particles.len(), 200);
        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_particle_system_spawn() {
        let mut system = ParticleSystem::new(100);

        system.spawn(400.0, 300.0, 10, 100.0, 1.0, 5.0, 0xFFFFFF);

        assert_eq!(system.active_count(), 10);
    }

    #[test]
    fn test_particle_system_spawn_directional() {
        let mut system = ParticleSystem::new(100);

        system.spawn_directional(400.0, 300.0, 1.0, 0.0, 0.5, 15, 100.0, 1.0, 5.0, 0x00FF00);

        assert_eq!(system.active_count(), 15);
    }

    #[test]
    fn test_particle_system_update() {
        let mut system = ParticleSystem::new(100);
        system.spawn(400.0, 300.0, 5, 100.0, 1.0, 5.0, 0xFFFFFF);

        // Get initial positions
        let initial: Vec<(f32, f32)> = system.get_active().iter().map(|p| (p.x, p.y)).collect();

        system.update(0.1);

        // Positions should have changed
        let updated: Vec<(f32, f32)> = system.get_active().iter().map(|p| (p.x, p.y)).collect();
        assert_ne!(initial, updated);
    }

    #[test]
    fn test_particle_system_particles_expire() {
        let mut system = ParticleSystem::new(100);
        system.spawn(400.0, 300.0, 10, 100.0, 0.1, 5.0, 0xFFFFFF); // Short lifetime

        // Update past lifetime
        for _ in 0..20 {
            system.update(0.016);
        }

        // All particles should have expired
        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_particle_system_get_active() {
        let mut system = ParticleSystem::new(100);
        system.spawn(400.0, 300.0, 5, 100.0, 1.0, 5.0, 0xFFFFFF);

        let active = system.get_active();

        assert_eq!(active.len(), 5);
        for particle in active {
            assert!(particle.active);
        }
    }

    #[test]
    fn test_particle_system_clear() {
        let mut system = ParticleSystem::new(100);
        system.spawn(400.0, 300.0, 10, 100.0, 1.0, 5.0, 0xFFFFFF);
        assert_eq!(system.active_count(), 10);

        system.clear();

        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_particle_system_ring_buffer() {
        let mut system = ParticleSystem::new(10);

        // Spawn more particles than pool size
        system.spawn(400.0, 300.0, 15, 100.0, 1.0, 5.0, 0xFFFFFF);

        // Should have wrapped around
        assert!(system.active_count() <= 10);
    }

    // =========================================================================
    // JuiceEffects Particle Integration Tests
    // =========================================================================

    #[test]
    fn test_juice_effects_on_goal_spawns_particles() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");

        // Should have spawned celebration particles
        assert!(juice.particles.active_count() > 0);
    }

    #[test]
    fn test_juice_effects_on_paddle_hit_at_spawns_particles() {
        let mut juice = JuiceEffects::new();
        juice.on_paddle_hit_at(50.0, 300.0, false);

        // Should have spawned spark particles
        assert!(juice.particles.active_count() > 0);
    }

    #[test]
    fn test_juice_effects_particles_update() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");
        let initial_count = juice.particles.active_count();

        // Update many times
        for _ in 0..100 {
            juice.update(100.0, 200.0, 0.016);
        }

        // Particles should have decayed
        assert!(juice.particles.active_count() < initial_count);
    }

    #[test]
    fn test_juice_effects_reset_clears_particles() {
        let mut juice = JuiceEffects::new();
        juice.on_goal(400.0, 300.0, "+1");
        assert!(juice.particles.active_count() > 0);

        juice.reset();

        assert_eq!(juice.particles.active_count(), 0);
    }
}
