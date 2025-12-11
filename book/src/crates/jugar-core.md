# jugar-core

Core engine functionality: ECS, game loop, components, and state management.

## ECS (Entity-Component-System)

Built on `hecs` for high-performance entity management.

### Entities

```rust
use jugar_core::prelude::*;

let mut world = World::new();

// Spawn entity
let entity = world.spawn();

// Check if alive
assert!(world.is_alive(entity));

// Despawn
world.despawn(entity);
```

### Components

```rust
// Add components
world.add_component(entity, Position::new(100.0, 200.0));
world.add_component(entity, Velocity::new(1.0, 0.0));
world.add_component(entity, Health(100));

// Get component
if let Some(pos) = world.get_component::<Position>(entity) {
    println!("Position: {:?}", pos);
}

// Remove component
world.remove_component::<Health>(entity);
```

### Queries

```rust
// Query all entities with Position and Velocity
for (entity, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
    pos.x += vel.x * dt;
    pos.y += vel.y * dt;
}

// Query with filter
for (entity, pos) in world.query::<&Position>()
    .filter::<&Player>()
{
    // Only entities with both Position and Player
}
```

## Built-in Components

| Component | Description |
|-----------|-------------|
| `Position` | 2D position (x, y) |
| `Velocity` | 2D velocity (x, y) |
| `Rotation` | Angle in radians |
| `Scale` | Uniform or non-uniform scale |
| `Transform` | Combined position/rotation/scale |
| `Sprite` | Sprite rendering info |
| `Collider` | Collision shape |
| `RigidBody` | Physics body |

## Resources

Global state accessible from the world:

```rust
// Add resource
world.add_resource(Score(0));

// Get resource
if let Some(score) = world.get_resource::<Score>() {
    println!("Score: {}", score.0);
}

// Mutate resource
if let Some(score) = world.get_resource_mut::<Score>() {
    score.0 += 10;
}
```

## Game Loop

Fixed timestep with variable rendering:

```rust
pub struct GameLoop {
    fixed_dt: f32,
    accumulator: f32,
}

impl GameLoop {
    pub fn update(&mut self, dt: f32, mut update_fn: impl FnMut(f32)) {
        self.accumulator += dt;

        while self.accumulator >= self.fixed_dt {
            update_fn(self.fixed_dt);
            self.accumulator -= self.fixed_dt;
        }
    }

    pub fn interpolation_alpha(&self) -> f32 {
        self.accumulator / self.fixed_dt
    }
}
```
