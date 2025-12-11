# WASM Build

Jugar compiles to pure WASM with **ABSOLUTE ZERO JavaScript**. This guide covers building and deploying to the web.

## Build Commands

### Basic WASM Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Using wasm-pack (Recommended for Web)

```bash
wasm-pack build --target web --out-dir pkg
```

### Using Make

```bash
# Build WASM
make build-wasm

# Build for web with wasm-pack
make build-web
```

## Output Verification

Verify your build has zero JavaScript:

```bash
# Should only show .wasm files
ls target/wasm32-unknown-unknown/release/*.wasm

# Verify with make target
make verify-no-js
```

## Web Deployment

### Minimal HTML Loader

Jugar games require a minimal HTML loader that:
- Fetches and instantiates the WASM module
- Forwards browser events to WASM
- Renders output from WASM

**Important**: The HTML loader contains ZERO game logic. All computation happens in WASM.

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>My Jugar Game</title>
    <style>
        body { margin: 0; overflow: hidden; }
        canvas { display: block; }
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init, { WebPlatform } from './pkg/my_game.js';

        async function main() {
            await init();
            const platform = new WebPlatform();

            // Event forwarding only - no game logic
            document.addEventListener('keydown', e => platform.key_down(e.code));
            document.addEventListener('keyup', e => platform.key_up(e.code));

            function frame(timestamp) {
                const commands = platform.frame(timestamp, '[]');
                // Render commands from WASM
                requestAnimationFrame(frame);
            }
            requestAnimationFrame(frame);
        }
        main();
    </script>
</body>
</html>
```

## Local Testing

```bash
# Build and serve
make build-web
make serve-web

# Open http://localhost:8080
```

## Performance Targets

| Metric | Target |
|--------|--------|
| WASM Binary Size | < 2 MiB |
| Gzipped Size | < 500 KB |
| Cold Start | < 100ms |
| Frame Rate | 60 FPS minimum |

## Optimization

### Size Optimization

In `Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
panic = 'abort'
```

### Use wasm-opt

```bash
wasm-opt -Oz -o optimized.wasm input.wasm
```

## Tiered Backend Selection

Jugar automatically selects the best compute backend at runtime:

| Tier | Backend | Capability |
|------|---------|------------|
| 1 | WebGPU compute shaders | 10,000+ rigid bodies |
| 2 | WASM SIMD 128-bit | 1,000+ rigid bodies |
| 3 | Scalar fallback | Basic physics |

Detection happens automatically via `trueno` capability probing.
