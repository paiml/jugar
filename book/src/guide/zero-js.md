# Zero JavaScript Policy

Jugar enforces **ABSOLUTE ZERO JavaScript** in all game computation. This is a critical architectural constraint.

## Why Zero JavaScript?

| Problem | JavaScript | Pure WASM |
|---------|-----------|-----------|
| **Determinism** | Non-deterministic | Fully deterministic |
| **GC Pauses** | Unpredictable pauses | No garbage collector |
| **Security** | Large attack surface | Sandboxed execution |
| **Performance** | JIT variability | Predictable performance |
| **Replay** | Difficult | Frame-perfect replay |

## What's Forbidden

```
❌ FORBIDDEN                         ✅ REQUIRED
─────────────────────────────────────────────────────
• JavaScript files (.js/.ts)         • Pure Rust only
• npm/node_modules/package.json      • wasm32-unknown-unknown
• Any JS bundler                     • web-sys (Rust bindings)
• JS interop beyond web-sys          • Single .wasm binary output
• wasm-bindgen-futures               • Pure async
• gloo, bevy, macroquad, ggez        • Batuta stack components
```

## What's Allowed

The **only** JavaScript permitted is a minimal HTML loader that:

1. Fetches and instantiates the WASM module
2. Forwards browser events to WASM (keydown, mouse, touch)
3. Renders output commands from WASM

**Zero game logic in JavaScript.** All computation happens in WASM.

## Verification

### Automated Check

```bash
make verify-no-js
```

This checks for:
- Standalone .js files (excluding wasm-pack output)
- .ts files (excluding .d.ts type definitions)
- package.json in project root
- node_modules directory
- Forbidden crates in Cargo.toml

### Manual Verification

```bash
# Check for JavaScript files
find . -name "*.js" -not -path "./target/*" -not -path "*/pkg/*"

# Check for TypeScript files
find . -name "*.ts" -not -name "*.d.ts" -not -path "./target/*"

# Check for npm artifacts
ls package.json node_modules 2>/dev/null
```

## Forbidden Crates

These crates violate the zero-JS policy:

| Crate | Reason |
|-------|--------|
| `wasm-bindgen-futures` | Relies on JS promises |
| `gloo` | JavaScript wrapper library |
| `bevy` | Large with JS dependencies |
| `macroquad` | JavaScript glue required |
| `ggez` | Not pure WASM |

## Use Batuta Stack Instead

All functionality comes from the Batuta ecosystem:

| Need | Use |
|------|-----|
| SIMD/GPU compute | `trueno` |
| ML/AI algorithms | `aprender` |
| Rendering | `trueno-viz` |
| Platform abstraction | `presentar-core` |
| Data loading | `alimentar` |
| Asset registry | `pacha` |

## Benefits

1. **Deterministic Replay**: Record inputs, replay exactly
2. **Testing**: Probar can test without a browser
3. **Security**: No JavaScript attack surface
4. **Performance**: Predictable, no GC pauses
5. **Size**: Single .wasm file < 2MB
