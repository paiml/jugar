//! Probar: Rust-Native Testing Framework for WASM Games
//!
//! Per spec Section 6.1: Probar (Spanish: "to test/prove") is a pure Rust
//! alternative to Playwright/Puppeteer, designed for WASM game testing.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    PROBAR Architecture                           │
//! ├─────────────────────────────────────────────────────────────────┤
//! │   ┌────────────┐    ┌────────────┐    ┌────────────┐            │
//! │   │ Test Spec  │    │ WASM       │    │ Headless   │            │
//! │   │ (Rust)     │───►│ Test       │───►│ Browser    │            │
//! │   │            │    │ Harness    │    │ (chromium) │            │
//! │   └────────────┘    └────────────┘    └────────────┘            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

#[allow(
    clippy::suboptimal_flops,
    clippy::cast_precision_loss,
    clippy::struct_excessive_bools,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::unnecessary_wraps,
    clippy::doc_markdown
)]
mod accessibility;
mod assertion;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown
)]
mod bridge;
mod browser;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    dead_code
)]
mod driver;
mod event;
mod fuzzer;
mod harness;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::unnecessary_wraps,
    clippy::doc_markdown
)]
mod locator;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::cast_precision_loss,
    clippy::format_push_string,
    clippy::needless_raw_string_hashes
)]
mod reporter;
mod result;
#[allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::unnecessary_wraps,
    clippy::doc_markdown,
    clippy::if_not_else,
    clippy::ptr_as_ptr,
    unsafe_code
)]
mod runtime;
mod simulation;
mod snapshot;
mod visual_regression;

pub use accessibility::{
    AccessibilityAudit, AccessibilityConfig, AccessibilityIssue, AccessibilityValidator, Color,
    ContrastAnalysis, ContrastPair, FlashDetector, FlashResult, FocusConfig, KeyboardIssue,
    Severity, MIN_CONTRAST_LARGE, MIN_CONTRAST_NORMAL, MIN_CONTRAST_UI,
};
pub use assertion::{Assertion, AssertionResult};
pub use bridge::{
    BridgeConnection, DiffRegion, EntitySnapshot, GameStateData, GameStateSnapshot, SnapshotCache,
    StateBridge, VisualDiff,
};
pub use browser::{Browser, BrowserConfig, Page};
#[cfg(feature = "browser")]
pub use driver::{BrowserController, ProbarDriver};
pub use driver::{
    DeviceDescriptor, DriverConfig, ElementHandle, MockDriver, NetworkInterceptor, NetworkResponse,
    PageMetrics, Screenshot,
};
pub use event::{InputEvent, Touch, TouchAction};
pub use fuzzer::{
    FuzzerConfig, InputFuzzer, InvariantCheck, InvariantChecker, InvariantViolation, Seed,
};
pub use harness::{TestHarness, TestResult, TestSuite};
pub use locator::{
    expect, BoundingBox, DragBuilder, DragOperation, Expect, ExpectAssertion, Locator,
    LocatorAction, LocatorOptions, LocatorQuery, Point, Selector, DEFAULT_POLL_INTERVAL_MS,
    DEFAULT_TIMEOUT_MS,
};
pub use reporter::{
    AndonCordPulled, FailureMode, Reporter, TestResultEntry, TestStatus, TraceData,
};
pub use result::{ProbarError, ProbarResult};
pub use runtime::{
    ComponentId, EntityId, FrameResult, GameHostState, MemoryView, ProbarComponent, ProbarEntity,
    RuntimeConfig, StateDelta, WasmRuntime,
};
pub use simulation::{
    run_replay, run_simulation, RandomWalkAgent, RecordedFrame, ReplayResult, SimulatedGameState,
    SimulationConfig, SimulationRecording,
};
pub use snapshot::{Snapshot, SnapshotConfig, SnapshotDiff};
pub use visual_regression::{ImageDiffResult, VisualRegressionConfig, VisualRegressionTester};

/// Prelude for convenient imports
pub mod prelude {
    pub use super::accessibility::*;
    pub use super::assertion::*;
    pub use super::bridge::*;
    pub use super::browser::*;
    pub use super::driver::*;
    pub use super::event::*;
    pub use super::fuzzer::*;
    pub use super::harness::*;
    pub use super::locator::*;
    pub use super::reporter::*;
    pub use super::result::*;
    pub use super::runtime::*;
    pub use super::simulation::*;
    pub use super::snapshot::*;
    pub use super::visual_regression::*;
}

/// Standard invariants for game testing
pub mod standard_invariants {
    pub use super::fuzzer::standard_invariants::*;
}

// Re-export derive macros when the `derive` feature is enabled (Phase 4: Poka-Yoke)
#[cfg(feature = "derive")]
pub use jugar_probar_derive::{probar_test, ProbarComponent, ProbarEntity, ProbarSelector};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ========================================================================
    // EXTREME TDD: Tests written FIRST per spec Section 6.1
    // ========================================================================

    mod browser_tests {
        use super::*;

        #[test]
        fn test_browser_config_defaults() {
            let config = BrowserConfig::default();
            assert!(config.headless);
            assert_eq!(config.viewport_width, 800);
            assert_eq!(config.viewport_height, 600);
        }

        #[test]
        fn test_browser_config_builder() {
            let config = BrowserConfig::default()
                .with_viewport(1024, 768)
                .with_headless(false);
            assert!(!config.headless);
            assert_eq!(config.viewport_width, 1024);
            assert_eq!(config.viewport_height, 768);
        }
    }

    mod touch_tests {
        use super::*;

        #[test]
        fn test_touch_tap() {
            let touch = Touch::tap(100.0, 200.0);
            assert!((touch.x - 100.0).abs() < f32::EPSILON);
            assert!((touch.y - 200.0).abs() < f32::EPSILON);
            assert!(matches!(touch.action, TouchAction::Tap));
        }

        #[test]
        fn test_touch_swipe() {
            let touch = Touch::swipe(0.0, 0.0, 100.0, 0.0, 300);
            assert!(matches!(touch.action, TouchAction::Swipe { .. }));
        }

        #[test]
        fn test_touch_hold() {
            let touch = Touch::hold(50.0, 50.0, 500);
            assert!(matches!(touch.action, TouchAction::Hold { .. }));
        }
    }

    mod assertion_tests {
        use super::*;

        #[test]
        fn test_assertion_equals_pass() {
            let result = Assertion::equals(&42, &42);
            assert!(result.passed);
        }

        #[test]
        fn test_assertion_equals_fail() {
            let result = Assertion::equals(&42, &43);
            assert!(!result.passed);
            assert!(result.message.contains("expected"));
        }

        #[test]
        fn test_assertion_contains() {
            let result = Assertion::contains("hello world", "world");
            assert!(result.passed);
        }

        #[test]
        fn test_assertion_in_range() {
            let result = Assertion::in_range(5.0, 0.0, 10.0);
            assert!(result.passed);

            let result = Assertion::in_range(15.0, 0.0, 10.0);
            assert!(!result.passed);
        }
    }

    mod snapshot_tests {
        use super::*;

        #[test]
        fn test_snapshot_creation() {
            let snapshot = Snapshot::new("test-snapshot", vec![0, 1, 2, 3]);
            assert_eq!(snapshot.name, "test-snapshot");
            assert_eq!(snapshot.data.len(), 4);
        }

        #[test]
        fn test_snapshot_diff_identical() {
            let snap1 = Snapshot::new("test", vec![1, 2, 3]);
            let snap2 = Snapshot::new("test", vec![1, 2, 3]);
            let diff = snap1.diff(&snap2);
            assert!(diff.is_identical());
        }

        #[test]
        fn test_snapshot_diff_different() {
            let snap1 = Snapshot::new("test", vec![1, 2, 3]);
            let snap2 = Snapshot::new("test", vec![1, 2, 4]);
            let diff = snap1.diff(&snap2);
            assert!(!diff.is_identical());
            assert!(diff.difference_count > 0);
        }

        #[test]
        fn test_snapshot_config() {
            let config = SnapshotConfig::default();
            assert!(!config.update_snapshots);
            assert!(config.threshold > 0.0);
        }
    }

    mod harness_tests {
        use super::*;

        #[test]
        fn test_test_suite_creation() {
            let suite = TestSuite::new("Game Tests");
            assert_eq!(suite.name, "Game Tests");
            assert!(suite.tests.is_empty());
        }

        #[test]
        fn test_test_result_pass() {
            let result = TestResult::pass("test_example");
            assert!(result.passed);
            assert_eq!(result.name, "test_example");
        }

        #[test]
        fn test_test_result_fail() {
            let result = TestResult::fail("test_example", "assertion failed");
            assert!(!result.passed);
            assert!(result.error.is_some());
        }

        #[test]
        fn test_harness_run_empty_suite() {
            let harness = TestHarness::new();
            let suite = TestSuite::new("Empty");
            let results = harness.run(&suite);
            assert!(results.all_passed());
            assert_eq!(results.total(), 0);
        }
    }

    mod input_event_tests {
        use super::*;

        #[test]
        fn test_input_event_touch() {
            let event = InputEvent::touch(100.0, 200.0);
            assert!(matches!(event, InputEvent::Touch { .. }));
        }

        #[test]
        fn test_input_event_key() {
            let event = InputEvent::key_press("ArrowUp");
            assert!(matches!(event, InputEvent::KeyPress { .. }));
        }

        #[test]
        fn test_input_event_mouse() {
            let event = InputEvent::mouse_click(50.0, 75.0);
            assert!(matches!(event, InputEvent::MouseClick { .. }));
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_probar_error_display() {
            let err = ProbarError::BrowserNotFound;
            let msg = err.to_string();
            assert!(msg.contains("browser") || msg.contains("Browser"));
        }

        #[test]
        fn test_probar_error_timeout() {
            let err = ProbarError::Timeout { ms: 5000 };
            let msg = err.to_string();
            assert!(msg.contains("5000"));
        }
    }
}
