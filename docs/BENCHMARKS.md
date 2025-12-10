# Benchmark Methodology

This document describes the statistical methodology used for performance benchmarks in Jugar.

## Sample Size Justification

All benchmarks use Criterion.rs with the following configuration:

- **Warm-up**: 3 seconds to reach steady-state performance
- **Sample Size**: 100 measurements per benchmark
- **Measurement Time**: 5 seconds per benchmark iteration
- **Confidence Level**: 95% (default Criterion setting)

The sample size of 100 is chosen based on:
1. Central Limit Theorem requirements (n ≥ 30 for normal approximation)
2. Practical trade-off between precision and CI runtime
3. Criterion's automatic outlier detection and removal

## Statistical Measures

### Confidence Intervals

Criterion provides 95% confidence intervals for all timing measurements:
```
time:   [1.0261 µs 1.0303 µs 1.0347 µs]
         ^^^^^^^^  ^^^^^^^^  ^^^^^^^^
         lower     median    upper
         bound              bound
```

### Effect Size

Performance changes between runs are reported with:
- **Percentage change**: Relative difference from baseline
- **Change classification**: improvement/regression/no-change
- **Statistical significance**: t-test with p < 0.05 threshold

## Benchmark Categories

### ECS Iteration (`ecs_iteration`)
Measures entity-component iteration performance:
- **10 entities**: Baseline small-world performance
- **100 entities**: Typical mobile game entity count
- **1,000 entities**: Mid-range game entity count
- **10,000 entities**: Stress test for large entity counts

### Game Loop (`game_loop`)
Measures fixed-timestep game loop overhead:
- **Frame time consistency**: Standard deviation of frame times
- **Accumulator behavior**: Drift detection over 1000 frames

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run with detailed HTML report
cargo criterion

# Compare against baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

## Reproducibility

Benchmarks are deterministic when run:
1. On the same hardware configuration
2. With CPU frequency scaling disabled (`cpupower frequency-set -g performance`)
3. With minimal background processes
4. After system warm-up (not immediately after boot)

The CI environment uses dedicated runners with consistent specifications to minimize variance.
