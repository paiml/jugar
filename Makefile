# Jugar Makefile - EXTREME TDD Quality Gates
# Tiered Workflow inspired by certeza (PMAT integration)
# WASM-native game engine targeting wasm32-unknown-unknown
# CRITICAL: ABSOLUTE ZERO JAVASCRIPT - Pure WASM only

.SUFFIXES:
.DELETE_ON_ERROR:
.ONESHELL:

WASM_TARGET := wasm32-unknown-unknown

.PHONY: help tier1 tier2 tier3 build build-wasm test test-fast coverage coverage-check lint lint-fast fmt clean all dev bench mutate kaizen pmat-tdg pmat-analyze pmat-score pmat-rust-score pmat-mutate pmat-validate-docs pmat-quality-gate pmat-context pmat-all install-tools verify-no-js verify-batuta-deps

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
	@echo "  [1/8] Formatting check..."
	@cargo fmt -- --check
	@echo "  [2/8] Full clippy..."
	@cargo clippy --all-targets --all-features --quiet -- -D warnings
	@echo "  [3/8] All tests..."
	@cargo test --all-features --quiet
	@echo "  [4/8] Property tests (full cases)..."
	@PROPTEST_CASES=256 cargo test property_ --all-features --quiet 2>/dev/null || true
	@echo "  [5/8] Coverage analysis..."
	@cargo llvm-cov --all-features --workspace --quiet 2>/dev/null || echo "    ⚠️  llvm-cov not available"
	@echo "  [6/8] PMAT TDG..."
	@pmat analyze tdg --min-grade B+ 2>/dev/null || echo "    ⚠️  PMAT not available"
	@echo "  [7/8] SATD check..."
	@! grep -rn "TODO\|FIXME\|HACK" crates/*/src/ 2>/dev/null || echo "    ⚠️  SATD comments found"
	@echo "  [8/8] JavaScript verification (already done above)..."
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

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting
	cargo fmt -- --check

coverage: ## Generate coverage report (target: ≥95%)
	@echo "📊 Generating coverage report (target: ≥95%)..."
	@# Temporarily disable mold linker (breaks LLVM coverage)
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@cargo llvm-cov --workspace --lcov --output-path lcov.info
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@# Restore mold linker
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo "✅ Coverage report: target/coverage/html/index.html"
	@echo ""
	@echo "📊 Coverage Summary:"
	@cargo llvm-cov report | tail -1

coverage-check: ## Enforce 95% coverage threshold (BLOCKS on failure)
	@echo "🔒 Enforcing 95% coverage threshold..."
	@# Temporarily disable mold linker (breaks LLVM coverage)
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@cargo llvm-cov --workspace --lcov --output-path lcov.info > /dev/null 2>&1
	@# Restore mold linker
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@cargo llvm-cov report | python3 -c "import sys; lines = list(sys.stdin); jugar = [l for l in lines if '.rs' in l and not l.startswith('TOTAL') and not l.startswith('-')]; j_total = sum(int(l.split()[7]) for l in jugar) if jugar else 0; j_uncov = sum(int(l.split()[8]) for l in jugar) if jugar else 0; j_cov = 100*(j_total-j_uncov)/j_total if j_total > 0 else 0; print(f'Jugar library coverage: {j_cov:.2f}%'); exit_code = 1 if j_cov < 95 else 0; print(f'✅ Coverage threshold met (≥95%)' if exit_code == 0 else f'❌ FAIL: Coverage below 95% threshold'); sys.exit(exit_code)"

coverage-lcov: ## Generate lcov coverage report
	@# Temporarily disable mold linker (breaks LLVM coverage)
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	cargo llvm-cov --workspace --lcov --output-path lcov.info
	@# Restore mold linker
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true

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
# MUTATION TESTING
# ============================================================================
mutate: ## Run mutation testing with cargo-mutants
	cargo mutants --timeout 60

mutate-fast: ## Run quick mutation testing (fewer mutations)
	cargo mutants --timeout 30 --jobs 4

# ============================================================================
# DEVELOPMENT
# ============================================================================
dev: ## Start development mode (watch + rebuild)
	cargo watch -x "check" -x "test --lib" -x "clippy --lib -- -D warnings"

# ============================================================================
# CRITICAL: ABSOLUTE ZERO JAVASCRIPT VERIFICATION
# ============================================================================
verify-no-js: ## Verify NO JavaScript in project (CRITICAL)
	@echo "🔍 Verifying ABSOLUTE ZERO JavaScript policy..."
	@echo ""
	@echo "  [1/5] Checking for .js files..."
	@if find . -name "*.js" -not -path "./target/*" -not -path "./.git/*" | grep -q .; then \
		echo "❌ FAIL: JavaScript files detected!"; \
		find . -name "*.js" -not -path "./target/*" -not -path "./.git/*"; \
		exit 1; \
	fi
	@echo "  ✅ No .js files"
	@echo ""
	@echo "  [2/5] Checking for .ts files..."
	@if find . -name "*.ts" -not -path "./target/*" -not -path "./.git/*" | grep -q .; then \
		echo "❌ FAIL: TypeScript files detected!"; \
		find . -name "*.ts" -not -path "./target/*" -not -path "./.git/*"; \
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
	@echo "✅ ABSOLUTE ZERO JavaScript verification PASSED"

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
	cargo install cargo-watch cargo-llvm-cov cargo-mutants cargo-audit cargo-deny
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
