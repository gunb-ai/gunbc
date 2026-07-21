#!/usr/bin/env bash
# Discriminating green-by-execution witness for the Gate-1 emit import-closure
# derivation (PR #6960 / emit_import_closure_root).
#
# WHAT IT PROVES (non-vacuous, DESIGN 5): a namespace module (ZERO `import`
# statements, v2 corpus shape) that references cross-module names emits VALID,
# cargo-COMPILING Rust WITH the reference-derived use-line pass in
# 05_emit_rust.dag, and FAILS to compile WITHOUT it (the exact regression:
# unresolved names -> E0425/E0433). The RED half is confirmed at dev time with a
# pass-disabled emitter binary (you cannot commit "the emitter without the fix"
# as a floor test), so this script stages BOTH binaries.
#
# USAGE:
#   scripts/namespace_import_closure_witness.sh <WITH_PASS_GUNBC> <NO_PASS_GUNBC>
#
# where WITH_PASS_GUNBC is a gunbc built from this branch's 05_emit_rust.dag
# (the reference_derived_use_lines pass live) and NO_PASS_GUNBC is a gunbc built
# from origin/main's v1_compiler_emit_rust.rs (no pass). Build the control with:
#   git show origin/main:src/v1/stage0/src/v1_compiler_emit_rust.rs \
#     > src/v1/stage0/src/v1_compiler_emit_rust.rs   # swap
#   cargo build --release -p v1-compiler --bin gunbc # -> no-pass gunbc
#   git checkout -- src/v1/stage0/src/v1_compiler_emit_rust.rs   # restore
#
# EXIT 0 iff: with-pass closure cargo-builds AND no-pass closure does NOT.
set -uo pipefail

WITH_PASS="${1:?with-pass gunbc path}"
NO_PASS="${2:?no-pass gunbc path}"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# --- The fixture: a repr-clean (no String/Symbol, no generic-in-signature)
#     provider + a namespace CONSUMER that references it cross-module without
#     importing it. Must live under the workspace root (repo_relative_path). ---
FIX="$ROOT/.witness_namespace_import_closure"
rm -rf "$FIX"; mkdir -p "$FIX/witness/pilot"
cat > "$FIX/witness/pilot/emit_provider.dag" <<'DAG'
module witness.pilot.emit_provider

type PilotColor = PilotRed | PilotGreen | PilotBlue

fn pilot_provider_flag() -> Bool { true }

fn pilot_provider_default() -> PilotColor { PilotRed }
DAG
cat > "$FIX/witness/pilot/emit_consumer.dag" <<'DAG'
module witness.pilot.emit_consumer

fn pilot_consumer_flag() -> Bool { pilot_provider_flag() }

fn pilot_consumer_default() -> PilotColor { pilot_provider_default() }
DAG

emit_and_build() {
  local bin="$1" out="$2"
  rm -rf "$out"; mkdir -p "$out"
  "$bin" compile --source-root "$FIX" \
    --entry "$FIX/witness/pilot/emit_consumer.dag" \
    --output-dir "$out" --target rust --dependency-pool-index primary-precedence \
    >/dev/null 2>&1
  ( cd "$out" && RUSTC_WRAPPER="" cargo build 2>&1 | grep -qE 'error\[E[0-9]+\]' )
  # returns 0 (found errors -> RED) / 1 (no errors -> GREEN)
}

TMP="$(mktemp -d)"
echo "== WITH pass ($WITH_PASS): expect cargo GREEN =="
if emit_and_build "$WITH_PASS" "$TMP/with"; then
  echo "FAIL: with-pass closure did NOT compile (found rustc errors)"; WITH_RED=1
else
  echo "OK: with-pass namespace closure cargo-compiles"; WITH_RED=0
fi

echo "== WITHOUT pass ($NO_PASS): expect cargo RED (E0425/E0433 unresolved refs) =="
if emit_and_build "$NO_PASS" "$TMP/without"; then
  echo "OK: no-pass closure FAILS to compile (regression reproduced)"; WITHOUT_RED=1
else
  echo "FAIL: no-pass closure compiled — witness is VACUOUS"; WITHOUT_RED=0
fi

rm -rf "$FIX" "$TMP"
if [ "$WITH_RED" = 0 ] && [ "$WITHOUT_RED" = 1 ]; then
  echo "WITNESS PASS: greens-with / reds-without (non-vacuous)"; exit 0
else
  echo "WITNESS FAIL"; exit 1
fi
