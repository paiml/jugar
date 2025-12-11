# jugar-input

Unified input handling for touch, mouse, keyboard, and gamepad.

## Keyboard

```rust
use jugar_input::prelude::*;

let input = InputState::new();

// Key states
if input.key_pressed(KeyCode::Space) {
    // Just pressed this frame
}

if input.key_held(KeyCode::W) {
    // Currently held down
}

if input.key_released(KeyCode::Escape) {
    // Just released this frame
}
```

## Mouse

```rust
// Position
let pos = input.mouse_position();

// Buttons
if input.mouse_pressed(MouseButton::Left) {
    spawn_projectile(pos);
}

if input.mouse_held(MouseButton::Right) {
    aim_at(pos);
}

// Scroll wheel
let scroll = input.scroll_delta();
camera.zoom += scroll.y * 0.1;
```

## Touch

```rust
// All active touches
for touch in input.touches() {
    match touch.phase {
        TouchPhase::Started => {
            // New touch
        }
        TouchPhase::Moved => {
            // Touch moved
            let delta = touch.position - touch.previous_position;
        }
        TouchPhase::Ended | TouchPhase::Cancelled => {
            // Touch ended
        }
    }
}

// Gestures
if let Some(pinch) = input.pinch_gesture() {
    camera.zoom *= pinch.scale;
}

if let Some(pan) = input.pan_gesture() {
    camera.position -= pan.delta;
}
```

## Gamepad

```rust
// Check if gamepad connected
if let Some(gamepad) = input.gamepad(0) {
    // Buttons
    if gamepad.button_pressed(GamepadButton::South) {
        player.jump();
    }

    // Analog sticks
    let left_stick = gamepad.left_stick();
    player.velocity.x = left_stick.x * max_speed;

    let right_stick = gamepad.right_stick();
    player.aim_direction = right_stick.normalize();

    // Triggers
    let fire_intensity = gamepad.right_trigger();
    if fire_intensity > 0.5 {
        player.fire();
    }
}
```

## Virtual Joystick

For touch devices:

```rust
let mut joystick = VirtualJoystick::new()
    .position(100.0, 400.0)
    .radius(60.0)
    .dead_zone(0.1);

// Update with touch input
joystick.update(&input);

// Get direction
let direction = joystick.direction();
player.velocity = direction * max_speed;
```

## Input Abstraction

Unify different input methods:

```rust
// Abstract action
let move_input = input.get_axis_2d(
    // Keyboard
    KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D,
    // Gamepad
    GamepadAxis::LeftStickX, GamepadAxis::LeftStickY,
);

// Returns normalized Vec2 from any input source
player.velocity = move_input * max_speed;
```

## Key Codes

Common key codes:

| Category | Keys |
|----------|------|
| Letters | `A` - `Z` |
| Numbers | `Key0` - `Key9` |
| Arrows | `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight` |
| Special | `Space`, `Enter`, `Escape`, `Tab`, `Backspace` |
| Modifiers | `ShiftLeft`, `ControlLeft`, `AltLeft` |
| Function | `F1` - `F12` |
