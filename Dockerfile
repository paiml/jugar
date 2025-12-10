# Jugar Game Engine - Development Container
# Provides reproducible build environment for WASM compilation

FROM rust:1.83-slim-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack for WASM builds
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install additional Rust targets and tools
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add clippy rustfmt \
    && cargo install cargo-llvm-cov cargo-mutants

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build dependencies first (for caching)
RUN cargo build --release

# Default command runs tests
CMD ["cargo", "test", "--all-features"]
