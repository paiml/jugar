# jugar-yaml

ELI5 YAML-First declarative game definitions. Define entire games in YAML.

## Overview

`jugar-yaml` provides a declarative way to define games using YAML configuration files. This follows the "ELI5" (Explain Like I'm 5) philosophy - making game development accessible.

## Basic Structure

```yaml
game:
  title: "My First Game"
  width: 800
  height: 600
  background: "#1a1a2e"

entities:
  - name: player
    position: [400, 300]
    sprite: "player.png"
    components:
      - type: velocity
        value: [0, 0]
      - type: collider
        shape: circle
        radius: 20

  - name: enemy
    position: [100, 100]
    sprite: "enemy.png"
    ai:
      behavior: patrol
      waypoints:
        - [100, 100]
        - [700, 100]
        - [700, 500]
        - [100, 500]
```

## Loading Games

```rust
use jugar_yaml::prelude::*;

// Load from file
let game = GameDefinition::load("game.yaml")?;

// Create engine from definition
let engine = JugarEngine::from_definition(game);
engine.run(game_loop);
```

## Components

Define components declaratively:

```yaml
entities:
  - name: ball
    position: [400, 300]
    components:
      # Physics
      - type: rigid_body
        mass: 1.0
        restitution: 0.9

      # Rendering
      - type: sprite
        texture: "ball.png"
        scale: [1.0, 1.0]

      # Custom data
      - type: custom
        data:
          health: 100
          damage: 10
```

## Behaviors

AI behaviors in YAML:

```yaml
entities:
  - name: guard
    ai:
      behavior: behavior_tree
      tree:
        type: selector
        children:
          - type: sequence
            children:
              - type: condition
                check: "health < 20"
              - type: action
                do: flee

          - type: sequence
            children:
              - type: condition
                check: "sees_player"
              - type: action
                do: attack

          - type: action
            do: patrol
```

## Input Mapping

```yaml
input:
  actions:
    move_up:
      - key: W
      - key: ArrowUp
      - gamepad: LeftStickUp

    move_down:
      - key: S
      - key: ArrowDown
      - gamepad: LeftStickDown

    fire:
      - key: Space
      - mouse: Left
      - gamepad: South
```

## Scenes

Define multiple scenes:

```yaml
scenes:
  main_menu:
    entities:
      - name: title
        position: [400, 200]
        text: "My Game"
        font_size: 48

      - name: start_button
        position: [400, 350]
        button:
          text: "Start"
          action: load_scene:gameplay

  gameplay:
    entities:
      - name: player
        # ...
```

## Templates

Reusable entity templates:

```yaml
templates:
  coin:
    sprite: "coin.png"
    components:
      - type: collider
        shape: circle
        radius: 15
      - type: custom
        data:
          value: 10

entities:
  - template: coin
    position: [100, 200]

  - template: coin
    position: [200, 200]

  - template: coin
    position: [300, 200]
    components:
      - type: custom
        data:
          value: 50  # Override
```

## Physics World

```yaml
physics:
  gravity: [0, 980]
  iterations: 8

  static_bodies:
    - name: ground
      position: [400, 580]
      shape:
        type: box
        size: [800, 40]

    - name: left_wall
      position: [0, 300]
      shape:
        type: box
        size: [20, 600]
```

## Validation

The YAML is validated at load time:

```rust
let result = GameDefinition::load("game.yaml");
match result {
    Ok(game) => { /* valid */ }
    Err(YamlError::InvalidPosition(msg)) => {
        println!("Position error: {}", msg);
    }
    Err(YamlError::MissingField(field)) => {
        println!("Missing required field: {}", field);
    }
    // ...
}
```

## Example: Complete Pong

```yaml
game:
  title: "YAML Pong"
  width: 800
  height: 600

physics:
  gravity: [0, 0]

entities:
  - name: ball
    position: [400, 300]
    components:
      - type: velocity
        value: [300, 200]
      - type: collider
        shape: circle
        radius: 10

  - name: paddle_left
    position: [50, 300]
    components:
      - type: collider
        shape: box
        size: [10, 80]
    input:
      up: W
      down: S

  - name: paddle_right
    position: [750, 300]
    components:
      - type: collider
        shape: box
        size: [10, 80]
    ai:
      behavior: follow_ball
      speed: 200

ui:
  - name: score
    position: [400, 50]
    text: "0 - 0"
    font_size: 32
```
