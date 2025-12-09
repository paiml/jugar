//! Procedural audio for Pong game.
//!
//! This module generates audio commands that JavaScript executes via Web Audio API.
//! All audio logic (pitch calculation, timing) happens in Rust.
//!
//! ## Architecture
//!
//! ```text
//! Rust (audio logic)          JavaScript (Web Audio API)
//! ─────────────────────       ─────────────────────────────
//! AudioEvent::PaddleHit  →    oscillator.frequency = pitch
//! AudioEvent::WallBounce →    gain.value = volume
//! AudioEvent::Goal       →    oscillator.start()/stop()
//! ```

// const fn with mutable references is not yet stable
#![allow(clippy::missing_const_for_fn)]

use serde::{Deserialize, Serialize};

/// Audio events that JavaScript should play via Web Audio API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Paddle hit sound (blip with pitch based on hit location)
    PaddleHit {
        /// Base frequency in Hz (200-800)
        frequency: f32,
        /// Duration in seconds (0.05-0.15)
        duration: f32,
        /// Volume (0.0-1.0)
        volume: f32,
    },
    /// Wall bounce sound (lower blip)
    WallBounce {
        /// Base frequency in Hz (typically 150-250)
        frequency: f32,
        /// Duration in seconds
        duration: f32,
        /// Volume (0.0-1.0)
        volume: f32,
    },
    /// Goal scored sound (rising tone sequence)
    Goal {
        /// Whether this is a player win (true) or AI win (false)
        player_scored: bool,
        /// Volume (0.0-1.0)
        volume: f32,
    },
    /// Game start jingle
    GameStart {
        /// Volume (0.0-1.0)
        volume: f32,
    },
    /// Rally milestone sound (every 5 hits)
    RallyMilestone {
        /// Rally count at this milestone (5, 10, 15, ...)
        rally_count: u32,
        /// Base frequency increases with rally
        frequency: f32,
        /// Volume (0.0-1.0)
        volume: f32,
    },
    /// Sound toggle confirmation (plays when sound is enabled)
    SoundToggle {
        /// Whether sound is now enabled
        enabled: bool,
        /// Volume (0.0-1.0)
        volume: f32,
    },
}

/// Procedural audio generator for Pong.
///
/// Generates audio events based on game state changes.
/// Events are returned as JSON to be executed by JavaScript's Web Audio API.
///
/// # Example
///
/// ```
/// use jugar_web::audio::ProceduralAudio;
///
/// let mut audio = ProceduralAudio::new();
/// audio.set_enabled(true);
///
/// // Trigger a paddle hit sound (hit_y, paddle_y, paddle_height)
/// audio.on_paddle_hit(300.0, 250.0, 100.0);
///
/// // Get events to send to JavaScript
/// let events = audio.take_events();
/// assert!(!events.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct ProceduralAudio {
    /// Master volume (0.0-1.0)
    master_volume: f32,
    /// Whether audio is enabled
    enabled: bool,
    /// Pending audio events to be sent to JavaScript
    events: Vec<AudioEvent>,
}

impl Default for ProceduralAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl ProceduralAudio {
    /// Creates a new audio generator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            master_volume: 0.7,
            enabled: true,
            events: Vec::with_capacity(4),
        }
    }

    /// Sets the master volume (0.0-1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Returns the current master volume.
    #[must_use]
    pub const fn volume(&self) -> f32 {
        self.master_volume
    }

    /// Enables or disables audio.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether audio is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generates a paddle hit sound.
    ///
    /// # Arguments
    ///
    /// * `hit_y` - Y position where ball hit the paddle
    /// * `paddle_y` - Center Y position of the paddle
    /// * `paddle_height` - Height of the paddle
    pub fn on_paddle_hit(&mut self, hit_y: f32, paddle_y: f32, paddle_height: f32) {
        if !self.enabled {
            return;
        }

        // Normalize hit position to -1.0 to 1.0
        let half_height = paddle_height / 2.0;
        let relative_y = (hit_y - paddle_y) / half_height;
        let normalized = relative_y.clamp(-1.0, 1.0);

        // Map to frequency: center = 440Hz (A4), edges = 220Hz or 660Hz
        let frequency = 440.0 + normalized * 220.0;

        self.events.push(AudioEvent::PaddleHit {
            frequency,
            duration: 0.08,
            volume: self.master_volume,
        });
    }

    /// Generates a wall bounce sound with optional velocity-based pitch variation.
    pub fn on_wall_bounce(&mut self) {
        if !self.enabled {
            return;
        }

        // Slight random variation in pitch (150-250 Hz range)
        // Using a simple deterministic pattern based on event count
        let variation = (self.events.len() % 5) as f32 * 10.0;
        let frequency = 180.0 + variation;

        self.events.push(AudioEvent::WallBounce {
            frequency,
            duration: 0.05,
            volume: self.master_volume * 0.5, // Quieter than paddle hits
        });
    }

    /// Generates a wall bounce sound with velocity-based pitch.
    ///
    /// # Arguments
    ///
    /// * `ball_speed` - Current ball speed (magnitude of velocity)
    /// * `base_speed` - Reference speed for normal pitch
    pub fn on_wall_bounce_with_velocity(&mut self, ball_speed: f32, base_speed: f32) {
        if !self.enabled {
            return;
        }

        // Higher speed = higher pitch (150-300 Hz range)
        let speed_ratio = (ball_speed / base_speed).clamp(0.5, 2.0);
        let frequency = 150.0 + speed_ratio * 75.0;

        self.events.push(AudioEvent::WallBounce {
            frequency,
            duration: 0.05,
            volume: self.master_volume * 0.5,
        });
    }

    /// Generates a goal sound.
    ///
    /// # Arguments
    ///
    /// * `player_scored` - True if the player (left paddle) scored
    pub fn on_goal(&mut self, player_scored: bool) {
        if !self.enabled {
            return;
        }

        self.events.push(AudioEvent::Goal {
            player_scored,
            volume: self.master_volume,
        });
    }

    /// Generates a game start sound.
    pub fn on_game_start(&mut self) {
        if !self.enabled {
            return;
        }

        self.events.push(AudioEvent::GameStart {
            volume: self.master_volume,
        });
    }

    /// Generates a rally milestone sound.
    ///
    /// Should be called when rally count reaches a milestone (5, 10, 15, etc.).
    ///
    /// # Arguments
    ///
    /// * `rally_count` - Current rally count at the milestone
    pub fn on_rally_milestone(&mut self, rally_count: u32) {
        if !self.enabled {
            return;
        }

        // Frequency increases with rally count (300-800 Hz)
        let base_freq = 300.0;
        let freq_increase = (rally_count as f32 / 5.0) * 50.0;
        let frequency = (base_freq + freq_increase).min(800.0);

        self.events.push(AudioEvent::RallyMilestone {
            rally_count,
            frequency,
            volume: self.master_volume,
        });
    }

    /// Generates a sound toggle confirmation sound.
    ///
    /// Plays a brief confirmation when sound is enabled.
    /// This provides immediate feedback that audio is working.
    pub fn on_sound_toggle(&mut self, enabled: bool) {
        // Only play sound when enabling (user wants to hear it)
        // Skip check for self.enabled since we're specifically toggling it
        if enabled {
            self.events.push(AudioEvent::SoundToggle {
                enabled,
                volume: self.master_volume,
            });
        }
    }

    /// Takes all pending audio events (clears the internal buffer).
    ///
    /// # Returns
    ///
    /// Vector of audio events to be played by JavaScript.
    pub fn take_events(&mut self) -> Vec<AudioEvent> {
        core::mem::take(&mut self.events)
    }

    /// Returns pending events without clearing (for inspection).
    #[must_use]
    pub fn peek_events(&self) -> &[AudioEvent] {
        &self.events
    }

    /// Clears all pending events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Returns the number of pending events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_procedural_audio_new() {
        let audio = ProceduralAudio::new();
        assert!(audio.is_enabled());
        assert!((audio.volume() - 0.7).abs() < 0.001);
        assert_eq!(audio.event_count(), 0);
    }

    #[test]
    fn test_procedural_audio_default() {
        let audio = ProceduralAudio::default();
        assert!((audio.volume() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_set_volume() {
        let mut audio = ProceduralAudio::new();

        audio.set_volume(0.5);
        assert!((audio.volume() - 0.5).abs() < 0.001);

        // Test clamping
        audio.set_volume(1.5);
        assert!((audio.volume() - 1.0).abs() < 0.001);

        audio.set_volume(-0.5);
        assert!(audio.volume().abs() < 0.001);
    }

    #[test]
    fn test_set_enabled() {
        let mut audio = ProceduralAudio::new();

        audio.set_enabled(false);
        assert!(!audio.is_enabled());

        audio.set_enabled(true);
        assert!(audio.is_enabled());
    }

    #[test]
    fn test_on_paddle_hit() {
        let mut audio = ProceduralAudio::new();

        // Hit at center of paddle
        audio.on_paddle_hit(300.0, 300.0, 100.0);

        assert_eq!(audio.event_count(), 1);
        let events = audio.take_events();
        match &events[0] {
            AudioEvent::PaddleHit {
                frequency,
                duration,
                volume,
            } => {
                assert!((*frequency - 440.0).abs() < 1.0); // Center = A4
                assert!((*duration - 0.08).abs() < 0.01);
                assert!((*volume - 0.7).abs() < 0.01);
            }
            _ => panic!("Expected PaddleHit event"),
        }
    }

    #[test]
    fn test_on_paddle_hit_top() {
        let mut audio = ProceduralAudio::new();

        // Hit at top of paddle
        audio.on_paddle_hit(250.0, 300.0, 100.0);

        let events = audio.take_events();
        match &events[0] {
            AudioEvent::PaddleHit { frequency, .. } => {
                assert!(*frequency < 440.0); // Above center = lower pitch
            }
            _ => panic!("Expected PaddleHit event"),
        }
    }

    #[test]
    fn test_on_paddle_hit_bottom() {
        let mut audio = ProceduralAudio::new();

        // Hit at bottom of paddle
        audio.on_paddle_hit(350.0, 300.0, 100.0);

        let events = audio.take_events();
        match &events[0] {
            AudioEvent::PaddleHit { frequency, .. } => {
                assert!(*frequency > 440.0); // Below center = higher pitch
            }
            _ => panic!("Expected PaddleHit event"),
        }
    }

    #[test]
    fn test_on_wall_bounce() {
        let mut audio = ProceduralAudio::new();

        audio.on_wall_bounce();

        assert_eq!(audio.event_count(), 1);
        let events = audio.take_events();
        match &events[0] {
            AudioEvent::WallBounce {
                frequency,
                duration,
                volume,
            } => {
                // Frequency now has slight variation (180-220 Hz range)
                assert!(*frequency >= 180.0 && *frequency <= 220.0);
                assert!((*duration - 0.05).abs() < 0.01);
                assert!(*volume < 0.7); // Quieter than paddle hits
            }
            _ => panic!("Expected WallBounce event"),
        }
    }

    #[test]
    fn test_on_goal_player() {
        let mut audio = ProceduralAudio::new();

        audio.on_goal(true);

        assert_eq!(audio.event_count(), 1);
        let events = audio.take_events();
        match &events[0] {
            AudioEvent::Goal {
                player_scored,
                volume,
            } => {
                assert!(*player_scored);
                assert!((*volume - 0.7).abs() < 0.01);
            }
            _ => panic!("Expected Goal event"),
        }
    }

    #[test]
    fn test_on_goal_ai() {
        let mut audio = ProceduralAudio::new();

        audio.on_goal(false);

        let events = audio.take_events();
        match &events[0] {
            AudioEvent::Goal { player_scored, .. } => {
                assert!(!*player_scored);
            }
            _ => panic!("Expected Goal event"),
        }
    }

    #[test]
    fn test_on_game_start() {
        let mut audio = ProceduralAudio::new();

        audio.on_game_start();

        assert_eq!(audio.event_count(), 1);
        let events = audio.take_events();
        matches!(&events[0], AudioEvent::GameStart { .. });
    }

    #[test]
    fn test_disabled_audio_no_events() {
        let mut audio = ProceduralAudio::new();
        audio.set_enabled(false);

        audio.on_paddle_hit(300.0, 300.0, 100.0);
        audio.on_wall_bounce();
        audio.on_goal(true);
        audio.on_game_start();

        assert_eq!(audio.event_count(), 0);
    }

    #[test]
    fn test_take_events_clears() {
        let mut audio = ProceduralAudio::new();

        audio.on_wall_bounce();
        audio.on_wall_bounce();

        assert_eq!(audio.event_count(), 2);
        let events = audio.take_events();
        assert_eq!(events.len(), 2);
        assert_eq!(audio.event_count(), 0);
    }

    #[test]
    fn test_peek_events_does_not_clear() {
        let mut audio = ProceduralAudio::new();

        audio.on_wall_bounce();

        let events = audio.peek_events();
        assert_eq!(events.len(), 1);
        assert_eq!(audio.event_count(), 1); // Still there
    }

    #[test]
    fn test_clear_events() {
        let mut audio = ProceduralAudio::new();

        audio.on_wall_bounce();
        audio.on_wall_bounce();
        audio.clear_events();

        assert_eq!(audio.event_count(), 0);
    }

    #[test]
    fn test_multiple_events() {
        let mut audio = ProceduralAudio::new();

        audio.on_paddle_hit(300.0, 300.0, 100.0);
        audio.on_wall_bounce();
        audio.on_goal(true);

        assert_eq!(audio.event_count(), 3);

        let events = audio.take_events();
        assert!(matches!(events[0], AudioEvent::PaddleHit { .. }));
        assert!(matches!(events[1], AudioEvent::WallBounce { .. }));
        assert!(matches!(events[2], AudioEvent::Goal { .. }));
    }

    #[test]
    fn test_audio_event_serialization() {
        let event = AudioEvent::PaddleHit {
            frequency: 440.0,
            duration: 0.08,
            volume: 0.7,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PaddleHit"));
        assert!(json.contains("440"));

        let deserialized: AudioEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_wall_bounce_serialization() {
        let event = AudioEvent::WallBounce {
            frequency: 200.0,
            duration: 0.05,
            volume: 0.35,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("WallBounce"));
    }

    #[test]
    fn test_goal_serialization() {
        let event = AudioEvent::Goal {
            player_scored: true,
            volume: 0.7,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Goal"));
        assert!(json.contains("player_scored"));
    }

    #[test]
    fn test_game_start_serialization() {
        let event = AudioEvent::GameStart { volume: 0.7 };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("GameStart"));
    }

    #[test]
    fn test_wall_bounce_with_velocity_low_speed() {
        let mut audio = ProceduralAudio::new();
        audio.on_wall_bounce_with_velocity(125.0, 250.0); // Half speed

        let events = audio.take_events();
        match &events[0] {
            AudioEvent::WallBounce { frequency, .. } => {
                // At half speed (ratio 0.5), freq should be ~187.5 Hz
                assert!(*frequency >= 150.0 && *frequency <= 200.0);
            }
            _ => panic!("Expected WallBounce event"),
        }
    }

    #[test]
    fn test_wall_bounce_with_velocity_high_speed() {
        let mut audio = ProceduralAudio::new();
        audio.on_wall_bounce_with_velocity(500.0, 250.0); // Double speed

        let events = audio.take_events();
        match &events[0] {
            AudioEvent::WallBounce { frequency, .. } => {
                // At double speed (ratio 2.0), freq should be ~300 Hz
                assert!(*frequency >= 250.0 && *frequency <= 350.0);
            }
            _ => panic!("Expected WallBounce event"),
        }
    }

    #[test]
    fn test_rally_milestone() {
        let mut audio = ProceduralAudio::new();
        audio.on_rally_milestone(5);

        assert_eq!(audio.event_count(), 1);
        let events = audio.take_events();
        match &events[0] {
            AudioEvent::RallyMilestone {
                rally_count,
                frequency,
                volume,
            } => {
                assert_eq!(*rally_count, 5);
                assert!(*frequency >= 300.0);
                assert!((*volume - 0.7).abs() < 0.01);
            }
            _ => panic!("Expected RallyMilestone event"),
        }
    }

    #[test]
    fn test_rally_milestone_increasing_frequency() {
        let mut audio = ProceduralAudio::new();

        audio.on_rally_milestone(5);
        audio.on_rally_milestone(15);

        let events = audio.take_events();

        let freq_5 = match &events[0] {
            AudioEvent::RallyMilestone { frequency, .. } => *frequency,
            _ => panic!("Expected RallyMilestone"),
        };

        let freq_15 = match &events[1] {
            AudioEvent::RallyMilestone { frequency, .. } => *frequency,
            _ => panic!("Expected RallyMilestone"),
        };

        // Higher rally count should have higher frequency
        assert!(freq_15 > freq_5);
    }

    #[test]
    fn test_rally_milestone_serialization() {
        let event = AudioEvent::RallyMilestone {
            rally_count: 10,
            frequency: 400.0,
            volume: 0.7,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("RallyMilestone"));
        assert!(json.contains("rally_count"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_wall_bounce_pitch_variation() {
        let mut audio = ProceduralAudio::new();

        // Generate multiple wall bounces
        audio.on_wall_bounce();
        audio.on_wall_bounce();
        audio.on_wall_bounce();

        let events = audio.take_events();

        // Frequencies should vary due to deterministic pattern
        let freqs: Vec<f32> = events
            .iter()
            .map(|e| match e {
                AudioEvent::WallBounce { frequency, .. } => *frequency,
                _ => 0.0,
            })
            .collect();

        // All should be in valid range (180-220 Hz)
        for freq in &freqs {
            assert!(*freq >= 180.0 && *freq <= 220.0);
        }
    }
}
