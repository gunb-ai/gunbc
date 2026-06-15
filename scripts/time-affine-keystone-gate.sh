#!/usr/bin/env bash
# Keystone gate for std.time_affine (affine-instant time carrier).
#
# Proves BOTH directions of the affine law, so the model can't pass while the
# law silently regresses:
#   POSITIVE: the carrier + witness consumer compile AND cargo-check clean.
#   NEGATIVE: `instant + instant` is REJECTED (cannot add Instant to Instant).
#
# Run from the gunbc repo root after:
#   cargo build --release -p v2-compiler --bin gunbc
#
# Env knobs (match scripts/v4-nat-semiring-rung-gate.sh so it works on the
# self-hosted runner): output goes under $RUNNER_TEMP (co-located with the
# runner's warm CARGO_HOME / registry config) so `cargo check` of the emitted
# crate resolves OFFLINE from the cache the v2-compiler build populated. A
# hardcoded /tmp output sits outside the runner's cargo config and fails to
# resolve deps under the CI egress quarantine.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
BIN="${V2_COMPILER:-target/release/gunbc}"
CARGO="${CARGO_BIN:-cargo}"
OUT="${RUNNER_TEMP:-/tmp}/time-affine-gate"
[ -x "$BIN" ] || { echo "FAIL: build $BIN first (cargo build --release -p v2-compiler --bin gunbc)"; exit 2; }
mkdir -p "$OUT"

fail() { echo "GATE FAIL: $1"; exit 1; }

echo "== POSITIVE: carrier + witness compile + cargo check =="
"$BIN" compile --source-root dsl/examples/time_affine_check --source-root dsl/std \
  --output-dir "$OUT/pos" --target rust 2>&1 | tail -1
"$CARGO" check --manifest-path "$OUT/pos/Cargo.toml" >"$OUT/pos-check.log" 2>&1 \
  || { echo "--- pos-check.log tail ---"; tail -20 "$OUT/pos-check.log"; fail "positive consumer did not cargo-check"; }
echo "  OK: positive gate green"

echo "== NEGATIVE: instant + instant must be rejected =="
"$BIN" compile --source-root dsl/examples/time_affine_reject --source-root dsl/std \
  --output-dir "$OUT/neg" --target rust 2>&1 | tail -1
if "$CARGO" check --manifest-path "$OUT/neg/Cargo.toml" >"$OUT/neg-check.log" 2>&1; then
  fail "instant + instant COMPILED — the affine law has regressed (Instant gained Add)"
fi
grep -q "cannot add" "$OUT/neg-check.log" \
  || { echo "--- neg-check.log tail ---"; tail -20 "$OUT/neg-check.log"; fail "negative gate failed for the wrong reason (expected E0369 'cannot add')"; }
echo "  OK: instant + instant rejected ($(grep -m1 'cannot add' "$OUT/neg-check.log" | sed 's/^ *//'))"

echo "GATE PASS"
