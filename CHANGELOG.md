# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial WASM-native game engine implementation
- ECS architecture with hecs
- Fixed timestep game loop (Heijunka principle)
- Physics system with tiered backends (WebGPU/SIMD/Scalar)
- AI systems: GOAP planner, Behavior Trees, Steering behaviors
- Responsive viewport management (mobile to 32:9 ultrawide)
- Spatial 2D audio with channel mixing
- Procedural generation: noise, dungeons, WFC
- Touch/Mouse/Keyboard/Gamepad input abstraction

## [0.1.0] - 2024-12-09

### Added
- Initial release
- 9 crates: jugar, jugar-core, jugar-physics, jugar-ai, jugar-render, jugar-ui, jugar-input, jugar-audio, jugar-procgen
- 162 tests
- PMAT quality compliance
- WASM build target support
