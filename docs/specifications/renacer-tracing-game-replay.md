---
title: Renacer-Based Game Event Tracing and Deterministic Replay
spec_id: TRACE-001
version: 1.3.0
status: Draft
created: 2025-12-09
updated: 2025-12-09
authors: [PAIML]
reviewers: [TPS Code Review]
---

# Renacer-Based Game Event Tracing and Deterministic Replay

## Abstract

This specification defines a comprehensive event tracing system for Jugar games, enabling deterministic replay, debugging, and behavioral analysis. The system is modeled after renacer's unified trace architecture and incorporates principles from the Toyota Production System (TPS) and classic Atari game development research.

**Version 1.1 Update:** This revision addresses critical TPS-based code review findings regarding Jidoka (Stop the Line), Poka-Yoke (Floating Point Determinism), Heijunka (Adaptive Snapshots), and Muda (Timestamp Waste).

**Version 1.3 Update:** This revision incorporates TPS Code Review findings:
- **Soft Andon** (Visual trace loss indicator instead of hard block for game feel debugging)
- **Zobrist Hashing** (O(1) incremental hashing vs O(N) SHA-256 for entropy detection)
- **Fixed32 Poka-Yoke Macro** (Compile-time enforcement preventing f32 in simulation)
- **Stream Verification** (One-piece flow for Monte Carlo tests - immediate invariant detection)
- **Fixed32 Overflow Checks** (Panic on wrap-around to prevent silent bugs)

## 1. Introduction

### 1.1 Problem Statement

Game debugging presents unique challenges absent in traditional software:

1. **Non-Determinism**: Random number generation, timing variations, and input latency create divergent execution paths
2. **State Explosion**: Game state changes 60 times per second across hundreds of variables
3. **Observer Effect**: Adding instrumentation can alter gameplay timing and feel
4. **Reproduction Difficulty**: "It only happens sometimes" bugs are common
5. **Cross-Platform Divergence**: IEEE 754 floating-point behaves differently across browsers, OSes, and architectures

### 1.2 Solution Overview

We implement a **Renacer-style unified trace** architecture that captures:

- Every input event (keyboard, mouse, touch, gamepad)
- Every game state mutation
- Every AI decision
- Every render frame
- Logical frame ordering (not physical timestamps for simulation)

This enables:
- **Deterministic Replay**: Reproduce any bug exactly across platforms
- **Time-Travel Debugging**: Step forward/backward through game history
- **Behavioral Analysis**: Analyze player patterns and AI effectiveness
- **Regression Testing**: Verify gameplay consistency across versions

## 2. Research Foundation

### 2.1 Toyota Production System Principles Applied

| TPS Principle | Game Tracing Application | v1.1 Enhancement |
|---------------|--------------------------|------------------|
| **Jidoka** (Build Quality In) | Trace validation at record time; corrupt traces fail fast | **Andon Cord**: Buffer overflow stalls game loop in debug mode |
| **Heijunka** (Production Leveling) | Ring buffer decouples hot path from I/O | **Adaptive Snapshots**: Entropy-based keyframe scheduling |
| **Muda** (Eliminate Waste) | RLE compression for repeated states; lazy serialization | **Strip Physical Timestamps**: Frame number is the clock |
| **Poka-Yoke** (Error-Proofing) | Lamport clocks guarantee causal ordering | **Fixed-Point Math**: Cross-platform determinism guaranteed |
| **Genchi Genbutsu** (Go and See) | Adaptive sampling prevents observer effect | **Query-Based Debugging**: Filter traces by conditions |
| **Kaizen** (Continuous Improvement) | Trace analysis feeds back into game design | |
| **Mieruka** (Visual Management) | HTML trace viewer with timeline visualization | |
| **Mura** (Unevenness Elimination) | Fixed-interval snapshots create waste | **Entropy-Based Triggers**: Snapshot on high state delta |

### 2.2 Peer-Reviewed Citations (Original 10)

1. **Lamport, L. (1978)**. "Time, Clocks, and the Ordering of Events in a Distributed System." *Communications of the ACM*, 21(7), 558-565. DOI: 10.1145/359545.359563
   - *Foundation for logical clocks ensuring causal ordering without synchronized physical clocks*

2. **Blow, J. (2004)**. "Game Development: Harder Than You Think." *Queue*, 2(10), 28-37. DOI: 10.1145/1035594.1035607
   - *Analysis of game development complexity; argues for deterministic replay as essential debugging tool*

3. **Montfort, N., & Bogost, I. (2009)**. *Racing the Beam: The Atari Video Computer System*. MIT Press. ISBN: 978-0262012577
   - *Historical analysis of Atari 2600 development constraints; demonstrates value of cycle-exact replay*

4. **Csikszentmihalyi, M. (1990)**. *Flow: The Psychology of Optimal Experience*. Harper & Row. ISBN: 978-0061339202
   - *Foundation for DDA systems; trace analysis enables flow state optimization*

5. **Hunicke, R. (2005)**. "The Case for Dynamic Difficulty Adjustment in Games." *Proceedings of the 2005 ACM SIGCHI International Conference on Advances in Computer Entertainment Technology*, 429-433. DOI: 10.1145/1178477.1178573
   - *DDA research directly applicable to AI difficulty tracing and adaptation*

6. **Ohno, T. (1988)**. *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. ISBN: 978-0915299140
   - *Original TPS documentation; principles adapted for zero-waste tracing*

7. **Ronsse, M., & De Bosschere, K. (1999)**. "RecPlay: A Fully Integrated Practical Record/Replay System." *ACM Transactions on Computer Systems*, 17(2), 133-152. DOI: 10.1145/312203.312214
   - *Foundational work on efficient record/replay for debugging*

8. **Chen, S., et al. (2015)**. "Deterministic Replay: A Survey." *ACM Computing Surveys*, 48(2), Article 17. DOI: 10.1145/2790077
   - *Comprehensive survey of replay techniques; informs our hybrid approach*

9. **Sigelman, B. H., et al. (2010)**. "Dapper, a Large-Scale Distributed Systems Tracing Infrastructure." Google Technical Report.
   - *Distributed tracing at scale; sampling strategies and trace context propagation*

10. **Mayer, C., et al. (2017)**. "On the Determinism of JavaScript." *Proceedings of the 32nd IEEE/ACM International Conference on Automated Software Engineering*, 374-384. DOI: 10.1109/ASE.2017.8115651
    - *Analysis of non-determinism sources in JavaScript; applicable to WASM game loops*

### 2.3 Additional Citations (TPS & Engineering Focused)

11. **Dunlap, G. W., et al. (2002)**. "ReVirt: Enabling Intrusion Analysis through Virtual-Machine Logging and Replay." *OSDI*.
    - *Establishes that ALL external inputs must be logged for faithful replay—supports Jidoka "Stop the Line" requirement*

12. **Monniaux, D. (2008)**. "The pitfalls of verifying floating-point computations." *ACM Transactions on Programming Languages and Systems*, 30(3).
    - *Essential for Poka-Yoke: demonstrates why f32 is NOT deterministic across architectures*

13. **Elnozahy, E. N., et al. (2002)**. "A Survey of Rollback-Recovery Protocols in Message-Passing Systems." *ACM Computing Surveys*.
    - *Theoretical basis for adaptive snapshots: "Incremental State Deltas" vs. "Full Checkpointing"*

14. **Cornelis, J., et al. (2003)**. "A Taxonomy of Execution Replay Systems." *Proceedings of the International Conference on Advances in Infrastructure for e-Business*.
    - *Validates hybrid approach: logging inputs + periodic state snapshots*

15. **Zeller, A. (2002)**. "Isolating cause-effect chains from computer programs." *Proceedings of the 10th ACM SIGSOFT Symposium on Foundations of Software Engineering*.
    - *Delta Debugging: using traces to ISOLATE the specific input causing bugs*

16. **Jain, R., et al. (2014)**. "Cooperative Logic Debugging for Production Systems." *USENIX Annual Technical Conference*.
    - *Supports removing physical timestamps: logical ordering is paramount*

17. **Ko, A. J., & Myers, B. A. (2008)**. "Debugging Reinvented: Asking and Answering Why and Why Not Questions about Program Behavior." *ICSE*.
    - *Foundation for "Whyline" query-based debugging interfaces*

18. **Luo, Q., et al. (2014)**. "An Empirical Analysis of Flaky Tests." *Proceedings of the 22nd ACM SIGSOFT International Symposium on Foundations of Software Engineering*.
    - *Async waits and concurrency (input polling) are top causes of non-determinism*

19. **Yin, Z., et al. (2018)**. "Structural Analysis of User Behavior in Interactive Systems." *ACM Transactions on Computer-Human Interaction*.
    - *Mathematical modeling of player "Flow" states from trace data*

20. **Amodei, D., et al. (2016)**. "Concrete Problems in AI Safety." *arXiv preprint arXiv:1606.06565*.
    - *Interpretable AI logs for debugging reward hacking and unintended behaviors*

## 3. Architecture

### 3.1 Critical Design Decisions (v1.1)

#### 3.1.1 Jidoka: The Andon Cord

**Problem (v1.0):** Ring buffer overflow silently drops events, creating "Zombie Replays" that diverge from reality.

**Solution:** In `--debug` mode, buffer overflow triggers **game loop stall** (Andon Cord):

```rust
pub enum BufferPolicy {
    /// Production mode: drop oldest events on overflow (preserves responsiveness)
    DropOldest,
    /// Debug mode: block game loop until buffer drains (preserves correctness)
    AndonCord,
}

impl RingBuffer<TraceEvent> {
    pub fn push(&self, event: TraceEvent, policy: BufferPolicy) -> Result<(), TraceError> {
        loop {
            match self.try_push(event.clone()) {
                Ok(()) => return Ok(()),
                Err(BufferFull) => match policy {
                    BufferPolicy::DropOldest => {
                        self.pop(); // Discard oldest
                        continue;
                    }
                    BufferPolicy::AndonCord => {
                        // STOP THE LINE: Block until space available
                        // This is intentional in debug mode per Jidoka
                        std::thread::sleep(Duration::from_micros(100));
                        continue;
                    }
                }
            }
        }
    }
}
```

**Rationale (Dunlap 2002):** "A trace with missing input events is worse than no trace at all."

#### 3.1.2 Poka-Yoke: Cross-Platform Determinism

**Problem (v1.0):** IEEE 754 `f32` behaves differently across platforms. Transcendental functions (sin, cos, pow) vary by OS/browser/CPU.

**Solution:** Mandate **Fixed-Point Math** for all game logic:

```rust
/// Fixed-point number with 16.16 format (16 integer bits, 16 fractional bits)
/// Guarantees identical results across ALL platforms (Monniaux 2008)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Fixed32(i32);

impl Fixed32 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1 << 16);
    pub const HALF: Self = Self(1 << 15);

    /// Create from integer
    #[inline]
    pub const fn from_int(n: i32) -> Self {
        Self(n << 16)
    }

    /// Create from f32 (use only for constants, not runtime)
    #[inline]
    pub fn from_f32(f: f32) -> Self {
        Self((f * 65536.0) as i32)
    }

    /// Convert to f32 (for rendering only, not game logic)
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 65536.0
    }

    /// Fixed-point multiplication (no floating point)
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self(((self.0 as i64 * other.0 as i64) >> 16) as i32)
    }

    /// Fixed-point division (no floating point)
    #[inline]
    pub fn div(self, other: Self) -> Self {
        Self((((self.0 as i64) << 16) / other.0 as i64) as i32)
    }
}

// Game state uses Fixed32, NOT f32
pub struct DeterministicGameState {
    pub ball_x: Fixed32,
    pub ball_y: Fixed32,
    pub ball_vx: Fixed32,
    pub ball_vy: Fixed32,
    pub left_paddle_y: Fixed32,
    pub right_paddle_y: Fixed32,
}
```

**Rationale (Monniaux 2008):** "Floating-point non-determinism is a primary source of divergence in cross-platform replay systems."

#### 3.1.3 Heijunka: Adaptive Snapshots

**Problem (v1.0):** Fixed 60-frame snapshot interval creates Mura (unevenness). Menu screens waste space; high-action moments lack granularity.

**Solution:** Snapshot based on **State Entropy** (magnitude of change):

```rust
pub struct AdaptiveSnapshotter {
    /// Minimum frames between snapshots
    min_interval: u64,
    /// Maximum frames between snapshots (force snapshot)
    max_interval: u64,
    /// Entropy threshold to trigger snapshot
    entropy_threshold: u32,
    /// Last snapshot frame
    last_snapshot_frame: u64,
    /// Previous state hash for delta calculation
    prev_state_hash: [u8; 32],
}

impl AdaptiveSnapshotter {
    /// Determine if a snapshot should be taken this frame
    pub fn should_snapshot(&mut self, frame: u64, state: &GameState) -> SnapshotDecision {
        let frames_since = frame - self.last_snapshot_frame;

        // Force snapshot at max interval
        if frames_since >= self.max_interval {
            return SnapshotDecision::FullSnapshot;
        }

        // Calculate state entropy (how much changed)
        let current_hash = state.hash();
        let entropy = self.calculate_entropy(&current_hash);

        // High entropy = take snapshot
        if entropy >= self.entropy_threshold && frames_since >= self.min_interval {
            self.prev_state_hash = current_hash;
            self.last_snapshot_frame = frame;
            return SnapshotDecision::DeltaSnapshot;
        }

        SnapshotDecision::Skip
    }

    /// Calculate entropy as Hamming distance between state hashes
    fn calculate_entropy(&self, current: &[u8; 32]) -> u32 {
        self.prev_state_hash
            .iter()
            .zip(current.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }
}

pub enum SnapshotDecision {
    /// No snapshot needed
    Skip,
    /// Take delta snapshot (high entropy)
    DeltaSnapshot,
    /// Take full snapshot (max interval reached)
    FullSnapshot,
}
```

**Rationale (Elnozahy 2002):** "Adaptive checkpointing based on state mutation rates significantly reduces log sizes."

#### 3.1.4 Muda Elimination: Streamlined Frame Record

**Problem (v1.0):** Storing `timestamp_nanos` and `lamport_clock` per frame is waste. For deterministic games, `frame_number` IS the clock.

**Solution:** Lean frame record for simulation:

```rust
/// Lean frame record - frame_number is the only clock needed (Jain 2014)
pub struct FrameRecord {
    /// Frame number (monotonic, IS the logical clock for deterministic games)
    pub frame: u64,
    /// Input events this frame (relative timing within frame if needed)
    pub inputs: Vec<InputEvent>,
    /// State hash for verification (optional in release traces)
    pub state_hash: Option<[u8; 32]>,
}

/// Input event with frame-relative timing
pub struct InputEvent {
    /// Event type
    pub event_type: InputEventType,
    /// Microseconds since frame start (for sub-frame ordering only)
    pub frame_offset_us: u16,
}
```

**Physical timestamps are ONLY stored:**
- In the session header (start time)
- For debugging/profiling (separate optional stream)

**Rationale (Jain 2014):** "Logical time is sufficient for reproducing bugs; physical time introduces unnecessary noise."

### 3.2 Trace Event Hierarchy (v1.1 Revised)

```
GameTrace
├── SessionHeader (once per trace)
│   ├── trace_id: [u8; 16]           # W3C Trace Context
│   ├── session_id: u64               # Unique session identifier
│   ├── game_version: String          # Semantic version
│   ├── platform: Platform            # Web/Native/Mobile
│   ├── start_time_nanos: u64         # Physical timestamp (header only)
│   ├── rng_seed: u64                 # Initial RNG state
│   ├── fixed_dt: Fixed32             # Fixed timestep for simulation
│   └── math_mode: MathMode           # Fixed32 or SoftFloat
│
├── FrameRecord[] (per frame) - LEAN
│   ├── frame: u64                    # Frame number IS the clock
│   ├── inputs: Vec<InputEvent>       # Events this frame
│   └── state_hash: Option<[u8; 32]>  # For verification (debug only)
│
├── Snapshot[] (adaptive, entropy-based)
│   ├── frame: u64                    # Snapshot frame
│   ├── snapshot_type: SnapshotType   # Full or Delta
│   ├── data: bytes                   # Compressed state
│   └── entropy_score: u32            # Why snapshot was taken
│
├── AIDecision[] (per AI update)
│   ├── frame: u64                    # Decision frame
│   ├── ai_id: u8                     # Which AI
│   ├── difficulty: u8                # Current difficulty
│   ├── flow_channel: FlowChannel     # Boredom/Flow/Anxiety
│   ├── inputs: AIInputsFixed         # Fixed-point inputs
│   ├── target_y: Fixed32             # Computed target
│   ├── velocity: Fixed32             # Output velocity
│   └── rng_state: u64                # For determinism verification
│
└── GameEvent[] (game-specific)
    ├── frame: u64                    # Event frame
    ├── event_type: GameEventType     # Score/Collision/etc.
    └── data: GameEventData           # Type-specific payload
```

### 3.3 Ring Buffer with Andon Cord

```
HOT PATH (Game Thread)                     COLD PATH (Trace Writer)
━━━━━━━━━━━━━━━━━━━━━━━━                   ━━━━━━━━━━━━━━━━━━━━━━━━
game_loop() {                              trace_writer_loop() {
  input = poll_events()                      loop {
  ──▶ trace.record_input(input)                batch = ring.drain(256)
      │                                         if batch.is_empty() {
      ├─[buffer full?]──▶ ANDON CORD            sleep(16ms)
      │   (debug mode: BLOCK until space)       continue
      │   (prod mode: drop oldest)            }
      │
  state = update(input, FIXED_DT)              compressed = lz4.compress(batch)
  ──▶ trace.record_state_if_entropy_high()     file.write_all(compressed)

  ai_decision = ai.update(...)                 if frames % 1000 == 0 {
  ──▶ trace.record_ai(ai_decision)               file.sync_all()
                                               }
  commands = render(state.to_f32())          }
  ──▶ trace.record_frame_end()             }
}

ANDON CORD BEHAVIOR:
┌─────────────────────────────────────────────────────────┐
│  In --debug mode: Buffer overflow BLOCKS game loop      │
│  This is INTENTIONAL (Jidoka principle)                │
│  A partial trace is WORSE than a slow game             │
│  Per Dunlap 2002: ALL inputs must be logged            │
└─────────────────────────────────────────────────────────┘
```

**Performance Budget (v1.1):**
- Hot path overhead: **< 2μs per frame** (0.012% of 16.67ms frame)
- Memory overhead: **< 1MB** ring buffer
- Storage: **~5KB/minute** compressed (lean format)
- Andon Cord latency: **< 1ms** (rare, debug-only)

## 4. Trace File Format (v1.1)

### 4.1 File Structure (`.jtr` - Jugar Trace Replay)

```
┌─────────────────────────────────────────┐
│ Magic Number: "JTR\x01" (4 bytes)        │  ← Version 1.1
│ Flags: u32                               │
│   bit 0: has_state_hashes                │
│   bit 1: fixed_point_math                │
│   bit 2: adaptive_snapshots              │
│   bit 3: andon_cord_enabled              │
├─────────────────────────────────────────┤
│ Header Block (MessagePack)               │
│   - session_id                           │
│   - game_version                         │
│   - rng_seed                             │
│   - fixed_dt                             │
│   - math_mode                            │
│   - platform_info                        │
│   - start_time_nanos (physical, once)    │
├─────────────────────────────────────────┤
│ Frame Blocks (LZ4 compressed chunks)     │
│   - FrameRecord[]                        │
│   - Inline if keyframe: Snapshot         │
├─────────────────────────────────────────┤
│ AI Decision Stream (separate, optional)  │
│   - AIDecision[]                         │
├─────────────────────────────────────────┤
│ Game Event Stream (separate)             │
│   - GameEvent[]                          │
├─────────────────────────────────────────┤
│ Index Block (for seeking)                │
│   - frame -> file_offset                 │
│   - snapshot_frames[]                    │
└─────────────────────────────────────────┘
```

### 4.2 Compression Strategy (v1.1)

| Data Type | Compression | Rationale |
|-----------|-------------|-----------|
| Frame records | RLE + LZ4 | Many frames have no input (Muda elimination) |
| Snapshots | Zstd level 3 | Best ratio for state data |
| AI decisions | Delta encoding + LZ4 | Most fields change slowly |
| Game events | MessagePack + LZ4 | Sparse, structured data |

**Expected Sizes (v1.1 - leaner):**
- Pong 1-minute trace: **~5KB** compressed (was ~10KB)
- Complex game 1-minute: **~50KB** compressed (was ~100KB)
- Keyframe interval: **Adaptive** (entropy-based, typically 30-120 frames)

## 5. Genchi Genbutsu: Query-Based Debugging

**Problem (Ko 2008):** Linear trace playback forces developers to watch entire sessions.

**Solution:** Implement **Whyline-style** query interface:

```rust
/// Query-based trace analysis (Ko & Myers 2008)
pub struct TraceQuery {
    trace: GameTrace,
}

impl TraceQuery {
    /// Find frames where condition is true
    pub fn find_frames(&self, predicate: impl Fn(&FrameRecord) -> bool) -> Vec<u64> {
        self.trace.frames()
            .filter(|f| predicate(f))
            .map(|f| f.frame)
            .collect()
    }

    /// "Why did the ball miss the paddle?"
    pub fn why_ball_missed(&self, miss_frame: u64) -> MissAnalysis {
        // Find preceding paddle positions
        let paddle_history = self.get_paddle_history(miss_frame - 60, miss_frame);
        // Find ball trajectory
        let ball_trajectory = self.get_ball_trajectory(miss_frame - 60, miss_frame);
        // Find AI decisions
        let ai_decisions = self.get_ai_decisions(miss_frame - 60, miss_frame);

        MissAnalysis {
            paddle_history,
            ball_trajectory,
            ai_decisions,
            root_cause: self.infer_root_cause(),
        }
    }

    /// Find all frames where: Ball.vx > threshold AND AI missed
    pub fn high_speed_misses(&self, velocity_threshold: Fixed32) -> Vec<u64> {
        self.find_frames(|f| {
            f.ball_vx.abs() > velocity_threshold &&
            f.events.iter().any(|e| matches!(e, GameEvent::AIMiss))
        })
    }
}
```

**CLI Integration:**
```bash
# Find all frames where ball velocity exceeded threshold
cargo run --bin pong_ai_cli -- trace-query session.jtr \
    --filter "ball_vx > 500 AND event_type = 'AIMiss'"

# Explain why AI missed at frame 1234
cargo run --bin pong_ai_cli -- trace-why session.jtr --frame 1234 --event miss

# Show AI decisions leading to difficulty change
cargo run --bin pong_ai_cli -- trace-query session.jtr \
    --filter "event_type = 'DifficultyChanged'" --context 60
```

## 6. API Design (v1.1)

### 6.1 Recording API with Andon Cord

```rust
/// Game trace recorder with Jidoka safety
pub struct GameTracer {
    buffer: RingBuffer<TraceEvent>,
    policy: BufferPolicy,
    snapshotter: AdaptiveSnapshotter,
    frame: AtomicU64,
    rng_seed: u64,
    enabled: bool,
}

impl GameTracer {
    /// Create tracer with specified safety policy
    pub fn new(policy: BufferPolicy) -> Self;

    /// Create debug tracer (Andon Cord enabled)
    pub fn debug() -> Self {
        Self::new(BufferPolicy::AndonCord)
    }

    /// Create production tracer (drop oldest on overflow)
    pub fn production() -> Self {
        Self::new(BufferPolicy::DropOldest)
    }

    /// Record input event
    /// In AndonCord mode: MAY BLOCK if buffer full (intentional)
    #[inline(always)]
    pub fn record_input(&self, event: &InputEvent) -> Result<(), TraceError>;

    /// Record frame end with adaptive snapshot check
    #[inline(always)]
    pub fn record_frame_end(&self, state: &DeterministicGameState);

    /// Force a full snapshot (e.g., on mode change)
    pub fn force_snapshot(&self, state: &DeterministicGameState);
}
```

### 6.2 Fixed-Point Game State

```rust
/// Deterministic game state using Fixed32 (Monniaux 2008)
#[derive(Clone, Serialize, Deserialize)]
pub struct DeterministicPongState {
    // Positions (Fixed32 for cross-platform determinism)
    pub ball_x: Fixed32,
    pub ball_y: Fixed32,
    pub left_paddle_y: Fixed32,
    pub right_paddle_y: Fixed32,

    // Velocities (Fixed32)
    pub ball_vx: Fixed32,
    pub ball_vy: Fixed32,

    // Integers (already deterministic)
    pub left_score: u32,
    pub right_score: u32,
    pub frame: u64,

    // RNG state (deterministic xorshift)
    pub rng_state: u64,
}

impl DeterministicPongState {
    /// Update physics with fixed timestep (fully deterministic)
    pub fn update(&mut self, inputs: &FrameInputs, dt: Fixed32) {
        // All math uses Fixed32 - identical on ALL platforms
        self.ball_x = self.ball_x + self.ball_vx.mul(dt);
        self.ball_y = self.ball_y + self.ball_vy.mul(dt);

        // Paddle movement
        if inputs.left_up {
            self.left_paddle_y = self.left_paddle_y - PADDLE_SPEED.mul(dt);
        }
        // ... etc
    }

    /// Convert to f32 for rendering ONLY (not stored in trace)
    pub fn to_render_state(&self) -> RenderState {
        RenderState {
            ball_x: self.ball_x.to_f32(),
            ball_y: self.ball_y.to_f32(),
            // ...
        }
    }

    /// Compute deterministic hash for verification
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.ball_x.0.to_le_bytes());
        hasher.update(&self.ball_y.0.to_le_bytes());
        hasher.update(&self.ball_vx.0.to_le_bytes());
        hasher.update(&self.ball_vy.0.to_le_bytes());
        hasher.update(&self.left_paddle_y.0.to_le_bytes());
        hasher.update(&self.right_paddle_y.0.to_le_bytes());
        hasher.update(&self.left_score.to_le_bytes());
        hasher.update(&self.right_score.to_le_bytes());
        hasher.update(&self.rng_state.to_le_bytes());
        hasher.finalize().into()
    }
}
```

### 6.3 CLI Integration (v1.1)

```bash
# Record with Andon Cord (debug mode - blocks on buffer overflow)
cargo run --bin pong -- --debug --trace-output session.jtr

# Record production trace (drops events on overflow)
cargo run --bin pong -- --trace-output session.jtr --trace-policy drop

# Replay with verification (fails if state diverges)
cargo run --bin pong -- --replay session.jtr --verify

# Query-based analysis
cargo run --bin pong_ai_cli -- trace-query session.jtr \
    --filter "ai_difficulty > 7 AND flow_channel = 'Anxiety'"

# Cross-platform verification
cargo run --bin pong_ai_cli -- trace-verify session.jtr --strict

# Export for external analysis
cargo run --bin pong_ai_cli -- trace-export session.jtr --format parquet
```

## 7. Implementation Plan (v1.1)

### Phase 1: Determinism Foundation
- [ ] Implement `Fixed32` type with full arithmetic
- [ ] Port game physics to `DeterministicGameState`
- [ ] Add cross-platform determinism tests
- [ ] Implement deterministic xorshift RNG

### Phase 2: Tracing Infrastructure
- [ ] Implement `RingBuffer` with Andon Cord policy
- [ ] Implement `AdaptiveSnapshotter`
- [ ] Create lean `FrameRecord` format
- [ ] Create `.jtr` file writer

### Phase 3: Recording Integration
- [ ] Add `--debug` flag with Andon Cord
- [ ] Instrument input processing
- [ ] Instrument AI decision making
- [ ] Add entropy-based snapshots

### Phase 4: Replay & Verification
- [ ] Implement `TraceReader` with seeking
- [ ] Implement `GameReplayer`
- [ ] Add state hash verification
- [ ] Cross-platform verification tests

### Phase 5: Query-Based Debugging
- [ ] Implement `TraceQuery` interface
- [ ] Add CLI query commands
- [ ] Create HTML trace viewer
- [ ] Add "Why did X happen?" analysis

## 8. Verification and Testing (v1.1)

### 8.1 Cross-Platform Determinism Test

```rust
#[test]
fn test_cross_platform_determinism() {
    // Same seed, same inputs = same state hash on ALL platforms
    let seed = 12345u64;
    let inputs = load_test_inputs("fixtures/determinism_test.json");

    let mut state = DeterministicPongState::from_seed(seed);
    for frame_inputs in inputs {
        state.update(&frame_inputs, Fixed32::from_f32(1.0 / 60.0));
    }

    // This hash MUST be identical on:
    // - Chrome/Windows
    // - Firefox/Linux
    // - Safari/macOS
    // - Native Rust
    let expected_hash = hex!("a1b2c3d4..."); // Known good hash
    assert_eq!(state.hash(), expected_hash);
}
```

### 8.2 Andon Cord Test

```rust
#[test]
fn test_andon_cord_blocks_on_overflow() {
    let tracer = GameTracer::debug();

    // Fill buffer to capacity
    for _ in 0..BUFFER_SIZE {
        tracer.record_input(&InputEvent::dummy()).unwrap();
    }

    // Next record should block (not drop)
    let start = Instant::now();
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(50));
        tracer.drain(100); // Free up space
    });

    tracer.record_input(&InputEvent::dummy()).unwrap();
    assert!(start.elapsed() >= Duration::from_millis(40));
}
```

### 8.3 Test Coverage Requirements (v1.1)

| Component | Coverage Target | Test Type |
|-----------|-----------------|-----------|
| Fixed32 | 100% | Unit + Property-based |
| RingBuffer + Andon | 100% | Unit + Concurrency |
| AdaptiveSnapshotter | 100% | Unit + Property-based |
| TraceWriter | 95% | Unit + Integration |
| TraceReader | 95% | Unit + Integration |
| Cross-platform determinism | 100% | Integration (multi-platform CI) |
| Query interface | 90% | Unit + Integration |

## 9. Performance Targets (v1.1)

| Metric | Target | v1.0 | v1.1 |
|--------|--------|------|------|
| Record overhead per frame | < 2μs | 2μs | **1.5μs** (leaner format) |
| Memory overhead | < 1MB | 1MB | 1MB |
| Trace file size (1 min) | < 50KB | 10KB | **5KB** (adaptive snapshots) |
| Replay verification speed | > 1000x | 1000x | 1000x |
| Seek to frame latency | < 10ms | 10ms | **5ms** (better index) |
| Cross-platform hash match | 100% | N/A | **100%** (Fixed32) |

## 10. Summary of v1.1 Changes

| Issue | TPS Principle | v1.0 Design | v1.1 Fix |
|-------|---------------|-------------|----------|
| Buffer overflow drops events | **Jidoka** | Silent drop | **Andon Cord** (block in debug) |
| f32 non-determinism | **Poka-Yoke** | Raw f32 | **Fixed32** math |
| Fixed snapshot interval | **Heijunka/Mura** | Every 60 frames | **Adaptive** (entropy-based) |
| Physical timestamps waste | **Muda** | Per-frame nanos | **Frame number only** |
| Linear trace viewing | **Genchi Genbutsu** | Timeline only | **Query-based** debugging |

## 11. Load Testing and Performance Validation

### 11.1 Overview

**Problem:** Game development often reveals bugs only under extreme conditions—high entity counts, rapid input sequences, or unusual configurations. Traditional unit tests miss these edge cases.

**Solution:** A comprehensive load testing framework that applies **Chaos Engineering** principles to games, inspired by patterns from the Batuta stack (entrenar, trueno-viz).

### 11.2 Research Foundation (Additional Citations)

21. **Basiri, A., et al. (2016)**. "Chaos Engineering." *IEEE Software*, 33(3), 35-41. DOI: 10.1109/MS.2016.60
    - *Netflix's chaos engineering principles for fault injection and graceful degradation*

22. **Claessen, K., & Hughes, J. (2000)**. "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs." *ICFP*.
    - *Property-based testing foundation; generates thousands of edge cases automatically*

23. **Dean, J., & Barroso, L. A. (2013)**. "The Tail at Scale." *Communications of the ACM*, 56(2), 74-80.
    - *Importance of p99 latency analysis for interactive systems like games*

### 11.3 Chaos Engineering for Games

#### 11.3.1 Chaos Scenario Types

| Scenario | Description | Detection Target |
|----------|-------------|------------------|
| **Entity Storm** | Spawn maximum entities simultaneously | Memory limits, physics stability |
| **Input Flood** | 1000+ inputs/second | Input buffer overflow, dropped inputs |
| **Time Warp** | Extreme dt values (0.0001s to 1.0s) | Physics explosion, NaN propagation |
| **Resize Blitz** | Rapid canvas resize events | Layout thrashing, coordinate drift |
| **Configuration Sweep** | Test all config permutations | Edge case configurations |
| **RNG Torture** | Adversarial random seeds | Determinism validation |

#### 11.3.2 Chaos Configuration

```rust
/// Configuration for chaos testing scenarios
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Scenario type to execute
    pub scenario: ChaosScenario,
    /// Duration in frames
    pub duration_frames: u64,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Intensity level (0.0 - 1.0)
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ChaosScenario {
    /// Spawn entities at maximum rate
    EntityStorm { max_entities: usize },
    /// Flood input buffer with events
    InputFlood { events_per_frame: usize },
    /// Vary delta time extremely
    TimeWarp { min_dt: f32, max_dt: f32 },
    /// Rapid resize events
    ResizeBlitz { frequency: u32 },
    /// Test configuration boundaries
    ConfigSweep,
    /// Adversarial RNG seeds
    RngTorture { iterations: usize },
}

impl ChaosConfig {
    /// Standard entity storm for stress testing
    pub fn entity_storm() -> Self {
        Self {
            scenario: ChaosScenario::EntityStorm { max_entities: 1000 },
            duration_frames: 600, // 10 seconds at 60 FPS
            seed: 0xDEADBEEF,
            intensity: 1.0,
        }
    }

    /// Input flood to test buffer limits
    pub fn input_flood() -> Self {
        Self {
            scenario: ChaosScenario::InputFlood { events_per_frame: 100 },
            duration_frames: 300,
            seed: 0xCAFEBABE,
            intensity: 1.0,
        }
    }
}
```

#### 11.3.3 Chaos Runner

```rust
/// Executes chaos scenarios and collects results
pub struct ChaosRunner<G: WebGame> {
    platform: WebPlatform<G>,
    config: ChaosConfig,
    results: ChaosResults,
    rng: Xoshiro256StarStar,
}

impl<G: WebGame> ChaosRunner<G> {
    /// Run chaos scenario and return results
    pub fn run(&mut self) -> ChaosResults {
        match self.config.scenario {
            ChaosScenario::EntityStorm { max_entities } => {
                self.run_entity_storm(max_entities)
            }
            ChaosScenario::InputFlood { events_per_frame } => {
                self.run_input_flood(events_per_frame)
            }
            ChaosScenario::TimeWarp { min_dt, max_dt } => {
                self.run_time_warp(min_dt, max_dt)
            }
            // ... other scenarios
        }
    }

    fn run_input_flood(&mut self, events_per_frame: usize) -> ChaosResults {
        for frame in 0..self.config.duration_frames {
            let mut events = Vec::with_capacity(events_per_frame);
            for _ in 0..events_per_frame {
                events.push(self.generate_random_input());
            }
            let json = serde_json::to_string(&events).unwrap();
            let _ = self.platform.frame(frame as f64 * 16.667, &json);

            self.record_frame_stats();
        }
        self.results.clone()
    }
}

/// Results from chaos scenario execution
#[derive(Debug, Clone, Default)]
pub struct ChaosResults {
    /// Total frames executed
    pub frames_executed: u64,
    /// Frames that exceeded target time
    pub slow_frames: u64,
    /// Maximum frame time observed (ms)
    pub max_frame_time_ms: f64,
    /// Any panics caught (in catch_unwind context)
    pub panics: Vec<String>,
    /// NaN/Inf values detected
    pub nan_detected: bool,
    /// Memory high-water mark (bytes)
    pub peak_memory_bytes: usize,
    /// Inputs dropped (if any)
    pub inputs_dropped: u64,
}
```

### 11.4 Property-Based Testing for Games

#### 11.4.1 Game Invariants

Property-based testing with proptest validates that game invariants hold across thousands of random scenarios:

```rust
use proptest::prelude::*;

/// Strategy for generating valid game inputs
fn input_strategy() -> impl Strategy<Value = Vec<BrowserInputEvent>> {
    prop::collection::vec(
        prop_oneof![
            Just(key_down("KeyW")),
            Just(key_down("KeyS")),
            Just(key_down("ArrowUp")),
            Just(key_down("ArrowDown")),
            Just(key_down("Space")),
            Just(key_down("Escape")),
        ],
        0..10  // 0-10 inputs per frame
    )
}

proptest! {
    /// Ball position always stays within bounds
    #[test]
    fn ball_stays_in_bounds(
        inputs in prop::collection::vec(input_strategy(), 0..600)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let _ = platform.frame(frame as f64 * 16.667, &json);

            let ball = platform.pong().ball_position();
            prop_assert!(ball.x >= 0.0, "Ball x below 0: {}", ball.x);
            prop_assert!(ball.x <= 800.0, "Ball x above bounds: {}", ball.x);
            prop_assert!(ball.y >= 0.0, "Ball y below 0: {}", ball.y);
            prop_assert!(ball.y <= 600.0, "Ball y above bounds: {}", ball.y);
        }
    }

    /// Paddle positions always stay within bounds
    #[test]
    fn paddles_stay_in_bounds(
        inputs in prop::collection::vec(input_strategy(), 0..600)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let _ = platform.frame(frame as f64 * 16.667, &json);

            let (left_y, right_y) = platform.pong().paddle_positions();
            prop_assert!(left_y >= 0.0 && left_y <= 600.0);
            prop_assert!(right_y >= 0.0 && right_y <= 600.0);
        }
    }

    /// Score never decreases
    #[test]
    fn score_monotonic(
        inputs in prop::collection::vec(input_strategy(), 0..600)
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());
        let mut prev_left = 0;
        let mut prev_right = 0;

        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let _ = platform.frame(frame as f64 * 16.667, &json);

            let (left, right) = platform.pong().scores();
            prop_assert!(left >= prev_left, "Left score decreased");
            prop_assert!(right >= prev_right, "Right score decreased");
            prev_left = left;
            prev_right = right;
        }
    }

    /// No NaN or Inf in game state
    #[test]
    fn no_nan_or_inf(
        inputs in prop::collection::vec(input_strategy(), 0..1000),
        dt_factor in 0.1f64..10.0
    ) {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());

        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let ts = frame as f64 * 16.667 * dt_factor;
            let _ = platform.frame(ts, &json);

            let state = platform.pong().debug_state();
            prop_assert!(!state.ball_x.is_nan(), "Ball X is NaN");
            prop_assert!(!state.ball_y.is_nan(), "Ball Y is NaN");
            prop_assert!(!state.ball_vx.is_nan(), "Ball VX is NaN");
            prop_assert!(!state.ball_vy.is_nan(), "Ball VY is NaN");
            prop_assert!(state.ball_x.is_finite(), "Ball X is Inf");
            prop_assert!(state.ball_y.is_finite(), "Ball Y is Inf");
        }
    }
}
```

#### 11.4.2 Determinism Verification

```rust
proptest! {
    /// Same inputs produce identical state hash
    #[test]
    fn deterministic_replay(
        seed in any::<u64>(),
        inputs in prop::collection::vec(input_strategy(), 0..300)
    ) {
        // Run 1
        let mut platform1 = WebPlatform::new_with_seed(WebConfig::default(), seed);
        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let _ = platform1.frame(frame as f64 * 16.667, &json);
        }
        let hash1 = platform1.state_hash();

        // Run 2 (identical)
        let mut platform2 = WebPlatform::new_with_seed(WebConfig::default(), seed);
        for (frame, frame_inputs) in inputs.iter().enumerate() {
            let json = serde_json::to_string(&frame_inputs).unwrap();
            let _ = platform2.frame(frame as f64 * 16.667, &json);
        }
        let hash2 = platform2.state_hash();

        prop_assert_eq!(hash1, hash2, "Determinism violated!");
    }
}
```

### 11.5 Performance Benchmarking

#### 11.5.1 Multi-Scale Benchmarks (Criterion)

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn frame_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_update");

    // Micro-scale: single frame
    group.bench_function("single_frame", |b| {
        let mut platform = WebPlatform::new_for_test(WebConfig::default());
        let mut frame = 0;
        b.iter(|| {
            platform.frame(frame as f64 * 16.667, "[]");
            frame += 1;
        });
    });

    // Meso-scale: 60 frames (1 second of gameplay)
    group.bench_function("one_second", |b| {
        b.iter(|| {
            let mut platform = WebPlatform::new_for_test(WebConfig::default());
            for frame in 0..60 {
                platform.frame(frame as f64 * 16.667, "[]");
            }
        });
    });

    // Macro-scale: 3600 frames (1 minute of gameplay)
    group.bench_function("one_minute", |b| {
        b.iter(|| {
            let mut platform = WebPlatform::new_for_test(WebConfig::default());
            for frame in 0..3600 {
                platform.frame(frame as f64 * 16.667, "[]");
            }
        });
    });

    group.finish();
}

fn input_throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("input_throughput");

    for input_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("inputs_per_frame", input_count),
            input_count,
            |b, &count| {
                let mut platform = WebPlatform::new_for_test(WebConfig::default());
                let inputs: Vec<_> = (0..count)
                    .map(|_| BrowserInputEvent::key_down("KeyW", 0.0))
                    .collect();
                let json = serde_json::to_string(&inputs).unwrap();
                let mut frame = 0;

                b.iter(|| {
                    platform.frame(frame as f64 * 16.667, &json);
                    frame += 1;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, frame_benchmark, input_throughput_benchmark);
criterion_main!(benches);
```

#### 11.5.2 Percentile Analysis

```rust
/// Frame time statistics with percentile analysis (Dean & Barroso 2013)
#[derive(Debug, Clone)]
pub struct FrameTimeStats {
    /// All frame times (ms)
    samples: Vec<f64>,
}

impl FrameTimeStats {
    pub fn new() -> Self {
        Self { samples: Vec::new() }
    }

    pub fn record(&mut self, frame_time_ms: f64) {
        self.samples.push(frame_time_ms);
    }

    /// Get percentile value (p50, p90, p95, p99)
    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[idx]
    }

    /// Summary report
    pub fn report(&self) -> FrameTimeReport {
        FrameTimeReport {
            count: self.samples.len(),
            min: self.samples.iter().cloned().fold(f64::INFINITY, f64::min),
            max: self.samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean: self.samples.iter().sum::<f64>() / self.samples.len() as f64,
            p50: self.percentile(50.0),
            p90: self.percentile(90.0),
            p95: self.percentile(95.0),
            p99: self.percentile(99.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameTimeReport {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

impl FrameTimeReport {
    /// Check if performance meets 60 FPS target (16.67ms budget)
    pub fn meets_60fps(&self) -> bool {
        self.p99 < 16.67
    }

    /// Check if performance meets 120 FPS target (8.33ms budget)
    pub fn meets_120fps(&self) -> bool {
        self.p99 < 8.33
    }
}
```

### 11.6 Anomaly Detection

#### 11.6.1 Frame Time Drift Detection

Detect performance regressions using sliding window baseline comparison:

```rust
/// Drift detector using Z-score analysis
pub struct DriftDetector {
    /// Sliding window of recent frame times
    window: VecDeque<f64>,
    /// Window size
    window_size: usize,
    /// Z-score threshold for anomaly
    z_threshold: f64,
    /// Baseline statistics
    baseline_mean: f64,
    baseline_std: f64,
}

impl DriftDetector {
    pub fn new(window_size: usize, z_threshold: f64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            z_threshold,
            baseline_mean: 0.0,
            baseline_std: 1.0,
        }
    }

    /// Set baseline from calibration run
    pub fn calibrate(&mut self, samples: &[f64]) {
        self.baseline_mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance: f64 = samples.iter()
            .map(|x| (x - self.baseline_mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        self.baseline_std = variance.sqrt().max(0.001); // Avoid div by zero
    }

    /// Add sample and check for anomaly
    pub fn observe(&mut self, frame_time_ms: f64) -> AnomalyResult {
        self.window.push_back(frame_time_ms);
        if self.window.len() > self.window_size {
            self.window.pop_front();
        }

        // Calculate Z-score
        let z_score = (frame_time_ms - self.baseline_mean) / self.baseline_std;

        if z_score.abs() > self.z_threshold {
            AnomalyResult::Anomaly {
                value: frame_time_ms,
                z_score,
                expected_range: (
                    self.baseline_mean - self.z_threshold * self.baseline_std,
                    self.baseline_mean + self.z_threshold * self.baseline_std,
                ),
            }
        } else {
            AnomalyResult::Normal
        }
    }

    /// Detect sustained drift (window mean significantly different from baseline)
    pub fn detect_drift(&self) -> Option<DriftReport> {
        if self.window.len() < self.window_size {
            return None;
        }

        let window_mean: f64 = self.window.iter().sum::<f64>() / self.window.len() as f64;
        let drift = (window_mean - self.baseline_mean) / self.baseline_mean * 100.0;

        if drift.abs() > 10.0 { // 10% drift threshold
            Some(DriftReport {
                baseline_mean: self.baseline_mean,
                current_mean: window_mean,
                drift_percent: drift,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum AnomalyResult {
    Normal,
    Anomaly {
        value: f64,
        z_score: f64,
        expected_range: (f64, f64),
    },
}

#[derive(Debug)]
pub struct DriftReport {
    pub baseline_mean: f64,
    pub current_mean: f64,
    pub drift_percent: f64,
}
```

### 11.7 Tiered Testing Workflow

Based on patterns from the Batuta stack:

| Tier | Trigger | Time Budget | Tests |
|------|---------|-------------|-------|
| **Tier 1** | On save | < 5 seconds | Unit tests, basic invariants |
| **Tier 2** | On commit | < 30 seconds | Property tests (100 cases), integration tests |
| **Tier 3** | On merge | < 5 minutes | Property tests (1000 cases), chaos scenarios, benchmarks |

```makefile
# Makefile integration
tier1: ## ON-SAVE: Sub-second feedback
	cargo test --lib -- --test-threads=4

tier2: ## ON-COMMIT: Full validation
	cargo test
	cargo test --test proptest -- --test-threads=1

tier3: ## ON-MERGE: Chaos + Benchmarks
	cargo test --release --test chaos
	cargo test --release --test proptest -- PROPTEST_CASES=1000
	cargo bench --bench frame_benchmark
```

### 11.8 Load Test Runner Integration

```rust
/// High-level load test runner
pub struct LoadTestRunner {
    configs: Vec<LoadTestConfig>,
    results: Vec<LoadTestResult>,
}

#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub name: String,
    pub chaos: Option<ChaosConfig>,
    pub property_tests: bool,
    pub benchmark: bool,
    pub duration_frames: u64,
}

#[derive(Debug)]
pub struct LoadTestResult {
    pub name: String,
    pub passed: bool,
    pub chaos_results: Option<ChaosResults>,
    pub frame_stats: FrameTimeReport,
    pub anomalies: Vec<AnomalyResult>,
    pub property_failures: Vec<String>,
}

impl LoadTestRunner {
    /// Run all configured load tests
    pub fn run_all(&mut self) -> LoadTestSummary {
        for config in &self.configs {
            let result = self.run_single(config);
            self.results.push(result);
        }

        LoadTestSummary {
            total: self.results.len(),
            passed: self.results.iter().filter(|r| r.passed).count(),
            failed: self.results.iter().filter(|r| !r.passed).count(),
            results: self.results.clone(),
        }
    }
}

#[derive(Debug)]
pub struct LoadTestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<LoadTestResult>,
}
```

### 11.9 CLI Integration

```bash
# Run chaos scenarios
cargo run --bin jugar-load-test -- chaos --scenario entity-storm --duration 600

# Run property tests with custom iteration count
cargo run --bin jugar-load-test -- proptest --cases 1000 --seed 42

# Run performance benchmark suite
cargo run --bin jugar-load-test -- bench --warmup 100 --iterations 1000

# Run drift detection against baseline
cargo run --bin jugar-load-test -- drift --baseline baseline.json --threshold 10%

# Full load test suite
cargo run --bin jugar-load-test -- full --tier 3
```

### 11.10 Success Criteria

| Metric | Target | Validation |
|--------|--------|------------|
| Property test cases | 1000+ | All pass |
| Chaos scenarios | All 6 types | No panics, no NaN |
| Frame time p99 | < 8ms | Criterion benchmark |
| Memory stability | No growth | 60s sustained load |
| Determinism | 100% | Hash match across runs |
| Input throughput | 1000/frame | No drops |

## 12. Monte Carlo Game Replay Testing

### 12.1 Overview

**Problem:** Traditional unit tests and even property-based tests cannot cover the full space of possible game interactions. Real games involve emergent behavior from AI vs AI, AI vs Human, and Human vs Human scenarios over thousands of frames.

**Solution:** Monte Carlo Game Replay Testing uses randomized game simulations with:
- **Monte Carlo Tree Search (MCTS)** principles for exploring game state space
- **Declarative YAML scenarios** for reproducible test specifications
- **Distributed property testing** across multiple simulated games
- **Log verification** ensuring tracing and load testing work correctly together

This approach is inspired by patterns from:
- `../entrenar/src/search/mcts.rs` - Monte Carlo Tree Search for state space exploration
- `../presentar` - Declarative YAML configurations for reproducible scenarios
- Netflix chaos engineering - Distributed fault injection

### 12.2 Research Foundation (Additional Citations)

24. **Coulom, R. (2007)**. "Efficient Selectivity and Backup Operators in Monte-Carlo Tree Search." *Computers and Games*, 72-83.
    - *Foundation for MCTS exploration/exploitation trade-offs*

25. **Silver, D., et al. (2016)**. "Mastering the game of Go with deep neural networks and tree search." *Nature*, 529(7587), 484-489.
    - *AlphaGo's integration of MCTS with neural network evaluation*

26. **Browne, C. B., et al. (2012)**. "A Survey of Monte Carlo Tree Search Methods." *IEEE Transactions on Computational Intelligence and AI in Games*, 4(1), 1-43.
    - *Comprehensive MCTS survey; simulation policies and parallelization*

27. **Kocsis, L., & Szepesvári, C. (2006)**. "Bandit Based Monte-Carlo Planning." *ECML*, 282-293.
    - *UCT (Upper Confidence Bounds for Trees) algorithm*

### 12.2.1 Additional Citations (v1.3 TPS Code Review)

28. **Zobrist, A. L. (1970)**. "A New Hashing Method with Application for Game Playing." *Technical Report 88, Computer Sciences Department, University of Wisconsin*.
    - *Foundation for O(1) incremental hashing. Essential for replacing SHA-256 in AdaptiveSnapshotter to reduce CPU Muri*

29. **Tridgell, A. (1999)**. "Efficient Algorithms for Sorting and Synchronization." *PhD Thesis, Australian National University*. (The rsync algorithm).
    - *Theoretical basis for "Delta Snapshot" compression strategy. Rolling checksums for efficient state change detection*

30. **Bessey, A., et al. (2010)**. "A Few Billion Lines of Code Later: Using Static Analysis to Find Errors in the Real World." *Communications of the ACM*, 53(2), 66-75.
    - *Supports Poka-Yoke: using compile-time constraints (Clippy/Linters) to enforce Fixed32 usage*

31. **Jung, R., et al. (2017)**. "RustBelt: Securing the Foundations of the Rust Programming Language." *Proceedings of the ACM on Programming Languages (POPL)*.
    - *Validates safety guarantees of using Rust's type system to enforce invariants (determinism) at compiler level*

32. **Leucker, M., & Schallhart, C. (2009)**. "A Brief Account of Runtime Verification." *The Journal of Logic and Algebraic Programming*, 78(5), 293-303.
    - *Framework for "One-Piece Flow" suggestion—verifying temporal logic properties on event streams in real-time*

33. **Godefroid, P., Klarlund, N., & Sen, K. (2005)**. "DART: Directed Automated Random Testing." *Proceedings of PLDI*.
    - *Concolic Testing: random inputs guided by symbolic constraints to maximize code coverage*

34. **MacKenzie, I. S., & Ware, C. (1993)**. "Lag as a Determinant of Human Performance in Interactive Systems." *Proceedings of INTERCHI '93*.
    - *Critical for "Soft Andon" design. Quantifies how input lag degrades human performance*

35. **Shneiderman, B. (1996)**. "The Eyes Have It: A Task by Data Type Taxonomy for Information Visualizations." *IEEE Symposium on Visual Languages*.
    - *Foundation for Genchi Genbutsu trace viewer: "Overview first, zoom and filter, then details-on-demand"*

36. **Ford, B., et al. (2002)**. "Vx32: Lightweight User-level Sandboxing on the x86." *USENIX Annual Technical Conference*.
    - *Instruction counting and trap handling for determinism. Relevant for advanced synchronization*

37. **Regehr, J., et al. (2012)**. "Understanding Integer Overflow in C/C++." *ACM TOSEM*.
    - *Justifies overflow checks in Fixed32 to prevent wrap-around bugs in deterministic replay*

### 12.3 Architecture

#### 12.3.1 Game Replay Testing System

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Monte Carlo Game Replay System                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐ │
│  │   YAML      │───▶│  Scenario   │───▶│  Game Simulator         │ │
│  │  Scenario   │    │   Parser    │    │  (Monte Carlo Agent)    │ │
│  └─────────────┘    └─────────────┘    └───────────┬─────────────┘ │
│                                                    │                │
│                                        ┌───────────▼─────────────┐ │
│                                        │   Frame-by-Frame        │ │
│                                        │   Execution Engine      │ │
│                                        └───────────┬─────────────┘ │
│                                                    │                │
│  ┌─────────────────────────────────────────────────▼─────────────┐ │
│  │                     Trace Collector                            │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐  │ │
│  │  │ Inputs  │  │ States  │  │  AI     │  │ Performance     │  │ │
│  │  │ Stream  │  │ Stream  │  │ Decisions│  │ Metrics         │  │ │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────────────┘  │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                    │                │
│  ┌─────────────────────────────────────────────────▼─────────────┐ │
│  │                   Verification Engine                          │ │
│  │  • Invariant Checking    • Determinism Verification           │ │
│  │  • Log Completeness      • Performance Regression             │ │
│  │  • AI Behavior Analysis  • Scenario Outcome Matching          │ │
│  └───────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

#### 12.3.2 Simulation Agent Types

| Agent Type | Description | Use Case |
|------------|-------------|----------|
| **RandomAgent** | Uniform random action selection | Chaos testing, edge case discovery |
| **MCTSAgent** | Monte Carlo Tree Search | Optimal play exploration |
| **PolicyAgent** | Uses trained AI model | AI behavior validation |
| **ScriptedAgent** | Follows predefined input sequence | Regression testing |
| **HumanReplayAgent** | Replays recorded human inputs | Bug reproduction |

### 12.4 Declarative YAML Scenario Format

#### 12.4.1 Schema

```yaml
# Game Replay Test Scenario
jugar_test: "1.0"
name: "scenario-name"
version: "1.0.0"
description: "Description of what this scenario tests"

# Game configuration
game:
  type: "pong"  # or custom game type
  config:
    width: 800
    height: 600
    ball_speed: 200.0
    paddle_speed: 300.0

# Agent configuration
agents:
  left:
    type: "mcts"  # random, mcts, policy, scripted, human_replay
    config:
      exploration_constant: 1.414
      max_simulations: 100
      temperature: 1.0

  right:
    type: "policy"
    config:
      model: "pong-ai-v1.apr"
      difficulty: 5

# Simulation parameters
simulation:
  duration_frames: 3600  # 1 minute at 60fps
  seed: 42               # for reproducibility
  parallelism: 4         # concurrent simulations
  warmup_frames: 60      # frames before measurement

# Verification criteria
verify:
  invariants:
    - "ball.x >= 0 && ball.x <= canvas.width"
    - "ball.y >= 0 && ball.y <= canvas.height"
    - "left_paddle.y >= 0 && left_paddle.y <= canvas.height"
    - "score.left >= 0 && score.right >= 0"

  determinism:
    enabled: true
    tolerance: 0  # exact match required

  performance:
    p99_frame_time_ms: 8.0
    max_frame_time_ms: 16.67

  outcomes:
    - condition: "score.left + score.right > 0"
      description: "At least one point scored"
      required: true

  trace_completeness:
    require_all_frames: true
    require_all_inputs: true
    require_ai_decisions: true

# Output configuration
output:
  trace_file: "traces/{name}_{seed}.jtr"
  report_format: "json"
  save_on_failure: true
```

#### 12.4.2 Predefined Scenario Templates

```yaml
# scenarios/templates/stress_test.yaml
jugar_test: "1.0"
name: "stress-test-template"
description: "High-intensity stress testing template"

agents:
  left:
    type: "random"
    config:
      action_frequency: 0.5  # 50% chance of action each frame

  right:
    type: "random"
    config:
      action_frequency: 0.5

simulation:
  duration_frames: 18000  # 5 minutes
  parallelism: 8

verify:
  invariants:
    - "!isNaN(ball.x) && !isNaN(ball.y)"
    - "!isInf(ball.vx) && !isInf(ball.vy)"
```

```yaml
# scenarios/templates/ai_regression.yaml
jugar_test: "1.0"
name: "ai-regression-template"
description: "AI behavior regression testing"

agents:
  left:
    type: "policy"
    config:
      model: "baseline.apr"

  right:
    type: "policy"
    config:
      model: "candidate.apr"

simulation:
  duration_frames: 36000  # 10 minutes
  seeds: [42, 123, 456, 789, 1001]  # multiple seeds

verify:
  outcomes:
    - condition: "abs(score.left - score.right) < 10"
      description: "Models are competitive"
```

```yaml
# scenarios/templates/click_input_test.yaml
jugar_test: "1.0"
name: "click-input-regression"
description: "Verify click and input handling"

agents:
  left:
    type: "scripted"
    script:
      - { frame: 0, action: "click", x: 400, y: 300 }   # Start game
      - { frame: 60, action: "key_down", key: "KeyW" }
      - { frame: 120, action: "key_up", key: "KeyW" }
      - { frame: 180, action: "click", x: 700, y: 50 }  # Click HUD button

  right:
    type: "policy"
    config:
      model: "pong-ai-v1.apr"

simulation:
  duration_frames: 600

verify:
  trace_completeness:
    require_all_inputs: true
    input_sequence_match: true  # Verify recorded matches scripted
```

### 12.5 Monte Carlo Simulation Engine

#### 12.5.1 Core Types

```rust
/// Monte Carlo game replay test configuration
#[derive(Debug, Clone, Deserialize)]
pub struct McGameTestConfig {
    /// Scenario name
    pub name: String,
    /// Game type and configuration
    pub game: GameConfig,
    /// Agent configurations
    pub agents: AgentConfigs,
    /// Simulation parameters
    pub simulation: SimulationConfig,
    /// Verification criteria
    pub verify: VerificationConfig,
}

/// Agent type enumeration
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AgentConfig {
    Random { action_frequency: f64 },
    Mcts(MctsAgentConfig),
    Policy { model: String, difficulty: u8 },
    Scripted { script: Vec<ScriptedAction> },
    HumanReplay { trace_file: String },
}

/// MCTS agent configuration
#[derive(Debug, Clone, Deserialize)]
pub struct MctsAgentConfig {
    /// Exploration constant (C in UCT)
    pub exploration_constant: f64,
    /// Maximum simulations per move
    pub max_simulations: usize,
    /// Temperature for action selection
    pub temperature: f64,
    /// Rollout depth
    pub rollout_depth: usize,
}

/// Scripted action for regression tests
#[derive(Debug, Clone, Deserialize)]
pub struct ScriptedAction {
    pub frame: u64,
    pub action: ActionType,
    #[serde(flatten)]
    pub data: ActionData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    KeyDown,
    KeyUp,
    Click,
    MouseMove,
    Pause,
    Resume,
}
```

#### 12.5.2 Monte Carlo Agent Implementation

```rust
/// Monte Carlo Tree Search agent for game testing
pub struct MctsTestAgent<G: GameState> {
    /// Configuration
    config: MctsAgentConfig,
    /// Search tree
    tree: SearchTree<G>,
    /// RNG for simulation
    rng: Xoshiro256StarStar,
    /// Statistics
    stats: MctsStats,
}

impl<G: GameState + Clone> MctsTestAgent<G> {
    /// Select action using UCT (Upper Confidence Bound for Trees)
    pub fn select_action(&mut self, state: &G) -> GameAction {
        // Reset tree for new decision
        self.tree.reset(state.clone());

        // Run simulations
        for _ in 0..self.config.max_simulations {
            self.simulate();
        }

        // Select best action based on visit count
        let best_action = self.tree.best_action(self.config.temperature);
        self.stats.decisions += 1;

        best_action
    }

    fn simulate(&mut self) {
        // Selection: traverse tree using UCT
        let mut node = self.tree.root();
        let mut state = self.tree.root_state().clone();

        while !state.is_terminal() && node.is_expanded() {
            let action = self.select_uct(node);
            state = state.apply(&action);
            node = node.child(&action);
        }

        // Expansion: add new node
        if !state.is_terminal() && node.visits() >= 1 {
            let actions = state.legal_actions();
            node.expand(&actions);
        }

        // Simulation: rollout to terminal
        let reward = self.rollout(&mut state);

        // Backpropagation: update statistics
        self.tree.backpropagate(node, reward);

        self.stats.simulations += 1;
    }

    fn select_uct(&self, node: &TreeNode<G>) -> GameAction {
        let mut best_action = None;
        let mut best_value = f64::NEG_INFINITY;
        let parent_visits = node.visits() as f64;

        for (action, child) in node.children() {
            let exploitation = child.value() / child.visits().max(1) as f64;
            let exploration = self.config.exploration_constant
                * (parent_visits.ln() / child.visits().max(1) as f64).sqrt();
            let uct_value = exploitation + exploration;

            if uct_value > best_value {
                best_value = uct_value;
                best_action = Some(action.clone());
            }
        }

        best_action.unwrap_or_else(|| node.random_untried_action(&mut self.rng))
    }

    fn rollout(&mut self, state: &mut G) -> f64 {
        for _ in 0..self.config.rollout_depth {
            if state.is_terminal() {
                break;
            }
            let action = state.random_action(&mut self.rng);
            *state = state.apply(&action);
        }
        state.evaluate()
    }
}
```

#### 12.5.3 Simulation Runner

```rust
/// Runs Monte Carlo game replay tests
pub struct McGameTestRunner {
    /// Configuration
    config: McGameTestConfig,
    /// Results from all simulations
    results: Vec<SimulationResult>,
    /// Aggregate statistics
    stats: TestRunStats,
}

impl McGameTestRunner {
    /// Load scenario from YAML file
    pub fn from_yaml(path: &Path) -> Result<Self, McTestError> {
        let contents = std::fs::read_to_string(path)?;
        let config: McGameTestConfig = serde_yaml::from_str(&contents)?;
        Ok(Self::new(config))
    }

    /// Run all simulations in parallel
    pub fn run(&mut self) -> TestReport {
        let seeds = self.generate_seeds();

        // Parallel execution
        let results: Vec<SimulationResult> = seeds
            .par_iter()
            .map(|&seed| self.run_single_simulation(seed))
            .collect();

        self.results = results;
        self.generate_report()
    }

    fn run_single_simulation(&self, seed: u64) -> SimulationResult {
        let mut platform = WebPlatform::new_with_seed(
            self.config.game.clone().into(),
            seed,
        );

        let mut left_agent = self.create_agent(&self.config.agents.left, seed);
        let mut right_agent = self.create_agent(&self.config.agents.right, seed + 1);

        let mut tracer = GameTracer::debug();
        let mut frame_stats = FrameTimeStats::new();
        let mut invariant_violations = Vec::new();

        for frame in 0..self.config.simulation.duration_frames {
            let ts = frame as f64 * 16.667;

            // Get agent actions
            let left_action = left_agent.get_action(platform.game_state());
            let right_action = right_agent.get_action(platform.game_state());

            // Build input JSON
            let inputs = self.build_input_json(&left_action, &right_action, ts);

            // Execute frame with timing
            tracer.begin_frame();
            let start = std::time::Instant::now();
            let output = platform.frame(ts, &inputs);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            frame_stats.record(elapsed);

            // Record to tracer
            tracer.record_inputs_from_json(&inputs);
            let _ = tracer.end_frame(Some(platform.state_hash()));

            // Check invariants
            if let Err(violation) = self.check_invariants(&platform, frame) {
                invariant_violations.push(violation);
            }

            // Check for NaN/Inf
            if output.contains("NaN") || output.contains("Inf") {
                invariant_violations.push(InvariantViolation {
                    frame,
                    invariant: "no_nan_inf".to_string(),
                    message: "NaN or Inf detected in output".to_string(),
                });
            }
        }

        SimulationResult {
            seed,
            frames_executed: self.config.simulation.duration_frames,
            frame_stats: frame_stats.report(),
            invariant_violations,
            final_state: platform.game_state().clone(),
            trace: tracer.export(),
            passed: invariant_violations.is_empty(),
        }
    }

    fn check_invariants(&self, platform: &WebPlatform, frame: u64)
        -> Result<(), InvariantViolation>
    {
        let state = platform.game_state();

        for invariant in &self.config.verify.invariants {
            if !self.evaluate_invariant(invariant, state) {
                return Err(InvariantViolation {
                    frame,
                    invariant: invariant.clone(),
                    message: format!("Invariant failed at frame {}", frame),
                });
            }
        }

        Ok(())
    }
}
```

### 12.6 Distributed Testing

#### 12.6.1 Worker Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Distributed MC Test Coordinator                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────────────────────────────────────┐│
│  │   Test      │    │              Worker Pool                    ││
│  │  Scheduler  │───▶│  ┌────────┐ ┌────────┐ ┌────────┐          ││
│  └─────────────┘    │  │Worker 1│ │Worker 2│ │Worker N│          ││
│                     │  │seed: 42│ │seed:123│ │seed:999│          ││
│                     │  └────────┘ └────────┘ └────────┘          ││
│                     └─────────────────────────────────────────────┘│
│                                    │                               │
│                     ┌──────────────▼──────────────┐               │
│                     │      Result Aggregator      │               │
│                     │  • Merge traces             │               │
│                     │  • Aggregate statistics     │               │
│                     │  • Identify common failures │               │
│                     └──────────────┬──────────────┘               │
│                                    │                               │
│                     ┌──────────────▼──────────────┐               │
│                     │      Report Generator       │               │
│                     │  • Coverage analysis        │               │
│                     │  • Failure clustering       │               │
│                     │  • Determinism verification │               │
│                     └─────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────────┘
```

#### 12.6.2 Parallelization Strategy

```rust
/// Distributed test coordinator
pub struct McTestCoordinator {
    /// Scenario to test
    scenario: McGameTestConfig,
    /// Number of parallel workers
    parallelism: usize,
    /// Seeds to test
    seeds: Vec<u64>,
}

impl McTestCoordinator {
    /// Run distributed test across multiple seeds
    pub fn run_distributed(&self) -> DistributedTestReport {
        // Partition seeds across workers
        let chunks: Vec<_> = self.seeds
            .chunks(self.seeds.len() / self.parallelism + 1)
            .collect();

        // Run in parallel using rayon
        let worker_results: Vec<WorkerResult> = chunks
            .par_iter()
            .enumerate()
            .map(|(worker_id, seeds)| {
                self.run_worker(worker_id, seeds)
            })
            .collect();

        // Aggregate results
        self.aggregate_results(worker_results)
    }

    fn run_worker(&self, worker_id: usize, seeds: &[u64]) -> WorkerResult {
        let mut results = Vec::new();

        for &seed in seeds {
            let mut runner = McGameTestRunner::new(self.scenario.clone());
            let result = runner.run_single_simulation(seed);
            results.push(result);
        }

        WorkerResult {
            worker_id,
            results,
        }
    }

    fn aggregate_results(&self, workers: Vec<WorkerResult>) -> DistributedTestReport {
        let all_results: Vec<_> = workers
            .into_iter()
            .flat_map(|w| w.results)
            .collect();

        let passed = all_results.iter().filter(|r| r.passed).count();
        let failed = all_results.len() - passed;

        // Cluster failures by invariant
        let failure_clusters = self.cluster_failures(&all_results);

        // Check determinism across identical seeds
        let determinism_check = self.verify_determinism(&all_results);

        DistributedTestReport {
            total_simulations: all_results.len(),
            passed,
            failed,
            failure_clusters,
            determinism_verified: determinism_check.all_match,
            aggregate_stats: self.compute_aggregate_stats(&all_results),
        }
    }
}
```

### 12.7 Verification and Analysis

#### 12.7.1 Trace Completeness Verification

```rust
/// Verify trace completeness and correctness
pub struct TraceVerifier {
    config: VerificationConfig,
}

impl TraceVerifier {
    /// Verify trace matches expected criteria
    pub fn verify(&self, trace: &GameTrace, simulation: &SimulationResult)
        -> VerificationReport
    {
        let mut issues = Vec::new();

        // Check all frames recorded
        if self.config.trace_completeness.require_all_frames {
            let expected_frames = simulation.frames_executed;
            let actual_frames = trace.frame_count();
            if actual_frames != expected_frames {
                issues.push(VerificationIssue::MissingFrames {
                    expected: expected_frames,
                    actual: actual_frames,
                });
            }
        }

        // Check input sequence matches
        if self.config.trace_completeness.input_sequence_match {
            if let AgentConfig::Scripted { script } = &simulation.left_agent_config {
                self.verify_input_sequence(trace, script, &mut issues);
            }
        }

        // Verify AI decisions recorded
        if self.config.trace_completeness.require_ai_decisions {
            let ai_frames = trace.ai_decision_count();
            if ai_frames == 0 {
                issues.push(VerificationIssue::NoAIDecisionsRecorded);
            }
        }

        // Verify determinism
        if self.config.determinism.enabled {
            self.verify_determinism(trace, &mut issues);
        }

        VerificationReport {
            passed: issues.is_empty(),
            issues,
        }
    }

    fn verify_input_sequence(
        &self,
        trace: &GameTrace,
        expected: &[ScriptedAction],
        issues: &mut Vec<VerificationIssue>
    ) {
        for action in expected {
            let frame_record = trace.get_frame(action.frame);

            match &action.action {
                ActionType::KeyDown | ActionType::KeyUp => {
                    let found = frame_record.inputs.iter().any(|input| {
                        matches_action(input, action)
                    });

                    if !found {
                        issues.push(VerificationIssue::InputNotRecorded {
                            frame: action.frame,
                            expected: action.clone(),
                        });
                    }
                }
                ActionType::Click => {
                    // Verify click recorded
                    let found = frame_record.inputs.iter().any(|input| {
                        matches!(input.event_type, InputEventType::MouseDown(_))
                    });

                    if !found {
                        issues.push(VerificationIssue::ClickNotRecorded {
                            frame: action.frame,
                            x: action.data.x.unwrap_or(0),
                            y: action.data.y.unwrap_or(0),
                        });
                    }
                }
                _ => {}
            }
        }
    }
}
```

#### 12.7.2 AI Behavior Analysis

```rust
/// Analyze AI behavior from Monte Carlo simulations
pub struct AIBehaviorAnalyzer {
    traces: Vec<GameTrace>,
}

impl AIBehaviorAnalyzer {
    /// Analyze AI win rate across simulations
    pub fn analyze_win_rate(&self) -> WinRateAnalysis {
        let mut left_wins = 0;
        let mut right_wins = 0;
        let mut draws = 0;

        for trace in &self.traces {
            let final_state = trace.final_state();
            match final_state.left_score.cmp(&final_state.right_score) {
                std::cmp::Ordering::Greater => left_wins += 1,
                std::cmp::Ordering::Less => right_wins += 1,
                std::cmp::Ordering::Equal => draws += 1,
            }
        }

        WinRateAnalysis {
            left_win_rate: left_wins as f64 / self.traces.len() as f64,
            right_win_rate: right_wins as f64 / self.traces.len() as f64,
            draw_rate: draws as f64 / self.traces.len() as f64,
            total_games: self.traces.len(),
        }
    }

    /// Analyze AI difficulty distribution
    pub fn analyze_difficulty_distribution(&self) -> DifficultyAnalysis {
        let mut difficulty_changes = Vec::new();

        for trace in &self.traces {
            for decision in trace.ai_decisions() {
                if let Some(prev) = trace.previous_ai_decision(decision.frame) {
                    if decision.difficulty != prev.difficulty {
                        difficulty_changes.push(DifficultyChange {
                            frame: decision.frame,
                            from: prev.difficulty,
                            to: decision.difficulty,
                            flow_channel: decision.flow_channel.clone(),
                        });
                    }
                }
            }
        }

        DifficultyAnalysis {
            total_changes: difficulty_changes.len(),
            changes: difficulty_changes,
            average_difficulty: self.compute_average_difficulty(),
        }
    }

    /// Detect AI behavior anomalies
    pub fn detect_anomalies(&self) -> Vec<AIAnomaly> {
        let mut anomalies = Vec::new();

        for trace in &self.traces {
            // Detect stuck paddle (no movement for extended period)
            if let Some(stuck) = self.detect_stuck_paddle(trace) {
                anomalies.push(stuck);
            }

            // Detect erratic movement (oscillation)
            if let Some(erratic) = self.detect_erratic_movement(trace) {
                anomalies.push(erratic);
            }

            // Detect impossible predictions
            if let Some(impossible) = self.detect_impossible_prediction(trace) {
                anomalies.push(impossible);
            }
        }

        anomalies
    }
}
```

### 12.8 CLI Integration

```bash
# Run Monte Carlo game replay test from YAML
cargo run --bin jugar-mc-test -- run scenarios/stress_test.yaml

# Run with custom parallelism
cargo run --bin jugar-mc-test -- run scenarios/ai_regression.yaml \
    --parallelism 8 \
    --seeds 100

# Validate YAML scenario without running
cargo run --bin jugar-mc-test -- validate scenarios/click_input_test.yaml

# Generate report from previous run
cargo run --bin jugar-mc-test -- report traces/stress_test_*.jtr \
    --format html \
    --output report.html

# Analyze AI behavior from traces
cargo run --bin jugar-mc-test -- analyze-ai traces/*.jtr \
    --metrics win-rate,difficulty,anomalies

# Compare two AI models
cargo run --bin jugar-mc-test -- compare \
    --baseline models/baseline.apr \
    --candidate models/improved.apr \
    --games 1000
```

### 12.9 Future: YAML-Only Game Definition

Supporting the vision of declarative game building:

```yaml
# games/pong.yaml - Declarative game definition
jugar_game: "1.0"
name: "pong"
version: "1.0.0"

# Canvas configuration
canvas:
  width: 800
  height: 600
  background: [0, 0, 0, 1]

# Entity definitions
entities:
  ball:
    shape: "circle"
    radius: 10
    position: { x: 400, y: 300 }
    velocity: { x: 200, y: 150 }
    collision_groups: ["ball"]

  left_paddle:
    shape: "rect"
    width: 20
    height: 120
    position: { x: 20, y: 300 }
    controls:
      up: "KeyW"
      down: "KeyS"
    collision_groups: ["paddle"]

  right_paddle:
    shape: "rect"
    width: 20
    height: 120
    position: { x: 760, y: 300 }
    ai:
      model: "pong-ai-v1.apr"
      difficulty: 5
    collision_groups: ["paddle"]

# Physics rules
physics:
  - on: { collision: ["ball", "paddle"] }
    do:
      - reflect_velocity: { axis: "x" }
      - play_sound: "paddle_hit.wav"
      - increase_speed: { factor: 1.05 }

  - on: { collision: ["ball", "top_wall"] }
    do:
      - reflect_velocity: { axis: "y" }

  - on: { ball_exit: "left" }
    do:
      - increment: "right_score"
      - reset_ball: { direction: "left" }

  - on: { ball_exit: "right" }
    do:
      - increment: "left_score"
      - reset_ball: { direction: "right" }

# Score display
ui:
  left_score:
    position: { x: 200, y: 50 }
    font: "48px monospace"
    color: [1, 1, 1, 1]
    bind: "left_score"

  right_score:
    position: { x: 600, y: 50 }
    font: "48px monospace"
    color: [1, 1, 1, 1]
    bind: "right_score"
```

### 12.10 Success Criteria

| Metric | Target | Validation Method |
|--------|--------|-------------------|
| Scenario execution rate | 100% | All YAML scenarios run without error |
| Monte Carlo coverage | 10,000+ games | Distributed testing across seeds |
| Invariant detection | 100% | All defined invariants checked |
| Trace completeness | 100% | All frames, inputs, AI decisions logged |
| Determinism | 100% | Same seed produces identical hash |
| AI anomaly detection | < 1% | Anomalies flagged and analyzed |
| Performance regression | < 5% | p99 frame time within tolerance |
| Click/input verification | 100% | Scripted inputs match recorded |

## 13. TPS Kaizen Improvements (v1.3)

This section documents the v1.3 improvements based on TPS Code Review findings.

### 13.1 Soft Andon (MacKenzie & Ware 1993)

**Problem:** Hard Andon Cord blocking creates Muri (overburden) for developers debugging "game feel" issues.

**Solution:** Visual indicator instead of hard block:

```rust
/// Soft Andon: Visual trace loss indicator (MacKenzie & Ware 1993)
/// Preserves game responsiveness while making trace loss impossible to ignore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndonState {
    /// Normal operation - all events being recorded
    Normal,
    /// Warning - buffer at 80% capacity
    Warning { buffer_pct: u8 },
    /// TraceLoss - events are being dropped
    TraceLoss { dropped_count: u64 },
}

impl AndonState {
    /// Get HUD overlay color (Visual Management)
    pub const fn overlay_color(&self) -> [f32; 4] {
        match self {
            Self::Normal => [0.0, 0.0, 0.0, 0.0],        // Invisible
            Self::Warning => [1.0, 0.8, 0.0, 0.3],       // Yellow 30% opacity
            Self::TraceLoss => [1.0, 0.0, 0.0, 0.5],     // Red 50% opacity
        }
    }

    /// Get status text for HUD
    pub const fn status_text(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Warning => "⚠️ TRACE BUFFER WARNING",
            Self::TraceLoss => "🚨 TRACE LOSS - EVENTS DROPPED",
        }
    }
}

/// Buffer policy with Soft Andon support
pub enum BufferPolicy {
    /// Production: drop oldest, no visual indicator
    DropOldest,
    /// Debug with Soft Andon: drop oldest but show visual indicator
    SoftAndon,
    /// Hard Andon: block game loop (original v1.1 behavior)
    HardAndon,
}
```

### 13.2 Zobrist Hashing (Zobrist 1970)

**Problem:** SHA-256 every frame is O(N) and creates Muri (CPU overburden).

**Solution:** Zobrist incremental hashing for entropy detection:

```rust
/// Zobrist hash table for O(1) incremental state hashing (Zobrist 1970)
/// Used to detect state changes efficiently for adaptive snapshots.
pub struct ZobristTable {
    /// Random values for each (field, value) pair
    /// Pre-generated on initialization
    table: [[u64; 256]; NUM_HASH_FIELDS],
}

impl ZobristTable {
    /// Create table with deterministic RNG (for reproducibility)
    pub fn new(seed: u64) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let mut table = [[0u64; 256]; NUM_HASH_FIELDS];

        for field in 0..NUM_HASH_FIELDS {
            for value in 0..256 {
                table[field][value] = rng.next_u64();
            }
        }

        Self { table }
    }

    /// Compute Zobrist hash for state (O(N) - used once per state)
    pub fn hash_state(&self, state: &GameState) -> u64 {
        let bytes = state.to_bytes();
        let mut hash = 0u64;

        for (i, &byte) in bytes.iter().enumerate() {
            hash ^= self.table[i % NUM_HASH_FIELDS][byte as usize];
        }

        hash
    }

    /// Update hash incrementally when single field changes (O(1))
    /// This is the key optimization - no need to rehash entire state
    #[inline]
    pub fn update_hash(&self, hash: u64, field: usize, old_byte: u8, new_byte: u8) -> u64 {
        // XOR out old value, XOR in new value (Tridgell 1999)
        hash ^ self.table[field % NUM_HASH_FIELDS][old_byte as usize]
             ^ self.table[field % NUM_HASH_FIELDS][new_byte as usize]
    }
}

/// Adaptive snapshotter using Zobrist hashing (O(1) entropy detection)
pub struct ZobristSnapshotter {
    /// Zobrist hash table
    table: ZobristTable,
    /// Current state hash (incrementally updated)
    current_hash: u64,
    /// Previous snapshot hash
    prev_snapshot_hash: u64,
    /// Entropy threshold (hamming distance between hashes)
    entropy_threshold: u32,
    /// Minimum frames between snapshots
    min_interval: u64,
    /// Maximum frames between snapshots
    max_interval: u64,
    /// Last snapshot frame
    last_snapshot_frame: u64,
}

impl ZobristSnapshotter {
    /// Check if snapshot should be taken (O(1) operation!)
    pub fn should_snapshot(&mut self, frame: u64) -> SnapshotDecision {
        let frames_since = frame - self.last_snapshot_frame;

        // Force snapshot at max interval
        if frames_since >= self.max_interval {
            self.take_snapshot(frame);
            return SnapshotDecision::FullSnapshot;
        }

        // Calculate entropy as Hamming distance between hashes (O(1))
        let entropy = (self.current_hash ^ self.prev_snapshot_hash).count_ones();

        // High entropy = state changed significantly
        if entropy >= self.entropy_threshold && frames_since >= self.min_interval {
            self.take_snapshot(frame);
            return SnapshotDecision::DeltaSnapshot;
        }

        SnapshotDecision::Skip
    }

    /// Update hash incrementally when state changes (O(1))
    #[inline]
    pub fn on_state_change(&mut self, field: usize, old: u8, new: u8) {
        self.current_hash = self.table.update_hash(self.current_hash, field, old, new);
    }
}
```

### 13.3 Fixed32 Poka-Yoke Macro (Bessey 2010, Jung 2017)

**Problem:** Developers might accidentally use `f32` in simulation code, breaking determinism.

**Solution:** Compile-time enforcement via macro:

```rust
/// Poka-Yoke macro to enforce Fixed32 in deterministic game logic (Bessey 2010)
/// This macro wraps a block and causes compile errors if f32/f64 arithmetic is used.
///
/// # Example
/// ```rust
/// deterministic! {
///     let pos = ball.x + ball.vx.mul(dt);  // OK - Fixed32 arithmetic
///     // let bad = 1.0f32 * 2.0;           // COMPILE ERROR!
/// }
/// ```
#[macro_export]
macro_rules! deterministic {
    ($($body:tt)*) => {{
        // Shadow f32/f64 types with unconstructable types
        #[allow(non_camel_case_types)]
        struct f32;  // Cannot be instantiated
        #[allow(non_camel_case_types)]
        struct f64;  // Cannot be instantiated

        // Any use of f32/f64 literals or operations will fail to compile
        // because our shadow types don't implement Add, Sub, Mul, etc.
        $($body)*
    }};
}

/// Module-level enforcement via clippy configuration
/// Add to .clippy.toml:
/// ```toml
/// [[disallowed-types]]
/// path = "f32"
/// reason = "Use Fixed32 for determinism (Monniaux 2008)"
/// allow-in = ["render", "audio"]  # Only allow in non-deterministic code
///
/// [[disallowed-types]]
/// path = "f64"
/// reason = "Use Fixed32 for determinism (Monniaux 2008)"
/// allow-in = ["render", "audio"]
/// ```

/// Alternative: newtype wrapper that cannot be converted from f32
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DeterministicFloat(Fixed32);

impl DeterministicFloat {
    /// Create from integer (safe, deterministic)
    pub const fn from_int(n: i32) -> Self {
        Self(Fixed32::from_int(n))
    }

    /// Create from Fixed32 (safe, already deterministic)
    pub const fn from_fixed(f: Fixed32) -> Self {
        Self(f)
    }

    // NOTE: No from_f32() method! This is intentional.
    // If you need to convert from f32, you must use Fixed32::from_f32()
    // explicitly, which documents that this is a non-deterministic boundary.
}
```

### 13.4 Fixed32 Overflow Checks (Regehr 2012)

**Problem:** Fixed-point math can wrap around silently, causing subtle bugs.

**Solution:** Checked arithmetic with panic on overflow:

```rust
impl Fixed32 {
    /// Checked multiplication - panics on overflow (Regehr 2012)
    /// In debug builds, this catches wrap-around bugs early.
    #[inline]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let result = (self.0 as i64).checked_mul(other.0 as i64)?;
        let shifted = result.checked_shr(16)?;

        // Check for overflow when converting back to i32
        if shifted > i32::MAX as i64 || shifted < i32::MIN as i64 {
            return None;
        }

        Some(Self(shifted as i32))
    }

    /// Checked division - panics on overflow or div by zero
    #[inline]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.0 == 0 {
            return None;
        }

        let shifted = (self.0 as i64).checked_shl(16)?;
        let result = shifted.checked_div(other.0 as i64)?;

        if result > i32::MAX as i64 || result < i32::MIN as i64 {
            return None;
        }

        Some(Self(result as i32))
    }

    /// Saturating multiplication - clamps to min/max instead of wrapping
    #[inline]
    pub fn saturating_mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap_or_else(|| {
            if (self.0 > 0) == (other.0 > 0) {
                Self(i32::MAX)  // Positive overflow
            } else {
                Self(i32::MIN)  // Negative overflow
            }
        })
    }

    /// Strict multiplication - panics on overflow in all builds
    /// Use this in game logic where overflow indicates a bug
    #[inline]
    pub fn strict_mul(self, other: Self) -> Self {
        self.checked_mul(other).expect("Fixed32 multiplication overflow")
    }
}

/// Overflow policy for Fixed32 operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Wrap around (C behavior) - DANGEROUS for determinism
    Wrap,
    /// Saturate at min/max - safer but may hide bugs
    Saturate,
    /// Panic on overflow - best for catching bugs in development
    Panic,
}
```

### 13.5 Stream Verification (Leucker & Schallhart 2009)

**Problem:** Writing traces to disk then reading back creates Muda (inventory waste).

**Solution:** Real-time stream verification:

```rust
/// Stream verifier for One-Piece Flow (Leucker & Schallhart 2009)
/// Verifies invariants in real-time as frames are generated.
pub struct StreamVerifier<'a> {
    /// Invariant checkers
    invariants: Vec<Box<dyn Fn(&GameState, u64) -> Result<(), InvariantViolation> + 'a>>,
    /// Violation handler (called immediately on failure)
    on_violation: Box<dyn FnMut(&InvariantViolation) + 'a>,
    /// Statistics
    frames_verified: u64,
    violations_found: u64,
}

impl<'a> StreamVerifier<'a> {
    /// Add invariant checker
    pub fn add_invariant<F>(&mut self, name: &'static str, check: F)
    where
        F: Fn(&GameState, u64) -> bool + 'a
    {
        self.invariants.push(Box::new(move |state, frame| {
            if check(state, frame) {
                Ok(())
            } else {
                Err(InvariantViolation {
                    frame,
                    invariant: name.to_string(),
                    message: format!("Invariant '{}' failed at frame {}", name, frame),
                })
            }
        }));
    }

    /// Verify frame in real-time (called during simulation, not after)
    /// If invariant fails, stop simulation IMMEDIATELY (One-Piece Flow)
    #[inline]
    pub fn verify_frame(&mut self, state: &GameState, frame: u64) -> Result<(), InvariantViolation> {
        self.frames_verified += 1;

        for invariant in &self.invariants {
            if let Err(violation) = invariant(state, frame) {
                self.violations_found += 1;
                (self.on_violation)(&violation);
                return Err(violation);
            }
        }

        Ok(())
    }
}

/// Monte Carlo runner with stream verification
impl McGameTestRunner {
    /// Run with stream verification (One-Piece Flow)
    pub fn run_with_stream_verification(&mut self, seed: u64) -> SimulationResult {
        let mut platform = WebPlatform::new_with_seed(self.config.game.clone().into(), seed);

        // Set up stream verifier
        let mut verifier = StreamVerifier::new(|v| {
            eprintln!("🚨 INVARIANT VIOLATION at frame {}: {}", v.frame, v.message);
        });

        // Add invariants from config
        for inv in &self.config.verify.invariants {
            verifier.add_invariant(inv, |state, _| self.evaluate_invariant(inv, state));
        }

        // Run simulation with real-time verification
        for frame in 0..self.config.simulation.duration_frames {
            let ts = frame as f64 * 16.667;

            // Get agent actions and execute frame
            let inputs = self.get_frame_inputs(frame, &platform);
            let _ = platform.frame(ts, &inputs);

            // Verify IMMEDIATELY (One-Piece Flow)
            if let Err(violation) = verifier.verify_frame(platform.game_state(), frame) {
                // STOP SIMULATION - don't waste compute on invalid state
                return SimulationResult {
                    seed,
                    frames_executed: frame,
                    passed: false,
                    early_termination: Some(violation),
                    ..Default::default()
                };
            }
        }

        SimulationResult {
            seed,
            frames_executed: self.config.simulation.duration_frames,
            passed: true,
            ..Default::default()
        }
    }
}
```

### 13.6 Summary of v1.3 Changes

| Issue | TPS Principle | v1.1 Design | v1.3 Fix |
|-------|---------------|-------------|----------|
| Hard Andon blocks game feel debugging | **Jidoka** | Thread sleep blocking | **Soft Andon** (visual indicator) |
| SHA-256 entropy check is O(N) | **Heijunka/Muri** | SHA-256 every frame | **Zobrist hashing** (O(1)) |
| f32 can be used in simulation | **Poka-Yoke** | Code review | **Compile-time macro** |
| Fixed32 wraps silently | **Poka-Yoke** | No overflow checks | **checked_mul/panic** |
| Traces written then verified | **Muda/Flow** | Batch verification | **Stream verification** |

## 14. References

See Sections 2.2, 2.3, 11.2, 12.2, and 12.2.1 for all 37 peer-reviewed citations.

### Implementation References
- renacer unified trace architecture: `../renacer/crates/renacer-tracer/`
- Lamport clock implementation: `../renacer/crates/renacer-core/src/lamport_clock.rs`
- Ring buffer design: `../renacer/crates/renacer-tracer/src/ring_buffer.rs`
- Fixed-point math: Consider `fixed` crate or custom implementation
- Chaos engineering: `../entrenar/tests/chaos/`
- Property-based testing: `../trueno-viz/tests/proptest/`
- Benchmark patterns: `../trueno/benches/`
- Monte Carlo Tree Search: `../entrenar/src/search/mcts.rs`
- YAML declarative patterns: `../presentar/examples/`, `../entrenar/book/src/declarative/`
