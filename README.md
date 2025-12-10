<p align="center">
  <img src="https://raw.githubusercontent.com/paiml/jugar/main/docs/assets/jugar-hero.svg" alt="Jugar Game Engine" width="800"/>
</p>

<h1 align="center">Jugar</h1>

<p align="center">
  <strong>WASM-Native Universal Game Engine</strong><br>
  <em>Mobile-first to 49" ultrawide - Pure WASM, Zero JavaScript</em>
</p>

<p align="center">
  <a href="https://github.com/paiml/jugar/actions/workflows/pmat-quality.yml">
    <img src="https://github.com/paiml/jugar/actions/workflows/pmat-quality.yml/badge.svg" alt="CI">
  </a>
  <a href="https://crates.io/crates/jugar">
    <img src="https://img.shields.io/crates/v/jugar.svg" alt="Crates.io">
  </a>
</p>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Examples](#examples)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**Jugar** (Spanish: "to play") is a WASM-native game engine built on the [Batuta Sovereign AI Stack](https://github.com/paiml/batuta). It compiles to a single `.wasm` binary with **ABSOLUTE ZERO JavaScript dependencies**, making it ideal for secure, deterministic game development.

```rust
use jugar::prelude::*;

fn main() {
    let mut engine = JugarEngine::new(JugarConfig::default());

    engine.run(|ctx| {
        // Your game logic here
        if ctx.input().key_pressed(KeyCode::Escape) {
            return LoopControl::Exit;
        }
        LoopControl::Continue
    });
}
```

## Features

### Core Engine
- **Pure WASM** - Compiles to `wasm32-unknown-unknown` with zero JavaScript
- **ECS Architecture** - High-performance Entity-Component-System using [hecs](https://crates.io/crates/hecs)
- **Fixed Timestep** - Deterministic physics via Heijunka (leveling) principle
- **Responsive Design** - Scales from mobile portrait to 32:9 ultrawide monitors

### Physics
- **Tiered Backends** - WebGPU → WASM-SIMD → Scalar automatic fallback
- **Rigid Bodies** - Static and dynamic body simulation
- **Collision Detection** - Spatial hashing for efficient broad-phase

### AI Systems
- **GOAP Planner** - Goal-Oriented Action Planning for emergent behavior
- **Behavior Trees** - Modular, composable AI decision making
- **Steering Behaviors** - Reynolds-style autonomous agent movement

### Rendering
- **Viewport Management** - Safe area calculation for all aspect ratios
- **Anchor System** - UI positioning that adapts to screen dimensions
- **Resolution Independence** - Pixel-perfect scaling across devices

### Audio
- **Spatial 2D Audio** - Distance-based attenuation and stereo panning
- **Channel Mixing** - Master, Music, Effects, Voice, Ambient channels
- **Listener System** - Player-relative audio positioning

### Procedural Generation
- **Value Noise** - Configurable octaves, persistence, lacunarity
- **Dungeon Generation** - BSP-based room placement with corridors
- **Wave Function Collapse** - Constraint-based procedural generation

## Quick Start

### Installation

Add Jugar to your `Cargo.toml`:

```toml
[dependencies]
jugar = "0.1"
```

### Minimal Example

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

### WASM Build

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --release

# Verify no JavaScript in output (PMAT compliance)
ls target/wasm32-unknown-unknown/release/*.wasm
```

## Architecture

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

### Crate Structure

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

## Examples

### Universal Pong

A responsive Pong implementation that works from mobile to 32:9 ultrawide:

```bash
# Run the Pong web demo
make build-web && make serve-web
# Open http://localhost:8080
```

Features:
- Touch controls (mobile)
- Keyboard (W/S, Up/Down)
- Gamepad support
- Responsive paddle positioning
- AI opponent with .apr ML model (Dynamic Difficulty Adjustment)
- SHAP-like explainability widgets

### Probar Testing Examples

```bash
# Deterministic simulation with replay verification
cargo run --example pong_simulation -p jugar-probar

# Playwright-style locator API demo
cargo run --example locator_demo -p jugar-probar

# WCAG accessibility checking
cargo run --example accessibility_demo -p jugar-probar
```

## Testing

### Probar: Rust-Native WASM Game Testing

Jugar uses **Probar** (Spanish: "to test/prove") - a pure Rust testing framework that replaces Playwright for E2E testing. Unlike Playwright, Probar can directly inspect game state without browser automation overhead.

```bash
# Run all E2E tests (39 tests, ~3 seconds)
make test-e2e

# Run with verbose output
make test-e2e-verbose

# Or directly via cargo
cargo test -p jugar-web --test probar_pong
```

### Probar vs Playwright

| Aspect | Playwright | Probar |
|--------|-----------|--------|
| **Language** | TypeScript | Pure Rust |
| **Browser** | Required (Chromium) | Not needed |
| **Game State** | Black box (DOM only) | Direct API access |
| **CI Setup** | Node.js + browser | Just `cargo test` |
| **Zero JS** | ❌ Violates constraint | ✅ Pure Rust |

### Test Suites

```bash
# Unit tests (fast, <2 min)
make test-fast

# All tests with coverage
make coverage

# Property-based tests (QuickCheck-style)
make test-property

# Chaos engineering tests
cargo test -p jugar-web --test chaos
```

## Quality Standards

Jugar follows **PMAT (Pragmatic Metrics for Agile Teams)** quality methodology:

| Metric | Target | Status |
|--------|--------|--------|
| Test Coverage | ≥95% | ✅ |
| Mutation Score | ≥80% | ✅ |
| TDG Grade | A+ | ✅ |
| SATD Comments | 0 | ✅ |
| Unsafe Code | 0 blocks | ✅ |
| JavaScript | 0 bytes | ✅ |

### Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Mieruka** (Visual Control) | Telemetry overlays in dev builds |
| **Poka-Yoke** (Error Proofing) | Type-safe APIs with `Result<T, E>` |
| **Jidoka** (Autonomation) | Fail-fast on invalid state |
| **Heijunka** (Leveling) | Fixed timestep game loop |
| **Genchi Genbutsu** (Go & See) | Examples as source of truth |

## Documentation

- **[API Documentation](https://docs.rs/jugar)** - Complete API reference
- **[Specification](docs/jugar-spec.md)** - Full engine specification
- **[Examples](examples/)** - Runnable example games

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/paiml/jugar.git
cd jugar

# Run tier 1 (sub-second feedback)
make tier1

# Run tier 2 (full validation)
make tier2

# Run tier 3 (mutation testing)
make tier3
```

### Quality Gates

All PRs must pass:
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-features`
- `pmat analyze tdg --min-grade B+`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Batuta Sovereign AI Stack](https://github.com/paiml/batuta) - Core compute and AI layers
- [hecs](https://crates.io/crates/hecs) - ECS implementation
- [glam](https://crates.io/crates/glam) - Math library

---

<p align="center">
  <sub>Built with the <a href="https://github.com/paiml/batuta">Batuta Sovereign AI Stack</a></sub>
</p>
