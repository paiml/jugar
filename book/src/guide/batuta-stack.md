# Batuta Stack Integration

Jugar is built on the **Batuta Sovereign AI Stack** - a collection of pure Rust crates for AI and compute.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    BATUTA SOVEREIGN AI STACK                     │
│                   (USE THESE COMPONENTS FIRST)                   │
├─────────────────────────────────────────────────────────────────┤
│  trueno v0.7+     │  SIMD/GPU compute primitives (MANDATORY)    │
│  aprender v0.14+  │  ML algorithms, behavior trees (MANDATORY)  │
│  trueno-viz       │  WebGPU/WebGL2 rendering                    │
│  presentar-core   │  Platform abstraction, event loop           │
│  alimentar        │  Data loading with encryption               │
│  pacha            │  Asset registry with signatures             │
└─────────────────────────────────────────────────────────────────┘
```

## Mandatory Components

### trueno

SIMD and GPU compute primitives. Required for physics acceleration.

```toml
# Cargo.toml
[dependencies]
trueno = "0.7"

# Or for local development
trueno = { path = "../batuta/crates/trueno" }
```

Usage:
```rust
use trueno::prelude::*;

// SIMD-accelerated vector operations
let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
let b = Vec4::new(5.0, 6.0, 7.0, 8.0);
let c = a + b;  // Uses SIMD when available
```

### aprender

ML algorithms and behavior trees. Required for AI systems.

```toml
[dependencies]
aprender = "0.14"
```

Usage:
```rust
use aprender::prelude::*;

// Behavior tree
let tree = BehaviorTree::new()
    .selector()
        .sequence()
            .condition(|ctx| ctx.health < 20)
            .action(|ctx| ctx.flee())
        .end()
        .action(|ctx| ctx.patrol())
    .end()
    .build();
```

## Dependency Decision Tree

```
Need a capability?
    │
    ├─► Does batuta stack have it? ──► YES ──► USE IT (mandatory)
    │                                    │
    │                                    └─► Extend it if needed
    │
    └─► NO ──► Can we build it in pure Rust? ──► YES ──► Build it
                                                  │
                                                  └─► NO ──► REJECT
                                                        (find another way)
```

## Forbidden Crates

Never import these - they violate zero-JS or use non-Batuta components:

| Crate | Reason |
|-------|--------|
| `bevy` | Heavy, JS dependencies |
| `macroquad` | JavaScript glue required |
| `ggez` | Not pure WASM |
| `wasm-bindgen-futures` | JS promise dependency |
| `gloo` | JavaScript wrapper |

## Version Synchronization

Keep Batuta dependencies in sync:

```bash
# Check current versions
cargo tree | grep trueno
cargo tree | grep aprender

# Check latest versions
cargo search trueno
cargo search aprender

# Update
cargo update trueno aprender
```

## Local Development

For contributing to both Jugar and Batuta:

```toml
# Cargo.toml - Use local paths
[dependencies]
trueno = { path = "../batuta/crates/trueno" }
aprender = { path = "../batuta/crates/aprender" }
```

For releases, switch to crates.io:

```toml
# Cargo.toml - Use crates.io
[dependencies]
trueno = "0.7"
aprender = "0.14"
```

## Verification

```bash
# Verify batuta dependencies are used
make verify-batuta-deps

# Outputs:
# ✅ trueno dependency found
# ✅ aprender dependency found
# ✅ Using local batuta components (recommended for development)
```
