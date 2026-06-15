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
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
BIN=target/release/gunbc
[ -x "$BIN" ] || { echo "FAIL: build $BIN first (cargo build --release -p v2-compiler --bin gunbc)"; exit 2; }

fail() { echo "GATE FAIL: $1"; exit 1; }

echo "== POSITIVE: carrier + witness compile + cargo check =="
"$BIN" compile --source-root dsl/examples/time_affine_check --source-root dsl/std \
  --output-dir /tmp/ta-pos --target rust 2>&1 | tail -1
cargo check --manifest-path /tmp/ta-pos/Cargo.toml >/tmp/ta-pos-check.log 2>&1 \
  || fail "positive consumer did not cargo-check (see /tmp/ta-pos-check.log)"
echo "  OK: positive gate green"

echo "== NEGATIVE: instant + instant must be rejected =="
"$BIN" compile --source-root dsl/examples/time_affine_reject --source-root dsl/std \
  --output-dir /tmp/ta-neg --target rust 2>&1 | tail -1
if cargo check --manifest-path /tmp/ta-neg/Cargo.toml >/tmp/ta-neg-check.log 2>&1; then
  fail "instant + instant COMPILED — the affine law has regressed (Instant gained Add)"
fi
grep -q "cannot add" /tmp/ta-neg-check.log \
  || fail "negative gate failed for the wrong reason (expected E0369 'cannot add'; see /tmp/ta-neg-check.log)"
echo "  OK: instant + instant rejected ($(grep -m1 'cannot add' /tmp/ta-neg-check.log | sed 's/^ *//'))"

echo "GATE PASS"
