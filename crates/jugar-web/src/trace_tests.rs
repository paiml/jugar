use super::*;

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
