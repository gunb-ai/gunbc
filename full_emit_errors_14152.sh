#!/usr/bin/env bash
# The emitted-crate build stops at its FIRST error, and the self-host refusal line carries only
# `first_error=`, so a head that carries three distinct emission defects reports one per push. This
# instrument runs the CI-path gate, then rebuilds each RETAINED emitted crate directly and prints
# EVERY error, so the whole set is known in one round instead of one per 15-minute cycle.
# USAGE: bash full_emit_errors_14152.sh <sha> <label> <repair|name-keyed|main>
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"; KIND="${3:?kind}"
WORK=/tmp/fee_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
d="$WORK/repo"
git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "VOID clone"; exit 7; }
( cd "$d" && git checkout -q "$SHA" ) || { echo "VOID checkout"; exit 7; }
echo "MEASURED_AT $(cd $d && git rev-parse HEAD)"
nm=$(grep -c "type_variable_node(id: g)" "$d/src/v1/04_infer.dag" || true)
sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$d/src/v1/04_infer.dag" || true)
nw=$(grep -c "mint_only_without_actuals" "$d/src/v1/04_infer.dag" || true)
acc=$(grep -c "UnifyGenericsAcc" "$d/src/v1/04_infer.dag" || true)
echo "GUARD namekeyed=$nm span_keyed=$sp narrow=$nw acc=$acc"
case "$KIND" in
  repair)     { [ "$acc" = 0 ] && [ "$sp" = 1 ] && [ "$nw" -ge 2 ]; } || { echo "VOID repair guard"; exit 8; } ;;
  main)       { [ "$acc" = 0 ] && [ "$sp" = 0 ] && [ "$nw" = 0 ]; } || { echo "VOID main guard (main carries neither mint nor narrowing)"; exit 8; } ;;
  name-keyed) { [ "$acc" = 0 ] && [ "$sp" = 0 ] && [ "$nm" -ge 2 ]; } || { echo "VOID name-keyed guard"; exit 8; } ;;
esac
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/tgt}"; mkdir -p "$CARGO_TARGET_DIR"
( cd "$d" && CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$CARGO_TARGET_DIR/release/gunbc"
echo "build rc=$brc exists=$([ -x "$bin" ] && echo yes || echo no)"
[ $brc -eq 0 ] && [ -x "$bin" ] || { grep -aE "^error" "$WORK/build.log" | head -8; echo "VOID no gunbc"; exit 6; }
( cd "$d" && "$bin" test //gunbc/instruments:self-host ) > "$WORK/self_host.log" 2>&1
rc=$?
echo "SELF_HOST_RC=$rc lines=$(wc -l < "$WORK/self_host.log")"
grep -aoE "V2-NATIVE REFUSAL cause=[A-Za-z]+ first_error=[^\n]{0,160}" "$WORK/self_host.log" | head -4
# Every retained emitted crate, rebuilt directly, so ALL errors surface.
n=0
for root in $(find /tmp -maxdepth 6 -type d -name 'gunbc-emit-compile-*' 2>/dev/null | sort); do
  for crate in $(find "$root" -maxdepth 2 -name Cargo.toml 2>/dev/null | sort); do
    cd "$(dirname "$crate")" || continue
    n=$((n+1))
    out="$WORK/crate_$n.log"
    CARGO_TARGET_DIR="$WORK/cargo_$n" CARGO_PROFILE_DEV_DEBUG=0 timeout -k 30 900 cargo build --release > "$out" 2>&1
    crc=$?
    tag=$(basename "$(dirname "$crate")")
    echo "=== CRATE $n [$tag] build rc=$crc  errors=$(grep -acE '^error' "$out")"
    grep -aE "^error" "$out" | sort -u | head -12 | cut -c1-210
    grep -aoE "\-\-> [^ ]+" "$out" | sort -u | head -12 | sed 's/^/    site: /'
  done
done
echo "CRATES_BUILT=$n"
echo "T=$(date +%T) done"
