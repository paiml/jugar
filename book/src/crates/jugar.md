# jugar

The main entry point crate that provides `JugarEngine` and the prelude.

## Usage

```toml
[dependencies]
jugar = "0.1"
```

## JugarEngine

The central engine struct:

```rust
use jugar::prelude::*;

fn main() {
    let config = JugarConfig {
        width: 1920,
        height: 1080,
        title: "My Game".to_string(),
        fixed_dt: 1.0 / 60.0,
        ..Default::default()
    };

    let mut engine = JugarEngine::new(config);

    engine.run(|ctx| {
        // Game logic here
        if ctx.input().key_pressed(KeyCode::Escape) {
            return LoopControl::Exit;
        }
        LoopControl::Continue
    });
}
```

## Configuration

```rust
pub struct JugarConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fixed_dt: f32,          // Physics timestep
    pub debug_overlay: bool,     // Show debug info
    pub vsync: bool,
}
```

## Prelude

The prelude re-exports common types:

```rust
use jugar::prelude::*;

// Includes:
// - JugarEngine, JugarConfig
// - Position, Velocity, Entity
// - KeyCode, MouseButton
// - Vec2, Vec3, Mat4
// - LoopControl
// - GameContext
```

## Game Context

Available during the game loop:

```rust
engine.run(|ctx: &mut GameContext| {
    // Time
    let dt = ctx.delta_time();
    let total = ctx.total_time();

    // Input
    let input = ctx.input();

    // World (ECS)
    let world = ctx.world();
    let world_mut = ctx.world_mut();

    LoopControl::Continue
});
```
