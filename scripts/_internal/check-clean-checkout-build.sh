#!/usr/bin/env bash
# scripts/_internal/check-clean-checkout-build.sh
#
# Clean-checkout build gate for the v0.1.0-rc.0 release rehearsal
# (RELEASE_TODO.md §0). Rehearses the experience a first-time user has on a
# fresh machine: clone main, build the binary, run --help, compile the public
# hero demo, and `cargo check` the emitted Rust.
#
# Lives under scripts/_internal/ because it exists to vet the *release path*
# from inside the internal repo. The publish-snapshot strip list excludes
# scripts/_internal/, so this never reaches the public repo.
#
# Defaults can be overridden via env:
#   GUNBC_REPO_URL  — repo to clone (default: git@github.com:gunb-ai/gunbc.git)
#   GUNBC_REF       — branch/tag/sha to check out (default: main)
#   WORK_DIR        — scratch directory (default: a fresh mktemp -d)
#   KEEP_WORK_DIR   — set to 1 to keep WORK_DIR after success
#
# Exit codes:
#   0   all steps green
#   1   any step failed; WORK_DIR is preserved for inspection
#   2   bad invocation
#
# Usage:
#   scripts/_internal/check-clean-checkout-build.sh
#   GUNBC_REF=v0.1.0-rc.0 scripts/_internal/check-clean-checkout-build.sh
set -euo pipefail

REPO_URL="${GUNBC_REPO_URL:-git@github.com:gunb-ai/gunbc.git}"
REF="${GUNBC_REF:-main}"
KEEP_WORK_DIR="${KEEP_WORK_DIR:-0}"

if [[ $# -gt 0 ]]; then
  echo "error: this script takes no positional arguments (configure via env)" >&2
  echo "  see header for GUNBC_REPO_URL / GUNBC_REF / WORK_DIR / KEEP_WORK_DIR" >&2
  exit 2
fi

if [[ -n "${WORK_DIR:-}" ]]; then
  mkdir -p "$WORK_DIR"
else
  WORK_DIR="$(mktemp -d -t gunbc-clean-checkout-XXXXXX)"
fi

CLONE_DIR="$WORK_DIR/gunbc"
EMIT_DIR="$WORK_DIR/weather-out"

cleanup_on_success() {
  if [[ "$KEEP_WORK_DIR" == "1" ]]; then
    echo "WORK_DIR preserved at: $WORK_DIR (KEEP_WORK_DIR=1)"
  else
    rm -rf "$WORK_DIR"
  fi
}

preserve_on_fail() {
  local rc=$?
  if [[ $rc -ne 0 ]]; then
    echo
    echo "FAIL (exit $rc). WORK_DIR preserved for inspection: $WORK_DIR" >&2
  fi
  exit $rc
}
trap preserve_on_fail EXIT

step() {
  echo
  echo "=== [$(date -u +%H:%M:%S)] $* ==="
}

step "Clone $REPO_URL @ $REF (depth=1) into $CLONE_DIR"
git clone --depth=1 --branch "$REF" --single-branch "$REPO_URL" "$CLONE_DIR"

cd "$CLONE_DIR"

step "cargo build --release -p v2-compiler --bin gunbc"
cargo build --release -p v2-compiler --bin gunbc

GUNBC="$CLONE_DIR/target/release/gunbc"
if [[ ! -x "$GUNBC" ]]; then
  echo "error: expected binary at $GUNBC after cargo build, not found" >&2
  exit 1
fi

step "gunbc --help (sanity: binary runs and prints usage)"
"$GUNBC" --help

step "gunbc compile dsl/examples/weather/weather.dag --target rust"
mkdir -p "$EMIT_DIR"
"$GUNBC" compile \
  --source-root dsl/examples/weather \
  --source-root dsl/std \
  --output-dir "$EMIT_DIR" \
  --target rust

step "cargo check on emitted Rust at $EMIT_DIR"
if [[ ! -f "$EMIT_DIR/Cargo.toml" ]]; then
  echo "error: gunbc compile did not produce $EMIT_DIR/Cargo.toml" >&2
  exit 1
fi
cargo check --manifest-path "$EMIT_DIR/Cargo.toml"

echo
echo "=== clean-checkout build gate: PASS ==="
echo "  repo:    $REPO_URL @ $REF"
echo "  binary:  $GUNBC"
echo "  emitted: $EMIT_DIR"

trap - EXIT
cleanup_on_success
