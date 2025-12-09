//! # jugar-web
//!
//! WASM browser integration for the Jugar game engine.
//!
//! This crate provides the web platform layer that bridges Jugar to browsers.
//! All game logic runs in Rust/WASM with **ABSOLUTE ZERO JavaScript computation**.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Browser (JavaScript)                    │
//! │  - Event listeners (keyboard, mouse, touch)                  │
//! │  - requestAnimationFrame loop                               │
//! │  - Canvas2D rendering (drawing only)                        │
//! └─────────────────────────┬────────────────────────────────────┘
//!                           │ JSON Events ↓  ↑ JSON Commands
//! ┌─────────────────────────┴────────────────────────────────────┐
//! │                      WebPlatform (Rust/WASM)                  │
//! │  - Input translation (browser events → InputState)          │
//! │  - Game logic (Pong, etc.)                                  │
//! │  - Render command generation (Canvas2DCommand)              │
//! │  - Time management (DOMHighResTimeStamp → seconds)          │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```javascript
//! // JavaScript (minimal event forwarding + Canvas2D execution)
//! import init, { WebPlatform } from './jugar_web.js';
//!
//! const platform = new WebPlatform('{"width":800,"height":600}');
//! const events = [];
//!
//! document.addEventListener('keydown', (e) => {
//!     events.push({ event_type: 'KeyDown', timestamp: e.timeStamp, data: { key: e.code } });
//! });
//!
//! function frame(timestamp) {
//!     const commands = JSON.parse(platform.frame(timestamp, JSON.stringify(events)));
//!     events.length = 0;
//!
//!     // Execute Canvas2D commands
//!     for (const cmd of commands) {
//!         switch (cmd.type) {
//!             case 'Clear': ctx.fillStyle = rgba(cmd.color); ctx.fillRect(0, 0, w, h); break;
//!             case 'FillRect': ctx.fillStyle = rgba(cmd.color); ctx.fillRect(cmd.x, cmd.y, cmd.width, cmd.height); break;
//!             // ...
//!         }
//!     }
//!
//!     requestAnimationFrame(frame);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::doc_markdown)]

pub mod ai;
pub mod audio;
pub mod compute;
pub mod input;
pub mod juice;
pub mod platform;
pub mod render;
pub mod simd;
pub mod time;

// Re-export main types for convenience
pub use audio::{AudioEvent, ProceduralAudio};
pub use compute::{
    detect_compute_capability, ComputeBenchmarkResult, ComputeCapability, ComputeDemo,
    ComputeDemoState, ComputeTier, GpuShaderInfo, ShaderType, PARTICLE_PHYSICS_WGSL,
};
pub use input::{
    process_input_events, translate_gamepad_axis, translate_gamepad_button, translate_key,
    translate_mouse_button, BrowserEventData, BrowserInputEvent, InputTranslationError,
};
pub use platform::{
    DebugInfo, FrameOutput, GameState, PongGame, WebConfig, WebGame, WebPlatform, WebPlatformError,
};
pub use render::{
    convert_render_command, convert_render_queue, Canvas2DCommand, Color, RenderFrame, TextAlign,
    TextBaseline,
};
pub use simd::{
    batch_distance_squared, batch_particle_update, batch_update_positions, check_paddle_collisions,
    detect_compute_backend, trueno_backend_to_compute_backend, ComputeBackend, SimdBenchmark,
    SimdVec2,
};
pub use time::{
    calculate_delta_time, clamp_delta_time, dom_timestamp_to_seconds, seconds_to_dom_timestamp,
    FrameTimer, DEFAULT_MAX_DELTA_TIME, TARGET_DT_120FPS, TARGET_DT_30FPS, TARGET_DT_60FPS,
};

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn test_public_exports() {
        // Verify all public exports are accessible
        let _ = WebConfig::default();
        let _ = Color::BLACK;
        let _ = TextAlign::default();
        let _ = TextBaseline::default();
        let _ = RenderFrame::new();
        let _ = FrameTimer::new();
        let _ = dom_timestamp_to_seconds(1000.0);
    }

    #[test]
    fn test_input_exports() {
        let _ = translate_key("Space");
        let _ = translate_mouse_button(0);
        let _ = translate_gamepad_button(0);
        let _ = translate_gamepad_axis(0);
    }

    #[test]
    fn test_time_constants() {
        assert!(DEFAULT_MAX_DELTA_TIME > 0.0);
        assert!(TARGET_DT_60FPS > 0.0);
        assert!(TARGET_DT_30FPS > TARGET_DT_60FPS);
        assert!(TARGET_DT_120FPS < TARGET_DT_60FPS);
    }
}
