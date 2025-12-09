//! # jugar-input
//!
//! Unified input handling for touch, mouse, and gamepad.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Input errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    /// Invalid gamepad index
    #[error("Gamepad {0} not connected")]
    GamepadNotConnected(u32),
}

/// Result type for input operations
pub type Result<T> = core::result::Result<T, InputError>;

/// Input device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputDevice {
    /// Touch screen
    Touch,
    /// Mouse
    Mouse,
    /// Keyboard
    Keyboard,
    /// Gamepad/Controller
    Gamepad(u32),
}

/// Touch/Mouse button state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ButtonState {
    /// Not pressed
    #[default]
    Released,
    /// Just pressed this frame
    JustPressed,
    /// Held down
    Pressed,
    /// Just released this frame
    JustReleased,
}

impl ButtonState {
    /// Returns true if the button is currently down
    #[must_use]
    pub const fn is_down(self) -> bool {
        matches!(self, Self::JustPressed | Self::Pressed)
    }

    /// Returns true if the button was just pressed
    #[must_use]
    pub const fn just_pressed(self) -> bool {
        matches!(self, Self::JustPressed)
    }

    /// Returns true if the button was just released
    #[must_use]
    pub const fn just_released(self) -> bool {
        matches!(self, Self::JustReleased)
    }

    /// Advances the state after a frame
    #[must_use]
    pub const fn advance(self) -> Self {
        match self {
            Self::JustPressed => Self::Pressed,
            Self::JustReleased => Self::Released,
            other => other,
        }
    }
}

/// Mouse button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button
    Middle,
    /// Extra button (index)
    Extra(u8),
}

/// Touch event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchEvent {
    /// Touch identifier
    pub id: u32,
    /// Position in screen coordinates
    pub position: Vec2,
    /// Change in position since last event
    pub delta: Vec2,
    /// Touch phase
    pub phase: TouchPhase,
    /// Pressure (0.0 to 1.0, if available)
    pub pressure: f32,
}

impl TouchEvent {
    /// Creates a new touch event
    #[must_use]
    pub const fn new(position: Vec2) -> Self {
        Self {
            id: 0,
            position,
            delta: Vec2::ZERO,
            phase: TouchPhase::Started,
            pressure: 1.0,
        }
    }

    /// Sets the touch ID
    #[must_use]
    pub const fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    /// Sets the phase
    #[must_use]
    pub const fn with_phase(mut self, phase: TouchPhase) -> Self {
        self.phase = phase;
        self
    }
}

/// Touch phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TouchPhase {
    /// Touch just started
    Started,
    /// Touch moved
    Moved,
    /// Touch ended normally
    Ended,
    /// Touch was cancelled
    Cancelled,
}

/// Keyboard key code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Arrow keys
    Up,
    /// Arrow down
    Down,
    /// Arrow left
    Left,
    /// Arrow right
    Right,
    /// Space bar
    Space,
    /// Enter/Return
    Enter,
    /// Escape
    Escape,
    /// Letter keys (A-Z)
    Letter(char),
    /// Number keys (0-9)
    Number(u8),
    /// Function keys (F1-F12)
    Function(u8),
}

/// Gamepad button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    /// A button (Xbox) / Cross (`PlayStation`)
    South,
    /// B button (Xbox) / Circle (`PlayStation`)
    East,
    /// X button (Xbox) / Square (`PlayStation`)
    West,
    /// Y button (Xbox) / Triangle (`PlayStation`)
    North,
    /// Left bumper/shoulder
    LeftBumper,
    /// Right bumper/shoulder
    RightBumper,
    /// Left stick press
    LeftStick,
    /// Right stick press
    RightStick,
    /// Start/Options
    Start,
    /// Select/Share
    Select,
    /// D-pad up
    DPadUp,
    /// D-pad down
    DPadDown,
    /// D-pad left
    DPadLeft,
    /// D-pad right
    DPadRight,
}

/// Gamepad axis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    /// Left stick X
    LeftStickX,
    /// Left stick Y
    LeftStickY,
    /// Right stick X
    RightStickX,
    /// Right stick Y
    RightStickY,
    /// Left trigger
    LeftTrigger,
    /// Right trigger
    RightTrigger,
}

/// Gamepad state
#[derive(Debug, Clone, Default)]
pub struct GamepadState {
    /// Connected status
    pub connected: bool,
    /// Button states
    pub buttons: [ButtonState; 14],
    /// Axis values (-1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers)
    pub axes: [f32; 6],
}

impl GamepadState {
    /// Gets a button state
    #[must_use]
    pub const fn button(&self, button: GamepadButton) -> ButtonState {
        let idx = button as usize;
        if idx < self.buttons.len() {
            self.buttons[idx]
        } else {
            ButtonState::Released
        }
    }

    /// Gets an axis value
    #[must_use]
    pub const fn axis(&self, axis: GamepadAxis) -> f32 {
        let idx = axis as usize;
        if idx < self.axes.len() {
            self.axes[idx]
        } else {
            0.0
        }
    }

    /// Gets left stick as a Vec2
    #[must_use]
    pub const fn left_stick(&self) -> Vec2 {
        Vec2::new(
            self.axis(GamepadAxis::LeftStickX),
            self.axis(GamepadAxis::LeftStickY),
        )
    }

    /// Gets right stick as a Vec2
    #[must_use]
    pub const fn right_stick(&self) -> Vec2 {
        Vec2::new(
            self.axis(GamepadAxis::RightStickX),
            self.axis(GamepadAxis::RightStickY),
        )
    }
}

/// Unified input state manager
#[derive(Debug, Default)]
pub struct InputState {
    /// Mouse position
    pub mouse_position: Vec2,
    /// Mouse delta
    pub mouse_delta: Vec2,
    /// Mouse button states
    pub mouse_buttons: [ButtonState; 5],
    /// Active touches
    pub touches: Vec<TouchEvent>,
    /// Keyboard key states
    keys: std::collections::HashMap<KeyCode, ButtonState>,
    /// Gamepad states (up to 4)
    pub gamepads: [GamepadState; 4],
}

impl InputState {
    /// Creates a new input state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets mouse button state
    #[must_use]
    pub const fn mouse_button(&self, button: MouseButton) -> ButtonState {
        let idx = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Extra(n) => 3 + n as usize,
        };
        if idx < self.mouse_buttons.len() {
            self.mouse_buttons[idx]
        } else {
            ButtonState::Released
        }
    }

    /// Gets key state
    #[must_use]
    pub fn key(&self, key: KeyCode) -> ButtonState {
        self.keys
            .get(&key)
            .copied()
            .unwrap_or(ButtonState::Released)
    }

    /// Sets key state
    pub fn set_key(&mut self, key: KeyCode, state: ButtonState) {
        let _ = self.keys.insert(key, state);
    }

    /// Gets primary touch (or mouse as touch)
    #[must_use]
    pub fn primary_pointer(&self) -> Option<Vec2> {
        self.touches
            .first()
            .map(|touch| touch.position)
            .or_else(|| {
                if self.mouse_button(MouseButton::Left).is_down() {
                    Some(self.mouse_position)
                } else {
                    None
                }
            })
    }

    /// Checks if there's any input this frame
    #[must_use]
    pub fn has_input(&self) -> bool {
        !self.touches.is_empty()
            || self.mouse_buttons.iter().any(|b| b.is_down())
            || self.keys.values().any(|b| b.is_down())
            || self.gamepads.iter().any(|g| g.connected)
    }

    /// Advances button states after a frame
    pub fn advance_frame(&mut self) {
        for button in &mut self.mouse_buttons {
            *button = button.advance();
        }
        for state in self.keys.values_mut() {
            *state = state.advance();
        }
        for gamepad in &mut self.gamepads {
            for button in &mut gamepad.buttons {
                *button = button.advance();
            }
        }
        self.mouse_delta = Vec2::ZERO;
    }

    /// Clears all touches
    pub fn clear_touches(&mut self) {
        self.touches.clear();
    }

    /// Checks if a key is currently pressed (down)
    #[must_use]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.key(key).is_down()
    }

    /// Sets a key as pressed or released
    pub fn set_key_pressed(&mut self, key: KeyCode, pressed: bool) {
        let state = if pressed {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        };
        self.set_key(key, state);
    }

    /// Clears transient input events (advances frame states, clears touches with ended/cancelled phase)
    pub fn clear_events(&mut self) {
        // Remove ended/cancelled touches
        self.touches
            .retain(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled));

        // Advance button states (JustPressed -> Pressed, JustReleased -> Released)
        self.advance_frame();
    }
}

/// Action binding for input abstraction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAction {
    /// Action name
    pub name: String,
    /// Bound keys
    pub keys: Vec<KeyCode>,
    /// Bound mouse buttons
    pub mouse_buttons: Vec<MouseButton>,
    /// Bound gamepad buttons
    pub gamepad_buttons: Vec<GamepadButton>,
}

impl InputAction {
    /// Creates a new action
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keys: Vec::new(),
            mouse_buttons: Vec::new(),
            gamepad_buttons: Vec::new(),
        }
    }

    /// Adds a key binding
    #[must_use]
    pub fn with_key(mut self, key: KeyCode) -> Self {
        self.keys.push(key);
        self
    }

    /// Adds a mouse button binding
    #[must_use]
    pub fn with_mouse_button(mut self, button: MouseButton) -> Self {
        self.mouse_buttons.push(button);
        self
    }

    /// Adds a gamepad button binding
    #[must_use]
    pub fn with_gamepad_button(mut self, button: GamepadButton) -> Self {
        self.gamepad_buttons.push(button);
        self
    }

    /// Checks if the action is active
    #[must_use]
    pub fn is_active(&self, input: &InputState) -> bool {
        self.keys.iter().any(|k| input.key(*k).is_down())
            || self
                .mouse_buttons
                .iter()
                .any(|b| input.mouse_button(*b).is_down())
            || self.gamepad_buttons.iter().any(|b| {
                input
                    .gamepads
                    .iter()
                    .any(|g| g.connected && g.button(*b).is_down())
            })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ==================== BUTTON STATE TESTS ====================

    #[test]
    fn test_button_state_advance() {
        assert_eq!(ButtonState::JustPressed.advance(), ButtonState::Pressed);
        assert_eq!(ButtonState::JustReleased.advance(), ButtonState::Released);
        assert_eq!(ButtonState::Pressed.advance(), ButtonState::Pressed);
        assert_eq!(ButtonState::Released.advance(), ButtonState::Released);
    }

    #[test]
    fn test_button_state_queries() {
        assert!(ButtonState::JustPressed.is_down());
        assert!(ButtonState::Pressed.is_down());
        assert!(!ButtonState::Released.is_down());
        assert!(!ButtonState::JustReleased.is_down());
        assert!(ButtonState::JustPressed.just_pressed());
        assert!(!ButtonState::Pressed.just_pressed());
        assert!(ButtonState::JustReleased.just_released());
        assert!(!ButtonState::Released.just_released());
    }

    #[test]
    fn test_button_state_default() {
        let state = ButtonState::default();
        assert_eq!(state, ButtonState::Released);
    }

    // ==================== TOUCH EVENT TESTS ====================

    #[test]
    fn test_touch_event() {
        let touch = TouchEvent::new(Vec2::new(100.0, 200.0))
            .with_id(1)
            .with_phase(TouchPhase::Moved);

        assert_eq!(touch.id, 1);
        assert_eq!(touch.phase, TouchPhase::Moved);
        assert!((touch.position.x - 100.0).abs() < f32::EPSILON);
        assert!((touch.position.y - 200.0).abs() < f32::EPSILON);
        assert!((touch.pressure - 1.0).abs() < f32::EPSILON);
        assert_eq!(touch.delta, Vec2::ZERO);
    }

    #[test]
    fn test_touch_event_defaults() {
        let touch = TouchEvent::new(Vec2::new(50.0, 50.0));
        assert_eq!(touch.id, 0);
        assert_eq!(touch.phase, TouchPhase::Started);
    }

    // ==================== INPUT STATE TESTS ====================

    #[test]
    fn test_input_state_new() {
        let state = InputState::new();
        assert_eq!(state.mouse_position, Vec2::ZERO);
        assert_eq!(state.mouse_delta, Vec2::ZERO);
        assert!(state.touches.is_empty());
    }

    #[test]
    fn test_input_state_mouse() {
        let mut state = InputState::new();
        state.mouse_position = Vec2::new(100.0, 200.0);
        state.mouse_buttons[0] = ButtonState::JustPressed;

        assert!(state.mouse_button(MouseButton::Left).is_down());
        assert!(!state.mouse_button(MouseButton::Right).is_down());
        assert!(!state.mouse_button(MouseButton::Middle).is_down());
    }

    #[test]
    fn test_input_state_mouse_extra_buttons() {
        let mut state = InputState::new();
        state.mouse_buttons[3] = ButtonState::Pressed;
        state.mouse_buttons[4] = ButtonState::Pressed;

        assert!(state.mouse_button(MouseButton::Extra(0)).is_down());
        assert!(state.mouse_button(MouseButton::Extra(1)).is_down());
        // Out of range should return Released
        assert!(!state.mouse_button(MouseButton::Extra(10)).is_down());
    }

    #[test]
    fn test_input_state_keyboard() {
        let mut state = InputState::new();
        state.set_key(KeyCode::Space, ButtonState::JustPressed);

        assert!(state.key(KeyCode::Space).just_pressed());
        assert!(!state.key(KeyCode::Enter).is_down());
    }

    #[test]
    fn test_input_state_advance() {
        let mut state = InputState::new();
        state.mouse_buttons[0] = ButtonState::JustPressed;
        state.set_key(KeyCode::Space, ButtonState::JustPressed);
        state.mouse_delta = Vec2::new(10.0, 20.0);

        state.advance_frame();

        assert_eq!(state.mouse_buttons[0], ButtonState::Pressed);
        assert_eq!(state.key(KeyCode::Space), ButtonState::Pressed);
        assert_eq!(state.mouse_delta, Vec2::ZERO);
    }

    #[test]
    fn test_input_state_advance_gamepad() {
        let mut state = InputState::new();
        state.gamepads[0].connected = true;
        state.gamepads[0].buttons[0] = ButtonState::JustPressed;

        state.advance_frame();

        assert_eq!(state.gamepads[0].buttons[0], ButtonState::Pressed);
    }

    #[test]
    fn test_input_state_clear_touches() {
        let mut state = InputState::new();
        state.touches.push(TouchEvent::new(Vec2::new(50.0, 50.0)));
        state.touches.push(TouchEvent::new(Vec2::new(100.0, 100.0)));

        state.clear_touches();

        assert!(state.touches.is_empty());
    }

    #[test]
    fn test_input_state_has_input() {
        let mut state = InputState::new();
        assert!(!state.has_input());

        state.mouse_buttons[0] = ButtonState::Pressed;
        assert!(state.has_input());

        state.mouse_buttons[0] = ButtonState::Released;
        state.touches.push(TouchEvent::new(Vec2::ZERO));
        assert!(state.has_input());
    }

    #[test]
    fn test_input_state_has_input_gamepad() {
        let mut state = InputState::new();
        assert!(!state.has_input());

        state.gamepads[0].connected = true;
        assert!(state.has_input());
    }

    #[test]
    fn test_input_state_has_input_keys() {
        let mut state = InputState::new();
        assert!(!state.has_input());

        state.set_key(KeyCode::Space, ButtonState::Pressed);
        assert!(state.has_input());
    }

    // ==================== PRIMARY POINTER TESTS ====================

    #[test]
    fn test_primary_pointer_touch() {
        let mut state = InputState::new();
        state.touches.push(TouchEvent::new(Vec2::new(50.0, 50.0)));

        let pointer = state.primary_pointer();
        assert_eq!(pointer, Some(Vec2::new(50.0, 50.0)));
    }

    #[test]
    fn test_primary_pointer_mouse() {
        let mut state = InputState::new();
        state.mouse_position = Vec2::new(100.0, 100.0);
        state.mouse_buttons[0] = ButtonState::Pressed;

        let pointer = state.primary_pointer();
        assert_eq!(pointer, Some(Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn test_primary_pointer_none() {
        let state = InputState::new();
        let pointer = state.primary_pointer();
        assert_eq!(pointer, None);
    }

    #[test]
    fn test_primary_pointer_touch_priority() {
        let mut state = InputState::new();
        state.mouse_position = Vec2::new(100.0, 100.0);
        state.mouse_buttons[0] = ButtonState::Pressed;
        state.touches.push(TouchEvent::new(Vec2::new(50.0, 50.0)));

        // Touch should take priority over mouse
        let pointer = state.primary_pointer();
        assert_eq!(pointer, Some(Vec2::new(50.0, 50.0)));
    }

    // ==================== GAMEPAD STATE TESTS ====================

    #[test]
    fn test_gamepad_state() {
        let mut gamepad = GamepadState::default();
        gamepad.axes[0] = 0.5; // Left stick X
        gamepad.axes[1] = -0.3; // Left stick Y

        let stick = gamepad.left_stick();
        assert!((stick.x - 0.5).abs() < f32::EPSILON);
        assert!((stick.y - (-0.3)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gamepad_right_stick() {
        let mut gamepad = GamepadState::default();
        gamepad.axes[2] = 0.7; // Right stick X
        gamepad.axes[3] = 0.8; // Right stick Y

        let stick = gamepad.right_stick();
        assert!((stick.x - 0.7).abs() < f32::EPSILON);
        assert!((stick.y - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gamepad_button() {
        let mut gamepad = GamepadState::default();
        gamepad.buttons[0] = ButtonState::Pressed;

        assert!(gamepad.button(GamepadButton::South).is_down());
        assert!(!gamepad.button(GamepadButton::North).is_down());
    }

    #[test]
    fn test_gamepad_axis() {
        let mut gamepad = GamepadState::default();
        gamepad.axes[4] = 0.5; // Left trigger
        gamepad.axes[5] = 0.8; // Right trigger

        assert!((gamepad.axis(GamepadAxis::LeftTrigger) - 0.5).abs() < f32::EPSILON);
        assert!((gamepad.axis(GamepadAxis::RightTrigger) - 0.8).abs() < f32::EPSILON);
    }

    // ==================== INPUT ACTION TESTS ====================

    #[test]
    fn test_input_action() {
        let action = InputAction::new("jump")
            .with_key(KeyCode::Space)
            .with_gamepad_button(GamepadButton::South);

        let mut state = InputState::new();
        assert!(!action.is_active(&state));

        state.set_key(KeyCode::Space, ButtonState::Pressed);
        assert!(action.is_active(&state));
    }

    #[test]
    fn test_input_action_with_mouse() {
        let action = InputAction::new("fire").with_mouse_button(MouseButton::Left);

        let mut state = InputState::new();
        assert!(!action.is_active(&state));

        state.mouse_buttons[0] = ButtonState::Pressed;
        assert!(action.is_active(&state));
    }

    #[test]
    fn test_input_action_with_gamepad() {
        let action = InputAction::new("jump").with_gamepad_button(GamepadButton::South);

        let mut state = InputState::new();
        assert!(!action.is_active(&state));

        state.gamepads[0].connected = true;
        state.gamepads[0].buttons[0] = ButtonState::Pressed;
        assert!(action.is_active(&state));
    }

    #[test]
    fn test_input_action_name() {
        let action = InputAction::new("test_action");
        assert_eq!(action.name, "test_action");
    }

    // ==================== INPUT DEVICE TESTS ====================

    #[test]
    fn test_input_device_variants() {
        // Test that all variants can be constructed
        assert_eq!(InputDevice::Touch, InputDevice::Touch);
        assert_eq!(InputDevice::Mouse, InputDevice::Mouse);
        assert_eq!(InputDevice::Keyboard, InputDevice::Keyboard);
        assert_eq!(InputDevice::Gamepad(0), InputDevice::Gamepad(0));
        assert_ne!(InputDevice::Gamepad(0), InputDevice::Gamepad(1));
    }

    // ==================== INPUT ERROR TESTS ====================

    #[test]
    fn test_input_error_display() {
        let err = InputError::GamepadNotConnected(2);
        let msg = format!("{err}");
        assert!(msg.contains("Gamepad"));
        assert!(msg.contains('2'));
        assert!(msg.contains("not connected"));
    }
}
