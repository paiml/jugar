# Pong AI Design Specification

## Purpose

This document specifies the AI opponent design for the Jugar Pong demo, with the primary goal of **showcasing the aprender `.apr` model format** as a portable, trainable, reproducible AI artifact.

## Five Whys Root Cause Analysis

| Level | Question | Answer |
|-------|----------|--------|
| 1 | Why build Pong demo? | Showcase Jugar WASM engine |
| 2 | Why does Jugar exist? | Demonstrate Batuta Sovereign AI Stack |
| 3 | Why demonstrate aprender? | `.apr` model format is core innovation |
| 4 | Why is `.apr` important? | Portable, trainable, reproducible AI |
| **5** | **ROOT CAUSE** | **Demo must showcase `.apr` as transparent, scientific AI artifact** |

## Research Foundation

### Peer-Reviewed Sources

1. **Dynamic Difficulty Adjustment (DDA)**
   - [MDPI Virtual Worlds (2024)](https://www.mdpi.com/2813-2084/3/2/12): Hybrid DDA approaches promising
   - [arXiv Personalized DDA (2024)](https://arxiv.org/html/2408.06818v1): IL + RL combination
   - [Wiley DDA Review (2018)](https://onlinelibrary.wiley.com/doi/10.1155/2018/5681652): Comprehensive taxonomy

2. **Flow Theory (Csikszentmihalyi)**
   - [Think Game Design](https://thinkgamedesign.com/flow-theory-game-design/): Three-channel model
   - [Game Developer](https://www.gamedeveloper.com/design/the-flow-applied-to-game-design): Microflow + Macroflow

3. **Reproducibility in RL**
   - [Karpathy (2016)](http://karpathy.github.io/2016/05/31/rl/): Policy gradient, deterministic seeds
   - [arXiv Reproducibility (2022)](https://arxiv.org/abs/2203.01075): Minimal traces for verification

## Design Principles

### 1. Transparency (`.apr` Showcase)

The AI model MUST be:
- **JSON-serializable**: Human-readable model parameters
- **Downloadable**: Users can save the `.apr` file
- **Inspectable**: Clear parameter meanings documented
- **Modifiable**: Users can edit and reload models

### 2. Reproducibility (Scientific Rigor)

The AI MUST be:
- **Deterministic**: Same inputs → same outputs (given seed)
- **Seeded**: RNG state stored in model
- **Traceable**: Action sequences can be replayed
- **Verifiable**: Unit tests prove determinism

### 3. Flow Theory (Player Engagement)

The AI MUST implement:
- **Three-Channel Model**: Detect boredom ↔ flow ↔ anxiety
- **Dynamic Difficulty Adjustment**: Adapt to player skill
- **Skill Estimation**: Track player performance metrics
- **Gradual Adaptation**: Smooth difficulty transitions

## `.apr` Model Schema

```json
{
  "$schema": "https://paiml.com/schemas/apr/v1",
  "metadata": {
    "name": "Pong AI v1",
    "version": "1.0.0",
    "description": "Flow Theory-based adaptive Pong opponent",
    "author": "PAIML",
    "license": "MIT",
    "created": "2025-01-01T00:00:00Z"
  },
  "model_type": "behavior_tree",
  "determinism": {
    "seed": 12345,
    "rng_algorithm": "xorshift64"
  },
  "flow_theory": {
    "skill_window_size": 10,
    "adaptation_rate": 0.1,
    "boredom_threshold": 0.7,
    "anxiety_threshold": 0.3,
    "target_win_rate": 0.5
  },
  "difficulty_profiles": [
    {
      "level": 0,
      "name": "Novice",
      "reaction_delay_ms": 500,
      "prediction_accuracy": 0.30,
      "max_paddle_speed": 200,
      "error_magnitude": 50,
      "aggression": 0.1
    },
    // ... levels 1-8 ...
    {
      "level": 9,
      "name": "Expert",
      "reaction_delay_ms": 50,
      "prediction_accuracy": 0.95,
      "max_paddle_speed": 600,
      "error_magnitude": 5,
      "aggression": 0.9
    }
  ]
}
```

## Flow Theory Implementation

### Three-Channel Model

```
        High Challenge
             │
    ┌────────┼────────┐
    │   ANXIETY       │
    │  (frustrated)   │
    │                 │
Low ├───── FLOW ──────┤ High
Skill│  (engaged)     │ Skill
    │                 │
    │   BOREDOM       │
    │  (disengaged)   │
    └────────┼────────┘
             │
        Low Challenge
```

### Skill Estimation Algorithm

```rust
fn estimate_skill(&self) -> f32 {
    // Weighted combination of metrics
    let hit_rate = hits / (hits + misses);
    let rally_factor = avg_rally_length / MAX_USEFUL_RALLY;
    let reaction_quality = reaction_time_score();

    // Flow-optimal skill estimate
    (hit_rate * 0.4 + rally_factor * 0.3 + reaction_quality * 0.3)
        .clamp(0.0, 1.0)
}
```

### DDA State Machine

```
┌─────────────────────────────────────────────────────────┐
│                     DDA Controller                       │
├─────────────────────────────────────────────────────────┤
│  State: ASSESSING → ADJUSTING → STABLE → ASSESSING      │
│                                                          │
│  On point scored:                                        │
│    1. Update player metrics                              │
│    2. Estimate player skill                              │
│    3. Detect flow state (boredom/flow/anxiety)           │
│    4. Calculate target difficulty                        │
│    5. Smoothly adjust AI difficulty                      │
└─────────────────────────────────────────────────────────┘
```

## Difficulty Profile Curve

Based on [Karpathy's research](http://karpathy.github.io/2016/05/31/rl/), difficulty parameters follow exponential curves:

| Level | Reaction (ms) | Accuracy | Speed | Error | Description |
|-------|---------------|----------|-------|-------|-------------|
| 0 | 500 | 30% | 200 | 50px | "Training wheels" |
| 1 | 450 | 37% | 244 | 45px | Beginner |
| 2 | 389 | 44% | 289 | 40px | Easy |
| 3 | 322 | 52% | 333 | 35px | Casual |
| 4 | 250 | 59% | 378 | 29px | Normal |
| 5 | 180 | 66% | 422 | 24px | Challenging |
| 6 | 120 | 73% | 467 | 18px | Hard |
| 7 | 80 | 80% | 511 | 13px | Very Hard |
| 8 | 56 | 87% | 556 | 8px | Expert |
| 9 | 50 | 95% | 600 | 5px | Master |

Formula: `reaction = 500 * (1-t)² + 50` where `t = level/9`

## Demo UI Requirements

### HUD Elements

```
┌─────────────────────────────────────────────────────────┐
│ [Demo] [1P] [2P]        PONG        [1x][5x][10x][100x] │
│                                                          │
│  AI: ████████░░ (8/10)    Player Skill: 0.65            │
│                                                          │
│         3                           7                    │
│                                                          │
│    ███                    O                        ███   │
│    ███                                             ███   │
│    ███                                             ███   │
│                                                          │
│  Rally: 12    Best: 23                                   │
│                                                          │
│  [Download .apr (491 bytes)]   github.com/paiml/jugar   │
└─────────────────────────────────────────────────────────┘
```

### Interactive Elements

1. **Game Mode Buttons**: Demo / 1P / 2P
2. **Speed Buttons**: 1x, 5x, 10x, 50x, 100x, 1000x
3. **AI Difficulty**: Click to cycle (0-9) or keyboard (0-9)
4. **Download .apr**: Export current model as JSON file

## Speed Multiplier (Demo Acceleration)

For showcasing AI learning/adaptation:

| Speed | Physics/sec | Use Case |
|-------|-------------|----------|
| 1x | 60 | Normal gameplay |
| 5x | 300 | Watch AI adapt |
| 10x | 600 | Quick demonstration |
| 50x | 3,000 | Training visualization |
| 100x | 6,000 | Rapid iteration |
| 1000x | 60,000 | Stress test |

**Safety**: Photosensitivity warning for speeds > 10x

## Test Requirements

### Unit Tests (95% coverage)

```rust
#[test] fn test_skill_estimation_empty_history()
#[test] fn test_skill_estimation_perfect_player()
#[test] fn test_skill_estimation_poor_player()
#[test] fn test_dda_increases_on_boredom()
#[test] fn test_dda_decreases_on_anxiety()
#[test] fn test_deterministic_rng_same_seed()
#[test] fn test_difficulty_curve_monotonic()
#[test] fn test_apr_serialization_roundtrip()
#[test] fn test_model_size_under_1kb()
```

### Integration Tests

```rust
#[test] fn test_full_game_with_dda()
#[test] fn test_model_download_and_reload()
#[test] fn test_speed_multiplier_physics_accuracy()
```

## Success Criteria

1. [ ] `.apr` file downloadable via button click
2. [ ] Model JSON is human-readable and documented
3. [ ] AI behavior is deterministic (same seed = same game)
4. [ ] DDA keeps player in "flow" state (40-60% win rate)
5. [ ] Speed multiplier works up to 1000x
6. [ ] All tests pass with 95% coverage
7. [ ] Model size < 1KB (491 bytes target)

## References

- Csikszentmihalyi, M. (1990). *Flow: The Psychology of Optimal Experience*
- Hunicke, R. (2005). "The Case for Dynamic Difficulty Adjustment in Games"
- Karpathy, A. (2016). "Deep Reinforcement Learning: Pong from Pixels"
- Zohaib, M. (2018). "Dynamic Difficulty Adjustment (DDA) in Computer Games: A Review"
