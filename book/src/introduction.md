# Introduction

**Jugar** (Spanish: "to play") is a WASM-native universal game engine built on the [Batuta Sovereign AI Stack](https://github.com/paiml/batuta). It compiles to a single `.wasm` binary with **ABSOLUTE ZERO JavaScript dependencies**, making it ideal for secure, deterministic game development.

## Why Jugar?

- **Pure WASM**: Single `.wasm` binary with zero JavaScript
- **Mobile-First to Ultrawide**: Scales from phones to 49" 32:9 monitors
- **Batuta Ecosystem**: Built on trueno (SIMD/GPU compute) and aprender (ML/AI)
- **Extreme TDD**: 1500+ tests, 95%+ coverage, mutation testing
- **Toyota Way**: Quality-first design principles baked in

## Key Features

| Feature | Description |
|---------|-------------|
| **ECS Architecture** | High-performance Entity-Component-System |
| **Physics** | WebGPU → WASM-SIMD → Scalar tiered backends |
| **AI Systems** | GOAP, Behavior Trees, Steering Behaviors |
| **Responsive UI** | Anchor-based layouts for any screen |
| **Spatial Audio** | 2D positional audio with channel mixing |
| **Procgen** | Noise, dungeons, Wave Function Collapse |
| **Testing** | Probar: Rust-native WASM game testing |

## Quick Example

```rust
use jugar::prelude::*;

fn main() {
    let mut engine = JugarEngine::new(JugarConfig::default());

    engine.run(|ctx| {
        if ctx.input().key_pressed(KeyCode::Escape) {
            return LoopControl::Exit;
        }
        LoopControl::Continue
    });
}
```

## Design Philosophy: The Toyota Way

> "The right process will produce the right results." — Toyota Way

| Principle | Application in Jugar |
|-----------|----------------------|
| **Mieruka** (Visual Control) | Telemetry overlays in dev builds |
| **Poka-Yoke** (Error Proofing) | Type-safe APIs prevent bugs at compile time |
| **Jidoka** (Autonomation) | Fail-fast on invalid state |
| **Heijunka** (Leveling) | Fixed timestep for deterministic physics |
| **Genchi Genbutsu** (Go & See) | Examples as source of truth |

## Next Steps

- [Quick Start](./guide/quick-start.md) - Get up and running in 5 minutes
- [Architecture](./guide/architecture.md) - Understand the engine structure
- [Probar Testing](./probar/overview.md) - Learn about WASM-native testing
