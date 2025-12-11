# Quality Standards

Jugar follows strict quality standards based on PMAT methodology and Toyota Production System principles.

## Quality Metrics

| Metric | Minimum | Target | Status |
|--------|---------|--------|--------|
| Test Coverage | 85% | 95% | ✅ |
| Mutation Score | 80% | 90% | ✅ |
| TDG Grade | B+ | A+ | ✅ |
| SATD Comments | 5 | 0 | ✅ |
| Unsafe Code | 0 | 0 | ✅ |
| JavaScript | 0 bytes | 0 bytes | ✅ |

## Running Quality Checks

### Tiered Workflow

```bash
# Tier 1: ON-SAVE (sub-second)
make tier1

# Tier 2: ON-COMMIT (1-5 minutes)
make tier2

# Tier 3: ON-MERGE (hours)
make tier3
```

### Individual Checks

```bash
# Formatting
cargo fmt -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Coverage
make coverage

# Mutation testing
make mutate
```

## PMAT Tools

```bash
# Technical Debt Grading
make pmat-tdg

# Repository score
make pmat-score

# Rust project score
make pmat-rust-score

# All PMAT checks
make pmat-all
```

## Test Coverage

### Target: 95%

```bash
# Generate coverage report
make coverage

# Open HTML report
make coverage-open

# CI mode (LCOV)
make coverage-ci
```

### Excluded from Coverage

- Binary files (`bin/*.rs`)
- External crate internals (`wasmtime.*`)
- Proc-macro crates (`probar-derive`)
- Browser-only code (`browser.rs`)

## Mutation Testing

### Target: 80% Kill Rate

```bash
# Full mutation testing
make mutate

# Quick (single module)
make mutate-quick

# Specific file
make mutate-file FILE=crates/jugar-web/src/juice.rs

# View report
make mutate-report
```

## Technical Debt Grading

### Target: B+ Minimum

The TDG system grades code quality:

| Grade | Description |
|-------|-------------|
| A+ | Excellent - minimal debt |
| A  | Very good |
| A- | Good |
| B+ | Acceptable |
| B  | Needs improvement |
| C  | Significant debt |
| D  | Critical debt |
| F  | Failing |

## Continuous Improvement (Kaizen)

```bash
# Run kaizen analysis
make kaizen

# Outputs:
# - Code metrics (LOC, complexity)
# - Coverage analysis
# - Technical debt grade
# - Improvement recommendations
```

## Pre-Commit Checks

Before committing, always run:

```bash
make tier2
```

This verifies:
- Zero JavaScript (policy)
- Batuta dependencies (policy)
- Formatting
- Linting
- All tests
- Property tests
- Coverage
- TDG grade
- SATD comments

## CI/CD Pipeline

All PRs must pass:

```yaml
- cargo fmt -- --check
- cargo clippy --all-targets -- -D warnings
- cargo test --all-features
- make verify-no-js
- make pmat-tdg
```

## Quality Gate Failures

If quality gates fail:

1. **Coverage below 95%**: Add missing tests
2. **Mutation score below 80%**: Strengthen assertions
3. **TDG below B+**: Address technical debt
4. **JavaScript detected**: Remove and use Rust/WASM
5. **Clippy warnings**: Fix all warnings

## Tools Required

```bash
# Install all quality tools
make install-tools

# Includes:
# - cargo-llvm-cov (coverage)
# - cargo-mutants (mutation testing)
# - cargo-nextest (fast test runner)
# - cargo-audit (security)
# - pmat (quality metrics)
```
