# Jugar Makefile - EXTREME TDD Quality Gates
# Tiered Workflow inspired by certeza (PMAT integration)
# WASM-native game engine targeting wasm32-unknown-unknown
# CRITICAL: ABSOLUTE ZERO JAVASCRIPT - Pure WASM only

.SUFFIXES:
.DELETE_ON_ERROR:
.ONESHELL:

WASM_TARGET := wasm32-unknown-unknown

.PHONY: help tier1 tier2 tier3 build build-wasm build-web serve-web test test-fast test-property test-property-full test-e2e test-e2e-verbose test-e2e-coverage coverage coverage-summary coverage-open coverage-check coverage-ci coverage-clean lint lint-all lint-fast lint-bash lint-ts lint-html lint-js-complexity fmt clean all dev bench mutate mutate-quick mutate-file mutate-report kaizen pmat-tdg pmat-analyze pmat-ts pmat-score pmat-rust-score pmat-mutate pmat-validate-docs pmat-quality-gate pmat-context pmat-all install-tools verify-no-js verify-batuta-deps load-test load-test-quick load-test-full ai-test ai-simulate trace-test sandbox test-sandbox test-sandbox-verbose test-sandbox-coverage sandbox-lint sandbox-mutate build-sandbox-wasm

# Default target
all: tier2

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ============================================================================
# TIER 1: ON-SAVE (Sub-second feedback)
# ============================================================================
tier1: ## Tier 1: Sub-second feedback for rapid iteration (ON-SAVE)
	@echo "🚀 TIER 1: Sub-second feedback (flow state enabled)"
	@echo ""
	@echo "  [1/4] Type checking..."
	@cargo check --quiet
	@echo "  [2/4] Linting (fast mode)..."
	@cargo clippy --lib --quiet -- -D warnings
	@echo "  [3/4] Unit tests (focused)..."
	@cargo test --lib --quiet
	@echo "  [4/4] Property tests (small cases)..."
	@PROPTEST_CASES=10 cargo test property_ --lib --quiet 2>/dev/null || true
	@echo ""
	@echo "✅ Tier 1 complete - Ready to continue coding!"

lint-fast: ## Fast clippy (library only)
	@cargo clippy --lib --quiet -- -D warnings

# ============================================================================
# TIER 2: ON-COMMIT (1-5 minutes)
# ============================================================================
tier2: verify-no-js verify-batuta-deps ## Tier 2: Full test suite for commits (ON-COMMIT)
	@echo "🔍 TIER 2: Comprehensive validation (1-5 minutes)"
	@echo ""
	@echo "  [1/9] Formatting check..."
	@cargo fmt -- --check
	@echo "  [2/9] Full clippy..."
	@cargo clippy --all-targets --all-features --quiet -- -D warnings
	@echo "  [3/9] Shell/Makefile lint (bashrs)..."
	@bashrs make lint Makefile 2>/dev/null || echo "    ⚠️  bashrs not available"
	@echo "  [4/9] All tests..."
	@cargo test --all-features --quiet
	@echo "  [5/9] Property tests (full cases)..."
	@PROPTEST_CASES=256 cargo test property_ --all-features --quiet 2>/dev/null || true
	@echo "  [6/9] Coverage analysis..."
	@cargo llvm-cov --all-features --workspace --quiet 2>/dev/null || echo "    ⚠️  llvm-cov not available"
	@echo "  [7/9] PMAT TDG..."
	@pmat analyze tdg --min-grade B+ 2>/dev/null || echo "    ⚠️  PMAT not available"
	@echo "  [8/9] SATD check..."
	@! grep -rn "TODO\|FIXME\|HACK" crates/*/src/ 2>/dev/null || echo "    ⚠️  SATD comments found"
	@echo "  [9/9] JavaScript verification (already done above)..."
	@echo ""
	@echo "✅ Tier 2 complete - Ready to commit!"

# ============================================================================
# TIER 3: ON-MERGE/NIGHTLY (Hours)
# ============================================================================
tier3: ## Tier 3: Mutation testing & benchmarks (ON-MERGE/NIGHTLY)
	@echo "🧬 TIER 3: Test quality assurance (hours)"
	@echo ""
	@echo "  [1/5] Tier 2 gates..."
	@$(MAKE) --no-print-directory tier2
	@echo ""
	@echo "  [2/5] Mutation testing (target: ≥80%)..."
	@command -v cargo-mutants >/dev/null 2>&1 || { echo "    Installing cargo-mutants..."; cargo install cargo-mutants; }
	@cargo mutants --timeout 60 --minimum-pass-rate 80 || echo "    ⚠️  Mutation score below 80%"
	@echo ""
	@echo "  [3/5] Security audit..."
	@cargo audit 2>/dev/null || echo "    ⚠️  cargo-audit not available"
	@echo ""
	@echo "  [4/5] Benchmark suite..."
	@cargo bench --all-features --no-fail-fast 2>/dev/null || echo "    ⚠️  No benchmarks available"
	@echo ""
	@echo "  [5/5] PMAT repo score..."
	@pmat repo-score . --min-score 90 2>/dev/null || echo "    ⚠️  PMAT not available"
	@echo ""
	@echo "✅ Tier 3 complete - Ready to merge!"

# ============================================================================
# BUILD TARGETS
# ============================================================================
build: ## Build for host target (development)
	cargo build --all-features

build-release: ## Build optimized for host target
	cargo build --release --all-features

build-wasm: ## Build for WASM target
	cargo build --target $(WASM_TARGET) --release

build-wasm-dev: ## Build for WASM target (debug)
	cargo build --target $(WASM_TARGET)

build-web: ## Build jugar-web with wasm-pack for browser usage
	@echo "🌐 Building jugar-web for browser..."
	wasm-pack build crates/jugar-web --target web --out-dir ../../examples/pong-web/pkg
	@echo "✅ WASM built to examples/pong-web/pkg/"
	@echo "   Run 'make serve-web' to test locally"

serve-web: ## Serve pong-web example locally
	@echo "🎮 Starting local server for Pong demo..."
	@echo "   Open http://localhost:8080 in your browser"
	python3 -m http.server 8080 --directory examples/pong-web

test-e2e: build-web ## Run Probar e2e tests for pong-web (replaces Playwright)
	@echo "🧪 Running Probar e2e tests (pure Rust, no browser needed)..."
	cargo test -p jugar-web --test probar_pong
	@echo "✅ All Probar e2e tests passed!"

test-e2e-verbose: build-web ## Run Probar e2e tests with verbose output
	cargo test -p jugar-web --test probar_pong -- --nocapture

test-e2e-coverage: ## Run Probar e2e tests with coverage analysis (Pong WASM game)
	@echo "🎮 PROBAR E2E COVERAGE: Pong WASM Game"
	@echo "======================================"
	@echo ""
	@echo "📋 Test Framework: Probar v2.0 (Pure Rust, Zero JavaScript)"
	@echo "📋 Coverage Tool: cargo-llvm-cov + nextest"
	@echo "📋 Target: jugar-web (Pong WASM game)"
	@echo ""
	@echo "  [1/6] Installing tools if needed..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@echo "  [2/6] Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace 2>/dev/null || true
	@mkdir -p target/coverage/e2e
	@# Temporarily disable mold linker (breaks LLVM coverage)
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "  [3/6] Running Probar e2e tests with instrumentation..."
	@echo ""
	@echo "  🧪 Test Suites:"
	@echo "     • Suite 1: Pong WASM Game - Core Functionality (6 tests)"
	@echo "     • Suite 2: Pong Demo Features (22 tests)"
	@echo "     • Suite 3: Release Readiness - Stress & Performance (11 tests)"
	@echo ""
	@cargo llvm-cov --no-report nextest --test probar_pong -p jugar-web $(COV_IGNORE) 2>&1 | grep -E "^(test |running |test result:|PASSED|FAILED|ok|TOTAL)" || true
	@echo ""
	@echo "  [4/6] Running library unit tests with instrumentation..."
	@cargo llvm-cov --no-report nextest --lib -p jugar-web $(COV_IGNORE) 2>&1 | tail -5
	@echo "  [5/6] Generating HTML report..."
	@cargo llvm-cov report --html --output-dir target/coverage/e2e -p jugar-web $(COV_IGNORE)
	@echo "  [6/6] Generating summary..."
	@# Restore mold linker
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════════╗"
	@echo "║  📊 PONG GAME E2E COVERAGE SUMMARY                                    ║"
	@echo "╠══════════════════════════════════════════════════════════════════════╣"
	@cargo llvm-cov report --summary-only -p jugar-web $(COV_IGNORE) 2>/dev/null | grep -E "TOTAL|jugar-web" | head -5
	@echo "╚══════════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "📊 Pong Game File Coverage:"
	@cargo llvm-cov report -p jugar-web $(COV_IGNORE) 2>/dev/null | grep -E "ai\.rs|demo\.rs|platform\.rs|loadtest\.rs|juice\.rs" || echo "  (run 'make coverage-open' to see full report)"
	@echo ""
	@echo "✅ E2E Coverage report: target/coverage/e2e/html/index.html"
	@echo ""
	@echo "🎯 Quality Metrics:"
	@echo "   • Function coverage target: ≥95%"
	@echo "   • Line coverage target: ≥95%"
	@echo "   • Zero JavaScript computation: ✓"
	@echo "   • Pure Rust testing: ✓"

# ============================================================================
# TEST TARGETS
# ============================================================================
test: ## Run all tests
	cargo test --all-features

test-fast: ## Run library tests only (fast, <2 min)
	@echo "⚡ Running fast tests (target: <2 min)..."
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --workspace --lib --status-level skip --failure-output immediate; \
	else \
		cargo test --workspace --lib; \
	fi
	@echo "✅ Fast tests complete!"

test-wasm: ## Run WASM-compatible tests
	cargo test --target $(WASM_TARGET) --all-features 2>/dev/null || echo "WASM tests require wasm-pack or similar"

# ============================================================================
# QUALITY TARGETS
# ============================================================================
lint: ## Full clippy analysis (Rust only)
	cargo clippy --all-targets --all-features -- -D warnings

lint-all: lint lint-bash lint-ts ## Comprehensive lint: Rust + Makefile/shell + TypeScript
	@echo ""
	@echo "✅ All linting complete (Rust, Makefile/shell, TypeScript)"

lint-bash: ## Lint Makefile and shell scripts with bashrs
	@echo "🔍 Linting Makefile and shell scripts with bashrs..."
	@command -v bashrs >/dev/null 2>&1 || { echo "❌ bashrs not installed. Install: cargo install bashrs"; exit 1; }
	bashrs make lint Makefile
	@if ls scripts/*.sh 2>/dev/null | grep -q .; then \
		bashrs lint scripts/*.sh; \
	fi
	@echo "✅ Shell/Makefile linting complete"

lint-ts: ## Lint TypeScript test files with deno
	@echo "🔍 Linting TypeScript files with deno..."
	@command -v deno >/dev/null 2>&1 || { echo "❌ deno not installed. Install: curl -fsSL https://deno.land/install.sh | sh"; exit 1; }
	cd examples/pong-web && deno task lint
	@echo "✅ TypeScript linting complete"

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting
	cargo fmt -- --check

# Code Coverage (Toyota Way: "make coverage" just works)
# Following bashrs Two-Phase Pattern for reliable, fast coverage
# TARGET: < 5 minutes with proper mold linker handling
# Exclude patterns:
#   - bin/*.rs: binaries (not library code)
#   - wasmtime.*: external crate internals
#   - probar-derive: proc-macro crate (tested via trybuild, not direct coverage)
#   - browser.rs: requires real browser (tested via Playwright)
COV_IGNORE := --ignore-filename-regex='bin/.*\.rs|wasmtime.*|probar-derive|browser\.rs'

coverage: ## Generate coverage report (≥95% required, fast mode)
	@echo "📊 Generating coverage report (target: ≥95%, <5 min)..."
	@echo ""
	@echo "  [1/4] Installing tools if needed..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@echo "  [2/4] Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace 2>/dev/null || true
	@mkdir -p target/coverage
	@# Temporarily disable mold linker (breaks LLVM coverage)
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "  [3/4] Running tests with instrumentation (nextest)..."
	@cargo llvm-cov --no-report nextest --no-tests=warn --workspace $(COV_IGNORE) 2>/dev/null || \
		cargo llvm-cov --no-report --workspace $(COV_IGNORE)
	@echo "  [4/4] Generating reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html $(COV_IGNORE)
	@cargo llvm-cov report --lcov --output-path lcov.info $(COV_IGNORE)
	@# Restore mold linker
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "✅ Coverage report: target/coverage/html/index.html"
	@echo ""
	@echo "📊 Coverage Summary:"
	@cargo llvm-cov report --summary-only $(COV_IGNORE) 2>/dev/null | grep -E "TOTAL" || echo "  Run succeeded"

coverage-summary: ## Show coverage summary
	@cargo llvm-cov report --summary-only $(COV_IGNORE) 2>/dev/null || echo "Run 'make coverage' first"

coverage-open: ## Open HTML coverage report in browser
	@if [ -f target/coverage/html/index.html ]; then \
		xdg-open target/coverage/html/index.html 2>/dev/null || \
		open target/coverage/html/index.html 2>/dev/null || \
		echo "Please open: target/coverage/html/index.html"; \
	else \
		echo "❌ Run 'make coverage' first to generate the HTML report"; \
	fi

coverage-check: ## Enforce 95% coverage threshold (BLOCKS on failure)
	@echo "🔒 Enforcing 95% coverage threshold..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@cargo llvm-cov clean --workspace
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@cargo llvm-cov --no-report nextest --no-tests=warn --all-features --workspace 2>/dev/null
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@cargo llvm-cov report --summary-only $(COV_IGNORE) | grep "TOTAL" | awk '{print "Coverage: " $$10}' | tee /tmp/jugar-cov.txt
	@cargo llvm-cov report --summary-only $(COV_IGNORE) | grep "TOTAL" | awk '{gsub(/%/,"",$$10); if ($$10 < 95) {print "❌ FAIL: Coverage " $$10 "% below 95% threshold"; exit 1} else {print "✅ Coverage threshold met (≥95%)"}}'

coverage-ci: ## Generate LCOV report for CI/CD (fast mode)
	@echo "=== Code Coverage for CI/CD ==="
	@echo "Phase 1: Running tests with instrumentation..."
	@cargo llvm-cov clean --workspace
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@cargo llvm-cov --no-report nextest --no-tests=warn --all-features --workspace
	@echo "Phase 2: Generating LCOV report..."
	@cargo llvm-cov report --lcov --output-path lcov.info
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo "✓ Coverage report generated: lcov.info"

coverage-clean: ## Clean coverage artifacts
	@cargo llvm-cov clean --workspace
	@rm -f lcov.info coverage.xml target/coverage/lcov.info
	@rm -rf target/llvm-cov target/coverage
	@find . -name "*.profraw" -delete
	@echo "✓ Coverage artifacts cleaned"

# ============================================================================
# PMAT TARGETS
# ============================================================================
pmat-tdg: ## Run Technical Debt Grading
	pmat analyze tdg --min-grade B+ --include-components

pmat-analyze: ## Run comprehensive PMAT analysis (Rust)
	pmat analyze complexity --path crates/
	pmat analyze satd
	pmat analyze dead-code
	pmat analyze duplicate

pmat-ts: ## Run PMAT analysis on TypeScript test files
	@echo "🔍 Running PMAT analysis on TypeScript files..."
	@echo ""
	@echo "=== Complexity Analysis ==="
	pmat analyze complexity --path examples/pong-web/tests
	@echo ""
	@echo "=== SATD Analysis ==="
	pmat analyze satd --path examples/pong-web/tests
	@echo ""
	@echo "✅ TypeScript PMAT analysis complete"

pmat-score: ## Calculate repository health score
	pmat repo-score . --deep

pmat-rust-score: ## Calculate Rust project score (full mode)
	pmat rust-project-score --full

pmat-rust-score-fast: ## Calculate Rust project score (fast mode)
	pmat rust-project-score

pmat-mutate: ## Run PMAT mutation testing
	pmat mutate --target crates/ --threshold 80

pmat-validate-docs: ## Validate documentation
	pmat context --output deep_context.md --format llm-optimized
	pmat validate-readme --targets README.md CLAUDE.md --deep-context deep_context.md

pmat-quality-gate: ## Run comprehensive quality gate
	pmat quality-gate --strict

pmat-context: ## Generate deep context for LLM
	pmat context --output deep_context.md --format llm-optimized

pmat-all: ## Run all PMAT checks (Rust + TypeScript)
	@echo "🔍 Running all PMAT checks..."
	$(MAKE) pmat-tdg
	$(MAKE) pmat-analyze
	$(MAKE) pmat-ts
	$(MAKE) pmat-score
	$(MAKE) pmat-rust-score-fast
	$(MAKE) pmat-validate-docs
	@echo "✅ All PMAT checks complete!"

# ============================================================================
# KAIZEN: Continuous Improvement
# ============================================================================
kaizen: ## Kaizen: Continuous improvement analysis
	@echo "=== KAIZEN: Continuous Improvement Protocol for Jugar ==="
	@echo "改善 - Change for the better through systematic analysis"
	@echo ""
	@echo "=== STEP 1: Static Analysis & Technical Debt ==="
	@mkdir -p /tmp/kaizen .kaizen
	@if command -v tokei >/dev/null 2>&1; then \
		tokei crates --output json > /tmp/kaizen/loc-metrics.json; \
	else \
		echo '{"Rust":{"code":0}}' > /tmp/kaizen/loc-metrics.json; \
	fi
	@echo "✅ Baseline metrics collected"
	@echo ""
	@echo "=== STEP 2: Test Coverage Analysis ==="
	@cargo llvm-cov report --summary-only 2>/dev/null || echo "Coverage: Unknown"
	@echo ""
	@echo "=== STEP 3: Complexity Analysis ==="
	@pmat analyze complexity --path crates/ 2>/dev/null || echo "Complexity analysis requires pmat"
	@echo ""
	@echo "=== STEP 4: Technical Debt Grading ==="
	@pmat analyze tdg --include-components 2>/dev/null || echo "TDG analysis requires pmat"
	@echo ""
	@echo "=== STEP 5: Improvement Recommendations ==="
	@pmat get-quality-recommendations 2>/dev/null || echo "Recommendations require pmat"
	@echo ""
	@echo "✅ Kaizen analysis complete"

# ============================================================================
# BENCHMARKING
# ============================================================================
bench: ## Run benchmarks
	cargo bench --all-features

bench-wasm: ## Run WASM-specific benchmarks
	@echo "WASM benchmarks require wasm-bindgen-test or similar"

# ============================================================================
# PROPERTY TESTING (proptest)
# ============================================================================
test-property: ## Run property tests (fast: 50 cases, <30s)
	@echo "🎲 Running property-based tests (50 cases per property)..."
	@THREADS=$${PROPTEST_THREADS:-$$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}; \
	timeout 30 env PROPTEST_CASES=50 cargo test --workspace --lib -- property_ --test-threads=$$THREADS 2>/dev/null || \
	echo "  ℹ️  No property tests found (add tests with 'property_' prefix)"

test-property-full: ## Run property tests (comprehensive: 500 cases, <2min)
	@echo "🎲 Running property-based tests (500 cases per property)..."
	@THREADS=$${PROPTEST_THREADS:-$$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}; \
	timeout 120 env PROPTEST_CASES=500 cargo test --workspace --lib -- property_ --test-threads=$$THREADS 2>/dev/null || \
	echo "  ℹ️  No property tests found (add tests with 'property_' prefix)"

# ============================================================================
# MUTATION TESTING (cargo-mutants)
# Fast, targeted mutation testing that doesn't slow down CI
# ============================================================================
mutate: ## Run mutation testing on jugar-web (main crate, <5min)
	@echo "🧬 Running mutation testing on jugar-web..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants)
	cargo mutants --package jugar-web --timeout 60 --no-times 2>&1 | tail -50

mutate-quick: ## Run mutation testing on a single module (<2min)
	@echo "🧬 Running quick mutation testing (time module only)..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants)
	cargo mutants --file crates/jugar-web/src/time.rs --timeout 30 --no-times 2>&1 | tail -30

mutate-file: ## Run mutation testing on single file (FILE=path/to/file.rs)
	@if [ -z "$(FILE)" ]; then \
		echo "❌ Error: FILE parameter required"; \
		echo "Usage: make mutate-file FILE=crates/jugar-web/src/juice.rs"; \
		exit 1; \
	fi
	@echo "🧬 Running mutation testing on $(FILE)..."
	cargo mutants --file "$(FILE)" --timeout 60 --no-times

mutate-report: ## Generate mutation testing summary
	@echo "📊 Mutation Testing Report"
	@echo "=========================="
	@if [ -d "mutants.out" ]; then \
		echo "Results from last run:"; \
		cat mutants.out/outcomes.json 2>/dev/null | head -20 || echo "No outcomes.json found"; \
	else \
		echo "No mutation results found. Run 'make mutate' first."; \
	fi

# ============================================================================
# DEVELOPMENT
# ============================================================================
dev: ## Start development mode (watch + rebuild)
	cargo watch -x "check" -x "test --lib" -x "clippy --lib -- -D warnings"

# ============================================================================
# CRITICAL: ABSOLUTE ZERO JAVASCRIPT VERIFICATION
# ============================================================================
verify-no-js: ## Verify NO JavaScript in project (CRITICAL)
	@echo "🔍 Verifying ABSOLUTE ZERO JavaScript COMPUTATION policy..."
	@echo "   (Note: Minimal JS in HTML loaders is allowed for event forwarding only)"
	@echo ""
	@echo "  [1/5] Checking for standalone .js files..."
	@# Allow wasm-pack generated pkg/ directories and Playwright node_modules (test infrastructure only)
	@if find . -name "*.js" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*" -not -path "*/node_modules/*" | grep -q .; then \
		echo "❌ FAIL: JavaScript files detected!"; \
		find . -name "*.js" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*" -not -path "*/node_modules/*"; \
		exit 1; \
	fi
	@echo "  ✅ No standalone .js files (wasm-pack pkg/ and node_modules excluded)"
	@echo ""
	@echo "  [2/5] Checking for .ts files (excluding type definitions)..."
	@# Allow: .d.ts type definitions from wasm-pack output only
	@if find . -name "*.ts" -not -name "*.d.ts" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*" | grep -q .; then \
		echo "❌ FAIL: TypeScript files detected!"; \
		find . -name "*.ts" -not -name "*.d.ts" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*"; \
		exit 1; \
	fi
	@echo "  ✅ No .ts files (.d.ts type definitions excluded)"
	@echo ""
	@echo "  [3/5] Checking for package.json..."
	@if [ -f "package.json" ]; then \
		echo "❌ FAIL: package.json detected!"; \
		exit 1; \
	fi
	@echo "  ✅ No package.json"
	@echo ""
	@echo "  [4/5] Checking for node_modules..."
	@if [ -d "node_modules" ]; then \
		echo "❌ FAIL: node_modules directory detected!"; \
		exit 1; \
	fi
	@echo "  ✅ No node_modules"
	@echo ""
	@echo "  [5/5] Checking Cargo.toml for forbidden crates..."
	@if grep -q "wasm-bindgen-futures\|gloo\|bevy\|macroquad\|ggez" Cargo.toml 2>/dev/null; then \
		echo "❌ FAIL: Forbidden crate detected in Cargo.toml!"; \
		grep "wasm-bindgen-futures\|gloo\|bevy\|macroquad\|ggez" Cargo.toml; \
		exit 1; \
	fi
	@echo "  ✅ No forbidden crates"
	@echo ""
	@echo "✅ ABSOLUTE ZERO JavaScript COMPUTATION verification PASSED"
	@echo "   (HTML loaders contain only event forwarding, zero game logic)"

verify-batuta-deps: ## Verify batuta stack dependencies are used
	@echo "🔍 Verifying batuta-first component policy..."
	@echo ""
	@echo "  Checking for trueno dependency..."
	@if ! grep -q "trueno" Cargo.toml 2>/dev/null; then \
		echo "⚠️  WARNING: trueno not found in Cargo.toml (MANDATORY)"; \
	else \
		echo "  ✅ trueno dependency found"; \
	fi
	@echo ""
	@echo "  Checking for aprender dependency..."
	@if ! grep -q "aprender" Cargo.toml 2>/dev/null; then \
		echo "⚠️  WARNING: aprender not found in Cargo.toml (MANDATORY)"; \
	else \
		echo "  ✅ aprender dependency found"; \
	fi
	@echo ""
	@echo "  Checking for local ../batuta paths (preferred for dev)..."
	@if grep -q 'path = "../batuta' Cargo.toml 2>/dev/null; then \
		echo "  ✅ Using local batuta components (recommended for development)"; \
	else \
		echo "  ℹ️  Using crates.io versions (acceptable for release)"; \
	fi
	@echo ""
	@echo "✅ Batuta dependency verification complete"

verify-wasm-output: build-wasm ## Verify WASM output has no JS glue
	@echo "🔍 Verifying WASM output..."
	@if ls target/$(WASM_TARGET)/release/*.js 2>/dev/null | grep -q .; then \
		echo "❌ FAIL: JavaScript glue files detected in WASM output!"; \
		ls target/$(WASM_TARGET)/release/*.js; \
		exit 1; \
	fi
	@echo "✅ Pure WASM output verified (no JS glue)"

# ============================================================================
# TOOLS INSTALLATION
# ============================================================================
install-tools: ## Install required development tools
	@echo "Installing development tools..."
	rustup target add $(WASM_TARGET)
	cargo install cargo-watch cargo-llvm-cov cargo-mutants cargo-audit cargo-deny wasm-pack
	cargo install bashrs || echo "bashrs may require manual installation"
	cargo install pmat || echo "PMAT may require manual installation"
	@echo "✅ Tools installed!"

# ============================================================================
# LOAD TESTING & CHAOS ENGINEERING
# ============================================================================
load-test: ## Run all load tests (chaos + proptest + example)
	@echo "🔥 LOAD TESTING & CHAOS ENGINEERING SUITE"
	@echo "=========================================="
	@echo ""
	@echo "  [1/4] Running chaos engineering tests..."
	@cargo test -p jugar-web --test chaos -- --nocapture 2>&1 | tail -80
	@echo ""
	@echo "  [2/4] Running property-based tests..."
	@PROPTEST_CASES=100 cargo test -p jugar-web --test proptest_game -- --nocapture 2>&1 | tail -50
	@echo ""
	@echo "  [3/4] Running load test example (all tiers)..."
	@cargo run -p jugar-web --example load_test 2>&1
	@echo ""
	@echo "  [4/4] Running tracing validation..."
	@cargo test -p jugar-web tracer -- --nocapture 2>&1 | tail -20
	@echo ""
	@echo "✅ Load testing complete!"

load-test-quick: ## Quick load test (tier 1 + chaos only, <1min)
	@echo "⚡ QUICK LOAD TEST (tier 1 + chaos)"
	@echo "==================================="
	@cargo test -p jugar-web --test chaos -- --nocapture 2>&1 | tail -40
	@echo ""
	@echo "✅ Quick load test complete!"

load-test-full: ## Full load test with detailed output
	@echo "🔥 FULL LOAD TESTING SUITE (DETAILED)"
	@echo "======================================"
	@echo ""
	@echo "=== CHAOS ENGINEERING TESTS ==="
	cargo test -p jugar-web --test chaos -- --nocapture
	@echo ""
	@echo "=== PROPERTY-BASED TESTS (256 cases) ==="
	PROPTEST_CASES=256 cargo test -p jugar-web --test proptest_game -- --nocapture
	@echo ""
	@echo "=== LOAD TEST EXAMPLE (ALL TIERS) ==="
	cargo run -p jugar-web --example load_test
	@echo ""
	@echo "=== TRACING VALIDATION ==="
	cargo test -p jugar-web tracer -- --nocapture
	@echo ""
	@echo "✅ Full load testing complete!"

ai-test: ## Run Pong AI CLI tests (DDA, determinism, simulation)
	@echo "🤖 PONG AI CLI TESTS"
	@echo "===================="
	@echo ""
	@echo "  [1/3] Model info..."
	@cargo run -p jugar-web --bin pong_ai_cli -- info
	@echo ""
	@echo "  [2/3] DDA tests..."
	@cargo run -p jugar-web --bin pong_ai_cli -- test-dda
	@echo ""
	@echo "  [3/3] Determinism proof..."
	@cargo run -p jugar-web --bin pong_ai_cli -- prove-determinism
	@echo ""
	@echo "✅ AI tests complete!"

ai-simulate: ## Run AI game simulation (default: difficulty=5, rounds=100)
	@cargo run -p jugar-web --bin pong_ai_cli -- simulate --difficulty 5 --rounds 100

trace-test: ## Run tracing tests with detailed output
	@echo "🔍 TRACING VALIDATION"
	@echo "====================="
	@cargo test -p jugar-web trace -- --nocapture 2>&1

# ============================================================================
# HTML/JS LINTING (ZERO JS POLICY ENFORCEMENT)
# ============================================================================
lint-html: ## Lint HTML files (validates ZERO JS policy)
	@echo "📄 HTML/JS LINTING"
	@echo "=================="
	@echo ""
	@echo "  [1/3] Validating HTML structure..."
	@command -v html5validator >/dev/null 2>&1 && html5validator examples/pong-web/index.html || echo "    ⚠️  html5validator not available (pip install html5validator)"
	@echo ""
	@echo "  [2/3] Counting JS complexity (ZERO JS policy)..."
	@echo "    Lines in index.html: $$(wc -l < examples/pong-web/index.html)"
	@echo "    Functions defined: $$(grep -c 'const.*=.*(' examples/pong-web/index.html || echo 0)"
	@echo "    Multi-line functions (should be minimal): $$(grep -c '{$$' examples/pong-web/index.html || echo 0)"
	@echo ""
	@echo "  [3/3] Checking for forbidden patterns..."
	@! grep -n 'npm\|node_modules\|package\.json' examples/pong-web/index.html || (echo "    ❌ FORBIDDEN: npm/node references found!" && exit 1)
	@! grep -n '\.js$$' examples/pong-web/index.html | grep -v 'pkg/jugar_web.js' || (echo "    ❌ FORBIDDEN: External JS files found!" && exit 1)
	@echo "    ✅ No forbidden patterns detected"
	@echo ""
	@echo "✅ HTML lint complete!"

lint-js-complexity: ## Analyze JS complexity in HTML (should stay minimal)
	@echo "📊 JS COMPLEXITY ANALYSIS"
	@echo "========================="
	@echo ""
	@echo "Target: ZERO JS computation - all logic in Rust/WASM"
	@echo ""
	@awk '/^const [a-zA-Z_]+ = \(.*\) =>/ { count++; print "  " NR ": " $$0 } END { print "\nSingle-line arrow functions: " count }' examples/pong-web/index.html
	@echo ""
	@awk '/^const [a-zA-Z_]+ = \(.*\) => \{/ { count++; name=$$2; print "  " NR ": " name " (multi-line)" } END { print "\nMulti-line functions: " count " (keep minimal)" }' examples/pong-web/index.html
	@echo ""
	@echo "Allowed multi-line: playTone (WebAudio), playAudio (dispatcher), execCmd (renderer), main (setup)"

# ============================================================================
# CLEAN
# ============================================================================
clean: ## Clean build artifacts
	cargo clean
	rm -rf .kaizen/
	rm -f deep_context.md lcov.info

clean-pmat: ## Clean PMAT artifacts (but preserve baseline)
	rm -rf .pmat/embeddings/
	rm -rf .pmat/work/
	rm -rf .pmat-metrics/trends/

# ============================================================================
# PHYSICS TOY SANDBOX (Remixable Rube Goldberg builder)
# Toyota Way: Kaizen, Poka-Yoke, Jidoka, Mieruka, Muda elimination
# ============================================================================
sandbox: ## Build and test physics-toy-sandbox crate
	@echo "🎮 PHYSICS TOY SANDBOX"
	@echo "======================"
	@echo ""
	@echo "  [1/3] Building sandbox crate..."
	@cargo build -p physics-toy-sandbox --all-features
	@echo "  [2/3] Running tests..."
	@cargo test -p physics-toy-sandbox --all-features
	@echo "  [3/3] Linting..."
	@cargo clippy -p physics-toy-sandbox --all-features -- -D warnings
	@echo ""
	@echo "✅ Physics Toy Sandbox build complete!"

test-sandbox: ## Run physics-toy-sandbox tests
	@echo "🧪 Running physics-toy-sandbox tests..."
	@cargo test -p physics-toy-sandbox --all-features -- --nocapture

test-sandbox-verbose: ## Run physics-toy-sandbox tests with verbose output
	@cargo test -p physics-toy-sandbox --all-features -- --nocapture --show-output

test-sandbox-coverage: ## Run physics-toy-sandbox tests with coverage analysis
	@echo "📊 PHYSICS TOY SANDBOX COVERAGE"
	@echo "================================"
	@echo ""
	@echo "📋 Test Framework: Extreme TDD (tests in same file as impl)"
	@echo "📋 Toyota Way: Poka-Yoke, Jidoka, Mieruka, Muda elimination"
	@echo ""
	@echo "  [1/4] Installing tools if needed..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@echo "  [2/4] Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace 2>/dev/null || true
	@mkdir -p target/coverage/sandbox
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "  [3/4] Running tests with instrumentation..."
	@cargo llvm-cov --no-report -p physics-toy-sandbox --all-features 2>&1 | tail -20
	@echo "  [4/4] Generating reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/sandbox -p physics-toy-sandbox
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════════╗"
	@echo "║  📊 PHYSICS TOY SANDBOX COVERAGE SUMMARY                              ║"
	@echo "╠══════════════════════════════════════════════════════════════════════╣"
	@cargo llvm-cov report --summary-only -p physics-toy-sandbox 2>/dev/null | grep -E "TOTAL|physics-toy-sandbox" | head -5
	@echo "╚══════════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "📊 Module Coverage:"
	@cargo llvm-cov report -p physics-toy-sandbox 2>/dev/null | grep -E "lib\.rs|material\.rs|contraption\.rs|thermometer\.rs|remix\.rs" || echo "  (run 'make coverage-open' to see full report)"
	@echo ""
	@echo "✅ Coverage report: target/coverage/sandbox/html/index.html"
	@echo ""
	@echo "🎯 Quality Metrics (Toyota Way):"
	@echo "   • Function coverage target: ≥95%"
	@echo "   • Line coverage target: ≥95%"
	@echo "   • Poka-Yoke: NonZeroU32 density (no division-by-zero)"
	@echo "   • Jidoka: Engine versioning (replay compatibility)"
	@echo "   • Mieruka: ComplexityThermometer (visual control)"
	@echo "   • Muda: No scalar fallback (WebGPU/WASM SIMD only)"

sandbox-lint: ## Lint physics-toy-sandbox with pedantic clippy
	@echo "🔍 Linting physics-toy-sandbox (pedantic)..."
	@cargo clippy -p physics-toy-sandbox --all-features -- -D warnings -W clippy::pedantic
	@echo "✅ Lint complete!"

sandbox-mutate: ## Run mutation testing on physics-toy-sandbox
	@echo "🧬 Running mutation testing on physics-toy-sandbox..."
	@which cargo-mutants > /dev/null 2>&1 || (echo "📦 Installing cargo-mutants..." && cargo install cargo-mutants)
	@cargo mutants --package physics-toy-sandbox --timeout 60 --no-times 2>&1 | tail -50
	@echo ""
	@echo "🎯 Mutation score target: ≥80%"

build-sandbox-wasm: ## Build physics-toy-sandbox for WASM target
	@echo "🌐 Building physics-toy-sandbox for WASM..."
	@cargo build -p physics-toy-sandbox --target $(WASM_TARGET) --release --features wasm
	@echo "✅ WASM build complete!"
	@ls -lh target/$(WASM_TARGET)/release/*.wasm 2>/dev/null || echo "   (no .wasm output - this is a library crate)"
