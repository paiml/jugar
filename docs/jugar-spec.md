# Jugar: WASM-Native Universal Game Engine

**Version**: 0.3.0
**Status**: Specification
**Layer**: 7 (Presentation)
**Target**: `wasm32-unknown-unknown` (Pure WASM, **ABSOLUTE ZERO JavaScript**)
**Scalability**: Mobile-First to 49" Ultra-Wide (Universal)

---

## ⚠️ CRITICAL CONSTRAINTS

### ABSOLUTE ZERO JAVASCRIPT POLICY

```
┌────────────────────────────────────────────────────────────────┐
│  ❌ FORBIDDEN                    │  ✅ REQUIRED                │
├──────────────────────────────────┼─────────────────────────────┤
│  • JavaScript files (.js)        │  • Pure Rust only           │
│  • TypeScript files (.ts)        │  • wasm32-unknown-unknown   │
│  • npm/node_modules              │  • wasm-bindgen (Rust-side) │
│  • package.json                  │  • web-sys (Rust bindings)  │
│  • Any JS bundler (webpack, etc) │  • No JS glue code          │
│  • JS interop beyond web-sys     │  • Direct WebAPI calls      │
└──────────────────────────────────┴─────────────────────────────┘
```

**Rationale**: JavaScript introduces non-determinism, GC pauses, and breaks the pure WASM security model. Jugar must compile to a single `.wasm` binary with ZERO JavaScript dependencies.

### BATUTA-FIRST COMPONENT POLICY

**Before importing ANY external crate, first check `../batuta` ecosystem:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    BATUTA SOVEREIGN AI STACK                     │
│                   (USE THESE COMPONENTS FIRST)                   │
├─────────────────────────────────────────────────────────────────┤
│  trueno v0.7+     │  SIMD/GPU compute primitives (MANDATORY)    │
│  aprender v0.14+  │  ML algorithms, behavior trees (MANDATORY)  │
│  trueno-viz       │  WebGPU/WebGL2 rendering                    │
│  presentar-core   │  Platform abstraction, event loop           │
│  alimentar        │  Data loading with encryption               │
│  pacha            │  Asset registry with signatures             │
└─────────────────────────────────────────────────────────────────┘
```

**Dependency Decision Tree**:
```
Need a capability?
    │
    ├─► Does batuta stack have it? ──► YES ──► USE IT (mandatory)
    │                                    │
    │                                    └─► Extend it if needed
    │
    └─► NO ──► Can we build it in pure Rust? ──► YES ──► Build it
                                                  │
                                                  └─► NO ──► REJECT
                                                        (find another way)
```

**NEVER import**:
- `bevy`, `macroquad`, `ggez` (use batuta stack instead)
- `wasm-bindgen-futures` with JS promises (use pure async)
- Any crate requiring JavaScript shims
- Any crate with `wasm32-unknown-emscripten` target

---

## 1. Executive Summary

Jugar (Spanish: "to play") is a universal, WASM-native game engine built on the **Batuta Sovereign AI Stack**. It is designed to be **mobile-first**, prioritizing battery life and touch interfaces, while natively supporting **ultra-wide (32:9) 49-inch monitors** through resolution-independent rendering and reactive UI layouts.

It leverages **GPU WASM** (via WebGPU compute shaders through `trueno`) where possible for physics and AI acceleration, falling back to SIMD-optimized WASM on legacy devices. **All computation happens in pure WASM with zero JavaScript.**

### 1.1 Design Philosophy: The Toyota Way

> "The right process will produce the right results." — Toyota Way

Jugar embodies **Mieruka** (Visual Control) and **Poka-Yoke** (Error Proofing) to ensure a robust, defect-free development environment.

| Principle | Application in Jugar |
|-----------|----------------------|
| **Mieruka (Visual Control)** | Telemetry overlays and visual debugging are enabled by default in dev builds. "No hidden state." |
| **Poka-Yoke (Error Proofing)** | Rust's type system (e.g., `Option<T>`, `Result<T, E>`) prevents null pointer exceptions and unhandled errors at compile time. |
| **Jidoka (Autonomation)** | The engine panics immediately on invalid state detection (fail-fast) with `console_error_panic_hook`, stopping the line to fix the root cause. |
| **Heijunka (Leveling)** | Fixed timestep logic (`fixed_dt`) ensures physics consistency across variable frame rates (30fps mobile to 144Hz+ monitors). |
| **Genchi Genbutsu (Go & See)** | "Go to the source." The `examples/` directory serves as the single source of truth for implementation patterns. |
| **Just-in-Time (JIT)** | Assets are streamed on-demand; render pipelines are compiled lazily to minimize startup time on mobile. |
| **Kaizen (Continuous Improvement)** | Hot-reloadable assets and state serialization allow for rapid iterative loops. |

## 2. Architecture

### 2.1 Pure WASM Stack (ABSOLUTE ZERO JavaScript)

Jugar is built **exclusively** on the **Batuta Sovereign AI Stack** components. No external game engine crates are permitted. All rendering, compute, and AI functionality comes from the batuta ecosystem.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    JUGAR WASM BUNDLE (Single .wasm file)                │
│                         NO JAVASCRIPT WHATSOEVER                        │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Game Loop  │  │  AI Agents  │  │ Render (UI) │  │ Reactive UI │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                │                │            │
│         └────────────────┼────────────────┼────────────────┘            │
│                          │                │                             │
│  ┌───────────────────────▼────────────────▼───────────────────────┐    │
│  │              APRENDER (AI Layer) - ../batuta/aprender           │    │
│  │  Decision Trees · RL Agents · GOAP · Navigation Meshes         │    │
│  │  crates.io: aprender v0.14+  │  Local: ../batuta preferred     │    │
│  └───────────────────────┬────────────────────────────────────────┘    │
│                          │                                             │
│  ┌───────────────────────▼────────────────────────────────────────┐    │
│  │              TRUENO (Compute Layer) - ../batuta/trueno          │    │
│  │  WebGPU Compute Shaders (Primary) · SIMD WASM (Fallback)       │    │
│  │  Physics · Collision · Particle Systems · Fluid Dynamics       │    │
│  │  crates.io: trueno v0.7+  │  Local: ../batuta preferred        │    │
│  └───────────────────────┬────────────────────────────────────────┘    │
│                          │                                             │
│  ┌───────────────────────▼────────────────────────────────────────┐    │
│  │            TRUENO-VIZ (Render Layer) - batuta ecosystem         │    │
│  │  WebGPU/WebGL2 Backend · 32:9 Aspect Ratio Support             │    │
│  │  Resolution Independent Canvas · SDF Text                      │    │
│  └───────────────────────┬────────────────────────────────────────┘    │
│                          │                                             │
│  ┌───────────────────────▼────────────────────────────────────────┐    │
│  │            PRESENTAR-CORE (Platform) - batuta ecosystem         │    │
│  │  Event Loop · Touch/Gamepad/Kbm · Audio · Haptic Feedback      │    │
│  │  Direct web-sys calls · NO JS interop                          │    │
│  └────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────┐
                              │  Browser    │
                              │  WebAPIs    │
                              └──────┬──────┘
                                     │
                              web-sys bindings (Rust)
                                     │
                              NO JAVASCRIPT LAYER
```

### 2.2 Batuta Stack Integration

**Cargo.toml Dependency Strategy**:

```toml
[dependencies]
# MANDATORY: Use local batuta components when developing
# Switch to crates.io versions for releases

# Option 1: Local development (preferred during active development)
trueno = { path = "../batuta/crates/trueno" }
aprender = { path = "../batuta/crates/aprender" }

# Option 2: crates.io release (for published versions)
# trueno = "0.7"
# aprender = "0.14"

# FORBIDDEN: External game engines
# bevy = "..."      # ❌ NEVER
# macroquad = "..." # ❌ NEVER
# ggez = "..."      # ❌ NEVER

# FORBIDDEN: JavaScript-dependent crates
# wasm-bindgen-futures = "..." # ❌ Uses JS promises
# gloo = "..."                 # ❌ JS interop layer
```

**Build Verification**:

```bash
# Verify NO JavaScript in output
wasm-pack build --target web --release
ls pkg/*.js && echo "❌ FAIL: JavaScript detected!" && exit 1
echo "✅ PASS: Pure WASM bundle"
```

### 2.3 Universal Scaling Strategy

Jugar employs a **hybrid scaling model** to support both 6-inch phone screens and 49-inch ultrawide monitors:

1.  **Gameplay Layer**: Uses a **Safe Area** approach. Core gameplay occurs in a 16:9 central zone. Peripheral areas on ultrawide monitors are used for extended visibility (peripheral vision) or non-critical ambiance, ensuring 32:9 users have immersion without gameplay advantage (or disadvantage).
2.  **UI Layer**: Anchored layout system (Top-Left, Bottom-Right, Center, etc.) with responsive scaling. UI elements scale based on the shortest screen dimension (height on landscape) to remain readable on mobile.
3.  **Input Abstraction**: "Tap to Move" (Mobile) maps to "Click to Move" (Desktop) automatically. Virtual joysticks appear only on touch devices.

### 2.4 Crate Structure

```
jugar/
├── Cargo.toml
├── crates/
│   ├── jugar-core/         # ECS, Game Loop, State Management
│   ├── jugar-physics/      # Trueno wrapper (WebGPU/SIMD selector)
│   ├── jugar-ai/           # Aprender wrapper (Behavior Trees, GOAP)
│   ├── jugar-render/       # Trueno-Viz wrapper (Responsive Camera)
│   ├── jugar-ui/           # *NEW* Responsive UI anchoring system
│   ├── jugar-input/        # Unified Input (Touch/Mouse/Gamepad)
│   ├── jugar-audio/        # Spatial Audio
│   ├── jugar-procgen/      # WFC, Noise, Dungeon Gen
│   └── jugar/              # Entry point
└── examples/
    ├── universal_pong/     # Pong scaling from mobile to 32:9
    └── ...
```

## 3. Core ECS (Entity-Component-System)

(See previous version for Entity/Component implementations. Key additions below.)

### 3.1 Components for Universal Design

```rust
// jugar-core/src/components.rs

/// UI Anchor for responsive layout
#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Stretch, // Fills available space
}

/// UI Element component
#[derive(Debug, Clone)]
pub struct UiElement {
    pub anchor: Anchor,
    pub offset: Vec2,
    pub size: Vec2, // In relative units or pixels
    pub scale_mode: ScaleMode, // PixelPerfect or Adaptive
}

/// Camera component with Aspect Ratio handling
#[derive(Debug, Clone)]
pub struct Camera {
    pub zoom: f32,
    pub target_resolution: Option<Vec2>, // For pixel art
    pub keep_aspect: bool,
    pub fov: f32, // Field of View for 3D/Perspective
}
```

## 4. Physics Engine (GPU-Accelerated)

Jugar's physics engine (`jugar-physics`) detects environment capabilities at runtime.

*   **Tier 1 (Desktop/High-End Mobile):** Uses `trueno` WebGPU compute shaders for massive body counts (10,000+ rigid bodies) and fluid simulation.
*   **Tier 2 (Standard Mobile):** Uses `trueno` WASM SIMD (128-bit) implementation.
*   **Tier 3 (Legacy):** Scalar fallback (rarely used).

```rust
// jugar-physics/src/lib.rs

pub enum PhysicsBackend {
    WebGpu,
    WasmSimd,
    Scalar,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        // Genchi Genbutsu: Check actual hardware capabilities
        let backend = if detect_webgpu() {
            PhysicsBackend::WebGpu
        } else {
            PhysicsBackend::WasmSimd
        };
        // ...
    }
}
```

## 5. AI Systems (Aprender-Powered)

Integration of **GOAP (Goal-Oriented Action Planning)** alongside Behavior Trees for more organic agent behavior in open worlds.

```rust
// jugar-ai/src/goap.rs

pub struct Planner {
    actions: Vec<Action>,
    goals: Vec<Goal>,
}

impl Planner {
    pub fn plan(&self, world_state: State) -> Option<Vec<Action>> {
        // A* search through action space to satisfy goal state
        // ...
    }
}
```

## 11. Academic References (Peer-Reviewed)

### Engine Architecture & ECS
1.  **Bilas, S.** (2002). *A Data-Driven Game Object System*. Proceedings of GDC 2002. (Foundational paper on ECS concepts).
2.  **Nystrom, R.** (2014). *Game Programming Patterns*. Genever Benning.
3.  **Fabian, R.** (2018). *Data-Oriented Design*. Richard Fabian. (Essential for cache-coherent ECS).
4.  **Gamma, E., Helm, R., Johnson, R., & Vlissides, J.** (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley.
5.  **Gregory, J.** (2018). *Game Engine Architecture, 3rd Edition*. CRC Press.

### WebAssembly & Performance
6.  **Haas, A., Rossberg, A., Schuff, D. L., Titzer, B. L., Holman, M., Gohman, D., ... & Bastien, J. F.** (2017). *Bringing the Web up to Speed with WebAssembly*. Proceedings of the 38th ACM SIGPLAN Conference on Programming Language Design and Implementation (PLDI).
7.  **Lerner, B. S., et al.** (2020). *WebAssembly: A New Era of Native Code on the Web*. IEEE Security & Privacy.
8.  **Jangda, A., et al.** (2019). *Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code*. USENIX ATC '19.

### Physics & Simulation
9.  **Catto, E.** (2005). *Iterative Dynamics with Temporal Coherence*. Game Developers Conference (GDC).
10. **Ericson, C.** (2004). *Real-Time Collision Detection*. Morgan Kaufmann.
11. **Teschner, M., Heidelberger, B., Müller, M., Pomeranets, D., & Gross, M.** (2003). *Optimized Spatial Hashing for Collision Detection of Deformable Objects*. VMV 2003.
12. **Stam, J.** (1999). *Stable Fluids*. Proceedings of SIGGRAPH 99.
13. **Müller, M., Heidelberger, B., Hennix, M., & Ratcliff, J.** (2007). *Position Based Dynamics*. Journal of Visual Communication and Image Representation.

### Artificial Intelligence
14. **Orkin, J.** (2006). *Three States and a Plan: The AI of F.E.A.R.*. Game Developers Conference (GDC). (GOAP).
15. **Isla, D.** (2005). *Handling Complexity in the Halo 2 AI*. Game Developers Conference (GDC). (Behavior Trees).
16. **Reynolds, C. W.** (1987). *Flocks, Herds and Schools: A Distributed Behavioral Model*. SIGGRAPH '87.
17. **Reynolds, C. W.** (1999). *Steering Behaviors for Autonomous Characters*. Game Developers Conference (GDC).
18. **Hart, P. E., Nilsson, N. J., & Raphael, B.** (1968). *A Formal Basis for the Heuristic Determination of Minimum Cost Paths*. IEEE Transactions on Systems Science and Cybernetics. (A* Algorithm).
19. **Colledanchise, M., & Ögren, P.** (2018). *Behavior Trees in Robotics and AI*. CRC Press.
20. **Millington, I.** (2019). *AI for Games, 3rd Edition*. CRC Press.

### Procedural Generation & Rendering
21. **Perlin, K.** (1985). *An Image Synthesizer*. SIGGRAPH Computer Graphics.
22. **Perlin, K.** (2002). *Improving Noise*. ACM Transactions on Graphics (SIGGRAPH).
23. **Shaker, N., Togelius, J., & Nelson, M. J.** (2016). *Procedural Content Generation in Games*. Springer.
24. **Akenine-Möller, T., Haines, E., & Hoffman, N.** (2018). *Real-Time Rendering, 4th Edition*. AK Peters.
25. **McGuire, M., & Mara, M.** (2011). *The Graphics Codex*. Casual Effects.

## 12. WASM Quality Framework

### 12.1 WASM Quality Gates (Lessons from WOS)

Jugar adopts production-proven WASM quality standards from the WOS (WASM Operating System) project, which achieved perfection status with 670/670 tests passing, 91.2% mutation score, and zero defects.

#### Binary Size Budgets

| Target | Size | Rationale |
|--------|------|-----------|
| **Core WASM** | < 2 MiB | Mobile network budget |
| **Gzipped** | < 500 KB | 3G acceptable load time |
| **No WASI Imports** | 0 | Browser-only, zero dependencies |

```rust
// Cargo.toml workspace lints - enforce lean WASM
[workspace.lints.clippy]
std_instead_of_core = "warn"
std_instead_of_alloc = "warn"

[profile.release-wasm]
opt-level = "z"     # Size optimization
lto = "fat"         # Aggressive LTO
panic = "abort"     # Smaller binary
strip = true        # Strip symbols
```

#### Memory Safety (WOS Standard)

```rust
// All crates MUST enforce safe Rust for educational/production parity
#![forbid(unsafe_code)]

// Exception: jugar-physics may use unsafe for SIMD intrinsics
// with comprehensive safety documentation and MIRI validation
```

### 12.2 SQLite-Inspired Testing Framework

WOS adapted SQLite's legendary 608:1 test-to-code ratio methodology for browser-based WASM. Jugar adopts this four-harness approach:

#### Harness 1: Game Canary Tests (GCT)

**Purpose**: Validate user-facing workflows in production browsers
**Target**: 80% user action coverage

```rust
// Core canary tests for game engine
#[cfg(test)]
mod canary {
    /// C01: Game boots to playable state within 500ms
    #[test]
    fn test_cold_start_within_budget() {
        let start = Instant::now();
        let engine = JugarEngine::new();
        assert!(start.elapsed() < Duration::from_millis(500));
        assert!(engine.is_ready());
    }

    /// C02: Touch input registers within 16ms (single frame)
    #[test]
    fn test_input_latency_single_frame() {
        let mut engine = JugarEngine::new();
        let touch_event = TouchEvent::new(Vec2::new(100.0, 100.0));
        let latency = engine.process_input(touch_event);
        assert!(latency < Duration::from_millis(16));
    }

    /// C03: Physics step completes within frame budget
    #[test]
    fn test_physics_frame_budget() {
        let mut world = PhysicsWorld::new();
        // Add 1000 bodies (mobile target)
        for _ in 0..1000 {
            world.add_body(RigidBody::default());
        }
        let step_time = world.step(1.0 / 60.0);
        assert!(step_time < Duration::from_millis(8)); // Half frame budget
    }
}
```

#### Harness 2: Core Validation Suite (CVS)

**Purpose**: 100% coverage of game engine subsystems
**Inspiration**: SQLite TH3 (100% branch coverage)

| Subsystem | Test Count | Coverage Target |
|-----------|------------|-----------------|
| ECS | 50+ | 100% API |
| Physics | 100+ | All collision types |
| Rendering | 40+ | All pipelines |
| Input | 30+ | All devices |
| Audio | 20+ | Spatial + mixing |
| AI | 60+ | All behaviors |

#### Harness 3: Differential Testing Suite (DTS)

**Purpose**: Compare game engine against reference implementations
**Methodology**: Deterministic replay validation

```rust
/// Differential test: Physics simulation matches reference
#[test]
fn test_physics_determinism() {
    let seed = 42u64;

    // Run simulation with seed
    let result1 = run_physics_simulation(seed, 1000);
    let result2 = run_physics_simulation(seed, 1000);

    // Identical seeds MUST produce identical results
    assert_eq!(result1, result2);
}

/// Differential test: AI behavior is reproducible
#[test]
fn test_ai_determinism() {
    let mut ctx = DeterministicContext::new(42);
    let mut agent = GoapAgent::new(&mut ctx);

    let actions1 = agent.plan(WorldState::test());

    // Reset and replay
    let mut ctx = DeterministicContext::new(42);
    let mut agent = GoapAgent::new(&mut ctx);
    let actions2 = agent.plan(WorldState::test());

    assert_eq!(actions1, actions2);
}
```

#### Harness 4: Chaos Engineering Suite (CES)

**Purpose**: Validate graceful degradation under failure
**Inspiration**: SQLite dbsqlfuzz (1 billion tests/day)

```rust
/// Chaos: Survive WebGPU context loss
#[test]
fn test_webgpu_context_loss_recovery() {
    let mut engine = JugarEngine::new();
    engine.render_frame(); // Normal frame

    // Simulate GPU context loss (tab backgrounded, etc.)
    engine.inject_fault(Fault::GpuContextLost);

    // Engine should fallback gracefully
    assert!(engine.render_frame().is_ok());
    assert_eq!(engine.backend(), PhysicsBackend::WasmSimd);
}

/// Chaos: Memory pressure handling
#[test]
fn test_memory_pressure_response() {
    let mut engine = JugarEngine::new();

    // Fill memory to simulate pressure
    engine.inject_fault(Fault::MemoryPressure { available_mb: 50 });

    // Engine should shed non-critical resources
    assert!(engine.memory_usage_mb() < 50);
    assert!(engine.is_playable()); // Core gameplay intact
}

/// Chaos: localStorage quota exceeded
#[test]
fn test_storage_quota_exceeded() {
    let mut save_system = SaveSystem::new();

    // Inject storage failure
    save_system.inject_fault(Fault::StorageQuotaExceeded);

    // Should warn user, not crash
    let result = save_system.save_game(GameState::test());
    assert!(matches!(result, SaveResult::WarningStorageFull));
}
```

### 12.3 Mutation Testing Lessons

WOS achieved 89.24% mutation score through targeted behavioral testing. Key lessons applied to Jugar:

#### Behavioral Tests > Parser Tests

**Anti-pattern** (from WOS Round 1):
```rust
// BAD: Tests that parsing works, but NOT that behavior is correct
#[test]
fn test_parse_movement_command() {
    let cmd = parse_input("move_left");
    assert_eq!(cmd, Command::MoveLeft);
}
// This test doesn't catch if MoveLeft actually moves the player!
```

**Correct Pattern**:
```rust
// GOOD: Tests actual behavioral outcome
#[test]
fn test_move_left_execution() {
    let mut player = Entity::new();
    player.add(Position(Vec2::new(100.0, 0.0)));

    execute_command(&mut player, Command::MoveLeft);

    assert_eq!(player.get::<Position>().0.x, 90.0); // Actually moved!
}
```

#### Mutation Testing Targets

| Component | Min Score | Focus Areas |
|-----------|-----------|-------------|
| ECS Core | 90% | State transitions, queries |
| Physics | 85% | Boundary conditions, collision |
| AI | 85% | Decision logic, state machines |
| Input | 90% | Event propagation, mapping |
| Overall | 80% | Workspace aggregate |

#### Equivalent Mutants (Acceptable)

Some mutants are functionally equivalent due to control flow. Document and accept:
- Dead code paths protected by early returns
- Boolean logic in unreachable branches
- Arithmetic in clamped value ranges

### 12.4 Anomaly Testing Categories

Adapted from SQLite's OOM testing for game engine context:

| Category | Tests | Examples |
|----------|-------|----------|
| **Memory** | OOM during any operation | Asset loading, entity spawning |
| **GPU** | Context loss, shader compilation failure | WebGPU unavailable, fallback |
| **Storage** | localStorage full, corruption | Save/load, preferences |
| **Timing** | Slow frames, long pauses | Tab backgrounded, GC pressure |
| **Input** | Invalid events, rapid fire | Touch spam, gamepad disconnect |
| **Network** | Timeout, corruption | Asset streaming, multiplayer |

### 12.5 Quality Metrics Dashboard

Real-time quality monitoring during development:

```
┌─────────────────────────────────────────────────────────────┐
│ JUGAR Quality Dashboard - Sprint 14                         │
├─────────────────────────────────────────────────────────────┤
│ WASM Binary: 1.8 MiB / 2.0 MiB budget ✅                   │
│ Gzipped: 412 KB / 500 KB budget ✅                         │
│ WASI Imports: 0 ✅                                         │
├─────────────────────────────────────────────────────────────┤
│ Test Coverage: 96% (target: 95%) ✅                        │
│ Mutation Score: 87% (target: 80%) ✅                       │
│ Canary Tests: 50/50 passing ✅                             │
│ Chaos Tests: 30/30 passing ✅                              │
├─────────────────────────────────────────────────────────────┤
│ Unsafe Code: 0 blocks (forbid enforced) ✅                 │
│ SATD Comments: 0 ✅                                         │
│ Clippy Warnings: 0 ✅                                      │
│ TDG Grade: A+ ✅                                           │
└─────────────────────────────────────────────────────────────┘
```

## 13. Performance & Quality Goals

| Metric | Target | Toyota Principle |
|--------|--------|------------------|
| **Crash Rate** | 0% (in logic) | *Poka-Yoke* (Type safety) |
| **Startup Time** | < 500ms (Mobile) | *Just-in-Time* |
| **Frame Rate** | 60 FPS (Min) | *Heijunka* (Flow leveling) |
| **GC Pauses** | 0ms (No GC) | *Muri* (Overburden prevention) |
| **Resolution** | Responsive | *Mieruka* (Visual clarity everywhere) |
| **WASM Size** | < 2 MiB | *Muda* (Waste elimination) |
| **Cold Start** | < 100ms WASM load | *JIT* (Lazy initialization) |
| **Test Coverage** | ≥ 95% | *Jidoka* (Built-in quality) |
| **Mutation Score** | ≥ 80% | *Kaizen* (Continuous improvement) |

## 14. Tiered Testing Workflow

Adapted from WOS's certeza-inspired workflow:

### Tier 1: ON-SAVE (Sub-second)

```makefile
tier1:  ## Flow-state feedback
	cargo check --quiet
	cargo clippy --lib --quiet -- -D warnings
	cargo test --lib --quiet
```

### Tier 2: ON-COMMIT (1-5 minutes)

```makefile
tier2:  ## Pre-commit validation
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features
	cargo llvm-cov --all-features --fail-under-lines 95
	pmat analyze tdg --min-grade B+
```

### Tier 3: ON-MERGE/NIGHTLY (Hours)

```makefile
tier3:  ## Comprehensive validation
	$(MAKE) tier2
	cargo mutants --timeout 60 --minimum-pass-rate 80
	cargo audit
	wasm-pack build --release --target web
	# Verify WASM size budget
	@WASM_SIZE=$$(stat -c%s pkg/jugar_bg.wasm); \
	if [ "$$WASM_SIZE" -gt 2097152 ]; then \
		echo "❌ WASM exceeds 2MiB budget"; exit 1; \
	fi
```

---

**Document Version**: 0.3.0
**Last Updated**: 2025-12-09
**Authors**: PAIML Team
**WOS Quality Lessons Applied**: PERFECTION-ACHIEVED-2025-10-25