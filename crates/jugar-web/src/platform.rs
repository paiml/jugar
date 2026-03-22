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
        let ai_btn_x = bar_x + bar_width + 100.0; // 240.0 - after "X/9 Nightmare" text
        let ai_plus_x = ai_btn_x + ai_btn_size + 5.0; // 265.0
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

        // Sound toggle button (next to model info, after [I] hint)
        let sound_width = Self::button_width("Sound", char_width, button_padding);
        let info_x = 10.0 + download_width + 5.0;
        let sound_toggle = ButtonRect::new(
            info_x + info_width + 25.0, // 25.0 accounts for "[I]" hint spacing
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

    /// Process keyboard shortcuts with edge detection (speed, mode, UI toggles).
    fn handle_keyboard_shortcuts(&mut self, input: &InputState) {
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
        if key_d && !self.key_d_was_pressed {
            self.game_mode = if self.game_mode == GameMode::Demo {
                GameMode::SinglePlayer
            } else {
                GameMode::Demo
            };
        }
        if key_i && !self.key_i_was_pressed {
            self.show_model_info = !self.show_model_info;
        }
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
    }

    /// Handle state transitions (menu, pause, game over). Returns true if game
    /// should continue to physics update.
    fn handle_state_transitions(&mut self, input: &InputState) -> bool {
        let mouse_pressed = input.mouse_button(MouseButton::Left).is_down();
        let mouse_just_clicked = mouse_pressed && !self.mouse_was_pressed;
        self.mouse_was_pressed = mouse_pressed;

        if mouse_just_clicked {
            let pos = input.mouse_position;
            self.handle_hud_click(pos.x, pos.y);
        }

        let space_pressed = input.is_key_pressed(jugar_input::KeyCode::Space);
        let escape_pressed = input.is_key_pressed(jugar_input::KeyCode::Escape);
        let space_just_pressed = space_pressed && !self.space_was_pressed;
        let escape_just_pressed = escape_pressed && !self.escape_was_pressed;
        self.space_was_pressed = space_pressed;
        self.escape_was_pressed = escape_pressed;

        match self.state {
            GameState::Menu | GameState::GameOver => {
                if space_just_pressed {
                    self.reset_game();
                    self.state = GameState::Playing;
                    self.audio.on_game_start();
                }
                false
            }
            GameState::Playing => {
                if escape_just_pressed {
                    self.state = GameState::Paused;
                    false
                } else {
                    true
                }
            }
            GameState::Paused => {
                if space_just_pressed || escape_just_pressed {
                    self.state = GameState::Playing;
                }
                false
            }
        }
    }

    /// Update paddle positions based on AI or human input.
    fn update_paddles(&mut self, input: &InputState, dt: f32) {
        let half_paddle = self.paddle_height / 2.0;

        if self.game_mode.left_is_ai() {
            if let Some(ref mut left_ai) = self.left_ai {
                let ai_velocity = left_ai.update(
                    self.ball_x,
                    self.ball_y,
                    -self.ball_vx,
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
            if input.is_key_pressed(jugar_input::KeyCode::Letter('W')) {
                self.left_paddle_y -= self.paddle_speed * dt;
            }
            if input.is_key_pressed(jugar_input::KeyCode::Letter('S')) {
                self.left_paddle_y += self.paddle_speed * dt;
            }
        }

        if self.game_mode.right_is_ai() {
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
            if input.is_key_pressed(jugar_input::KeyCode::Up) {
                self.right_paddle_y -= self.paddle_speed * dt;
            }
            if input.is_key_pressed(jugar_input::KeyCode::Down) {
                self.right_paddle_y += self.paddle_speed * dt;
            }
        }

        self.left_paddle_y = self
            .left_paddle_y
            .clamp(half_paddle, self.height - half_paddle);
        self.right_paddle_y = self
            .right_paddle_y
            .clamp(half_paddle, self.height - half_paddle);
    }

    /// Run ball physics: movement, wall bounces, paddle collisions, scoring.
    fn update_ball_physics(&mut self, dt: f32) {
        let half_paddle = self.paddle_height / 2.0;

        self.prev_ball_y = self.ball_y;
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        // Wall bounces
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
        if wall_bounced {
            self.juice.on_wall_bounce();
            let ball_speed = self.ball_vx.hypot(self.ball_vy);
            self.audio.on_wall_bounce_with_velocity(ball_speed, 250.0);
        }

        // Paddle collisions
        let left_paddle_x = 20.0 + self.paddle_width;
        let right_paddle_x = self.width - 20.0 - self.paddle_width;

        if self.ball_x - self.ball_radius < left_paddle_x
            && self.ball_x - self.ball_radius > 20.0
            && self.ball_y > self.left_paddle_y - half_paddle
            && self.ball_y < self.left_paddle_y + half_paddle
        {
            self.ball_x = left_paddle_x + self.ball_radius;
            self.ball_vx = self.ball_vx.abs() * 1.05;
            self.rally_count += 1;
            self.juice.on_paddle_hit_at(self.ball_x, self.ball_y, false);
            self.audio
                .on_paddle_hit(self.ball_y, self.left_paddle_y, self.paddle_height);
            if self.rally_count.is_multiple_of(5) {
                self.audio.on_rally_milestone(self.rally_count);
            }
            if let Some(ref mut ai) = self.ai {
                ai.record_player_hit();
            }
        }

        if self.ball_x + self.ball_radius > right_paddle_x
            && self.ball_x + self.ball_radius < self.width - 20.0
            && self.ball_y > self.right_paddle_y - half_paddle
            && self.ball_y < self.right_paddle_y + half_paddle
        {
            self.ball_x = right_paddle_x - self.ball_radius;
            self.ball_vx = -self.ball_vx.abs() * 1.05;
            self.rally_count += 1;
            self.juice.on_paddle_hit_at(self.ball_x, self.ball_y, true);
            self.audio
                .on_paddle_hit(self.ball_y, self.right_paddle_y, self.paddle_height);
            if self.rally_count.is_multiple_of(5) {
                self.audio.on_rally_milestone(self.rally_count);
            }
        }

        // Scoring
        self.check_scoring();

        // Update effects
        self.juice.update(self.ball_x, self.ball_y, dt);
        self.bg_time += dt;
    }

    /// Check if ball has passed a paddle boundary and handle scoring.
    fn check_scoring(&mut self) {
        if self.ball_x < 0.0 {
            self.right_score += 1;
            if self.rally_count > self.high_score {
                self.high_score = self.rally_count;
            }
            self.rally_count = 0;
            self.juice.on_goal(self.width * 0.75, 50.0, "+1");
            self.audio.on_goal(false);
            if self.game_mode != GameMode::Demo {
                if let Some(ref mut ai) = self.ai {
                    ai.record_player_miss();
                    ai.adapt_difficulty();
                }
            }
            if self.right_score >= self.winning_score {
                self.state = GameState::GameOver;
            } else {
                self.reset_ball();
            }
        } else if self.ball_x > self.width {
            self.left_score += 1;
            if self.rally_count > self.high_score {
                self.high_score = self.rally_count;
            }
            self.rally_count = 0;
            self.juice.on_goal(self.width * 0.25, 50.0, "+1");
            self.audio.on_goal(true);
            if self.game_mode != GameMode::Demo {
                if let Some(ref mut ai) = self.ai {
                    ai.adapt_difficulty();
                }
            }
            if self.left_score >= self.winning_score {
                self.state = GameState::GameOver;
            } else {
                self.reset_ball();
            }
        }
    }
}

impl WebGame for PongGame {
    fn update(&mut self, input: &InputState, dt: f64) {
        self.handle_keyboard_shortcuts(input);

        if !self.handle_state_transitions(input) {
            return;
        }

        let speed_mult = self.speed_multiplier.value();
        let dt = dt as f32 * speed_mult as f32;

        self.update_paddles(input, dt);
        self.update_ball_physics(dt);
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

#[path = "platform_render.rs"]
mod platform_render;

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
#[path = "platform_tests.rs"]
mod platform_tests;
