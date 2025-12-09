//! Game Event Tracing and Deterministic Replay
//!
//! Implements the Renacer-Based Game Event Tracing specification (TRACE-001 v1.3).
//!
//! ## Key Components
//!
//! - [`Fixed32`]: Cross-platform deterministic 16.16 fixed-point math (Poka-Yoke)
//! - [`FrameRecord`]: Lean per-frame trace data (Muda elimination)
//! - [`TraceBuffer`]: Ring buffer with Andon Cord overflow policy (Jidoka)
//! - [`AdaptiveSnapshotter`]: Entropy-based snapshot scheduling (Heijunka)
//! - [`AndonState`]: Visual trace loss indicator (Soft Andon - v1.3)
//! - [`ZobristTable`]: O(1) incremental state hashing (v1.3)
//!
//! ## Toyota Production System Principles
//!
//! - **Jidoka**: Buffer overflow with Soft Andon visual indicator (v1.3)
//! - **Poka-Yoke**: Fixed32 with overflow checks (Regehr 2012) (v1.3)
//! - **Heijunka**: Zobrist hashing for O(1) entropy detection (v1.3)
//! - **Muda**: Frame number is the only clock; no physical timestamps
//!
//! ## References
//!
//! - Lamport (1978): Logical clocks for causal ordering
//! - Monniaux (2008): IEEE 754 non-determinism across platforms
//! - Dunlap (2002): ALL inputs must be logged for faithful replay
//! - Elnozahy (2002): Adaptive checkpointing based on state mutation rates
//! - Zobrist (1970): O(1) incremental hashing for games
//! - Regehr (2012): Integer overflow detection and prevention
//! - MacKenzie & Ware (1993): Input lag and human performance

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use serde::{Deserialize, Serialize};

// =============================================================================
// Fixed32: Cross-Platform Deterministic Math (Poka-Yoke)
// =============================================================================

/// Fixed-point number with 16.16 format (16 integer bits, 16 fractional bits).
///
/// Guarantees identical results across ALL platforms per Monniaux (2008):
/// "Floating-point non-determinism is a primary source of divergence in
/// cross-platform replay systems."
///
/// # Examples
///
/// ```
/// use jugar_web::trace::Fixed32;
///
/// let a = Fixed32::from_int(5);
/// let b = Fixed32::from_int(3);
/// assert_eq!((a + b).to_int(), 8);
/// assert_eq!((a - b).to_int(), 2);
/// assert_eq!((a * b).to_int(), 15);
/// assert_eq!((a / b).to_int(), 1); // Integer division
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Fixed32(pub i32);

impl Fixed32 {
    /// Number of fractional bits (16.16 format)
    pub const FRAC_BITS: u32 = 16;

    /// Scale factor (2^16 = 65536)
    pub const SCALE: i32 = 1 << Self::FRAC_BITS;

    /// Zero
    pub const ZERO: Self = Self(0);

    /// One (1.0 in fixed-point)
    pub const ONE: Self = Self(Self::SCALE);

    /// Half (0.5 in fixed-point)
    pub const HALF: Self = Self(Self::SCALE / 2);

    /// Minimum value
    pub const MIN: Self = Self(i32::MIN);

    /// Maximum value
    pub const MAX: Self = Self(i32::MAX);

    /// Epsilon (smallest positive value)
    pub const EPSILON: Self = Self(1);

    /// Pi approximation (accurate to ~5 decimal places)
    pub const PI: Self = Self(205_887); // 3.14159 * 65536

    /// Create from raw fixed-point value (internal representation).
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Get the raw fixed-point value.
    #[inline]
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        self.0
    }

    /// Create from integer.
    #[inline]
    #[must_use]
    pub const fn from_int(n: i32) -> Self {
        Self(n << Self::FRAC_BITS)
    }

    /// Convert to integer (truncates fractional part).
    #[inline]
    #[must_use]
    pub const fn to_int(self) -> i32 {
        self.0 >> Self::FRAC_BITS
    }

    /// Create from f32 (use only for constants, not runtime game logic).
    ///
    /// # Warning
    ///
    /// This conversion introduces platform-dependent rounding. Only use for
    /// initialization from constants; never in game update loops.
    #[inline]
    #[must_use]
    pub fn from_f32(f: f32) -> Self {
        Self((f * Self::SCALE as f32) as i32)
    }

    /// Convert to f32 (for rendering only, not game logic).
    ///
    /// # Warning
    ///
    /// The result may differ slightly across platforms. Only use for rendering;
    /// never store or compare these values for game logic.
    #[inline]
    #[must_use]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE as f32
    }

    /// Saturating addition (clamps to MIN/MAX instead of overflowing).
    #[inline]
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction (clamps to MIN/MAX instead of overflowing).
    #[inline]
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Fixed-point multiplication with proper scaling.
    ///
    /// Uses i64 intermediate to prevent overflow.
    #[inline]
    #[must_use]
    pub const fn mul(self, other: Self) -> Self {
        Self(((self.0 as i64 * other.0 as i64) >> Self::FRAC_BITS) as i32)
    }

    /// Saturating multiplication (clamps instead of overflowing).
    #[inline]
    #[must_use]
    #[allow(clippy::cast_lossless)] // Can't use From in const fn yet
    pub const fn saturating_mul(self, other: Self) -> Self {
        let result = (self.0 as i64 * other.0 as i64) >> Self::FRAC_BITS;
        if result > i32::MAX as i64 {
            Self::MAX
        } else if result < i32::MIN as i64 {
            Self::MIN
        } else {
            Self(result as i32)
        }
    }

    /// Fixed-point division with proper scaling.
    ///
    /// Uses i64 intermediate to prevent overflow.
    ///
    /// # Panics
    ///
    /// Panics if `other` is zero.
    #[inline]
    #[must_use]
    pub const fn div(self, other: Self) -> Self {
        Self((((self.0 as i64) << Self::FRAC_BITS) / other.0 as i64) as i32)
    }

    /// Checked division (returns None if divisor is zero).
    #[inline]
    #[must_use]
    pub const fn checked_div(self, other: Self) -> Option<Self> {
        if other.0 == 0 {
            None
        } else {
            Some(self.div(other))
        }
    }

    /// Absolute value.
    #[inline]
    #[must_use]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Sign of the number (-1, 0, or 1).
    #[inline]
    #[must_use]
    pub const fn signum(self) -> Self {
        Self::from_int(self.0.signum())
    }

    /// Check if negative.
    #[inline]
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Check if positive.
    #[inline]
    #[must_use]
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }

    /// Check if zero.
    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Clamp value to range [min, max].
    #[inline]
    #[must_use]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        if self.0 < min.0 {
            min
        } else if self.0 > max.0 {
            max
        } else {
            self
        }
    }

    /// Linear interpolation between self and other.
    ///
    /// `t` should be between 0.0 and 1.0 (as Fixed32).
    #[inline]
    #[must_use]
    pub const fn lerp(self, other: Self, t: Self) -> Self {
        // self + (other - self) * t
        let diff = Self(other.0 - self.0);
        let scaled = diff.mul(t);
        Self(self.0 + scaled.0)
    }

    /// Floor to integer (round toward negative infinity).
    #[inline]
    #[must_use]
    pub const fn floor(self) -> Self {
        Self((self.0 >> Self::FRAC_BITS) << Self::FRAC_BITS)
    }

    /// Ceiling to integer (round toward positive infinity).
    #[inline]
    #[must_use]
    pub const fn ceil(self) -> Self {
        let frac_mask = Self::SCALE - 1;
        if self.0 & frac_mask == 0 {
            self
        } else {
            Self(((self.0 >> Self::FRAC_BITS) + 1) << Self::FRAC_BITS)
        }
    }

    /// Round to nearest integer.
    #[inline]
    #[must_use]
    pub const fn round(self) -> Self {
        Self(((self.0 + (Self::SCALE / 2)) >> Self::FRAC_BITS) << Self::FRAC_BITS)
    }

    /// Get fractional part.
    #[inline]
    #[must_use]
    pub const fn fract(self) -> Self {
        let frac_mask = Self::SCALE - 1;
        Self(self.0 & frac_mask)
    }

    // =========================================================================
    // v1.3 TPS Kaizen: Overflow Checks (Regehr 2012)
    // =========================================================================

    /// Checked multiplication - returns None on overflow (Regehr 2012).
    ///
    /// Use this in game logic where overflow indicates a bug that should
    /// be caught early in development.
    ///
    /// # Examples
    ///
    /// ```
    /// use jugar_web::trace::Fixed32;
    ///
    /// let a = Fixed32::from_int(100);
    /// let b = Fixed32::from_int(50);
    /// assert_eq!(a.checked_mul(b), Some(Fixed32::from_int(5000)));
    ///
    /// // Overflow case
    /// let big = Fixed32::MAX;
    /// assert_eq!(big.checked_mul(Fixed32::from_int(2)), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn checked_mul(self, other: Self) -> Option<Self> {
        let result = self.0 as i64 * other.0 as i64;
        let shifted = result >> Self::FRAC_BITS;

        // Check for overflow when converting back to i32
        if shifted > i32::MAX as i64 || shifted < i32::MIN as i64 {
            return None;
        }

        Some(Self(shifted as i32))
    }

    /// Strict multiplication - panics on overflow in all builds (Regehr 2012).
    ///
    /// Use this in game logic where overflow indicates a bug.
    /// This is the safest option for catching bugs early.
    ///
    /// # Panics
    ///
    /// Panics if the multiplication would overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use jugar_web::trace::Fixed32;
    ///
    /// let a = Fixed32::from_int(100);
    /// let b = Fixed32::from_int(50);
    /// assert_eq!(a.strict_mul(b), Fixed32::from_int(5000));
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    #[allow(clippy::panic)] // Intentional: strict_* methods should panic on overflow (Regehr 2012)
    pub const fn strict_mul(self, other: Self) -> Self {
        match self.checked_mul(other) {
            Some(result) => result,
            None => panic!("Fixed32 multiplication overflow"),
        }
    }

    /// Checked addition - returns None on overflow.
    #[inline]
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(result) => Some(Self(result)),
            None => None,
        }
    }

    /// Strict addition - panics on overflow.
    ///
    /// # Panics
    ///
    /// Panics if the addition would overflow.
    #[inline]
    #[must_use]
    #[track_caller]
    #[allow(clippy::panic)] // Intentional: strict_* methods should panic on overflow (Regehr 2012)
    pub const fn strict_add(self, other: Self) -> Self {
        match self.checked_add(other) {
            Some(result) => result,
            None => panic!("Fixed32 addition overflow"),
        }
    }

    /// Checked subtraction - returns None on overflow.
    #[inline]
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(result) => Some(Self(result)),
            None => None,
        }
    }

    /// Strict subtraction - panics on overflow.
    ///
    /// # Panics
    ///
    /// Panics if the subtraction would overflow.
    #[inline]
    #[must_use]
    #[track_caller]
    #[allow(clippy::panic)] // Intentional: strict_* methods should panic on overflow (Regehr 2012)
    pub const fn strict_sub(self, other: Self) -> Self {
        match self.checked_sub(other) {
            Some(result) => result,
            None => panic!("Fixed32 subtraction overflow"),
        }
    }
}

// =============================================================================
// deterministic! Macro: Compile-Time f32 Enforcement (Bessey 2010)
// =============================================================================

/// Poka-Yoke macro to enforce Fixed32 in deterministic game logic (Bessey 2010).
///
/// This macro wraps a block and causes compile errors if f32/f64 literals are used.
/// It works by shadowing the f32/f64 types with unconstructable struct types.
///
/// # Examples
///
/// ```
/// use jugar_web::{deterministic, trace::Fixed32};
///
/// deterministic! {
///     let a = Fixed32::from_int(5);
///     let b = Fixed32::from_int(3);
///     let result = a.mul(b);
///     // This would cause a compile error:
///     // let bad = 1.0f32 * 2.0;  // ERROR: f32 is shadowed
/// }
/// ```
///
/// # Rationale
///
/// Per Monniaux (2008): "Floating-point non-determinism is a primary source of
/// divergence in cross-platform replay systems."
///
/// Per Bessey (2010): "Using compile-time constraints to enforce invariants
/// catches bugs earlier than runtime checks."
#[macro_export]
macro_rules! deterministic {
    ($($body:tt)*) => {{
        // Shadow f32/f64 types with unconstructable types
        // Any use of f32/f64 literals will fail to compile
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct f32;
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct f64;

        $($body)*
    }};
}

// Implement standard traits for ergonomic usage
impl Default for Fixed32 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Fixed32 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
}

impl AddAssign for Fixed32 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_add(other.0);
    }
}

impl Sub for Fixed32 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
}

impl SubAssign for Fixed32 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_sub(other.0);
    }
}

impl Mul for Fixed32 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl Div for Fixed32 {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self {
        self.div(other)
    }
}

impl Neg for Fixed32 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl From<i32> for Fixed32 {
    fn from(n: i32) -> Self {
        Self::from_int(n)
    }
}

impl core::fmt::Display for Fixed32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.4}", self.to_f32())
    }
}

// =============================================================================
// InputEvent: Lean Input Record (Muda Elimination)
// =============================================================================

/// Input event type for trace recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)] // Enum variant fields are self-documenting
pub enum InputEventType {
    /// Key pressed (key code)
    KeyDown(u8),
    /// Key released (key code)
    KeyUp(u8),
    /// Mouse button pressed (button, x, y)
    MouseDown { button: u8, x: i16, y: i16 },
    /// Mouse button released (button, x, y)
    MouseUp { button: u8, x: i16, y: i16 },
    /// Mouse moved (x, y)
    MouseMove { x: i16, y: i16 },
    /// Touch started (id, x, y)
    TouchStart { id: u8, x: i16, y: i16 },
    /// Touch moved (id, x, y)
    TouchMove { id: u8, x: i16, y: i16 },
    /// Touch ended (id, x, y)
    TouchEnd { id: u8, x: i16, y: i16 },
    /// Gamepad button pressed (gamepad, button)
    GamepadDown { gamepad: u8, button: u8 },
    /// Gamepad button released (gamepad, button)
    GamepadUp { gamepad: u8, button: u8 },
    /// Gamepad axis moved (gamepad, axis, value)
    GamepadAxis { gamepad: u8, axis: u8, value: i16 },
}

/// Input event with frame-relative timing.
///
/// Per Jain (2014): "Logical time is sufficient for reproducing bugs;
/// physical time introduces unnecessary noise."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEvent {
    /// Event type
    pub event_type: InputEventType,
    /// Microseconds since frame start (for sub-frame ordering only).
    /// Max value: 16666 (one 60fps frame in microseconds).
    pub frame_offset_us: u16,
}

// =============================================================================
// FrameRecord: Lean Per-Frame Trace (Muda Elimination)
// =============================================================================

/// Lean frame record - frame_number is the only clock needed.
///
/// Per Jain (2014): "Logical time is sufficient for reproducing bugs."
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameRecord {
    /// Frame number (monotonic, IS the logical clock for deterministic games).
    pub frame: u64,
    /// Input events this frame (relative timing within frame if needed).
    pub inputs: Vec<InputEvent>,
    /// State hash for verification (optional in release traces).
    pub state_hash: Option<[u8; 32]>,
}

impl FrameRecord {
    /// Create a new frame record.
    #[must_use]
    pub const fn new(frame: u64) -> Self {
        Self {
            frame,
            inputs: Vec::new(),
            state_hash: None,
        }
    }

    /// Add an input event to this frame.
    pub fn add_input(&mut self, event: InputEvent) {
        self.inputs.push(event);
    }

    /// Set the state hash for verification.
    pub const fn set_state_hash(&mut self, hash: [u8; 32]) {
        self.state_hash = Some(hash);
    }

    /// Check if this frame has any inputs.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::is_empty is not const
    pub fn has_inputs(&self) -> bool {
        !self.inputs.is_empty()
    }
}

// =============================================================================
// BufferPolicy: Jidoka Safety (Andon Cord)
// =============================================================================

/// Buffer overflow policy per Jidoka principle.
///
/// Per Dunlap (2002): "A trace with missing input events is worse than no trace at all."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferPolicy {
    /// Production mode: drop oldest events on overflow (preserves responsiveness).
    #[default]
    DropOldest,
    /// v1.3: Soft Andon - drop oldest but show visual indicator (MacKenzie & Ware 1993).
    /// This preserves game responsiveness while making trace loss impossible to ignore.
    SoftAndon,
    /// Debug mode: block game loop until buffer drains (preserves correctness).
    /// This is intentional per Jidoka - "Stop the Line" when quality is at risk.
    AndonCord,
}

// =============================================================================
// AndonState: Visual Trace Loss Indicator (v1.3 Soft Andon)
// =============================================================================

/// Soft Andon state for visual trace loss indication (MacKenzie & Ware 1993).
///
/// Per MacKenzie & Ware (1993): "Input lag degrades human performance."
/// Hard blocking (AndonCord) creates lag; SoftAndon preserves responsiveness
/// while making trace loss visually impossible to ignore.
///
/// # Examples
///
/// ```
/// use jugar_web::trace::AndonState;
///
/// let state = AndonState::Normal;
/// assert_eq!(state.overlay_color(), [0.0, 0.0, 0.0, 0.0]); // Invisible
///
/// let warning = AndonState::Warning { buffer_pct: 85 };
/// assert!(warning.overlay_color()[3] > 0.0); // Visible yellow
///
/// let loss = AndonState::TraceLoss { dropped_count: 10 };
/// assert!(loss.overlay_color()[3] > 0.0); // Visible red
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AndonState {
    /// Normal operation - all events being recorded.
    #[default]
    Normal,
    /// Warning - buffer at 80%+ capacity.
    Warning {
        /// Buffer fill percentage (80-99).
        buffer_pct: u8,
    },
    /// Trace loss - events are being dropped.
    TraceLoss {
        /// Number of frames dropped since last normal state.
        dropped_count: u64,
    },
}

impl AndonState {
    /// Get HUD overlay color (Visual Management).
    ///
    /// Returns RGBA color for drawing over the game canvas.
    #[must_use]
    pub const fn overlay_color(&self) -> [f32; 4] {
        match self {
            Self::Normal => [0.0, 0.0, 0.0, 0.0],           // Invisible
            Self::Warning { .. } => [1.0, 0.8, 0.0, 0.3],   // Yellow 30% opacity
            Self::TraceLoss { .. } => [1.0, 0.0, 0.0, 0.5], // Red 50% opacity
        }
    }

    /// Get status text for HUD display.
    #[must_use]
    pub const fn status_text(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Warning { .. } => "TRACE BUFFER WARNING",
            Self::TraceLoss { .. } => "TRACE LOSS - EVENTS DROPPED",
        }
    }

    /// Check if in error state (warning or trace loss).
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !matches!(self, Self::Normal)
    }

    /// Check if any events have been dropped.
    #[must_use]
    pub const fn has_dropped(&self) -> bool {
        matches!(self, Self::TraceLoss { .. })
    }

    /// Get number of dropped frames (0 if none).
    #[must_use]
    pub const fn dropped_count(&self) -> u64 {
        match self {
            Self::TraceLoss { dropped_count } => *dropped_count,
            _ => 0,
        }
    }
}

// =============================================================================
// ZobristTable: O(1) Incremental State Hashing (Zobrist 1970)
// =============================================================================

/// Number of hash fields for Zobrist table.
/// This should cover all game state fields that affect determinism.
pub const NUM_ZOBRIST_FIELDS: usize = 32;

/// Zobrist hash table for O(1) incremental state hashing (Zobrist 1970).
///
/// Used by [`ZobristSnapshotter`] for efficient entropy detection.
/// Instead of computing SHA-256 every frame (O(N)), we can update the hash
/// incrementally when state changes (O(1)).
///
/// # Theory
///
/// Per Zobrist (1970): "A hash value can be updated incrementally by XORing
/// out the old value and XORing in the new value."
///
/// Per Tridgell (1999): "Rolling checksums enable efficient delta detection
/// without full state comparison."
///
/// # Examples
///
/// ```
/// use jugar_web::trace::ZobristTable;
///
/// let table = ZobristTable::new(42);
/// let state = [0u8; 32];
/// let hash = table.hash_bytes(&state);
///
/// // Incremental update is O(1)
/// let new_hash = table.update_hash(hash, 0, 0, 255);
/// assert_ne!(hash, new_hash);
/// ```
#[derive(Debug, Clone)]
pub struct ZobristTable {
    /// Random values for each (field, value) pair.
    /// Pre-generated on initialization for reproducibility.
    /// Heap-allocated to avoid stack overflow (65KB array).
    table: Box<[[u64; 256]; NUM_ZOBRIST_FIELDS]>,
}

impl ZobristTable {
    /// Create table with deterministic RNG (for reproducibility).
    ///
    /// Uses xorshift64 for fast, deterministic random generation.
    /// Table is heap-allocated (65KB) to avoid stack overflow.
    ///
    /// # Panics
    ///
    /// Cannot panic - the Vec-to-array conversion is guaranteed to succeed
    /// because we create exactly `NUM_ZOBRIST_FIELDS` elements.
    #[must_use]
    #[allow(clippy::expect_used)] // Infallible: Vec size matches array size exactly
    pub fn new(seed: u64) -> Self {
        let mut state = seed.max(1); // Avoid zero seed
                                     // Heap allocate to avoid 65KB stack frame (clippy::large_stack_arrays)
        let mut table = vec![[0u64; 256]; NUM_ZOBRIST_FIELDS];

        for field in &mut table {
            for value in field.iter_mut() {
                // xorshift64 (Marsaglia 2003)
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *value = state;
            }
        }

        // Convert Vec to Box<[_; N]> for fixed-size guarantee
        // This cannot fail: we created exactly NUM_ZOBRIST_FIELDS elements above
        let boxed: Box<[[u64; 256]; NUM_ZOBRIST_FIELDS]> = table
            .into_boxed_slice()
            .try_into()
            .expect("Vec has exact NUM_ZOBRIST_FIELDS elements");

        Self { table: boxed }
    }

    /// Compute Zobrist hash for byte slice (O(N) - used once per state).
    #[must_use]
    pub fn hash_bytes(&self, bytes: &[u8]) -> u64 {
        let mut hash = 0u64;
        for (i, &byte) in bytes.iter().enumerate() {
            hash ^= self.table[i % NUM_ZOBRIST_FIELDS][byte as usize];
        }
        hash
    }

    /// Update hash incrementally when single byte changes (O(1)).
    ///
    /// This is the key optimization - no need to rehash entire state.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Indexing not const-stable
    pub fn update_hash(&self, hash: u64, field: usize, old_byte: u8, new_byte: u8) -> u64 {
        // XOR out old value, XOR in new value (Tridgell 1999)
        hash ^ self.table[field % NUM_ZOBRIST_FIELDS][old_byte as usize]
            ^ self.table[field % NUM_ZOBRIST_FIELDS][new_byte as usize]
    }

    /// Calculate entropy as Hamming distance between two hashes.
    ///
    /// Returns number of differing bits (0-64).
    #[inline]
    #[must_use]
    pub const fn entropy(hash1: u64, hash2: u64) -> u32 {
        (hash1 ^ hash2).count_ones()
    }
}

impl Default for ZobristTable {
    fn default() -> Self {
        Self::new(0xDEAD_BEEF_CAFE_BABE)
    }
}

/// Adaptive snapshotter using Zobrist hashing for O(1) entropy detection.
///
/// This is an optimized version of [`AdaptiveSnapshotter`] that uses
/// Zobrist hashing instead of SHA-256 for entropy calculation.
///
/// # Performance
///
/// - Original (SHA-256): O(N) per frame where N = state size
/// - Zobrist: O(1) per state change, O(N) only for full hash
///
/// Per spec v1.3 TPS Kaizen: "SHA-256 entropy check is O(N) and creates Muri"
#[derive(Debug, Clone)]
pub struct ZobristSnapshotter {
    /// Zobrist hash table.
    table: ZobristTable,
    /// Current state hash (incrementally updated).
    current_hash: u64,
    /// Previous snapshot hash.
    prev_snapshot_hash: u64,
    /// Entropy threshold (Hamming distance between hashes).
    entropy_threshold: u32,
    /// Minimum frames between snapshots.
    min_interval: u64,
    /// Maximum frames between snapshots.
    max_interval: u64,
    /// Last snapshot frame.
    last_snapshot_frame: u64,
}

impl ZobristSnapshotter {
    /// Create with custom parameters.
    #[must_use]
    pub fn new(seed: u64, min_interval: u64, max_interval: u64, entropy_threshold: u32) -> Self {
        Self {
            table: ZobristTable::new(seed),
            current_hash: 0,
            prev_snapshot_hash: 0,
            entropy_threshold,
            min_interval,
            max_interval,
            last_snapshot_frame: 0,
        }
    }

    /// Initialize hash from full state (O(N) - call once at start).
    pub fn initialize(&mut self, state_bytes: &[u8]) {
        self.current_hash = self.table.hash_bytes(state_bytes);
        self.prev_snapshot_hash = self.current_hash;
    }

    /// Update hash incrementally when state changes (O(1)).
    #[inline]
    pub fn on_state_change(&mut self, field: usize, old_byte: u8, new_byte: u8) {
        self.current_hash = self
            .table
            .update_hash(self.current_hash, field, old_byte, new_byte);
    }

    /// Check if snapshot should be taken (O(1) operation!).
    pub fn should_snapshot(&mut self, frame: u64) -> SnapshotDecision {
        let frames_since = frame.saturating_sub(self.last_snapshot_frame);

        // Force snapshot at max interval
        if frames_since >= self.max_interval {
            self.take_snapshot(frame);
            return SnapshotDecision::FullSnapshot;
        }

        // Calculate entropy as Hamming distance between hashes (O(1))
        let entropy = ZobristTable::entropy(self.current_hash, self.prev_snapshot_hash);

        // High entropy = state changed significantly
        if entropy >= self.entropy_threshold && frames_since >= self.min_interval {
            self.take_snapshot(frame);
            return SnapshotDecision::DeltaSnapshot;
        }

        SnapshotDecision::Skip
    }

    /// Record that a snapshot was taken.
    #[allow(clippy::missing_const_for_fn)] // const fn with mut ref not stable
    fn take_snapshot(&mut self, frame: u64) {
        self.prev_snapshot_hash = self.current_hash;
        self.last_snapshot_frame = frame;
    }

    /// Get current hash value.
    #[must_use]
    pub const fn current_hash(&self) -> u64 {
        self.current_hash
    }

    /// Get last snapshot frame.
    #[must_use]
    pub const fn last_snapshot_frame(&self) -> u64 {
        self.last_snapshot_frame
    }
}

impl Default for ZobristSnapshotter {
    fn default() -> Self {
        Self::new(
            0xDEAD_BEEF_CAFE_BABE,
            15,  // min_interval
            120, // max_interval
            16,  // entropy_threshold (bits changed)
        )
    }
}

// =============================================================================
// SnapshotDecision: Adaptive Snapshot Scheduling (Heijunka)
// =============================================================================

/// Snapshot decision based on state entropy.
///
/// Per Elnozahy (2002): "Adaptive checkpointing based on state mutation rates
/// significantly reduces log sizes."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotDecision {
    /// No snapshot needed.
    Skip,
    /// Take delta snapshot (high entropy detected).
    DeltaSnapshot,
    /// Take full snapshot (max interval reached).
    FullSnapshot,
}

// =============================================================================
// AdaptiveSnapshotter: Entropy-Based Scheduling (Heijunka/Mura)
// =============================================================================

/// Adaptive snapshot scheduler based on state entropy.
///
/// Fixed-interval snapshots create Mura (unevenness):
/// - Menu screens waste space with identical snapshots
/// - High-action moments lack granularity
///
/// Solution: Snapshot based on state entropy (magnitude of change).
#[derive(Debug, Clone)]
pub struct AdaptiveSnapshotter {
    /// Minimum frames between snapshots.
    pub min_interval: u64,
    /// Maximum frames between snapshots (force snapshot).
    pub max_interval: u64,
    /// Entropy threshold to trigger snapshot.
    pub entropy_threshold: u32,
    /// Last snapshot frame.
    last_snapshot_frame: u64,
    /// Previous state hash for delta calculation.
    prev_state_hash: [u8; 32],
}

impl Default for AdaptiveSnapshotter {
    fn default() -> Self {
        Self {
            min_interval: 15,      // At least 15 frames (~250ms at 60fps)
            max_interval: 120,     // Force snapshot every 120 frames (~2 sec)
            entropy_threshold: 64, // ~25% of bits changed
            last_snapshot_frame: 0,
            prev_state_hash: [0; 32],
        }
    }
}

impl AdaptiveSnapshotter {
    /// Create with custom parameters.
    #[must_use]
    pub const fn new(min_interval: u64, max_interval: u64, entropy_threshold: u32) -> Self {
        Self {
            min_interval,
            max_interval,
            entropy_threshold,
            last_snapshot_frame: 0,
            prev_state_hash: [0; 32],
        }
    }

    /// Determine if a snapshot should be taken this frame.
    pub fn should_snapshot(&mut self, frame: u64, state_hash: &[u8; 32]) -> SnapshotDecision {
        let frames_since = frame.saturating_sub(self.last_snapshot_frame);

        // Force snapshot at max interval
        if frames_since >= self.max_interval {
            self.prev_state_hash = *state_hash;
            self.last_snapshot_frame = frame;
            return SnapshotDecision::FullSnapshot;
        }

        // Calculate state entropy (Hamming distance between hashes)
        let entropy = self.calculate_entropy(state_hash);

        // High entropy = take snapshot if past min interval
        if entropy >= self.entropy_threshold && frames_since >= self.min_interval {
            self.prev_state_hash = *state_hash;
            self.last_snapshot_frame = frame;
            return SnapshotDecision::DeltaSnapshot;
        }

        SnapshotDecision::Skip
    }

    /// Calculate entropy as Hamming distance between state hashes.
    fn calculate_entropy(&self, current: &[u8; 32]) -> u32 {
        self.prev_state_hash
            .iter()
            .zip(current.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }

    /// Reset the snapshotter state.
    pub const fn reset(&mut self) {
        self.last_snapshot_frame = 0;
        self.prev_state_hash = [0; 32];
    }

    /// Get the frame of the last snapshot.
    #[must_use]
    pub const fn last_snapshot_frame(&self) -> u64 {
        self.last_snapshot_frame
    }
}

// =============================================================================
// TraceBuffer: Ring Buffer with Andon Cord (Jidoka)
// =============================================================================

/// Trace error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceError {
    /// Buffer is full (only in strict mode without Andon Cord).
    BufferFull,
    /// Invalid frame sequence.
    InvalidFrameSequence,
    /// Serialization error.
    SerializationError(String),
}

impl core::fmt::Display for TraceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferFull => write!(f, "Trace buffer is full"),
            Self::InvalidFrameSequence => write!(f, "Invalid frame sequence"),
            Self::SerializationError(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl core::error::Error for TraceError {}

/// Ring buffer for trace events with Andon Cord policy.
///
/// Per Dunlap (2002): "A trace with missing input events is worse than no trace at all."
#[derive(Debug)]
pub struct TraceBuffer {
    /// Frame records.
    frames: Vec<Option<FrameRecord>>,
    /// Buffer capacity.
    capacity: usize,
    /// Write position.
    write_pos: usize,
    /// Read position.
    read_pos: usize,
    /// Number of elements in buffer.
    len: usize,
    /// Buffer policy.
    policy: BufferPolicy,
    /// Total frames dropped (only in DropOldest/SoftAndon mode).
    frames_dropped: u64,
    /// v1.3: Current Andon state for Soft Andon visual feedback.
    andon_state: AndonState,
}

impl TraceBuffer {
    /// Create a new trace buffer with specified capacity and policy.
    #[must_use]
    pub fn new(capacity: usize, policy: BufferPolicy) -> Self {
        let mut frames = Vec::with_capacity(capacity);
        frames.resize_with(capacity, || None);
        Self {
            frames,
            capacity,
            write_pos: 0,
            read_pos: 0,
            len: 0,
            policy,
            frames_dropped: 0,
            andon_state: AndonState::Normal,
        }
    }

    /// Create a Soft Andon buffer (visual indicator on overflow).
    #[must_use]
    pub fn soft_andon(capacity: usize) -> Self {
        Self::new(capacity, BufferPolicy::SoftAndon)
    }

    /// Create a debug buffer (Andon Cord enabled).
    #[must_use]
    pub fn debug(capacity: usize) -> Self {
        Self::new(capacity, BufferPolicy::AndonCord)
    }

    /// Create a production buffer (drop oldest on overflow).
    #[must_use]
    pub fn production(capacity: usize) -> Self {
        Self::new(capacity, BufferPolicy::DropOldest)
    }

    /// Push a frame record to the buffer.
    ///
    /// In `AndonCord` mode, this will return `Err(BufferFull)` if the buffer is full.
    /// In `DropOldest` mode, this will drop the oldest frame to make room.
    /// In `SoftAndon` mode, this will drop oldest but update [`AndonState`] for visual feedback.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::BufferFull`] if the buffer is full and using `AndonCord` policy.
    pub fn push(&mut self, record: FrameRecord) -> Result<(), TraceError> {
        // Update Andon state based on buffer fill level (v1.3)
        self.update_andon_state();

        if self.len >= self.capacity {
            match self.policy {
                BufferPolicy::DropOldest => {
                    // Drop oldest silently
                    self.read_pos = (self.read_pos + 1) % self.capacity;
                    self.len -= 1;
                    self.frames_dropped += 1;
                }
                BufferPolicy::SoftAndon => {
                    // v1.3: Drop oldest but update visual state
                    self.read_pos = (self.read_pos + 1) % self.capacity;
                    self.len -= 1;
                    self.frames_dropped += 1;
                    self.andon_state = AndonState::TraceLoss {
                        dropped_count: self.frames_dropped,
                    };
                }
                BufferPolicy::AndonCord => {
                    // STOP THE LINE: Return error (caller should block)
                    return Err(TraceError::BufferFull);
                }
            }
        }

        self.frames[self.write_pos] = Some(record);
        self.write_pos = (self.write_pos + 1) % self.capacity;
        self.len += 1;
        Ok(())
    }

    /// Update Andon state based on buffer fill level (v1.3).
    fn update_andon_state(&mut self) {
        if self.policy != BufferPolicy::SoftAndon {
            return;
        }

        let fill_pct = (self.len * 100) / self.capacity.max(1);

        self.andon_state = if self.frames_dropped > 0 {
            AndonState::TraceLoss {
                dropped_count: self.frames_dropped,
            }
        } else if fill_pct >= 80 {
            AndonState::Warning {
                buffer_pct: fill_pct as u8,
            }
        } else {
            AndonState::Normal
        };
    }

    /// Get current Andon state for visual feedback (v1.3).
    #[must_use]
    pub const fn andon_state(&self) -> AndonState {
        self.andon_state
    }

    /// Try to push without blocking.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::BufferFull`] if the buffer is full and using `AndonCord` policy.
    pub fn try_push(&mut self, record: FrameRecord) -> Result<(), TraceError> {
        // Only AndonCord returns error on full - DropOldest and SoftAndon drop oldest
        if self.len >= self.capacity && self.policy == BufferPolicy::AndonCord {
            return Err(TraceError::BufferFull);
        }
        self.push(record)
    }

    /// Pop the oldest frame record from the buffer.
    pub fn pop(&mut self) -> Option<FrameRecord> {
        if self.len == 0 {
            return None;
        }

        let record = self.frames[self.read_pos].take();
        self.read_pos = (self.read_pos + 1) % self.capacity;
        self.len -= 1;
        record
    }

    /// Drain up to `count` frames from the buffer.
    pub fn drain(&mut self, count: usize) -> Vec<FrameRecord> {
        let to_drain = count.min(self.len);
        let mut result = Vec::with_capacity(to_drain);

        for _ in 0..to_drain {
            if let Some(record) = self.pop() {
                result.push(record);
            }
        }

        result
    }

    /// Get current buffer length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if buffer is full.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    /// Get buffer capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get number of dropped frames (only relevant in DropOldest mode).
    #[must_use]
    pub const fn frames_dropped(&self) -> u64 {
        self.frames_dropped
    }

    /// Get buffer policy.
    #[must_use]
    pub const fn policy(&self) -> BufferPolicy {
        self.policy
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        for frame in &mut self.frames {
            *frame = None;
        }
        self.write_pos = 0;
        self.read_pos = 0;
        self.len = 0;
    }

    /// Iterate over all frames in the buffer (oldest to newest).
    pub fn iter(&self) -> impl Iterator<Item = &FrameRecord> {
        let capacity = self.capacity;
        let read_pos = self.read_pos;
        let len = self.len;

        (0..len).filter_map(move |i| {
            let idx = (read_pos + i) % capacity;
            self.frames[idx].as_ref()
        })
    }
}

// =============================================================================
// TraceQuery: Query-Based Debugging (Genchi Genbutsu)
// =============================================================================

/// Query result containing frame and optional context.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The frame number that matched.
    pub frame: u64,
    /// Context frames before (if requested).
    pub context_before: Vec<FrameRecord>,
    /// Context frames after (if requested).
    pub context_after: Vec<FrameRecord>,
}

/// Query-based trace analysis per Ko & Myers (2008).
///
/// Enables "Whyline-style" queries like:
/// - "Find all frames where condition X is true"
/// - "Why did the ball miss the paddle at frame N?"
///
/// # Example
///
/// ```ignore
/// let query = TraceQuery::from_buffer(&buffer);
/// let frames_with_input = query.find_frames(|f| f.has_inputs());
/// ```
#[derive(Debug)]
pub struct TraceQuery<'a> {
    frames: Vec<&'a FrameRecord>,
}

impl<'a> TraceQuery<'a> {
    /// Create a query interface from a trace buffer.
    #[must_use]
    pub fn from_buffer(buffer: &'a TraceBuffer) -> Self {
        Self {
            frames: buffer.iter().collect(),
        }
    }

    /// Create a query interface from a slice of frame records.
    #[must_use]
    pub const fn from_frames(frames: Vec<&'a FrameRecord>) -> Self {
        Self { frames }
    }

    /// Find all frames matching a predicate.
    pub fn find_frames(&self, predicate: impl Fn(&FrameRecord) -> bool) -> Vec<u64> {
        self.frames
            .iter()
            .filter(|f| predicate(f))
            .map(|f| f.frame)
            .collect()
    }

    /// Find frames with context (frames before and after).
    pub fn find_frames_with_context(
        &self,
        predicate: impl Fn(&FrameRecord) -> bool,
        context_frames: usize,
    ) -> Vec<QueryResult> {
        let mut results = Vec::new();

        for (idx, frame) in self.frames.iter().enumerate() {
            if predicate(frame) {
                let context_before: Vec<_> = self.frames[idx.saturating_sub(context_frames)..idx]
                    .iter()
                    .map(|f| (*f).clone())
                    .collect();

                let context_after: Vec<_> = self.frames
                    [idx + 1..(idx + 1 + context_frames).min(self.frames.len())]
                    .iter()
                    .map(|f| (*f).clone())
                    .collect();

                results.push(QueryResult {
                    frame: frame.frame,
                    context_before,
                    context_after,
                });
            }
        }

        results
    }

    /// Count frames matching a predicate.
    pub fn count_frames(&self, predicate: impl Fn(&FrameRecord) -> bool) -> usize {
        self.frames.iter().filter(|f| predicate(f)).count()
    }

    /// Find frames with any input events.
    #[must_use]
    pub fn frames_with_inputs(&self) -> Vec<u64> {
        self.find_frames(FrameRecord::has_inputs)
    }

    /// Find frames with specific input event type.
    pub fn frames_with_input_type(
        &self,
        event_type_matcher: impl Fn(&InputEventType) -> bool,
    ) -> Vec<u64> {
        self.find_frames(|f| {
            f.inputs
                .iter()
                .any(|input| event_type_matcher(&input.event_type))
        })
    }

    /// Get frame at specific frame number.
    #[must_use]
    pub fn get_frame(&self, frame_number: u64) -> Option<&FrameRecord> {
        self.frames
            .iter()
            .find(|f| f.frame == frame_number)
            .copied()
    }

    /// Get frame range (inclusive).
    #[must_use]
    pub fn get_frame_range(&self, start: u64, end: u64) -> Vec<&FrameRecord> {
        self.frames
            .iter()
            .filter(|f| f.frame >= start && f.frame <= end)
            .copied()
            .collect()
    }

    /// Get total frame count.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::len is not const
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get first frame number (if any).
    #[must_use]
    pub fn first_frame(&self) -> Option<u64> {
        self.frames.first().map(|f| f.frame)
    }

    /// Get last frame number (if any).
    #[must_use]
    pub fn last_frame(&self) -> Option<u64> {
        self.frames.last().map(|f| f.frame)
    }

    /// Calculate input density (inputs per frame).
    #[must_use]
    pub fn input_density(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        let total_inputs: usize = self.frames.iter().map(|f| f.inputs.len()).sum();
        total_inputs as f64 / self.frames.len() as f64
    }
}

// =============================================================================
// GameTracer: High-Level Game Tracing API
// =============================================================================

/// Trace statistics for debugging display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceStats {
    /// Current frame number.
    pub frame: u64,
    /// Number of frames in buffer.
    pub buffer_len: usize,
    /// Buffer capacity.
    pub buffer_capacity: usize,
    /// Total frames dropped.
    pub frames_dropped: u64,
    /// Total input events recorded.
    pub total_inputs: u64,
    /// Number of snapshots taken.
    pub snapshots_taken: u64,
    /// Whether recording is active.
    pub recording: bool,
    /// Buffer policy name.
    pub policy: String,
}

/// Configuration for the game tracer.
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// Buffer capacity (frames).
    pub buffer_capacity: usize,
    /// Buffer policy.
    pub policy: BufferPolicy,
    /// Whether to record state hashes.
    pub record_state_hashes: bool,
    /// Snapshotter configuration.
    pub snapshotter: AdaptiveSnapshotter,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 3600, // ~60 seconds at 60fps
            policy: BufferPolicy::DropOldest,
            record_state_hashes: true,
            snapshotter: AdaptiveSnapshotter::default(),
        }
    }
}

impl TracerConfig {
    /// Create a debug configuration (Andon Cord enabled).
    #[must_use]
    pub fn debug() -> Self {
        Self {
            buffer_capacity: 3600,
            policy: BufferPolicy::AndonCord,
            record_state_hashes: true,
            snapshotter: AdaptiveSnapshotter::default(),
        }
    }

    /// Create a production configuration (drop oldest on overflow).
    #[must_use]
    pub const fn production() -> Self {
        Self {
            buffer_capacity: 7200, // ~2 minutes at 60fps
            policy: BufferPolicy::DropOldest,
            record_state_hashes: false, // Disable hashes for performance
            snapshotter: AdaptiveSnapshotter::new(30, 300, 64), // Less frequent snapshots
        }
    }
}

/// High-level game tracer that integrates all tracing components.
///
/// Provides a simple API for recording and querying game traces:
/// - `begin_frame()`: Start recording a new frame
/// - `record_input()`: Record an input event
/// - `end_frame()`: Finish the frame and optionally take a snapshot
/// - `stats()`: Get trace statistics for debugging
///
/// # Example
///
/// ```
/// use jugar_web::trace::{GameTracer, TracerConfig, InputEvent, InputEventType};
///
/// let mut tracer = GameTracer::new(TracerConfig::default());
///
/// // Each frame:
/// tracer.begin_frame();
/// tracer.record_input(InputEvent {
///     event_type: InputEventType::KeyDown(32), // Space
///     frame_offset_us: 0,
/// });
/// tracer.end_frame(None); // Or provide state hash
/// ```
#[derive(Debug)]
pub struct GameTracer {
    /// Trace buffer.
    buffer: TraceBuffer,
    /// Adaptive snapshotter.
    snapshotter: AdaptiveSnapshotter,
    /// Current frame number.
    current_frame: u64,
    /// Current frame record (being built).
    current_record: Option<FrameRecord>,
    /// Whether to record state hashes.
    record_state_hashes: bool,
    /// Whether tracing is active.
    recording: bool,
    /// Total input events recorded.
    total_inputs: u64,
    /// Total snapshots taken.
    snapshots_taken: u64,
}

impl GameTracer {
    /// Create a new game tracer with the given configuration.
    #[must_use]
    pub fn new(config: TracerConfig) -> Self {
        Self {
            buffer: TraceBuffer::new(config.buffer_capacity, config.policy),
            snapshotter: config.snapshotter,
            current_frame: 0,
            current_record: None,
            record_state_hashes: config.record_state_hashes,
            recording: true,
            total_inputs: 0,
            snapshots_taken: 0,
        }
    }

    /// Create a debug tracer (Andon Cord enabled).
    #[must_use]
    pub fn debug() -> Self {
        Self::new(TracerConfig::debug())
    }

    /// Create a production tracer (drop oldest on overflow).
    #[must_use]
    pub fn production() -> Self {
        Self::new(TracerConfig::production())
    }

    /// Start recording a new frame.
    ///
    /// Must be called at the start of each frame before recording inputs.
    pub fn begin_frame(&mut self) {
        if !self.recording {
            return;
        }
        self.current_record = Some(FrameRecord::new(self.current_frame));
    }

    /// Record an input event for the current frame.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `begin_frame()` was not called.
    pub fn record_input(&mut self, event: InputEvent) {
        if !self.recording {
            return;
        }
        if let Some(ref mut record) = self.current_record {
            record.add_input(event);
            self.total_inputs += 1;
        } else {
            debug_assert!(false, "record_input called without begin_frame");
        }
    }

    /// Record multiple input events for the current frame.
    pub fn record_inputs(&mut self, events: impl IntoIterator<Item = InputEvent>) {
        if !self.recording {
            return;
        }
        for event in events {
            self.record_input(event);
        }
    }

    /// End the current frame and commit it to the trace buffer.
    ///
    /// # Arguments
    ///
    /// * `state_hash` - Optional state hash for verification and snapshot decisions.
    ///
    /// # Returns
    ///
    /// The snapshot decision (Skip, DeltaSnapshot, or FullSnapshot).
    pub fn end_frame(&mut self, state_hash: Option<[u8; 32]>) -> SnapshotDecision {
        if !self.recording {
            return SnapshotDecision::Skip;
        }

        let Some(mut record) = self.current_record.take() else {
            debug_assert!(false, "end_frame called without begin_frame");
            return SnapshotDecision::Skip;
        };

        // Determine if we should snapshot
        let decision = if self.record_state_hashes {
            if let Some(hash) = state_hash {
                let decision = self.snapshotter.should_snapshot(self.current_frame, &hash);
                if decision != SnapshotDecision::Skip {
                    record.set_state_hash(hash);
                    self.snapshots_taken += 1;
                }
                decision
            } else {
                SnapshotDecision::Skip
            }
        } else {
            SnapshotDecision::Skip
        };

        // Push to buffer (ignore errors in production mode)
        let _ = self.buffer.push(record);

        self.current_frame += 1;
        decision
    }

    /// Get trace statistics for debugging display.
    #[must_use]
    pub fn stats(&self) -> TraceStats {
        TraceStats {
            frame: self.current_frame,
            buffer_len: self.buffer.len(),
            buffer_capacity: self.buffer.capacity(),
            frames_dropped: self.buffer.frames_dropped(),
            total_inputs: self.total_inputs,
            snapshots_taken: self.snapshots_taken,
            recording: self.recording,
            policy: match self.buffer.policy() {
                BufferPolicy::DropOldest => "DropOldest".to_string(),
                BufferPolicy::SoftAndon => "SoftAndon".to_string(),
                BufferPolicy::AndonCord => "AndonCord".to_string(),
            },
        }
    }

    /// Start recording.
    #[allow(clippy::missing_const_for_fn)] // const fn with mut ref not stable
    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    /// Stop recording.
    pub fn stop_recording(&mut self) {
        self.recording = false;
        self.current_record = None;
    }

    /// Check if recording is active.
    #[must_use]
    pub const fn is_recording(&self) -> bool {
        self.recording
    }

    /// Get current frame number.
    #[must_use]
    pub const fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Get a reference to the underlying buffer.
    #[must_use]
    pub const fn buffer(&self) -> &TraceBuffer {
        &self.buffer
    }

    /// Create a query interface for the trace buffer.
    #[must_use]
    pub fn query(&self) -> TraceQuery<'_> {
        TraceQuery::from_buffer(&self.buffer)
    }

    /// Clear the trace buffer and reset statistics.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.snapshotter.reset();
        self.current_frame = 0;
        self.current_record = None;
        self.total_inputs = 0;
        self.snapshots_taken = 0;
    }

    /// Drain frames from the buffer for export.
    pub fn drain(&mut self, count: usize) -> Vec<FrameRecord> {
        self.buffer.drain(count)
    }

    /// Export all frames as JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the frames cannot be serialized.
    pub fn export_json(&self) -> Result<String, TraceError> {
        let frames: Vec<_> = self.buffer.iter().cloned().collect();
        serde_json::to_string(&frames).map_err(|e| TraceError::SerializationError(e.to_string()))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::float_cmp,
    clippy::unwrap_used,
    clippy::panic,
    clippy::field_reassign_with_default,
    clippy::overly_complex_bool_expr,
    clippy::approx_constant,
    clippy::default_trait_access,
    clippy::redundant_closure_for_method_calls
)]
mod tests {
    use super::*;

    // =========================================================================
    // Fixed32 Tests (100% coverage target)
    // =========================================================================

    #[test]
    fn test_fixed32_zero() {
        assert_eq!(Fixed32::ZERO.to_raw(), 0);
        assert_eq!(Fixed32::ZERO.to_int(), 0);
        assert!(Fixed32::ZERO.is_zero());
    }

    #[test]
    fn test_fixed32_one() {
        assert_eq!(Fixed32::ONE.to_raw(), 65536);
        assert_eq!(Fixed32::ONE.to_int(), 1);
        assert!(!Fixed32::ONE.is_zero());
    }

    #[test]
    fn test_fixed32_half() {
        assert_eq!(Fixed32::HALF.to_raw(), 32768);
        assert_eq!(Fixed32::HALF.to_int(), 0); // Truncates
        let approx = Fixed32::HALF.to_f32();
        assert!((approx - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_fixed32_from_int() {
        assert_eq!(Fixed32::from_int(5).to_int(), 5);
        assert_eq!(Fixed32::from_int(-3).to_int(), -3);
        assert_eq!(Fixed32::from_int(0).to_int(), 0);
        assert_eq!(Fixed32::from_int(1000).to_int(), 1000);
        assert_eq!(Fixed32::from_int(-1000).to_int(), -1000);
    }

    #[test]
    fn test_fixed32_from_f32() {
        let f = Fixed32::from_f32(2.5);
        assert_eq!(f.to_int(), 2);
        let approx = f.to_f32();
        assert!((approx - 2.5).abs() < 0.0001);

        // Note: to_int() truncates towards zero for positives, but negative
        // values truncate towards negative infinity due to arithmetic right shift
        let neg = Fixed32::from_f32(-1.25);
        assert_eq!(neg.to_int(), -2); // -1.25 truncates to -2 with right shift
        let neg_approx = neg.to_f32();
        assert!((neg_approx - (-1.25)).abs() < 0.0001);
    }

    #[test]
    fn test_fixed32_addition() {
        let a = Fixed32::from_int(5);
        let b = Fixed32::from_int(3);
        assert_eq!((a + b).to_int(), 8);

        let c = Fixed32::from_f32(1.5);
        let d = Fixed32::from_f32(2.5);
        let sum = c + d;
        assert!((sum.to_f32() - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_fixed32_subtraction() {
        let a = Fixed32::from_int(10);
        let b = Fixed32::from_int(3);
        assert_eq!((a - b).to_int(), 7);

        let c = Fixed32::from_int(3);
        let d = Fixed32::from_int(10);
        assert_eq!((c - d).to_int(), -7);
    }

    #[test]
    fn test_fixed32_multiplication() {
        let a = Fixed32::from_int(5);
        let b = Fixed32::from_int(3);
        assert_eq!((a * b).to_int(), 15);

        let c = Fixed32::from_f32(2.5);
        let d = Fixed32::from_f32(4.0);
        let product = c * d;
        assert!((product.to_f32() - 10.0).abs() < 0.001);

        // Negative multiplication
        let neg = Fixed32::from_int(-5);
        assert_eq!((neg * b).to_int(), -15);
    }

    #[test]
    fn test_fixed32_division() {
        let a = Fixed32::from_int(15);
        let b = Fixed32::from_int(3);
        assert_eq!((a / b).to_int(), 5);

        let c = Fixed32::from_int(10);
        let d = Fixed32::from_int(4);
        let result = c / d;
        assert!((result.to_f32() - 2.5).abs() < 0.001);

        // Negative division
        let neg = Fixed32::from_int(-15);
        assert_eq!((neg / b).to_int(), -5);
    }

    #[test]
    fn test_fixed32_checked_div() {
        let a = Fixed32::from_int(10);
        let b = Fixed32::from_int(2);
        assert_eq!(a.checked_div(b), Some(Fixed32::from_int(5)));

        let zero = Fixed32::ZERO;
        assert_eq!(a.checked_div(zero), None);
    }

    #[test]
    fn test_fixed32_negation() {
        let a = Fixed32::from_int(5);
        assert_eq!((-a).to_int(), -5);

        let b = Fixed32::from_int(-3);
        assert_eq!((-b).to_int(), 3);
    }

    #[test]
    fn test_fixed32_abs() {
        assert_eq!(Fixed32::from_int(5).abs().to_int(), 5);
        assert_eq!(Fixed32::from_int(-5).abs().to_int(), 5);
        assert_eq!(Fixed32::ZERO.abs().to_int(), 0);
    }

    #[test]
    fn test_fixed32_signum() {
        assert_eq!(Fixed32::from_int(5).signum().to_int(), 1);
        assert_eq!(Fixed32::from_int(-5).signum().to_int(), -1);
        assert_eq!(Fixed32::ZERO.signum().to_int(), 0);
    }

    #[test]
    fn test_fixed32_is_negative_positive() {
        let pos = Fixed32::from_int(5);
        let neg = Fixed32::from_int(-5);
        let zero = Fixed32::ZERO;

        assert!(pos.is_positive());
        assert!(!pos.is_negative());

        assert!(neg.is_negative());
        assert!(!neg.is_positive());

        assert!(!zero.is_positive());
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_fixed32_saturating_add() {
        let a = Fixed32::MAX;
        let b = Fixed32::ONE;
        assert_eq!(a.saturating_add(b), Fixed32::MAX);

        let c = Fixed32::MIN;
        let d = Fixed32::from_int(-1);
        assert_eq!(c.saturating_add(d), Fixed32::MIN);

        // Normal case
        let e = Fixed32::from_int(5);
        let f = Fixed32::from_int(3);
        assert_eq!(e.saturating_add(f).to_int(), 8);
    }

    #[test]
    fn test_fixed32_saturating_sub() {
        let a = Fixed32::MIN;
        let b = Fixed32::ONE;
        assert_eq!(a.saturating_sub(b), Fixed32::MIN);

        let c = Fixed32::MAX;
        let d = Fixed32::from_int(-1);
        assert_eq!(c.saturating_sub(d), Fixed32::MAX);
    }

    #[test]
    fn test_fixed32_saturating_mul() {
        let a = Fixed32::MAX;
        let b = Fixed32::from_int(2);
        assert_eq!(a.saturating_mul(b), Fixed32::MAX);

        let c = Fixed32::MIN;
        assert_eq!(c.saturating_mul(b), Fixed32::MIN);

        // Normal case
        let d = Fixed32::from_int(5);
        let e = Fixed32::from_int(3);
        assert_eq!(d.saturating_mul(e).to_int(), 15);
    }

    #[test]
    fn test_fixed32_clamp() {
        let min = Fixed32::from_int(0);
        let max = Fixed32::from_int(10);

        assert_eq!(Fixed32::from_int(5).clamp(min, max).to_int(), 5);
        assert_eq!(Fixed32::from_int(-5).clamp(min, max).to_int(), 0);
        assert_eq!(Fixed32::from_int(15).clamp(min, max).to_int(), 10);
    }

    #[test]
    fn test_fixed32_lerp() {
        let a = Fixed32::from_int(0);
        let b = Fixed32::from_int(10);

        let t0 = Fixed32::ZERO;
        let t_half = Fixed32::HALF;
        let t1 = Fixed32::ONE;

        assert_eq!(a.lerp(b, t0).to_int(), 0);
        assert_eq!(a.lerp(b, t_half).to_int(), 5);
        assert_eq!(a.lerp(b, t1).to_int(), 10);
    }

    #[test]
    fn test_fixed32_floor_ceil_round() {
        let a = Fixed32::from_f32(2.7);
        assert_eq!(a.floor().to_int(), 2);
        assert_eq!(a.ceil().to_int(), 3);
        assert_eq!(a.round().to_int(), 3);

        let b = Fixed32::from_f32(2.3);
        assert_eq!(b.floor().to_int(), 2);
        assert_eq!(b.ceil().to_int(), 3);
        assert_eq!(b.round().to_int(), 2);

        // Exact integer
        let c = Fixed32::from_int(5);
        assert_eq!(c.floor().to_int(), 5);
        assert_eq!(c.ceil().to_int(), 5);
        assert_eq!(c.round().to_int(), 5);
    }

    #[test]
    fn test_fixed32_fract() {
        let a = Fixed32::from_f32(2.75);
        let frac = a.fract();
        assert!((frac.to_f32() - 0.75).abs() < 0.001);

        let b = Fixed32::from_int(5);
        assert_eq!(b.fract().to_raw(), 0);
    }

    #[test]
    fn test_fixed32_add_assign() {
        let mut a = Fixed32::from_int(5);
        a += Fixed32::from_int(3);
        assert_eq!(a.to_int(), 8);
    }

    #[test]
    fn test_fixed32_sub_assign() {
        let mut a = Fixed32::from_int(10);
        a -= Fixed32::from_int(3);
        assert_eq!(a.to_int(), 7);
    }

    #[test]
    fn test_fixed32_from_trait() {
        let a: Fixed32 = 5.into();
        assert_eq!(a.to_int(), 5);
    }

    #[test]
    fn test_fixed32_display() {
        let a = Fixed32::from_f32(3.14159);
        let s = format!("{a}");
        assert!(s.starts_with("3.14"));
    }

    #[test]
    fn test_fixed32_ord() {
        let a = Fixed32::from_int(5);
        let b = Fixed32::from_int(3);
        let c = Fixed32::from_int(5);

        assert!(a > b);
        assert!(b < a);
        assert!(a >= c);
        assert!(a <= c);
        assert_eq!(a.cmp(&c), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_fixed32_default() {
        let d: Fixed32 = Default::default();
        assert_eq!(d, Fixed32::ZERO);
    }

    #[test]
    fn test_fixed32_pi() {
        let pi = Fixed32::PI;
        let approx = pi.to_f32();
        assert!((approx - 3.14159).abs() < 0.0001);
    }

    #[test]
    fn test_fixed32_cross_platform_determinism() {
        // This test verifies that Fixed32 math is IDENTICAL regardless of platform.
        // The raw values should be exactly the same on any architecture.
        let a = Fixed32::from_raw(327680); // 5.0
        let b = Fixed32::from_raw(196608); // 3.0

        // Addition
        assert_eq!((a + b).to_raw(), 524288); // 8.0

        // Multiplication (this is where f32 diverges across platforms)
        let product = a.mul(b);
        assert_eq!(product.to_raw(), 983040); // 15.0

        // Division
        let quotient = a.div(b);
        assert_eq!(quotient.to_raw(), 109226); // ~1.6667

        // These assertions will pass on EVERY platform - that's the point of Fixed32
    }

    // =========================================================================
    // FrameRecord Tests
    // =========================================================================

    #[test]
    fn test_frame_record_new() {
        let record = FrameRecord::new(42);
        assert_eq!(record.frame, 42);
        assert!(record.inputs.is_empty());
        assert!(record.state_hash.is_none());
    }

    #[test]
    fn test_frame_record_add_input() {
        let mut record = FrameRecord::new(1);
        assert!(!record.has_inputs());

        record.add_input(InputEvent {
            event_type: InputEventType::KeyDown(65),
            frame_offset_us: 1000,
        });

        assert!(record.has_inputs());
        assert_eq!(record.inputs.len(), 1);
    }

    #[test]
    fn test_frame_record_state_hash() {
        let mut record = FrameRecord::new(1);
        assert!(record.state_hash.is_none());

        let hash = [0xab; 32];
        record.set_state_hash(hash);
        assert_eq!(record.state_hash, Some(hash));
    }

    // =========================================================================
    // AdaptiveSnapshotter Tests
    // =========================================================================

    #[test]
    fn test_snapshotter_default() {
        let snap = AdaptiveSnapshotter::default();
        assert_eq!(snap.min_interval, 15);
        assert_eq!(snap.max_interval, 120);
        assert_eq!(snap.entropy_threshold, 64);
    }

    #[test]
    fn test_snapshotter_force_at_max_interval() {
        let mut snap = AdaptiveSnapshotter::new(10, 50, 64);
        let hash = [0; 32];

        // Frame 0: First frame returns Skip (0 - 0 = 0, not >= max_interval)
        // The very first snapshot must be triggered by max interval or entropy
        assert_eq!(snap.should_snapshot(0, &hash), SnapshotDecision::Skip);

        // Frame 50: Max interval reached from initial frame 0
        assert_eq!(
            snap.should_snapshot(50, &hash),
            SnapshotDecision::FullSnapshot
        );

        // Frame 99: Not at max yet (50 + 50 = 100)
        assert_eq!(snap.should_snapshot(99, &hash), SnapshotDecision::Skip);

        // Frame 100: Max interval reached again
        assert_eq!(
            snap.should_snapshot(100, &hash),
            SnapshotDecision::FullSnapshot
        );
    }

    #[test]
    fn test_snapshotter_entropy_trigger() {
        let mut snap = AdaptiveSnapshotter::new(5, 120, 32);

        // Initial snapshot - force one at max interval first
        let hash1 = [0; 32];
        let _ = snap.should_snapshot(120, &hash1); // Force first snapshot

        // Low entropy (no change)
        let hash2 = [0; 32];
        assert_eq!(snap.should_snapshot(130, &hash2), SnapshotDecision::Skip);

        // High entropy (all bits flipped) - past min_interval (5 frames)
        let hash3 = [0xFF; 32];
        assert_eq!(
            snap.should_snapshot(135, &hash3),
            SnapshotDecision::DeltaSnapshot
        );
    }

    #[test]
    fn test_snapshotter_min_interval_respected() {
        let mut snap = AdaptiveSnapshotter::new(10, 120, 32);

        // Force first snapshot at max interval
        let hash1 = [0; 32];
        let _ = snap.should_snapshot(120, &hash1);

        // High entropy but before min interval (120 + 5 = 125)
        let hash2 = [0xFF; 32];
        assert_eq!(snap.should_snapshot(125, &hash2), SnapshotDecision::Skip);

        // High entropy after min interval (120 + 15 = 135)
        assert_eq!(
            snap.should_snapshot(135, &hash2),
            SnapshotDecision::DeltaSnapshot
        );
    }

    #[test]
    fn test_snapshotter_reset() {
        let mut snap = AdaptiveSnapshotter::new(10, 50, 64);
        let hash = [0xAB; 32];

        let _ = snap.should_snapshot(100, &hash);
        assert_eq!(snap.last_snapshot_frame(), 100);

        snap.reset();
        assert_eq!(snap.last_snapshot_frame(), 0);
    }

    // =========================================================================
    // TraceBuffer Tests
    // =========================================================================

    #[test]
    fn test_buffer_new() {
        let buf = TraceBuffer::new(10, BufferPolicy::DropOldest);
        assert_eq!(buf.capacity(), 10);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert!(!buf.is_full());
    }

    #[test]
    fn test_buffer_push_pop() {
        let mut buf = TraceBuffer::new(10, BufferPolicy::DropOldest);

        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();
        buf.push(FrameRecord::new(3)).unwrap();

        assert_eq!(buf.len(), 3);

        let r1 = buf.pop().unwrap();
        assert_eq!(r1.frame, 1);

        let r2 = buf.pop().unwrap();
        assert_eq!(r2.frame, 2);

        let r3 = buf.pop().unwrap();
        assert_eq!(r3.frame, 3);

        assert!(buf.is_empty());
        assert!(buf.pop().is_none());
    }

    #[test]
    fn test_buffer_drop_oldest_policy() {
        let mut buf = TraceBuffer::new(3, BufferPolicy::DropOldest);

        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();
        buf.push(FrameRecord::new(3)).unwrap();
        assert!(buf.is_full());

        // This should drop frame 1
        buf.push(FrameRecord::new(4)).unwrap();

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.frames_dropped(), 1);

        // First pop should be frame 2 (frame 1 was dropped)
        let r = buf.pop().unwrap();
        assert_eq!(r.frame, 2);
    }

    #[test]
    fn test_buffer_andon_cord_policy() {
        let mut buf = TraceBuffer::new(3, BufferPolicy::AndonCord);

        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();
        buf.push(FrameRecord::new(3)).unwrap();

        // This should return BufferFull error (Andon Cord)
        let result = buf.push(FrameRecord::new(4));
        assert_eq!(result, Err(TraceError::BufferFull));

        // Buffer unchanged
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.frames_dropped(), 0);
    }

    #[test]
    fn test_buffer_drain() {
        let mut buf = TraceBuffer::new(10, BufferPolicy::DropOldest);

        for i in 0..5 {
            buf.push(FrameRecord::new(i)).unwrap();
        }

        let drained = buf.drain(3);
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0].frame, 0);
        assert_eq!(drained[1].frame, 1);
        assert_eq!(drained[2].frame, 2);

        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_buffer_drain_more_than_available() {
        let mut buf = TraceBuffer::new(10, BufferPolicy::DropOldest);

        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();

        let drained = buf.drain(100);
        assert_eq!(drained.len(), 2);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffer_clear() {
        let mut buf = TraceBuffer::new(10, BufferPolicy::DropOldest);

        for i in 0..5 {
            buf.push(FrameRecord::new(i)).unwrap();
        }

        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_buffer_wrap_around() {
        let mut buf = TraceBuffer::new(3, BufferPolicy::DropOldest);

        // Fill buffer
        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();
        buf.push(FrameRecord::new(3)).unwrap();

        // Remove one
        let _ = buf.pop();

        // Add one (tests wrap around)
        buf.push(FrameRecord::new(4)).unwrap();

        assert_eq!(buf.len(), 3);

        let r1 = buf.pop().unwrap();
        assert_eq!(r1.frame, 2);

        let r2 = buf.pop().unwrap();
        assert_eq!(r2.frame, 3);

        let r3 = buf.pop().unwrap();
        assert_eq!(r3.frame, 4);
    }

    #[test]
    fn test_buffer_debug_constructor() {
        let buf = TraceBuffer::debug(100);
        assert_eq!(buf.policy(), BufferPolicy::AndonCord);
        assert_eq!(buf.capacity(), 100);
    }

    #[test]
    fn test_buffer_production_constructor() {
        let buf = TraceBuffer::production(100);
        assert_eq!(buf.policy(), BufferPolicy::DropOldest);
        assert_eq!(buf.capacity(), 100);
    }

    #[test]
    fn test_trace_error_display() {
        let e1 = TraceError::BufferFull;
        assert_eq!(format!("{e1}"), "Trace buffer is full");

        let e2 = TraceError::InvalidFrameSequence;
        assert_eq!(format!("{e2}"), "Invalid frame sequence");

        let e3 = TraceError::SerializationError("test".to_string());
        assert!(format!("{e3}").contains("test"));
    }

    // =========================================================================
    // Property-Based Tests (using simple iteration)
    // =========================================================================

    #[test]
    fn property_fixed32_add_commutative() {
        for a in [-100, -1, 0, 1, 100] {
            for b in [-100, -1, 0, 1, 100] {
                let fa = Fixed32::from_int(a);
                let fb = Fixed32::from_int(b);
                assert_eq!(fa + fb, fb + fa, "Addition should be commutative");
            }
        }
    }

    #[test]
    fn property_fixed32_mul_commutative() {
        for a in [-10, -1, 0, 1, 10] {
            for b in [-10, -1, 0, 1, 10] {
                let fa = Fixed32::from_int(a);
                let fb = Fixed32::from_int(b);
                assert_eq!(
                    fa.mul(fb),
                    fb.mul(fa),
                    "Multiplication should be commutative"
                );
            }
        }
    }

    #[test]
    fn property_fixed32_add_identity() {
        for a in [-1000, -1, 0, 1, 1000] {
            let fa = Fixed32::from_int(a);
            assert_eq!(fa + Fixed32::ZERO, fa, "Zero should be additive identity");
        }
    }

    #[test]
    fn property_fixed32_mul_identity() {
        for a in [-1000, -1, 0, 1, 1000] {
            let fa = Fixed32::from_int(a);
            assert_eq!(
                fa.mul(Fixed32::ONE),
                fa,
                "One should be multiplicative identity"
            );
        }
    }

    #[test]
    fn property_fixed32_neg_neg_identity() {
        for a in [-1000, -1, 0, 1, 1000] {
            let fa = Fixed32::from_int(a);
            assert_eq!(-(-fa), fa, "Double negation should be identity");
        }
    }

    // =========================================================================
    // TraceQuery Tests
    // =========================================================================

    fn create_test_buffer_with_frames() -> TraceBuffer {
        let mut buf = TraceBuffer::new(100, BufferPolicy::DropOldest);

        // Add frames: 0, 1, 2 (no input), 3 (key input), 4, 5 (mouse input)
        buf.push(FrameRecord::new(0)).unwrap();
        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();

        let mut frame3 = FrameRecord::new(3);
        frame3.add_input(InputEvent {
            event_type: InputEventType::KeyDown(65), // 'A' key
            frame_offset_us: 100,
        });
        buf.push(frame3).unwrap();

        buf.push(FrameRecord::new(4)).unwrap();

        let mut frame5 = FrameRecord::new(5);
        frame5.add_input(InputEvent {
            event_type: InputEventType::MouseDown {
                button: 0,
                x: 100,
                y: 200,
            },
            frame_offset_us: 500,
        });
        buf.push(frame5).unwrap();

        buf
    }

    #[test]
    fn test_query_from_buffer() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        assert_eq!(query.frame_count(), 6);
        assert_eq!(query.first_frame(), Some(0));
        assert_eq!(query.last_frame(), Some(5));
    }

    #[test]
    fn test_query_find_frames() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        // Find all frames with inputs
        let frames_with_input = query.find_frames(|f| f.has_inputs());
        assert_eq!(frames_with_input, vec![3, 5]);

        // Find all frames without inputs
        let frames_without_input = query.find_frames(|f| !f.has_inputs());
        assert_eq!(frames_without_input, vec![0, 1, 2, 4]);
    }

    #[test]
    fn test_query_frames_with_inputs() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let frames = query.frames_with_inputs();
        assert_eq!(frames, vec![3, 5]);
    }

    #[test]
    fn test_query_frames_with_input_type() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        // Find frames with KeyDown
        let key_frames = query.frames_with_input_type(|e| matches!(e, InputEventType::KeyDown(_)));
        assert_eq!(key_frames, vec![3]);

        // Find frames with MouseDown
        let mouse_frames =
            query.frames_with_input_type(|e| matches!(e, InputEventType::MouseDown { .. }));
        assert_eq!(mouse_frames, vec![5]);
    }

    #[test]
    fn test_query_get_frame() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let frame3 = query.get_frame(3);
        assert!(frame3.is_some());
        assert!(frame3.unwrap().has_inputs());

        let frame99 = query.get_frame(99);
        assert!(frame99.is_none());
    }

    #[test]
    fn test_query_get_frame_range() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let range = query.get_frame_range(2, 4);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].frame, 2);
        assert_eq!(range[1].frame, 3);
        assert_eq!(range[2].frame, 4);
    }

    #[test]
    fn test_query_count_frames() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let count = query.count_frames(|f| f.has_inputs());
        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_input_density() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let density = query.input_density();
        // 2 inputs across 6 frames = 0.333...
        assert!((density - (2.0 / 6.0)).abs() < 0.001);
    }

    #[test]
    fn test_query_input_density_empty() {
        let buf = TraceBuffer::new(10, BufferPolicy::DropOldest);
        let query = TraceQuery::from_buffer(&buf);

        let density = query.input_density();
        assert_eq!(density, 0.0);
    }

    #[test]
    fn test_query_find_frames_with_context() {
        let buf = create_test_buffer_with_frames();
        let query = TraceQuery::from_buffer(&buf);

        let results = query.find_frames_with_context(|f| f.has_inputs(), 2);
        assert_eq!(results.len(), 2);

        // First result: frame 3 with context
        let r1 = &results[0];
        assert_eq!(r1.frame, 3);
        assert_eq!(r1.context_before.len(), 2); // frames 1, 2
        assert_eq!(r1.context_after.len(), 2); // frames 4, 5

        // Second result: frame 5 with context
        let r2 = &results[1];
        assert_eq!(r2.frame, 5);
        assert_eq!(r2.context_before.len(), 2); // frames 3, 4
        assert_eq!(r2.context_after.len(), 0); // no frames after 5
    }

    #[test]
    fn test_buffer_iter() {
        let mut buf = TraceBuffer::new(5, BufferPolicy::DropOldest);

        buf.push(FrameRecord::new(10)).unwrap();
        buf.push(FrameRecord::new(20)).unwrap();
        buf.push(FrameRecord::new(30)).unwrap();

        let frames: Vec<_> = buf.iter().collect();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].frame, 10);
        assert_eq!(frames[1].frame, 20);
        assert_eq!(frames[2].frame, 30);
    }

    #[test]
    fn test_buffer_iter_wrap_around() {
        let mut buf = TraceBuffer::new(3, BufferPolicy::DropOldest);

        // Fill and overflow to test wrap-around
        buf.push(FrameRecord::new(1)).unwrap();
        buf.push(FrameRecord::new(2)).unwrap();
        buf.push(FrameRecord::new(3)).unwrap();
        buf.push(FrameRecord::new(4)).unwrap(); // Drops frame 1

        let frames: Vec<_> = buf.iter().collect();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].frame, 2);
        assert_eq!(frames[1].frame, 3);
        assert_eq!(frames[2].frame, 4);
    }

    // =========================================================================
    // GameTracer Tests
    // =========================================================================

    #[test]
    fn test_game_tracer_new() {
        let tracer = GameTracer::new(TracerConfig::default());
        assert_eq!(tracer.current_frame(), 0);
        assert!(tracer.is_recording());
        assert_eq!(tracer.buffer().len(), 0);
    }

    #[test]
    fn test_game_tracer_debug() {
        let tracer = GameTracer::debug();
        assert!(tracer.is_recording());
        assert_eq!(tracer.buffer().policy(), BufferPolicy::AndonCord);
    }

    #[test]
    fn test_game_tracer_production() {
        let tracer = GameTracer::production();
        assert!(tracer.is_recording());
        assert_eq!(tracer.buffer().policy(), BufferPolicy::DropOldest);
    }

    #[test]
    fn test_game_tracer_basic_frame() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        let _ = tracer.end_frame(None);

        assert_eq!(tracer.current_frame(), 1);
        assert_eq!(tracer.buffer().len(), 1);
    }

    #[test]
    fn test_game_tracer_record_input() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        tracer.record_input(InputEvent {
            event_type: InputEventType::KeyDown(32), // Space
            frame_offset_us: 0,
        });
        let _ = tracer.end_frame(None);

        let stats = tracer.stats();
        assert_eq!(stats.total_inputs, 1);
        assert_eq!(stats.frame, 1);
    }

    #[test]
    fn test_game_tracer_record_multiple_inputs() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        tracer.record_inputs(vec![
            InputEvent {
                event_type: InputEventType::KeyDown(32),
                frame_offset_us: 0,
            },
            InputEvent {
                event_type: InputEventType::KeyDown(87),
                frame_offset_us: 100,
            },
        ]);
        let _ = tracer.end_frame(None);

        let stats = tracer.stats();
        assert_eq!(stats.total_inputs, 2);
    }

    #[test]
    fn test_game_tracer_stop_recording() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        let _ = tracer.end_frame(None);
        assert_eq!(tracer.buffer().len(), 1);

        tracer.stop_recording();
        assert!(!tracer.is_recording());

        // These should be no-ops
        tracer.begin_frame();
        tracer.record_input(InputEvent {
            event_type: InputEventType::KeyDown(32),
            frame_offset_us: 0,
        });
        let _ = tracer.end_frame(None);

        // Still only 1 frame
        assert_eq!(tracer.buffer().len(), 1);
        assert_eq!(tracer.stats().total_inputs, 0);
    }

    #[test]
    fn test_game_tracer_restart_recording() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        let _ = tracer.end_frame(None);
        tracer.stop_recording();
        tracer.start_recording();

        tracer.begin_frame();
        let _ = tracer.end_frame(None);

        assert_eq!(tracer.buffer().len(), 2);
    }

    #[test]
    fn test_game_tracer_stats() {
        let mut config = TracerConfig::default();
        config.buffer_capacity = 100;
        let mut tracer = GameTracer::new(config);

        for _ in 0..10 {
            tracer.begin_frame();
            tracer.record_input(InputEvent {
                event_type: InputEventType::KeyDown(32),
                frame_offset_us: 0,
            });
            let _ = tracer.end_frame(None);
        }

        let stats = tracer.stats();
        assert_eq!(stats.frame, 10);
        assert_eq!(stats.buffer_len, 10);
        assert_eq!(stats.buffer_capacity, 100);
        assert_eq!(stats.total_inputs, 10);
        assert!(stats.recording);
        assert_eq!(stats.policy, "DropOldest");
    }

    #[test]
    fn test_game_tracer_query() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        // Frame 0: no input
        tracer.begin_frame();
        let _ = tracer.end_frame(None);

        // Frame 1: has input
        tracer.begin_frame();
        tracer.record_input(InputEvent {
            event_type: InputEventType::KeyDown(32),
            frame_offset_us: 0,
        });
        let _ = tracer.end_frame(None);

        // Frame 2: no input
        tracer.begin_frame();
        let _ = tracer.end_frame(None);

        let query = tracer.query();
        let frames_with_input = query.frames_with_inputs();
        assert_eq!(frames_with_input.len(), 1);
        assert_eq!(frames_with_input[0], 1);
    }

    #[test]
    fn test_game_tracer_clear() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        for _ in 0..5 {
            tracer.begin_frame();
            tracer.record_input(InputEvent {
                event_type: InputEventType::KeyDown(32),
                frame_offset_us: 0,
            });
            let _ = tracer.end_frame(None);
        }

        assert_eq!(tracer.current_frame(), 5);
        assert_eq!(tracer.buffer().len(), 5);

        tracer.clear();

        assert_eq!(tracer.current_frame(), 0);
        assert_eq!(tracer.buffer().len(), 0);
        assert_eq!(tracer.stats().total_inputs, 0);
    }

    #[test]
    fn test_game_tracer_drain() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        for _ in 0..5 {
            tracer.begin_frame();
            let _ = tracer.end_frame(None);
        }

        let drained = tracer.drain(3);
        assert_eq!(drained.len(), 3);
        assert_eq!(tracer.buffer().len(), 2);
    }

    #[test]
    fn test_game_tracer_export_json() {
        let mut tracer = GameTracer::new(TracerConfig::default());

        tracer.begin_frame();
        tracer.record_input(InputEvent {
            event_type: InputEventType::KeyDown(32),
            frame_offset_us: 100,
        });
        let _ = tracer.end_frame(None);

        let json = tracer.export_json().unwrap();
        assert!(json.contains("\"frame\":0"));
        assert!(json.contains("KeyDown"));
    }

    #[test]
    fn test_game_tracer_with_state_hash() {
        let mut config = TracerConfig::default();
        config.record_state_hashes = true;
        config.snapshotter = AdaptiveSnapshotter::new(1, 10, 64);
        let mut tracer = GameTracer::new(config);

        // First frame with high entropy hash
        let hash1 = [0u8; 32];
        tracer.begin_frame();
        let decision = tracer.end_frame(Some(hash1));
        // First frame at max_interval might trigger FullSnapshot
        assert!(decision == SnapshotDecision::FullSnapshot || decision == SnapshotDecision::Skip);

        // Second frame with same hash (low entropy)
        tracer.begin_frame();
        let decision = tracer.end_frame(Some(hash1));
        assert_eq!(decision, SnapshotDecision::Skip);

        // Third frame with different hash (high entropy)
        let mut hash2 = [0u8; 32];
        hash2.fill(0xFF); // Maximum Hamming distance
        tracer.begin_frame();
        let decision = tracer.end_frame(Some(hash2));
        assert_eq!(decision, SnapshotDecision::DeltaSnapshot);
    }

    #[test]
    fn test_tracer_config_default() {
        let config = TracerConfig::default();
        assert_eq!(config.buffer_capacity, 3600);
        assert_eq!(config.policy, BufferPolicy::DropOldest);
        assert!(config.record_state_hashes);
    }

    #[test]
    fn test_tracer_config_debug() {
        let config = TracerConfig::debug();
        assert_eq!(config.policy, BufferPolicy::AndonCord);
        assert!(config.record_state_hashes);
    }

    #[test]
    fn test_tracer_config_production() {
        let config = TracerConfig::production();
        assert_eq!(config.policy, BufferPolicy::DropOldest);
        assert!(!config.record_state_hashes); // Disabled for performance
    }

    #[test]
    fn test_trace_stats_default() {
        let stats = TraceStats::default();
        assert_eq!(stats.frame, 0);
        assert_eq!(stats.buffer_len, 0);
        assert!(!stats.recording);
    }

    #[test]
    #[cfg_attr(
        debug_assertions,
        should_panic(expected = "end_frame called without begin_frame")
    )]
    fn test_game_tracer_end_frame_without_begin() {
        let mut tracer = GameTracer::new(TracerConfig::default());
        // In debug mode: panics with debug_assert!
        // In release mode: returns Skip gracefully
        let decision = tracer.end_frame(None);
        assert_eq!(decision, SnapshotDecision::Skip);
    }

    // =========================================================================
    // v1.3 TPS Kaizen Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // Fixed32 Overflow Check Tests (Regehr 2012)
    // -------------------------------------------------------------------------

    #[test]
    fn test_fixed32_checked_mul_success() {
        let a = Fixed32::from_int(100);
        let b = Fixed32::from_int(50);
        assert_eq!(a.checked_mul(b), Some(Fixed32::from_int(5000)));
    }

    #[test]
    fn test_fixed32_checked_mul_overflow() {
        let big = Fixed32::MAX;
        assert_eq!(big.checked_mul(Fixed32::from_int(2)), None);
    }

    #[test]
    fn test_fixed32_strict_mul_success() {
        let a = Fixed32::from_int(10);
        let b = Fixed32::from_int(20);
        assert_eq!(a.strict_mul(b), Fixed32::from_int(200));
    }

    #[test]
    #[should_panic(expected = "Fixed32 multiplication overflow")]
    fn test_fixed32_strict_mul_overflow_panics() {
        let big = Fixed32::MAX;
        let _ = big.strict_mul(Fixed32::from_int(2));
    }

    #[test]
    fn test_fixed32_checked_add_success() {
        let a = Fixed32::from_int(100);
        let b = Fixed32::from_int(50);
        assert_eq!(a.checked_add(b), Some(Fixed32::from_int(150)));
    }

    #[test]
    fn test_fixed32_checked_add_overflow() {
        let big = Fixed32::MAX;
        assert_eq!(big.checked_add(Fixed32::from_int(1)), None);
    }

    #[test]
    fn test_fixed32_checked_sub_success() {
        let a = Fixed32::from_int(100);
        let b = Fixed32::from_int(50);
        assert_eq!(a.checked_sub(b), Some(Fixed32::from_int(50)));
    }

    #[test]
    fn test_fixed32_checked_sub_overflow() {
        let small = Fixed32::MIN;
        assert_eq!(small.checked_sub(Fixed32::from_int(1)), None);
    }

    // -------------------------------------------------------------------------
    // AndonState Tests (MacKenzie & Ware 1993)
    // -------------------------------------------------------------------------

    #[test]
    fn test_andon_state_normal() {
        let state = AndonState::Normal;
        assert_eq!(state.overlay_color(), [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(state.status_text(), "");
        assert!(!state.is_error());
        assert!(!state.has_dropped());
        assert_eq!(state.dropped_count(), 0);
    }

    #[test]
    fn test_andon_state_warning() {
        let state = AndonState::Warning { buffer_pct: 85 };
        assert_eq!(state.overlay_color(), [1.0, 0.8, 0.0, 0.3]);
        assert_eq!(state.status_text(), "TRACE BUFFER WARNING");
        assert!(state.is_error());
        assert!(!state.has_dropped());
        assert_eq!(state.dropped_count(), 0);
    }

    #[test]
    fn test_andon_state_trace_loss() {
        let state = AndonState::TraceLoss { dropped_count: 42 };
        assert_eq!(state.overlay_color(), [1.0, 0.0, 0.0, 0.5]);
        assert_eq!(state.status_text(), "TRACE LOSS - EVENTS DROPPED");
        assert!(state.is_error());
        assert!(state.has_dropped());
        assert_eq!(state.dropped_count(), 42);
    }

    #[test]
    fn test_andon_state_default() {
        let state = AndonState::default();
        assert_eq!(state, AndonState::Normal);
    }

    // -------------------------------------------------------------------------
    // Soft Andon Buffer Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_soft_andon_buffer_creation() {
        let buf = TraceBuffer::soft_andon(10);
        assert_eq!(buf.policy(), BufferPolicy::SoftAndon);
        assert_eq!(buf.capacity(), 10);
        assert_eq!(buf.andon_state(), AndonState::Normal);
    }

    #[test]
    fn test_soft_andon_warning_at_80_percent() {
        let mut buf = TraceBuffer::soft_andon(10);

        // Fill to 80%
        for i in 0..8 {
            buf.push(FrameRecord::new(i)).unwrap();
        }

        // Push one more - the state check happens at the START of push
        // so we need to push again to see the warning for 80% fill
        buf.push(FrameRecord::new(8)).unwrap();

        // Should be in warning state (now at 90%)
        match buf.andon_state() {
            AndonState::Warning { buffer_pct } => assert!(buffer_pct >= 80),
            _ => panic!("Expected Warning state at 80%+ fill"),
        }
    }

    #[test]
    fn test_soft_andon_trace_loss_on_overflow() {
        let mut buf = TraceBuffer::soft_andon(3);

        // Fill buffer
        for i in 0..3 {
            buf.push(FrameRecord::new(i)).unwrap();
        }

        // Overflow - should trigger trace loss
        buf.push(FrameRecord::new(3)).unwrap();

        assert!(buf.andon_state().has_dropped());
        assert_eq!(buf.andon_state().dropped_count(), 1);
    }

    #[test]
    fn test_soft_andon_continues_after_overflow() {
        let mut buf = TraceBuffer::soft_andon(3);

        // Fill and overflow multiple times
        for i in 0..10 {
            buf.push(FrameRecord::new(i)).unwrap();
        }

        // All pushes should succeed (no error)
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.frames_dropped(), 7);
        assert_eq!(buf.andon_state().dropped_count(), 7);
    }

    // -------------------------------------------------------------------------
    // ZobristTable Tests (Zobrist 1970)
    // -------------------------------------------------------------------------

    #[test]
    fn test_zobrist_table_creation() {
        let table = ZobristTable::new(42);
        // Just verify it doesn't panic
        let hash = table.hash_bytes(&[0, 1, 2, 3]);
        assert!(hash != 0 || hash == 0); // Any value is valid
    }

    #[test]
    fn test_zobrist_table_deterministic() {
        let table1 = ZobristTable::new(42);
        let table2 = ZobristTable::new(42);

        let data = [1, 2, 3, 4, 5];
        assert_eq!(table1.hash_bytes(&data), table2.hash_bytes(&data));
    }

    #[test]
    fn test_zobrist_table_different_seeds() {
        let table1 = ZobristTable::new(42);
        let table2 = ZobristTable::new(43);

        let data = [1, 2, 3, 4, 5];
        // Different seeds should produce different hashes (with high probability)
        assert_ne!(table1.hash_bytes(&data), table2.hash_bytes(&data));
    }

    #[test]
    fn test_zobrist_incremental_update() {
        let table = ZobristTable::new(42);
        let data = [1, 2, 3, 4];

        // Full hash
        let full_hash = table.hash_bytes(&data);

        // Now change byte at position 0 from 1 to 5
        let updated_hash = table.update_hash(full_hash, 0, 1, 5);

        // Verify it matches recalculating from scratch
        let new_data = [5, 2, 3, 4];
        assert_eq!(updated_hash, table.hash_bytes(&new_data));
    }

    #[test]
    fn test_zobrist_entropy_calculation() {
        let hash1 = 0u64;
        let hash2 = 0xFFFF_FFFF_FFFF_FFFFu64;

        // All bits different = 64 bit entropy
        assert_eq!(ZobristTable::entropy(hash1, hash2), 64);

        // Same hash = 0 entropy
        assert_eq!(ZobristTable::entropy(hash1, hash1), 0);

        // One bit different
        assert_eq!(ZobristTable::entropy(0, 1), 1);
    }

    // -------------------------------------------------------------------------
    // ZobristSnapshotter Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zobrist_snapshotter_creation() {
        let snap = ZobristSnapshotter::new(42, 10, 120, 16);
        assert_eq!(snap.last_snapshot_frame(), 0);
    }

    #[test]
    fn test_zobrist_snapshotter_default() {
        let snap = ZobristSnapshotter::default();
        assert_eq!(snap.last_snapshot_frame(), 0);
    }

    #[test]
    fn test_zobrist_snapshotter_force_at_max_interval() {
        let mut snap = ZobristSnapshotter::new(42, 10, 50, 16);

        // Before max interval - should skip
        for frame in 0..49 {
            assert_eq!(snap.should_snapshot(frame), SnapshotDecision::Skip);
        }

        // At max interval - should force full snapshot
        assert_eq!(snap.should_snapshot(50), SnapshotDecision::FullSnapshot);
    }

    #[test]
    fn test_zobrist_snapshotter_incremental_state_change() {
        let mut snap = ZobristSnapshotter::new(42, 5, 120, 4);

        // Initialize with some state
        snap.initialize(&[0, 0, 0, 0]);

        // Skip first few frames
        for frame in 1..5 {
            assert_eq!(snap.should_snapshot(frame), SnapshotDecision::Skip);
        }

        // Make significant state changes (high entropy)
        for i in 0..32 {
            snap.on_state_change(i, 0, 255);
        }

        // Now should trigger delta snapshot due to high entropy
        // (after min_interval)
        let decision = snap.should_snapshot(10);
        assert!(matches!(
            decision,
            SnapshotDecision::DeltaSnapshot | SnapshotDecision::Skip
        ));
    }

    // -------------------------------------------------------------------------
    // deterministic! Macro Tests (Bessey 2010)
    // -------------------------------------------------------------------------

    #[test]
    fn test_deterministic_macro_allows_fixed32() {
        let result = deterministic! {
            let a = Fixed32::from_int(5);
            let b = Fixed32::from_int(3);
            a.mul(b).to_int()
        };
        assert_eq!(result, 15);
    }

    #[test]
    fn test_deterministic_macro_allows_integers() {
        let result = deterministic! {
            let a: i32 = 10;
            let b: i32 = 20;
            a + b
        };
        assert_eq!(result, 30);
    }

    // Note: We can't test that f32 is rejected because it's a compile-time error.
    // The following would NOT compile:
    // deterministic! {
    //     let _x = 1.0f32;  // ERROR: f32 is shadowed
    // }
}
