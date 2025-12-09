//! Browser input event translation
//!
//! Translates browser events (keyboard, mouse, touch) to Jugar's `InputState`.
//! All computation happens in Rust - JavaScript only forwards raw events.

use glam::Vec2;
use jugar_input::{
    ButtonState, GamepadAxis, GamepadButton, InputState, KeyCode, MouseButton, TouchEvent,
    TouchPhase,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Input translation errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum InputTranslationError {
    /// Failed to parse input JSON
    #[error("Failed to parse input JSON: {0}")]
    InvalidJson(String),
    /// Unknown event type
    #[error("Unknown event type: {0}")]
    UnknownEventType(String),
    /// Invalid event data
    #[error("Invalid event data: {0}")]
    InvalidData(String),
}

/// Browser input event from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInputEvent {
    /// Type of the event
    pub event_type: String,
    /// Timestamp in milliseconds (DOMHighResTimeStamp)
    pub timestamp: f64,
    /// Event-specific data
    pub data: BrowserEventData,
}

/// Event-specific data
///
/// Note: Variants are ordered from most specific (more fields) to least specific
/// because serde's `untagged` tries them in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BrowserEventData {
    /// Keyboard event data
    Key {
        /// JavaScript key code (e.g., "KeyW", "Space", "ArrowUp")
        key: String,
    },
    /// Gamepad axis event data (has 3 unique fields)
    GamepadAxis {
        /// Gamepad index
        gamepad: u8,
        /// Axis index
        axis: u8,
        /// Axis value (-1.0 to 1.0)
        value: f32,
    },
    /// Touch event data (has id, x, y - 3 fields)
    Touch {
        /// Touch identifier
        id: u32,
        /// X position in pixels
        x: f32,
        /// Y position in pixels
        y: f32,
    },
    /// Mouse button event data (has button, x, y - 3 fields)
    MouseButton {
        /// Button index (0=left, 1=middle, 2=right)
        button: u8,
        /// X position in pixels
        x: f32,
        /// Y position in pixels
        y: f32,
    },
    /// Gamepad button event data (has 2 fields)
    GamepadButton {
        /// Gamepad index
        gamepad: u8,
        /// Button index
        button: u8,
    },
    /// Mouse move event data (has x, y - 2 fields, least specific)
    MouseMove {
        /// X position in pixels
        x: f32,
        /// Y position in pixels
        y: f32,
    },
}

/// Translates a JavaScript key code to Jugar KeyCode
#[must_use]
pub fn translate_key(js_key: &str) -> Option<KeyCode> {
    match js_key {
        // Arrow keys
        "ArrowUp" => Some(KeyCode::Up),
        "ArrowDown" => Some(KeyCode::Down),
        "ArrowLeft" => Some(KeyCode::Left),
        "ArrowRight" => Some(KeyCode::Right),

        // Special keys
        "Space" => Some(KeyCode::Space),
        "Enter" => Some(KeyCode::Enter),
        "Escape" => Some(KeyCode::Escape),

        // Letter keys (KeyA through KeyZ)
        key if key.starts_with("Key") && key.len() == 4 => {
            let c = key.chars().nth(3)?;
            if c.is_ascii_uppercase() {
                Some(KeyCode::Letter(c))
            } else {
                None
            }
        }

        // Number keys (Digit0 through Digit9)
        key if key.starts_with("Digit") && key.len() == 6 => {
            let c = key.chars().nth(5)?;
            let n = c.to_digit(10)? as u8;
            Some(KeyCode::Number(n))
        }

        // Function keys (F1 through F12)
        key if key.starts_with('F') && key.len() <= 3 => {
            let n: u8 = key[1..].parse().ok()?;
            if (1..=12).contains(&n) {
                Some(KeyCode::Function(n))
            } else {
                None
            }
        }

        _ => None,
    }
}

/// Translates a JavaScript mouse button index to Jugar MouseButton
#[must_use]
pub const fn translate_mouse_button(button: u8) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        n => MouseButton::Extra(n.saturating_sub(3)),
    }
}

/// Translates a JavaScript gamepad button index to Jugar GamepadButton
#[must_use]
pub const fn translate_gamepad_button(button: u8) -> Option<GamepadButton> {
    match button {
        0 => Some(GamepadButton::South),
        1 => Some(GamepadButton::East),
        2 => Some(GamepadButton::West),
        3 => Some(GamepadButton::North),
        4 => Some(GamepadButton::LeftBumper),
        5 => Some(GamepadButton::RightBumper),
        8 => Some(GamepadButton::Select),
        9 => Some(GamepadButton::Start),
        10 => Some(GamepadButton::LeftStick),
        11 => Some(GamepadButton::RightStick),
        12 => Some(GamepadButton::DPadUp),
        13 => Some(GamepadButton::DPadDown),
        14 => Some(GamepadButton::DPadLeft),
        15 => Some(GamepadButton::DPadRight),
        _ => None,
    }
}

/// Translates a JavaScript gamepad axis index to Jugar GamepadAxis
#[must_use]
pub const fn translate_gamepad_axis(axis: u8) -> Option<GamepadAxis> {
    match axis {
        0 => Some(GamepadAxis::LeftStickX),
        1 => Some(GamepadAxis::LeftStickY),
        2 => Some(GamepadAxis::RightStickX),
        3 => Some(GamepadAxis::RightStickY),
        _ => None,
    }
}

/// Processes a batch of browser input events into InputState
///
/// # Errors
///
/// Returns an error if the JSON is malformed or contains invalid data.
pub fn process_input_events(
    events_json: &str,
    state: &mut InputState,
) -> Result<(), InputTranslationError> {
    if events_json.is_empty() || events_json == "[]" {
        return Ok(());
    }

    let events: Vec<BrowserInputEvent> = serde_json::from_str(events_json)
        .map_err(|e| InputTranslationError::InvalidJson(e.to_string()))?;

    for event in events {
        process_single_event(&event, state)?;
    }

    Ok(())
}

/// Processes a single browser input event
#[allow(clippy::too_many_lines)]
fn process_single_event(
    event: &BrowserInputEvent,
    state: &mut InputState,
) -> Result<(), InputTranslationError> {
    match event.event_type.as_str() {
        "KeyDown" => {
            if let BrowserEventData::Key { key } = &event.data {
                if let Some(key_code) = translate_key(key) {
                    let current = state.key(key_code);
                    if !current.is_down() {
                        state.set_key(key_code, ButtonState::JustPressed);
                    }
                }
            }
        }
        "KeyUp" => {
            if let BrowserEventData::Key { key } = &event.data {
                if let Some(key_code) = translate_key(key) {
                    state.set_key(key_code, ButtonState::JustReleased);
                }
            }
        }
        "MouseMove" => {
            if let BrowserEventData::MouseMove { x, y } = &event.data {
                let old_pos = state.mouse_position;
                state.mouse_position = Vec2::new(*x, *y);
                state.mouse_delta = state.mouse_position - old_pos;
            }
        }
        "MouseDown" => {
            if let BrowserEventData::MouseButton { button, x, y } = &event.data {
                state.mouse_position = Vec2::new(*x, *y);
                let idx = mouse_button_index(*button);
                if idx < state.mouse_buttons.len() {
                    state.mouse_buttons[idx] = ButtonState::JustPressed;
                }
            }
        }
        "MouseUp" => {
            if let BrowserEventData::MouseButton { button, x, y } = &event.data {
                state.mouse_position = Vec2::new(*x, *y);
                let idx = mouse_button_index(*button);
                if idx < state.mouse_buttons.len() {
                    state.mouse_buttons[idx] = ButtonState::JustReleased;
                }
            }
        }
        "TouchStart" => {
            if let BrowserEventData::Touch { id, x, y } = &event.data {
                state.touches.push(
                    TouchEvent::new(Vec2::new(*x, *y))
                        .with_id(*id)
                        .with_phase(TouchPhase::Started),
                );
            }
        }
        "TouchMove" => {
            if let BrowserEventData::Touch { id, x, y } = &event.data {
                // Update existing touch or add new one
                if let Some(touch) = state.touches.iter_mut().find(|t| t.id == *id) {
                    touch.delta = Vec2::new(*x, *y) - touch.position;
                    touch.position = Vec2::new(*x, *y);
                    touch.phase = TouchPhase::Moved;
                }
            }
        }
        "TouchEnd" => {
            if let BrowserEventData::Touch { id, .. } = &event.data {
                if let Some(touch) = state.touches.iter_mut().find(|t| t.id == *id) {
                    touch.phase = TouchPhase::Ended;
                }
            }
        }
        "TouchCancel" => {
            if let BrowserEventData::Touch { id, .. } = &event.data {
                if let Some(touch) = state.touches.iter_mut().find(|t| t.id == *id) {
                    touch.phase = TouchPhase::Cancelled;
                }
            }
        }
        "GamepadButtonDown" => {
            if let BrowserEventData::GamepadButton { gamepad, button } = &event.data {
                let gp_idx = *gamepad as usize;
                if gp_idx < state.gamepads.len() {
                    if let Some(btn) = translate_gamepad_button(*button) {
                        let btn_idx = btn as usize;
                        if btn_idx < state.gamepads[gp_idx].buttons.len() {
                            state.gamepads[gp_idx].buttons[btn_idx] = ButtonState::JustPressed;
                            state.gamepads[gp_idx].connected = true;
                        }
                    }
                }
            }
        }
        "GamepadButtonUp" => {
            if let BrowserEventData::GamepadButton { gamepad, button } = &event.data {
                let gp_idx = *gamepad as usize;
                if gp_idx < state.gamepads.len() {
                    if let Some(btn) = translate_gamepad_button(*button) {
                        let btn_idx = btn as usize;
                        if btn_idx < state.gamepads[gp_idx].buttons.len() {
                            state.gamepads[gp_idx].buttons[btn_idx] = ButtonState::JustReleased;
                        }
                    }
                }
            }
        }
        "GamepadAxisMove" => {
            if let BrowserEventData::GamepadAxis {
                gamepad,
                axis,
                value,
            } = &event.data
            {
                let gp_idx = *gamepad as usize;
                if gp_idx < state.gamepads.len() {
                    if let Some(ax) = translate_gamepad_axis(*axis) {
                        let ax_idx = ax as usize;
                        if ax_idx < state.gamepads[gp_idx].axes.len() {
                            state.gamepads[gp_idx].axes[ax_idx] = *value;
                            state.gamepads[gp_idx].connected = true;
                        }
                    }
                }
            }
        }
        "GamepadConnected" => {
            if let BrowserEventData::GamepadButton { gamepad, .. } = &event.data {
                let gp_idx = *gamepad as usize;
                if gp_idx < state.gamepads.len() {
                    state.gamepads[gp_idx].connected = true;
                }
            }
        }
        "GamepadDisconnected" => {
            if let BrowserEventData::GamepadButton { gamepad, .. } = &event.data {
                let gp_idx = *gamepad as usize;
                if gp_idx < state.gamepads.len() {
                    state.gamepads[gp_idx].connected = false;
                }
            }
        }
        unknown => {
            return Err(InputTranslationError::UnknownEventType(unknown.to_string()));
        }
    }

    Ok(())
}

/// Maps JS mouse button index to internal array index
const fn mouse_button_index(button: u8) -> usize {
    match button {
        0 => 0, // Left
        1 => 2, // Middle (JS uses 1, we use 2)
        2 => 1, // Right (JS uses 2, we use 1)
        n => (n as usize).saturating_sub(3).saturating_add(3),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ==================== KEY TRANSLATION TESTS ====================

    #[test]
    fn test_translate_arrow_keys() {
        assert_eq!(translate_key("ArrowUp"), Some(KeyCode::Up));
        assert_eq!(translate_key("ArrowDown"), Some(KeyCode::Down));
        assert_eq!(translate_key("ArrowLeft"), Some(KeyCode::Left));
        assert_eq!(translate_key("ArrowRight"), Some(KeyCode::Right));
    }

    #[test]
    fn test_translate_special_keys() {
        assert_eq!(translate_key("Space"), Some(KeyCode::Space));
        assert_eq!(translate_key("Enter"), Some(KeyCode::Enter));
        assert_eq!(translate_key("Escape"), Some(KeyCode::Escape));
    }

    #[test]
    fn test_translate_letter_keys() {
        assert_eq!(translate_key("KeyA"), Some(KeyCode::Letter('A')));
        assert_eq!(translate_key("KeyW"), Some(KeyCode::Letter('W')));
        assert_eq!(translate_key("KeyS"), Some(KeyCode::Letter('S')));
        assert_eq!(translate_key("KeyD"), Some(KeyCode::Letter('D')));
        assert_eq!(translate_key("KeyZ"), Some(KeyCode::Letter('Z')));
    }

    #[test]
    fn test_translate_number_keys() {
        assert_eq!(translate_key("Digit0"), Some(KeyCode::Number(0)));
        assert_eq!(translate_key("Digit1"), Some(KeyCode::Number(1)));
        assert_eq!(translate_key("Digit9"), Some(KeyCode::Number(9)));
    }

    #[test]
    fn test_translate_function_keys() {
        assert_eq!(translate_key("F1"), Some(KeyCode::Function(1)));
        assert_eq!(translate_key("F5"), Some(KeyCode::Function(5)));
        assert_eq!(translate_key("F12"), Some(KeyCode::Function(12)));
    }

    #[test]
    fn test_translate_unknown_key() {
        assert_eq!(translate_key("Unknown"), None);
        assert_eq!(translate_key(""), None);
        assert_eq!(translate_key("Key"), None); // Too short
        assert_eq!(translate_key("KeyAB"), None); // Too long
        assert_eq!(translate_key("F13"), None); // Out of range
        assert_eq!(translate_key("F0"), None); // Out of range
    }

    // ==================== MOUSE BUTTON TESTS ====================

    #[test]
    fn test_translate_mouse_buttons() {
        assert_eq!(translate_mouse_button(0), MouseButton::Left);
        assert_eq!(translate_mouse_button(1), MouseButton::Middle);
        assert_eq!(translate_mouse_button(2), MouseButton::Right);
        assert_eq!(translate_mouse_button(3), MouseButton::Extra(0));
        assert_eq!(translate_mouse_button(4), MouseButton::Extra(1));
    }

    // ==================== GAMEPAD BUTTON TESTS ====================

    #[test]
    fn test_translate_gamepad_buttons() {
        assert_eq!(translate_gamepad_button(0), Some(GamepadButton::South));
        assert_eq!(translate_gamepad_button(1), Some(GamepadButton::East));
        assert_eq!(translate_gamepad_button(2), Some(GamepadButton::West));
        assert_eq!(translate_gamepad_button(3), Some(GamepadButton::North));
        assert_eq!(translate_gamepad_button(4), Some(GamepadButton::LeftBumper));
        assert_eq!(
            translate_gamepad_button(5),
            Some(GamepadButton::RightBumper)
        );
        assert_eq!(translate_gamepad_button(8), Some(GamepadButton::Select));
        assert_eq!(translate_gamepad_button(9), Some(GamepadButton::Start));
        assert_eq!(translate_gamepad_button(12), Some(GamepadButton::DPadUp));
        assert_eq!(translate_gamepad_button(13), Some(GamepadButton::DPadDown));
        assert_eq!(translate_gamepad_button(14), Some(GamepadButton::DPadLeft));
        assert_eq!(translate_gamepad_button(15), Some(GamepadButton::DPadRight));
    }

    #[test]
    fn test_translate_gamepad_button_invalid() {
        assert_eq!(translate_gamepad_button(6), None);
        assert_eq!(translate_gamepad_button(7), None);
        assert_eq!(translate_gamepad_button(16), None);
        assert_eq!(translate_gamepad_button(255), None);
    }

    // ==================== GAMEPAD AXIS TESTS ====================

    #[test]
    fn test_translate_gamepad_axes() {
        assert_eq!(translate_gamepad_axis(0), Some(GamepadAxis::LeftStickX));
        assert_eq!(translate_gamepad_axis(1), Some(GamepadAxis::LeftStickY));
        assert_eq!(translate_gamepad_axis(2), Some(GamepadAxis::RightStickX));
        assert_eq!(translate_gamepad_axis(3), Some(GamepadAxis::RightStickY));
    }

    #[test]
    fn test_translate_gamepad_axis_invalid() {
        assert_eq!(translate_gamepad_axis(4), None);
        assert_eq!(translate_gamepad_axis(255), None);
    }

    // ==================== EVENT PROCESSING TESTS ====================

    #[test]
    fn test_process_empty_events() {
        let mut state = InputState::new();
        assert!(process_input_events("", &mut state).is_ok());
        assert!(process_input_events("[]", &mut state).is_ok());
    }

    #[test]
    fn test_process_key_down() {
        let mut state = InputState::new();
        let events = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"Space"}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.key(KeyCode::Space).just_pressed());
    }

    #[test]
    fn test_process_key_up() {
        let mut state = InputState::new();
        state.set_key(KeyCode::Space, ButtonState::Pressed);

        let events = r#"[{"event_type":"KeyUp","timestamp":0,"data":{"key":"Space"}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.key(KeyCode::Space).just_released());
    }

    #[test]
    fn test_process_mouse_move() {
        let mut state = InputState::new();
        let events = r#"[{"event_type":"MouseMove","timestamp":0,"data":{"x":100.0,"y":200.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!((state.mouse_position.x - 100.0).abs() < f32::EPSILON);
        assert!((state.mouse_position.y - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_mouse_down() {
        let mut state = InputState::new();
        let events =
            r#"[{"event_type":"MouseDown","timestamp":0,"data":{"button":0,"x":50.0,"y":60.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.mouse_button(MouseButton::Left).just_pressed());
        assert!((state.mouse_position.x - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_mouse_up() {
        let mut state = InputState::new();
        state.mouse_buttons[0] = ButtonState::Pressed;

        let events =
            r#"[{"event_type":"MouseUp","timestamp":0,"data":{"button":0,"x":50.0,"y":60.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.mouse_button(MouseButton::Left).just_released());
    }

    #[test]
    fn test_process_touch_start() {
        let mut state = InputState::new();
        let events =
            r#"[{"event_type":"TouchStart","timestamp":0,"data":{"id":1,"x":100.0,"y":200.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert_eq!(state.touches.len(), 1);
        // Note: ID is set via with_id() builder
        assert_eq!(state.touches[0].phase, TouchPhase::Started);
    }

    #[test]
    fn test_process_touch_move() {
        let mut state = InputState::new();
        // Create touch with same ID that will be moved
        state
            .touches
            .push(TouchEvent::new(Vec2::new(100.0, 200.0)).with_id(1));

        let events =
            r#"[{"event_type":"TouchMove","timestamp":0,"data":{"id":1,"x":150.0,"y":250.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        // TouchMove should update existing touch phase
        assert_eq!(state.touches[0].phase, TouchPhase::Moved);
        assert!((state.touches[0].position.x - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_touch_end() {
        let mut state = InputState::new();
        state
            .touches
            .push(TouchEvent::new(Vec2::new(100.0, 200.0)).with_id(1));

        let events = r#"[{"event_type":"TouchEnd","timestamp":0,"data":{"id":1,"x":0.0,"y":0.0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert_eq!(state.touches[0].phase, TouchPhase::Ended);
    }

    #[test]
    fn test_process_gamepad_button() {
        let mut state = InputState::new();
        let events =
            r#"[{"event_type":"GamepadButtonDown","timestamp":0,"data":{"gamepad":0,"button":0}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.gamepads[0].connected);
        assert!(state.gamepads[0]
            .button(GamepadButton::South)
            .just_pressed());
    }

    #[test]
    fn test_process_gamepad_axis() {
        let mut state = InputState::new();
        let events = r#"[{"event_type":"GamepadAxisMove","timestamp":0,"data":{"gamepad":0,"axis":0,"value":0.75}}]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.gamepads[0].connected);
        assert!((state.gamepads[0].axis(GamepadAxis::LeftStickX) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_multiple_events() {
        let mut state = InputState::new();
        let events = r#"[
            {"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}},
            {"event_type":"KeyDown","timestamp":1,"data":{"key":"Space"}},
            {"event_type":"MouseMove","timestamp":2,"data":{"x":400.0,"y":300.0}}
        ]"#;

        assert!(process_input_events(events, &mut state).is_ok());
        assert!(state.key(KeyCode::Letter('W')).just_pressed());
        assert!(state.key(KeyCode::Space).just_pressed());
        assert!((state.mouse_position.x - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_invalid_json() {
        let mut state = InputState::new();
        let result = process_input_events("not json", &mut state);
        assert!(matches!(result, Err(InputTranslationError::InvalidJson(_))));
    }

    #[test]
    fn test_process_unknown_event_type() {
        let mut state = InputState::new();
        let events = r#"[{"event_type":"Unknown","timestamp":0,"data":{"key":"Space"}}]"#;
        let result = process_input_events(events, &mut state);
        assert!(matches!(
            result,
            Err(InputTranslationError::UnknownEventType(_))
        ));
    }

    // ==================== ERROR DISPLAY TESTS ====================

    #[test]
    fn test_error_display() {
        let err = InputTranslationError::InvalidJson("test".to_string());
        assert!(format!("{err}").contains("test"));

        let err = InputTranslationError::UnknownEventType("foo".to_string());
        assert!(format!("{err}").contains("foo"));

        let err = InputTranslationError::InvalidData("bar".to_string());
        assert!(format!("{err}").contains("bar"));
    }

    // ==================== MOUSE BUTTON INDEX TESTS ====================

    #[test]
    fn test_mouse_button_index() {
        assert_eq!(mouse_button_index(0), 0); // Left
        assert_eq!(mouse_button_index(1), 2); // Middle (swapped)
        assert_eq!(mouse_button_index(2), 1); // Right (swapped)
        assert_eq!(mouse_button_index(3), 3); // Extra
    }
}
