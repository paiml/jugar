# jugar-physics

Physics simulation with tiered backend selection.

## Backend Tiers

| Tier | Backend | Capability |
|------|---------|------------|
| 1 | WebGPU compute shaders | 10,000+ rigid bodies |
| 2 | WASM SIMD 128-bit | 1,000+ rigid bodies |
| 3 | Scalar fallback | Basic physics |

Backend is selected automatically at runtime.

## Rigid Bodies

```rust
use jugar_physics::prelude::*;

let mut physics = PhysicsWorld::new();

// Create static body
let ground = physics.create_static_body(
    Position::new(400.0, 550.0),
    Collider::box_shape(800.0, 100.0),
);

// Create dynamic body
let ball = physics.create_dynamic_body(
    Position::new(400.0, 100.0),
    Collider::circle(20.0),
    RigidBodyConfig {
        mass: 1.0,
        restitution: 0.8,
        friction: 0.3,
    },
);
```

## Collision Detection

Spatial hashing for broad-phase:

```rust
// Check collisions
for contact in physics.get_contacts() {
    match (contact.body_a, contact.body_b) {
        (a, b) if a == player && is_coin(b) => {
            collect_coin(b);
        }
        _ => {}
    }
}
```

## Forces and Impulses

```rust
// Apply force (continuous)
physics.apply_force(body, Vec2::new(0.0, -100.0));

// Apply impulse (instant)
physics.apply_impulse(body, Vec2::new(500.0, 0.0));

// Set velocity directly
physics.set_velocity(body, Vec2::new(10.0, 0.0));
```

## Raycasting

```rust
let ray = Ray::new(
    Vec2::new(100.0, 100.0),  // origin
    Vec2::new(1.0, 0.0),       // direction
);

if let Some(hit) = physics.raycast(ray, 500.0) {
    println!("Hit body {:?} at {:?}", hit.body, hit.point);
}
```

## Configuration

```rust
let config = PhysicsConfig {
    gravity: Vec2::new(0.0, 980.0),  // pixels/s²
    iterations: 8,                     // solver iterations
    sleep_threshold: 0.1,              // velocity threshold
};

let mut physics = PhysicsWorld::with_config(config);
```
