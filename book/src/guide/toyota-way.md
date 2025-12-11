# Toyota Way Principles

Jugar embodies the Toyota Production System (TPS) principles to ensure quality and reliability.

> "The right process will produce the right results." — Toyota Way

## Core Principles

### Mieruka (Visual Control)

**"Make problems visible"**

In Jugar:
- Telemetry overlays enabled by default in dev builds
- Visual debugging shows physics bodies, AI paths, collision shapes
- No hidden state - everything is inspectable

```rust
// Enable debug visualization
let config = JugarConfig {
    debug_overlay: true,
    show_physics_bodies: true,
    show_ai_paths: true,
    ..Default::default()
};
```

### Poka-Yoke (Error Proofing)

**"Mistake-proof the process"**

In Jugar:
- Rust's type system prevents null pointers and unhandled errors
- `Option<T>` and `Result<T, E>` enforce explicit error handling
- Type-safe entity selectors in Probar

```rust
// Compile-time error prevention
let entity: Entity = world.spawn();  // Cannot be null
let pos: Option<&Position> = world.get_component(entity);

// Must handle the Option
match pos {
    Some(p) => println!("Position: {:?}", p),
    None => println!("Entity has no position"),
}
```

### Jidoka (Autonomation)

**"Stop and fix problems immediately"**

In Jugar:
- Fail-fast on invalid state with `console_error_panic_hook`
- Never continue with corrupted state
- Tests fail immediately on assertion failure

```rust
// Fail-fast on invalid state
impl World {
    pub fn add_component<T>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            panic!("Cannot add component to dead entity");  // Jidoka
        }
        // ...
    }
}
```

### Heijunka (Leveling)

**"Level the workload"**

In Jugar:
- Fixed timestep logic (`fixed_dt`) ensures consistent physics
- Works identically on 30fps mobile and 144Hz monitors
- No frame-rate dependent behavior

```rust
const FIXED_DT: f32 = 1.0 / 60.0;  // 60 updates per second

// Physics runs at fixed rate regardless of frame rate
while accumulator >= FIXED_DT {
    physics_update(FIXED_DT);
    accumulator -= FIXED_DT;
}
```

### Genchi Genbutsu (Go & See)

**"Go to the source to understand"**

In Jugar:
- `examples/` directory is the source of truth
- Every feature has a working example
- Documentation follows code, not the other way around

```bash
# See how physics works
cargo run --example physics_demo

# See how AI works
cargo run --example ai_behavior_tree

# See Probar in action
cargo run --example pong_simulation -p jugar-probar
```

### Just-in-Time (JIT)

**"Produce only what is needed, when needed"**

In Jugar:
- Assets are streamed on-demand
- Render pipelines are compiled lazily
- Minimal startup time on mobile

```rust
// Lazy asset loading
let texture = assets.load_texture("player.png");  // Deferred
// Texture only loads when first used
```

### Kaizen (Continuous Improvement)

**"Always be improving"**

In Jugar:
- Hot-reloadable assets for rapid iteration
- State serialization for checkpoint/restore
- Makefile `kaizen` target for improvement analysis

```bash
# Run kaizen analysis
make kaizen

# Outputs:
# - Code metrics
# - Coverage analysis
# - Complexity analysis
# - Technical debt grading
# - Improvement recommendations
```

## Quality Metrics

| Metric | Target | Principle |
|--------|--------|-----------|
| Test Coverage | ≥95% | Poka-Yoke |
| Mutation Score | ≥80% | Jidoka |
| SATD Comments | 0 | Kaizen |
| Unsafe Code | 0 | Poka-Yoke |
| TDG Grade | A+ | Kaizen |

## Tiered Workflow

Inspired by Toyota's quality gates:

```bash
# Tier 1: ON-SAVE (sub-second)
make tier1  # Type check + fast tests

# Tier 2: ON-COMMIT (1-5 minutes)
make tier2  # Full validation

# Tier 3: ON-MERGE (hours)
make tier3  # Mutation testing + benchmarks
```
