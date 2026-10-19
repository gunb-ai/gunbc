#!/usr/bin/env bash
# ONE side of the E0277 A/B, v3. v1/v2 died in source-discovery: `--source-root .` walks
# docs/measurements/value_position_block_nests/multi_d{10,14}/dag/gunbc/t.dag, two files declaring
# module gunbc.t, and compile refuses with a module-path collision before emitting anything. The fix
# is not different flags: stage the tree without those measurement fixtures, which the closure does
# not import. One dispatch per side; the emitted file comes back gzipped for an offline diff.
# USAGE: bash side_emit3_14152.sh <sha> <label>
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"
WORK=/tmp/side3_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
d="$WORK/repo"
git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "VOID clone"; exit 7; }
( cd "$d" && git checkout -q "$SHA" ) || { echo "VOID checkout"; exit 7; }
echo "MEASURED_AT $(cd $d && git rev-parse HEAD)"
nm=$(grep -c "type_variable_node(id: g)" "$d/src/v1/04_infer.dag" || true)
sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$d/src/v1/04_infer.dag" || true)
echo "GUARD namekeyed=$nm span_keyed=$sp"
sha256sum "$d/dag/std/realization_measurement.dag" | sed 's|^|SRC_SHA |'
TGT="${CARGO_TARGET_DIR:-$WORK/tgt}"; mkdir -p "$TGT"
echo "T=$(date +%T) build gunbc target=$TGT"
( cd "$d" && CARGO_TARGET_DIR="$TGT" CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$TGT/release/gunbc"
echo "build rc=$brc exists=$([ -x "$bin" ] && echo yes || echo no)"
[ $brc -eq 0 ] && [ -x "$bin" ] || { grep -aE "^error" "$WORK/build.log" | head -8; echo "VOID no gunbc"; exit 6; }
S="$WORK/src"; mkdir -p "$S"
tar -C "$d" --exclude='./docs/measurements' --exclude='./target' --exclude='./.git' -cf - . 2>/dev/null | tar -C "$S" -xf -
echo "staged files=$(find "$S" -name '*.dag' | wc -l) .dag  collision_files=$(find "$S" -path '*value_position_block_nests*' | wc -l)"
mkdir -p "$WORK/emit"
echo "T=$(date +%T) emit --entry src/v2/compiler/source_root_eval.dag"
( cd "$S" && "$bin" compile --source-dir . --output-dir "$WORK/emit" --target rust --entry src/v2/compiler/source_root_eval.dag ) > "$WORK/emit.log" 2>&1
echo "EMIT_RC=$?"
echo "--- emit log: hard diagnostics / refusal / first error ---"
grep -aoE "produced [0-9]+ hard diagnostic|V2-NATIVE REFUSAL cause=[A-Za-z]+|error\[E[0-9]+\][^\"]{0,100}" "$WORK/emit.log" | head -6
tail -4 "$WORK/emit.log"
f=$(find "$WORK/emit" -name std_realization_measurement.rs | head -1)
if [ -z "$f" ]; then echo "VOID_NO_EMITTED_FILE"; find "$WORK/emit" -maxdepth 2 2>/dev/null | head -12; exit 5; fi
echo "EMITTED_FILE=$f lines=$(wc -l < "$f") sha256=$(sha256sum "$f" | cut -c1-16)"
echo "=== blamed function verbatim ==="
awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' "$f" | head -20 | cut -c1-210
echo "=== signatures carrying <S> ==="
grep -n "pub fn .*<S>" "$f" | head -12 | cut -c1-210
echo -n "S_clone_bound="; grep -ac "S: Clone" "$f"
echo -n " clone_calls="; grep -ac "\.clone()" "$f"
echo " mint_ids="; grep -acE '[A-Za-z]+@[^ (]*:[0-9]+-[0-9]+' "$f"
echo "BEGIN_EMITTED_GZ"; gzip -c "$f" | base64 -w 100; echo "END_EMITTED_GZ"
echo "T=$(date +%T) done"
