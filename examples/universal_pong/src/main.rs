//! Universal Pong
//!
//! A Pong game that works responsively from mobile portrait to 32:9 ultrawide.
//! Demonstrates:
//! - Responsive anchor-based UI layout
//! - Touch, mouse, and gamepad input
//! - Physics with velocity components
//! - ECS architecture

use glam::Vec2;
use jugar::prelude::*;

// Game constants (in world units based on 1080p reference)
const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 120.0;
const PADDLE_SPEED: f32 = 500.0;
const BALL_SIZE: f32 = 20.0;
const BALL_SPEED: f32 = 400.0;
const PADDLE_MARGIN: f32 = 50.0;

/// Game-specific components
mod components {
    use glam::Vec2;

    /// Marks an entity as a paddle
    #[derive(Debug, Clone, Copy)]
    pub struct Paddle {
        pub player: u8,
    }

    /// Marks an entity as the ball
    #[derive(Debug, Clone, Copy)]
    pub struct Ball {
        pub velocity: Vec2,
    }

    /// Score component
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Score {
        pub player1: u32,
        pub player2: u32,
    }
}

use components::*;

/// Game state for Pong
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PongState {
    /// Waiting to start
    Ready,
    /// Game in progress
    Playing,
    /// Game paused
    Paused,
    /// Game over
    GameOver,
}

/// Main Pong game
struct PongGame {
    state: PongState,
    paddle1: Entity,
    paddle2: Entity,
    ball: Entity,
    score: Score,
    game_width: f32,
    game_height: f32,
}

impl PongGame {
    fn new(engine: &mut JugarEngine) -> Self {
        // Calculate game bounds based on viewport safe area
        let viewport = engine.viewport();
        let game_width = viewport.safe_area.width;
        let game_height = viewport.safe_area.height;

        let world = engine.world_mut();

        // Create paddle 1 (left)
        let paddle1 = world.spawn();
        world.add_component(
            paddle1,
            Position::new(-game_width / 2.0 + PADDLE_MARGIN, 0.0),
        );
        world.add_component(paddle1, Velocity::zero());
        world.add_component(paddle1, Paddle { player: 1 });

        // Create paddle 2 (right)
        let paddle2 = world.spawn();
        world.add_component(
            paddle2,
            Position::new(game_width / 2.0 - PADDLE_MARGIN, 0.0),
        );
        world.add_component(paddle2, Velocity::zero());
        world.add_component(paddle2, Paddle { player: 2 });

        // Create ball
        let ball = world.spawn();
        world.add_component(ball, Position::zero());
        world.add_component(
            ball,
            Ball {
                velocity: Vec2::new(BALL_SPEED, BALL_SPEED * 0.5),
            },
        );

        Self {
            state: PongState::Ready,
            paddle1,
            paddle2,
            ball,
            score: Score::default(),
            game_width,
            game_height,
        }
    }

    fn update(&mut self, engine: &mut JugarEngine) -> LoopControl {
        let dt = engine.time().delta;

        // Handle escape first
        if engine.input().key(KeyCode::Escape).just_pressed() {
            return LoopControl::Exit;
        }

        match self.state {
            PongState::Ready => {
                if engine.input().key(KeyCode::Space).just_pressed()
                    || engine.input().mouse_button(MouseButton::Left).just_pressed()
                    || !engine.input().touches.is_empty()
                {
                    self.state = PongState::Playing;
                }
            }
            PongState::Playing => {
                self.update_playing(engine, dt);

                if engine.input().key(KeyCode::Letter('P')).just_pressed() {
                    self.state = PongState::Paused;
                }
            }
            PongState::Paused => {
                if engine.input().key(KeyCode::Space).just_pressed()
                    || engine.input().key(KeyCode::Letter('P')).just_pressed()
                {
                    self.state = PongState::Playing;
                }
            }
            PongState::GameOver => {
                if engine.input().key(KeyCode::Space).just_pressed() {
                    self.reset(engine);
                    self.state = PongState::Ready;
                }
            }
        }

        LoopControl::Continue
    }

    fn update_playing(&mut self, engine: &mut JugarEngine, dt: f32) {
        let half_height = self.game_height / 2.0 - PADDLE_HEIGHT / 2.0;
        let viewport_width = engine.viewport().width as f32;
        let viewport_height = engine.viewport().height as f32;

        // Calculate player inputs from input state
        let (p1_vel, p2_vel) = {
            let input = engine.input();

            // Player 1 input (W/S keys or touch left side)
            let mut p1_vel = 0.0;
            if input.key(KeyCode::Letter('W')).is_down() {
                p1_vel = PADDLE_SPEED;
            } else if input.key(KeyCode::Letter('S')).is_down() {
                p1_vel = -PADDLE_SPEED;
            }

            // Touch input for player 1 (left half of screen)
            for touch in &input.touches {
                if touch.position.x < viewport_width / 2.0 {
                    let center_y = viewport_height / 2.0;
                    if touch.position.y < center_y {
                        p1_vel = PADDLE_SPEED;
                    } else {
                        p1_vel = -PADDLE_SPEED;
                    }
                }
            }

            // Player 2 input (Up/Down arrows or touch right side)
            let mut p2_vel = 0.0;
            if input.key(KeyCode::Up).is_down() {
                p2_vel = PADDLE_SPEED;
            } else if input.key(KeyCode::Down).is_down() {
                p2_vel = -PADDLE_SPEED;
            }

            // Touch input for player 2 (right half of screen)
            for touch in &input.touches {
                if touch.position.x >= viewport_width / 2.0 {
                    let center_y = viewport_height / 2.0;
                    if touch.position.y < center_y {
                        p2_vel = PADDLE_SPEED;
                    } else {
                        p2_vel = -PADDLE_SPEED;
                    }
                }
            }

            // Gamepad input for player 1
            for gamepad in &input.gamepads {
                if gamepad.connected {
                    let stick_y = gamepad.left_stick().y;
                    if stick_y.abs() > 0.1 {
                        p1_vel = stick_y * PADDLE_SPEED;
                    }
                }
            }

            (p1_vel, p2_vel)
        };

        // Update paddle positions
        let world = engine.world_mut();

        // Update paddle 1
        if let Some(pos) = world.get_component_mut::<Position>(self.paddle1) {
            pos.y += p1_vel * dt;
            pos.y = pos.y.clamp(-half_height, half_height);
        }

        // Update paddle 2
        if let Some(pos) = world.get_component_mut::<Position>(self.paddle2) {
            pos.y += p2_vel * dt;
            pos.y = pos.y.clamp(-half_height, half_height);
        }

        // Get current positions and ball velocity (copy them to avoid borrow issues)
        let ball_vel = world
            .get_component::<Ball>(self.ball)
            .map(|b| b.velocity)
            .unwrap_or(Vec2::ZERO);

        let ball_pos = world
            .get_component::<Position>(self.ball)
            .map(|p| Vec2::new(p.x, p.y))
            .unwrap_or(Vec2::ZERO);

        let p1_pos = world
            .get_component::<Position>(self.paddle1)
            .map(|p| Vec2::new(p.x, p.y))
            .unwrap_or(Vec2::ZERO);

        let p2_pos = world
            .get_component::<Position>(self.paddle2)
            .map(|p| Vec2::new(p.x, p.y))
            .unwrap_or(Vec2::ZERO);

        // Calculate new ball position
        let mut new_ball_pos = ball_pos + ball_vel * dt;
        let mut new_ball_vel = ball_vel;

        // Bounce off top and bottom
        let ball_half_height = self.game_height / 2.0 - BALL_SIZE / 2.0;
        if new_ball_pos.y > ball_half_height {
            new_ball_pos.y = ball_half_height;
            new_ball_vel.y = -new_ball_vel.y;
        } else if new_ball_pos.y < -ball_half_height {
            new_ball_pos.y = -ball_half_height;
            new_ball_vel.y = -new_ball_vel.y;
        }

        // Paddle 1 collision (left)
        if new_ball_pos.x - BALL_SIZE / 2.0 < p1_pos.x + PADDLE_WIDTH / 2.0
            && new_ball_pos.x > p1_pos.x
            && (new_ball_pos.y - p1_pos.y).abs() < PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0
        {
            new_ball_pos.x = p1_pos.x + PADDLE_WIDTH / 2.0 + BALL_SIZE / 2.0;
            new_ball_vel.x = new_ball_vel.x.abs();
            // Add spin based on where ball hit paddle
            let offset = (new_ball_pos.y - p1_pos.y) / (PADDLE_HEIGHT / 2.0);
            new_ball_vel.y += offset * BALL_SPEED * 0.5;
        }

        // Paddle 2 collision (right)
        if new_ball_pos.x + BALL_SIZE / 2.0 > p2_pos.x - PADDLE_WIDTH / 2.0
            && new_ball_pos.x < p2_pos.x
            && (new_ball_pos.y - p2_pos.y).abs() < PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0
        {
            new_ball_pos.x = p2_pos.x - PADDLE_WIDTH / 2.0 - BALL_SIZE / 2.0;
            new_ball_vel.x = -new_ball_vel.x.abs();
            // Add spin based on where ball hit paddle
            let offset = (new_ball_pos.y - p2_pos.y) / (PADDLE_HEIGHT / 2.0);
            new_ball_vel.y += offset * BALL_SPEED * 0.5;
        }

        // Score detection
        let half_width = self.game_width / 2.0;
        let mut scored = false;
        if new_ball_pos.x < -half_width {
            self.score.player2 += 1;
            scored = true;
        } else if new_ball_pos.x > half_width {
            self.score.player1 += 1;
            scored = true;
        }

        // Apply updates to world
        if scored {
            self.reset_ball(world);
        } else {
            // Update ball position and velocity
            if let Some(pos) = world.get_component_mut::<Position>(self.ball) {
                pos.x = new_ball_pos.x;
                pos.y = new_ball_pos.y;
            }
            if let Some(ball) = world.get_component_mut::<Ball>(self.ball) {
                ball.velocity = new_ball_vel;
            }
        }

        // Check for game over
        if self.score.player1 >= 10 || self.score.player2 >= 10 {
            self.state = PongState::GameOver;
        }
    }

    fn reset_ball(&self, world: &mut World) {
        if let Some(pos) = world.get_component_mut::<Position>(self.ball) {
            pos.x = 0.0;
            pos.y = 0.0;
        }
        if let Some(ball) = world.get_component_mut::<Ball>(self.ball) {
            // Alternate direction based on who scored
            let dir = if self.score.player1 > self.score.player2 {
                1.0
            } else {
                -1.0
            };
            ball.velocity = Vec2::new(BALL_SPEED * dir, BALL_SPEED * 0.3);
        }
    }

    fn reset(&mut self, engine: &mut JugarEngine) {
        self.score = Score::default();
        let world = engine.world_mut();
        self.reset_ball(world);

        // Reset paddle positions
        if let Some(pos) = world.get_component_mut::<Position>(self.paddle1) {
            pos.y = 0.0;
        }
        if let Some(pos) = world.get_component_mut::<Position>(self.paddle2) {
            pos.y = 0.0;
        }
    }

    fn render_info(&self) {
        // In a real implementation, this would render to the screen
        // For now, we just track the state
        match self.state {
            PongState::Ready => {
                println!("Press SPACE to start");
            }
            PongState::Playing => {
                println!(
                    "Score: {} - {} | P1: W/S | P2: Up/Down",
                    self.score.player1, self.score.player2
                );
            }
            PongState::Paused => {
                println!("PAUSED - Press P to resume");
            }
            PongState::GameOver => {
                let winner = if self.score.player1 >= 10 {
                    "Player 1"
                } else {
                    "Player 2"
                };
                println!("GAME OVER - {} wins! Press SPACE to restart", winner);
            }
        }
    }
}

fn main() {
    println!("Universal Pong - Jugar Engine Demo");
    println!("===================================");
    println!("Works on: Mobile Portrait, Mobile Landscape, Desktop, Ultrawide (32:9)");
    println!();
    println!("Controls:");
    println!("  Player 1: W/S keys or touch left side");
    println!("  Player 2: Up/Down arrows or touch right side");
    println!("  Gamepad: Left stick for Player 1");
    println!("  Space: Start/Restart");
    println!("  P: Pause");
    println!("  Escape: Quit");
    println!();

    // Create engine with default 1080p configuration
    let config = JugarConfig::default().with_title("Universal Pong");
    let mut engine = JugarEngine::new(config);

    // Create game
    let mut game = PongGame::new(&mut engine);

    // Print initial state
    game.render_info();

    // Run game loop
    engine.run(|engine| {
        let control = game.update(engine);
        game.render_info();
        control
    });

    println!("\nThanks for playing!");
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let config = JugarConfig::default();
        let mut engine = JugarEngine::new(config);
        let game = PongGame::new(&mut engine);

        assert_eq!(game.state, PongState::Ready);
        assert_eq!(game.score.player1, 0);
        assert_eq!(game.score.player2, 0);
    }

    #[test]
    fn test_paddle_entities_exist() {
        let config = JugarConfig::default();
        let mut engine = JugarEngine::new(config);
        let game = PongGame::new(&mut engine);

        // Check paddles have positions
        assert!(engine
            .world()
            .get_component::<Position>(game.paddle1)
            .is_some());
        assert!(engine
            .world()
            .get_component::<Position>(game.paddle2)
            .is_some());
    }

    #[test]
    fn test_ball_entity_exists() {
        let config = JugarConfig::default();
        let mut engine = JugarEngine::new(config);
        let game = PongGame::new(&mut engine);

        // Check ball has position and ball component
        assert!(engine
            .world()
            .get_component::<Position>(game.ball)
            .is_some());
        assert!(engine.world().get_component::<Ball>(game.ball).is_some());
    }

    #[test]
    fn test_paddle_positions() {
        let config = JugarConfig::default();
        let mut engine = JugarEngine::new(config);
        let game = PongGame::new(&mut engine);

        let p1_pos = engine
            .world()
            .get_component::<Position>(game.paddle1)
            .unwrap();
        let p2_pos = engine
            .world()
            .get_component::<Position>(game.paddle2)
            .unwrap();

        // Paddle 1 should be on the left
        assert!(p1_pos.x < 0.0);
        // Paddle 2 should be on the right
        assert!(p2_pos.x > 0.0);
        // Both should be centered vertically
        assert!((p1_pos.y).abs() < f32::EPSILON);
        assert!((p2_pos.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset_ball() {
        let config = JugarConfig::default();
        let mut engine = JugarEngine::new(config);
        let game = PongGame::new(&mut engine);

        // Move ball
        if let Some(pos) = engine
            .world_mut()
            .get_component_mut::<Position>(game.ball)
        {
            pos.x = 100.0;
            pos.y = 50.0;
        }

        // Reset
        game.reset_ball(engine.world_mut());

        // Check ball is at center
        let ball_pos = engine
            .world()
            .get_component::<Position>(game.ball)
            .unwrap();
        assert!((ball_pos.x).abs() < f32::EPSILON);
        assert!((ball_pos.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_score_component() {
        let score = Score::default();
        assert_eq!(score.player1, 0);
        assert_eq!(score.player2, 0);
    }

    #[test]
    fn test_pong_states() {
        assert_ne!(PongState::Ready, PongState::Playing);
        assert_ne!(PongState::Playing, PongState::Paused);
        assert_ne!(PongState::Paused, PongState::GameOver);
    }
}
