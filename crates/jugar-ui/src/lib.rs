//! # jugar-ui
//!
//! Responsive UI system with anchor-based layout for mobile to ultrawide displays.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use jugar_core::{Anchor, Rect, ScaleMode, UiElement};

/// UI system errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UiError {
    /// Widget not found
    #[error("Widget '{0}' not found")]
    WidgetNotFound(String),
}

/// Result type for UI operations
pub type Result<T> = core::result::Result<T, UiError>;

/// Widget identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub String);

impl WidgetId {
    /// Creates a new widget ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// UI container for managing widgets
#[derive(Debug, Default)]
pub struct UiContainer {
    widgets: Vec<(WidgetId, UiElement)>,
    viewport_size: Vec2,
}

impl UiContainer {
    /// Creates a new UI container
    #[must_use]
    pub const fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            widgets: Vec::new(),
            viewport_size: Vec2::new(viewport_width, viewport_height),
        }
    }

    /// Updates the viewport size
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_size = Vec2::new(width, height);
    }

    /// Adds a widget
    pub fn add_widget(&mut self, id: impl Into<String>, element: UiElement) -> WidgetId {
        let widget_id = WidgetId::new(id);
        self.widgets.push((widget_id.clone(), element));
        widget_id
    }

    /// Gets a widget by ID
    #[must_use]
    pub fn get_widget(&self, id: &WidgetId) -> Option<&UiElement> {
        self.widgets
            .iter()
            .find(|(wid, _)| wid == id)
            .map(|(_, elem)| elem)
    }

    /// Gets a mutable widget by ID
    pub fn get_widget_mut(&mut self, id: &WidgetId) -> Option<&mut UiElement> {
        self.widgets
            .iter_mut()
            .find(|(wid, _)| wid == id)
            .map(|(_, elem)| elem)
    }

    /// Removes a widget
    pub fn remove_widget(&mut self, id: &WidgetId) -> bool {
        if let Some(idx) = self.widgets.iter().position(|(wid, _)| wid == id) {
            let _ = self.widgets.remove(idx);
            true
        } else {
            false
        }
    }

    /// Returns the number of widgets
    #[must_use]
    pub fn widget_count(&self) -> usize {
        self.widgets.len()
    }

    /// Calculates the screen position of a widget
    #[must_use]
    pub fn calculate_widget_position(&self, element: &UiElement) -> Vec2 {
        let scale = match element.scale_mode {
            ScaleMode::Adaptive => self.viewport_size.y.min(self.viewport_size.x) / 1080.0,
            ScaleMode::PixelPerfect | ScaleMode::Fixed => 1.0,
        };

        let (ax, ay) = element.anchor.normalized();
        Vec2::new(
            self.viewport_size.x.mul_add(ax, element.offset.x * scale),
            self.viewport_size.y.mul_add(ay, element.offset.y * scale),
        )
    }

    /// Calculates the screen bounds of a widget
    #[must_use]
    pub fn calculate_widget_bounds(&self, element: &UiElement) -> Rect {
        let pos = self.calculate_widget_position(element);
        let scale = match element.scale_mode {
            ScaleMode::Adaptive => self.viewport_size.y.min(self.viewport_size.x) / 1080.0,
            ScaleMode::PixelPerfect | ScaleMode::Fixed => 1.0,
        };

        let scaled_size = element.size * scale;
        let (ax, ay) = element.anchor.normalized();

        Rect::new(
            scaled_size.x.mul_add(-ax, pos.x),
            scaled_size.y.mul_add(-ay, pos.y),
            scaled_size.x,
            scaled_size.y,
        )
    }

    /// Performs hit testing to find widget at a position
    #[must_use]
    pub fn hit_test(&self, position: Vec2) -> Option<&WidgetId> {
        // Check in reverse order (highest z-order first, assuming later = higher)
        for (id, element) in self.widgets.iter().rev() {
            if !element.visible {
                continue;
            }
            let bounds = self.calculate_widget_bounds(element);
            if bounds.contains_point(position.x, position.y) {
                return Some(id);
            }
        }
        None
    }

    /// Returns widgets sorted by z-order for rendering
    #[must_use]
    pub fn sorted_for_render(&self) -> Vec<(&WidgetId, &UiElement)> {
        let mut sorted: Vec<_> = self
            .widgets
            .iter()
            .filter(|(_, elem)| elem.visible)
            .map(|(id, elem)| (id, elem))
            .collect();
        sorted.sort_by_key(|(_, elem)| elem.z_order);
        sorted
    }
}

/// Button widget state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ButtonState {
    /// Normal state
    #[default]
    Normal,
    /// Mouse/touch hovering
    Hovered,
    /// Being pressed
    Pressed,
    /// Disabled
    Disabled,
}

/// Button widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    /// Visual element
    pub element: UiElement,
    /// Button text
    pub text: String,
    /// Current state
    pub state: ButtonState,
}

impl Button {
    /// Creates a new button
    #[must_use]
    pub fn new(text: impl Into<String>, size: Vec2) -> Self {
        Self {
            element: UiElement::new(size),
            text: text.into(),
            state: ButtonState::Normal,
        }
    }

    /// Sets the anchor
    #[must_use]
    pub const fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.element = self.element.with_anchor(anchor);
        self
    }

    /// Sets the offset
    #[must_use]
    pub const fn with_offset(mut self, offset: Vec2) -> Self {
        self.element = self.element.with_offset(offset);
        self
    }

    /// Checks if the button is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        !matches!(self.state, ButtonState::Disabled)
    }

    /// Checks if the button was just clicked
    #[must_use]
    pub const fn is_clicked(&self) -> bool {
        matches!(self.state, ButtonState::Pressed)
    }
}

/// Text label widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Visual element
    pub element: UiElement,
    /// Label text
    pub text: String,
    /// Text color (RGBA)
    pub color: [f32; 4],
    /// Font size
    pub font_size: f32,
}

impl Label {
    /// Creates a new label
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            element: UiElement::new(Vec2::new(200.0, 30.0)),
            text: text.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            font_size: 16.0,
        }
    }

    /// Sets the anchor
    #[must_use]
    pub const fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.element = self.element.with_anchor(anchor);
        self
    }

    /// Sets the color
    #[must_use]
    pub const fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Sets the font size
    #[must_use]
    pub const fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id() {
        let id = WidgetId::new("button1");
        assert_eq!(id.0, "button1");
    }

    #[test]
    fn test_ui_container_add_widget() {
        let mut container = UiContainer::new(1920.0, 1080.0);
        let id = container.add_widget("test", UiElement::default());
        assert_eq!(container.widget_count(), 1);
        assert!(container.get_widget(&id).is_some());
    }

    #[test]
    fn test_ui_container_remove_widget() {
        let mut container = UiContainer::new(1920.0, 1080.0);
        let id = container.add_widget("test", UiElement::default());
        assert!(container.remove_widget(&id));
        assert_eq!(container.widget_count(), 0);
    }

    #[test]
    fn test_widget_position_center() {
        let container = UiContainer::new(1920.0, 1080.0);
        let element = UiElement::new(Vec2::new(100.0, 50.0)).with_anchor(Anchor::Center);

        let pos = container.calculate_widget_position(&element);
        assert!((pos.x - 960.0).abs() < 1.0);
        assert!((pos.y - 540.0).abs() < 1.0);
    }

    #[test]
    fn test_widget_bounds() {
        let container = UiContainer::new(1920.0, 1080.0);
        let element = UiElement::new(Vec2::new(100.0, 50.0))
            .with_anchor(Anchor::TopLeft)
            .with_offset(Vec2::new(10.0, 10.0));

        let bounds = container.calculate_widget_bounds(&element);
        assert!((bounds.x - 10.0).abs() < 1.0);
        assert!((bounds.y - 10.0).abs() < 1.0);
        assert!((bounds.width - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_hit_test() {
        let mut container = UiContainer::new(1920.0, 1080.0);
        let id = container.add_widget(
            "button",
            UiElement::new(Vec2::new(100.0, 50.0))
                .with_anchor(Anchor::TopLeft)
                .with_offset(Vec2::new(10.0, 10.0)),
        );

        // Inside button
        let hit = container.hit_test(Vec2::new(50.0, 30.0));
        assert_eq!(hit, Some(&id));

        // Outside button
        let miss = container.hit_test(Vec2::new(500.0, 500.0));
        assert!(miss.is_none());
    }

    #[test]
    fn test_hit_test_respects_visibility() {
        let mut container = UiContainer::new(1920.0, 1080.0);
        let mut element = UiElement::new(Vec2::new(100.0, 50.0))
            .with_anchor(Anchor::TopLeft)
            .with_offset(Vec2::new(10.0, 10.0));
        element.visible = false;
        let _ = container.add_widget("button", element);

        // Should miss invisible widget
        let hit = container.hit_test(Vec2::new(50.0, 30.0));
        assert!(hit.is_none());
    }

    #[test]
    fn test_button_state() {
        let button = Button::new("Click Me", Vec2::new(100.0, 50.0));
        assert!(button.is_enabled());
        assert!(!button.is_clicked());
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new("Hello").with_font_size(24.0);
        assert_eq!(label.text, "Hello");
        assert!((label.font_size - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sorted_for_render() {
        let mut container = UiContainer::new(1920.0, 1080.0);
        let _ = container.add_widget("low", UiElement::default().with_z_order(0));
        let _ = container.add_widget("high", UiElement::default().with_z_order(10));
        let _ = container.add_widget("mid", UiElement::default().with_z_order(5));

        let sorted = container.sorted_for_render();
        assert_eq!(sorted[0].0 .0, "low");
        assert_eq!(sorted[1].0 .0, "mid");
        assert_eq!(sorted[2].0 .0, "high");
    }
}
