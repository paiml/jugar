# Universal Pong

A responsive Pong implementation that works from mobile to 32:9 ultrawide.

## Features

- Touch controls (mobile)
- Keyboard (W/S, Up/Down)
- Gamepad support
- Responsive paddle positioning
- AI opponent with Dynamic Difficulty Adjustment
- SHAP-like explainability widgets

## Running

```bash
# Build WASM
make build-web

# Serve locally
make serve-web

# Open http://localhost:8080
```

## Controls

| Input | Player 1 | Player 2 |
|-------|----------|----------|
| Keyboard | W/S | Up/Down |
| Touch | Left side | Right side |
| Gamepad | Left stick | Right stick |

## Architecture

### Game State

```rust
pub struct PongState {
    pub ball: Ball,
    pub paddle_left: Paddle,
    pub paddle_right: Paddle,
    pub score_left: u32,
    pub score_right: u32,
    pub game_mode: GameMode,
}
```

### Ball Physics

```rust
impl Ball {
    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;

        // Wall bounce
        if self.position.y <= 0.0 || self.position.y >= 600.0 {
            self.velocity.y = -self.velocity.y;
        }
    }

    pub fn check_paddle_collision(&mut self, paddle: &Paddle) {
        if self.bounds().intersects(paddle.bounds()) {
            self.velocity.x = -self.velocity.x;
            // Add spin based on hit position
            let offset = (self.position.y - paddle.position.y) / paddle.height;
            self.velocity.y += offset * 200.0;
        }
    }
}
```

### AI Opponent

The AI uses a trained model (.apr format) with Dynamic Difficulty Adjustment:

```rust
pub struct PongAI {
    model: AprModel,
    difficulty: f32,  // 0.0 - 1.0
}

impl PongAI {
    pub fn update(&mut self, state: &PongState) -> f32 {
        // Predict optimal position
        let optimal = self.model.predict(state);

        // Add error based on difficulty
        let error = (1.0 - self.difficulty) * random_offset();

        optimal + error
    }
}
```

## Responsive Design

### Viewport Scaling

```rust
// Safe area for 16:9 gameplay
let safe = viewport.safe_area();

// Extended for ultrawide
let extended = viewport.extended_area();

// Position paddles at edges
let left_paddle_x = safe.left + PADDLE_MARGIN;
let right_paddle_x = safe.right - PADDLE_MARGIN;
```

### Touch Zones

```rust
// Left half: Player 1
// Right half: Player 2
fn get_touch_player(touch_x: f32, screen_width: f32) -> Player {
    if touch_x < screen_width / 2.0 {
        Player::Left
    } else {
        Player::Right
    }
}
```

## Testing

```bash
# Run Probar E2E tests
make test-e2e

# Run with verbose output
make test-e2e-verbose
```

### Test Suites

| Suite | Tests | Coverage |
|-------|-------|----------|
| Core Functionality | 6 | WASM loading, rendering |
| Demo Features | 22 | Game modes, HUD, AI |
| Release Readiness | 11 | Stress tests, edge cases |

## Explainability Widgets

The AI's decision-making is visualized:

```rust
pub struct ShapWidget {
    features: Vec<FeatureContribution>,
}

impl ShapWidget {
    pub fn render(&self) -> Vec<RenderCommand> {
        // Show feature contributions as bars
        self.features.iter().map(|f| {
            RenderCommand::DrawRect {
                width: f.contribution.abs() * 100.0,
                color: if f.contribution > 0.0 { GREEN } else { RED },
                // ...
            }
        }).collect()
    }
}
```

## Source Code

- `crates/jugar-web/src/demo.rs` - Pong demo implementation
- `crates/jugar-web/src/ai.rs` - AI opponent
- `examples/pong-web/index.html` - HTML loader
