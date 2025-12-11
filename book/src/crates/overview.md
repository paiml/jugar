# Crates Overview

Jugar is organized as a workspace with multiple specialized crates.

## Crate Structure

```
crates/
├── jugar/               # Main entry point
├── jugar-core/          # ECS, Game Loop, Components
├── jugar-physics/       # Rigid body simulation
├── jugar-ai/            # GOAP, Behavior Trees
├── jugar-render/        # Viewport, Anchors
├── jugar-ui/            # Widget system
├── jugar-input/         # Touch/Mouse/KB/Gamepad
├── jugar-audio/         # Spatial 2D audio
├── jugar-procgen/       # Noise, Dungeons, WFC
├── jugar-yaml/          # Declarative game definitions
├── jugar-probar/        # WASM-native testing
├── jugar-probar-derive/ # Proc-macro for type-safe selectors
├── jugar-web/           # Web platform bindings
└── physics-toy-sandbox/ # Physics demo crate
```

## Summary Table

| Crate | Description | Tests |
|-------|-------------|-------|
| `jugar` | Main entry point, JugarEngine | 17 |
| `jugar-core` | ECS, Game Loop, Components | 52 |
| `jugar-physics` | Rigid body simulation | 7 |
| `jugar-ai` | GOAP, Behavior Trees | 17 |
| `jugar-render` | Viewport, Anchors | 10 |
| `jugar-ui` | Widget system | 10 |
| `jugar-input` | Touch/Mouse/KB/Gamepad | 10 |
| `jugar-audio` | Spatial 2D audio | 21 |
| `jugar-procgen` | Noise, Dungeons, WFC | 18 |
| `jugar-yaml` | ELI5 YAML-First declarative games | 334 |
| `jugar-probar` | Rust-native WASM game testing | 128 |
| `jugar-web` | WASM Web platform | 95 |

**Total: 1500+ tests**

## Dependency Graph

```
jugar (entry)
├── jugar-core
│   └── hecs, glam
├── jugar-physics
│   └── trueno (SIMD/GPU)
├── jugar-ai
│   └── aprender (ML)
├── jugar-render
│   └── trueno-viz
├── jugar-ui
│   └── jugar-render
├── jugar-input
├── jugar-audio
└── jugar-procgen
```

## Feature Flags

Each crate supports optional features:

```toml
[dependencies]
jugar = { version = "0.1", features = ["full"] }
```

| Feature | Includes |
|---------|----------|
| `default` | Core engine |
| `ai` | jugar-ai |
| `audio` | jugar-audio |
| `procgen` | jugar-procgen |
| `yaml` | jugar-yaml |
| `full` | All features |
