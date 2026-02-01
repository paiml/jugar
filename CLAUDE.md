# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ CRITICAL CONSTRAINTS

### ABSOLUTE ZERO JAVASCRIPT

```
❌ FORBIDDEN                         ✅ REQUIRED
─────────────────────────────────────────────────────
• JavaScript files (.js/.ts)         • Pure Rust only
• npm/node_modules/package.json      • wasm32-unknown-unknown
• Any JS bundler                     • web-sys (Rust bindings)
• JS interop beyond web-sys          • Single .wasm binary output
```

**Rationale**: JavaScript introduces non-determinism and GC pauses. Jugar compiles to a single `.wasm` file with ZERO JS.

### BATUTA-FIRST COMPONENT POLICY

**Before importing ANY crate, check `../batuta` ecosystem first:**

| Component | Usage | Status |
|-----------|-------|--------|
| `trueno` | SIMD/GPU compute | **MANDATORY** |
| `aprender` | ML/AI algorithms | **MANDATORY** |
| `trueno-viz` | Rendering | Preferred |
| `presentar-core` | Platform abstraction | Preferred |

**NEVER import**: `bevy`, `macroquad`, `ggez`, `wasm-bindgen-futures`, `gloo`

```toml
# Cargo.toml - Local development (preferred)
trueno = { path = "../batuta/crates/trueno" }
aprender = { path = "../batuta/crates/aprender" }

# For release: use crates.io versions
# trueno = "0.7"
# aprender = "0.14"
```

## Project Overview

Jugar (Spanish: "to play") is a WASM-native universal game engine built on the **Batuta Sovereign AI Stack**, targeting `wasm32-unknown-unknown` with **ABSOLUTE ZERO JavaScript**. It supports mobile-first design scaling up to 49" ultra-wide monitors (32:9 aspect ratio).

## Build Commands

```bash
# Tiered workflow (recommended)
make tier1          # ON-SAVE: Sub-second feedback
make tier2          # ON-COMMIT: Full validation (1-5 min)
make tier3          # ON-MERGE: Mutation testing + benchmarks

# Build
make build          # Host target (dev)
make build-wasm     # WASM target (release)

# Test
make test           # All tests
make test-fast      # Library tests only

# Quality
make lint           # Full clippy
make fmt            # Format code
make coverage       # Generate coverage report
```

## PMAT Quality Gates

This project uses PMAT (PAIML MCP Agent Toolkit) for EXTREME TDD:

```bash
# Quality checks
make pmat-tdg              # Technical Debt Grading (min: B+)
make pmat-score            # Repository health score (min: 90)
make pmat-rust-score       # Rust project score (min: 150/211)
make pmat-validate-docs    # Documentation validation
make pmat-all              # Run all PMAT checks

# Continuous improvement
make kaizen                # Kaizen analysis cycle
make mutate                # Mutation testing (≥80% kill rate)
```

### Quality Thresholds

| Metric | Minimum | Target |
|--------|---------|--------|
| Test Coverage | 95% | 98% |
| Mutation Score | 80% | 90% |
| TDG Grade | B+ | A- |
| Repo Score | 90/110 | 100/110 |
| WASM Size | - | < 2MB |

## Architecture

### Layer Stack (Bottom to Top)

1. **Presentar-Core** - Platform abstraction (event loop, input, audio, haptics)
2. **Trueno** - Compute layer with WebGPU compute shaders (primary) and WASM SIMD fallback
3. **Trueno-Viz** - Render layer (WebGPU/WebGL2, SDF text, resolution-independent canvas)
4. **Aprender** - AI layer (behavior trees, GOAP, RL agents, navigation meshes)
5. **Jugar** - Game engine (ECS, game loop, physics, UI, procedural generation)

### Crate Structure

```
crates/
├── jugar-core/      # ECS, Game Loop, State Management
├── jugar-physics/   # Trueno wrapper (WebGPU/SIMD runtime selection)
├── jugar-ai/        # Aprender wrapper (Behavior Trees, GOAP)
├── jugar-render/    # Trueno-Viz wrapper (Responsive Camera)
├── jugar-ui/        # Responsive UI anchoring system
├── jugar-input/     # Unified Input (Touch/Mouse/Gamepad)
├── jugar-audio/     # Spatial Audio
├── jugar-procgen/   # WFC, Noise, Dungeon Generation
└── jugar/           # Entry point crate
```

### Physics Backend Selection

Runtime capability detection selects backend:
- **Tier 1**: WebGPU compute shaders (10,000+ rigid bodies)
- **Tier 2**: WASM SIMD 128-bit
- **Tier 3**: Scalar fallback

### Universal Scaling Model

- **Gameplay Layer**: 16:9 safe area with peripheral extension for ultrawide
- **UI Layer**: Anchor-based responsive layout scaling on shortest dimension
- **Input**: Touch/Click abstraction with virtual joysticks on touch devices only

## Design Principles (Toyota Way)

- **Mieruka**: Visual debugging enabled by default in dev builds
- **Poka-Yoke**: Rust type system prevents invalid states at compile time
- **Jidoka**: Fail-fast with `console_error_panic_hook` on invalid state
- **Heijunka**: Fixed timestep (`fixed_dt`) for physics consistency across frame rates
- **Genchi Genbutsu**: `examples/` directory is the source of truth for patterns
- **JIT**: Lazy asset streaming and pipeline compilation for mobile startup
- **Kaizen**: Hot-reloadable assets and state serialization

## Performance Targets

| Metric | Target |
|--------|--------|
| Crash Rate | 0% (type safety) |
| Startup Time | < 500ms (Mobile) |
| Frame Rate | 60 FPS minimum |
| GC Pauses | 0ms (No GC) |
| WASM Binary | < 2 MiB |
| Gzipped | < 500 KB |
| Cold Start | < 100ms WASM load |

## WASM Quality Framework (WOS Lessons)

This project applies production-proven WASM quality standards from the WOS project.

### Four-Harness Testing (SQLite-Inspired)

1. **Game Canary Tests (GCT)**: 80% user action coverage in production browsers
2. **Core Validation Suite (CVS)**: 100% coverage of engine subsystems
3. **Differential Testing Suite (DTS)**: Deterministic replay validation
4. **Chaos Engineering Suite (CES)**: Graceful degradation under failure

### Key Lessons

- **Behavioral Tests > Parser Tests**: Test actual outcomes, not just parsing
- **100% Safe Rust**: `#![forbid(unsafe_code)]` enforced (except physics SIMD)
- **No WASI Imports**: Browser-only, zero external dependencies
- **Equivalent Mutants**: Document and accept dead-code path mutations

### Anomaly Testing Categories

| Category | Examples |
|----------|----------|
| Memory | OOM during asset loading |
| GPU | WebGPU context loss recovery |
| Storage | localStorage quota exceeded |
| Timing | Tab backgrounded handling |
| Input | Gamepad disconnect |

See `docs/jugar-spec.md` Section 12 for complete WASM Quality Framework.


## Stack Documentation Search

**IMPORTANT: Proactively use the batuta RAG oracle when:**
- Looking up patterns from other stack components (trueno SIMD, aprender ML, realizar inference)
- Finding ECS, physics, and rendering patterns from ground truth corpora
- Understanding WASM-native build patterns across the stack
- Researching game AI, input handling, and audio integration approaches

```bash
# Index all stack documentation (run once, persists to ~/.cache/batuta/rag/)
batuta oracle --rag-index

# Search across the entire stack
batuta oracle --rag "your question here"

# Jugar-specific examples
batuta oracle --rag "ECS entity component system architecture"
batuta oracle --rag "WASM-native zero JavaScript patterns"
batuta oracle --rag "physics collision detection broad phase"
batuta oracle --rag "game AI behavior tree implementation"
batuta oracle --rag "GPU wgpu render pipeline setup"
```

The RAG index (341+ docs) includes CLAUDE.md, README.md, and source files from all stack components plus Python ground truth corpora for cross-language pattern matching.
