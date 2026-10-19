#!/usr/bin/env bash
# DECISIVE A/B for the E0277 `S: Clone` red. One dispatch, both sides, because the runner's /tmp is
# fresh per dispatch. Each side: fresh clone at a printed SHA, tree staged WITHOUT docs/measurements
# (two fixtures there declare module gunbc.t and the compile refuses on a module-path collision
# before emitting -- the void that killed the first two A/B attempts), gunbc built from that side's
# own tree, then emit the source_root_eval closure. Prints per side: the tree GUARD, the emitted
# file's sha256, the blamed function verbatim, and the counters. Then diffs the two emitted files.
# USAGE: bash both_emit3_14152.sh <head_sha> <main_sha>
set -uo pipefail
run_side() { # $1 sha  $2 tag  $3 targetdir
  local sha="$1" tag="$2" tgt="$3"
  local w="/tmp/b3_$tag"; rm -rf "$w"; mkdir -p "$w"
  local d="$w/repo"
  git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "[$tag] VOID clone"; return 7; }
  ( cd "$d" && git checkout -q "$sha" ) || { echo "[$tag] VOID checkout"; return 7; }
  echo "[$tag] MEASURED_AT $(cd $d && git rev-parse HEAD)"
  local nm sp; nm=$(grep -c "type_variable_node(id: g)" "$d/src/v1/04_infer.dag" || true)
  sp=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' "$d/src/v1/04_infer.dag" || true)
  echo "[$tag] GUARD namekeyed=$nm span_keyed=$sp"
  sha256sum "$d/dag/std/realization_measurement.dag" | sed "s|^|[$tag] SRC_SHA |"
  mkdir -p "$tgt"
  echo "[$tag] T=$(date +%T) build target=$tgt"
  ( cd "$d" && CARGO_TARGET_DIR="$tgt" CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$w/build.log" 2>&1
  local rc=$?; local bin="$tgt/release/gunbc"
  echo "[$tag] build rc=$rc exists=$([ -x "$bin" ] && echo yes || echo no)"
  [ $rc -eq 0 ] && [ -x "$bin" ] || { grep -aE "^error" "$w/build.log" | head -6; return 6; }
  local s="$w/src"; mkdir -p "$s"
  tar -C "$d" --exclude='./target' --exclude='./.git' -cf - . 2>/dev/null | tar -C "$s" -xf -
  # rm -rf, NOT --exclude: the tar exclude silently left the fixture (collisions=1) and the emit then
  # refused with a module-path collision before writing anything -- a void that looked like an emit bug.
  rm -rf "$s/docs/measurements"
  echo "[$tag] staged_dags=$(find "$s" -name '*.dag' | wc -l) collisions=$(find "$s" -path '*value_position_block_nests*' | wc -l)"
  mkdir -p "$w/emit"
  ( cd "$s" && "$bin" compile --source-dir . --output-dir "$w/emit" --target rust --entry src/v2/compiler/source_root_eval.dag ) > "$w/emit.log" 2>&1
  echo "[$tag] EMIT_RC=$? $(grep -aoE 'produced [0-9]+ hard diagnostic' "$w/emit.log" | head -1)"
  echo "[$tag] --- emit log (first 25 lines) ---"; head -25 "$w/emit.log"
  local f=$(find "$w/emit" -name std_realization_measurement.rs | head -1)
  if [ -z "$f" ]; then echo "[$tag] VOID_NO_EMITTED_FILE"; find "$w/emit" -maxdepth 2 2>/dev/null | head -8; return 5; fi
  echo "[$tag] EMITTED sha256=$(sha256sum "$f" | cut -c1-16) lines=$(wc -l < "$f")"
  echo "[$tag] S_clone_bound=$(grep -ac 'S: Clone' "$f") clone_calls=$(grep -ac '\.clone()' "$f") mint_ids=$(grep -acE '[A-Za-z]+@[^ (]*:[0-9]+-[0-9]+' "$f")"
  echo "[$tag] --- blamed CALLER verbatim ---"
  awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' "$f" | head -18 | cut -c1-210
  echo "[$tag] --- CALLEE signatures (the decisive lines: Rc wrapper present or not) ---"
  grep -n "pub fn cost_account_measured_from_time\|pub fn cost_account_time_count" "$f" | cut -c1-210
  echo "[$tag] line 341 of the emitted file:"; sed -n '341p' "$f" | cut -c1-210
  cp "$f" "/tmp/emitted_$tag.rs"
  return 0
}
run_side "${1:?head sha}" head /root/gunbc_target_side_head || { echo "VOID head side"; exit 5; }
run_side "${2:?main sha}" main /root/gunbc_target_side_main || { echo "VOID main side"; exit 5; }
echo "=== VERDICT ==="
if cmp -s /tmp/emitted_head.rs /tmp/emitted_main.rs; then
  echo "EMITTED_IDENTICAL -- my change did not alter this emitted file; the mint is NOT the cause of the E0277"
else
  echo "EMITTED_DIFFERS"; diff /tmp/emitted_main.rs /tmp/emitted_head.rs | head -50 | cut -c1-210
fi
