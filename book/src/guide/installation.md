# Installation

## Requirements

- **Rust**: 1.70 or later
- **Target**: `wasm32-unknown-unknown`

## Setup

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Add WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Install Development Tools (Optional)

```bash
# For web builds
cargo install wasm-pack

# For quality tooling
cargo install cargo-llvm-cov cargo-mutants cargo-nextest

# Install all tools at once
make install-tools
```

## Adding Jugar to Your Project

### From crates.io

```toml
[dependencies]
jugar = "0.1"
```

### From Git (Latest)

```toml
[dependencies]
jugar = { git = "https://github.com/paiml/jugar" }
```

### Local Development

If you're contributing or developing locally:

```toml
[dependencies]
jugar = { path = "../jugar/crates/jugar" }
```

## Feature Flags

Jugar supports optional features:

```toml
[dependencies]
jugar = { version = "0.1", features = ["full"] }
```

| Feature | Description |
|---------|-------------|
| `default` | Core engine functionality |
| `ai` | GOAP and Behavior Trees |
| `audio` | Spatial 2D audio |
| `procgen` | Procedural generation |
| `full` | All features |

## Verify Installation

Create a test project:

```bash
cargo new jugar-test
cd jugar-test
echo 'jugar = "0.1"' >> Cargo.toml
cargo build --target wasm32-unknown-unknown
```

If the build succeeds with no JavaScript output, you're ready to go!
