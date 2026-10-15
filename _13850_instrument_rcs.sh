#!/usr/bin/env bash
# Pin-checked instrument rcs. Refuse if HEAD is not EXPECTED_PIN (default: this tree's HEAD at script start).
set -euo pipefail
REPO="${REPO:-$PWD}"
cd "$REPO"
PIN="$(git rev-parse HEAD)"
EXPECTED_PIN="${EXPECTED_PIN:-$PIN}"
if [ "$PIN" != "$EXPECTED_PIN" ]; then
  echo "INSTRUMENT_REFUSED pin_mismatch HEAD=$PIN EXPECTED=$EXPECTED_PIN" >&2
  exit 2
fi
echo "INSTRUMENT_PIN_SHA=$PIN"
export GITHUB_SHA="$PIN"
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
export TMPDIR="${TMPDIR:-/tmp}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO/target-private-$PIN}"
mkdir -p "$CARGO_TARGET_DIR"
unset GUNBC_BIND_MEMORY_CGROUP_BYTES || true
unset GUNBC_MEMORY_ENVELOPE_REQUIRED || true
BOUND=$((22 * 1024 * 1024 * 1024))
LEAF=/sys/fs/cgroup/rb157.instruments.bound
mkdir -p "$LEAF"
echo '+memory' >/sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
printf '%s\n' "$BOUND" >"$LEAF/memory.max" || true
printf '%s\n' "$$" >"$LEAF/cgroup.procs" || true

cargo build --release -p v1-compiler --bin gunbc
GUNBC="$CARGO_TARGET_DIR/release/gunbc"
test -x "$GUNBC"
SEED_SHA="$(sha256sum "$GUNBC" | awk '{print $1}')"
echo "INSTRUMENT_SEED_BIN path=$GUNBC sha256=$SEED_SHA"
mkdir -p "$REPO/target/instrument-rcs"

run_one() {
  local label="$1"
  local log="$REPO/target/instrument-rcs/${label##*:}.log"
  echo "INSTRUMENT_BEGIN label=$label"
  set +e
  "$GUNBC" test "$label" 2>&1 | tee "$log"
  local rc=${PIPESTATUS[0]}
  set -e
  echo "INSTRUMENT_RC label=$label rc=$rc"
  return 0
}

run_one "//gunbc/instruments:self-host"
run_one "//gunbc/instruments:v2-native-frontier"
run_one "//gunbc/instruments:type-declaration-use-census"
echo "INSTRUMENT_DONE pin=$PIN seed_sha256=$SEED_SHA"
