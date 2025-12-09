//! Canvas2D render commands for browser execution.
//!
//! This module provides JSON-serializable render commands that are executed
//! by a minimal JavaScript Canvas2D renderer. All computation happens in Rust;
//! JavaScript only draws primitives.

#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

/// A color represented as RGBA components (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0.0 to 1.0)
    pub r: f32,
    /// Green component (0.0 to 1.0)
    pub g: f32,
    /// Blue component (0.0 to 1.0)
    pub b: f32,
    /// Alpha component (0.0 to 1.0)
    pub a: f32,
}

impl Color {
    /// Creates a new color from RGBA components.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from RGBA array.
    #[must_use]
    pub const fn from_array(rgba: [f32; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    /// Converts to RGBA array.
    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Converts to CSS rgba() string.
    #[must_use]
    pub fn to_css_rgba(self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            self.a
        )
    }

    /// Black color.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// White color.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Transparent color.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Red color.
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// Green color.
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);

    /// Blue color.
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl From<[f32; 4]> for Color {
    fn from(rgba: [f32; 4]) -> Self {
        Self::from_array(rgba)
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.to_array()
    }
}

/// Text alignment options for Canvas2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    /// Left-aligned text.
    #[default]
    Left,
    /// Center-aligned text.
    Center,
    /// Right-aligned text.
    Right,
}

/// Text baseline options for Canvas2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextBaseline {
    /// Top baseline.
    Top,
    /// Middle baseline.
    #[default]
    Middle,
    /// Bottom baseline.
    Bottom,
    /// Alphabetic baseline.
    Alphabetic,
}

/// Canvas2D render commands that are serialized to JSON.
///
/// These commands are designed to be minimal and directly map to Canvas2D API calls.
/// The JavaScript renderer simply iterates over commands and executes them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Canvas2DCommand {
    /// Clear the entire canvas with a color.
    Clear {
        /// Fill color for clearing
        color: Color,
    },

    /// Fill a rectangle.
    FillRect {
        /// X position
        x: f32,
        /// Y position
        y: f32,
        /// Width
        width: f32,
        /// Height
        height: f32,
        /// Fill color
        color: Color,
    },

    /// Stroke a rectangle outline.
    StrokeRect {
        /// X position
        x: f32,
        /// Y position
        y: f32,
        /// Width
        width: f32,
        /// Height
        height: f32,
        /// Stroke color
        color: Color,
        /// Line width
        line_width: f32,
    },

    /// Fill a circle.
    FillCircle {
        /// Center X
        x: f32,
        /// Center Y
        y: f32,
        /// Radius
        radius: f32,
        /// Fill color
        color: Color,
    },

    /// Stroke a circle outline.
    StrokeCircle {
        /// Center X
        x: f32,
        /// Center Y
        y: f32,
        /// Radius
        radius: f32,
        /// Stroke color
        color: Color,
        /// Line width
        line_width: f32,
    },

    /// Draw a line.
    Line {
        /// Start X
        x1: f32,
        /// Start Y
        y1: f32,
        /// End X
        x2: f32,
        /// End Y
        y2: f32,
        /// Line color
        color: Color,
        /// Line width
        line_width: f32,
    },

    /// Draw text.
    FillText {
        /// Text content
        text: String,
        /// X position
        x: f32,
        /// Y position
        y: f32,
        /// CSS font string (e.g., "48px monospace")
        font: String,
        /// Text color
        color: Color,
        /// Text alignment
        align: TextAlign,
        /// Text baseline
        baseline: TextBaseline,
    },

    /// Draw an image/sprite from a loaded texture.
    DrawImage {
        /// Texture ID (index into loaded textures array)
        texture_id: u32,
        /// Destination X
        x: f32,
        /// Destination Y
        y: f32,
        /// Destination width
        width: f32,
        /// Destination height
        height: f32,
    },

    /// Draw a portion of an image (sprite sheet).
    DrawImageSlice {
        /// Texture ID
        texture_id: u32,
        /// Source X in texture
        src_x: f32,
        /// Source Y in texture
        src_y: f32,
        /// Source width
        src_width: f32,
        /// Source height
        src_height: f32,
        /// Destination X
        dst_x: f32,
        /// Destination Y
        dst_y: f32,
        /// Destination width
        dst_width: f32,
        /// Destination height
        dst_height: f32,
    },

    /// Save the current canvas state.
    Save,

    /// Restore the previously saved canvas state.
    Restore,

    /// Translate the canvas origin.
    Translate {
        /// X translation
        x: f32,
        /// Y translation
        y: f32,
    },

    /// Rotate the canvas.
    Rotate {
        /// Rotation angle in radians
        angle: f32,
    },

    /// Scale the canvas.
    Scale {
        /// X scale factor
        x: f32,
        /// Y scale factor
        y: f32,
    },

    /// Set global alpha (opacity).
    SetAlpha {
        /// Alpha value (0.0 to 1.0)
        alpha: f32,
    },
}

/// A frame's worth of render commands.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderFrame {
    /// The commands to execute this frame.
    pub commands: Vec<Canvas2DCommand>,
}

impl RenderFrame {
    /// Creates a new empty render frame.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a render frame with an initial capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
        }
    }

    /// Clears all commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Adds a command to the frame.
    pub fn push(&mut self, cmd: Canvas2DCommand) {
        self.commands.push(cmd);
    }

    /// Clears the canvas with a color.
    pub fn clear_screen(&mut self, color: Color) {
        self.push(Canvas2DCommand::Clear { color });
    }

    /// Draws a filled rectangle.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.push(Canvas2DCommand::FillRect {
            x,
            y,
            width,
            height,
            color,
        });
    }

    /// Draws a filled circle.
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.push(Canvas2DCommand::FillCircle {
            x,
            y,
            radius,
            color,
        });
    }

    /// Draws text.
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32, font: &str, color: Color) {
        self.push(Canvas2DCommand::FillText {
            text: text.to_string(),
            x,
            y,
            font: font.to_string(),
            color,
            align: TextAlign::default(),
            baseline: TextBaseline::default(),
        });
    }

    /// Draws text with alignment options.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_text_aligned(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font: &str,
        color: Color,
        align: TextAlign,
        baseline: TextBaseline,
    ) {
        self.push(Canvas2DCommand::FillText {
            text: text.to_string(),
            x,
            y,
            font: font.to_string(),
            color,
            align,
            baseline,
        });
    }

    /// Draws a line.
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, line_width: f32) {
        self.push(Canvas2DCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            line_width,
        });
    }

    /// Strokes a rectangle outline.
    pub fn stroke_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        line_width: f32,
    ) {
        self.push(Canvas2DCommand::StrokeRect {
            x,
            y,
            width,
            height,
            color,
            line_width,
        });
    }

    /// Returns the number of commands.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::len() is not const
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Checks if the frame has no commands.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::is_empty() is not const
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Serializes the frame to JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.commands)
    }

    /// Serializes the frame to pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.commands)
    }
}

/// Converts jugar-render `RenderCommand` to `Canvas2DCommand`.
///
/// This bridges the platform-agnostic render commands to Canvas2D-specific commands.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn convert_render_command(cmd: &jugar_render::RenderCommand) -> Option<Canvas2DCommand> {
    match cmd {
        jugar_render::RenderCommand::Clear { color } => Some(Canvas2DCommand::Clear {
            color: Color::from_array(*color),
        }),
        jugar_render::RenderCommand::DrawRect { rect, color } => Some(Canvas2DCommand::FillRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            color: Color::from_array(*color),
        }),
        jugar_render::RenderCommand::DrawSprite { .. } => {
            // Sprites require texture management which is handled separately
            None
        }
    }
}

/// Converts a slice of `RenderCommand`s to a `RenderFrame`.
#[must_use]
pub fn convert_render_queue(commands: &[jugar_render::RenderCommand]) -> RenderFrame {
    let mut frame = RenderFrame::with_capacity(commands.len());
    for cmd in commands {
        if let Some(canvas_cmd) = convert_render_command(cmd) {
            frame.push(canvas_cmd);
        }
    }
    frame
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.5, 0.25, 0.75, 1.0);
        assert!((color.r - 0.5).abs() < f32::EPSILON);
        assert!((color.g - 0.25).abs() < f32::EPSILON);
        assert!((color.b - 0.75).abs() < f32::EPSILON);
        assert!((color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_from_array() {
        let color = Color::from_array([0.1, 0.2, 0.3, 0.4]);
        assert!((color.r - 0.1).abs() < f32::EPSILON);
        assert!((color.g - 0.2).abs() < f32::EPSILON);
        assert!((color.b - 0.3).abs() < f32::EPSILON);
        assert!((color.a - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_to_array() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);
        let arr = color.to_array();
        assert!((arr[0] - 0.1).abs() < f32::EPSILON);
        assert!((arr[1] - 0.2).abs() < f32::EPSILON);
        assert!((arr[2] - 0.3).abs() < f32::EPSILON);
        assert!((arr[3] - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_to_css_rgba() {
        let color = Color::new(1.0, 0.5, 0.0, 0.8);
        let css = color.to_css_rgba();
        assert_eq!(css, "rgba(255, 127, 0, 0.8)");
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_default() {
        let color = Color::default();
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn test_color_from_trait() {
        let color: Color = [0.5, 0.5, 0.5, 1.0].into();
        assert!((color.r - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_into_array() {
        let color = Color::new(0.5, 0.5, 0.5, 1.0);
        let arr: [f32; 4] = color.into();
        assert!((arr[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_text_align_default() {
        assert_eq!(TextAlign::default(), TextAlign::Left);
    }

    #[test]
    fn test_text_baseline_default() {
        assert_eq!(TextBaseline::default(), TextBaseline::Middle);
    }

    #[test]
    fn test_canvas2d_command_clear_serialization() {
        let cmd = Canvas2DCommand::Clear {
            color: Color::BLACK,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"Clear\""));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_fill_rect_serialization() {
        let cmd = Canvas2DCommand::FillRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            color: Color::WHITE,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"FillRect\""));
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"width\":100"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_fill_circle_serialization() {
        let cmd = Canvas2DCommand::FillCircle {
            x: 50.0,
            y: 50.0,
            radius: 25.0,
            color: Color::RED,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"FillCircle\""));
        assert!(json.contains("\"radius\":25"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_stroke_rect_serialization() {
        let cmd = Canvas2DCommand::StrokeRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 150.0,
            color: Color::GREEN,
            line_width: 2.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"StrokeRect\""));
        assert!(json.contains("\"line_width\":2"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_stroke_circle_serialization() {
        let cmd = Canvas2DCommand::StrokeCircle {
            x: 100.0,
            y: 100.0,
            radius: 50.0,
            color: Color::BLUE,
            line_width: 3.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"StrokeCircle\""));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_line_serialization() {
        let cmd = Canvas2DCommand::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::WHITE,
            line_width: 1.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"Line\""));
        assert!(json.contains("\"x1\":0"));
        assert!(json.contains("\"x2\":100"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_fill_text_serialization() {
        let cmd = Canvas2DCommand::FillText {
            text: "Score: 42".to_string(),
            x: 10.0,
            y: 30.0,
            font: "24px monospace".to_string(),
            color: Color::WHITE,
            align: TextAlign::Left,
            baseline: TextBaseline::Top,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"FillText\""));
        assert!(json.contains("\"text\":\"Score: 42\""));
        assert!(json.contains("\"font\":\"24px monospace\""));
        assert!(json.contains("\"align\":\"left\""));
        assert!(json.contains("\"baseline\":\"top\""));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_draw_image_serialization() {
        let cmd = Canvas2DCommand::DrawImage {
            texture_id: 0,
            x: 100.0,
            y: 100.0,
            width: 64.0,
            height: 64.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"DrawImage\""));
        assert!(json.contains("\"texture_id\":0"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_draw_image_slice_serialization() {
        let cmd = Canvas2DCommand::DrawImageSlice {
            texture_id: 1,
            src_x: 0.0,
            src_y: 0.0,
            src_width: 32.0,
            src_height: 32.0,
            dst_x: 200.0,
            dst_y: 200.0,
            dst_width: 64.0,
            dst_height: 64.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"DrawImageSlice\""));
        assert!(json.contains("\"src_width\":32"));
        assert!(json.contains("\"dst_width\":64"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_canvas2d_command_transform_serialization() {
        // Save
        let save = Canvas2DCommand::Save;
        let json = serde_json::to_string(&save).unwrap();
        assert!(json.contains("\"type\":\"Save\""));

        // Restore
        let restore = Canvas2DCommand::Restore;
        let json = serde_json::to_string(&restore).unwrap();
        assert!(json.contains("\"type\":\"Restore\""));

        // Translate
        let translate = Canvas2DCommand::Translate { x: 50.0, y: 100.0 };
        let json = serde_json::to_string(&translate).unwrap();
        assert!(json.contains("\"type\":\"Translate\""));

        // Rotate
        let rotate = Canvas2DCommand::Rotate {
            angle: core::f32::consts::PI,
        };
        let json = serde_json::to_string(&rotate).unwrap();
        assert!(json.contains("\"type\":\"Rotate\""));

        // Scale
        let scale = Canvas2DCommand::Scale { x: 2.0, y: 2.0 };
        let json = serde_json::to_string(&scale).unwrap();
        assert!(json.contains("\"type\":\"Scale\""));
    }

    #[test]
    fn test_canvas2d_command_set_alpha_serialization() {
        let cmd = Canvas2DCommand::SetAlpha { alpha: 0.5 };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"SetAlpha\""));
        assert!(json.contains("\"alpha\":0.5"));

        let deserialized: Canvas2DCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_render_frame_new() {
        let frame = RenderFrame::new();
        assert!(frame.is_empty());
        assert_eq!(frame.len(), 0);
    }

    #[test]
    fn test_render_frame_with_capacity() {
        let frame = RenderFrame::with_capacity(100);
        assert!(frame.is_empty());
        assert!(frame.commands.capacity() >= 100);
    }

    #[test]
    fn test_render_frame_push() {
        let mut frame = RenderFrame::new();
        frame.push(Canvas2DCommand::Clear {
            color: Color::BLACK,
        });
        assert_eq!(frame.len(), 1);
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_render_frame_clear() {
        let mut frame = RenderFrame::new();
        frame.push(Canvas2DCommand::Clear {
            color: Color::BLACK,
        });
        frame.clear();
        assert!(frame.is_empty());
    }

    #[test]
    fn test_render_frame_clear_screen() {
        let mut frame = RenderFrame::new();
        frame.clear_screen(Color::BLACK);
        assert_eq!(frame.len(), 1);
        assert_eq!(
            frame.commands[0],
            Canvas2DCommand::Clear {
                color: Color::BLACK
            }
        );
    }

    #[test]
    fn test_render_frame_fill_rect() {
        let mut frame = RenderFrame::new();
        frame.fill_rect(10.0, 20.0, 100.0, 50.0, Color::WHITE);
        assert_eq!(frame.len(), 1);
        match &frame.commands[0] {
            Canvas2DCommand::FillRect {
                x,
                y,
                width,
                height,
                ..
            } => {
                assert!((x - 10.0).abs() < f32::EPSILON);
                assert!((y - 20.0).abs() < f32::EPSILON);
                assert!((width - 100.0).abs() < f32::EPSILON);
                assert!((height - 50.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected FillRect"),
        }
    }

    #[test]
    fn test_render_frame_fill_circle() {
        let mut frame = RenderFrame::new();
        frame.fill_circle(50.0, 50.0, 25.0, Color::RED);
        assert_eq!(frame.len(), 1);
        match &frame.commands[0] {
            Canvas2DCommand::FillCircle { x, y, radius, .. } => {
                assert!((x - 50.0).abs() < f32::EPSILON);
                assert!((y - 50.0).abs() < f32::EPSILON);
                assert!((radius - 25.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected FillCircle"),
        }
    }

    #[test]
    fn test_render_frame_fill_text() {
        let mut frame = RenderFrame::new();
        frame.fill_text("Hello", 10.0, 20.0, "24px sans-serif", Color::WHITE);
        assert_eq!(frame.len(), 1);
        match &frame.commands[0] {
            Canvas2DCommand::FillText { text, font, .. } => {
                assert_eq!(text, "Hello");
                assert_eq!(font, "24px sans-serif");
            }
            _ => panic!("Expected FillText"),
        }
    }

    #[test]
    fn test_render_frame_fill_text_aligned() {
        let mut frame = RenderFrame::new();
        frame.fill_text_aligned(
            "Centered",
            100.0,
            50.0,
            "32px monospace",
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        assert_eq!(frame.len(), 1);
        match &frame.commands[0] {
            Canvas2DCommand::FillText {
                align, baseline, ..
            } => {
                assert_eq!(*align, TextAlign::Center);
                assert_eq!(*baseline, TextBaseline::Middle);
            }
            _ => panic!("Expected FillText"),
        }
    }

    #[test]
    fn test_render_frame_line() {
        let mut frame = RenderFrame::new();
        frame.line(0.0, 0.0, 100.0, 100.0, Color::WHITE, 2.0);
        assert_eq!(frame.len(), 1);
        match &frame.commands[0] {
            Canvas2DCommand::Line {
                x1,
                y1,
                x2,
                y2,
                line_width,
                ..
            } => {
                assert!((x1 - 0.0).abs() < f32::EPSILON);
                assert!((y1 - 0.0).abs() < f32::EPSILON);
                assert!((x2 - 100.0).abs() < f32::EPSILON);
                assert!((y2 - 100.0).abs() < f32::EPSILON);
                assert!((line_width - 2.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn test_render_frame_to_json() {
        let mut frame = RenderFrame::new();
        frame.clear_screen(Color::BLACK);
        frame.fill_rect(50.0, 200.0, 20.0, 120.0, Color::WHITE);

        let json = frame.to_json().unwrap();
        assert!(json.contains("\"type\":\"Clear\""));
        assert!(json.contains("\"type\":\"FillRect\""));
    }

    #[test]
    fn test_render_frame_to_json_pretty() {
        let mut frame = RenderFrame::new();
        frame.clear_screen(Color::BLACK);

        let json = frame.to_json_pretty().unwrap();
        assert!(json.contains('\n')); // Pretty print has newlines
    }

    #[test]
    fn test_render_frame_default() {
        let frame = RenderFrame::default();
        assert!(frame.is_empty());
    }

    #[test]
    fn test_pong_like_frame() {
        // Simulate a Pong game frame
        let mut frame = RenderFrame::new();

        // Clear screen black
        frame.clear_screen(Color::BLACK);

        // Draw center line
        frame.line(400.0, 0.0, 400.0, 600.0, Color::WHITE, 2.0);

        // Left paddle
        frame.fill_rect(20.0, 250.0, 10.0, 100.0, Color::WHITE);

        // Right paddle
        frame.fill_rect(770.0, 250.0, 10.0, 100.0, Color::WHITE);

        // Ball
        frame.fill_circle(400.0, 300.0, 10.0, Color::WHITE);

        // Scores
        frame.fill_text_aligned(
            "3",
            200.0,
            50.0,
            "48px monospace",
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Top,
        );
        frame.fill_text_aligned(
            "5",
            600.0,
            50.0,
            "48px monospace",
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Top,
        );

        assert_eq!(frame.len(), 7);

        let json = frame.to_json().unwrap();
        // Verify JSON can be parsed
        let commands: Vec<Canvas2DCommand> = serde_json::from_str(&json).unwrap();
        assert_eq!(commands.len(), 7);
    }

    #[test]
    fn test_convert_render_command_clear() {
        let cmd = jugar_render::RenderCommand::Clear {
            color: [0.0, 0.0, 0.0, 1.0],
        };
        let converted = convert_render_command(&cmd).unwrap();
        assert!(matches!(converted, Canvas2DCommand::Clear { .. }));
    }

    #[test]
    fn test_convert_render_command_draw_rect() {
        let cmd = jugar_render::RenderCommand::DrawRect {
            rect: jugar_core::Rect::new(10.0, 20.0, 100.0, 50.0),
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let converted = convert_render_command(&cmd).unwrap();
        match converted {
            Canvas2DCommand::FillRect {
                x,
                y,
                width,
                height,
                ..
            } => {
                assert!((x - 10.0).abs() < f32::EPSILON);
                assert!((y - 20.0).abs() < f32::EPSILON);
                assert!((width - 100.0).abs() < f32::EPSILON);
                assert!((height - 50.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected FillRect"),
        }
    }

    #[test]
    fn test_convert_render_command_sprite_returns_none() {
        use glam::Vec2;
        use jugar_core::Position;
        let cmd = jugar_render::RenderCommand::DrawSprite {
            texture_id: 0,
            position: Position::zero(),
            size: Vec2::new(64.0, 64.0),
            source: None,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        assert!(convert_render_command(&cmd).is_none());
    }

    #[test]
    fn test_convert_render_queue() {
        let commands = vec![
            jugar_render::RenderCommand::Clear {
                color: [0.0, 0.0, 0.0, 1.0],
            },
            jugar_render::RenderCommand::DrawRect {
                rect: jugar_core::Rect::new(0.0, 0.0, 100.0, 100.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let frame = convert_render_queue(&commands);
        assert_eq!(frame.len(), 2);
    }

    #[test]
    fn test_convert_render_queue_skips_sprites() {
        use glam::Vec2;
        use jugar_core::Position;
        let commands = vec![
            jugar_render::RenderCommand::Clear {
                color: [0.0, 0.0, 0.0, 1.0],
            },
            jugar_render::RenderCommand::DrawSprite {
                texture_id: 0,
                position: Position::zero(),
                size: Vec2::new(64.0, 64.0),
                source: None,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            jugar_render::RenderCommand::DrawRect {
                rect: jugar_core::Rect::new(0.0, 0.0, 100.0, 100.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let frame = convert_render_queue(&commands);
        assert_eq!(frame.len(), 2); // Sprite is skipped
    }

    #[test]
    fn test_text_align_serialization() {
        assert_eq!(serde_json::to_string(&TextAlign::Left).unwrap(), "\"left\"");
        assert_eq!(
            serde_json::to_string(&TextAlign::Center).unwrap(),
            "\"center\""
        );
        assert_eq!(
            serde_json::to_string(&TextAlign::Right).unwrap(),
            "\"right\""
        );
    }

    #[test]
    fn test_text_baseline_serialization() {
        assert_eq!(
            serde_json::to_string(&TextBaseline::Top).unwrap(),
            "\"top\""
        );
        assert_eq!(
            serde_json::to_string(&TextBaseline::Middle).unwrap(),
            "\"middle\""
        );
        assert_eq!(
            serde_json::to_string(&TextBaseline::Bottom).unwrap(),
            "\"bottom\""
        );
        assert_eq!(
            serde_json::to_string(&TextBaseline::Alphabetic).unwrap(),
            "\"alphabetic\""
        );
    }
}
