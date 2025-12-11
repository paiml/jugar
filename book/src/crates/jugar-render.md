# jugar-render

Viewport management and resolution-independent rendering.

## Viewport

```rust
use jugar_render::prelude::*;

let viewport = Viewport::new(1920, 1080);

// Safe area for 16:9 gameplay
let safe_area = viewport.safe_area();

// Extended area for ultrawide
let extended = viewport.extended_area();

// Current aspect ratio
let ratio = viewport.aspect_ratio();
```

## Aspect Ratio Handling

```rust
// Define safe gameplay area
let gameplay_area = viewport.calculate_safe_area(AspectRatio::HD_16_9);

// For 32:9 ultrawide, extends horizontally
// For 9:16 portrait, extends vertically
```

## Anchor System

Position UI elements relative to screen edges:

```rust
use jugar_render::anchor::*;

// Corner anchors
let top_left = Anchor::TopLeft { margin: 10.0 };
let bottom_right = Anchor::BottomRight { margin: 10.0 };

// Edge anchors
let top_center = Anchor::TopCenter { margin: 20.0 };
let left_center = Anchor::LeftCenter { margin: 20.0 };

// Center
let center = Anchor::Center;

// Calculate screen position
let screen_pos = anchor.calculate(viewport.dimensions());
```

## Camera

```rust
use jugar_render::camera::*;

let mut camera = Camera2D::new(viewport.dimensions());

// Follow target
camera.follow(player_position, dt);

// Smooth follow
camera.follow_smooth(player_position, 0.1, dt);

// Zoom
camera.set_zoom(2.0);

// Convert coordinates
let world_pos = camera.screen_to_world(mouse_pos);
let screen_pos = camera.world_to_screen(entity_pos);
```

## Responsive Scaling

```rust
// Scale UI based on shortest dimension
let ui_scale = viewport.ui_scale_factor();

// Scale gameplay based on reference resolution
let game_scale = viewport.game_scale_factor(1920, 1080);
```

## Render Commands

Jugar uses a command-based rendering system:

```rust
pub enum RenderCommand {
    DrawSprite {
        texture: TextureId,
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        color: Color,
    },
    DrawRect {
        position: Vec2,
        size: Vec2,
        color: Color,
    },
    DrawText {
        text: String,
        position: Vec2,
        font_size: f32,
        color: Color,
    },
    // ...
}
```

Commands are serialized to JSON and sent to the HTML renderer.
