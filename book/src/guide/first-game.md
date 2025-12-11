# Your First Game

Let's build a simple game step by step to understand Jugar's core concepts.

## Game Concept

We'll create a simple "Collect the Coins" game:
- Player moves with WASD or arrow keys
- Coins spawn randomly
- Collect coins to increase score

## Step 1: Setup

```rust
use jugar::prelude::*;

// Define game components
#[derive(Clone, Copy)]
struct Player;

#[derive(Clone, Copy)]
struct Coin;

#[derive(Clone, Copy)]
struct Score(u32);

fn main() {
    let config = JugarConfig {
        width: 800,
        height: 600,
        title: "Coin Collector".to_string(),
        ..Default::default()
    };

    let mut engine = JugarEngine::new(config);

    // Initialize game state
    setup(&mut engine);

    engine.run(game_loop);
}
```

## Step 2: Setup Function

```rust
fn setup(engine: &mut JugarEngine) {
    let world = engine.world_mut();

    // Spawn player at center
    let player = world.spawn();
    world.add_component(player, Player);
    world.add_component(player, Position::new(400.0, 300.0));
    world.add_component(player, Velocity::new(0.0, 0.0));

    // Spawn initial coins
    for _ in 0..5 {
        spawn_coin(world);
    }

    // Add score resource
    world.add_resource(Score(0));
}

fn spawn_coin(world: &mut World) {
    let x = fastrand::f32() * 800.0;
    let y = fastrand::f32() * 600.0;

    let coin = world.spawn();
    world.add_component(coin, Coin);
    world.add_component(coin, Position::new(x, y));
}
```

## Step 3: Game Loop

```rust
fn game_loop(ctx: &mut GameContext) -> LoopControl {
    // Handle input
    handle_input(ctx);

    // Update physics
    update_movement(ctx);

    // Check collisions
    check_coin_collection(ctx);

    LoopControl::Continue
}

fn handle_input(ctx: &mut GameContext) {
    let input = ctx.input();
    let world = ctx.world_mut();

    let speed = 200.0; // pixels per second
    let mut vel = Vec2::ZERO;

    if input.key_held(KeyCode::W) || input.key_held(KeyCode::ArrowUp) {
        vel.y -= speed;
    }
    if input.key_held(KeyCode::S) || input.key_held(KeyCode::ArrowDown) {
        vel.y += speed;
    }
    if input.key_held(KeyCode::A) || input.key_held(KeyCode::ArrowLeft) {
        vel.x -= speed;
    }
    if input.key_held(KeyCode::D) || input.key_held(KeyCode::ArrowRight) {
        vel.x += speed;
    }

    // Update player velocity
    for (_, (_, velocity)) in world.query::<(&Player, &mut Velocity)>() {
        *velocity = Velocity(vel);
    }
}

fn update_movement(ctx: &mut GameContext) {
    let dt = ctx.delta_time();
    let world = ctx.world_mut();

    for (_, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }
}

fn check_coin_collection(ctx: &mut GameContext) {
    let world = ctx.world_mut();
    let collect_radius = 20.0;

    // Get player position
    let player_pos = world.query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(_, (_, pos))| *pos);

    let Some(player_pos) = player_pos else { return };

    // Find coins to collect
    let mut collected = Vec::new();
    for (entity, (_, coin_pos)) in world.query::<(&Coin, &Position)>() {
        let dx = player_pos.x - coin_pos.x;
        let dy = player_pos.y - coin_pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < collect_radius {
            collected.push(entity);
        }
    }

    // Remove collected coins and update score
    for entity in collected {
        world.despawn(entity);
        if let Some(score) = world.get_resource_mut::<Score>() {
            score.0 += 1;
        }
        // Spawn a new coin
        spawn_coin(world);
    }
}
```

## Step 4: Build and Run

```bash
# Native
cargo run

# WASM
cargo build --target wasm32-unknown-unknown --release
```

## Testing Your Game

Add tests using Probar:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use jugar_probar::Assertion;

    #[test]
    fn test_coin_collection_increases_score() {
        let mut engine = JugarEngine::new(JugarConfig::default());
        setup(&mut engine);

        let initial_score = engine.world().get_resource::<Score>().unwrap().0;

        // Simulate collecting a coin
        // ... test logic

        let assertion = Assertion::equals(&initial_score, &0);
        assert!(assertion.passed);
    }
}
```

## Next Steps

- [WASM Build](./wasm-build.md) - Deploy to the web
- [Architecture](./architecture.md) - Deep dive into engine design
- [Probar Testing](../probar/overview.md) - Comprehensive testing
