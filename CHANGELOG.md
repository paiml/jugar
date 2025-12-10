# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-12-10

### Added
- SHAP-style AI explainability widget showing real-time feature importance
- `.apr` AI profile format with downloadable trained models
- Three game modes: SinglePlayer, TwoPlayer, Demo
- Touch and keyboard controls with fullscreen support
- Rally counter and goal sound effects
- Comprehensive Playwright browser tests (38 tests)
- Load testing framework with chaos tests and drift detection

### Fixed
- SHAP widget positioning to not overlap AI difficulty controls
- Sound button position calculation accounting for [I] hint spacing
- AI +/- buttons moved right to not cover difficulty text

### Changed
- Widget title from ".apr SHAP" to ".apr ML Model" for clarity
- Test coverage increased to 676 tests (95% coverage)

## [0.1.0] - 2025-12-09

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
