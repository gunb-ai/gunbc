#!/usr/bin/env bash
# Characterize whichever emitted-crate error the CI-path gate reports, WITHOUT assuming which file it
# is in. The refusal line carries `first_error=<msg> @ <path>:<line>:<col>`; the path is inside the
# RETAINED emit tree, which lives under a per-run /tmp/gunbc-emit-compile-* directory. Print the blamed
# line with context, the enclosing function, the mint-id and UNRESOLVED counters for that file, and the
# matching source function in the .dag corpus so the defect can be attributed to a producer rather than
# guessed at. One dispatch, because /tmp is fresh per dispatch.
# USAGE: bash emitted_error_site_14152.sh <sha> <label> <repair|name-keyed|main>
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"; KIND="${3:?kind}"
WORK=/tmp/ees_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
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
  repair) { [ "$acc" = 0 ] && [ "$sp" = 1 ] && [ "$nw" -ge 2 ]; } || { echo "VOID repair guard"; exit 8; } ;;
  main)   { [ "$acc" = 0 ] && [ "$sp" = 0 ] && [ "$nw" = 0 ]; } || { echo "VOID main guard"; exit 8; } ;;
esac
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/tgt}"; mkdir -p "$CARGO_TARGET_DIR"
( cd "$d" && CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$CARGO_TARGET_DIR/release/gunbc"
echo "build rc=$brc exists=$([ -x "$bin" ] && echo yes || echo no)"
[ $brc -eq 0 ] && [ -x "$bin" ] || { grep -aE "^error" "$WORK/build.log" | head -6; echo "VOID no gunbc"; exit 6; }
( cd "$d" && "$bin" test //gunbc/instruments:self-host ) > "$WORK/self_host.log" 2>&1
echo "SELF_HOST_RC=$?"
grep -a "V2-NATIVE REFUSAL" "$WORK/self_host.log" | head -4 | cut -c1-600
grep -a "error" "$WORK/self_host.log" | grep -avE "^[[:space:]]*$" | sort -u | head -12 | cut -c1-400
grep -aoE "produced [0-9]+ hard diagnostic" "$WORK/self_host.log" | head -2
# Locate the blamed emitted file anywhere under the retained emit trees.
loc=$(grep -aoE "@ [^ ]+\.(rs):[0-9]+:[0-9]+" "$WORK/self_host.log" | head -1)
echo "FIRST_ERROR_LOC=$loc"
rel=$(echo "$loc" | sed 's/^@ //; s/:[0-9]*:[0-9]*$//')
base=$(basename "$rel")
echo "LOOKING_FOR=$base"
ALLC=$(find /tmp -maxdepth 8 -path '*gunbc-emit-compile*' -name "$base" 2>/dev/null | sort)
echo "COPIES=$(echo "$ALLC" | grep -c . )"
echo "$ALLC" | sed 's/^/  copy: /' | head -10
f=""
for c in $ALLC; do
  m=$(grep -ac "UNRESOLVED_" "$c" || true)
  echo "  copy_markers $m  $(basename $(dirname $(dirname $(dirname $c))))/$base"
  [ "$m" != "0" ] && [ -z "$f" ] && f="$c"
done
[ -n "$f" ] || f=$(echo "$ALLC" | head -1)
echo "EMITTED_FOUND=${f:-NONE}"
[ -n "$f" ] || { echo "VOID emitted file not found"; exit 5; }
line=$(echo "$loc" | sed 's/.*:\([0-9]*\):[0-9]*$/\1/')
echo "=== emitted lines around $line ==="
awk -v L="$line" 'NR>=L-7 && NR<=L+4 {printf "%5d| %s\n", NR, $0}' "$f" | cut -c1-215
echo "=== enclosing fn ==="
awk -v L="$line" 'NR<=L && /pub fn /{ln=NR; s=$0} END{printf "  nearest pub fn at %d: %s\n", ln, substr(s,1,200)}' "$f"
echo "=== counters for this emitted file ==="
echo -n "UNRESOLVED_CompilerError="; grep -ac "UNRESOLVED_CompilerError" "$f"
echo -n " UNRESOLVED_TypeVariable="; grep -ac "UNRESOLVED_TypeVariable" "$f"
echo -n " mint_ids="; grep -acE '[A-Za-z]+@[^ (]",' "$f"; echo -n " mint_ids_loose="; grep -acE '@[A-Za-z0-9_/.]+\.dag:[0-9]+-[0-9]+' "$f"
echo "=== every UNRESOLVED_* site in this file ==="
grep -anoE "UNRESOLVED_[A-Za-z]+" "$f" | head -12
echo "=== the .dag source of that module, if identifiable ==="
mod=$(echo "$base" | sed 's/^v2_//; s/^v1_//; s/\.rs$//')
echo "  module token: $mod"
cand=$(grep -rl "^module .*${mod//_/.}" "$d/src" "$d/dag" 2>/dev/null | head -2)
echo "  candidate sources: ${cand:-none}"
for c in $cand; do echo "  --- $c (inhabitance-ish fns) ---"; grep -n "fn .*inhabit" "$c" | head -6 | cut -c1-190; done
echo "T=$(date +%T) done"
