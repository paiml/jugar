# Physics Toy Sandbox

A remixable Rube Goldberg physics builder.

## Overview

The Physics Toy Sandbox is a creative physics playground where users can:
- Build contraptions from various materials
- Watch physics simulations run
- Remix and share designs
- Learn physics concepts through play

## Running

```bash
# Build
make sandbox

# Test
make test-sandbox

# Build WASM
make build-sandbox-wasm
```

## Concepts

### Materials

```rust
pub enum Material {
    Wood,
    Metal,
    Rubber,
    Glass,
    Stone,
}

impl Material {
    pub fn density(&self) -> f32 {
        match self {
            Material::Wood => 0.6,
            Material::Metal => 7.8,
            Material::Rubber => 1.1,
            Material::Glass => 2.5,
            Material::Stone => 2.4,
        }
    }

    pub fn restitution(&self) -> f32 {
        match self {
            Material::Wood => 0.3,
            Material::Metal => 0.2,
            Material::Rubber => 0.9,
            Material::Glass => 0.1,
            Material::Stone => 0.1,
        }
    }
}
```

### Contraptions

```rust
pub struct Contraption {
    pub parts: Vec<Part>,
    pub joints: Vec<Joint>,
    pub version: EngineVersion,
}

pub struct Part {
    pub shape: Shape,
    pub material: Material,
    pub position: Vec2,
    pub rotation: f32,
}

pub enum Joint {
    Hinge { a: PartId, b: PartId, anchor: Vec2 },
    Spring { a: PartId, b: PartId, stiffness: f32 },
    Rope { a: PartId, b: PartId, length: f32 },
}
```

### Complexity Thermometer

Visual control for contraption complexity (Mieruka principle):

```rust
pub struct ComplexityThermometer {
    part_count: u32,
    joint_count: u32,
    physics_budget: f32,
}

impl ComplexityThermometer {
    pub fn score(&self) -> f32 {
        let parts = self.part_count as f32 * 1.0;
        let joints = self.joint_count as f32 * 2.0;
        (parts + joints) / self.physics_budget
    }

    pub fn color(&self) -> Color {
        let score = self.score();
        if score < 0.5 {
            Color::GREEN
        } else if score < 0.8 {
            Color::YELLOW
        } else {
            Color::RED
        }
    }
}
```

## Remix System

Contraptions can be remixed and shared:

```rust
pub struct RemixMetadata {
    pub original_author: String,
    pub original_version: String,
    pub remix_chain: Vec<RemixInfo>,
}

impl Contraption {
    pub fn remix(&self, author: &str) -> Contraption {
        Contraption {
            parts: self.parts.clone(),
            joints: self.joints.clone(),
            version: EngineVersion::current(),
            metadata: RemixMetadata {
                remix_chain: {
                    let mut chain = self.metadata.remix_chain.clone();
                    chain.push(RemixInfo {
                        author: author.to_string(),
                        timestamp: now(),
                    });
                    chain
                },
                ..self.metadata.clone()
            },
        }
    }
}
```

## Engine Versioning

Replay compatibility across versions (Jidoka principle):

```rust
pub struct EngineVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl EngineVersion {
    pub fn is_compatible(&self, other: &EngineVersion) -> bool {
        self.major == other.major
    }

    pub fn current() -> Self {
        EngineVersion { major: 1, minor: 0, patch: 0 }
    }
}
```

## Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Poka-Yoke** | `NonZeroU32` for density (no division by zero) |
| **Jidoka** | Engine versioning for replay compatibility |
| **Mieruka** | ComplexityThermometer for visual control |
| **Muda** | No scalar fallback (WebGPU/WASM SIMD only) |

## Testing

```bash
# All tests
make test-sandbox

# With coverage
make test-sandbox-coverage

# Mutation testing
make sandbox-mutate
```

## Source Code

- `crates/physics-toy-sandbox/src/lib.rs` - Main library
- `crates/physics-toy-sandbox/src/material.rs` - Material system
- `crates/physics-toy-sandbox/src/contraption.rs` - Contraption building
- `crates/physics-toy-sandbox/src/remix.rs` - Remix system
