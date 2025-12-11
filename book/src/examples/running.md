# Running Examples

## Available Examples

### Probar Testing Examples

```bash
# Deterministic simulation with replay
cargo run --example pong_simulation -p jugar-probar

# Playwright-style locator API demo
cargo run --example locator_demo -p jugar-probar

# WCAG accessibility checking
cargo run --example accessibility_demo -p jugar-probar

# Coverage instrumentation demo
cargo run --example coverage_demo -p jugar-probar
```

### Web Examples

```bash
# Build and serve Pong
make build-web
make serve-web
# Open http://localhost:8080
```

### Physics Sandbox

```bash
# Build and test
make sandbox

# Build for WASM
make build-sandbox-wasm
```

## Example Output

### Pong Simulation

```
=== Probar Pong Simulation Demo ===

--- Demo 1: Pong Simulation ---
Initial state:
  Ball: (400.0, 300.0)
  Paddles: P1=300.0, P2=300.0
  Score: 0 - 0

Simulating 300 frames...

Final state after 300 frames:
  Ball: (234.5, 412.3)
  Paddles: P1=180.0, P2=398.2
  Score: 2 - 1
  State valid: true

--- Demo 2: Deterministic Replay ---
Recording simulation (seed=42, frames=500)...
  Completed: true
  Final hash: 6233835744931225727

Replaying simulation...
  Determinism verified: true
  Hashes match: true

--- Demo 3: Invariant Fuzzing ---
Running 100 fuzz iterations...
  All invariants held across all iterations
  Invariants checked:
    - ball_in_bounds: 100/100 passed
    - score_valid: 100/100 passed
    - paddle_in_bounds: 100/100 passed
```

### Locator Demo

```
=== Probar Locator Demo ===

--- Basic Locators ---
Locator::id("player") -> Found entity #42
Locator::tag("enemy") -> Found 5 entities
Locator::component::<Ball>() -> Found 1 entity

--- Compound Locators ---
Locator::tag("enemy").and(has_component::<Weapon>()) -> Found 2 entities
Locator::within_radius(player_pos, 100.0) -> Found 3 entities

--- Type-Safe Locators ---
@[derive(Entity)] Player -> Found at (400, 300)
@[derive(Entity)] Enemy -> Found 5 instances
```

### Accessibility Demo

```
=== Probar Accessibility Demo ===

--- Color Contrast ---
Text "Score: 0" - Foreground: #FFFFFF, Background: #1A1A2E
  Contrast ratio: 12.5:1 (WCAG AAA: PASS)

Button "Start" - Foreground: #000000, Background: #4CAF50
  Contrast ratio: 6.2:1 (WCAG AA: PASS)

--- Photosensitivity ---
Analyzed 180 frames (3 seconds at 60fps)
  Max flashes per second: 0.3
  Safe for photosensitive users: YES

--- Color Blindness ---
Protanopia simulation: All important elements distinguishable
Deuteranopia simulation: All important elements distinguishable

✅ Accessibility check passed!
```

## Creating Your Own Examples

### Project Structure

```
my-jugar-game/
├── Cargo.toml
├── src/
│   └── main.rs
└── examples/
    └── demo.rs
```

### Cargo.toml

```toml
[package]
name = "my-jugar-game"
version = "0.1.0"
edition = "2021"

[dependencies]
jugar = "0.1"

[dev-dependencies]
jugar-probar = "0.1"

[[example]]
name = "demo"
path = "examples/demo.rs"
```

### Example Code

```rust
// examples/demo.rs
use jugar::prelude::*;

fn main() {
    let config = JugarConfig::default();
    let mut engine = JugarEngine::new(config);

    println!("=== My Jugar Demo ===");

    // Demo your game features
    let player = engine.world_mut().spawn();
    engine.world_mut().add_component(player, Position::new(100.0, 100.0));

    println!("Spawned player at (100, 100)");

    // Run a few frames
    for i in 0..10 {
        engine.update(1.0 / 60.0);
        println!("Frame {}", i);
    }

    println!("Demo complete!");
}
```

### Run Your Example

```bash
cargo run --example demo
```
