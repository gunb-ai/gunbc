#!/usr/bin/env bash
# ONE side of the E0277 A/B. The two-sided script timed out at the 45-min runner cap (two cold
# builds) and, worse, reported EMITTED_DIFF_RC=0 while BOTH emits had failed (rc=1) and no emitted
# file was found -- a void run that looked like a result. This version fails loudly and hands the
# emitted file back so the two sides are diffed offline, one dispatch per compiler.
# USAGE: bash side_emit_14152.sh <sha> <label>
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"
WORK=/tmp/sideemit_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
d="$WORK/repo"
echo "T=$(date +%T) clone $SHA"
git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "VOID clone"; exit 7; }
( cd "$d" && git checkout -q "$SHA" ) || { echo "VOID checkout"; exit 7; }
echo "MEASURED_AT $(cd $d && git rev-parse HEAD)"
nm=$(grep -c "type_variable_node(id: g)" "$d/src/v1/04_infer.dag" || true)
sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$d/src/v1/04_infer.dag" || true)
echo "GUARD namekeyed=$nm span_keyed=$sp"
sha256sum "$d/dag/std/realization_measurement.dag" | sed 's|^|SRC_SHA |'
TGT="${CARGO_TARGET_DIR:-$WORK/tgt}"; mkdir -p "$TGT"
echo "T=$(date +%T) build gunbc (target=$TGT)"
( cd "$d" && CARGO_TARGET_DIR="$TGT" CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$TGT/release/gunbc"
echo "build rc=$brc exists=$([ -x "$bin" ] && echo yes || echo no)"
if [ $brc -ne 0 ] || [ ! -x "$bin" ]; then echo "VOID no gunbc"; grep -aE "^error" "$WORK/build.log" | head -8; exit 6; fi
mkdir -p "$WORK/emit"
echo "T=$(date +%T) emit --entry dag/std/realization_measurement.dag"
( cd "$d" && "$bin" compile --source-root . --entry dag/std/realization_measurement.dag --output-dir "$WORK/emit" --target rust ) > "$WORK/emit.log" 2>&1
erc=$?
echo "EMIT_RC=$erc"
echo "=== emit log tail (NEVER swallow this again) ==="
tail -25 "$WORK/emit.log"
f=$(find "$WORK/emit" -name 'std_realization_measurement.rs' | head -1)
if [ -z "$f" ]; then echo "VOID_NO_EMITTED_FILE -- this side produced nothing, no conclusion allowed"; exit 5; fi
echo "EMITTED_FILE=$f  bytes=$(wc -c < "$f") lines=$(wc -l < "$f")  sha256=$(sha256sum "$f" | cut -c1-16)"
echo "=== the blamed function, verbatim ==="
awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' "$f" | head -18 | cut -c1-200
echo "=== signature lines carrying S ==="
grep -n "pub fn .*<S>" "$f" | head -20 | cut -c1-200
echo "=== counters ==="
echo -n "clone_calls="; grep -ac "\.clone()" "$f"
echo -n "S_clone_bound="; grep -ac "S: Clone" "$f"
echo -n "mint_ids="; grep -acE '[A-Za-z]+@[^ (]*:[0-9]+-[0-9]+' "$f"
echo "=== lines 330-350 (the blamed range) ==="
sed -n '330,350p' "$f" | cat -n | cut -c1-200
echo "BEGIN_EMITTED_GZ"
gzip -c "$f" | base64 -w 100
echo "END_EMITTED_GZ"
echo "T=$(date +%T) done"
