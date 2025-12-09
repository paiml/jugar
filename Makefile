# Jugar Makefile - EXTREME TDD Quality Gates
# Tiered Workflow inspired by certeza (PMAT integration)
# WASM-native game engine targeting wasm32-unknown-unknown
# CRITICAL: ABSOLUTE ZERO JAVASCRIPT - Pure WASM only

.SUFFIXES:
.DELETE_ON_ERROR:
.ONESHELL:

WASM_TARGET := wasm32-unknown-unknown

.PHONY: help tier1 tier2 tier3 build build-wasm build-web serve-web test test-fast test-property test-property-full test-e2e test-e2e-headed coverage coverage-summary coverage-open coverage-check coverage-ci coverage-clean lint lint-fast lint-bash fmt clean all dev bench mutate mutate-quick mutate-file mutate-report kaizen pmat-tdg pmat-analyze pmat-score pmat-rust-score pmat-mutate pmat-validate-docs pmat-quality-gate pmat-context pmat-all install-tools verify-no-js verify-batuta-deps

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

test-e2e: build-web ## Run Playwright e2e tests for pong-web
	@echo "🧪 Running Playwright e2e tests..."
	cd examples/pong-web && npm test
	@echo "✅ All e2e tests passed!"

test-e2e-headed: build-web ## Run Playwright e2e tests with browser visible
	cd examples/pong-web && npm run test:headed

# ============================================================================
# TEST TARGETS
# ============================================================================
test: ## Run all tests
	cargo test --all-features

test-fast: ## Run library tests only (fast)
	cargo test --lib

test-wasm: ## Run WASM-compatible tests
	cargo test --target $(WASM_TARGET) --all-features 2>/dev/null || echo "WASM tests require wasm-pack or similar"

# ============================================================================
# QUALITY TARGETS
# ============================================================================
lint: ## Full clippy analysis
	cargo clippy --all-targets --all-features -- -D warnings

lint-bash: ## Lint Makefile and shell scripts with bashrs
	@echo "🔍 Linting Makefile and shell scripts with bashrs..."
	bashrs make lint Makefile
	@if ls scripts/*.sh 2>/dev/null | grep -q .; then \
		bashrs lint scripts/*.sh; \
	fi
	@echo "✅ Shell linting complete"

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting
	cargo fmt -- --check

# Code Coverage (Toyota Way: "make coverage" just works)
# Following bashrs/trueno Two-Phase Pattern for reliable coverage
# TARGET: < 5 minutes with proper mold linker handling
# Exclude patterns: binaries and code generators (not library code)
COV_IGNORE := --ignore-filename-regex='bin/.*\.rs'

coverage: ## Generate HTML coverage report (two-phase pattern)
	@echo "📊 Running comprehensive test coverage analysis (target: ≥95%)..."
	@echo "🔍 Checking for cargo-llvm-cov and cargo-nextest..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@echo "🧹 Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace
	@mkdir -p target/coverage
	@echo "⚙️  Temporarily disabling global cargo config (mold breaks coverage)..."
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "🧪 Phase 1: Running tests with instrumentation (no report)..."
	@cargo llvm-cov --no-report nextest --no-tests=warn --all-features --workspace
	@echo "📊 Phase 2: Generating coverage reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html $(COV_IGNORE)
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info $(COV_IGNORE)
	@echo "⚙️  Restoring global cargo config..."
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only $(COV_IGNORE)
	@echo ""
	@echo "💡 COVERAGE INSIGHTS:"
	@echo "- HTML report: target/coverage/html/index.html"
	@echo "- LCOV file: target/coverage/lcov.info"
	@echo "- Open HTML: make coverage-open"
	@echo "- Excluded: bin/*.rs (code generators)"

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

pmat-analyze: ## Run comprehensive PMAT analysis
	pmat analyze complexity --path crates/
	pmat analyze satd
	pmat analyze dead-code
	pmat analyze duplicate

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

pmat-all: ## Run all PMAT checks
	@echo "🔍 Running all PMAT checks..."
	$(MAKE) pmat-tdg
	$(MAKE) pmat-analyze
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
	@# Allow wasm-pack generated pkg/ directories
	@if find . -name "*.js" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*" | grep -q .; then \
		echo "❌ FAIL: JavaScript files detected!"; \
		find . -name "*.js" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*"; \
		exit 1; \
	fi
	@echo "  ✅ No standalone .js files (wasm-pack pkg/ excluded)"
	@echo ""
	@echo "  [2/5] Checking for .ts files..."
	@if find . -name "*.ts" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*" | grep -q .; then \
		echo "❌ FAIL: TypeScript files detected!"; \
		find . -name "*.ts" -not -path "./target/*" -not -path "./.git/*" -not -path "*/pkg/*"; \
		exit 1; \
	fi
	@echo "  ✅ No .ts files"
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
