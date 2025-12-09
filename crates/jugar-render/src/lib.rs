//! # jugar-render
//!
//! Rendering system for Jugar with responsive camera and resolution-independent canvas.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use jugar_core::{Anchor, Camera, Position, Rect, ScaleMode};

/// Rendering errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RenderError {
    /// Invalid viewport dimensions
    #[error("Invalid viewport: {width}x{height}")]
    InvalidViewport {
        /// Width
        width: u32,
        /// Height
        height: u32,
    },
}

/// Result type for render operations
pub type Result<T> = core::result::Result<T, RenderError>;

/// Aspect ratio presets
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatio {
    /// Mobile portrait (9:16)
    MobilePortrait,
    /// Mobile landscape (16:9)
    MobileLandscape,
    /// Desktop standard (16:9)
    Standard,
    /// Ultrawide (21:9)
    Ultrawide,
    /// Super ultrawide (32:9) - 49" monitors
    SuperUltrawide,
    /// Custom aspect ratio
    Custom(f32, f32),
}

impl AspectRatio {
    /// Returns the aspect ratio as a float (width/height)
    #[must_use]
    pub fn ratio(self) -> f32 {
        match self {
            Self::MobilePortrait => 9.0 / 16.0,
            Self::MobileLandscape | Self::Standard => 16.0 / 9.0,
            Self::Ultrawide => 21.0 / 9.0,
            Self::SuperUltrawide => 32.0 / 9.0,
            Self::Custom(w, h) => w / h,
        }
    }

    /// Detects aspect ratio from dimensions
    #[must_use]
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        let ratio = width as f32 / height as f32;
        if (ratio - 9.0 / 16.0).abs() < 0.1 {
            Self::MobilePortrait
        } else if (ratio - 16.0 / 9.0).abs() < 0.1 {
            Self::Standard
        } else if (ratio - 21.0 / 9.0).abs() < 0.1 {
            Self::Ultrawide
        } else if (ratio - 32.0 / 9.0).abs() < 0.1 {
            Self::SuperUltrawide
        } else {
            Self::Custom(width as f32, height as f32)
        }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::Standard
    }
}

/// Viewport representing the rendering area
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Safe area for gameplay (16:9 within any aspect ratio)
    pub safe_area: Rect,
    /// Detected aspect ratio
    pub aspect_ratio: AspectRatio,
}

impl Viewport {
    /// Creates a new viewport
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let aspect_ratio = AspectRatio::from_dimensions(width, height);
        let safe_area = calculate_safe_area(width, height);
        Self {
            width,
            height,
            safe_area,
            aspect_ratio,
        }
    }

    /// Resizes the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.aspect_ratio = AspectRatio::from_dimensions(width, height);
        self.safe_area = calculate_safe_area(width, height);
    }

    /// Converts screen coordinates to world coordinates
    #[must_use]
    pub fn screen_to_world(&self, screen_pos: Vec2, camera: &Camera) -> Vec2 {
        let center = Vec2::new(self.width as f32 / 2.0, self.height as f32 / 2.0);
        let offset = screen_pos - center;
        Vec2::new(
            camera.position.x + offset.x / camera.zoom,
            camera.position.y - offset.y / camera.zoom, // Y is flipped
        )
    }

    /// Converts world coordinates to screen coordinates
    #[must_use]
    pub fn world_to_screen(&self, world_pos: Vec2, camera: &Camera) -> Vec2 {
        let center = Vec2::new(self.width as f32 / 2.0, self.height as f32 / 2.0);
        Vec2::new(
            center.x + (world_pos.x - camera.position.x) * camera.zoom,
            center.y - (world_pos.y - camera.position.y) * camera.zoom,
        )
    }

    /// Checks if a world position is visible
    #[must_use]
    pub fn is_visible(&self, world_pos: Vec2, camera: &Camera) -> bool {
        let screen_pos = self.world_to_screen(world_pos, camera);
        screen_pos.x >= 0.0
            && screen_pos.x <= self.width as f32
            && screen_pos.y >= 0.0
            && screen_pos.y <= self.height as f32
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

/// Calculates the 16:9 safe area within any viewport
fn calculate_safe_area(width: u32, height: u32) -> Rect {
    let target_ratio = 16.0 / 9.0;
    let current_ratio = width as f32 / height as f32;

    if current_ratio > target_ratio {
        // Wider than 16:9 - pillarbox
        let safe_width = height as f32 * target_ratio;
        let offset = (width as f32 - safe_width) / 2.0;
        Rect::new(offset, 0.0, safe_width, height as f32)
    } else {
        // Taller than 16:9 - letterbox
        let safe_height = width as f32 / target_ratio;
        let offset = (height as f32 - safe_height) / 2.0;
        Rect::new(0.0, offset, width as f32, safe_height)
    }
}

/// Render command for batched rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderCommand {
    /// Clear the screen with a color
    Clear {
        /// RGBA color
        color: [f32; 4],
    },
    /// Draw a sprite
    DrawSprite {
        /// Texture ID
        texture_id: u32,
        /// Position
        position: Position,
        /// Size
        size: Vec2,
        /// Source rectangle (for sprite sheets)
        source: Option<Rect>,
        /// Tint color
        color: [f32; 4],
    },
    /// Draw a rectangle
    DrawRect {
        /// Rectangle bounds
        rect: Rect,
        /// Fill color
        color: [f32; 4],
    },
}

/// Render queue for batched rendering
#[derive(Debug, Default)]
pub struct RenderQueue {
    commands: Vec<RenderCommand>,
}

impl RenderQueue {
    /// Creates a new render queue
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the queue
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Adds a command to the queue
    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    /// Returns the commands
    #[must_use]
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    /// Returns the number of commands
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Checks if the queue is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Calculates UI element position based on anchor and viewport
#[must_use]
pub fn calculate_anchored_position(
    anchor: Anchor,
    offset: Vec2,
    element_size: Vec2,
    viewport: &Viewport,
    scale_mode: ScaleMode,
) -> Vec2 {
    let (ax, ay) = anchor.normalized();
    let vw = viewport.width as f32;
    let vh = viewport.height as f32;

    // Calculate base position from anchor
    let base_x = vw * ax;
    let base_y = vh * ay;

    // Apply scaling based on mode
    let scale = match scale_mode {
        ScaleMode::Adaptive => vh.min(vw) / 1080.0, // Scale based on shortest dimension
        ScaleMode::PixelPerfect => 1.0,
        ScaleMode::Fixed => 1.0,
    };

    // Calculate final position (centered on anchor point)
    Vec2::new(
        base_x + offset.x * scale - element_size.x * scale * ax,
        base_y + offset.y * scale - element_size.y * scale * ay,
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio_standard() {
        let ratio = AspectRatio::Standard;
        assert!((ratio.ratio() - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_aspect_ratio_ultrawide() {
        let ratio = AspectRatio::SuperUltrawide;
        assert!((ratio.ratio() - 32.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_aspect_ratio_detection() {
        assert!(matches!(
            AspectRatio::from_dimensions(1920, 1080),
            AspectRatio::Standard
        ));
        assert!(matches!(
            AspectRatio::from_dimensions(5120, 1440),
            AspectRatio::SuperUltrawide
        ));
    }

    #[test]
    fn test_viewport_safe_area_standard() {
        let viewport = Viewport::new(1920, 1080);
        // 16:9 should have full safe area
        assert!((viewport.safe_area.width - 1920.0).abs() < 1.0);
        assert!((viewport.safe_area.height - 1080.0).abs() < 1.0);
    }

    #[test]
    fn test_viewport_safe_area_ultrawide() {
        let viewport = Viewport::new(5120, 1440);
        // 32:9 should have pillarboxed 16:9 safe area
        let expected_width = 1440.0 * 16.0 / 9.0;
        assert!((viewport.safe_area.width - expected_width).abs() < 1.0);
    }

    #[test]
    fn test_screen_to_world_center() {
        let viewport = Viewport::new(800, 600);
        let camera = Camera::new();

        let center = Vec2::new(400.0, 300.0);
        let world = viewport.screen_to_world(center, &camera);

        assert!(world.x.abs() < 0.1);
        assert!(world.y.abs() < 0.1);
    }

    #[test]
    fn test_world_to_screen_roundtrip() {
        let viewport = Viewport::new(800, 600);
        let camera = Camera::new();

        let world_pos = Vec2::new(100.0, 50.0);
        let screen = viewport.world_to_screen(world_pos, &camera);
        let back = viewport.screen_to_world(screen, &camera);

        assert!((back.x - world_pos.x).abs() < 0.1);
        assert!((back.y - world_pos.y).abs() < 0.1);
    }

    #[test]
    fn test_render_queue() {
        let mut queue = RenderQueue::new();
        assert!(queue.is_empty());

        queue.push(RenderCommand::Clear {
            color: [0.0, 0.0, 0.0, 1.0],
        });
        assert_eq!(queue.len(), 1);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_anchored_position_top_left() {
        let viewport = Viewport::new(1920, 1080);
        let pos = calculate_anchored_position(
            Anchor::TopLeft,
            Vec2::new(10.0, 10.0),
            Vec2::new(100.0, 50.0),
            &viewport,
            ScaleMode::Fixed,
        );

        assert!((pos.x - 10.0).abs() < 1.0);
        assert!((pos.y - 10.0).abs() < 1.0);
    }

    #[test]
    fn test_anchored_position_center() {
        let viewport = Viewport::new(1920, 1080);
        let pos = calculate_anchored_position(
            Anchor::Center,
            Vec2::ZERO,
            Vec2::new(100.0, 50.0),
            &viewport,
            ScaleMode::Fixed,
        );

        // Should be centered
        assert!((pos.x - (960.0 - 50.0)).abs() < 1.0);
        assert!((pos.y - (540.0 - 25.0)).abs() < 1.0);
    }
}
