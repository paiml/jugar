# jugar-ui

Widget system for responsive game UI.

## Widgets

```rust
use jugar_ui::prelude::*;

// Text label
let label = Label::new("Score: 100")
    .font_size(24.0)
    .color(Color::WHITE);

// Button
let button = Button::new("Start Game")
    .size(200.0, 50.0)
    .on_click(|| {
        println!("Button clicked!");
    });

// Progress bar
let health_bar = ProgressBar::new(0.75)
    .size(200.0, 20.0)
    .foreground(Color::RED)
    .background(Color::DARK_GRAY);
```

## Layout

```rust
// Horizontal layout
let h_layout = HBox::new()
    .spacing(10.0)
    .add(button1)
    .add(button2)
    .add(button3);

// Vertical layout
let v_layout = VBox::new()
    .spacing(5.0)
    .add(title)
    .add(health_bar)
    .add(score_label);

// Grid layout
let grid = Grid::new(3, 3)
    .cell_size(64.0, 64.0)
    .spacing(4.0);
```

## Panels

```rust
// Container panel
let panel = Panel::new()
    .size(300.0, 200.0)
    .background(Color::rgba(0, 0, 0, 180))
    .padding(10.0)
    .add(v_layout);

// Anchored panel
let hud = Panel::new()
    .anchor(Anchor::TopLeft { margin: 10.0 })
    .add(health_bar)
    .add(score_label);
```

## Responsive Scaling

UI automatically scales based on screen size:

```rust
let ui = UiContext::new(viewport);

// Scale factor based on shortest dimension
let scale = ui.scale_factor();

// Reference resolution scaling
let scaled_size = ui.scale_from_reference(100.0, 1920);
```

## Input Handling

```rust
impl Widget for Button {
    fn handle_input(&mut self, input: &InputState) -> bool {
        let mouse_pos = input.mouse_position();

        if self.bounds.contains(mouse_pos) {
            if input.mouse_just_pressed(MouseButton::Left) {
                self.on_click.call();
                return true;
            }
        }
        false
    }
}
```

## Custom Widgets

```rust
struct MiniMap {
    position: Vec2,
    size: Vec2,
    entities: Vec<MinimapEntity>,
}

impl Widget for MiniMap {
    fn update(&mut self, dt: f32) {
        // Update minimap entities
    }

    fn render(&self) -> Vec<RenderCommand> {
        let mut commands = vec![];
        commands.push(RenderCommand::DrawRect {
            position: self.position,
            size: self.size,
            color: Color::rgba(0, 0, 0, 150),
        });
        // Draw entities as dots
        for entity in &self.entities {
            commands.push(RenderCommand::DrawCircle {
                position: entity.minimap_pos,
                radius: 3.0,
                color: entity.color,
            });
        }
        commands
    }
}
```
