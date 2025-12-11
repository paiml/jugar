# jugar-ai

AI systems: GOAP, Behavior Trees, and Steering Behaviors.

## GOAP (Goal-Oriented Action Planning)

```rust
use jugar_ai::goap::*;

// Define world state
let mut state = WorldState::new();
state.set("has_weapon", false);
state.set("enemy_dead", false);
state.set("at_armory", false);

// Define actions
let actions = vec![
    Action::new("go_to_armory")
        .precondition("at_armory", false)
        .effect("at_armory", true)
        .cost(5),

    Action::new("get_weapon")
        .precondition("at_armory", true)
        .precondition("has_weapon", false)
        .effect("has_weapon", true)
        .cost(2),

    Action::new("attack")
        .precondition("has_weapon", true)
        .effect("enemy_dead", true)
        .cost(1),
];

// Define goal
let goal = Goal::new()
    .require("enemy_dead", true);

// Plan
let planner = GoapPlanner::new();
let plan = planner.plan(&state, &goal, &actions);
// Returns: ["go_to_armory", "get_weapon", "attack"]
```

## Behavior Trees

```rust
use jugar_ai::behavior_tree::*;

let tree = BehaviorTree::new()
    .selector()                          // Try children until one succeeds
        .sequence()                       // All children must succeed
            .condition(|ctx| ctx.health < 20)
            .action(|ctx| ctx.flee())
        .end()
        .sequence()
            .condition(|ctx| ctx.sees_enemy())
            .action(|ctx| ctx.attack())
        .end()
        .action(|ctx| ctx.patrol())
    .end()
    .build();

// Tick the tree each frame
let status = tree.tick(&mut context);
match status {
    Status::Success => { /* completed */ }
    Status::Running => { /* still executing */ }
    Status::Failure => { /* failed */ }
}
```

## Node Types

| Node | Description |
|------|-------------|
| `Selector` | Try children until one succeeds |
| `Sequence` | Run children until one fails |
| `Parallel` | Run all children simultaneously |
| `Condition` | Check a predicate |
| `Action` | Execute behavior |
| `Decorator` | Modify child behavior |

## Steering Behaviors

```rust
use jugar_ai::steering::*;

let mut agent = SteeringAgent::new(position, max_speed);

// Individual behaviors
let seek = agent.seek(target);
let flee = agent.flee(danger);
let arrive = agent.arrive(destination, slow_radius);
let wander = agent.wander(circle_distance, circle_radius);

// Combine behaviors
let steering = SteeringCombiner::new()
    .add(seek, 1.0)
    .add(agent.separation(&neighbors), 2.0)
    .add(agent.cohesion(&neighbors), 0.5)
    .add(agent.alignment(&neighbors), 0.5)
    .calculate();

agent.apply_steering(steering, dt);
```

## Navigation Mesh

```rust
use jugar_ai::navmesh::*;

// Create navmesh from polygons
let navmesh = NavMesh::from_polygons(&walkable_areas);

// Find path
let path = navmesh.find_path(start, end);

// Smooth path
let smooth_path = navmesh.smooth_path(&path);
```
