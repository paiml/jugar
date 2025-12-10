# AI/ML Reproducibility

This document describes how Jugar ensures reproducible AI behavior for testing and debugging.

## Random Seed Management

### Deterministic AI Behavior

The Pong AI uses deterministic algorithms that don't require random seeds:

1. **Prediction**: Ball trajectory is calculated using physics equations, not Monte Carlo methods
2. **Decision Making**: AI decisions are based on deterministic state evaluation
3. **Difficulty Adaptation**: Uses fixed formulas based on win/loss ratios

### Seeded Random for Procedural Content

When random behavior is needed (e.g., procedural generation), Jugar uses seeded PRNGs:

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// Fixed seed for reproducible behavior
const SEED: u64 = 42;
let mut rng = ChaCha8Rng::seed_from_u64(SEED);
```

### Configuration

Seeds can be configured via:

```rust
let config = WebConfig {
    seed: Some(12345),  // Optional fixed seed
    ..Default::default()
};
```

## Model Versioning

### .apr Format

AI models are versioned using the `.apr` (Aprender Profile) format:

```json
{
  "metadata": {
    "name": "Pong AI",
    "version": "1.0.0",
    "created": "2025-12-10T00:00:00Z"
  },
  "difficulty_profiles": [...],
  "shap_weights": [...]
}
```

### Version Compatibility

- Models include semantic version in metadata
- Loader validates version compatibility before use
- Breaking changes require major version bump

## Determinism Testing

### Replay System

Game states can be recorded and replayed:

```rust
let trace = game.tracer().export();
// Later...
game.replay_from_trace(&trace);
```

### Determinism Verification

The test suite includes determinism checks:

```rust
#[test]
fn test_ai_determinism() {
    let config = WebConfig::default();
    let results1 = run_simulation(&config, 1000);
    let results2 = run_simulation(&config, 1000);
    assert_eq!(results1, results2, "AI should be deterministic");
}
```

## Best Practices

1. **Testing**: Always use fixed seeds in tests
2. **Debugging**: Enable trace recording for issue reproduction
3. **Production**: Use time-based seeds for variety
4. **Benchmarking**: Use consistent seeds across runs
