//! # jugar-audio
//!
//! Spatial audio system for Jugar with positional 3D sound and music playback.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use std::collections::HashMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Audio system errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AudioError {
    /// Audio source not found
    #[error("Audio source '{0}' not found")]
    SourceNotFound(String),
    /// Invalid audio format
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
    /// Playback error
    #[error("Playback error: {0}")]
    PlaybackError(String),
}

/// Result type for audio operations
pub type Result<T> = core::result::Result<T, AudioError>;

/// Audio source handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioHandle(pub u32);

/// Audio channel for mixing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioChannel {
    /// Master channel (affects all)
    Master,
    /// Music channel
    Music,
    /// Sound effects channel
    Effects,
    /// Voice/Dialog channel
    Voice,
    /// Ambient sounds channel
    Ambient,
}

impl Default for AudioChannel {
    fn default() -> Self {
        Self::Effects
    }
}

/// Audio playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Not playing
    #[default]
    Stopped,
    /// Currently playing
    Playing,
    /// Paused
    Paused,
}

/// Sound source for spatial audio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundSource {
    /// Source ID/name
    pub id: String,
    /// Position in world space
    pub position: Vec2,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pitch multiplier (1.0 = normal)
    pub pitch: f32,
    /// Whether to loop
    pub looping: bool,
    /// Audio channel
    pub channel: AudioChannel,
    /// Maximum distance for attenuation
    pub max_distance: f32,
    /// Reference distance (full volume)
    pub reference_distance: f32,
    /// Rolloff factor for distance attenuation
    pub rolloff: f32,
}

impl SoundSource {
    /// Creates a new sound source
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            position: Vec2::ZERO,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            channel: AudioChannel::Effects,
            max_distance: 1000.0,
            reference_distance: 1.0,
            rolloff: 1.0,
        }
    }

    /// Sets the position
    #[must_use]
    pub const fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Sets the volume
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Sets looping
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets the channel
    #[must_use]
    pub const fn with_channel(mut self, channel: AudioChannel) -> Self {
        self.channel = channel;
        self
    }

    /// Calculates volume based on distance from listener
    #[must_use]
    pub fn calculate_attenuation(&self, listener_pos: Vec2) -> f32 {
        let distance = self.position.distance(listener_pos);

        if distance <= self.reference_distance {
            return self.volume;
        }

        if distance >= self.max_distance {
            return 0.0;
        }

        // Linear rolloff
        let factor = 1.0
            - self.rolloff * (distance - self.reference_distance)
                / (self.max_distance - self.reference_distance);

        (self.volume * factor).clamp(0.0, 1.0)
    }
}

impl Default for SoundSource {
    fn default() -> Self {
        Self::new("unnamed")
    }
}

/// Audio listener (usually attached to camera/player)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioListener {
    /// Position in world space
    pub position: Vec2,
    /// Facing direction (for stereo panning)
    pub direction: Vec2,
}

impl AudioListener {
    /// Creates a new listener at the origin
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            direction: Vec2::new(0.0, 1.0), // Facing up
        }
    }

    /// Sets the position
    #[must_use]
    pub const fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Calculates stereo panning for a sound source (-1.0 left, 1.0 right)
    #[must_use]
    pub fn calculate_pan(&self, source_pos: Vec2) -> f32 {
        let to_source = source_pos - self.position;
        if to_source.length_squared() < 0.001 {
            return 0.0; // Source at listener position, center pan
        }

        let to_source_normalized = to_source.normalize();

        // Calculate right vector (perpendicular to direction)
        let right = Vec2::new(self.direction.y, -self.direction.x);

        // Dot product gives pan amount
        right.dot(to_source_normalized).clamp(-1.0, 1.0)
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel volumes for mixing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelVolumes {
    /// Master volume
    pub master: f32,
    /// Music volume
    pub music: f32,
    /// Effects volume
    pub effects: f32,
    /// Voice volume
    pub voice: f32,
    /// Ambient volume
    pub ambient: f32,
}

impl ChannelVolumes {
    /// Gets the volume for a channel
    #[must_use]
    pub fn get(&self, channel: AudioChannel) -> f32 {
        let channel_vol = match channel {
            AudioChannel::Master => 1.0,
            AudioChannel::Music => self.music,
            AudioChannel::Effects => self.effects,
            AudioChannel::Voice => self.voice,
            AudioChannel::Ambient => self.ambient,
        };
        self.master * channel_vol
    }

    /// Sets the volume for a channel
    pub fn set(&mut self, channel: AudioChannel, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        match channel {
            AudioChannel::Master => self.master = volume,
            AudioChannel::Music => self.music = volume,
            AudioChannel::Effects => self.effects = volume,
            AudioChannel::Voice => self.voice = volume,
            AudioChannel::Ambient => self.ambient = volume,
        }
    }
}

impl Default for ChannelVolumes {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            effects: 1.0,
            voice: 1.0,
            ambient: 0.7,
        }
    }
}

/// Audio playback instance
#[derive(Debug, Clone)]
pub struct PlayingSound {
    /// Handle
    pub handle: AudioHandle,
    /// Source configuration
    pub source: SoundSource,
    /// Current state
    pub state: PlaybackState,
    /// Playback time in seconds
    pub time: f32,
    /// Duration in seconds (0 if unknown)
    pub duration: f32,
}

impl PlayingSound {
    /// Creates a new playing sound
    #[must_use]
    pub fn new(handle: AudioHandle, source: SoundSource) -> Self {
        Self {
            handle,
            source,
            state: PlaybackState::Stopped,
            time: 0.0,
            duration: 0.0,
        }
    }

    /// Checks if finished (non-looping only)
    #[must_use]
    pub fn is_finished(&self) -> bool {
        !self.source.looping && self.duration > 0.0 && self.time >= self.duration
    }
}

/// Audio system for managing playback
pub struct AudioSystem {
    listener: AudioListener,
    volumes: ChannelVolumes,
    playing: HashMap<AudioHandle, PlayingSound>,
    next_handle: u32,
}

impl AudioSystem {
    /// Creates a new audio system
    #[must_use]
    pub fn new() -> Self {
        Self {
            listener: AudioListener::new(),
            volumes: ChannelVolumes::default(),
            playing: HashMap::new(),
            next_handle: 0,
        }
    }

    /// Gets the listener
    #[must_use]
    pub fn listener(&self) -> &AudioListener {
        &self.listener
    }

    /// Gets the listener mutably
    pub fn listener_mut(&mut self) -> &mut AudioListener {
        &mut self.listener
    }

    /// Sets the listener position
    pub fn set_listener_position(&mut self, position: Vec2) {
        self.listener.position = position;
    }

    /// Gets channel volumes
    #[must_use]
    pub fn volumes(&self) -> &ChannelVolumes {
        &self.volumes
    }

    /// Gets channel volumes mutably
    pub fn volumes_mut(&mut self) -> &mut ChannelVolumes {
        &mut self.volumes
    }

    /// Plays a sound and returns a handle
    pub fn play(&mut self, source: SoundSource) -> AudioHandle {
        let handle = AudioHandle(self.next_handle);
        self.next_handle += 1;

        let mut playing = PlayingSound::new(handle, source);
        playing.state = PlaybackState::Playing;

        let _ = self.playing.insert(handle, playing);
        handle
    }

    /// Stops a playing sound
    pub fn stop(&mut self, handle: AudioHandle) {
        if let Some(playing) = self.playing.get_mut(&handle) {
            playing.state = PlaybackState::Stopped;
        }
    }

    /// Pauses a playing sound
    pub fn pause(&mut self, handle: AudioHandle) {
        if let Some(playing) = self.playing.get_mut(&handle) {
            if playing.state == PlaybackState::Playing {
                playing.state = PlaybackState::Paused;
            }
        }
    }

    /// Resumes a paused sound
    pub fn resume(&mut self, handle: AudioHandle) {
        if let Some(playing) = self.playing.get_mut(&handle) {
            if playing.state == PlaybackState::Paused {
                playing.state = PlaybackState::Playing;
            }
        }
    }

    /// Gets a playing sound
    #[must_use]
    pub fn get(&self, handle: AudioHandle) -> Option<&PlayingSound> {
        self.playing.get(&handle)
    }

    /// Checks if a sound is playing
    #[must_use]
    pub fn is_playing(&self, handle: AudioHandle) -> bool {
        self.playing
            .get(&handle)
            .is_some_and(|p| p.state == PlaybackState::Playing)
    }

    /// Returns number of currently playing sounds
    #[must_use]
    pub fn playing_count(&self) -> usize {
        self.playing
            .values()
            .filter(|p| p.state == PlaybackState::Playing)
            .count()
    }

    /// Updates the audio system (advances time, removes finished)
    pub fn update(&mut self, dt: f32) {
        // Update playback times
        for playing in self.playing.values_mut() {
            if playing.state == PlaybackState::Playing {
                playing.time += dt;

                // Handle looping
                if playing.source.looping && playing.duration > 0.0 {
                    while playing.time >= playing.duration {
                        playing.time -= playing.duration;
                    }
                }
            }
        }

        // Remove finished sounds
        self.playing.retain(|_, p| {
            p.state != PlaybackState::Stopped && !p.is_finished()
        });
    }

    /// Calculates final volume for a sound (with attenuation and channel mixing)
    #[must_use]
    pub fn calculate_final_volume(&self, handle: AudioHandle) -> f32 {
        let Some(playing) = self.playing.get(&handle) else {
            return 0.0;
        };

        let attenuation = playing.source.calculate_attenuation(self.listener.position);
        let channel_volume = self.volumes.get(playing.source.channel);

        attenuation * channel_volume
    }

    /// Calculates stereo pan for a sound
    #[must_use]
    pub fn calculate_pan(&self, handle: AudioHandle) -> f32 {
        let Some(playing) = self.playing.get(&handle) else {
            return 0.0;
        };

        self.listener.calculate_pan(playing.source.position)
    }

    /// Stops all sounds
    pub fn stop_all(&mut self) {
        for playing in self.playing.values_mut() {
            playing.state = PlaybackState::Stopped;
        }
    }

    /// Stops all sounds in a channel
    pub fn stop_channel(&mut self, channel: AudioChannel) {
        for playing in self.playing.values_mut() {
            if playing.source.channel == channel {
                playing.state = PlaybackState::Stopped;
            }
        }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AudioSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioSystem")
            .field("playing_count", &self.playing_count())
            .field("listener", &self.listener)
            .finish()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_source_creation() {
        let source = SoundSource::new("explosion")
            .with_position(Vec2::new(10.0, 20.0))
            .with_volume(0.8);

        assert_eq!(source.id, "explosion");
        assert!((source.volume - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sound_source_attenuation_at_reference() {
        let source = SoundSource::new("test")
            .with_position(Vec2::ZERO)
            .with_volume(1.0);

        let attenuation = source.calculate_attenuation(Vec2::ZERO);
        assert!((attenuation - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sound_source_attenuation_at_max_distance() {
        let mut source = SoundSource::new("test")
            .with_position(Vec2::ZERO)
            .with_volume(1.0);
        source.max_distance = 100.0;
        source.reference_distance = 1.0;

        let attenuation = source.calculate_attenuation(Vec2::new(100.0, 0.0));
        assert!(attenuation.abs() < f32::EPSILON);
    }

    #[test]
    fn test_sound_source_attenuation_midpoint() {
        let mut source = SoundSource::new("test")
            .with_position(Vec2::ZERO)
            .with_volume(1.0);
        source.max_distance = 100.0;
        source.reference_distance = 0.0;
        source.rolloff = 1.0;

        let attenuation = source.calculate_attenuation(Vec2::new(50.0, 0.0));
        assert!((attenuation - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_audio_listener_pan_center() {
        let listener = AudioListener::new().with_position(Vec2::ZERO);
        let pan = listener.calculate_pan(Vec2::ZERO);
        assert!(pan.abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_listener_pan_right() {
        let listener = AudioListener::new().with_position(Vec2::ZERO);
        // Facing up (0, 1), sound to the right (1, 0)
        let pan = listener.calculate_pan(Vec2::new(10.0, 0.0));
        assert!(pan > 0.9); // Should be panned right
    }

    #[test]
    fn test_audio_listener_pan_left() {
        let listener = AudioListener::new().with_position(Vec2::ZERO);
        // Facing up (0, 1), sound to the left (-1, 0)
        let pan = listener.calculate_pan(Vec2::new(-10.0, 0.0));
        assert!(pan < -0.9); // Should be panned left
    }

    #[test]
    fn test_channel_volumes_default() {
        let volumes = ChannelVolumes::default();
        assert!((volumes.master - 1.0).abs() < f32::EPSILON);
        assert!((volumes.effects - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_channel_volumes_get() {
        let mut volumes = ChannelVolumes::default();
        volumes.master = 0.5;
        volumes.effects = 0.8;

        let final_vol = volumes.get(AudioChannel::Effects);
        assert!((final_vol - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_channel_volumes_set() {
        let mut volumes = ChannelVolumes::default();
        volumes.set(AudioChannel::Music, 0.5);
        assert!((volumes.music - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_channel_volumes_clamp() {
        let mut volumes = ChannelVolumes::default();
        volumes.set(AudioChannel::Master, 2.0);
        assert!((volumes.master - 1.0).abs() < f32::EPSILON);

        volumes.set(AudioChannel::Master, -1.0);
        assert!(volumes.master.abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_system_play() {
        let mut system = AudioSystem::new();
        let source = SoundSource::new("test");
        let handle = system.play(source);

        assert!(system.is_playing(handle));
        assert_eq!(system.playing_count(), 1);
    }

    #[test]
    fn test_audio_system_stop() {
        let mut system = AudioSystem::new();
        let source = SoundSource::new("test");
        let handle = system.play(source);

        system.stop(handle);
        assert!(!system.is_playing(handle));
    }

    #[test]
    fn test_audio_system_pause_resume() {
        let mut system = AudioSystem::new();
        let source = SoundSource::new("test");
        let handle = system.play(source);

        system.pause(handle);
        assert!(!system.is_playing(handle));

        let playing = system.get(handle).unwrap();
        assert_eq!(playing.state, PlaybackState::Paused);

        system.resume(handle);
        assert!(system.is_playing(handle));
    }

    #[test]
    fn test_audio_system_update_advances_time() {
        let mut system = AudioSystem::new();
        let source = SoundSource::new("test");
        let handle = system.play(source);

        system.update(1.0);

        let playing = system.get(handle).unwrap();
        assert!((playing.time - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_system_update_removes_stopped() {
        let mut system = AudioSystem::new();
        let source = SoundSource::new("test");
        let handle = system.play(source);

        system.stop(handle);
        system.update(0.1);

        assert!(system.get(handle).is_none());
    }

    #[test]
    fn test_audio_system_final_volume() {
        let mut system = AudioSystem::new();
        system.set_listener_position(Vec2::ZERO);

        let source = SoundSource::new("test")
            .with_position(Vec2::ZERO)
            .with_volume(1.0);
        let handle = system.play(source);

        let volume = system.calculate_final_volume(handle);
        assert!((volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_audio_system_stop_all() {
        let mut system = AudioSystem::new();
        system.play(SoundSource::new("test1"));
        system.play(SoundSource::new("test2"));

        assert_eq!(system.playing_count(), 2);

        system.stop_all();
        assert_eq!(system.playing_count(), 0);
    }

    #[test]
    fn test_audio_system_stop_channel() {
        let mut system = AudioSystem::new();
        let _ = system.play(SoundSource::new("music").with_channel(AudioChannel::Music));
        let _ = system.play(SoundSource::new("effect").with_channel(AudioChannel::Effects));

        system.stop_channel(AudioChannel::Music);
        system.update(0.0);

        assert_eq!(system.playing_count(), 1);
    }

    #[test]
    fn test_playing_sound_finished() {
        let mut playing = PlayingSound::new(AudioHandle(0), SoundSource::new("test"));
        playing.duration = 2.0;
        playing.time = 1.0;

        assert!(!playing.is_finished());

        playing.time = 2.5;
        assert!(playing.is_finished());
    }

    #[test]
    fn test_playing_sound_looping_not_finished() {
        let mut playing =
            PlayingSound::new(AudioHandle(0), SoundSource::new("test").with_looping(true));
        playing.duration = 2.0;
        playing.time = 5.0;

        assert!(!playing.is_finished());
    }
}
