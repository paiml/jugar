# API Documentation

## Online Documentation

The complete API documentation is available at:

**[https://docs.rs/jugar](https://docs.rs/jugar)**

## Generating Locally

```bash
cargo doc --open --no-deps
```

## Main Types

### JugarEngine

```rust
pub struct JugarEngine {
    // Fields are private
}

impl JugarEngine {
    pub fn new(config: JugarConfig) -> Self;
    pub fn from_definition(def: GameDefinition) -> Self;

    pub fn world(&self) -> &World;
    pub fn world_mut(&mut self) -> &mut World;

    pub fn run<F>(&mut self, game_loop: F)
    where
        F: FnMut(&mut GameContext) -> LoopControl;

    pub fn update(&mut self, dt: f32);
}
```

### JugarConfig

```rust
pub struct JugarConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fixed_dt: f32,
    pub debug_overlay: bool,
    pub vsync: bool,
}

impl Default for JugarConfig {
    fn default() -> Self {
        JugarConfig {
            width: 1920,
            height: 1080,
            title: "Jugar Game".to_string(),
            fixed_dt: 1.0 / 60.0,
            debug_overlay: cfg!(debug_assertions),
            vsync: true,
        }
    }
}
```

### World (ECS)

```rust
pub struct World { /* ... */ }

impl World {
    pub fn new() -> Self;

    // Entities
    pub fn spawn(&mut self) -> Entity;
    pub fn despawn(&mut self, entity: Entity);
    pub fn is_alive(&self, entity: Entity) -> bool;

    // Components
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T);
    pub fn remove_component<T: Component>(&mut self, entity: Entity);
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T>;

    // Queries
    pub fn query<Q: Query>(&self) -> QueryIter<Q>;

    // Resources
    pub fn add_resource<T: Resource>(&mut self, resource: T);
    pub fn get_resource<T: Resource>(&self) -> Option<&T>;
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T>;
}
```

### Components

```rust
// Position
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Velocity
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

// Transform
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}
```

### Input

```rust
pub struct InputState { /* ... */ }

impl InputState {
    // Keyboard
    pub fn key_pressed(&self, key: KeyCode) -> bool;
    pub fn key_held(&self, key: KeyCode) -> bool;
    pub fn key_released(&self, key: KeyCode) -> bool;

    // Mouse
    pub fn mouse_position(&self) -> Vec2;
    pub fn mouse_pressed(&self, button: MouseButton) -> bool;
    pub fn mouse_held(&self, button: MouseButton) -> bool;
    pub fn scroll_delta(&self) -> Vec2;

    // Touch
    pub fn touches(&self) -> &[Touch];

    // Gamepad
    pub fn gamepad(&self, id: usize) -> Option<&Gamepad>;
}
```

## Crate-Specific APIs

### jugar-physics

```rust
// PhysicsWorld
pub struct PhysicsWorld { /* ... */ }

impl PhysicsWorld {
    pub fn create_static_body(&mut self, pos: Position, collider: Collider) -> Body;
    pub fn create_dynamic_body(&mut self, pos: Position, collider: Collider, config: RigidBodyConfig) -> Body;
    pub fn apply_force(&mut self, body: Body, force: Vec2);
    pub fn apply_impulse(&mut self, body: Body, impulse: Vec2);
    pub fn raycast(&self, ray: Ray, max_distance: f32) -> Option<RayHit>;
}
```

### jugar-ai

```rust
// Behavior Trees
pub struct BehaviorTree { /* ... */ }

impl BehaviorTree {
    pub fn new() -> BehaviorTreeBuilder;
    pub fn tick(&self, context: &mut Context) -> Status;
}

// GOAP
pub struct GoapPlanner { /* ... */ }

impl GoapPlanner {
    pub fn plan(&self, state: &WorldState, goal: &Goal, actions: &[Action]) -> Option<Vec<Action>>;
}
```

### jugar-probar

```rust
// Assertions
pub struct Assertion {
    pub passed: bool,
    pub message: String,
}

impl Assertion {
    pub fn equals<T: PartialEq>(a: &T, b: &T) -> Assertion;
    pub fn in_range(value: f32, min: f32, max: f32) -> Assertion;
    pub fn is_true(condition: bool) -> Assertion;
    // ... more
}

// Simulation
pub fn run_simulation(config: SimulationConfig, input_fn: impl Fn(u32) -> Vec<InputEvent>) -> SimulationResult;
pub fn run_replay(recording: &Recording) -> ReplayResult;
```

## Prelude

The prelude re-exports common types:

```rust
pub use crate::{
    JugarEngine, JugarConfig, GameContext, LoopControl,
    World, Entity, Component,
    Position, Velocity, Transform,
    KeyCode, MouseButton, InputState,
    Vec2, Vec3, Mat4, Color,
};
```

Import with:

```rust
use jugar::prelude::*;
```
