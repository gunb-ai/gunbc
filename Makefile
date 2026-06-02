# Curated workspace Makefile.
# Maintained manually while the tool surface is being simplified.
# Naming convention:
#   make <target>      - verify only (CI-safe, fails on issues)
#   make <target>-fix  - auto-fix then verify (for dev)
#
# Dev default:     make test      (ensure generated artifacts, then test)
# Dev workflow:    make test-fix  (fmt/lint fix + ensure generated artifacts, then test)
# CI verification: make verify    (check generated artifacts)

.DEFAULT_GOAL := help

.PHONY: help preflight-fix ensure-codegen build-release-bins lint-upsert codegen build clean testgen testgen-check bootstrap-check verify verify-fix fmt-fix lint-fix test-all test test-xs test-s test-m test-l test-xl test-small test-medium test-large test-extra-large test-integration test-external check clippy fmt fmt-check test-fix check-fix clippy-fix bootstrap bootstrap-dry build-all build-all-dry design design-dry gist gist-dry gist-diff gist-diff-dry gist-recent gist-recent-dry infra infra-dry readme readme-dry workflow workflow-dry ci

# Preflight: auto-fix rustc warnings before running generators
preflight-fix:
	@cargo fix --workspace --all-targets --allow-dirty --allow-staged

# Ensure CLI entrypoints exist (bootstrap-safe)
ensure-codegen:
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-codegen -- codegen

# Build workspace binaries once for direct tool execution
build-release-bins: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo build --workspace --release --bins

# Lint upsert: fix if needed, then verify
lint-upsert: ensure-codegen preflight-fix
	@cargo clippy --all-targets -- -D warnings || (cargo clippy --fix --workspace --allow-dirty --allow-staged -- -D warnings && cargo clippy --all-targets -- -D warnings)

# Generate CLI entrypoints (DAG upsert)
codegen: lint-upsert
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-codegen-dag

# Full build transaction: codegen \u{2192} testgen \u{2192} cargo build
build: codegen testgen
	@RUSTFLAGS="-D warnings" cargo build --all-targets

# Clean build artifacts
clean:
	@cargo clean

# Regenerate tests from DAG structures and MockSpecs
testgen: lint-upsert
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-testgen

# Check if generated tests are stale
testgen-check: lint-upsert
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-testgen

# Check if generated bootstrap artifacts are stale
bootstrap-check: lint-upsert
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-bootstrap

# Verify generated artifacts match their generators
verify: lint-upsert
	@$(MAKE) bootstrap-check
	@$(MAKE) testgen-check

# Ensure generated artifacts are up to date
verify-fix: lint-upsert
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-bootstrap
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-testgen

# fmt-fix: apply formatting (alias for fmt)
fmt-fix:
	@cargo fmt

# lint-fix: auto-fix lint issues where possible
lint-fix: ensure-codegen
	@cargo clippy --fix --workspace --allow-dirty --allow-staged -- -D warnings

# Alias for test-xl (full Fermi budget)
test-all: ensure-codegen
	@$(MAKE) test-xl

help:
	@echo "gunbc tools - workspace Makefile"
	@echo ""
	@echo "Naming convention:"
	@echo "  make <target>      - verify only (CI-safe)"
	@echo "  make <target>-fix  - auto-fix then verify (for dev)"
	@echo ""
	@echo "Build commands:"
	@echo "  preflight-fix  - Preflight: auto-fix rustc warnings before running generators"
	@echo "  ensure-codegen  - Ensure CLI entrypoints exist (bootstrap-safe)"
	@echo "  build-release-bins  - Build workspace binaries once for direct tool execution"
	@echo "  lint-upsert  - Lint upsert: fix if needed, then verify"
	@echo "  codegen  - Generate CLI entrypoints (DAG upsert)"
	@echo "  build  - Full build transaction: codegen \u{2192} testgen \u{2192} cargo build"
	@echo "  clean  - Clean build artifacts"
	@echo "  testgen  - Regenerate tests from DAG structures and MockSpecs"
	@echo "  testgen-check  - Check if generated tests are stale"
	@echo "  bootstrap-check  - Check if generated bootstrap artifacts are stale"
	@echo "  verify  - Verify generated artifacts match their generators"
	@echo "  verify-fix  - Ensure generated artifacts are up to date"
	@echo "  fmt-fix  - fmt-fix: apply formatting (alias for fmt)"
	@echo "  lint-fix  - lint-fix: auto-fix lint issues where possible"
	@echo "  test-all  - Alias for test-xl (full Fermi budget)"
	@echo ""
	@echo "Development:"
	@echo "  test  - Alias for test-s (<=S)"
	@echo "  test-fix  - Alias for test-s (<=S) (fmt-fix + lint-fix first)"
	@echo "  test-xs  - Run tests (<=XS)"
	@echo "  test-s  - Run tests (<=S)"
	@echo "  test-m  - Run tests (<=M)"
	@echo "  test-l  - Run tests (<=L)"
	@echo "  test-xl  - Run tests (<=XL)"
	@echo "  test-small  - Alias for test-s"
	@echo "  test-medium  - Alias for test-m"
	@echo "  test-large  - Alias for test-l"
	@echo "  test-extra-large  - Alias for test-xl"
	@echo "  test-integration  - Run integration-focused tests"
	@echo "  test-external  - Run external/live-flow tests"
	@echo "  check  - Type check all targets"
	@echo "  check-fix  - Type check all targets (fmt-fix first)"
	@echo "  clippy  - Run clippy linter"
	@echo "  clippy-fix  - Run clippy linter (auto-fix)"
	@echo "  fmt  - Format all code"
	@echo "  fmt-check  - Format all code (check only)"
	@echo ""
	@echo "Tools:"
	@echo "  bootstrap   - Bootstrap"
	@echo "  build-all   - Build all"
	@echo "  design   - Design"
	@echo "  gist   - Gist (snapshot)"
	@echo "  gist-diff [BASE_REF=...]  - Gist diff"
	@echo "  gist-recent [SINCE=...]  - Gist recent changes"
	@echo "  infra [ENVIRONMENT=...] [RUNTIME=...] [SPEC_TARGETS=... ...] [TARGET=... ...] [SKIP=... ...]  - Infra"
	@echo "  readme   - Readme"
	@echo "  workflow   - Workflow"
	@echo ""
	@echo "  ci  - Run the exact CI pipeline (lint + test, matches .github/workflows/ci.yml)"
	@echo ""
	@echo "Add -dry suffix for dry-run (e.g., make bootstrap-dry)"

# ============================================================================
# Meta Targets - Development workflow commands
# ============================================================================

# test: Alias for test-s (<=S)
test:
	@$(MAKE) test-s
# test-fix: auto-fix then verify
test-fix: fmt-fix lint-fix
	@$(MAKE) test-s

# test-xs: Run tests (<=XS)
test-xs: build verify-fix
	@GUNBC_TEST_MAX_COST=XS RUSTFLAGS="-D warnings" cargo test

# test-s: Run tests (<=S)
test-s: build verify-fix
	@GUNBC_TEST_MAX_COST=S RUSTFLAGS="-D warnings" cargo test

# test-m: Run tests (<=M)
test-m: build verify-fix
	@GUNBC_TEST_MAX_COST=M RUSTFLAGS="-D warnings" cargo test

# test-l: Run tests (<=L)
test-l: build verify-fix
	@GUNBC_TEST_MAX_COST=L RUSTFLAGS="-D warnings" cargo test

# test-xl: Run tests (<=XL)
test-xl: build verify-fix
	@GUNBC_TEST_MAX_COST=XL RUSTFLAGS="-D warnings" cargo test

# test-small: Alias for test-s
test-small:
	@$(MAKE) test-s

# test-medium: Alias for test-m
test-medium:
	@$(MAKE) test-m

# test-large: Alias for test-l
test-large:
	@$(MAKE) test-l

# test-extra-large: Alias for test-xl
test-extra-large:
	@$(MAKE) test-xl

# test-integration: Run integration-focused tests
test-integration: build verify-fix
	@GUNBC_TEST_MAX_COST=XL RUSTFLAGS="-D warnings" cargo test integration

# test-external: Run external/live-flow tests
test-external: build verify-fix
	@GUNBC_TEST_MAX_COST=XL RUSTFLAGS="-D warnings" cargo test live_flow

# check: Type check all targets
check: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo check --all-targets
# check-fix: auto-fix then verify
check-fix: fmt-fix ensure-codegen
	@RUSTFLAGS="-D warnings" cargo check --all-targets

# clippy: Run clippy linter
clippy: ensure-codegen
	@cargo clippy --all-targets -- -D warnings
# clippy-fix: auto-fix then verify
clippy-fix: ensure-codegen
	@cargo clippy --fix --workspace --allow-dirty --allow-staged -- -D warnings

# fmt: Format all code
fmt:
	@cargo fmt
fmt-check:
	@cargo fmt --check

# gunbc-bootstrap entrypoints: 
bootstrap: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-bootstrap -q --release

bootstrap-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-bootstrap -q --release -- --dry-run strict


# gunbc-build-all entrypoints: 
build-all: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-build-all -q --release

build-all-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-build-all -q --release -- --dry-run strict


# gunbc-design entrypoints: 
design: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-design -q --release

design-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-design -q --release -- --dry-run strict


# gunbc-gist subcommands: gist, gist-diff, gist-recent
gist: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist

gist-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist --dry-run strict

gist-diff: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist-diff$(if $(BASE_REF), --base-ref $(BASE_REF))

gist-diff-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist-diff$(if $(BASE_REF), --base-ref $(BASE_REF)) --dry-run strict

gist-recent: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist-recent$(if $(SINCE), --since $(SINCE))

gist-recent-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-gist -q --release -- gist-recent$(if $(SINCE), --since $(SINCE)) --dry-run strict


# gunbc-readme entrypoints:
readme: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-readme -q --release

readme-dry: ensure-codegen
	@RUSTFLAGS="-D warnings" cargo run -p gunbc-codegen --bin gunbc-readme -q --release -- --dry-run strict

# ============================================================================
# CI — mirrors .github/workflows/ci.yml exactly
# ============================================================================

ci:
	RUSTFLAGS="-D warnings" cargo clippy --workspace --exclude gunbc-codegen -- -D warnings
	RUSTFLAGS="-D warnings" cargo clippy -p gunbc-codegen --lib -- -D warnings
	RUSTFLAGS="-D warnings" cargo test --workspace --exclude gunbc-codegen
	@$(MAKE) bootstrap-check
