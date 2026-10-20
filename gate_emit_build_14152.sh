#!/usr/bin/env bash
# GATE INSTRUMENT (stern-bat-401 msg_db49ee15, ruling (2)=(A)): pre-push gate must build the EMITTED
# self-host product, i.e. run what CI runs, not just count type-mismatch diagnostics. A head can pass
# the diagnostic-count gate and still fail CI on emitted Rust (E0277 `S: Clone` at
# stage0/src/std_realization_measurement.rs:341:72 on PR-1 head e5cc5d9).
#
# USAGE: bash gate_emit_build_14152.sh <sha> <label> <repair|name-keyed|main>
# ONE dispatch: fresh clone at the printed SHA, private target dir, release build of gunbc, then
# `gunbc test //gunbc/instruments:self-host`. Prints V2-NATIVE / EmittedCompilerBuildFailed lines and,
# if the emitted crate is retained, the source line CI blamed.
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"; KIND="${3:?kind repair|name-keyed|main}"
[ "$KIND" = "main" ] || { [ "$KIND" = "repair" ] || [ "$KIND" = "name-keyed" ]; } || { echo "VOID bad kind"; exit 7; }
WORK=/tmp/gateemit_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
d=$WORK/repo
echo "T=$(date +%T) clone begin"
git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" 2>/dev/null || { echo "VOID clone failed"; exit 7; }
( cd "$d" && git checkout -q "$SHA" ) || { echo "VOID checkout $SHA"; exit 7; }
echo "T=$(date +%T) MEASURED_AT $(cd $d && git rev-parse HEAD)"
# GUARD: which tree is this, so a .dag-only or wrong-branch tree cannot read as green
dg=$d/src/v1/04_infer.dag
acc=$(grep -c "UnifyGenericsAcc" "$dg" || true)
nm=$(grep -c "type_variable_node(id: g)" "$dg" || true)
sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$dg" || true)
echo "GUARD acc=$acc dag_namekeyed=$nm dag_span_keyed=$sp"
case "$KIND" in
  pin)        { [ "$acc" = 0 ] && [ "$nm" = 0 ] && [ "$sp" = 0 ] && grep -q "MINT-BOUNDARY-INVENTORY" "$d/src/v1/tests/src/call_site_mint_boundary_test.rs" && grep -q "mod call_site_mint_boundary_test;" "$d/src/v1/tests/src/lib.rs"; } || { echo "VOID pin tree guard failed (mint present, or pin row absent)"; exit 8; } ;;
  repair)     { [ "$acc" = 0 ] && [ "$nm" = 1 ] && [ "$sp" = 1 ]; } || { echo "VOID repair tree guard failed (would measure the wrong tree)"; exit 8; } ;;
  name-keyed) { [ "$acc" = 0 ] && [ "$sp" = 0 ] && [ "$nm" -ge 1 ]; } || { echo "VOID name-keyed tree guard failed"; exit 8; } ;;
  main)       { [ "$acc" = 0 ] && [ "$sp" = 0 ]; } || { echo "VOID main tree guard failed (main must carry neither mint)"; exit 8; } ;;
esac
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/tgt}"
mkdir -p "$CARGO_TARGET_DIR"
echo "T=$(date +%T) build gunbc"
( cd "$d" && CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$CARGO_TARGET_DIR/release/gunbc"
echo "build rc=$brc binary=${bin:-NONE} exists=$([ -x "$bin" ] && echo yes || echo no)"
[ $brc -eq 0 ] && [ -x "$bin" ] || { echo "VOID no gunbc"; tail -30 "$WORK/build.log"; exit 6; }
echo "T=$(date +%T) self-host test begin"
( cd "$d" && "$bin" test //gunbc/instruments:self-host ) > "$WORK/self_host.log" 2>&1
rc=$?
echo "SELF_HOST_RC=$rc lines=$(wc -l < "$WORK/self_host.log")"
echo "=== refusal lines ==="
grep -aoE "V2-NATIVE REFUSAL cause=[A-Za-z]+ first_error=[^@]*@ [^ ]+" "$WORK/self_host.log" | sed 's/^[^ ]*@ //' | sort -u | head -12
grep -ac "EmittedCompilerBuildFailed" "$WORK/self_host.log" | sed 's/^/EMITTED_BUILD_FAILED_LINES=/'
grep -aoE "retained_probe_root=[^ ]+" "$WORK/self_host.log" | sort -u | head -3
grep -aoE "error\[E[0-9]+\][^@]*@ [^ ]+" "$WORK/self_host.log" | sort -u | head -12
# If the probe root survived, show the exact emitted line CI blamed.
# Locate the emitted file by glob, NOT by the retained_probe_root= token: that token's value is
# followed by an em dash, and earlier greps cut the path away and silently skipped this section.
pr=$(grep -aoE "retained_probe_root=[^ ]+" "$WORK/self_host.log" | head -1 | cut -d= -f2)
f=""
[ -n "${pr:-}" ] && f=$(find "$pr" -name std_realization_measurement.rs 2>/dev/null | head -1)
if [ -z "$f" ]; then
  f=$(find /tmp -maxdepth 6 -path '*gunbc-emit-compile*' -name std_realization_measurement.rs 2>/dev/null | head -1)
  echo "LOCATED_BY_GLOB=${f:-NONE}"
fi
if [ -n "$f" ] && [ -f "$f" ]; then
  echo "EMITTED_FOUND=$f"
  if [ -n "$f" ]; then
    echo "=== EMITTED $f 335-345 ==="
    sed -n '335,345p' "$f" | cat -n | cut -c1-200
    echo "=== mint-id / unresolved leak in the emitted file ==="
    grep -ac 'S@' "$f" | sed 's/^/S_AT_COUNT=/'
    grep -ac 'UNRESOLVED_TypeVariable' "$f" | sed 's/^/UNRESOLVED_TypeVariable_COUNT=/'
    grep -ac 'UNRESOLVED_CompilerError' "$f" | sed 's/^/UNRESOLVED_CompilerError_COUNT=/'
    echo "EMITTED_SHA256 $(sha256sum "$f" | cut -c1-16)"
    echo "BEGIN_EMITTED_GZ"; gzip -c "$f" | base64 -w 100; echo "END_EMITTED_GZ"
  else
    echo "EMITTED_FILE_ABSENT under ${pr:-NONE}"
  fi
fi
echo "T=$(date +%T) done"
