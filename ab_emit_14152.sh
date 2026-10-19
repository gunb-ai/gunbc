#!/usr/bin/env bash
# Root-cause A/B for the E0277 `S: Clone` red on PR-1 head (stern-bat-401 msg_db49ee15 (1)).
# Emit ONLY the closure of dag/std/realization_measurement.dag, with gunbc built from each of two
# printed SHAs, and diff the emitted std_realization_measurement.rs. Same sources both sides -- the
# only difference is the compiler. Prints the emitted signature lines so the change is visible, not inferred.
# USAGE: bash ab_emit_14152.sh <sha_a> <sha_b> <label>
set -uo pipefail
A="${1:?sha_a}"; B="${2:?sha_b}"; LABEL="${3:?label}"
WORK=/tmp/abemit_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
build_and_emit() { # $1=sha $2=tag
  local sha="$1" tag="$2" d="$WORK/$2"
  echo "T=$(date +%T) [$tag] clone $sha"
  git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "VOID clone $tag"; return 7; }
  ( cd "$d" && git checkout -q "$sha" ) || { echo "VOID checkout $tag"; return 7; }
  echo "[$tag] MEASURED_AT $(cd $d && git rev-parse HEAD)"
  local tgt="$WORK/tgt_$2"; mkdir -p "$tgt"
  ( cd "$d" && CARGO_TARGET_DIR="$tgt" CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build_$2.log" 2>&1
  local rc=$?; local bin="$tgt/release/gunbc"
  echo "[$tag] build rc=$rc exists=$([ -x "$bin" ] && echo yes || echo no)"
  [ $rc -eq 0 ] && [ -x "$bin" ] || { tail -20 "$WORK/build_$2.log"; return 6; }
  # GUARD: which compiler is this
  local nm sp; nm=$(grep -c "type_variable_node(id: g)" "$d/src/v1/04_infer.dag" || true)
  sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$d/src/v1/04_infer.dag" || true)
  echo "[$tag] GUARD namekeyed=$nm span_keyed=$sp"
  # the tree under emit is the SOURCE tree, so .dag sources are identical apart from compiler code
  sha256sum "$d/dag/std/realization_measurement.dag" | sed "s|^|[$tag] SRC_SHA |"
  mkdir -p "$WORK/emit_$2"
  ( cd "$d" && "$bin" compile --source-root . --entry dag/std/realization_measurement.dag --output-dir "$WORK/emit_$2" --target rust ) > "$WORK/emit_$2.log" 2>&1
  echo "[$tag] emit rc=$? ($(grep -aoE 'produced [0-9]+ hard diagnostic' "$WORK/emit_$2.log" | head -1))"
  find "$WORK/emit_$2" -name 'std_realization_measurement.rs' | head -1 | tee "$WORK/path_$2"
}
build_and_emit "$A" a || exit $?
build_and_emit "$B" b || exit $?
fa=$(cat "$WORK/path_a"); fb=$(cat "$WORK/path_b")
echo "=== SOURCE identical? (the .dag under test) ==="
diff <(cat "$fa") <(cat "$fb") > "$WORK/emit.diff" 2>&1; drc=$?
echo "EMITTED_DIFF_RC=$drc lines=$(wc -l < "$WORK/emit.diff")"
echo "=== emitted signature lines (fn decls mentioning S) ==="
for t in a b; do
  f=$(cat "$WORK/path_$t")
  echo "--- [$t] ---"
  grep -n "pub fn .*<S>" "$f" | head -14 | cut -c1-190
done
echo "=== emitted diff (first 80 lines) ==="
head -80 "$WORK/emit.diff" | cut -c1-190
echo "=== witness_measured_rollup_differs_from_predicted_zero, both sides ==="
for t in a b; do
  f=$(cat "$WORK/path_$t")
  echo "--- [$t] ---"
  awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' "$f" | head -16 | cut -c1-190
done
echo "=== clone-bound survey in the emitted file ==="
for t in a b; do
  f=$(cat "$WORK/path_$t")
  echo -n "[$t] clone_calls="; grep -ac "\.clone()" "$f"
  echo -n "[$t] S_clone_bound="; grep -ac "S: Clone" "$f"
  echo -n "[$t] mint_ids="; grep -acE '[A-Z]@[^ (]*:[0-9]+-[0-9]+' "$f"
done
echo "T=$(date +%T) done"
