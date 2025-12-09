# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `jugar-web` crate for browser integration via Canvas2D JSON commands
- Pong demo game with full game juice effects (screen shake, ball trail, particles)
- trueno SIMD integration for batch physics operations (`simd.rs`)
- WebGPU compute shader demonstration infrastructure (`compute.rs`)
- Adaptive AI opponent with 10 difficulty levels
- Procedural audio system with Web Audio API integration
- Property-based testing with proptest (7 property tests)
- Mutation testing support with cargo-mutants
- Two-phase coverage reporting (bashrs/trueno pattern)
- bashrs Makefile linting integration
- Criterion benchmarks for performance regression testing

### Changed
- Upgraded trueno dependency to 0.8
- Upgraded aprender dependency to 0.16
- Expanded test suite to 312 tests (95% coverage target)

## [0.1.0] - 2024-12-09

### Added
- Initial WASM-native game engine with ABSOLUTE ZERO JavaScript computation
- 9 crates: jugar, jugar-core, jugar-physics, jugar-ai, jugar-render, jugar-ui, jugar-input, jugar-audio, jugar-procgen
- ECS architecture with custom implementation
- Fixed timestep game loop (Heijunka principle)
- Physics system with tiered backends (WebGPU/SIMD/Scalar)
- AI systems: GOAP planner, Behavior Trees, Steering behaviors
- Responsive viewport management (mobile to 32:9 ultrawide)
- Spatial 2D audio with channel mixing
- Procedural generation: noise, dungeons, WFC
- Touch/Mouse/Keyboard/Gamepad input abstraction
- 162 tests with PMAT quality compliance
- WASM build target support (`wasm32-unknown-unknown`)
