//! Web platform integration for the Jugar game engine.
//!
//! This module provides the main `WebPlatform` struct that bridges the Jugar engine
//! to browser APIs via wasm-bindgen. All game logic runs in Rust; JavaScript only
//! handles event forwarding and Canvas2D rendering.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::ai::PongAI;
use crate::audio::{AudioEvent, ProceduralAudio};
use crate::input::{process_input_events, InputTranslationError};
use crate::juice::JuiceEffects;
use crate::render::{Canvas2DCommand, Color, RenderFrame, TextAlign, TextBaseline};
use crate::time::FrameTimer;
use jugar_input::InputState;

/// Web platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// Canvas width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Canvas height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
    /// Target frames per second (for fixed timestep)
    #[serde(default = "default_fps")]
    pub target_fps: u32,
    /// Enable debug rendering
    #[serde(default)]
    pub debug: bool,
    /// Enable AI opponent (replaces Player 2)
    #[serde(default = "default_ai_enabled")]
    pub ai_enabled: bool,
}

const fn default_ai_enabled() -> bool {
    true // AI enabled by default for single-player experience
}

const fn default_width() -> u32 {
    800
}

const fn default_height() -> u32 {
    600
}

const fn default_fps() -> u32 {
    60
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            target_fps: 60,
            debug: false,
            ai_enabled: true,
        }
    }
}

impl WebConfig {
    /// Creates a new web config with specified dimensions.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            target_fps: 60,
            debug: false,
            ai_enabled: true,
        }
    }

    /// Parses configuration from JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes configuration to JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Error type for web platform operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebPlatformError {
    /// Invalid configuration
    InvalidConfig(String),
    /// Input processing error
    InputError(String),
    /// Render error
    RenderError(String),
}

impl core::fmt::Display for WebPlatformError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "Invalid config: {msg}"),
            Self::InputError(msg) => write!(f, "Input error: {msg}"),
            Self::RenderError(msg) => write!(f, "Render error: {msg}"),
        }
    }
}

impl core::error::Error for WebPlatformError {}

impl From<InputTranslationError> for WebPlatformError {
    fn from(err: InputTranslationError) -> Self {
        Self::InputError(err.to_string())
    }
}

/// Frame output returned to JavaScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameOutput {
    /// Render commands to execute
    pub commands: Vec<Canvas2DCommand>,
    /// Audio events to play via Web Audio API
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub audio_events: Vec<AudioEvent>,
    /// Debug information (only present if debug mode enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<DebugInfo>,
}

/// Debug information for development.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Delta time in milliseconds
    pub dt_ms: f64,
    /// Current FPS
    pub fps: f64,
    /// Frame count
    pub frame_count: u64,
    /// Input state summary
    pub input_summary: String,
}

/// Trait for game implementations that can run on the web platform.
///
/// Games implement this trait to integrate with the `WebPlatform`.
pub trait WebGame: Send {
    /// Called each frame with input and delta time.
    ///
    /// # Arguments
    ///
    /// * `input` - Current input state
    /// * `dt` - Delta time in seconds
    fn update(&mut self, input: &InputState, dt: f64);

    /// Called to generate render commands for the current frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - Render frame to push commands to
    fn render(&self, frame: &mut RenderFrame);

    /// Called when the canvas is resized.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    fn resize(&mut self, width: u32, height: u32);
}

/// Game state for menu/pause functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GameState {
    /// Waiting at menu for player to start
    #[default]
    Menu,
    /// Game is actively playing
    Playing,
    /// Game is paused
    Paused,
    /// Game over (one player reached winning score)
    GameOver,
}

/// A simple Pong game implementation for testing.
#[derive(Debug, Clone)]
pub struct PongGame {
    /// Canvas width
    width: f32,
    /// Canvas height
    height: f32,
    /// Left paddle Y position
    left_paddle_y: f32,
    /// Right paddle Y position
    right_paddle_y: f32,
    /// Ball X position
    ball_x: f32,
    /// Ball Y position
    ball_y: f32,
    /// Ball X velocity
    ball_vx: f32,
    /// Ball Y velocity
    ball_vy: f32,
    /// Left player score
    left_score: u32,
    /// Right player score
    right_score: u32,
    /// Paddle height
    paddle_height: f32,
    /// Paddle width
    paddle_width: f32,
    /// Ball radius
    ball_radius: f32,
    /// Paddle speed
    paddle_speed: f32,
    /// AI opponent (if enabled)
    ai: Option<PongAI>,
    /// Juice effects (screen shake, trails, etc.)
    juice: JuiceEffects,
    /// Previous ball Y for wall bounce detection
    prev_ball_y: f32,
    /// Current game state
    state: GameState,
    /// Winning score threshold
    winning_score: u32,
    /// Track if Space was pressed last frame (for edge detection)
    space_was_pressed: bool,
    /// Track if Escape was pressed last frame (for edge detection)
    escape_was_pressed: bool,
    /// Current rally count (consecutive hits without scoring)
    rally_count: u32,
    /// High score (persisted across games in session)
    high_score: u32,
    /// Background animation time accumulator
    bg_time: f32,
    /// Procedural audio generator
    audio: ProceduralAudio,
}

impl Default for PongGame {
    fn default() -> Self {
        Self::new(800.0, 600.0, true)
    }
}

impl PongGame {
    /// Creates a new Pong game with the given dimensions.
    #[must_use]
    pub fn new(width: f32, height: f32, ai_enabled: bool) -> Self {
        let ai = if ai_enabled {
            Some(PongAI::default())
        } else {
            None
        };

        Self {
            width,
            height,
            left_paddle_y: height / 2.0,
            right_paddle_y: height / 2.0,
            ball_x: width / 2.0,
            ball_y: height / 2.0,
            ball_vx: 200.0,
            ball_vy: 150.0,
            left_score: 0,
            right_score: 0,
            paddle_height: 100.0,
            paddle_width: 15.0,
            ball_radius: 10.0,
            paddle_speed: 400.0,
            ai,
            juice: JuiceEffects::new(),
            prev_ball_y: height / 2.0,
            state: GameState::Menu,
            winning_score: 11,
            space_was_pressed: false,
            escape_was_pressed: false,
            rally_count: 0,
            high_score: 0,
            bg_time: 0.0,
            audio: ProceduralAudio::new(),
        }
    }

    /// Resets the game to initial state.
    pub fn reset_game(&mut self) {
        self.left_paddle_y = self.height / 2.0;
        self.right_paddle_y = self.height / 2.0;
        self.ball_x = self.width / 2.0;
        self.ball_y = self.height / 2.0;
        self.ball_vx = 200.0;
        self.ball_vy = 150.0;
        self.left_score = 0;
        self.right_score = 0;
        self.prev_ball_y = self.height / 2.0;
        self.rally_count = 0;
        // High score persists across games - don't reset it
        self.juice.reset();
        if let Some(ref mut ai) = self.ai {
            ai.reset();
        }
    }

    /// Returns the current rally count.
    #[must_use]
    pub const fn rally_count(&self) -> u32 {
        self.rally_count
    }

    /// Returns the high score.
    #[must_use]
    pub const fn high_score(&self) -> u32 {
        self.high_score
    }

    /// Takes all pending audio events.
    pub fn take_audio_events(&mut self) -> Vec<AudioEvent> {
        self.audio.take_events()
    }

    /// Returns the current game state.
    #[must_use]
    pub const fn state(&self) -> GameState {
        self.state
    }

    /// Sets the game state (for testing).
    #[cfg(test)]
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable ref not stable
    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    /// Starts the game (transitions from Menu to Playing).
    pub fn start(&mut self) {
        if self.state == GameState::Menu {
            self.reset_game();
            self.state = GameState::Playing;
        }
    }

    /// Resets the ball to the center.
    fn reset_ball(&mut self) {
        self.ball_x = self.width / 2.0;
        self.ball_y = self.height / 2.0;
        // Reverse direction towards the player who lost
        self.ball_vx = -self.ball_vx.signum() * 200.0;
        self.ball_vy = if fastrand::bool() { 150.0 } else { -150.0 };
    }

    /// Returns the left score.
    #[must_use]
    pub const fn left_score(&self) -> u32 {
        self.left_score
    }

    /// Returns the right score.
    #[must_use]
    pub const fn right_score(&self) -> u32 {
        self.right_score
    }

    /// Returns the ball position.
    #[must_use]
    pub const fn ball_position(&self) -> (f32, f32) {
        (self.ball_x, self.ball_y)
    }
}

impl WebGame for PongGame {
    #[allow(clippy::too_many_lines)] // Game update logic is inherently complex
    fn update(&mut self, input: &InputState, dt: f64) {
        let dt = dt as f32;

        // Handle state transitions first
        let space_pressed = input.is_key_pressed(jugar_input::KeyCode::Space);
        let escape_pressed = input.is_key_pressed(jugar_input::KeyCode::Escape);

        // Edge detection for Space key
        let space_just_pressed = space_pressed && !self.space_was_pressed;
        let escape_just_pressed = escape_pressed && !self.escape_was_pressed;

        self.space_was_pressed = space_pressed;
        self.escape_was_pressed = escape_pressed;

        match self.state {
            GameState::Menu => {
                if space_just_pressed {
                    self.reset_game();
                    self.state = GameState::Playing;
                    self.audio.on_game_start();
                }
                return; // Don't update game logic in menu
            }
            GameState::Playing => {
                if escape_just_pressed {
                    self.state = GameState::Paused;
                    return;
                }
            }
            GameState::Paused => {
                if space_just_pressed || escape_just_pressed {
                    self.state = GameState::Playing;
                }
                return; // Don't update game logic when paused
            }
            GameState::GameOver => {
                if space_just_pressed {
                    self.reset_game();
                    self.state = GameState::Playing;
                    self.audio.on_game_start();
                }
                return; // Don't update game logic at game over
            }
        }

        // Game is Playing - run game logic
        let half_paddle = self.paddle_height / 2.0;

        // Store previous ball Y for wall bounce detection
        self.prev_ball_y = self.ball_y;

        // Left paddle controls (W/S keys) - Player 1
        if input.is_key_pressed(jugar_input::KeyCode::Letter('W')) {
            self.left_paddle_y -= self.paddle_speed * dt;
        }
        if input.is_key_pressed(jugar_input::KeyCode::Letter('S')) {
            self.left_paddle_y += self.paddle_speed * dt;
        }

        // Right paddle controls - AI or Player 2
        if let Some(ref mut ai) = self.ai {
            // AI controls the right paddle
            let ai_velocity = ai.update(
                self.ball_x,
                self.ball_y,
                self.ball_vx,
                self.ball_vy,
                self.right_paddle_y,
                self.paddle_height,
                self.width,
                self.height,
                dt,
            );
            self.right_paddle_y += ai_velocity * dt;
        } else {
            // Player 2 controls (Arrow keys)
            if input.is_key_pressed(jugar_input::KeyCode::Up) {
                self.right_paddle_y -= self.paddle_speed * dt;
            }
            if input.is_key_pressed(jugar_input::KeyCode::Down) {
                self.right_paddle_y += self.paddle_speed * dt;
            }
        }

        // Clamp paddles to screen
        self.left_paddle_y = self
            .left_paddle_y
            .clamp(half_paddle, self.height - half_paddle);
        self.right_paddle_y = self
            .right_paddle_y
            .clamp(half_paddle, self.height - half_paddle);

        // Update ball position
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        // Ball collision with top/bottom walls
        let mut wall_bounced = false;
        if self.ball_y - self.ball_radius < 0.0 {
            self.ball_y = self.ball_radius;
            self.ball_vy = self.ball_vy.abs();
            wall_bounced = true;
        } else if self.ball_y + self.ball_radius > self.height {
            self.ball_y = self.height - self.ball_radius;
            self.ball_vy = -self.ball_vy.abs();
            wall_bounced = true;
        }

        // Trigger wall bounce juice effect and audio
        if wall_bounced {
            self.juice.on_wall_bounce();
            // Use velocity-based pitch variation for wall bounce
            let ball_speed = self.ball_vx.hypot(self.ball_vy);
            self.audio.on_wall_bounce_with_velocity(ball_speed, 250.0);
        }

        // Track paddle hits/misses for AI difficulty adjustment
        let left_paddle_x = 20.0 + self.paddle_width;
        let right_paddle_x = self.width - 20.0 - self.paddle_width;

        // Left paddle collision (Player 1)
        if self.ball_x - self.ball_radius < left_paddle_x
            && self.ball_x - self.ball_radius > 20.0
            && self.ball_y > self.left_paddle_y - half_paddle
            && self.ball_y < self.left_paddle_y + half_paddle
        {
            self.ball_x = left_paddle_x + self.ball_radius;
            self.ball_vx = self.ball_vx.abs() * 1.05; // Speed up slightly

            // Increment rally counter
            self.rally_count += 1;

            // Juice: paddle hit effect with particles
            self.juice.on_paddle_hit_at(self.ball_x, self.ball_y, false);

            // Audio: paddle hit sound with pitch variation based on hit location
            self.audio
                .on_paddle_hit(self.ball_y, self.left_paddle_y, self.paddle_height);

            // Audio: rally milestone every 5 hits
            if self.rally_count.is_multiple_of(5) {
                self.audio.on_rally_milestone(self.rally_count);
            }

            // Record player hit for AI adaptation
            if let Some(ref mut ai) = self.ai {
                ai.record_player_hit();
            }
        }

        // Right paddle collision (AI or Player 2)
        if self.ball_x + self.ball_radius > right_paddle_x
            && self.ball_x + self.ball_radius < self.width - 20.0
            && self.ball_y > self.right_paddle_y - half_paddle
            && self.ball_y < self.right_paddle_y + half_paddle
        {
            self.ball_x = right_paddle_x - self.ball_radius;
            self.ball_vx = -self.ball_vx.abs() * 1.05; // Speed up slightly

            // Increment rally counter
            self.rally_count += 1;

            // Juice: paddle hit effect with particles
            self.juice.on_paddle_hit_at(self.ball_x, self.ball_y, true);

            // Audio: paddle hit sound with pitch variation based on hit location
            self.audio
                .on_paddle_hit(self.ball_y, self.right_paddle_y, self.paddle_height);

            // Audio: rally milestone every 5 hits
            if self.rally_count.is_multiple_of(5) {
                self.audio.on_rally_milestone(self.rally_count);
            }
        }

        // Scoring
        if self.ball_x < 0.0 {
            // Player missed - AI scores
            self.right_score += 1;

            // Update high score based on rally before resetting
            if self.rally_count > self.high_score {
                self.high_score = self.rally_count;
            }
            self.rally_count = 0;

            // Juice: goal scored effect
            self.juice.on_goal(self.width * 0.75, 50.0, "+1");

            // Audio: goal sound (player did NOT score)
            self.audio.on_goal(false);

            if let Some(ref mut ai) = self.ai {
                ai.record_player_miss();
                ai.adapt_difficulty();
            }

            // Check for game over
            if self.right_score >= self.winning_score {
                self.state = GameState::GameOver;
            } else {
                self.reset_ball();
            }
        } else if self.ball_x > self.width {
            // AI/Player 2 missed - Player 1 scores
            self.left_score += 1;

            // Update high score based on rally before resetting
            if self.rally_count > self.high_score {
                self.high_score = self.rally_count;
            }
            self.rally_count = 0;

            // Juice: goal scored effect
            self.juice.on_goal(self.width * 0.25, 50.0, "+1");

            // Audio: goal sound (player scored)
            self.audio.on_goal(true);

            if let Some(ref mut ai) = self.ai {
                ai.adapt_difficulty();
            }

            // Check for game over
            if self.left_score >= self.winning_score {
                self.state = GameState::GameOver;
            } else {
                self.reset_ball();
            }
        }

        // Update juice effects
        self.juice.update(self.ball_x, self.ball_y, dt);

        // Update background animation timer
        self.bg_time += dt;
    }

    #[allow(clippy::too_many_lines, clippy::suboptimal_flops)] // Render logic is inherently complex
    fn render(&self, frame: &mut RenderFrame) {
        // Get screen shake offset
        let (shake_x, shake_y) = self.juice.screen_shake.offset();

        // Clear screen
        frame.clear_screen(Color::BLACK);

        // Background animation: subtle animated dot grid
        let dot_spacing = 40.0;
        let dot_radius = 1.5;
        let wave_speed = 0.5;
        let wave_amplitude = 0.3;
        let num_cols = (self.width / dot_spacing).ceil() as i32;
        let num_rows = (self.height / dot_spacing).ceil() as i32;

        for row in 0..num_rows {
            for col in 0..num_cols {
                let base_x = (col as f32) * dot_spacing + dot_spacing / 2.0;
                let base_y = (row as f32) * dot_spacing + dot_spacing / 2.0;

                // Wave effect based on distance from center
                let dx = base_x - self.width / 2.0;
                let dy = base_y - self.height / 2.0;
                let dist = dx.hypot(dy);
                let wave_phase = dist * 0.02 - self.bg_time * wave_speed;
                let alpha = 0.1 + wave_amplitude * (wave_phase.sin() * 0.5 + 0.5);

                let dot_color = Color::new(0.2, 0.2, 0.4, alpha);
                frame.fill_circle(base_x + shake_x, base_y + shake_y, dot_radius, dot_color);
            }
        }

        // Draw center line (dashed effect via multiple lines)
        let dash_height = 20.0;
        let gap = 15.0;
        let center_x = self.width / 2.0 + shake_x;
        let num_dashes = (self.height / (dash_height + gap)).ceil() as usize;
        for i in 0..num_dashes {
            let y = (i as f32) * (dash_height + gap) + shake_y;
            frame.fill_rect(center_x - 2.0, y, 4.0, dash_height, Color::WHITE);
        }

        // Draw ball trail (behind ball)
        for (x, y, alpha) in self.juice.ball_trail.get_points() {
            let trail_color = Color::new(1.0, 1.0, 1.0, alpha * 0.5);
            let trail_radius = self.ball_radius * (0.3 + 0.7 * alpha);
            frame.fill_circle(x + shake_x, y + shake_y, trail_radius, trail_color);
        }

        // Get hit flash state
        let (left_flash, right_flash, flash_intensity) = self.juice.hit_flash.flash_state();

        // Draw left paddle
        let half_paddle = self.paddle_height / 2.0;
        let left_paddle_x = 20.0 + shake_x;
        let left_paddle_top = self.left_paddle_y - half_paddle + shake_y;

        let left_paddle_color = if left_flash {
            Color::new(1.0, 1.0, flash_intensity, 1.0)
        } else {
            Color::WHITE
        };
        frame.fill_rect(
            left_paddle_x,
            left_paddle_top,
            self.paddle_width,
            self.paddle_height,
            left_paddle_color,
        );

        // Draw right paddle
        let right_paddle_x = self.width - 20.0 - self.paddle_width + shake_x;
        let right_paddle_top = self.right_paddle_y - half_paddle + shake_y;

        let right_paddle_color = if right_flash {
            Color::new(1.0, 1.0, flash_intensity, 1.0)
        } else {
            Color::WHITE
        };
        frame.fill_rect(
            right_paddle_x,
            right_paddle_top,
            self.paddle_width,
            self.paddle_height,
            right_paddle_color,
        );

        // Draw ball with squash/stretch based on velocity
        // Calculate velocity magnitude for stretch effect
        let speed = self.ball_vx.hypot(self.ball_vy);
        let base_speed = 250.0; // Reference speed for no stretch
        let stretch_factor = (speed / base_speed).clamp(0.8, 1.5);

        // Calculate angle of movement for rotation
        let angle = self.ball_vy.atan2(self.ball_vx);

        // Squash perpendicular to movement, stretch along movement direction
        // We use cos/sin to decompose the stretch into x and y components
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Stretch along movement direction, compress perpendicular (preserve area)
        let stretch_along = stretch_factor;
        let stretch_perp = 1.0 / stretch_factor;

        // Calculate effective radii for x and y after rotation
        let rx = self.ball_radius * (stretch_along * cos_a).hypot(stretch_perp * sin_a);
        let ry = self.ball_radius * (stretch_along * sin_a).hypot(stretch_perp * cos_a);

        // Draw ellipse-like ball using the larger of the two radii for circle approximation
        // (true ellipse would require canvas transform, this gives a subtle effect)
        let avg_radius = f32::midpoint(rx, ry);
        frame.fill_circle(
            self.ball_x + shake_x,
            self.ball_y + shake_y,
            avg_radius,
            Color::WHITE,
        );

        // Draw scores
        frame.fill_text_aligned(
            &self.left_score.to_string(),
            self.width / 4.0 + shake_x,
            50.0 + shake_y,
            "48px monospace",
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Top,
        );
        frame.fill_text_aligned(
            &self.right_score.to_string(),
            3.0 * self.width / 4.0 + shake_x,
            50.0 + shake_y,
            "48px monospace",
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Top,
        );

        // Draw score popups
        for popup in &self.juice.score_popups {
            let popup_color = Color::new(1.0, 1.0, 0.0, popup.alpha()); // Yellow with alpha
            frame.fill_text_aligned(
                &popup.text,
                popup.x + shake_x,
                popup.y + shake_y,
                "32px monospace",
                popup_color,
                TextAlign::Center,
                TextBaseline::Middle,
            );
        }

        // Draw particles
        for particle in self.juice.particles.get_active() {
            let (r, g, b) = particle.rgb();
            let particle_color = Color::new(r, g, b, particle.alpha());
            frame.fill_circle(
                particle.x + shake_x,
                particle.y + shake_y,
                particle.size,
                particle_color,
            );
        }

        // Draw rally counter (only during gameplay with active rally)
        if self.state == GameState::Playing && self.rally_count > 0 {
            let rally_text = format!("Rally: {}", self.rally_count);
            // Color intensity increases with rally count
            let intensity = (self.rally_count as f32 / 20.0).min(1.0);
            let rally_color = Color::new(0.5 + intensity * 0.5, 1.0, 0.5 + intensity * 0.5, 0.8);
            frame.fill_text_aligned(
                &rally_text,
                self.width / 2.0 + shake_x,
                self.height - 30.0 + shake_y,
                "20px monospace",
                rally_color,
                TextAlign::Center,
                TextBaseline::Bottom,
            );
        }

        // Draw high score (if set)
        if self.high_score > 0 {
            let high_score_text = format!("Best Rally: {}", self.high_score);
            frame.fill_text_aligned(
                &high_score_text,
                self.width / 2.0 + shake_x,
                20.0 + shake_y,
                "14px monospace",
                Color::new(0.5, 0.5, 0.5, 0.7),
                TextAlign::Center,
                TextBaseline::Top,
            );
        }

        // Draw state-specific overlays
        match self.state {
            GameState::Menu => {
                // Semi-transparent overlay
                frame.fill_rect(
                    0.0,
                    0.0,
                    self.width,
                    self.height,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                // Title
                frame.fill_text_aligned(
                    "PONG",
                    self.width / 2.0,
                    self.height / 3.0,
                    "64px monospace",
                    Color::WHITE,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );

                // Instructions
                frame.fill_text_aligned(
                    "Press SPACE to Start",
                    self.width / 2.0,
                    self.height / 2.0,
                    "24px monospace",
                    Color::new(0.7, 0.7, 0.7, 1.0),
                    TextAlign::Center,
                    TextBaseline::Middle,
                );

                // Controls
                frame.fill_text_aligned(
                    "W/S - Move Paddle | ESC - Pause",
                    self.width / 2.0,
                    self.height * 0.65,
                    "16px monospace",
                    Color::new(0.5, 0.5, 0.5, 1.0),
                    TextAlign::Center,
                    TextBaseline::Middle,
                );
            }
            GameState::Paused => {
                // Semi-transparent overlay
                frame.fill_rect(
                    0.0,
                    0.0,
                    self.width,
                    self.height,
                    Color::new(0.0, 0.0, 0.0, 0.5),
                );

                frame.fill_text_aligned(
                    "PAUSED",
                    self.width / 2.0,
                    self.height / 2.0 - 30.0,
                    "48px monospace",
                    Color::WHITE,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );

                frame.fill_text_aligned(
                    "Press SPACE or ESC to Resume",
                    self.width / 2.0,
                    self.height / 2.0 + 30.0,
                    "20px monospace",
                    Color::new(0.7, 0.7, 0.7, 1.0),
                    TextAlign::Center,
                    TextBaseline::Middle,
                );
            }
            GameState::GameOver => {
                // Semi-transparent overlay
                frame.fill_rect(
                    0.0,
                    0.0,
                    self.width,
                    self.height,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                // Winner text
                let winner = if self.left_score >= self.winning_score {
                    "YOU WIN!"
                } else {
                    "GAME OVER"
                };
                let winner_color = if self.left_score >= self.winning_score {
                    Color::new(0.2, 1.0, 0.2, 1.0) // Green
                } else {
                    Color::new(1.0, 0.3, 0.3, 1.0) // Red
                };

                frame.fill_text_aligned(
                    winner,
                    self.width / 2.0,
                    self.height / 2.0 - 40.0,
                    "48px monospace",
                    winner_color,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );

                // Final score
                let score_text = format!("{} - {}", self.left_score, self.right_score);
                frame.fill_text_aligned(
                    &score_text,
                    self.width / 2.0,
                    self.height / 2.0 + 20.0,
                    "32px monospace",
                    Color::WHITE,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );

                frame.fill_text_aligned(
                    "Press SPACE to Play Again",
                    self.width / 2.0,
                    self.height / 2.0 + 80.0,
                    "20px monospace",
                    Color::new(0.7, 0.7, 0.7, 1.0),
                    TextAlign::Center,
                    TextBaseline::Middle,
                );
            }
            GameState::Playing => {
                // No overlay when playing
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        let old_width = self.width;
        let old_height = self.height;
        self.width = width as f32;
        self.height = height as f32;

        // Scale positions proportionally
        if old_width > 0.0 && old_height > 0.0 {
            self.ball_x = self.ball_x * self.width / old_width;
            self.ball_y = self.ball_y * self.height / old_height;
            self.left_paddle_y = self.left_paddle_y * self.height / old_height;
            self.right_paddle_y = self.right_paddle_y * self.height / old_height;
        }
    }
}

/// The main web platform struct exposed to JavaScript via wasm-bindgen.
///
/// This handles the game loop, input processing, and render command generation.
/// All computation happens in Rust; JavaScript only forwards events and draws.
#[wasm_bindgen]
#[allow(missing_debug_implementations)] // Cannot derive Debug with wasm_bindgen
pub struct WebPlatform {
    /// Configuration
    config: WebConfig,
    /// Frame timer for delta time calculation
    timer: FrameTimer,
    /// Input state
    input: InputState,
    /// Render frame buffer
    render_frame: RenderFrame,
    /// Current game (stored as boxed trait object)
    #[allow(dead_code)]
    game: Option<Box<dyn WebGame>>,
    /// Pong game (direct storage for WASM simplicity)
    pong: PongGame,
    /// Frame counter
    frame_count: u64,
}

#[wasm_bindgen]
impl WebPlatform {
    /// Creates a new `WebPlatform` with configuration from JSON.
    ///
    /// # Arguments
    ///
    /// * `config_json` - JSON string with configuration
    ///
    /// # Errors
    ///
    /// Returns a JavaScript error if the configuration is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<Self, JsValue> {
        let config: WebConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {e}")))?;

        let fixed_dt = 1.0 / f64::from(config.target_fps);
        let mut timer = FrameTimer::new();
        timer.set_fixed_dt(fixed_dt);

        let pong = PongGame::new(config.width as f32, config.height as f32, config.ai_enabled);

        Ok(Self {
            config,
            timer,
            input: InputState::new(),
            render_frame: RenderFrame::with_capacity(100),
            game: None,
            pong,
            frame_count: 0,
        })
    }

    /// Creates a new `WebPlatform` with default configuration.
    #[wasm_bindgen(js_name = "newDefault")]
    #[must_use]
    pub fn new_default() -> Self {
        let config = WebConfig::default();
        let pong = PongGame::new(config.width as f32, config.height as f32, config.ai_enabled);

        Self {
            config,
            timer: FrameTimer::new(),
            input: InputState::new(),
            render_frame: RenderFrame::with_capacity(100),
            game: None,
            pong,
            frame_count: 0,
        }
    }

    /// Processes a single frame.
    ///
    /// This is called from `requestAnimationFrame`. All game logic runs here.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - `DOMHighResTimeStamp` from `requestAnimationFrame`
    /// * `input_events_json` - JSON array of input events since last frame
    ///
    /// # Returns
    ///
    /// JSON string with render commands for Canvas2D execution.
    #[wasm_bindgen]
    pub fn frame(&mut self, timestamp: f64, input_events_json: &str) -> String {
        // Update timer and get delta time
        let dt = self.timer.update(timestamp);

        // Process input events
        // Ignore errors for now - invalid events are just skipped
        let _ = process_input_events(input_events_json, &mut self.input);

        // Update game logic
        self.pong.update(&self.input, dt);

        // Clear input events for next frame (key presses persist, events don't)
        self.input.clear_events();

        // Generate render commands
        self.render_frame.clear();
        self.pong.render(&mut self.render_frame);

        // Add debug info if enabled
        if self.config.debug {
            self.render_debug_info(dt);
        }

        // Take any pending audio events
        let audio_events = self.pong.take_audio_events();

        // Build frame output with optional debug info
        let output = FrameOutput {
            commands: self.render_frame.commands.clone(),
            audio_events,
            debug_info: if self.config.debug {
                Some(DebugInfo {
                    dt_ms: dt * 1000.0,
                    fps: self.timer.average_fps(),
                    frame_count: self.frame_count,
                    input_summary: String::new(),
                })
            } else {
                None
            },
        };
        self.frame_count += 1;

        // Serialize and return
        serde_json::to_string(&output).unwrap_or_else(|_| r#"{"commands":[]}"#.to_string())
    }

    /// Handles canvas resize.
    ///
    /// # Arguments
    ///
    /// * `width` - New canvas width in pixels
    /// * `height` - New canvas height in pixels
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.pong.resize(width, height);
    }

    /// Returns the current configuration as JSON.
    #[wasm_bindgen(js_name = "getConfig")]
    #[must_use]
    pub fn get_config(&self) -> String {
        self.config.to_json().unwrap_or_else(|_| "{}".to_string())
    }

    /// Returns current debug statistics as JSON.
    #[wasm_bindgen(js_name = "getStats")]
    #[must_use]
    pub fn get_stats(&self) -> String {
        let stats = serde_json::json!({
            "fps": self.timer.average_fps(),
            "frame_count": self.timer.frame_count(),
            "total_time": self.timer.total_time(),
        });
        stats.to_string()
    }

    /// Resets the timer (useful when tab becomes visible again).
    #[wasm_bindgen(js_name = "resetTimer")]
    pub fn reset_timer(&mut self) {
        self.timer.reset();
    }

    fn render_debug_info(&mut self, dt: f64) {
        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        let debug_text = format!("FPS: {:.0} | Frame: {}", fps, self.timer.frame_count());

        self.render_frame.fill_text_aligned(
            &debug_text,
            10.0,
            self.config.height as f32 - 10.0,
            "14px monospace",
            Color::GREEN,
            TextAlign::Left,
            TextBaseline::Bottom,
        );
    }
}

// Non-wasm methods for testing
impl WebPlatform {
    /// Creates a platform without wasm-bindgen (for testing).
    #[must_use]
    pub fn new_for_test(config: WebConfig) -> Self {
        let pong = PongGame::new(config.width as f32, config.height as f32, config.ai_enabled);

        Self {
            config,
            timer: FrameTimer::new(),
            input: InputState::new(),
            render_frame: RenderFrame::with_capacity(100),
            game: None,
            pong,
            frame_count: 0,
        }
    }

    /// Returns a reference to the input state (for testing).
    #[must_use]
    pub const fn input(&self) -> &InputState {
        &self.input
    }

    /// Returns a mutable reference to the input state (for testing).
    #[allow(clippy::missing_const_for_fn)] // const fn with mutable references not yet stable
    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    /// Returns a reference to the frame timer (for testing).
    #[must_use]
    pub const fn timer(&self) -> &FrameTimer {
        &self.timer
    }

    /// Returns a reference to the Pong game (for testing).
    #[must_use]
    pub const fn pong(&self) -> &PongGame {
        &self.pong
    }

    /// Returns a reference to the config (for testing).
    #[must_use]
    pub const fn config(&self) -> &WebConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_web_config_default() {
        let config = WebConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.target_fps, 60);
        assert!(!config.debug);
    }

    #[test]
    fn test_web_config_new() {
        let config = WebConfig::new(1920, 1080);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_web_config_from_json() {
        let json = r#"{"width":1024,"height":768,"target_fps":30,"debug":true}"#;
        let config = WebConfig::from_json(json).unwrap();
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.target_fps, 30);
        assert!(config.debug);
    }

    #[test]
    fn test_web_config_from_json_invalid() {
        let result = WebConfig::from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_web_config_to_json() {
        let config = WebConfig::new(800, 600);
        let json = config.to_json().unwrap();
        assert!(json.contains("800"));
        assert!(json.contains("600"));
    }

    #[test]
    fn test_web_platform_error_display() {
        let err = WebPlatformError::InvalidConfig("bad config".to_string());
        assert_eq!(err.to_string(), "Invalid config: bad config");

        let err = WebPlatformError::InputError("input failed".to_string());
        assert_eq!(err.to_string(), "Input error: input failed");

        let err = WebPlatformError::RenderError("render failed".to_string());
        assert_eq!(err.to_string(), "Render error: render failed");
    }

    #[test]
    fn test_web_platform_error_from_input_error() {
        let input_err = InputTranslationError::InvalidJson("test".to_string());
        let platform_err: WebPlatformError = input_err.into();
        assert!(matches!(platform_err, WebPlatformError::InputError(_)));
    }

    #[test]
    fn test_pong_game_new() {
        let game = PongGame::new(800.0, 600.0, false);
        assert!((game.width - 800.0).abs() < f32::EPSILON);
        assert!((game.height - 600.0).abs() < f32::EPSILON);
        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
    }

    #[test]
    fn test_pong_game_default() {
        let game = PongGame::default();
        assert!((game.width - 800.0).abs() < f32::EPSILON);
        assert!((game.height - 600.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pong_game_ball_position() {
        let game = PongGame::new(800.0, 600.0, false);
        let (x, y) = game.ball_position();
        assert!((x - 400.0).abs() < f32::EPSILON);
        assert!((y - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pong_game_update_no_input() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let input = InputState::new();
        let initial_ball_x = game.ball_x;

        game.update(&input, 0.016);

        // Ball should have moved
        assert!((game.ball_x - initial_ball_x).abs() > 1.0);
    }

    #[test]
    fn test_pong_game_paddle_movement() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let mut input = InputState::new();
        let initial_y = game.left_paddle_y;

        // Press W key
        input.set_key_pressed(jugar_input::KeyCode::Letter('W'), true);
        game.update(&input, 0.1);

        // Paddle should have moved up
        assert!(game.left_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_game_paddle_clamping() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let mut input = InputState::new();

        // Press W key many times to hit top
        input.set_key_pressed(jugar_input::KeyCode::Letter('W'), true);
        for _ in 0..100 {
            game.update(&input, 0.1);
        }

        // Paddle should be clamped to screen
        let half_paddle = game.paddle_height / 2.0;
        assert!(game.left_paddle_y >= half_paddle);
    }

    #[test]
    fn test_pong_game_render() {
        let game = PongGame::new(800.0, 600.0, false);
        let mut frame = RenderFrame::new();

        game.render(&mut frame);

        // Should have several commands (clear, center line dashes, 2 paddles, ball, 2 scores)
        assert!(frame.len() > 5);
    }

    #[test]
    fn test_pong_game_resize() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.resize(1600, 1200);

        assert!((game.width - 1600.0).abs() < f32::EPSILON);
        assert!((game.height - 1200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_web_platform_new_for_test() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        assert_eq!(platform.config().width, 800);
        assert_eq!(platform.config().height, 600);
    }

    #[test]
    fn test_web_platform_frame() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // First frame
        let result = platform.frame(0.0, "[]");
        assert!(!result.is_empty());
        assert!(result.contains("Clear"));

        // Second frame
        let result = platform.frame(16.667, "[]");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_web_platform_frame_with_input() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let input_json = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}}]"#;
        let _ = platform.frame(0.0, input_json);

        // Key should be registered
        assert!(platform
            .input()
            .is_key_pressed(jugar_input::KeyCode::Letter('W')));
    }

    #[test]
    fn test_web_platform_resize() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        platform.resize(1920, 1080);

        assert_eq!(platform.config().width, 1920);
        assert_eq!(platform.config().height, 1080);
    }

    #[test]
    fn test_web_platform_get_config() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let config_json = platform.get_config();
        assert!(config_json.contains("800"));
        assert!(config_json.contains("600"));
    }

    #[test]
    fn test_web_platform_get_stats() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Run a few frames
        let _ = platform.frame(0.0, "[]");
        let _ = platform.frame(16.667, "[]");
        let _ = platform.frame(33.333, "[]");

        let stats = platform.get_stats();
        assert!(stats.contains("fps"));
        assert!(stats.contains("frame_count"));
    }

    #[test]
    fn test_web_platform_reset_timer() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");
        let _ = platform.frame(1000.0, "[]");

        platform.reset_timer();

        assert_eq!(platform.timer().frame_count(), 0);
    }

    #[test]
    fn test_web_platform_debug_mode() {
        let config = WebConfig {
            debug: true,
            ..WebConfig::default()
        };
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");
        let result = platform.frame(16.667, "[]");

        // Debug text should be rendered
        assert!(result.contains("FPS"));
    }

    #[test]
    fn test_web_platform_input_accessors() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Test input_mut
        platform
            .input_mut()
            .set_key_pressed(jugar_input::KeyCode::Space, true);

        // Test input
        assert!(platform.input().is_key_pressed(jugar_input::KeyCode::Space));
    }

    #[test]
    fn test_web_platform_pong_accessor() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let pong = platform.pong();
        assert_eq!(pong.left_score(), 0);
        assert_eq!(pong.right_score(), 0);
    }

    #[test]
    fn test_pong_ball_wall_collision_top() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_y = 5.0; // Near top
        game.ball_vy = -100.0; // Moving up

        let input = InputState::new();
        game.update(&input, 0.1);

        // Ball should have bounced
        assert!(game.ball_vy > 0.0);
    }

    #[test]
    fn test_pong_ball_wall_collision_bottom() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_y = 595.0; // Near bottom
        game.ball_vy = 100.0; // Moving down

        let input = InputState::new();
        game.update(&input, 0.1);

        // Ball should have bounced
        assert!(game.ball_vy < 0.0);
    }

    #[test]
    fn test_pong_scoring_right() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_x = -10.0; // Past left edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // Right player should score
        assert_eq!(game.right_score(), 1);
    }

    #[test]
    fn test_pong_scoring_left() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.ball_x = 810.0; // Past right edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // Left player should score
        assert_eq!(game.left_score(), 1);
    }

    #[test]
    fn test_frame_output_serialization() {
        let output = FrameOutput {
            commands: vec![Canvas2DCommand::Clear {
                color: Color::BLACK,
            }],
            audio_events: vec![],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("commands"));
        assert!(!json.contains("debug_info")); // Skipped when None
        assert!(!json.contains("audio_events")); // Skipped when empty
    }

    #[test]
    fn test_frame_output_with_debug() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![],
            debug_info: Some(DebugInfo {
                dt_ms: 16.667,
                fps: 60.0,
                frame_count: 100,
                input_summary: "W pressed".to_string(),
            }),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("debug_info"));
        assert!(json.contains("fps"));
    }

    #[test]
    fn test_frame_output_with_audio() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![AudioEvent::GameStart { volume: 0.7 }],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("audio_events"));
        assert!(json.contains("GameStart"));
    }

    #[test]
    fn test_debug_info_serialization() {
        let info = DebugInfo {
            dt_ms: 16.667,
            fps: 60.0,
            frame_count: 42,
            input_summary: "test".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("16.667"));
        assert!(json.contains("60"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_pong_right_paddle_controls() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press ArrowDown key
        input.set_key_pressed(jugar_input::KeyCode::Down, true);
        game.update(&input, 0.1);

        // Right paddle should have moved down
        assert!(game.right_paddle_y > initial_y);
    }

    #[test]
    fn test_pong_right_paddle_up() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press ArrowUp key
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Right paddle should have moved up
        assert!(game.right_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_left_paddle_down() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        let mut input = InputState::new();
        let initial_y = game.left_paddle_y;

        // Press S key
        input.set_key_pressed(jugar_input::KeyCode::Letter('S'), true);
        game.update(&input, 0.1);

        // Left paddle should have moved down
        assert!(game.left_paddle_y > initial_y);
    }

    #[test]
    fn test_pong_paddle_collision_left() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
                                            // Position ball to hit left paddle (paddle at x=20..35)
                                            // Ball radius is 10, so ball_x - radius needs to be in (20, 35)
                                            // ball_x = 46 means left edge at 36, after moving left 1.6px (vx=-100, dt=0.016)
                                            // left edge becomes 36-1.6=34.4, which is in (20, 35)
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0; // Slower velocity so we stay in collision zone

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should have bounced (velocity reversed to positive)
        assert!(game.ball_vx > 0.0);
    }

    #[test]
    fn test_pong_paddle_collision_right() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
                                            // Position ball to hit right paddle (paddle at x=765..780)
                                            // Ball radius is 10, so ball_x + radius needs to be in (765, 780)
                                            // ball_x = 758 means right edge at 768, which is in (765, 780)
        game.ball_x = 758.0;
        game.ball_y = game.right_paddle_y;
        game.ball_vx = 200.0;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should have bounced (velocity reversed to negative)
        assert!(game.ball_vx < 0.0);
    }

    #[test]
    fn test_pong_resize_scales_positions() {
        let mut game = PongGame::new(800.0, 600.0, false);
        // Set ball to center
        game.ball_x = 400.0;
        game.ball_y = 300.0;

        game.resize(1600, 1200);

        // Ball should be scaled to new center
        assert!((game.ball_x - 800.0).abs() < 1.0);
        assert!((game.ball_y - 600.0).abs() < 1.0);
    }

    // =========================================================================
    // Game State Tests
    // =========================================================================

    #[test]
    fn test_game_state_default_is_menu() {
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.state(), GameState::Menu);
    }

    #[test]
    fn test_game_state_start_transitions_to_playing() {
        let mut game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.state(), GameState::Menu);

        game.start();

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_pause_transitions() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        // Press Escape to pause
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Escape, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Paused);
    }

    #[test]
    fn test_game_state_unpause_with_space() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);

        // First update with space not pressed (to set prev state)
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_menu_space_starts_game() {
        let mut game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.state(), GameState::Menu);

        // First update with space not pressed (to set prev state)
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_game_over_on_winning_score() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.left_score = 10; // One away from winning
        game.ball_x = 810.0; // Past right edge - left player scores

        let input = InputState::new();
        game.update(&input, 0.016);

        assert_eq!(game.left_score(), 11);
        assert_eq!(game.state(), GameState::GameOver);
    }

    #[test]
    fn test_game_state_reset_from_game_over() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::GameOver);
        game.left_score = 11;
        game.right_score = 5;

        // First update with space not pressed
        let input = InputState::new();
        game.update(&input, 0.016);

        // Now press space to restart
        let mut input = InputState::new();
        input.set_key_pressed(jugar_input::KeyCode::Space, true);
        game.update(&input, 0.016);

        assert_eq!(game.state(), GameState::Playing);
        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
    }

    #[test]
    fn test_game_state_no_update_when_paused() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);
        let initial_ball_x = game.ball_x;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Ball should not have moved
        assert!((game.ball_x - initial_ball_x).abs() < 0.001);
    }

    #[test]
    fn test_game_state_reset_game() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.left_score = 5;
        game.right_score = 3;
        game.ball_x = 100.0;

        game.reset_game();

        assert_eq!(game.left_score(), 0);
        assert_eq!(game.right_score(), 0);
        assert!((game.ball_x - 400.0).abs() < 1.0); // Back to center
    }

    // =========================================================================
    // Rally Counter Tests
    // =========================================================================

    #[test]
    fn test_rally_counter_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.rally_count(), 0);
    }

    #[test]
    fn test_rally_counter_increments_on_paddle_hit() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        assert_eq!(game.rally_count(), 0);

        // Position ball to hit left paddle
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0;

        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have incremented
        assert_eq!(game.rally_count(), 1);
    }

    #[test]
    fn test_rally_counter_resets_on_scoring() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.rally_count = 5; // Simulate some rallies
        game.ball_x = -10.0; // Ball past left edge - right player scores

        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have reset to 0
        assert_eq!(game.rally_count(), 0);
    }

    #[test]
    fn test_rally_counter_resets_on_game_reset() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.rally_count = 10;

        game.reset_game();

        assert_eq!(game.rally_count(), 0);
    }

    // =========================================================================
    // High Score Tests
    // =========================================================================

    #[test]
    fn test_high_score_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.high_score(), 0);
    }

    #[test]
    fn test_high_score_updates_when_rally_ends() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.rally_count = 8;
        game.ball_x = -10.0; // Ball past left edge - triggers scoring

        let input = InputState::new();
        game.update(&input, 0.016);

        // High score should be 8 (the rally count before reset)
        assert_eq!(game.high_score(), 8);
    }

    #[test]
    fn test_high_score_only_updates_if_higher() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.high_score = 10;
        game.rally_count = 5;
        game.ball_x = -10.0; // Ball past left edge

        let input = InputState::new();
        game.update(&input, 0.016);

        // High score should still be 10 (higher than 5)
        assert_eq!(game.high_score(), 10);
    }

    #[test]
    fn test_high_score_persists_across_reset() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.high_score = 15;

        game.reset_game();

        // High score should persist
        assert_eq!(game.high_score(), 15);
    }

    #[test]
    fn test_high_score_updates_on_both_sides_scoring() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        // Left player misses
        game.rally_count = 7;
        game.ball_x = -10.0;
        let input = InputState::new();
        game.update(&input, 0.016);
        assert_eq!(game.high_score(), 7);

        // Reset ball and rally
        game.ball_x = 400.0;

        // Now right player misses with a higher rally
        game.rally_count = 12;
        game.ball_x = 810.0;
        game.update(&input, 0.016);
        assert_eq!(game.high_score(), 12);
    }

    // =========================================================================
    // Background Animation Tests
    // =========================================================================

    #[test]
    fn test_bg_time_starts_at_zero() {
        let game = PongGame::new(800.0, 600.0, false);
        assert!((game.bg_time - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bg_time_increments_during_gameplay() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        assert!((game.bg_time - 0.0).abs() < f32::EPSILON);

        let input = InputState::new();
        game.update(&input, 0.5);

        assert!((game.bg_time - 0.5).abs() < 0.001);
    }
}
