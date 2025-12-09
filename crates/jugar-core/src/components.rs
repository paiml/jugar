//! Common game components for universal design
//!
//! These components support both mobile (6") and ultrawide (49") displays
//! through responsive anchoring and scaling systems.

use core::fmt;

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// 2D position component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Position {
    /// Creates a new position
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Creates a position at the origin
    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Converts to a glam Vec2
    #[must_use]
    pub const fn as_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Creates from a glam Vec2
    #[must_use]
    pub const fn from_vec2(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }

    /// Distance to another position
    #[must_use]
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// 2D velocity component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    /// X velocity
    pub x: f32,
    /// Y velocity
    pub y: f32,
}

impl Velocity {
    /// Creates a new velocity
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Creates a zero velocity
    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Returns the speed (magnitude)
    #[must_use]
    pub fn speed(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Normalizes the velocity to unit length, or returns zero if magnitude is zero
    #[must_use]
    pub fn normalized(self) -> Self {
        let mag = self.speed();
        if mag < f32::EPSILON {
            Self::zero()
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        }
    }

    /// Scales the velocity by a factor
    #[must_use]
    pub const fn scaled(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vel({:.2}, {:.2})", self.x, self.y)
    }
}

/// UI Anchor for responsive layout
///
/// Anchors determine how UI elements position themselves relative to
/// their parent container. This enables the same UI to work on both
/// mobile (6") and ultrawide (49") displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Anchor {
    /// Center of the container
    #[default]
    Center,
    /// Top-left corner
    TopLeft,
    /// Top-center
    TopCenter,
    /// Top-right corner
    TopRight,
    /// Middle-left
    MiddleLeft,
    /// Middle-right
    MiddleRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-center
    BottomCenter,
    /// Bottom-right corner
    BottomRight,
    /// Stretch to fill available space
    Stretch,
}

impl Anchor {
    /// Returns the normalized anchor point (0.0 to 1.0)
    #[must_use]
    pub const fn normalized(self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopCenter => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::MiddleLeft => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::MiddleRight => (1.0, 0.5),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomCenter => (0.5, 1.0),
            Self::BottomRight => (1.0, 1.0),
            Self::Stretch => (0.5, 0.5), // Center for stretch
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Center => write!(f, "center"),
            Self::TopLeft => write!(f, "top-left"),
            Self::TopCenter => write!(f, "top-center"),
            Self::TopRight => write!(f, "top-right"),
            Self::MiddleLeft => write!(f, "middle-left"),
            Self::MiddleRight => write!(f, "middle-right"),
            Self::BottomLeft => write!(f, "bottom-left"),
            Self::BottomCenter => write!(f, "bottom-center"),
            Self::BottomRight => write!(f, "bottom-right"),
            Self::Stretch => write!(f, "stretch"),
        }
    }
}

/// Scale mode for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ScaleMode {
    /// Scale based on shortest dimension (height on landscape)
    #[default]
    Adaptive,
    /// Maintain pixel-perfect rendering (for pixel art)
    PixelPerfect,
    /// Fixed size in pixels, no scaling
    Fixed,
}

/// UI Element component for responsive layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiElement {
    /// Anchor point for positioning
    pub anchor: Anchor,
    /// Offset from anchor point
    pub offset: Vec2,
    /// Size of the element
    pub size: Vec2,
    /// How the element scales
    pub scale_mode: ScaleMode,
    /// Whether the element is visible
    pub visible: bool,
    /// Z-order for layering (higher = on top)
    pub z_order: i32,
}

impl UiElement {
    /// Creates a new UI element with default settings
    #[must_use]
    pub fn new(size: Vec2) -> Self {
        Self {
            anchor: Anchor::Center,
            offset: Vec2::ZERO,
            size,
            scale_mode: ScaleMode::Adaptive,
            visible: true,
            z_order: 0,
        }
    }

    /// Sets the anchor
    #[must_use]
    pub const fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the offset
    #[must_use]
    pub const fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the scale mode
    #[must_use]
    pub const fn with_scale_mode(mut self, mode: ScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }

    /// Sets the z-order
    #[must_use]
    pub const fn with_z_order(mut self, z: i32) -> Self {
        self.z_order = z;
        self
    }

    /// Calculates the actual position given a container size
    #[must_use]
    pub fn calculate_position(&self, container_size: Vec2) -> Vec2 {
        let (ax, ay) = self.anchor.normalized();
        Vec2::new(
            container_size.x * ax + self.offset.x,
            container_size.y * ay + self.offset.y,
        )
    }
}

impl Default for UiElement {
    fn default() -> Self {
        Self::new(Vec2::new(100.0, 100.0))
    }
}

/// Camera component with aspect ratio handling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Zoom level (1.0 = normal)
    pub zoom: f32,
    /// Target resolution for pixel-perfect rendering
    pub target_resolution: Option<Vec2>,
    /// Whether to maintain aspect ratio
    pub keep_aspect: bool,
    /// Field of view for perspective (degrees)
    pub fov: f32,
    /// Camera position in world space
    pub position: Position,
}

impl Camera {
    /// Creates a new camera with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            target_resolution: None,
            keep_aspect: true,
            fov: 60.0,
            position: Position::zero(),
        }
    }

    /// Creates a camera for pixel art with fixed resolution
    #[must_use]
    pub fn pixel_art(width: f32, height: f32) -> Self {
        Self {
            zoom: 1.0,
            target_resolution: Some(Vec2::new(width, height)),
            keep_aspect: true,
            fov: 60.0,
            position: Position::zero(),
        }
    }

    /// Sets the zoom level
    #[must_use]
    pub const fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// Sets the position
    #[must_use]
    pub const fn with_position(mut self, pos: Position) -> Self {
        self.position = pos;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Sprite component for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Texture identifier
    pub texture_id: u32,
    /// Source rectangle in texture (for sprite sheets)
    pub source: Option<Rect>,
    /// Tint color (RGBA)
    pub color: [f32; 4],
    /// Flip horizontally
    pub flip_x: bool,
    /// Flip vertically
    pub flip_y: bool,
}

impl Sprite {
    /// Creates a new sprite with the given texture
    #[must_use]
    pub const fn new(texture_id: u32) -> Self {
        Self {
            texture_id,
            source: None,
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
        }
    }

    /// Sets the source rectangle for sprite sheets
    #[must_use]
    pub const fn with_source(mut self, rect: Rect) -> Self {
        self.source = Some(rect);
        self
    }

    /// Sets the tint color
    #[must_use]
    pub const fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Rectangle for collision and rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle at origin with given size
    #[must_use]
    pub const fn from_size(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    /// Checks if a point is inside the rectangle
    #[must_use]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Checks if two rectangles overlap
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns the center point
    #[must_use]
    pub const fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ==================== POSITION TESTS ====================

    #[test]
    fn test_position_new() {
        let pos = Position::new(10.0, 20.0);
        assert!((pos.x - 10.0).abs() < f32::EPSILON);
        assert!((pos.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position_zero() {
        let pos = Position::zero();
        assert!((pos.x).abs() < f32::EPSILON);
        assert!((pos.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert!((p1.distance_to(p2) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(1.5, 2.5);
        assert_eq!(format!("{pos}"), "(1.50, 2.50)");
    }

    // ==================== VELOCITY TESTS ====================

    #[test]
    fn test_velocity_speed() {
        let vel = Velocity::new(3.0, 4.0);
        assert!((vel.speed() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity_normalized() {
        let vel = Velocity::new(3.0, 4.0);
        let norm = vel.normalized();
        assert!((norm.speed() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_normalized_zero() {
        let vel = Velocity::zero();
        let norm = vel.normalized();
        assert!((norm.x).abs() < f32::EPSILON);
        assert!((norm.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_velocity_scaled() {
        let vel = Velocity::new(1.0, 2.0);
        let scaled = vel.scaled(2.0);
        assert!((scaled.x - 2.0).abs() < f32::EPSILON);
        assert!((scaled.y - 4.0).abs() < f32::EPSILON);
    }

    // ==================== ANCHOR TESTS ====================

    #[test]
    fn test_anchor_normalized() {
        assert_eq!(Anchor::TopLeft.normalized(), (0.0, 0.0));
        assert_eq!(Anchor::Center.normalized(), (0.5, 0.5));
        assert_eq!(Anchor::BottomRight.normalized(), (1.0, 1.0));
    }

    #[test]
    fn test_anchor_display() {
        assert_eq!(format!("{}", Anchor::TopLeft), "top-left");
        assert_eq!(format!("{}", Anchor::Center), "center");
    }

    // ==================== UI ELEMENT TESTS ====================

    #[test]
    fn test_ui_element_position_calculation() {
        let elem = UiElement::new(Vec2::new(100.0, 50.0))
            .with_anchor(Anchor::TopLeft)
            .with_offset(Vec2::new(10.0, 20.0));

        let pos = elem.calculate_position(Vec2::new(800.0, 600.0));
        assert!((pos.x - 10.0).abs() < f32::EPSILON);
        assert!((pos.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ui_element_center_position() {
        let elem = UiElement::new(Vec2::new(100.0, 50.0)).with_anchor(Anchor::Center);

        let pos = elem.calculate_position(Vec2::new(800.0, 600.0));
        assert!((pos.x - 400.0).abs() < f32::EPSILON);
        assert!((pos.y - 300.0).abs() < f32::EPSILON);
    }

    // ==================== RECT TESTS ====================

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains_point(50.0, 30.0));
        assert!(!rect.contains_point(0.0, 0.0));
        assert!(rect.contains_point(10.0, 10.0)); // Edge
    }

    #[test]
    fn test_rect_overlaps() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let (cx, cy) = rect.center();
        assert!((cx - 50.0).abs() < f32::EPSILON);
        assert!((cy - 25.0).abs() < f32::EPSILON);
    }

    // ==================== CAMERA TESTS ====================

    #[test]
    fn test_camera_default() {
        let cam = Camera::new();
        assert!((cam.zoom - 1.0).abs() < f32::EPSILON);
        assert!(cam.keep_aspect);
    }

    #[test]
    fn test_camera_pixel_art() {
        let cam = Camera::pixel_art(320.0, 240.0);
        assert!(cam.target_resolution.is_some());
        let res = cam.target_resolution.unwrap();
        assert!((res.x - 320.0).abs() < f32::EPSILON);
    }
}
