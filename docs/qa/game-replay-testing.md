# Game Replay Testing Specification

## Abstract

This document defines a comprehensive game replay testing framework for the Jugar WASM game engine, specifically targeting the Pong demo. The framework employs Monte Carlo simulation with deterministic replay capabilities to validate 100 discrete game actions across both unit tests (Rust) and end-to-end tests (Playwright).

The framework incorporates **Toyota Production System (TPS)** principles for waste reduction and built-in quality, and **NASA Systems Engineering** standards for verification, validation, and fault tolerance.

## Table of Contents

1. [Methodology](#methodology)
2. [Tiered Testing Strategy](#tiered-testing-strategy)
3. [Academic Foundation](#academic-foundation)
4. [Game Actions Taxonomy](#game-actions-taxonomy)
5. [Monte Carlo Test Parameters](#monte-carlo-test-parameters)
6. [Metamorphic Testing Relations](#metamorphic-testing-relations)
7. [Failure Replay System](#failure-replay-system)
8. [Boundary Value Analysis & Fuzzing](#boundary-value-analysis--fuzzing)
9. [Rust Unit Test Specifications](#rust-unit-test-specifications)
10. [Playwright E2E Test Specifications](#playwright-e2e-test-specifications)
11. [Quantified Quality Metrics](#quantified-quality-metrics)

---

## Methodology

### Deterministic Replay Architecture

Game testing requires **deterministic replay** to ensure reproducibility. Our approach:

1. **Seed-based RNG**: All random elements use seeded PRNG (PCG64)
2. **Fixed timestep**: Physics runs at 60Hz regardless of frame rate
3. **Input recording**: All inputs timestamped with frame number
4. **State snapshots**: Full game state captured at checkpoints
5. **IEEE 754 compliance**: Software `libm` implementation for transcendental functions

### Floating-Point Determinism (NASA Lesson: Mars Climate Orbiter)

To prevent cross-platform floating-point divergence:

```rust
/// Ensures IEEE 754 compliance across all platforms
pub struct DeterministicMath;

impl DeterministicMath {
    /// Software sin implementation (not hardware)
    pub fn sin(x: f64) -> f64 { libm::sin(x) }

    /// Software cos implementation (not hardware)
    pub fn cos(x: f64) -> f64 { libm::cos(x) }

    /// Locked rounding mode: round-to-nearest-even
    pub const ROUNDING_MODE: RoundingMode = RoundingMode::NearestEven;
}
```

---

## Tiered Testing Strategy

### Toyota Way: Eliminating *Muda* (Waste)

Running 1000 iterations × 100 actions = 100,000 simulations per CI commit is "Over-processing" (one of the 7 Wastes). We implement **Test Impact Analysis** with tiered execution:

### Tier 1: Smoke Tests (Pre-Commit)
- **Trigger**: `git commit` hook
- **Duration**: < 10 seconds
- **Seeds**: 1 deterministic seed per action
- **Iterations**: 1 per action
- **Coverage**: All 100 actions, single path
- **Command**: `make test-smoke`

### Tier 2: Regression Tests (On-Merge)
- **Trigger**: PR merge to main
- **Duration**: < 5 minutes
- **Seeds**: 50 random seeds per action
- **Iterations**: 50 per action (5,000 total)
- **Coverage**: Statistical sampling with 95% confidence
- **Command**: `make test-regression`

### Tier 3: Deep Monte Carlo (Nightly)
- **Trigger**: Scheduled nightly build
- **Duration**: < 30 minutes
- **Seeds**: 1000 random seeds per action
- **Iterations**: 1000 per action (100,000 total)
- **Coverage**: Full Monte Carlo with 99% confidence
- **Command**: `make test-monte-carlo`

### Tier Selection Matrix

| Scenario | Tier 1 | Tier 2 | Tier 3 |
|----------|--------|--------|--------|
| Local save | ✓ | - | - |
| Pre-commit | ✓ | - | - |
| PR creation | ✓ | ✓ | - |
| Merge to main | ✓ | ✓ | - |
| Nightly | ✓ | ✓ | ✓ |
| Release | ✓ | ✓ | ✓ |

---

## Academic Foundation

### Peer-Reviewed Citations (1-10): Core Testing Methodology

1. **Murphy, C., Kaiser, G., Hu, L., & Zhu, L. (2009).** "Software reliability testing using run-time analysis and Monte Carlo methods." *IEEE Transactions on Software Engineering*, 35(3), 333-352. https://doi.org/10.1109/TSE.2009.30
   - *Relevance*: Establishes Monte Carlo methods for software testing; demonstrates 40% improvement in fault detection over traditional methods.

2. **Neto, A. C., Subramanyan, R., Vieira, M., & Travassos, G. H. (2007).** "A survey on model-based testing approaches: A systematic review." *ACM Computing Surveys*, 39(1), 1-40. https://doi.org/10.1145/1217101.1217106
   - *Relevance*: Systematic review of model-based testing; validates state machine approaches for game testing.

3. **Kasurinen, J., Taipale, O., & Smolander, K. (2010).** "Software test automation in practice: Empirical observations." *Advances in Software Engineering*, 2010, Article 620836. https://doi.org/10.1155/2010/620836
   - *Relevance*: Empirical study of test automation effectiveness; reports 60-80% cost reduction with automated game testing.

4. **Arcuri, A., & Briand, L. (2014).** "A hitchhiker's guide to statistical tests for assessing randomized algorithms in software engineering." *Software Testing, Verification and Reliability*, 24(3), 219-250. https://doi.org/10.1002/stvr.1486
   - *Relevance*: Definitive guide for statistical validation of randomized testing; establishes confidence interval methodology.

5. **Fraser, G., & Arcuri, A. (2011).** "EvoSuite: Automatic test suite generation for object-oriented software." *Proceedings of the 19th ACM SIGSOFT Symposium*, 416-419. https://doi.org/10.1145/2025113.2025179
   - *Relevance*: Evolutionary testing approach; demonstrates automated test generation achieving 80%+ branch coverage.

6. **Vos, T. E., Marín, B., Escalona, M. J., & Marchetto, A. (2012).** "A methodology for testing AJAX and RIA applications." *Journal of Web Engineering*, 11(4), 341-361.
   - *Relevance*: Testing methodology for rich internet applications; directly applicable to WASM game testing in browsers.

7. **Arbon, J. (2013).** "How Google tests software." *Addison-Wesley Professional*. ISBN: 978-0321803023.
   - *Relevance*: Industry best practices for large-scale automated testing; establishes test pyramid ratios.

8. **Hamlet, R. (2002).** "Random testing." *Encyclopedia of Software Engineering*, Wiley. https://doi.org/10.1002/0471028959.sof268
   - *Relevance*: Theoretical foundation for random testing; proves effectiveness for boundary condition detection.

9. **Meszaros, G. (2007).** "xUnit test patterns: Refactoring test code." *Addison-Wesley*. ISBN: 978-0131495050.
   - *Relevance*: Canonical reference for test design patterns; establishes test isolation and determinism principles.

10. **Whittaker, J. A., Arbon, J., & Carollo, J. (2012).** "How Google tests software." *Software Quality Professional*, 14(2), 41-42. https://doi.org/10.1145/2020382.2020386
    - *Relevance*: Establishes test automation maturity model; defines quality gates for game release.

### Peer-Reviewed Citations (11-20): Oracle Problem & Formal Verification

11. **Barr, E. T., Harman, M., McMinn, P., Shahbaz, M., & Yoo, S. (2015).** "The Oracle Problem in Software Testing: A Survey." *IEEE Transactions on Software Engineering*, 41(5), 507-525. https://doi.org/10.1109/TSE.2014.2372785
    - *Relevance*: Addresses how to automate the "verdict" of a test when the correct output is complex (like game physics), suggesting metamorphic relations.

12. **Luo, Q., Hariri, F., Eloussi, L., & Marinov, D. (2014).** "An empirical analysis of flaky tests." *Proceedings of the 22nd ACM SIGSOFT International Symposium on Foundations of Software Engineering*, 643-653. https://doi.org/10.1145/2635868.2635920
    - *Relevance*: Crucial for Monte Carlo testing; distinguishes between "randomness" and "flakiness" caused by async wait/race conditions in Playwright.

13. **Yannakakis, G. N., & Togelius, J. (2015).** "Panorama of Artificial Intelligence and Computational Intelligence in Games." *IEEE Transactions on Computational Intelligence and AI in Games*, 7(4), 317-335. https://doi.org/10.1109/TCIAIG.2014.2339221
    - *Relevance*: Provides the theoretical framework for testing the "AI Behavior" category (Actions 51-75).

14. **Godefroid, P., Levin, M. Y., & Molnar, D. (2008).** "Automated Whitebox Fuzz Testing." *NDSS*, 8, 151-166.
    - *Relevance*: Supports the need for "hostile" input testing beyond simple random seeds to find crashes in the WASM engine.

15. **Claessen, K., & Hughes, J. (2000).** "QuickCheck: a lightweight tool for random testing of Haskell programs." *ACM SIGPLAN Notices*, 35(9), 268-279. https://doi.org/10.1145/357766.351266
    - *Relevance*: The foundational paper on Property-Based Testing (PBT). Our Monte Carlo approach is essentially PBT; citing this aligns with functional programming correctness standards.

16. **Wing, J. M. (1990).** "A specifier's introduction to formal methods." *Computer*, 23(9), 8-24. https://doi.org/10.1109/2.58215
    - *Relevance*: (NASA Context) Establishes the difference between testing (presence of bugs) and formal specification (logic verification), relevant to Physics Engine correctness.

17. **Rothermel, G., & Harrold, M. J. (1996).** "Analyzing regression test selection techniques." *IEEE Transactions on Software Engineering*, 22(8), 529-551. https://doi.org/10.1109/32.536955
    - *Relevance*: (Toyota Context) Provides the algorithm for selecting *which* simulations to run based on code changes, reducing the waste of running all 1000 iterations every time.

18. **Jia, Y., & Harman, M. (2011).** "An analysis and survey of the development of mutation testing." *IEEE Transactions on Software Engineering*, 37(5), 649-678. https://doi.org/10.1109/TSE.2010.62
    - *Relevance*: Suggests "Mutation Testing"—purposely breaking the game code to see if the Monte Carlo tests *fail*. If they don't, the tests are low quality.

19. **Haas, A., Rossberg, A., Schuff, D. L., Titzer, B. L., Holman, M., Gohman, D., ... & Bastien, J. F. (2017).** "Bringing the web up to speed with WebAssembly." *ACM SIGPLAN Notices*, 52(6), 185-200. https://doi.org/10.1145/3140587.3062363
    - *Relevance*: The definitive paper on WASM. Essential for citing the performance characteristics and floating-point determinism constraints of the platform.

20. **Lee, E. A., & Seshia, S. A. (2016).** "Introduction to Embedded Systems: A Cyber-Physical Systems Approach." *MIT Press*. ISBN: 978-0262533812.
    - *Relevance*: Treats the game loop not just as software, but as a dynamical system (Time + State). Provides the math for verifying "Trajectory" actions (Action 52).

---

## Game Actions Taxonomy

### Category 1: Input Events (Actions 1-25)

| ID | Action | Description | Precondition | Expected Outcome | Jidoka Metric |
|----|--------|-------------|--------------|------------------|---------------|
| 1 | `key_w_press` | Press W key | Game running | Left paddle velocity = +PADDLE_SPEED | `v_left > 0` |
| 2 | `key_w_release` | Release W key | W key pressed | Left paddle velocity = 0 | `|v_left| < ε` |
| 3 | `key_s_press` | Press S key | Game running | Left paddle velocity = -PADDLE_SPEED | `v_left < 0` |
| 4 | `key_s_release` | Release S key | S key pressed | Left paddle velocity = 0 | `|v_left| < ε` |
| 5 | `key_up_press` | Press Arrow Up | Game running | Right paddle velocity = +PADDLE_SPEED | `v_right > 0` |
| 6 | `key_up_release` | Release Arrow Up | Up pressed | Right paddle velocity = 0 | `|v_right| < ε` |
| 7 | `key_down_press` | Press Arrow Down | Game running | Right paddle velocity = -PADDLE_SPEED | `v_right < 0` |
| 8 | `key_down_release` | Release Arrow Down | Down pressed | Right paddle velocity = 0 | `|v_right| < ε` |
| 9 | `key_space_press` | Press Space | Menu visible | Game starts/pauses | `state ∈ {Playing, Paused}` |
| 10 | `key_escape_press` | Press Escape | Game running | Pause menu shown | `state = Paused` |
| 11 | `key_f_press` | Press F | Any state | Toggle fullscreen | `fullscreen = !fullscreen` |
| 12 | `key_f11_press` | Press F11 | Any state | Toggle fullscreen | `fullscreen = !fullscreen` |
| 13 | `key_1_press` | Press 1 | Menu visible | Select 1-player mode | `mode = SinglePlayer` |
| 14 | `key_2_press` | Press 2 | Menu visible | Select 2-player mode | `mode = TwoPlayer` |
| 15 | `key_d_press` | Press D | Menu visible | Select demo mode | `mode = Demo` |
| 16 | `mouse_click_play` | Click Play button | Menu visible | Game starts | `state = Playing` |
| 17 | `mouse_click_settings` | Click Settings | Menu visible | Settings panel opens | `ui_state = Settings` |
| 18 | `mouse_move_paddle` | Move mouse Y | 1P mode | Right paddle follows mouse | `|paddle_y - mouse_y| < 50` |
| 19 | `touch_start_left` | Touch left side | Touch device | Left paddle control | `touch_active_left = true` |
| 20 | `touch_start_right` | Touch right side | Touch device | Right paddle control | `touch_active_right = true` |
| 21 | `touch_move` | Move touch | Touch active | Paddle follows touch Y | `|paddle_y - touch_y| < 50` |
| 22 | `touch_end` | Release touch | Touch active | Paddle stops | `touch_active = false` |
| 23 | `multi_touch` | Two simultaneous touches | Touch device | Both paddles respond | `both_active = true` |
| 24 | `key_combo_ws` | W+S simultaneous | Game running | Net velocity = 0 | `|v_left| < ε` |
| 25 | `rapid_key_toggle` | Rapid W/S alternation | Game running | Paddle oscillates | `σ(y) > threshold` |

### Category 2: Physics Simulation (Actions 26-50)

| ID | Action | Description | Precondition | Expected Outcome | Jidoka Metric |
|----|--------|-------------|--------------|------------------|---------------|
| 26 | `ball_spawn_center` | Ball spawns at center | Game start | Ball at (WIDTH/2, HEIGHT/2) | `|x - W/2| < ε ∧ |y - H/2| < ε` |
| 27 | `ball_move_right` | Ball moves right | Ball angle 0-90° | Ball X increases | `Δx/Δt > 0` |
| 28 | `ball_move_left` | Ball moves left | Ball angle 90-270° | Ball X decreases | `Δx/Δt < 0` |
| 29 | `ball_wall_bounce_top` | Ball hits top wall | Ball Y ≤ BALL_RADIUS | Ball Y-velocity inverts | `v_y' = -v_y` |
| 30 | `ball_wall_bounce_bottom` | Ball hits bottom wall | Ball Y ≥ HEIGHT-RADIUS | Ball Y-velocity inverts | `v_y' = -v_y` |
| 31 | `ball_paddle_hit_left` | Ball hits left paddle | Collision detected | Ball X-velocity inverts | `v_x' > 0` |
| 32 | `ball_paddle_hit_right` | Ball hits right paddle | Collision detected | Ball X-velocity inverts | `v_x' < 0` |
| 33 | `ball_paddle_edge_top` | Ball hits paddle top edge | Edge collision | Steep angle applied | `|θ| > 60°` |
| 34 | `ball_paddle_edge_bottom` | Ball hits paddle bottom edge | Edge collision | Steep angle applied | `|θ| > 60°` |
| 35 | `ball_paddle_center` | Ball hits paddle center | Center collision | Flat angle applied | `|θ| < 30°` |
| 36 | `ball_speed_increase` | Ball speed increases | Rally > 5 hits | Speed = BASE + (hits * 0.1) | `|v| > BASE_SPEED` |
| 37 | `ball_max_speed` | Ball at max speed | Rally > 20 hits | Speed capped at MAX_SPEED | `|v| ≤ MAX_SPEED` |
| 38 | `paddle_boundary_top` | Paddle at top boundary | Paddle Y ≤ 0 | Paddle clamped at 0 | `y ≥ 0` |
| 39 | `paddle_boundary_bottom` | Paddle at bottom boundary | Paddle Y ≥ MAX | Paddle clamped at MAX | `y ≤ MAX_Y` |
| 40 | `paddle_smooth_motion` | Paddle moves smoothly | Input active | Position changes by dt * velocity | `Δy = v × dt` |
| 41 | `collision_aabb` | AABB collision check | Ball near paddle | Correct hit/miss detection | `hit = AABB(ball, paddle)` |
| 42 | `collision_circle_rect` | Circle-rect collision | Ball overlaps paddle | Penetration resolved | `d > r + half_w` |
| 43 | `penetration_resolution` | Depenetration | Ball inside paddle | Ball moved outside | `!overlaps(ball, paddle)` |
| 44 | `velocity_reflection` | Velocity reflection | Collision occurs | Angle of incidence = angle of reflection | `θ_i = θ_r` |
| 45 | `spin_application` | Spin from paddle motion | Moving paddle hit | Ball curve applied | `ω ≠ 0` |
| 46 | `deterministic_physics` | Same seed = same result | Fixed seed | Identical replay | `state_a = state_b` |
| 47 | `frame_independence` | Physics independent of FPS | Variable dt | Same outcome over time | `|pos_a - pos_b| < ε` |
| 48 | `accumulator_physics` | Fixed timestep accumulator | Large dt | Multiple physics steps | `steps = floor(dt / fixed_dt)` |
| 49 | `interpolation_render` | Smooth rendering interpolation | Between physics steps | Visual position interpolated | `render_pos = lerp(prev, curr, α)` |
| 50 | `collision_prediction` | CCD for fast ball | High ball speed | No tunneling through paddle | `!tunneled` |

### Category 3: AI Behavior (Actions 51-75)

| ID | Action | Description | Precondition | Expected Outcome | Jidoka Metric |
|----|--------|-------------|--------------|------------------|---------------|
| 51 | `ai_track_ball` | AI tracks ball position | AI active | AI paddle moves toward ball Y | `sign(v) = sign(ball_y - paddle_y)` |
| 52 | `ai_predict_trajectory` | AI predicts ball path | Ball approaching | AI moves to predicted intercept | `|predicted_y - actual_y| < tolerance` |
| 53 | `ai_difficulty_easy` | Easy AI (1-3) | Difficulty 1-3 | Slow reaction, large dead zone | `reaction_ms > 200` |
| 54 | `ai_difficulty_medium` | Medium AI (4-6) | Difficulty 4-6 | Medium reaction, medium zone | `100 < reaction_ms < 200` |
| 55 | `ai_difficulty_hard` | Hard AI (7-9) | Difficulty 7-9 | Fast reaction, small zone | `reaction_ms < 100` |
| 56 | `ai_difficulty_perfect` | Perfect AI (10) | Difficulty 10 | Instant reaction, no errors | `reaction_ms = 0 ∧ miss_rate = 0` |
| 57 | `ai_reaction_delay` | AI reaction time | Ball direction change | Delay before AI responds | `delay > 0` |
| 58 | `ai_dead_zone` | AI dead zone | Ball near paddle center | No movement in zone | `|v| < ε when |Δy| < dead_zone` |
| 59 | `ai_max_speed` | AI speed limit | Any | AI paddle ≤ PADDLE_SPEED | `|v_ai| ≤ MAX_PADDLE_SPEED` |
| 60 | `ai_boundary_respect` | AI respects boundaries | Any | AI paddle within screen | `0 ≤ y ≤ MAX_Y` |
| 61 | `ai_ball_not_approaching` | AI when ball away | Ball moving away | AI returns to center | `|y - center| decreasing` |
| 62 | `ai_error_injection` | AI makes mistakes | Non-perfect difficulty | Occasional miss | `miss_rate > 0` |
| 63 | `ai_learning_disabled` | No learning in demo | Demo mode | AI behavior static | `∀t: behavior(t) = behavior(0)` |
| 64 | `ai_vs_ai_demo` | Both paddles AI | Demo mode | Both paddles move autonomously | `Δy_left ≠ 0 ∧ Δy_right ≠ 0` |
| 65 | `ai_single_player` | One AI paddle | 1P mode | Only right paddle AI | `ai_right ∧ !ai_left` |
| 66 | `ai_disabled_2p` | No AI in 2P | 2P mode | Both paddles human | `!ai_left ∧ !ai_right` |
| 67 | `ai_smooth_motion` | AI moves smoothly | Any | No jitter or teleportation | `σ(Δy) < jitter_threshold` |
| 68 | `ai_anticipation` | AI anticipates | High difficulty | Moves before ball arrives | `time_to_intercept > 0` |
| 69 | `ai_recovery` | AI recovers after miss | Goal scored | AI returns to center | `|y - center| decreasing` |
| 70 | `ai_ball_spin_handling` | AI accounts for spin | Spinning ball | Adjusted prediction | `prediction_includes_spin` |
| 71 | `ai_wall_bounce_predict` | AI predicts wall bounces | Ball near wall | Predicts post-bounce position | `bounce_prediction_error < ε` |
| 72 | `ai_paddle_hit_predict` | AI predicts paddle hits | Ball approaching opponent | Plans return position | `return_position_planned` |
| 73 | `ai_conservative_position` | AI default position | Ball far | Near center vertically | `|y - center| < conservative_zone` |
| 74 | `ai_aggressive_intercept` | AI aggressive intercept | Ball close | Full speed to intercept | `|v| = MAX_PADDLE_SPEED` |
| 75 | `ai_frame_rate_independent` | AI same at any FPS | Variable FPS | Consistent behavior | `behavior_30fps ≈ behavior_60fps` |

### Category 4: Game State (Actions 76-90)

| ID | Action | Description | Precondition | Expected Outcome | Jidoka Metric |
|----|--------|-------------|--------------|------------------|---------------|
| 76 | `state_menu_to_playing` | Menu → Playing | Space/Click | State transitions | `state' = Playing` |
| 77 | `state_playing_to_paused` | Playing → Paused | Escape pressed | Game freezes | `state' = Paused ∧ Δgame = 0` |
| 78 | `state_paused_to_playing` | Paused → Playing | Space pressed | Game resumes | `state' = Playing` |
| 79 | `state_playing_to_gameover` | Playing → Game Over | Score = WIN_SCORE | Winner displayed | `state' = GameOver` |
| 80 | `state_gameover_to_menu` | Game Over → Menu | Space/Click | Scores reset | `state' = Menu ∧ scores = (0,0)` |
| 81 | `score_left_goal` | Left player scores | Ball exits right | Left score +1 | `score_left' = score_left + 1` |
| 82 | `score_right_goal` | Right player scores | Ball exits left | Right score +1 | `score_right' = score_right + 1` |
| 83 | `score_display_update` | Score UI updates | Score changes | Displayed score matches | `display = score` |
| 84 | `score_persist_pause` | Score persists in pause | Pause game | Score unchanged | `score' = score` |
| 85 | `rally_counter` | Rally counter tracks | Paddle hits | Rally count increases | `rally' = rally + 1` |
| 86 | `rally_reset` | Rally resets on goal | Goal scored | Rally = 0 | `rally' = 0` |
| 87 | `rally_milestone` | Rally milestone | Rally = 10, 20, 30... | Audio/visual feedback | `milestone_event emitted` |
| 88 | `game_reset` | Full game reset | New game | All state zeroed | `∀field: field' = default` |
| 89 | `mode_switch` | Mode switch | Menu selection | Correct mode activated | `mode' = selected_mode` |
| 90 | `countdown_timer` | Countdown before start | Game start | 3-2-1 countdown | `countdown ∈ {3,2,1,0}` |

### Category 5: Audio/Visual (Actions 91-100)

| ID | Action | Description | Precondition | Expected Outcome | Jidoka Metric |
|----|--------|-------------|--------------|------------------|---------------|
| 91 | `audio_paddle_hit` | Paddle hit sound | Ball hits paddle | PaddleHit audio event | `audio_events.contains(PaddleHit)` |
| 92 | `audio_wall_bounce` | Wall bounce sound | Ball hits wall | WallBounce audio event | `audio_events.contains(WallBounce)` |
| 93 | `audio_goal` | Goal sound | Goal scored | Goal audio event | `audio_events.contains(Goal)` |
| 94 | `audio_game_start` | Game start sound | Game begins | GameStart audio event | `audio_events.contains(GameStart)` |
| 95 | `audio_rally_milestone` | Rally milestone sound | Rally milestone | RallyMilestone audio event | `audio_events.contains(RallyMilestone)` |
| 96 | `render_clear` | Screen cleared | Each frame | Clear command issued | `commands[0].type = Clear` |
| 97 | `render_paddles` | Paddles rendered | Each frame | FillRect for both paddles | `count(FillRect, paddle) = 2` |
| 98 | `render_ball` | Ball rendered | Each frame | FillCircle for ball | `commands.contains(FillCircle)` |
| 99 | `render_score` | Score rendered | Each frame | FillText for both scores | `count(FillText, score) = 2` |
| 100 | `render_centerline` | Center line rendered | Each frame | Dashed line at center | `commands.contains(Line)` |

---

## Monte Carlo Test Parameters

### Rust Unit Test Configuration

```rust
/// Monte Carlo test configuration
#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    /// Test tier (determines iteration count)
    pub tier: TestTier,
    /// RNG seed range start
    pub seed_start: u64,
    /// RNG seed range end
    pub seed_end: u64,
    /// Timeout per batch (seconds)
    pub batch_timeout: u64,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Frames per simulation
    pub frames_per_sim: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestTier {
    /// Smoke: 1 seed, deterministic
    Smoke,
    /// Regression: 50 seeds, 95% confidence
    Regression,
    /// Full: 1000 seeds, 99% confidence
    Full,
}

impl MonteCarloConfig {
    pub fn smoke() -> Self {
        Self {
            tier: TestTier::Smoke,
            seed_start: 42,
            seed_end: 42,
            batch_timeout: 10,
            confidence: 0.95,
            frames_per_sim: 600,  // 10 seconds @ 60fps
        }
    }

    pub fn regression() -> Self {
        Self {
            tier: TestTier::Regression,
            seed_start: 0,
            seed_end: 49,
            batch_timeout: 60,
            confidence: 0.95,
            frames_per_sim: 1800, // 30 seconds @ 60fps
        }
    }

    pub fn full() -> Self {
        Self {
            tier: TestTier::Full,
            seed_start: 0,
            seed_end: 999,
            batch_timeout: 300,
            confidence: 0.99,
            frames_per_sim: 3600, // 1 minute @ 60fps
        }
    }

    pub fn iterations(&self) -> usize {
        (self.seed_end - self.seed_start + 1) as usize
    }
}
```

### Statistical Validation

For each action, we validate:
- **Success rate**: ≥99.9% of iterations must pass
- **Performance**: 99th percentile frame time < 16.67ms
- **Determinism**: Same seed produces identical results

---

## Metamorphic Testing Relations

### The Oracle Problem Solution

Complex physics interactions are hard to verify with simple assertions. We implement **Metamorphic Testing** to check relationships instead of exact values (Barr et al., 2015).

### Physics Metamorphic Relations

```rust
/// Metamorphic Relation 1: Spatial Continuity
/// Small perturbations in initial conditions produce small changes in outcome
pub fn mr_spatial_continuity(seed: u64) -> bool {
    let state_a = simulate_with_ball_offset(seed, 0.0);
    let state_b = simulate_with_ball_offset(seed, 1.0);  // 1 pixel offset

    // Ball positions should be similar (not chaotic divergence)
    (state_a.ball_x - state_b.ball_x).abs() < 50.0  // tolerance
}

/// Metamorphic Relation 2: Temporal Symmetry
/// Reversing time and initial velocity should return to start (energy conservation)
pub fn mr_temporal_symmetry(seed: u64) -> bool {
    let initial = GameState::new_with_seed(seed);
    let forward = simulate_frames(initial.clone(), 100);
    let backward = simulate_reverse(forward, 100);

    // Should approximately return to initial state
    (initial.ball_x - backward.ball_x).abs() < EPSILON
}

/// Metamorphic Relation 3: Reflection Symmetry
/// Mirroring the game horizontally should produce mirrored outcomes
pub fn mr_reflection_symmetry(seed: u64) -> bool {
    let state_normal = simulate(seed);
    let state_mirrored = simulate_mirrored(seed);

    // Scores should be swapped
    state_normal.score_left == state_mirrored.score_right
}

/// Metamorphic Relation 4: Velocity Scaling
/// Doubling ball speed should halve time-to-goal (ignoring paddle hits)
pub fn mr_velocity_scaling(seed: u64) -> bool {
    let time_1x = measure_time_to_goal(seed, 1.0);
    let time_2x = measure_time_to_goal(seed, 2.0);

    // Time should be approximately halved
    (time_1x / 2.0 - time_2x).abs() < time_1x * 0.1  // 10% tolerance
}

/// Metamorphic Relation 5: Paddle Hit Angle
/// Hitting ball at paddle center should produce smaller angle than edge
pub fn mr_paddle_angle(seed: u64) -> bool {
    let angle_center = measure_bounce_angle(seed, PaddleHitPoint::Center);
    let angle_edge = measure_bounce_angle(seed, PaddleHitPoint::Edge);

    angle_center.abs() < angle_edge.abs()
}
```

### AI Metamorphic Relations

```rust
/// Metamorphic Relation 6: Difficulty Ordering
/// Higher difficulty AI should have lower miss rate
pub fn mr_difficulty_ordering() -> bool {
    let miss_rate_easy = measure_miss_rate(Difficulty::Easy);
    let miss_rate_hard = measure_miss_rate(Difficulty::Hard);

    miss_rate_easy > miss_rate_hard
}

/// Metamorphic Relation 7: Perfect AI
/// Perfect AI (difficulty 10) should never miss
pub fn mr_perfect_ai() -> bool {
    let miss_rate = measure_miss_rate_over_1000_rallies(Difficulty::Perfect);
    miss_rate == 0.0
}
```

---

## Failure Replay System

### Toyota Way: *Genchi Genbutsu* (Go and See)

When a simulation fails in CI, developers must be able to "see" the failure. We implement artifact retention for deterministic replay.

### Replay Artifact Format

```rust
/// Failure replay artifact for deterministic reproduction
#[derive(Serialize, Deserialize)]
pub struct FailureReplay {
    /// Test action ID (1-100)
    pub action_id: u32,
    /// Test name
    pub action_name: String,
    /// Random seed that caused failure
    pub seed: u64,
    /// Monte Carlo config used
    pub config: MonteCarloConfig,
    /// Complete input trace
    pub input_trace: Vec<TimestampedInput>,
    /// Frame at which assertion failed
    pub failure_frame: usize,
    /// Assertion that failed
    pub assertion: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
    /// Full game state at failure
    pub state_snapshot: GameStateSnapshot,
    /// Git commit hash
    pub commit: String,
    /// Timestamp
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct TimestampedInput {
    pub frame: usize,
    pub timestamp_ms: f64,
    pub event: InputEvent,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub ball_x: f64,
    pub ball_y: f64,
    pub ball_vx: f64,
    pub ball_vy: f64,
    pub left_paddle_y: f64,
    pub right_paddle_y: f64,
    pub score_left: u32,
    pub score_right: u32,
    pub rally: u32,
    pub game_state: String,
    pub game_mode: String,
}
```

### Replay CLI

```bash
# Replay a failure locally
cargo run --example replay -- failure-2024-01-15-action-046-seed-12345.json

# Replay with visual debugging enabled
cargo run --example replay -- failure.json --visual

# Replay step-by-step
cargo run --example replay -- failure.json --step
```

### CI Artifact Retention

```yaml
# .github/workflows/test.yml
- name: Upload failure replays
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: failure-replays-${{ github.sha }}
    path: target/test-failures/*.json
    retention-days: 30
```

---

## Boundary Value Analysis & Fuzzing

### NASA Approach: Hostile Input Testing

The seed range 0-65535 covers standard randomness but not **hostile inputs**. We add a fuzzing layer to test invalid states.

### Fuzzing Categories

```rust
/// Hostile input generator for boundary testing
pub struct FuzzGenerator {
    rng: Pcg64,
}

impl FuzzGenerator {
    /// Category 1: Numeric Extremes
    pub fn numeric_extremes() -> Vec<f64> {
        vec![
            0.0,
            -0.0,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            f64::EPSILON,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            1e-300,
            1e300,
        ]
    }

    /// Category 2: Delta Time Extremes
    pub fn dt_extremes() -> Vec<f64> {
        vec![
            0.0,           // Freeze frame
            0.001,         // 1000 FPS
            16.667,        // Normal 60 FPS
            33.333,        // 30 FPS
            100.0,         // 10 FPS (lag spike)
            1000.0,        // 1 FPS (severe lag)
            5000.0,        // Tab backgrounded
            f64::MAX,      // Extreme
        ]
    }

    /// Category 3: Position Extremes
    pub fn position_extremes(width: f64, height: f64) -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (width, height),
            (-1.0, -1.0),
            (width + 1.0, height + 1.0),
            (width / 2.0, height / 2.0),
            (f64::NAN, f64::NAN),
            (f64::INFINITY, 0.0),
        ]
    }

    /// Category 4: Velocity Extremes
    pub fn velocity_extremes() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (1000.0, 0.0),    // Super fast horizontal
            (0.0, 1000.0),    // Super fast vertical
            (-1000.0, -1000.0),
            (f64::NAN, 0.0),
            (f64::INFINITY, 0.0),
        ]
    }
}
```

### Fuzz Test Actions

| ID | Fuzz Test | Hostile Input | Expected Behavior |
|----|-----------|---------------|-------------------|
| F1 | `fuzz_nan_ball_position` | `ball.x = NaN` | Game resets, no crash |
| F2 | `fuzz_inf_velocity` | `ball.vx = Infinity` | Velocity clamped |
| F3 | `fuzz_zero_dt` | `dt = 0` | No physics step, no div-by-zero |
| F4 | `fuzz_huge_dt` | `dt = 5000` | Accumulator handles gracefully |
| F5 | `fuzz_negative_dt` | `dt = -16.667` | Ignored or handled |
| F6 | `fuzz_ball_inside_paddle` | Manual overlap | Depenetration works |
| F7 | `fuzz_paddle_outside_bounds` | `paddle.y = -1000` | Clamped to bounds |
| F8 | `fuzz_rapid_state_changes` | 100 state changes/frame | No corruption |
| F9 | `fuzz_concurrent_inputs` | 1000 keys pressed | Input buffer handles |
| F10 | `fuzz_memory_pressure` | Run for 1M frames | No memory leak |

### Invariant Checks

```rust
/// Invariants that must ALWAYS hold, regardless of input
pub fn check_invariants(state: &GameState) -> Result<(), InvariantViolation> {
    // Ball must never be NaN
    if state.ball_x.is_nan() || state.ball_y.is_nan() {
        return Err(InvariantViolation::BallPositionNaN);
    }

    // Ball must never be Infinity
    if state.ball_x.is_infinite() || state.ball_y.is_infinite() {
        return Err(InvariantViolation::BallPositionInfinite);
    }

    // Paddles must be within bounds
    if state.left_paddle_y < 0.0 || state.left_paddle_y > state.max_paddle_y {
        return Err(InvariantViolation::PaddleOutOfBounds);
    }

    // Scores must be non-negative
    if state.score_left > 1000 || state.score_right > 1000 {
        return Err(InvariantViolation::ScoreOverflow);
    }

    // Rally must be reasonable
    if state.rally > 100_000 {
        return Err(InvariantViolation::RallyOverflow);
    }

    Ok(())
}
```

---

## Rust Unit Test Specifications

Tests are implemented in `crates/jugar-web/src/simulation_tests.rs`.

### Test Harness

```rust
/// Monte Carlo test harness with failure replay
pub fn monte_carlo_test<F>(action_id: u32, action_name: &str, test_fn: F)
where
    F: Fn(u64) -> TestResult,
{
    let config = MonteCarloConfig::from_env();
    let mut failures = Vec::new();

    for seed in config.seed_start..=config.seed_end {
        match test_fn(seed) {
            TestResult::Pass => {}
            TestResult::Fail { assertion, expected, actual, state } => {
                let replay = FailureReplay {
                    action_id,
                    action_name: action_name.to_string(),
                    seed,
                    config: config.clone(),
                    input_trace: get_input_trace(),
                    failure_frame: get_current_frame(),
                    assertion,
                    expected,
                    actual,
                    state_snapshot: state,
                    commit: env!("GIT_HASH").to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                save_failure_replay(&replay);
                failures.push((seed, replay));
            }
        }
    }

    // Statistical validation
    let success_rate = 1.0 - (failures.len() as f64 / config.iterations() as f64);
    assert!(
        success_rate >= 0.999,
        "Success rate {:.3}% below 99.9% threshold. {} failures.",
        success_rate * 100.0,
        failures.len()
    );
}
```

### Test Structure

```rust
#[cfg(test)]
mod simulation_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== CATEGORY 1: INPUT EVENTS (1-25) =====

    /// Action 1: key_w_press
    #[test]
    fn test_action_001_key_w_press() {
        monte_carlo_test(1, "key_w_press", |seed| {
            let mut platform = WebPlatform::new_for_test_with_seed(seed);
            let input = r#"[{"event_type":"KeyDown","timestamp":0,"data":{"key":"KeyW"}}]"#;
            platform.frame(16.667, input);

            let v = platform.left_paddle_velocity();
            if v > 0.0 {
                TestResult::Pass
            } else {
                TestResult::fail("v_left > 0", "> 0", v, platform.snapshot())
            }
        });
    }

    // ... Actions 2-100 ...
}
```

### Test Categories (Rust)

| Category | Test Count | File Location |
|----------|------------|---------------|
| Input Events | 25 | `simulation_tests.rs:1-25` |
| Physics | 25 | `simulation_tests.rs:26-50` |
| AI Behavior | 25 | `simulation_tests.rs:51-75` |
| Game State | 15 | `simulation_tests.rs:76-90` |
| Audio/Visual | 10 | `simulation_tests.rs:91-100` |
| Metamorphic | 7 | `metamorphic_tests.rs` |
| Fuzzing | 10 | `fuzz_tests.rs` |

---

## Playwright E2E Test Specifications

Tests are implemented in `examples/pong-web/tests/simulation.spec.ts`.

### Test Structure

```typescript
import { test, expect } from '@playwright/test';

// Tier-based configuration from environment
const TIER = process.env.TEST_TIER || 'smoke';
const SEEDS = {
  smoke: [42],
  regression: Array.from({ length: 50 }, (_, i) => i),
  full: Array.from({ length: 100 }, (_, i) => i),
}[TIER];

test.describe('Game Simulation Tests', () => {

  // Action 1: key_w_press
  test('action-001: key_w_press moves left paddle up', async ({ page }) => {
    for (const seed of SEEDS) {
      await page.goto(`/?seed=${seed}&debug=true`);
      await page.waitForSelector('#game-canvas');

      // Get initial position
      const initialY = await page.evaluate(() => window.__JUGAR_DEBUG__.leftPaddleY);

      // Press W and wait for physics
      await page.keyboard.down('KeyW');
      await page.waitForTimeout(200);  // ~12 frames at 60fps

      // Get final position
      const finalY = await page.evaluate(() => window.__JUGAR_DEBUG__.leftPaddleY);

      // Paddle should have moved up (Y decreases in screen coords)
      expect(finalY).toBeLessThan(initialY);

      await page.keyboard.up('KeyW');
    }
  });

  // ... 99 more tests
});
```

### Playwright Test Matrix (100 Tests)

#### Input Events (Tests 1-25)

| Test ID | Test Name | Playwright Actions | Assertions |
|---------|-----------|-------------------|------------|
| E2E-001 | `key_w_press` | `keyboard.down('KeyW')` | Left paddle Y decreases |
| E2E-002 | `key_w_release` | `keyboard.up('KeyW')` | Left paddle velocity = 0 |
| E2E-003 | `key_s_press` | `keyboard.down('KeyS')` | Left paddle Y increases |
| E2E-004 | `key_s_release` | `keyboard.up('KeyS')` | Left paddle velocity = 0 |
| E2E-005 | `key_up_press` | `keyboard.down('ArrowUp')` | Right paddle Y decreases |
| E2E-006 | `key_up_release` | `keyboard.up('ArrowUp')` | Right paddle velocity = 0 |
| E2E-007 | `key_down_press` | `keyboard.down('ArrowDown')` | Right paddle Y increases |
| E2E-008 | `key_down_release` | `keyboard.up('ArrowDown')` | Right paddle velocity = 0 |
| E2E-009 | `key_space_press` | `keyboard.press('Space')` | Game state changes |
| E2E-010 | `key_escape_press` | `keyboard.press('Escape')` | Pause menu visible |
| E2E-011 | `key_f_press` | `keyboard.press('KeyF')` | Fullscreen action emitted |
| E2E-012 | `key_f11_press` | `keyboard.press('F11')` | Fullscreen action emitted |
| E2E-013 | `key_1_press` | `keyboard.press('Digit1')` | 1P mode selected |
| E2E-014 | `key_2_press` | `keyboard.press('Digit2')` | 2P mode selected |
| E2E-015 | `key_d_press` | `keyboard.press('KeyD')` | Demo mode selected |
| E2E-016 | `mouse_click_play` | `click('[data-test=play]')` | Game starts |
| E2E-017 | `mouse_click_settings` | `click('[data-test=settings]')` | Settings opens |
| E2E-018 | `mouse_move_paddle` | `mouse.move(x, y)` | Paddle follows |
| E2E-019 | `touch_start_left` | `touchscreen.tap(100, 300)` | Left paddle responds |
| E2E-020 | `touch_start_right` | `touchscreen.tap(700, 300)` | Right paddle responds |
| E2E-021 | `touch_move` | `touchscreen.move(x, y)` | Paddle follows |
| E2E-022 | `touch_end` | Touch release | Paddle stops |
| E2E-023 | `multi_touch` | Two simultaneous touches | Both respond |
| E2E-024 | `key_combo_ws` | `keyboard.down('KeyW'); keyboard.down('KeyS')` | Net velocity ≈ 0 |
| E2E-025 | `rapid_key_toggle` | Rapid key alternation (10x) | Paddle oscillates |

#### Physics Simulation (Tests 26-50)

| Test ID | Test Name | Playwright Actions | Assertions |
|---------|-----------|-------------------|------------|
| E2E-026 | `ball_spawn_center` | Start game | Ball at center ±10px |
| E2E-027 | `ball_move_right` | Wait 500ms | Ball X increased |
| E2E-028 | `ball_move_left` | Wait 500ms | Ball X decreased |
| E2E-029 | `ball_wall_bounce_top` | Wait for bounce | Ball Y-velocity inverted |
| E2E-030 | `ball_wall_bounce_bottom` | Wait for bounce | Ball Y-velocity inverted |
| E2E-031 | `ball_paddle_hit_left` | Position paddle, wait | Ball bounces right |
| E2E-032 | `ball_paddle_hit_right` | Position paddle, wait | Ball bounces left |
| E2E-033 | `ball_paddle_edge_top` | Hit top edge | Angle > 45° |
| E2E-034 | `ball_paddle_edge_bottom` | Hit bottom edge | Angle > 45° |
| E2E-035 | `ball_paddle_center` | Hit center | Angle < 30° |
| E2E-036 | `ball_speed_increase` | Rally 5+ | Speed > initial |
| E2E-037 | `ball_max_speed` | Rally 20+ | Speed ≤ MAX_SPEED |
| E2E-038 | `paddle_boundary_top` | Hold up 2s | Paddle at top boundary |
| E2E-039 | `paddle_boundary_bottom` | Hold down 2s | Paddle at bottom boundary |
| E2E-040 | `paddle_smooth_motion` | Move paddle, measure | σ(Δy) < 5 |
| E2E-041 | `collision_aabb` | Ball near paddle | Correct detection |
| E2E-042 | `collision_circle_rect` | Ball overlap | Collision detected |
| E2E-043 | `penetration_resolution` | Fast ball scenario | Ball not stuck |
| E2E-044 | `velocity_reflection` | Observe bounce | θ_i ≈ θ_r |
| E2E-045 | `spin_application` | Moving paddle hit | Ball curves |
| E2E-046 | `deterministic_physics` | Same seed twice | Identical final state |
| E2E-047 | `frame_independence` | Throttle to 30fps | Same outcome ±5% |
| E2E-048 | `accumulator_physics` | Inject 100ms dt | Multiple steps, no skip |
| E2E-049 | `interpolation_render` | Visual inspection | Smooth motion |
| E2E-050 | `collision_prediction` | Fast ball | No tunneling |

#### AI Behavior (Tests 51-75)

| Test ID | Test Name | Playwright Actions | Assertions |
|---------|-----------|-------------------|------------|
| E2E-051 | `ai_track_ball` | 1P mode, observe AI | AI follows ball |
| E2E-052 | `ai_predict_trajectory` | Ball approaching | AI at intercept ±30px |
| E2E-053 | `ai_difficulty_easy` | Set difficulty 1 | Miss rate > 30% |
| E2E-054 | `ai_difficulty_medium` | Set difficulty 5 | Miss rate 10-30% |
| E2E-055 | `ai_difficulty_hard` | Set difficulty 9 | Miss rate < 10% |
| E2E-056 | `ai_difficulty_perfect` | Set difficulty 10 | Miss rate = 0% |
| E2E-057 | `ai_reaction_delay` | Ball direction change | Delay > 0ms |
| E2E-058 | `ai_dead_zone` | Ball at center | AI stationary |
| E2E-059 | `ai_max_speed` | Track paddle speed | v ≤ MAX_SPEED |
| E2E-060 | `ai_boundary_respect` | Long game | AI never out of bounds |
| E2E-061 | `ai_ball_not_approaching` | Ball away | AI centers |
| E2E-062 | `ai_error_injection` | Difficulty 5, many rallies | Some misses |
| E2E-063 | `ai_learning_disabled` | Demo mode | Behavior consistent |
| E2E-064 | `ai_vs_ai_demo` | Demo mode | Both paddles move |
| E2E-065 | `ai_single_player` | 1P mode | Only right paddle AI |
| E2E-066 | `ai_disabled_2p` | 2P mode | No AI movement |
| E2E-067 | `ai_smooth_motion` | Watch AI 5s | σ(Δy) < jitter_threshold |
| E2E-068 | `ai_anticipation` | High difficulty | Moves before arrival |
| E2E-069 | `ai_recovery` | After goal | AI centers within 2s |
| E2E-070 | `ai_ball_spin_handling` | Spinning ball | Adjusted prediction |
| E2E-071 | `ai_wall_bounce_predict` | Ball near wall | Correct prediction |
| E2E-072 | `ai_paddle_hit_predict` | Ball to opponent | Plans return |
| E2E-073 | `ai_conservative_position` | Ball far | Near center |
| E2E-074 | `ai_aggressive_intercept` | Ball close | Full speed |
| E2E-075 | `ai_frame_rate_independent` | Throttle to 30fps | Same behavior |

#### Game State (Tests 76-90)

| Test ID | Test Name | Playwright Actions | Assertions |
|---------|-----------|-------------------|------------|
| E2E-076 | `state_menu_to_playing` | Press Space | State = Playing |
| E2E-077 | `state_playing_to_paused` | Press Escape | State = Paused |
| E2E-078 | `state_paused_to_playing` | Press Space (when paused) | State = Playing |
| E2E-079 | `state_playing_to_gameover` | Win game (score 11) | State = GameOver |
| E2E-080 | `state_gameover_to_menu` | Press Space (at game over) | State = Menu |
| E2E-081 | `score_left_goal` | Ball exits right | Left score +1 |
| E2E-082 | `score_right_goal` | Ball exits left | Right score +1 |
| E2E-083 | `score_display_update` | Score change | UI matches state |
| E2E-084 | `score_persist_pause` | Pause mid-game | Score unchanged |
| E2E-085 | `rally_counter` | Multiple hits | Rally increases |
| E2E-086 | `rally_reset` | Goal scored | Rally = 0 |
| E2E-087 | `rally_milestone` | Rally = 10 | Milestone event |
| E2E-088 | `game_reset` | New game | All state zeroed |
| E2E-089 | `mode_switch` | Change mode | Correct mode active |
| E2E-090 | `countdown_timer` | Game start | 3-2-1 displayed |

#### Audio/Visual (Tests 91-100)

| Test ID | Test Name | Playwright Actions | Assertions |
|---------|-----------|-------------------|------------|
| E2E-091 | `audio_paddle_hit` | Ball hits paddle | PaddleHit in audio_events |
| E2E-092 | `audio_wall_bounce` | Ball hits wall | WallBounce in audio_events |
| E2E-093 | `audio_goal` | Goal scored | Goal in audio_events |
| E2E-094 | `audio_game_start` | Game starts | GameStart in audio_events |
| E2E-095 | `audio_rally_milestone` | Rally = 10 | RallyMilestone in audio_events |
| E2E-096 | `render_clear` | Each frame | Clear command in output |
| E2E-097 | `render_paddles` | Each frame | 2× FillRect (paddles) |
| E2E-098 | `render_ball` | Each frame | 1× FillCircle |
| E2E-099 | `render_score` | Each frame | 2× FillText (scores) |
| E2E-100 | `render_centerline` | Each frame | Line command |

---

## Quantified Quality Metrics

### Jidoka: Quantifying the Qualitative

All "subjective" outcomes are now mathematically defined:

| Term | Definition | Threshold |
|------|------------|-----------|
| "Smooth motion" | Variance of Δposition over 5-frame window | σ < 2.0 pixels |
| "No jitter" | Standard deviation of AI paddle movement | σ(Δy) < 1.5 pixels/frame |
| "Fast reaction" | AI response time after ball direction change | < 100ms |
| "Slow reaction" | AI response time | > 200ms |
| "Occasional miss" | AI miss rate over 100 rallies | 5% < rate < 50% |
| "Near center" | Distance from screen center | |y - center| < 100 pixels |
| "Full speed" | Paddle velocity | |v| > 0.95 × MAX_SPEED |
| "Steep angle" | Ball bounce angle from horizontal | |θ| > 60° |
| "Flat angle" | Ball bounce angle from horizontal | |θ| < 30° |

---

## Implementation Checklist

### Infrastructure
- [ ] Create `crates/jugar-web/src/simulation.rs` module
- [ ] Implement `MonteCarloConfig` with tier support
- [ ] Implement `FailureReplay` serialization
- [ ] Add `--replay` CLI command
- [ ] Add `window.__JUGAR_DEBUG__` interface to WASM

### Rust Unit Tests
- [ ] Implement `simulation_tests.rs` with 100 tests
- [ ] Implement `metamorphic_tests.rs` with 7 relations
- [ ] Implement `fuzz_tests.rs` with 10 hostile scenarios
- [ ] Add test harness with failure artifact generation

### Playwright E2E Tests
- [ ] Create `examples/pong-web/tests/simulation.spec.ts`
- [ ] Implement all 100 E2E tests
- [ ] Add tier-based seed configuration
- [ ] Add failure screenshot capture

### CI/CD Integration
- [ ] Add `make test-smoke` target (Tier 1)
- [ ] Add `make test-regression` target (Tier 2)
- [ ] Add `make test-monte-carlo` target (Tier 3)
- [ ] Configure GitHub Actions for tiered execution
- [ ] Configure failure artifact upload

### Documentation
- [ ] Document replay artifact format
- [ ] Document metamorphic relations
- [ ] Document fuzzing methodology
- [ ] Update CLAUDE.md with test commands

---

## Appendix A: References

### Core Testing Methodology (1-10)
1. Murphy et al. (2009) - Monte Carlo methods for software testing
2. Neto et al. (2007) - Model-based testing systematic review
3. Kasurinen et al. (2010) - Test automation empirical study
4. Arcuri & Briand (2014) - Statistical tests for randomized algorithms
5. Fraser & Arcuri (2011) - EvoSuite automatic test generation
6. Vos et al. (2012) - AJAX/RIA testing methodology
7. Arbon (2013) - How Google tests software
8. Hamlet (2002) - Random testing foundations
9. Meszaros (2007) - xUnit test patterns
10. Whittaker et al. (2012) - Google test automation maturity model

### Oracle Problem & Formal Verification (11-20)
11. Barr et al. (2015) - The Oracle Problem in Software Testing
12. Luo et al. (2014) - Empirical analysis of flaky tests
13. Yannakakis & Togelius (2015) - AI in Games
14. Godefroid et al. (2008) - Automated Whitebox Fuzz Testing
15. Claessen & Hughes (2000) - QuickCheck property-based testing
16. Wing (1990) - Formal methods introduction
17. Rothermel & Harrold (1996) - Regression test selection
18. Jia & Harman (2011) - Mutation testing survey
19. Haas et al. (2017) - WebAssembly specification
20. Lee & Seshia (2016) - Embedded Systems: Cyber-Physical approach

---

## Appendix B: Makefile Targets

```makefile
# Tiered test targets
.PHONY: test-smoke test-regression test-monte-carlo

test-smoke: ## Tier 1: Pre-commit smoke tests (< 10s)
	TEST_TIER=smoke cargo test --package jugar-web simulation_tests
	cd examples/pong-web && TEST_TIER=smoke npx playwright test

test-regression: ## Tier 2: On-merge regression tests (< 5 min)
	TEST_TIER=regression cargo test --package jugar-web simulation_tests
	cd examples/pong-web && TEST_TIER=regression npx playwright test

test-monte-carlo: ## Tier 3: Nightly Monte Carlo tests (< 30 min)
	TEST_TIER=full cargo test --package jugar-web simulation_tests --release
	cd examples/pong-web && TEST_TIER=full npx playwright test

test-metamorphic: ## Run metamorphic relation tests
	cargo test --package jugar-web metamorphic_tests

test-fuzz: ## Run fuzzing/boundary tests
	cargo test --package jugar-web fuzz_tests

replay: ## Replay a failure artifact
	cargo run --example replay -- $(REPLAY_FILE)
```
