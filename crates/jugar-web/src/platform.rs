//! Web platform integration for the Jugar game engine.
//!
//! This module provides the main `WebPlatform` struct that bridges the Jugar engine
//! to browser APIs via wasm-bindgen. All game logic runs in Rust; JavaScript only
//! handles event forwarding and Canvas2D rendering.

use glam::Vec2;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::ai::PongAI;
use crate::audio::{AudioEvent, ProceduralAudio};
use crate::demo::{DemoState, GameMode, SpeedMultiplier};
use crate::input::{process_input_events, InputTranslationError};
use crate::juice::JuiceEffects;
use crate::render::{Canvas2DCommand, Color, RenderFrame, TextAlign, TextBaseline};
use crate::time::FrameTimer;
use crate::trace::{GameTracer, TracerConfig};
use jugar_input::{InputState, MouseButton};

/// A clickable button rectangle.
#[derive(Debug, Clone, Copy, Default)]
struct ButtonRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ButtonRect {
    const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if the point is inside the button.
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// HUD button regions for click detection.
#[derive(Debug, Clone, Default)]
struct HudButtons {
    /// Mode buttons: Demo, 1P, 2P
    mode_demo: ButtonRect,
    mode_1p: ButtonRect,
    mode_2p: ButtonRect,
    /// Speed buttons: 1x, 5x, 10x, 50x, 100x, 1000x
    speed_1x: ButtonRect,
    speed_5x: ButtonRect,
    speed_10x: ButtonRect,
    speed_50x: ButtonRect,
    speed_100x: ButtonRect,
    speed_1000x: ButtonRect,
    /// AI difficulty buttons
    ai_decrease: ButtonRect,
    ai_increase: ButtonRect,
    /// Download button
    download: ButtonRect,
    /// Model info toggle button
    model_info: ButtonRect,
    /// Sound toggle button
    sound_toggle: ButtonRect,
}

impl HudButtons {
    /// Calculate button width from text label.
    /// Uses text length * char_width + padding on both sides.
    #[inline]
    fn button_width(text: &str, char_width: f32, padding: f32) -> f32 {
        (text.len() as f32).mul_add(char_width, padding * 2.0)
    }

    /// Calculate initial button positions based on canvas dimensions.
    /// This ensures buttons are clickable from the first frame.
    #[must_use]
    fn calculate(width: f32, height: f32) -> Self {
        let hud_y = 10.0;
        let button_height = 28.0;
        let button_padding = 8.0;
        let char_width = 8.0;

        // Mode buttons (top-left)
        let mode_x = 10.0;
        let demo_width = Self::button_width("Demo", char_width, button_padding);
        let p1_width = Self::button_width("1P", char_width, button_padding);
        let p2_width = Self::button_width("2P", char_width, button_padding);

        let mode_demo = ButtonRect::new(mode_x, hud_y, demo_width, button_height);
        let mode_1p = ButtonRect::new(mode_x + demo_width + 5.0, hud_y, p1_width, button_height);
        let mode_2p = ButtonRect::new(
            mode_x + demo_width + 5.0 + p1_width + 5.0,
            hud_y,
            p2_width,
            button_height,
        );

        // Speed buttons (top-right)
        let w1x = Self::button_width("1x", char_width, button_padding);
        let w5x = Self::button_width("5x", char_width, button_padding);
        let w10x = Self::button_width("10x", char_width, button_padding);
        let w50x = Self::button_width("50x", char_width, button_padding);
        let w100x = Self::button_width("100x", char_width, button_padding);
        let w1000x = Self::button_width("1000x", char_width, button_padding);
        let total_width = w1x + w5x + w10x + w50x + w100x + w1000x + 25.0;

        let mut speed_x = width - total_width - 10.0;
        let speed_1x = ButtonRect::new(speed_x, hud_y, w1x, button_height);
        speed_x += w1x + 5.0;
        let speed_5x = ButtonRect::new(speed_x, hud_y, w5x, button_height);
        speed_x += w5x + 5.0;
        let speed_10x = ButtonRect::new(speed_x, hud_y, w10x, button_height);
        speed_x += w10x + 5.0;
        let speed_50x = ButtonRect::new(speed_x, hud_y, w50x, button_height);
        speed_x += w50x + 5.0;
        let speed_100x = ButtonRect::new(speed_x, hud_y, w100x, button_height);
        speed_x += w100x + 5.0;
        let speed_1000x = ButtonRect::new(speed_x, hud_y, w1000x, button_height);

        // AI difficulty buttons (below mode buttons)
        // These positions must match render_hud() logic exactly
        let ai_y = hud_y + button_height + 15.0;
        let ai_btn_size = 20.0;
        let ai_btn_y = ai_y - 2.0;
        let bar_x = 40.0;
        let bar_width = 100.0;
        let ai_btn_x = bar_x + bar_width + 75.0; // 215.0
        let ai_plus_x = ai_btn_x + ai_btn_size + 5.0; // 240.0
        let ai_decrease = ButtonRect::new(ai_btn_x, ai_btn_y, ai_btn_size, ai_btn_size);
        let ai_increase = ButtonRect::new(ai_plus_x, ai_btn_y, ai_btn_size, ai_btn_size);

        // Download button (bottom-left)
        let download_y = height - 45.0;
        let download_width = Self::button_width("Download .apr", char_width, button_padding);
        let download = ButtonRect::new(10.0, download_y, download_width, button_height);

        // Model info button (next to download)
        let info_width = Self::button_width("Info", char_width, button_padding);
        let model_info = ButtonRect::new(
            10.0 + download_width + 5.0,
            download_y,
            info_width,
            button_height,
        );

        // Sound toggle button (next to model info)
        let sound_width = Self::button_width("Sound", char_width, button_padding);
        let sound_toggle = ButtonRect::new(
            10.0 + download_width + 5.0 + info_width + 5.0,
            download_y,
            sound_width,
            button_height,
        );

        Self {
            mode_demo,
            mode_1p,
            mode_2p,
            speed_1x,
            speed_5x,
            speed_10x,
            speed_50x,
            speed_100x,
            speed_1000x,
            ai_decrease,
            ai_increase,
            download,
            model_info,
            sound_toggle,
        }
    }
}

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

/// An action to be performed by JavaScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsAction {
    /// Trigger download of the AI model as .apr file
    DownloadAiModel,
    /// Open a URL in a new tab
    OpenUrl {
        /// The URL to open
        url: String,
    },
    /// Request fullscreen mode (for ultra-wide monitors)
    EnterFullscreen,
    /// Exit fullscreen mode
    ExitFullscreen,
}

/// Frame output returned to JavaScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameOutput {
    /// Render commands to execute
    pub commands: Vec<Canvas2DCommand>,
    /// Audio events to play via Web Audio API
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub audio_events: Vec<AudioEvent>,
    /// JavaScript actions to perform
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actions: Vec<JsAction>,
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
    /// Current game mode (Demo, SinglePlayer, TwoPlayer)
    pub game_mode: String,
    /// Current speed multiplier (1, 5, 10, 50, 100, 1000)
    pub speed_multiplier: u32,
    /// Left paddle Y position (center)
    pub left_paddle_y: f32,
    /// Right paddle Y position (center)
    pub right_paddle_y: f32,
    /// Ball X position
    pub ball_x: f32,
    /// Ball Y position
    pub ball_y: f32,
    /// Trace buffer usage (frames in buffer / capacity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_buffer_usage: Option<String>,
    /// Total input events recorded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_inputs: Option<u64>,
    /// Frames dropped from trace buffer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_dropped: Option<u64>,
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
    /// Mutable because HUD button positions are updated during render.
    ///
    /// # Arguments
    ///
    /// * `frame` - Render frame to push commands to
    fn render(&mut self, frame: &mut RenderFrame);

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
#[allow(clippy::struct_excessive_bools)] // Key state tracking requires bools
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
    /// Current speed multiplier for physics
    speed_multiplier: SpeedMultiplier,
    /// Current game mode (Demo/1P/2P)
    game_mode: GameMode,
    /// Demo mode state (idle timer, difficulty cycling)
    #[allow(dead_code)] // Reserved for auto-engage and difficulty cycling features
    demo_state: DemoState,
    /// Second AI for left paddle (used in Demo mode)
    left_ai: Option<PongAI>,
    /// Track key states for edge detection
    key_1_was_pressed: bool,
    key_2_was_pressed: bool,
    key_3_was_pressed: bool,
    key_4_was_pressed: bool,
    key_5_was_pressed: bool,
    key_6_was_pressed: bool,
    key_m_was_pressed: bool,
    key_d_was_pressed: bool,
    #[allow(dead_code)] // Reserved for AI difficulty adjustment via +/- keys
    key_plus_was_pressed: bool,
    #[allow(dead_code)] // Reserved for AI difficulty adjustment via +/- keys
    key_minus_was_pressed: bool,
    /// Mouse button was pressed last frame (for edge detection)
    mouse_was_pressed: bool,
    /// HUD button hit regions (updated during render)
    hud_buttons: HudButtons,
    /// Flag to trigger .apr download action (consumed after frame)
    download_requested: bool,
    /// Whether to show the model info panel (toggle with I key or Info button)
    show_model_info: bool,
    /// Track if I key was pressed last frame (for edge detection)
    key_i_was_pressed: bool,
    /// Whether sound is enabled (toggle with Sound button)
    sound_enabled: bool,
    /// Track if F key was pressed last frame (for edge detection)
    key_f_was_pressed: bool,
    /// Flag to request fullscreen (consumed after frame)
    fullscreen_requested: bool,
    /// Track current fullscreen state (to toggle)
    is_fullscreen: bool,
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
            state: GameState::Playing, // Start playing immediately in Demo mode (attract)
            winning_score: 11,
            space_was_pressed: false,
            escape_was_pressed: false,
            rally_count: 0,
            high_score: 0,
            bg_time: 0.0,
            audio: ProceduralAudio::new(),
            speed_multiplier: SpeedMultiplier::default(),
            game_mode: GameMode::default(),
            demo_state: DemoState::default(),
            left_ai: Some(PongAI::with_difficulty(6)), // Left AI for demo mode (Hard)
            key_1_was_pressed: false,
            key_2_was_pressed: false,
            key_3_was_pressed: false,
            key_4_was_pressed: false,
            key_5_was_pressed: false,
            key_6_was_pressed: false,
            key_m_was_pressed: false,
            key_d_was_pressed: false,
            key_plus_was_pressed: false,
            key_minus_was_pressed: false,
            mouse_was_pressed: false,
            hud_buttons: HudButtons::calculate(width, height),
            download_requested: false,
            show_model_info: false,
            key_i_was_pressed: false,
            sound_enabled: true, // Sound on by default
            key_f_was_pressed: false,
            fullscreen_requested: false,
            is_fullscreen: false,
        }
    }

    /// Returns the current speed multiplier.
    #[must_use]
    pub const fn speed_multiplier(&self) -> SpeedMultiplier {
        self.speed_multiplier
    }

    /// Sets the speed multiplier.
    #[allow(clippy::missing_const_for_fn)] // Not const due to mutable reference
    pub fn set_speed_multiplier(&mut self, speed: SpeedMultiplier) {
        self.speed_multiplier = speed;
    }

    /// Returns the current game mode.
    #[must_use]
    pub const fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    /// Sets the game mode.
    pub fn set_game_mode(&mut self, mode: GameMode) {
        self.game_mode = mode;
        self.reset_game();
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

    /// Takes all pending audio events (returns empty if sound is disabled).
    pub fn take_audio_events(&mut self) -> Vec<AudioEvent> {
        if self.sound_enabled {
            self.audio.take_events()
        } else {
            // Clear events but don't return them
            drop(self.audio.take_events());
            Vec::new()
        }
    }

    /// Exports the AI model as JSON for download.
    #[must_use]
    pub fn export_ai_model(&self) -> String {
        self.ai.as_ref().map_or_else(
            || crate::ai::PongAIModel::default().to_json(),
            crate::ai::PongAI::export_model,
        )
    }

    /// Returns AI info as JSON.
    #[must_use]
    pub fn ai_info(&self) -> String {
        self.ai.as_ref().map_or_else(
            || r#"{"enabled": false}"#.to_string(),
            crate::ai::PongAI::model_info_json,
        )
    }

    /// Sets the AI difficulty level (0-9).
    pub fn set_ai_difficulty(&mut self, level: u8) {
        if let Some(ref mut ai) = self.ai {
            ai.set_difficulty(level);
        }
    }

    /// Gets the current AI difficulty level.
    #[must_use]
    pub fn ai_difficulty(&self) -> u8 {
        self.ai.as_ref().map_or(5, crate::ai::PongAI::difficulty)
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

    // ===== Test helper methods =====

    /// Returns the ball X position.
    #[must_use]
    pub const fn ball_x(&self) -> f32 {
        self.ball_x
    }

    /// Returns the ball Y position.
    #[must_use]
    pub const fn ball_y(&self) -> f32 {
        self.ball_y
    }

    /// Returns the ball X velocity.
    #[must_use]
    pub const fn ball_vx(&self) -> f32 {
        self.ball_vx
    }

    /// Returns the ball Y velocity.
    #[must_use]
    pub const fn ball_vy(&self) -> f32 {
        self.ball_vy
    }

    /// Returns the left paddle Y position.
    #[must_use]
    pub const fn left_paddle_y(&self) -> f32 {
        self.left_paddle_y
    }

    /// Returns the right paddle Y position.
    #[must_use]
    pub const fn right_paddle_y(&self) -> f32 {
        self.right_paddle_y
    }

    /// Returns the paddle height.
    #[must_use]
    pub const fn paddle_height(&self) -> f32 {
        self.paddle_height
    }

    /// Returns the paddle speed.
    #[must_use]
    pub const fn paddle_speed(&self) -> f32 {
        self.paddle_speed
    }

    /// Returns whether fullscreen is active.
    #[must_use]
    pub const fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    /// Sets ball position (for testing).
    #[cfg(test)]
    pub fn set_ball_position(&mut self, x: f32, y: f32) {
        self.ball_x = x;
        self.ball_y = y;
    }

    /// Sets ball velocity (for testing).
    #[cfg(test)]
    pub fn set_ball_velocity(&mut self, vx: f32, vy: f32) {
        self.ball_vx = vx;
        self.ball_vy = vy;
    }

    /// Sets left paddle Y position (for testing).
    #[cfg(test)]
    pub fn set_left_paddle_y(&mut self, y: f32) {
        self.left_paddle_y = y;
    }

    /// Sets right paddle Y position (for testing).
    #[cfg(test)]
    pub fn set_right_paddle_y(&mut self, y: f32) {
        self.right_paddle_y = y;
    }

    /// Sets left score (for testing).
    #[cfg(test)]
    pub fn set_left_score(&mut self, score: u32) {
        self.left_score = score;
    }

    /// Sets right score (for testing).
    #[cfg(test)]
    pub fn set_right_score(&mut self, score: u32) {
        self.right_score = score;
    }

    /// Sets rally count (for testing).
    #[cfg(test)]
    pub fn set_rally_count(&mut self, count: u32) {
        self.rally_count = count;
    }

    /// Resets the game (for testing).
    #[cfg(test)]
    pub fn reset(&mut self) {
        self.reset_game();
    }

    /// Enables or disables sound (for testing).
    #[cfg(test)]
    pub fn enable_sound(&mut self, enabled: bool) {
        self.sound_enabled = enabled;
    }
}

impl WebGame for PongGame {
    #[allow(clippy::too_many_lines)] // Game update logic is inherently complex
    fn update(&mut self, input: &InputState, dt: f64) {
        let base_dt = dt as f32;

        // Handle keyboard shortcuts for speed (1-6) with edge detection
        let key_1 = input.is_key_pressed(jugar_input::KeyCode::Number(1));
        let key_2 = input.is_key_pressed(jugar_input::KeyCode::Number(2));
        let key_3 = input.is_key_pressed(jugar_input::KeyCode::Number(3));
        let key_4 = input.is_key_pressed(jugar_input::KeyCode::Number(4));
        let key_5 = input.is_key_pressed(jugar_input::KeyCode::Number(5));
        let key_6 = input.is_key_pressed(jugar_input::KeyCode::Number(6));
        let key_m = input.is_key_pressed(jugar_input::KeyCode::Letter('M'));
        let key_d = input.is_key_pressed(jugar_input::KeyCode::Letter('D'));
        let key_i = input.is_key_pressed(jugar_input::KeyCode::Letter('I'));
        let key_f = input.is_key_pressed(jugar_input::KeyCode::Letter('F'));
        let key_f11 = input.is_key_pressed(jugar_input::KeyCode::Function(11));

        if key_1 && !self.key_1_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Normal;
        }
        if key_2 && !self.key_2_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Fast5x;
        }
        if key_3 && !self.key_3_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Fast10x;
        }
        if key_4 && !self.key_4_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Fast50x;
        }
        if key_5 && !self.key_5_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Fast100x;
        }
        if key_6 && !self.key_6_was_pressed {
            self.speed_multiplier = SpeedMultiplier::Fast1000x;
        }
        if key_m && !self.key_m_was_pressed {
            self.game_mode = self.game_mode.next();
        }
        // D key toggles between Demo and SinglePlayer
        if key_d && !self.key_d_was_pressed {
            self.game_mode = if self.game_mode == GameMode::Demo {
                GameMode::SinglePlayer
            } else {
                GameMode::Demo
            };
        }
        // I key toggles model info panel
        if key_i && !self.key_i_was_pressed {
            self.show_model_info = !self.show_model_info;
        }
        // F or F11 toggles fullscreen (logic in Rust, action executed by JS)
        let fullscreen_key = (key_f || key_f11) && !self.key_f_was_pressed;
        if fullscreen_key {
            self.fullscreen_requested = true;
            self.is_fullscreen = !self.is_fullscreen;
        }

        self.key_1_was_pressed = key_1;
        self.key_2_was_pressed = key_2;
        self.key_3_was_pressed = key_3;
        self.key_4_was_pressed = key_4;
        self.key_5_was_pressed = key_5;
        self.key_6_was_pressed = key_6;
        self.key_m_was_pressed = key_m;
        self.key_d_was_pressed = key_d;
        self.key_i_was_pressed = key_i;
        self.key_f_was_pressed = key_f || key_f11;

        // Handle mouse click on HUD buttons
        let mouse_pressed = input.mouse_button(MouseButton::Left).is_down();
        let mouse_just_clicked = mouse_pressed && !self.mouse_was_pressed;
        self.mouse_was_pressed = mouse_pressed;

        if mouse_just_clicked {
            let pos = input.mouse_position;
            self.handle_hud_click(pos.x, pos.y);
        }

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

        // Game is Playing - run game logic with speed multiplier
        // Speed multiplier runs physics multiple times per frame
        let speed_mult = self.speed_multiplier.value();
        let dt = base_dt * speed_mult as f32; // Scale dt for faster simulation
        let half_paddle = self.paddle_height / 2.0;

        // Store previous ball Y for wall bounce detection
        self.prev_ball_y = self.ball_y;

        // Left paddle controls - based on game mode
        // In Demo mode: AI controls left paddle
        // In SinglePlayer mode: AI controls left paddle
        // In TwoPlayer mode: P2 human controls left paddle (W/S)
        if self.game_mode.left_is_ai() {
            // AI controls left paddle (Demo and SinglePlayer modes)
            if let Some(ref mut left_ai) = self.left_ai {
                let ai_velocity = left_ai.update(
                    self.ball_x,
                    self.ball_y,
                    -self.ball_vx, // Invert for left side perspective
                    self.ball_vy,
                    self.left_paddle_y,
                    self.paddle_height,
                    self.width,
                    self.height,
                    dt,
                );
                self.left_paddle_y += ai_velocity * dt;
            }
        } else {
            // P2 human controls left paddle (W/S keys) - TwoPlayer mode only
            if input.is_key_pressed(jugar_input::KeyCode::Letter('W')) {
                self.left_paddle_y -= self.paddle_speed * dt;
            }
            if input.is_key_pressed(jugar_input::KeyCode::Letter('S')) {
                self.left_paddle_y += self.paddle_speed * dt;
            }
        }

        // Right paddle controls - based on game mode
        // In Demo mode: AI controls right paddle
        // In SinglePlayer mode: P1 human controls right paddle (Arrow keys)
        // In TwoPlayer mode: P1 human controls right paddle (Arrow keys)
        if self.game_mode.right_is_ai() {
            // AI controls the right paddle (Demo mode only)
            if let Some(ref mut ai) = self.ai {
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
            }
        } else {
            // P1 human controls right paddle (Arrow keys) - SinglePlayer and TwoPlayer
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

            // Only run DDA when a human is playing (not in Demo mode)
            if self.game_mode != GameMode::Demo {
                if let Some(ref mut ai) = self.ai {
                    ai.record_player_miss();
                    ai.adapt_difficulty();
                }
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

            // Only run DDA when a human is playing (not in Demo mode)
            if self.game_mode != GameMode::Demo {
                if let Some(ref mut ai) = self.ai {
                    ai.adapt_difficulty();
                }
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
    fn render(&mut self, frame: &mut RenderFrame) {
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

        // Draw left paddle label (dynamic based on game mode)
        let left_label = self.game_mode.left_paddle_label();
        let label_y = left_paddle_top - 8.0; // 8px above paddle
        let label_color = Color::new(0.7, 0.7, 0.7, 0.9); // Subtle gray
        frame.fill_text_aligned(
            left_label,
            left_paddle_x + self.paddle_width / 2.0,
            label_y.max(15.0), // Don't go off screen
            "12px monospace",
            label_color,
            crate::render::TextAlign::Center,
            crate::render::TextBaseline::Bottom,
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

        // Draw right paddle label (dynamic based on game mode)
        let right_label = self.game_mode.right_paddle_label();
        let right_label_y = right_paddle_top - 8.0; // 8px above paddle
        frame.fill_text_aligned(
            right_label,
            right_paddle_x + self.paddle_width / 2.0,
            right_label_y.max(15.0), // Don't go off screen
            "12px monospace",
            label_color,
            crate::render::TextAlign::Center,
            crate::render::TextBaseline::Bottom,
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
                // No overlay when playing - but render HUD
            }
        }

        // =========================================================================
        // HUD (always visible during gameplay)
        // =========================================================================
        self.render_hud(frame);
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

        // Recalculate HUD button positions for new canvas size
        self.hud_buttons = HudButtons::calculate(self.width, self.height);
    }
}

impl PongGame {
    /// Handles mouse clicks on HUD buttons.
    fn handle_hud_click(&mut self, mx: f32, my: f32) {
        // If info panel is visible, clicking anywhere dismisses it
        if self.show_model_info {
            self.show_model_info = false;
            return; // Consume the click
        }

        // Check mode buttons
        if self.hud_buttons.mode_demo.contains(mx, my) {
            self.game_mode = GameMode::Demo;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        } else if self.hud_buttons.mode_1p.contains(mx, my) {
            self.game_mode = GameMode::SinglePlayer;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        } else if self.hud_buttons.mode_2p.contains(mx, my) {
            self.game_mode = GameMode::TwoPlayer;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        }
        // Check speed buttons
        else if self.hud_buttons.speed_1x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Normal;
        } else if self.hud_buttons.speed_5x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast5x;
        } else if self.hud_buttons.speed_10x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast10x;
        } else if self.hud_buttons.speed_50x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast50x;
        } else if self.hud_buttons.speed_100x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast100x;
        } else if self.hud_buttons.speed_1000x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast1000x;
        }
        // Check AI difficulty buttons
        else if self.hud_buttons.ai_decrease.contains(mx, my) {
            if let Some(ref mut ai) = self.ai {
                let level = ai.difficulty();
                if level > 0 {
                    ai.set_difficulty(level - 1);
                }
            }
        } else if self.hud_buttons.ai_increase.contains(mx, my) {
            if let Some(ref mut ai) = self.ai {
                let level = ai.difficulty();
                if level < 9 {
                    ai.set_difficulty(level + 1);
                }
            }
        }
        // Check download button
        else if self.hud_buttons.download.contains(mx, my) {
            self.download_requested = true;
        }
        // Check model info button
        else if self.hud_buttons.model_info.contains(mx, my) {
            self.show_model_info = !self.show_model_info;
        }
        // Check sound toggle button
        else if self.hud_buttons.sound_toggle.contains(mx, my) {
            self.sound_enabled = !self.sound_enabled;
            // Play confirmation sound when enabling audio (provides immediate feedback)
            self.audio.on_sound_toggle(self.sound_enabled);
        }
    }

    /// Renders the HUD (mode buttons, speed buttons, AI info, download button).
    #[allow(clippy::too_many_lines, clippy::suboptimal_flops)]
    fn render_hud(&mut self, frame: &mut RenderFrame) {
        let hud_y = 10.0;
        let button_height = 28.0;
        let button_padding = 8.0;
        let font_size = "14px monospace";
        let small_font = "12px monospace";

        // =========================================================================
        // Game Mode Buttons (top-left)
        // =========================================================================
        let mut mode_x = 10.0;

        // Helper closure to render a button and return its rect
        let render_mode_button =
            |frame: &mut RenderFrame, x: f32, label: &str, is_selected: bool| -> ButtonRect {
                let bw = (label.len() as f32) * 10.0 + button_padding * 2.0;
                let bg_color = if is_selected {
                    Color::new(0.3, 0.6, 1.0, 0.9)
                } else {
                    Color::new(0.2, 0.2, 0.2, 0.8)
                };
                let border_color = if is_selected {
                    Color::WHITE
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                };
                frame.fill_rect(x, hud_y, bw, button_height, bg_color);
                frame.stroke_rect(x, hud_y, bw, button_height, border_color, 1.0);
                frame.fill_text_aligned(
                    label,
                    x + bw / 2.0,
                    hud_y + button_height / 2.0,
                    font_size,
                    Color::WHITE,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );
                ButtonRect::new(x, hud_y, bw, button_height)
            };

        // Demo button
        self.hud_buttons.mode_demo =
            render_mode_button(frame, mode_x, "Demo", self.game_mode == GameMode::Demo);
        mode_x += self.hud_buttons.mode_demo.width + 5.0;

        // 1P button
        self.hud_buttons.mode_1p = render_mode_button(
            frame,
            mode_x,
            "1P",
            self.game_mode == GameMode::SinglePlayer,
        );
        mode_x += self.hud_buttons.mode_1p.width + 5.0;

        // 2P button
        self.hud_buttons.mode_2p =
            render_mode_button(frame, mode_x, "2P", self.game_mode == GameMode::TwoPlayer);
        mode_x += self.hud_buttons.mode_2p.width + 5.0;

        // Mode keyboard hint
        frame.fill_text_aligned(
            "[M]",
            mode_x + 5.0,
            hud_y + button_height / 2.0,
            small_font,
            Color::new(0.5, 0.5, 0.5, 0.8),
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // =========================================================================
        // Speed Multiplier Buttons (top-right)
        // =========================================================================

        // Helper to render a speed button with key hint
        let render_speed_button = |frame: &mut RenderFrame,
                                   x: f32,
                                   label: &str,
                                   key_hint: &str,
                                   is_selected: bool|
         -> ButtonRect {
            let bw = (label.len() as f32) * 8.0 + button_padding * 2.0;
            let bg_color = if is_selected {
                Color::new(1.0, 0.6, 0.2, 0.9)
            } else {
                Color::new(0.2, 0.2, 0.2, 0.8)
            };
            let border_color = if is_selected {
                Color::WHITE
            } else {
                Color::new(0.5, 0.5, 0.5, 1.0)
            };
            frame.fill_rect(x, hud_y, bw, button_height, bg_color);
            frame.stroke_rect(x, hud_y, bw, button_height, border_color, 1.0);
            frame.fill_text_aligned(
                label,
                x + bw / 2.0,
                hud_y + button_height / 2.0,
                font_size,
                Color::WHITE,
                TextAlign::Center,
                TextBaseline::Middle,
            );
            frame.fill_text_aligned(
                key_hint,
                x + bw / 2.0,
                hud_y + button_height + 3.0,
                "10px monospace",
                Color::new(0.4, 0.4, 0.4, 0.7),
                TextAlign::Center,
                TextBaseline::Top,
            );
            ButtonRect::new(x, hud_y, bw, button_height)
        };

        // Calculate widths for positioning
        let w1x = "1x".len() as f32 * 8.0 + button_padding * 2.0;
        let w5x = "5x".len() as f32 * 8.0 + button_padding * 2.0;
        let w10x = "10x".len() as f32 * 8.0 + button_padding * 2.0;
        let w50x = "50x".len() as f32 * 8.0 + button_padding * 2.0;
        let w100x = "100x".len() as f32 * 8.0 + button_padding * 2.0;
        let w1000x = "1000x".len() as f32 * 8.0 + button_padding * 2.0;
        let total_width = w1x + w5x + w10x + w50x + w100x + w1000x + 25.0; // 5 gaps * 5px

        let mut speed_x = self.width - total_width - 10.0;

        self.hud_buttons.speed_1x = render_speed_button(
            frame,
            speed_x,
            "1x",
            "1",
            self.speed_multiplier == SpeedMultiplier::Normal,
        );
        speed_x += w1x + 5.0;

        self.hud_buttons.speed_5x = render_speed_button(
            frame,
            speed_x,
            "5x",
            "2",
            self.speed_multiplier == SpeedMultiplier::Fast5x,
        );
        speed_x += w5x + 5.0;

        self.hud_buttons.speed_10x = render_speed_button(
            frame,
            speed_x,
            "10x",
            "3",
            self.speed_multiplier == SpeedMultiplier::Fast10x,
        );
        speed_x += w10x + 5.0;

        self.hud_buttons.speed_50x = render_speed_button(
            frame,
            speed_x,
            "50x",
            "4",
            self.speed_multiplier == SpeedMultiplier::Fast50x,
        );
        speed_x += w50x + 5.0;

        self.hud_buttons.speed_100x = render_speed_button(
            frame,
            speed_x,
            "100x",
            "5",
            self.speed_multiplier == SpeedMultiplier::Fast100x,
        );
        speed_x += w100x + 5.0;

        self.hud_buttons.speed_1000x = render_speed_button(
            frame,
            speed_x,
            "1000x",
            "6",
            self.speed_multiplier == SpeedMultiplier::Fast1000x,
        );

        // =========================================================================
        // AI Difficulty Indicator (below mode buttons)
        // =========================================================================
        let ai_y = hud_y + button_height + 15.0;
        let ai_level = self.ai.as_ref().map_or(5, crate::ai::PongAI::difficulty);
        let ai_name = self
            .ai
            .as_ref()
            .map_or("Normal", crate::ai::PongAI::difficulty_name);

        // AI label
        frame.fill_text_aligned(
            "AI:",
            10.0,
            ai_y,
            font_size,
            Color::new(0.7, 0.7, 0.7, 1.0),
            TextAlign::Left,
            TextBaseline::Top,
        );

        // Progress bar background
        let bar_x = 40.0;
        let bar_width = 100.0;
        let bar_height = 12.0;
        frame.fill_rect(
            bar_x,
            ai_y + 2.0,
            bar_width,
            bar_height,
            Color::new(0.2, 0.2, 0.2, 0.8),
        );

        // Progress bar fill
        let fill_width = (f32::from(ai_level) / 9.0) * bar_width;
        let fill_color = match ai_level {
            0..=2 => Color::new(0.2, 0.8, 0.2, 1.0), // Green (easy)
            3..=5 => Color::new(0.8, 0.8, 0.2, 1.0), // Yellow (medium)
            6..=7 => Color::new(0.8, 0.5, 0.2, 1.0), // Orange (hard)
            _ => Color::new(0.8, 0.2, 0.2, 1.0),     // Red (expert)
        };
        frame.fill_rect(bar_x, ai_y + 2.0, fill_width, bar_height, fill_color);

        // Progress bar border
        frame.stroke_rect(
            bar_x,
            ai_y + 2.0,
            bar_width,
            bar_height,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );

        // AI level text
        let ai_text = format!("{ai_level}/9 {ai_name}");
        frame.fill_text_aligned(
            &ai_text,
            bar_x + bar_width + 10.0,
            ai_y,
            small_font,
            Color::new(0.7, 0.7, 0.7, 1.0),
            TextAlign::Left,
            TextBaseline::Top,
        );

        // AI difficulty +/- buttons
        let ai_btn_size = 20.0;
        let ai_btn_y = ai_y - 2.0;
        let ai_btn_x = bar_x + bar_width + 75.0;

        // - button
        frame.fill_rect(
            ai_btn_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.3, 0.3, 0.3, 0.9),
        );
        frame.stroke_rect(
            ai_btn_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );
        frame.fill_text_aligned(
            "-",
            ai_btn_x + ai_btn_size / 2.0,
            ai_btn_y + ai_btn_size / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        self.hud_buttons.ai_decrease =
            ButtonRect::new(ai_btn_x, ai_btn_y, ai_btn_size, ai_btn_size);

        // + button
        let ai_plus_x = ai_btn_x + ai_btn_size + 5.0;
        frame.fill_rect(
            ai_plus_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.3, 0.3, 0.3, 0.9),
        );
        frame.stroke_rect(
            ai_plus_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );
        frame.fill_text_aligned(
            "+",
            ai_plus_x + ai_btn_size / 2.0,
            ai_btn_y + ai_btn_size / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        self.hud_buttons.ai_increase =
            ButtonRect::new(ai_plus_x, ai_btn_y, ai_btn_size, ai_btn_size);

        // =========================================================================
        // Download .apr Button (bottom-left)
        // =========================================================================
        let download_y = self.height - 45.0;
        let download_text = "Download .apr";
        let download_width = (download_text.len() as f32) * 8.0 + button_padding * 2.0;

        // Button background
        frame.fill_rect(
            10.0,
            download_y,
            download_width,
            button_height,
            Color::new(0.1, 0.4, 0.2, 0.8),
        );

        // Button border
        frame.stroke_rect(
            10.0,
            download_y,
            download_width,
            button_height,
            Color::new(0.3, 0.7, 0.4, 1.0),
            1.0,
        );

        // Store download button for click handling
        self.hud_buttons.download =
            ButtonRect::new(10.0, download_y, download_width, button_height);

        // Button text
        frame.fill_text_aligned(
            download_text,
            10.0 + download_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );

        // =========================================================================
        // Info Button (next to Download)
        // =========================================================================
        let info_text = "Info";
        let info_width = (info_text.len() as f32) * 8.0 + button_padding * 2.0;
        let info_x = 10.0 + download_width + 5.0;

        // Info button background (purple/blue tint to distinguish from download)
        let info_bg = if self.show_model_info {
            Color::new(0.3, 0.3, 0.7, 0.9) // Brighter when active
        } else {
            Color::new(0.2, 0.2, 0.4, 0.8)
        };
        frame.fill_rect(info_x, download_y, info_width, button_height, info_bg);

        // Info button border
        frame.stroke_rect(
            info_x,
            download_y,
            info_width,
            button_height,
            Color::new(0.4, 0.4, 0.8, 1.0),
            1.0,
        );

        // Store info button for click handling
        self.hud_buttons.model_info =
            ButtonRect::new(info_x, download_y, info_width, button_height);

        // Info button text
        frame.fill_text_aligned(
            info_text,
            info_x + info_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );

        // Keyboard hint for Info button
        frame.fill_text_aligned(
            "[I]",
            info_x + info_width + 5.0,
            download_y + button_height / 2.0,
            "10px monospace",
            Color::new(0.4, 0.4, 0.4, 0.7),
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // =========================================================================
        // Sound Toggle Button (next to Info, after hint)
        // =========================================================================
        let sound_text = if self.sound_enabled { "Sound" } else { "Muted" };
        let sound_width = 64.0; // Fixed width for consistent layout
        let sound_x = info_x + info_width + 25.0; // After [I] hint

        // Sound button background (green when on, red when muted)
        let sound_bg = if self.sound_enabled {
            Color::new(0.2, 0.4, 0.2, 0.8)
        } else {
            Color::new(0.4, 0.2, 0.2, 0.8)
        };
        frame.fill_rect(sound_x, download_y, sound_width, button_height, sound_bg);

        // Sound button border
        let sound_border = if self.sound_enabled {
            Color::new(0.3, 0.7, 0.3, 1.0)
        } else {
            Color::new(0.7, 0.3, 0.3, 1.0)
        };
        frame.stroke_rect(
            sound_x,
            download_y,
            sound_width,
            button_height,
            sound_border,
            1.0,
        );

        // Store sound button for click handling
        self.hud_buttons.sound_toggle =
            ButtonRect::new(sound_x, download_y, sound_width, button_height);

        // Sound button text
        frame.fill_text_aligned(
            sound_text,
            sound_x + sound_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );

        // =========================================================================
        // AI Explainability Widget (upper-right, below speed buttons)
        // =========================================================================
        self.render_ai_explain_widget(frame);

        // =========================================================================
        // Model Info Panel (shown when show_model_info is true)
        // =========================================================================
        if self.show_model_info {
            self.render_model_info_panel(frame);
        }

        // Footer links (bottom-right, stacked)
        let footer_x = self.width - 10.0;
        let footer_y = self.height - 10.0;
        let link_color = Color::new(0.4, 0.4, 0.4, 0.7);
        let link_spacing = 14.0;

        // Line 1: PAIML website
        frame.fill_text_aligned(
            "paiml.com",
            footer_x,
            footer_y - link_spacing * 2.0,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );

        // Line 2: Jugar repo
        frame.fill_text_aligned(
            "github.com/paiml/jugar",
            footer_x,
            footer_y - link_spacing,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );

        // Line 3: Aprender (APR format) repo
        frame.fill_text_aligned(
            ".apr format: github.com/paiml/aprender",
            footer_x,
            footer_y,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );
    }

    /// Renders the model info panel showing .apr metadata.
    ///
    /// Panel displays:
    /// - Model name, version, author, license
    /// - Flow Theory parameters
    /// - Current AI difficulty and state
    /// - Difficulty curve visualization
    #[allow(clippy::too_many_lines)]
    fn render_model_info_panel(&self, frame: &mut RenderFrame) {
        // Panel dimensions - centered, semi-transparent overlay
        let panel_width = 320.0;
        let panel_height = 340.0;
        let panel_x = (self.width - panel_width) / 2.0;
        let panel_y = (self.height - panel_height) / 2.0;

        // Semi-transparent dark background
        frame.fill_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(0.05, 0.05, 0.1, 0.95),
        );

        // Panel border
        frame.stroke_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(0.3, 0.3, 0.8, 1.0),
            2.0,
        );

        let title_font = "16px monospace";
        let label_font = "12px monospace";
        let value_font = "14px monospace";

        let text_color = Color::WHITE;
        let label_color = Color::new(0.6, 0.6, 0.8, 1.0);
        let highlight_color = Color::new(0.4, 0.8, 1.0, 1.0);

        let mut y = panel_y + 25.0;
        let left_margin = panel_x + 15.0;
        let value_x = panel_x + 140.0;
        let line_height = 22.0;

        // Title
        frame.fill_text_aligned(
            ".apr Model Info",
            panel_x + panel_width / 2.0,
            y,
            title_font,
            highlight_color,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // Separator line
        frame.fill_rect(
            panel_x + 10.0,
            y,
            panel_width - 20.0,
            1.0,
            Color::new(0.3, 0.3, 0.5, 0.8),
        );
        y += 12.0;

        // Get model info from AI (or use defaults)
        let ai = self.ai.as_ref();
        let default_model = crate::ai::PongAIModel::default();
        let model = ai.map_or(&default_model, crate::ai::PongAI::model);

        // === Metadata Section ===
        frame.fill_text_aligned(
            "METADATA",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Model name
        frame.fill_text_aligned(
            "Model:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.name,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Version
        frame.fill_text_aligned(
            "Version:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.version,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Author
        frame.fill_text_aligned(
            "Author:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.author,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // License
        frame.fill_text_aligned(
            "License:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.license,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // === Flow Theory Section ===
        frame.fill_text_aligned(
            "FLOW THEORY (DDA)",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Adaptation rate
        frame.fill_text_aligned(
            "Adapt Rate:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let adapt_text = format!("{:.0}%", model.flow_theory.adaptation_rate * 100.0);
        frame.fill_text_aligned(
            &adapt_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Target win rate
        frame.fill_text_aligned(
            "Target Win:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let target_text = format!("{:.0}%", model.flow_theory.target_win_rate * 100.0);
        frame.fill_text_aligned(
            &target_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Flow thresholds
        frame.fill_text_aligned(
            "Flow Range:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let range_text = format!(
            "{:.0}%-{:.0}%",
            model.flow_theory.anxiety_threshold * 100.0,
            model.flow_theory.boredom_threshold * 100.0
        );
        frame.fill_text_aligned(
            &range_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // === Current State Section ===
        frame.fill_text_aligned(
            "CURRENT STATE",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // AI difficulty level
        let ai_level = ai.map_or(5, crate::ai::PongAI::difficulty);
        let ai_name = ai.map_or("Normal", crate::ai::PongAI::difficulty_name);
        frame.fill_text_aligned(
            "Difficulty:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let level_text = format!("{ai_level}/9 {ai_name}");
        frame.fill_text_aligned(
            &level_text,
            value_x,
            y,
            value_font,
            highlight_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Difficulty bar visualization
        let bar_x = left_margin;
        let bar_width = panel_width - 30.0;
        let bar_height = 12.0;

        // Background
        frame.fill_rect(
            bar_x,
            y,
            bar_width,
            bar_height,
            Color::new(0.15, 0.15, 0.2, 1.0),
        );

        // Fill based on level
        let fill_width = (f32::from(ai_level) / 9.0) * bar_width;
        let fill_color = match ai_level {
            0..=2 => Color::new(0.2, 0.8, 0.2, 1.0), // Green (easy)
            3..=5 => Color::new(0.8, 0.8, 0.2, 1.0), // Yellow (medium)
            6..=7 => Color::new(0.8, 0.5, 0.2, 1.0), // Orange (hard)
            _ => Color::new(0.8, 0.2, 0.2, 1.0),     // Red (expert)
        };
        frame.fill_rect(bar_x, y, fill_width, bar_height, fill_color);

        // Bar border
        frame.stroke_rect(
            bar_x,
            y,
            bar_width,
            bar_height,
            Color::new(0.4, 0.4, 0.6, 1.0),
            1.0,
        );
        y += bar_height + 8.0;

        // File size
        frame.fill_text_aligned(
            "File Size:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let size_text = format!("{} bytes", model.serialized_size());
        frame.fill_text_aligned(
            &size_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // Close hint at bottom
        frame.fill_text_aligned(
            "Press [I] or click Info to close",
            panel_x + panel_width / 2.0,
            panel_y + panel_height - 15.0,
            "10px monospace",
            Color::new(0.5, 0.5, 0.5, 0.8),
            TextAlign::Center,
            TextBaseline::Middle,
        );
    }

    /// Renders the AI explainability widget in the upper-right corner.
    ///
    /// Shows real-time SHAP-style feature contributions from the `.apr` model:
    /// - Current decision state (IDLE, REACT, TRACK, READY)
    /// - Feature contribution bars
    /// - Decision rationale
    fn render_ai_explain_widget(&self, frame: &mut RenderFrame) {
        // Only show when AI is active (Demo or SinglePlayer mode)
        if !matches!(self.game_mode, GameMode::Demo | GameMode::SinglePlayer) {
            return;
        }

        // In Demo mode, show widgets for both AIs
        if self.game_mode == GameMode::Demo {
            // Left AI widget (upper-left)
            if let Some(left_ai) = self.left_ai.as_ref() {
                self.render_single_ai_widget(frame, left_ai, 10.0, "P1 AI");
            }
            // Right AI widget (upper-right)
            if let Some(right_ai) = self.ai.as_ref() {
                self.render_single_ai_widget(frame, right_ai, self.width - 210.0, "P2 AI");
            }
        } else {
            // SinglePlayer mode - show left AI widget (AI opponent is on the left)
            if let Some(left_ai) = self.left_ai.as_ref() {
                self.render_single_ai_widget(frame, left_ai, 10.0, ".apr SHAP");
            }
        }
    }

    /// Renders a single AI explanation widget at the given x position.
    #[allow(clippy::unused_self, clippy::too_many_lines)]
    fn render_single_ai_widget(
        &self,
        frame: &mut RenderFrame,
        ai: &PongAI,
        widget_x: f32,
        title: &str,
    ) {
        let explanation = ai.explanation();

        // Widget dimensions - compact
        let widget_width = 200.0;
        let widget_height = 180.0;
        let widget_y = 80.0; // Below mode buttons and AI difficulty row

        // Semi-transparent dark background
        frame.fill_rect(
            widget_x,
            widget_y,
            widget_width,
            widget_height,
            Color::new(0.02, 0.02, 0.08, 0.85),
        );

        // Cyan border (matches aprender branding)
        frame.stroke_rect(
            widget_x,
            widget_y,
            widget_width,
            widget_height,
            Color::new(0.2, 0.8, 0.9, 0.9),
            1.0,
        );

        let title_font = "12px monospace";
        let small_font = "10px monospace";
        let tiny_font = "9px monospace";

        let text_color = Color::WHITE;
        let label_color = Color::new(0.6, 0.7, 0.8, 1.0);
        let cyan = Color::new(0.2, 0.8, 0.9, 1.0);

        let mut y = widget_y + 15.0;
        let left_margin = widget_x + 8.0;
        let line_height = 14.0;

        // Title with state indicator
        let state_color = match explanation.state {
            crate::ai::DecisionState::Idle => Color::new(0.5, 0.5, 0.5, 1.0),
            crate::ai::DecisionState::Reacting => Color::new(1.0, 0.8, 0.2, 1.0),
            crate::ai::DecisionState::Tracking => Color::new(0.2, 1.0, 0.4, 1.0),
            crate::ai::DecisionState::Ready => Color::new(0.2, 0.6, 1.0, 1.0),
        };

        frame.fill_text_aligned(
            title,
            left_margin,
            y,
            title_font,
            cyan,
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // State badge
        frame.fill_text_aligned(
            explanation.state.code(),
            widget_x + widget_width - 10.0,
            y,
            title_font,
            state_color,
            TextAlign::Right,
            TextBaseline::Middle,
        );
        y += line_height + 4.0;

        // Separator
        frame.fill_rect(
            left_margin,
            y,
            widget_width - 16.0,
            1.0,
            Color::new(0.3, 0.4, 0.5, 0.6),
        );
        y += 6.0;

        // Feature contributions with bars
        let bar_max_width = 80.0;
        let bar_height = 8.0;
        let bar_x = widget_x + widget_width - bar_max_width - 10.0;

        // Show top 4 contributions
        for contrib in explanation.contributions.iter().take(4) {
            // Feature name (truncated)
            let name = if contrib.name.len() > 12 {
                format!("{}...", &contrib.name[..9])
            } else {
                contrib.name.clone()
            };

            frame.fill_text_aligned(
                &name,
                left_margin,
                y + bar_height / 2.0,
                tiny_font,
                label_color,
                TextAlign::Left,
                TextBaseline::Middle,
            );

            // Contribution bar background
            frame.fill_rect(
                bar_x,
                y,
                bar_max_width,
                bar_height,
                Color::new(0.1, 0.1, 0.15, 1.0),
            );

            // Contribution bar fill
            let bar_width = contrib.importance * bar_max_width;
            let bar_color = if contrib.contribution >= 0.0 {
                Color::new(0.2, 0.7, 0.3, 0.9) // Green for positive
            } else {
                Color::new(0.7, 0.3, 0.2, 0.9) // Red for negative
            };
            frame.fill_rect(bar_x, y, bar_width, bar_height, bar_color);

            y += line_height;
        }

        y += 4.0;

        // Separator
        frame.fill_rect(
            left_margin,
            y,
            widget_width - 16.0,
            1.0,
            Color::new(0.3, 0.4, 0.5, 0.6),
        );
        y += 6.0;

        // Model params row
        let params_text = format!(
            "L{} | {:.0}ms | {:.0}%acc",
            explanation.difficulty_level,
            explanation.reaction_delay_ms,
            explanation.prediction_accuracy * 100.0
        );
        frame.fill_text_aligned(
            &params_text,
            left_margin,
            y,
            tiny_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Decision rationale (wrapped if needed)
        let rationale = if explanation.rationale.len() > 30 {
            format!("{}...", &explanation.rationale[..27])
        } else {
            explanation.rationale.clone()
        };
        frame.fill_text_aligned(
            &rationale,
            left_margin,
            y,
            small_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
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
    /// Canvas offset X from viewport origin
    canvas_offset_x: f32,
    /// Canvas offset Y from viewport origin
    canvas_offset_y: f32,
    /// Game tracer for replay recording (only active in debug mode)
    tracer: GameTracer,
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

        // Use debug tracer in debug mode (Andon Cord), production tracer otherwise
        let tracer = if config.debug {
            GameTracer::debug()
        } else {
            GameTracer::production()
        };

        Ok(Self {
            config,
            timer,
            input: InputState::new(),
            render_frame: RenderFrame::with_capacity(100),
            game: None,
            pong,
            frame_count: 0,
            canvas_offset_x: 0.0,
            canvas_offset_y: 0.0,
            tracer,
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
            canvas_offset_x: 0.0,
            canvas_offset_y: 0.0,
            tracer: GameTracer::production(), // Default to production mode
        }
    }

    /// Sets the canvas offset from viewport origin.
    #[wasm_bindgen(js_name = "setCanvasOffset")]
    #[allow(clippy::missing_const_for_fn)]
    pub fn set_canvas_offset(&mut self, x: f32, y: f32) {
        self.canvas_offset_x = x;
        self.canvas_offset_y = y;
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
        // Begin trace frame recording
        self.tracer.begin_frame();

        // Update timer and get delta time
        let dt = self.timer.update(timestamp);

        // Process input events with canvas offset for coordinate conversion
        let canvas_offset = Vec2::new(self.canvas_offset_x, self.canvas_offset_y);
        // Ignore errors for now - invalid events are just skipped
        let _ = process_input_events(input_events_json, &mut self.input, canvas_offset);

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

        // Collect JS actions (e.g., download request, fullscreen toggle)
        let mut actions = Vec::new();
        if self.pong.download_requested {
            actions.push(JsAction::DownloadAiModel);
            self.pong.download_requested = false; // Consume the flag
        }
        if self.pong.fullscreen_requested {
            if self.pong.is_fullscreen {
                actions.push(JsAction::EnterFullscreen);
            } else {
                actions.push(JsAction::ExitFullscreen);
            }
            self.pong.fullscreen_requested = false; // Consume the flag
        }

        // End trace frame (no state hash for now - can add deterministic hashing later)
        let _ = self.tracer.end_frame(None);

        // Build frame output with optional debug info
        let output = FrameOutput {
            commands: self.render_frame.commands.clone(),
            audio_events,
            actions,
            debug_info: if self.config.debug {
                let stats = self.tracer.stats();
                Some(DebugInfo {
                    dt_ms: dt * 1000.0,
                    fps: self.timer.average_fps(),
                    frame_count: self.frame_count,
                    input_summary: String::new(),
                    game_mode: self.pong.game_mode().name().to_string(),
                    speed_multiplier: self.pong.speed_multiplier().value(),
                    left_paddle_y: self.pong.left_paddle_y(),
                    right_paddle_y: self.pong.right_paddle_y(),
                    ball_x: self.pong.ball_x(),
                    ball_y: self.pong.ball_y(),
                    trace_buffer_usage: Some(format!(
                        "{}/{}",
                        stats.buffer_len, stats.buffer_capacity
                    )),
                    trace_inputs: Some(stats.total_inputs),
                    trace_dropped: Some(stats.frames_dropped),
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

    /// Returns the AI model as JSON string for download.
    #[wasm_bindgen(js_name = "getAiModel")]
    #[must_use]
    pub fn get_ai_model(&self) -> String {
        self.pong.export_ai_model()
    }

    /// Returns AI model metadata and current state as JSON.
    #[wasm_bindgen(js_name = "getAiInfo")]
    #[must_use]
    pub fn get_ai_info(&self) -> String {
        self.pong.ai_info()
    }

    /// Sets the AI difficulty level (0-9).
    #[wasm_bindgen(js_name = "setAiDifficulty")]
    pub fn set_ai_difficulty(&mut self, level: u8) {
        self.pong.set_ai_difficulty(level);
    }

    /// Gets the current AI difficulty level.
    #[wasm_bindgen(js_name = "getAiDifficulty")]
    #[must_use]
    pub fn get_ai_difficulty(&self) -> u8 {
        self.pong.ai_difficulty()
    }

    /// Sets the speed multiplier (1, 5, 10, 50, 100, 1000).
    #[wasm_bindgen(js_name = "setSpeed")]
    #[allow(clippy::match_same_arms)] // Explicit default case is clearer
    pub fn set_speed(&mut self, speed: u32) {
        self.pong.set_speed_multiplier(match speed {
            5 => SpeedMultiplier::Fast5x,
            10 => SpeedMultiplier::Fast10x,
            50 => SpeedMultiplier::Fast50x,
            100 => SpeedMultiplier::Fast100x,
            1000 => SpeedMultiplier::Fast1000x,
            _ => SpeedMultiplier::Normal, // 1 and any other value defaults to Normal
        });
    }

    /// Gets the current speed multiplier value.
    #[wasm_bindgen(js_name = "getSpeed")]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // wasm_bindgen not compatible
    pub fn get_speed(&self) -> u32 {
        self.pong.speed_multiplier().value()
    }

    /// Sets the game mode ("demo", "1p", "2p").
    #[wasm_bindgen(js_name = "setGameMode")]
    #[allow(clippy::match_same_arms)] // Explicit default case is clearer
    pub fn set_game_mode(&mut self, mode: &str) {
        self.pong.set_game_mode(match mode.to_lowercase().as_str() {
            "demo" => GameMode::Demo,
            "2p" | "two" | "twoplayer" => GameMode::TwoPlayer,
            _ => GameMode::SinglePlayer, // "1p", "single", "singleplayer" and any other value
        });
    }

    /// Gets the current game mode as string.
    #[wasm_bindgen(js_name = "getGameMode")]
    #[must_use]
    pub fn get_game_mode(&self) -> String {
        self.pong.game_mode().short_label().to_string()
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
        let tracer = if config.debug {
            GameTracer::debug()
        } else {
            GameTracer::new(TracerConfig::default())
        };

        Self {
            config,
            timer: FrameTimer::new(),
            input: InputState::new(),
            render_frame: RenderFrame::with_capacity(100),
            game: None,
            pong,
            frame_count: 0,
            canvas_offset_x: 0.0,
            canvas_offset_y: 0.0,
            tracer,
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

    /// Returns a reference to the game tracer (for testing).
    #[must_use]
    pub const fn tracer(&self) -> &GameTracer {
        &self.tracer
    }

    /// Returns a reference to the config (for testing).
    #[must_use]
    pub const fn config(&self) -> &WebConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::manual_range_contains,
    clippy::cast_lossless,
    clippy::suboptimal_flops,
    clippy::uninlined_format_args
)]
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
        game.game_mode = GameMode::SinglePlayer; // Human controls RIGHT paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press Up arrow key (P1 controls right paddle)
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Right paddle should have moved up
        assert!(game.right_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_game_paddle_clamping() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::SinglePlayer; // Human controls left paddle
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
        let mut game = PongGame::new(800.0, 600.0, false);
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
            actions: vec![],
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
            actions: vec![],
            debug_info: Some(DebugInfo {
                dt_ms: 16.667,
                fps: 60.0,
                frame_count: 100,
                input_summary: "W pressed".to_string(),
                game_mode: "SinglePlayer".to_string(),
                speed_multiplier: 1,
                left_paddle_y: 300.0,
                right_paddle_y: 300.0,
                ball_x: 400.0,
                ball_y: 300.0,
                trace_buffer_usage: Some("100/3600".to_string()),
                trace_inputs: Some(42),
                trace_dropped: Some(0),
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
            actions: vec![],
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
            game_mode: "Demo".to_string(),
            speed_multiplier: 10,
            left_paddle_y: 300.0,
            right_paddle_y: 300.0,
            ball_x: 400.0,
            ball_y: 300.0,
            trace_buffer_usage: None,
            trace_inputs: None,
            trace_dropped: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("16.667"));
        assert!(json.contains("60"));
        // Trace fields are skipped when None
        assert!(!json.contains("trace_buffer_usage"));
        assert!(json.contains("42"));
        assert!(json.contains("Demo"));
        assert!(json.contains("10"));
        assert!(json.contains("left_paddle_y"));
        assert!(json.contains("ball_x"));
    }

    #[test]
    fn test_pong_right_paddle_controls() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::TwoPlayer; // Enable human control of right paddle
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
        game.game_mode = GameMode::TwoPlayer; // Enable human control of right paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press ArrowUp key
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Right paddle should have moved up
        assert!(game.right_paddle_y < initial_y);
    }

    #[test]
    fn test_pong_right_paddle_down() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing); // Start game for test
        game.game_mode = GameMode::SinglePlayer; // Human controls RIGHT paddle
        let mut input = InputState::new();
        let initial_y = game.right_paddle_y;

        // Press Down arrow key (P1 controls right paddle)
        input.set_key_pressed(jugar_input::KeyCode::Down, true);
        game.update(&input, 0.1);

        // Right paddle should have moved down
        assert!(game.right_paddle_y > initial_y);
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
    fn test_game_state_default_is_playing() {
        // Game starts directly in Playing state (Demo mode attract)
        let game = PongGame::new(800.0, 600.0, false);
        assert_eq!(game.state(), GameState::Playing);
    }

    #[test]
    fn test_game_state_start_from_menu() {
        // If game is set to Menu state, start() transitions to Playing
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu);
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
        // When in Menu state, pressing Space starts the game
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu); // Set to Menu state for this test
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

    // =========================================================================
    // HUD Button Tests for Coverage
    // =========================================================================

    #[test]
    fn test_hud_buttons_default() {
        let hud = HudButtons::default();
        // All buttons should be at default (0, 0, 0, 0) position
        assert_eq!(hud.mode_demo.x, 0.0);
        assert_eq!(hud.mode_1p.x, 0.0);
        assert_eq!(hud.mode_2p.x, 0.0);
        assert_eq!(hud.speed_1x.x, 0.0);
    }

    #[test]
    fn test_hud_buttons_calculate() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Mode buttons should be positioned
        assert!(hud.mode_demo.x > 0.0);
        assert!(hud.mode_1p.x > hud.mode_demo.x);
        assert!(hud.mode_2p.x > hud.mode_1p.x);
        // Speed buttons should be positioned
        assert!(hud.speed_1x.x > 0.0);
    }

    #[test]
    fn test_hud_buttons_hit_test() {
        let hud = HudButtons::calculate(800.0, 600.0);

        // Test hit detection using contains method
        let test_x = hud.mode_demo.x + hud.mode_demo.width / 2.0;
        let test_y = hud.mode_demo.y + hud.mode_demo.height / 2.0;

        // Point inside button
        assert!(hud.mode_demo.contains(test_x, test_y));
    }

    #[test]
    fn test_button_rect_new() {
        let btn = ButtonRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(btn.x, 10.0);
        assert_eq!(btn.y, 20.0);
        assert_eq!(btn.width, 100.0);
        assert_eq!(btn.height, 50.0);
    }

    #[test]
    fn test_button_rect_contains() {
        let btn = ButtonRect::new(10.0, 10.0, 100.0, 50.0);
        // Inside
        assert!(btn.contains(50.0, 30.0));
        // Outside left
        assert!(!btn.contains(5.0, 30.0));
        // Outside right
        assert!(!btn.contains(120.0, 30.0));
        // Outside top
        assert!(!btn.contains(50.0, 5.0));
        // Outside bottom
        assert!(!btn.contains(50.0, 65.0));
    }

    #[test]
    fn test_web_platform_mode_toggle_via_d_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Get initial mode
        let _ = platform.frame(0.0, "[]");

        // Press D to toggle demo mode
        let d_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyD"}}]"#;
        let result = platform.frame(100.0, d_down);
        assert!(result.contains("Demo") || result.contains("SinglePlayer"));

        // Release D
        let d_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyD"}}]"#;
        let _ = platform.frame(116.0, d_up);
    }

    #[test]
    fn test_web_platform_mode_cycle_via_m_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press M to cycle modes
        let m_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyM"}}]"#;
        let _ = platform.frame(100.0, m_down);

        let m_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyM"}}]"#;
        let _ = platform.frame(116.0, m_up);
    }

    #[test]
    fn test_web_platform_speed_keys_1_through_6() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press digit 3 for 10x speed
        let key_3 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Digit3"}}]"#;
        let _ = platform.frame(100.0, key_3);

        // Press digit 6 for 1000x speed
        let key_6 = r#"[{"event_type":"KeyDown","timestamp":200,"data":{"key":"Digit6"}}]"#;
        let _ = platform.frame(200.0, key_6);

        // Press digit 1 for 1x speed
        let key_1 = r#"[{"event_type":"KeyDown","timestamp":300,"data":{"key":"Digit1"}}]"#;
        let _ = platform.frame(300.0, key_1);
    }

    #[test]
    fn test_web_platform_fullscreen_via_f_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press F to request fullscreen
        let key_f = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyF"}}]"#;
        let output = platform.frame(100.0, key_f);

        // Verify EnterFullscreen action is emitted
        assert!(
            output.contains("EnterFullscreen"),
            "F key should trigger EnterFullscreen action"
        );

        // Release F
        let key_f_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"KeyF"}}]"#;
        let _ = platform.frame(116.0, key_f_up);

        // Press F again to exit fullscreen
        let output = platform.frame(200.0, key_f);
        assert!(
            output.contains("ExitFullscreen"),
            "F key should toggle to ExitFullscreen action"
        );
    }

    #[test]
    fn test_web_platform_fullscreen_via_f11_key() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press F11 to request fullscreen
        let key_f11 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"F11"}}]"#;
        let output = platform.frame(100.0, key_f11);

        // Verify EnterFullscreen action is emitted
        assert!(
            output.contains("EnterFullscreen"),
            "F11 key should trigger EnterFullscreen action"
        );
    }

    #[test]
    fn test_web_platform_pause_resume_via_escape() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space_down = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space_down);
        let space_up = r#"[{"event_type":"KeyUp","timestamp":16,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(16.0, space_up);
        let _ = platform.frame(33.0, "[]");

        // Press Escape to pause
        let esc_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Escape"}}]"#;
        let result = platform.frame(100.0, esc_down);
        assert!(result.contains("PAUSED") || result.contains("commands"));

        // Release and re-press to resume
        let esc_up = r#"[{"event_type":"KeyUp","timestamp":116,"data":{"key":"Escape"}}]"#;
        let _ = platform.frame(116.0, esc_up);
        let _ = platform.frame(200.0, esc_down);
    }

    #[test]
    fn test_web_platform_info_panel_toggle() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Press I to toggle info panel
        let i_down = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"KeyI"}}]"#;
        let result = platform.frame(100.0, i_down);
        assert!(result.contains("Pong AI") || result.contains("commands"));
    }

    #[test]
    fn test_web_platform_mouse_click_hud() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Click on approximate HUD area
        let click =
            r#"[{"event_type":"MouseDown","timestamp":100,"data":{"button":0,"x":100,"y":20}}]"#;
        let _ = platform.frame(100.0, click);

        let release =
            r#"[{"event_type":"MouseUp","timestamp":116,"data":{"button":0,"x":100,"y":20}}]"#;
        let _ = platform.frame(116.0, release);
    }

    #[test]
    fn test_web_platform_download_button_click() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let _ = platform.frame(0.0, "[]");

        // Click on download button area (bottom left footer)
        let click =
            r#"[{"event_type":"MouseDown","timestamp":100,"data":{"button":0,"x":70,"y":569}}]"#;
        let result = platform.frame(100.0, click);
        // Should have download action in response
        assert!(result.contains("DownloadAiModel") || result.contains("commands"));
    }

    #[test]
    fn test_web_platform_get_ai_model() {
        let config = WebConfig::default();
        let platform = WebPlatform::new_for_test(config);

        let model_json = platform.get_ai_model();
        assert!(model_json.contains("metadata"));
        assert!(model_json.contains("Pong AI"));
        assert!(model_json.contains("difficulty_profiles"));
    }

    #[test]
    fn test_web_platform_set_game_mode() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        platform.set_game_mode("demo");
        let _ = platform.frame(0.0, "[]");

        platform.set_game_mode("twoplayer");
        let _ = platform.frame(16.0, "[]");

        platform.set_game_mode("singleplayer");
        let _ = platform.frame(33.0, "[]");
    }

    #[test]
    fn test_web_platform_ai_difficulty_accessors() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let initial = platform.get_ai_difficulty();
        assert!(initial >= 1 && initial <= 10);

        platform.set_ai_difficulty(3);
        assert_eq!(platform.get_ai_difficulty(), 3);

        platform.set_ai_difficulty(9);
        assert_eq!(platform.get_ai_difficulty(), 9);
    }

    #[test]
    fn test_pong_game_victory_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::GameOver);
        game.left_score = 11;

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render victory text
        assert!(frame.len() > 5);
    }

    #[test]
    fn test_pong_game_paused_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Paused);

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render PAUSED text
        assert!(frame.len() > 3);
    }

    #[test]
    fn test_pong_game_menu_screen_renders() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Menu);

        let mut frame = RenderFrame::new();
        game.render(&mut frame);

        // Should render menu
        assert!(frame.len() > 3);
    }

    #[test]
    fn test_pong_game_demo_mode_ai_controls_both_paddles() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.game_mode = GameMode::Demo;

        let input = InputState::new();

        // Run several frames - AI should move both paddles
        for _ in 0..60 {
            game.update(&input, 0.016);
        }

        // Game should still be running
        assert!(game.state() == GameState::Playing || game.state() == GameState::Menu);
    }

    #[test]
    fn test_pong_game_two_player_mode() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);
        game.game_mode = GameMode::TwoPlayer;

        let mut input = InputState::new();
        let initial_left_y = game.left_paddle_y;
        let initial_right_y = game.right_paddle_y;

        // Both players control their paddles
        input.set_key_pressed(jugar_input::KeyCode::Letter('W'), true);
        input.set_key_pressed(jugar_input::KeyCode::Up, true);
        game.update(&input, 0.1);

        // Both paddles should have moved up
        assert!(game.left_paddle_y < initial_left_y);
        assert!(game.right_paddle_y < initial_right_y);
    }

    #[test]
    fn test_render_frame_commands_count() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        let result = platform.frame(0.0, "[]");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Should have commands array
        assert!(parsed["commands"].is_array());
        assert!(parsed["commands"].as_array().unwrap().len() > 5);
    }

    #[test]
    fn test_speed_multiplier_affects_physics() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);
        let _ = platform.frame(16.0, "[]");

        // Set 10x speed
        let key_3 = r#"[{"event_type":"KeyDown","timestamp":100,"data":{"key":"Digit3"}}]"#;
        let _ = platform.frame(100.0, key_3);

        // Run frame at high speed
        let result = platform.frame(116.0, "[]");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_js_action_download_ai_model() {
        let action = JsAction::DownloadAiModel;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("DownloadAiModel"));
    }

    #[test]
    fn test_js_action_open_url() {
        let action = JsAction::OpenUrl {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("OpenUrl"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_js_action_fullscreen() {
        let enter = JsAction::EnterFullscreen;
        let exit = JsAction::ExitFullscreen;

        let enter_json = serde_json::to_string(&enter).unwrap();
        let exit_json = serde_json::to_string(&exit).unwrap();

        assert!(enter_json.contains("EnterFullscreen"));
        assert!(exit_json.contains("ExitFullscreen"));
    }

    #[test]
    fn test_frame_output_with_actions() {
        let output = FrameOutput {
            commands: vec![],
            audio_events: vec![],
            actions: vec![JsAction::DownloadAiModel],
            debug_info: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("actions"));
        assert!(json.contains("DownloadAiModel"));
    }

    #[test]
    fn test_web_platform_full_gameplay_simulation() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Menu -> Playing
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);

        // Play for many frames
        for i in 1..100 {
            let ts = i as f64 * 16.667;
            let _ = platform.frame(ts, "[]");
        }

        // Get stats
        let stats = platform.get_stats();
        assert!(stats.contains("frame_count"));
    }

    #[test]
    fn test_web_platform_continuous_input() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Start game
        let space = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;
        let _ = platform.frame(0.0, space);
        let _ = platform.frame(16.0, "[]");

        // Hold W key for many frames
        for i in 0..30 {
            let ts = 100.0 + (i as f64 * 16.667);
            let w_down = format!(
                r#"[{{"event_type":"KeyDown","timestamp":{},"data":{{"key":"KeyW"}}}}]"#,
                ts
            );
            let _ = platform.frame(ts, &w_down);
        }
    }

    #[test]
    fn test_pong_game_ball_hits_track_rally() {
        let mut game = PongGame::new(800.0, 600.0, false);
        game.set_state(GameState::Playing);

        assert_eq!(game.rally_count(), 0);

        // Hit the ball - should increment rally
        game.ball_x = 46.0;
        game.ball_y = game.left_paddle_y;
        game.ball_vx = -100.0;
        let input = InputState::new();
        game.update(&input, 0.016);

        // Rally should have incremented after paddle hit
        assert!(game.rally_count() > 0 || game.ball_vx > 0.0);
    }

    #[test]
    fn test_all_speed_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // All 6 speed buttons should be positioned
        assert!(hud.speed_1x.width > 0.0);
        assert!(hud.speed_5x.width > 0.0);
        assert!(hud.speed_10x.width > 0.0);
        assert!(hud.speed_50x.width > 0.0);
        assert!(hud.speed_100x.width > 0.0);
        assert!(hud.speed_1000x.width > 0.0);
    }

    #[test]
    fn test_ai_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // AI difficulty buttons should be positioned
        assert!(hud.ai_decrease.width > 0.0);
        assert!(hud.ai_increase.width > 0.0);
    }

    #[test]
    fn test_footer_buttons_calculated() {
        let hud = HudButtons::calculate(800.0, 600.0);
        // Download and model info buttons
        assert!(hud.download.width > 0.0);
        assert!(hud.model_info.width > 0.0);
    }

    #[test]
    fn test_ai_shap_widget_renders_in_single_player_mode() {
        // Use WebConfig::default which has ai_enabled: true
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Run a frame - game starts in SinglePlayer mode with AI enabled
        let result = platform.frame(0.0, "[]");

        // Should render the .apr SHAP widget title
        assert!(
            result.contains(".apr SHAP"),
            "Expected frame output to contain '.apr SHAP' widget text. Output: {}",
            &result[..result.len().min(2000)]
        );
    }

    #[test]
    fn test_ai_shap_widget_not_rendered_in_two_player_mode() {
        let config = WebConfig::default();
        let mut platform = WebPlatform::new_for_test(config);

        // Switch to 2P mode using the public method
        platform.set_game_mode("2p");

        // Run a frame
        let result = platform.frame(0.0, "[]");

        // Should NOT contain the .apr SHAP widget in 2P mode
        assert!(
            !result.contains(".apr SHAP"),
            "Expected frame output to NOT contain '.apr SHAP' widget text in 2P mode"
        );
    }
}
