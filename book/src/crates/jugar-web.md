# jugar-web

Web platform bindings for running Jugar games in the browser.

## Overview

`jugar-web` provides the WASM bindings for running Jugar games in web browsers. It handles:

- WASM module initialization
- Event forwarding from browser to game
- Render command output to JavaScript
- WebAudio integration
- Touch/mouse/keyboard input

## WebPlatform

The main interface exposed to JavaScript:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebPlatform {
    game: GameState,
    config: WebConfig,
}

#[wasm_bindgen]
impl WebPlatform {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        // Initialize platform
    }

    pub fn frame(&mut self, timestamp: f64, input_json: &str) -> String {
        // Process frame, return render commands as JSON
    }

    pub fn key_down(&mut self, code: &str) {
        // Handle key press
    }

    pub fn key_up(&mut self, code: &str) {
        // Handle key release
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        // Handle mouse movement
    }

    pub fn touch_start(&mut self, id: u32, x: f32, y: f32) {
        // Handle touch start
    }
}
```

## Building

```bash
# Build with wasm-pack
wasm-pack build crates/jugar-web --target web --out-dir pkg

# Or use make
make build-web
```

## HTML Integration

Minimal loader (no game logic in JS):

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Jugar Game</title>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init, { WebPlatform } from './pkg/jugar_web.js';

        async function main() {
            await init();

            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');
            const platform = new WebPlatform(800, 600);

            // Event forwarding only
            document.addEventListener('keydown', e => {
                platform.key_down(e.code);
            });

            document.addEventListener('keyup', e => {
                platform.key_up(e.code);
            });

            canvas.addEventListener('mousemove', e => {
                platform.mouse_move(e.offsetX, e.offsetY);
            });

            function frame(timestamp) {
                const commands = platform.frame(timestamp, '[]');
                render(ctx, JSON.parse(commands));
                requestAnimationFrame(frame);
            }

            requestAnimationFrame(frame);
        }

        function render(ctx, commands) {
            for (const cmd of commands) {
                switch (cmd.type) {
                    case 'clear':
                        ctx.fillStyle = cmd.color;
                        ctx.fillRect(0, 0, 800, 600);
                        break;
                    case 'rect':
                        ctx.fillStyle = cmd.color;
                        ctx.fillRect(cmd.x, cmd.y, cmd.w, cmd.h);
                        break;
                    case 'text':
                        ctx.fillStyle = cmd.color;
                        ctx.font = `${cmd.size}px sans-serif`;
                        ctx.fillText(cmd.text, cmd.x, cmd.y);
                        break;
                }
            }
        }

        main();
    </script>
</body>
</html>
```

## Render Commands

JSON format for render commands:

```json
[
    {"type": "clear", "color": "#1a1a2e"},
    {"type": "rect", "x": 100, "y": 200, "w": 50, "h": 50, "color": "#ffffff"},
    {"type": "text", "x": 400, "y": 50, "text": "Score: 100", "size": 24, "color": "#ffffff"},
    {"type": "circle", "x": 300, "y": 300, "r": 10, "color": "#ff0000"}
]
```

## Audio Commands

```json
[
    {"type": "playTone", "frequency": 440, "duration": 0.1},
    {"type": "playSound", "id": "explosion", "volume": 0.8}
]
```

## Testing

E2E tests using Probar:

```bash
# Run Probar tests (replaces Playwright)
make test-e2e

# Or directly
cargo test -p jugar-web --test probar_pong
```

## Configuration

```rust
pub struct WebConfig {
    pub width: u32,
    pub height: u32,
    pub canvas_id: String,
    pub audio_enabled: bool,
    pub touch_enabled: bool,
}
```

## Feature Flags

```toml
[features]
default = ["audio"]
audio = []  # Enable WebAudio support
debug = []  # Enable debug overlays
```
