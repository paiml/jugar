# Architecture

Jugar follows a layered architecture built on the Batuta Sovereign AI Stack.

## Layer Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    JUGAR WASM BUNDLE (Single .wasm file)                │
│                         NO JAVASCRIPT WHATSOEVER                        │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Game Loop  │  │  AI Agents  │  │   Render    │  │ Responsive  │     │
│  │  (ECS)      │  │  (GOAP/BT)  │  │  (Viewport) │  │  UI Layout  │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                │                │            │
│  ┌──────┴────────────────┴────────────────┴────────────────┴──────┐     │
│  │                         jugar (entry)                           │     │
│  ├─────────────────────────────────────────────────────────────────┤     │
│  │  jugar-core   │  jugar-physics  │  jugar-ai    │  jugar-render  │     │
│  │  jugar-input  │  jugar-audio    │  jugar-ui    │  jugar-procgen │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                  │                                       │
│                          BATUTA ECOSYSTEM                                │
│                    (trueno v0.7+ / aprender v0.14+)                     │
└─────────────────────────────────────────────────────────────────────────┘
```

## Crate Structure

| Crate | Description | Dependencies |
|-------|-------------|--------------|
| `jugar` | Main entry point, JugarEngine | All crates |
| `jugar-core` | ECS, Game Loop, State | hecs, glam |
| `jugar-physics` | Rigid body simulation | trueno |
| `jugar-ai` | GOAP, Behavior Trees | aprender |
| `jugar-render` | Viewport, Anchors | trueno-viz |
| `jugar-ui` | Widget system | jugar-render |
| `jugar-input` | Touch/Mouse/KB/Gamepad | - |
| `jugar-audio` | Spatial 2D audio | - |
| `jugar-procgen` | Noise, Dungeons, WFC | - |
| `jugar-yaml` | Declarative game definitions | - |
| `jugar-probar` | WASM-native testing | wasmtime |
| `jugar-web` | Web platform bindings | wasm-bindgen |

## Core Components

### JugarEngine

The main engine struct that orchestrates all systems:

```rust
pub struct JugarEngine {
    world: World,           // ECS world
    config: JugarConfig,    // Engine configuration
    time: TimeState,        // Fixed timestep tracking
    input: InputState,      // Input handling
}
```

### Game Loop

Jugar uses a fixed timestep loop (Heijunka principle):

```rust
// Internal loop structure
loop {
    // Accumulate time
    accumulator += delta_time;

    // Fixed timestep updates
    while accumulator >= FIXED_DT {
        physics_update(FIXED_DT);
        ai_update(FIXED_DT);
        accumulator -= FIXED_DT;
    }

    // Variable timestep render
    render(accumulator / FIXED_DT);
}
```

### ECS (Entity-Component-System)

Built on `hecs` for high-performance entity management:

```rust
// Spawn entity with components
let entity = world.spawn();
world.add_component(entity, Position::new(0.0, 0.0));
world.add_component(entity, Velocity::new(1.0, 0.0));

// Query entities
for (entity, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
    pos.x += vel.x * dt;
    pos.y += vel.y * dt;
}
```

## Physics Backend Selection

Runtime capability detection selects the optimal backend:

```
┌─────────────────────────────────────────────────────────┐
│  Backend Selection (Automatic)                          │
├─────────────────────────────────────────────────────────┤
│  1. Check WebGPU support                                │
│     ├─► Available: Use compute shaders (10K+ bodies)    │
│     └─► Not available: Continue                         │
│  2. Check WASM SIMD support                             │
│     ├─► Available: Use SIMD 128-bit (1K+ bodies)        │
│     └─► Not available: Continue                         │
│  3. Use scalar fallback (basic physics)                 │
└─────────────────────────────────────────────────────────┘
```

## Responsive Design

Jugar supports mobile-first to 32:9 ultrawide:

### Safe Area Calculation

```rust
// 16:9 gameplay area with peripheral extension
let safe_area = viewport.calculate_safe_area(AspectRatio::HD_16_9);
let extended = viewport.calculate_extended_area();  // For ultrawide

// UI anchors adapt to screen dimensions
let anchor = Anchor::BottomCenter { margin: 20.0 };
let pos = anchor.calculate(viewport.dimensions());
```

### Universal Scaling Model

- **Gameplay Layer**: 16:9 safe area with peripheral extension for ultrawide
- **UI Layer**: Anchor-based responsive layout scaling on shortest dimension
- **Input**: Touch/Click abstraction with virtual joysticks on touch devices only
