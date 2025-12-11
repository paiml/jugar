# Quick Start

Get up and running with Jugar in 5 minutes.

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- (Optional) `wasm-pack` for web builds

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (optional, for web builds)
cargo install wasm-pack
```

## Create a New Project

```bash
cargo new my-game
cd my-game
```

## Add Dependencies

Add Jugar to your `Cargo.toml`:

```toml
[dependencies]
jugar = "0.1"
```

## Write Your First Game

Replace `src/main.rs`:

```rust
use jugar::prelude::*;

fn main() {
    // Create engine with default 1920x1080 configuration
    let mut engine = JugarEngine::new(JugarConfig::default());

    // Spawn a player entity
    let player = engine.world_mut().spawn();
    engine.world_mut().add_component(player, Position::new(100.0, 100.0));
    engine.world_mut().add_component(player, Velocity::new(0.0, 0.0));

    // Run the game loop
    engine.run(|ctx| {
        // Handle input
        let input = ctx.input();
        let mut vel = Vec2::ZERO;

        if input.key_held(KeyCode::W) { vel.y -= 1.0; }
        if input.key_held(KeyCode::S) { vel.y += 1.0; }
        if input.key_held(KeyCode::A) { vel.x -= 1.0; }
        if input.key_held(KeyCode::D) { vel.x += 1.0; }

        // Update velocity component
        // ... game logic

        LoopControl::Continue
    });
}
```

## Build and Run

### Native Build

```bash
cargo run
```

### WASM Build

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --release

# Verify no JavaScript in output
ls target/wasm32-unknown-unknown/release/*.wasm
```

## Next Steps

- [Your First Game](./first-game.md) - Complete game tutorial
- [WASM Build](./wasm-build.md) - Deploy to the web
- [Architecture](./architecture.md) - Understanding Jugar's design
