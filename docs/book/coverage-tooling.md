# WASM Coverage Tooling

This chapter covers Probar's novel WASM coverage instrumentation framework, which applies Toyota Production System (TPS) principles to achieve reliable, efficient coverage collection.

## Overview

Probar implements a **renderfarm-inspired** coverage system where:
- WASM modules are decomposed into **basic blocks** (like Pixar's RenderMan buckets)
- Blocks are grouped into **superblocks** for efficient scheduling
- Coverage is collected with **thread-local buffering** to eliminate contention
- **Soft Jidoka** distinguishes between fatal and recoverable errors

## Toyota Way Principles

### Poka-Yoke (Error Prevention)

Type-safe identifiers prevent mixing up different ID types at compile time:

```rust
use jugar_probar::coverage::{BlockId, FunctionId, EdgeId};

// These are distinct types - cannot be mixed
let block = BlockId::new(42);
let function = FunctionId::new(1);

// EdgeId encodes source and target in single u64
let edge = EdgeId::new(BlockId::new(10), BlockId::new(20));
assert_eq!(edge.source().as_u32(), 10);
assert_eq!(edge.target().as_u32(), 20);
```

**Key insight**: The Rust type system catches ID confusion at compile time, eliminating a class of runtime bugs.

### Muda Elimination (Thread-Local Buffering)

Traditional coverage tools use atomic increments for every block hit, causing O(N) contention:

```
Traditional: Thread → Atomic Counter (contention on every hit)
```

Probar uses thread-local buffering to batch updates:

```
Probar: Thread → Local Buffer → Batch Flush → Global Counter
```

```rust
use jugar_probar::coverage::{ThreadLocalCounters, BlockId};

// Create thread-local counters for 100 blocks
let mut counters = ThreadLocalCounters::new(100);

// Fast local increments (no atomics)
counters.increment(BlockId::new(0));
counters.increment(BlockId::new(1));
counters.increment(BlockId::new(0)); // Second hit

// Single atomic operation to flush
let flushed = counters.flush();
assert_eq!(flushed[0], 2); // Block 0 hit twice
assert_eq!(flushed[1], 1); // Block 1 hit once
```

**Complexity reduction**: O(N) atomic operations → O(B) batched operations, where B << N.

### Jidoka (Stop and Fix)

Per TPS §5.1.1, we distinguish between:

| Violation Type | Severity | Action |
|----------------|----------|--------|
| Uninstrumented execution | Critical | **Stop** - can't trust data |
| Impossible edge | Critical | **Stop** - instrumentation bug |
| Counter overflow | Warning | **LogAndContinue** - taint block |
| Coverage regression | Warning | **LogAndContinue** - record |

```rust
use jugar_probar::coverage::{CoverageViolation, JidokaAction, BlockId};

let violation = CoverageViolation::CounterOverflow {
    block_id: BlockId::new(42),
};

match violation.action() {
    JidokaAction::Stop => panic!("Fatal error"),
    JidokaAction::LogAndContinue => {
        // Record but continue collecting
        println!("Warning: {}", violation.description());
    }
    JidokaAction::Warn => { /* Just log */ }
}
```

### Heijunka (Work Leveling)

Basic blocks are too small for efficient parallel scheduling. Probar groups them into **superblocks**:

```rust
use jugar_probar::coverage::{SuperblockBuilder, BlockId, FunctionId};

let blocks: Vec<BlockId> = (0..1000).map(BlockId::new).collect();
let function = FunctionId::new(0);

// Group into superblocks of ~64 blocks each
let builder = SuperblockBuilder::new()
    .with_target_size(64)
    .with_max_size(256);

let superblocks = builder.build_from_blocks(&blocks, function);
// ~16 superblocks for work-stealing scheduler
```

## Coverage Collection

### Basic Session

```rust
use jugar_probar::coverage::{
    CoverageCollector, CoverageConfig, Granularity, BlockId,
};

// Configure coverage
let config = CoverageConfig::builder()
    .granularity(Granularity::BasicBlock)
    .parallel(true)
    .jidoka_enabled(true)
    .build();

let mut collector = CoverageCollector::new(config);

// Run coverage session
collector.begin_session("my_tests");

collector.begin_test("test_ball_movement");
collector.record_hit(BlockId::new(0));
collector.record_hit(BlockId::new(1));
collector.end_test();

collector.begin_test("test_paddle_input");
collector.record_hit(BlockId::new(2));
collector.end_test();

let report = collector.end_session();

// Analyze results
let summary = report.summary();
println!("Coverage: {:.1}%", summary.coverage_percent);
println!("Covered: {}/{}", summary.covered_blocks, summary.total_blocks);
```

### Parallel Execution with Superblocks

```rust
use jugar_probar::coverage::{
    CoverageExecutor, Superblock, SuperblockId, SuperblockResult,
    BlockId, FunctionId,
};

// Create superblocks
let sb1 = Superblock::new(
    SuperblockId::new(0),
    vec![BlockId::new(0), BlockId::new(1)],
    FunctionId::new(0),
);
let sb2 = Superblock::new(
    SuperblockId::new(1),
    vec![BlockId::new(2), BlockId::new(3)],
    FunctionId::new(0),
);

// Execute with work-stealing
let executor = CoverageExecutor::new(vec![sb1, sb2])
    .with_workers(4)
    .with_work_stealing(true);

let report = executor.execute(|superblock| {
    // Run tests for this superblock
    let success = run_tests_for(superblock);

    SuperblockResult {
        id: superblock.id(),
        success,
        error: if success { None } else { Some("Test failed".into()) },
    }
});
```

## Popperian Falsification

Following Karl Popper's scientific methodology, every coverage claim must be **falsifiable**. Probar supports four null hypotheses:

### H₀-COV-01: Determinism

Coverage should be identical across independent runs:

```rust
use jugar_probar::coverage::{CoverageHypothesis, NullificationConfig};

let hypothesis = CoverageHypothesis::determinism();
let observations = vec![95.0, 95.0, 95.0, 95.0, 95.0];

let result = hypothesis.evaluate(&observations);
println!("{}", result.report());
// H0-COV-01: NOT REJECTED (p=0.500, 95% CI [95.0, 95.0], d=0.00)
```

### H₀-COV-02: Completeness

Coverage should meet a threshold:

```rust
let hypothesis = CoverageHypothesis::completeness(90.0);
let observations = vec![92.0, 93.0, 91.5, 94.0, 92.5];

let result = hypothesis.evaluate(&observations);
assert!(!result.rejected); // Mean (92.6%) > threshold (90%)
```

### H₀-COV-03: No Regression

Coverage should not decrease from baseline:

```rust
let hypothesis = CoverageHypothesis::no_regression(88.0);
let current = vec![90.0, 89.0, 91.0, 88.5, 90.5];

let result = hypothesis.evaluate(&current);
assert!(!result.rejected); // Mean (89.8%) >= baseline (88%)
```

### H₀-COV-04: Mutation Correlation

Coverage should correlate with mutation score:

```rust
let hypothesis = CoverageHypothesis::mutation_correlation(0.8);
let coverage_scores = vec![95.0, 96.0, 94.0, 95.0, 95.0];

let result = hypothesis.evaluate(&coverage_scores);
// Estimated correlation based on coverage level
```

## Configuration Reference

### CoverageConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `granularity` | `Granularity` | `BasicBlock` | Function, BasicBlock, or Edge |
| `parallel` | `bool` | `false` | Enable parallel collection |
| `jidoka_enabled` | `bool` | `true` | Enable soft Jidoka |
| `flush_threshold` | `usize` | `1000` | Thread-local flush threshold |

### NullificationConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `runs` | `usize` | `5` | Independent runs (Princeton methodology) |
| `alpha` | `f64` | `0.05` | Significance level |

## Running the Example

```bash
cargo run --example coverage_demo -p jugar-probar
```

This demonstrates all Toyota Way principles in action.

## References

1. Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*
2. Popper, K. (1959). *The Logic of Scientific Discovery*
3. Liker, J. (2004). *The Toyota Way: 14 Management Principles*
4. Nethercote, N. & Seward, J. (2007). *Valgrind: A Framework for Heavyweight Dynamic Binary Instrumentation*
5. Cadar, C. et al. (2008). *KLEE: Unassisted and Automatic Generation of High-Coverage Tests*

See `docs/specifications/probar-wasm-coverage-tooling.md` for the complete specification with 35 peer-reviewed citations.
