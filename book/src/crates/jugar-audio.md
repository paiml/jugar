# jugar-audio

Spatial 2D audio with channel mixing.

## Basic Playback

```rust
use jugar_audio::prelude::*;

let mut audio = AudioEngine::new();

// Load sound
let sound = audio.load_sound("explosion.wav");

// Play sound
audio.play(sound);

// Play with volume
audio.play_with_volume(sound, 0.5);
```

## Channels

```rust
// Audio channels
pub enum Channel {
    Master,   // Overall volume
    Music,    // Background music
    Effects,  // Sound effects
    Voice,    // Dialogue
    Ambient,  // Environmental sounds
}

// Set channel volume
audio.set_channel_volume(Channel::Music, 0.7);
audio.set_channel_volume(Channel::Effects, 1.0);

// Play on specific channel
audio.play_on_channel(sound, Channel::Effects);
```

## Spatial Audio

```rust
// Create audio source at position
let source = audio.create_source(Position::new(500.0, 300.0));
audio.play_at_source(sound, source);

// Set listener position (usually player)
audio.set_listener_position(player_position);

// Update source position
audio.set_source_position(source, enemy_position);
```

## Attenuation

```rust
// Configure distance attenuation
let source = audio.create_source_with_config(
    position,
    SpatialConfig {
        min_distance: 50.0,   // Full volume within this range
        max_distance: 500.0,  // Inaudible beyond this range
        rolloff: Rolloff::Linear,
    },
);
```

## Rolloff Models

| Model | Description |
|-------|-------------|
| `Linear` | Linear falloff between min and max |
| `Inverse` | 1/distance falloff |
| `InverseSquare` | 1/distance² (realistic) |
| `None` | No distance attenuation |

## Stereo Panning

```rust
// Automatic panning based on position
// Sounds to the left of listener pan left
// Sounds to the right pan right

// Manual pan (-1.0 = left, 0.0 = center, 1.0 = right)
audio.set_pan(source, -0.5);
```

## Music

```rust
// Background music (loops by default)
let music = audio.load_music("background.ogg");
audio.play_music(music);

// Crossfade to new track
audio.crossfade_music(new_music, 2.0);  // 2 second fade

// Pause/resume
audio.pause_music();
audio.resume_music();
```

## Sound Groups

```rust
// Create a group for related sounds
let footsteps = SoundGroup::new()
    .add(audio.load_sound("step1.wav"))
    .add(audio.load_sound("step2.wav"))
    .add(audio.load_sound("step3.wav"));

// Play random sound from group
footsteps.play_random(&mut audio);

// Play sequential
footsteps.play_next(&mut audio);
```

## Example: Complete Setup

```rust
fn setup_audio(audio: &mut AudioEngine) {
    // Set channel volumes
    audio.set_channel_volume(Channel::Master, 0.8);
    audio.set_channel_volume(Channel::Music, 0.5);
    audio.set_channel_volume(Channel::Effects, 1.0);

    // Start background music
    let music = audio.load_music("theme.ogg");
    audio.play_music(music);
}

fn update_audio(audio: &mut AudioEngine, player_pos: Position) {
    // Update listener to follow player
    audio.set_listener_position(player_pos);
}

fn play_explosion(audio: &mut AudioEngine, position: Position) {
    let sound = audio.load_sound("explosion.wav");
    let source = audio.create_source(position);
    audio.play_at_source_on_channel(sound, source, Channel::Effects);
}
```
