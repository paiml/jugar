# Contributing to Jugar

Thank you for your interest in contributing to Jugar! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- Rust stable (latest)
- WASM target: `rustup target add wasm32-unknown-unknown`
- PMAT tool: `cargo install pmat`

### Development Setup

```bash
# Clone the repository
git clone https://github.com/paiml/jugar.git
cd jugar

# Run tier 1 (sub-second feedback)
make tier1

# Run tier 2 (full validation)
make tier2
```

## Development Workflow

Jugar uses a tiered quality workflow:

### Tier 1: On-Save (Sub-Second Feedback)

```bash
make tier1
```

Runs: `cargo check`, fast clippy subset

### Tier 2: On-Commit (1-5 Minutes)

```bash
make tier2
```

Runs: Full test suite, clippy, formatting, TDG analysis

### Tier 3: On-Merge (10-30 Minutes)

```bash
make tier3
```

Runs: Mutation testing, coverage analysis, WASM build verification

## Quality Standards

All contributions must meet these standards:

| Metric | Minimum | Target |
|--------|---------|--------|
| Test Coverage | 95% | 98% |
| Mutation Score | 80% | 90% |
| TDG Grade | B+ | A- |
| Clippy Warnings | 0 | 0 |

### Running Quality Checks

```bash
# Full PMAT analysis
make pmat-all

# Individual checks
make pmat-tdg           # Technical Debt Grading
make pmat-score         # Repository health score
make pmat-rust-score    # Rust project score
```

## Pull Request Process

1. **Fork** the repository
2. **Create** your feature branch from `main`
3. **Write** tests for your changes (TDD approach)
4. **Ensure** all quality gates pass:
   ```bash
   make tier2
   ```
5. **Commit** with descriptive messages
6. **Push** to your fork
7. **Open** a Pull Request

### Commit Message Format

```
<type>: <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat: add spatial audio distance attenuation

Implements inverse-square law attenuation for 2D spatial audio.
Configurable reference distance and rolloff factor.

Closes #42
```

## Coding Guidelines

### ABSOLUTE ZERO JavaScript

This is non-negotiable. Jugar compiles to pure WASM with no JavaScript dependencies:

- No `.js` or `.ts` files
- No `npm`, `node_modules`, or `package.json`
- No JavaScript bundlers
- Use `web-sys` for browser APIs

### Rust Style

- Use `rustfmt` for formatting
- Follow clippy lints (deny warnings)
- Prefer `Result<T, E>` over panics
- Document public APIs with `///` comments

### Testing

- Write tests before implementation (TDD)
- Aim for behavior tests, not implementation tests
- Use property-based testing where applicable
- Include edge cases and error conditions

### Dependencies

Before adding any dependency, check the Batuta ecosystem:

| Need | Use |
|------|-----|
| SIMD/GPU compute | `trueno` |
| ML/AI | `aprender` |
| Rendering | `trueno-viz` |

**Do NOT import**: `bevy`, `macroquad`, `ggez`, `wasm-bindgen-futures`, `gloo`

## Architecture

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues and discussions first
- Be respectful and constructive
