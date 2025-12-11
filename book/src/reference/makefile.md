# Makefile Targets

Complete reference for all Makefile targets.

## Quick Reference

| Target | Description |
|--------|-------------|
| `make tier1` | Sub-second feedback (ON-SAVE) |
| `make tier2` | Full validation (ON-COMMIT) |
| `make tier3` | Mutation testing (ON-MERGE) |
| `make build-web` | Build WASM for web |
| `make test-e2e` | Run Probar E2E tests |
| `make coverage` | Generate coverage report |

## Tiered Workflow

### Tier 1: ON-SAVE (Sub-second)

```bash
make tier1
```

Runs:
1. Type checking (`cargo check`)
2. Fast clippy (library only)
3. Unit tests
4. Property tests (10 cases)

### Tier 2: ON-COMMIT (1-5 minutes)

```bash
make tier2
```

Runs:
1. Zero JavaScript verification
2. Batuta dependency check
3. Format check
4. Full clippy
5. All tests
6. Property tests (256 cases)
7. Coverage analysis
8. TDG grade check

### Tier 3: ON-MERGE (Hours)

```bash
make tier3
```

Runs:
1. All Tier 2 checks
2. Mutation testing (80% target)
3. Security audit
4. Benchmarks
5. PMAT repo score

## Build Targets

```bash
make build           # Host target (dev)
make build-release   # Host target (optimized)
make build-wasm      # WASM target (release)
make build-wasm-dev  # WASM target (debug)
make build-web       # Build with wasm-pack
make serve-web       # Serve locally (port 8080)
```

## Test Targets

```bash
make test            # All tests
make test-fast       # Library tests only (<2 min)
make test-e2e        # Probar E2E tests
make test-e2e-verbose # E2E with output
make test-property   # Property tests (50 cases)
make test-property-full # Property tests (500 cases)
```

## Quality Targets

```bash
make lint            # Full clippy
make fmt             # Format code
make fmt-check       # Check formatting
```

## Coverage Targets

```bash
make coverage        # Generate HTML report
make coverage-summary # Show summary
make coverage-open   # Open in browser
make coverage-check  # Enforce 95% threshold
make coverage-ci     # Generate LCOV for CI
make coverage-clean  # Clean artifacts
```

## PMAT Targets

```bash
make pmat-tdg          # Technical Debt Grading
make pmat-analyze      # Complexity, SATD, dead code
make pmat-score        # Repository health score
make pmat-rust-score   # Rust project score
make pmat-validate-docs # Documentation validation
make pmat-all          # All PMAT checks
```

## Mutation Testing

```bash
make mutate          # jugar-web crate (<5 min)
make mutate-quick    # Single module (<2 min)
make mutate-file FILE=path/to/file.rs
make mutate-report   # View results
```

## Load Testing

```bash
make load-test       # All load tests
make load-test-quick # Quick chaos tests
make load-test-full  # Full detailed tests
make ai-test         # AI CLI tests
make ai-simulate     # Run AI simulation
```

## Verification

```bash
make verify-no-js    # Zero JavaScript check
make verify-batuta-deps # Batuta dependencies
make verify-wasm-output # Pure WASM output
```

## Kaizen

```bash
make kaizen          # Continuous improvement analysis
```

## Physics Sandbox

```bash
make sandbox         # Build and test
make test-sandbox    # Run tests
make test-sandbox-coverage # With coverage
make sandbox-lint    # Pedantic clippy
make sandbox-mutate  # Mutation testing
make build-sandbox-wasm # WASM build
```

## Development

```bash
make dev             # Watch mode
make install-tools   # Install required tools
make clean           # Clean build artifacts
make clean-pmat      # Clean PMAT artifacts
```

## Book

```bash
make book            # Build mdbook
make book-serve      # Serve locally
make book-open       # Open in browser
```

## Help

```bash
make help            # Show all targets with descriptions
```
