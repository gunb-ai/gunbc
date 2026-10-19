#!/usr/bin/env bash
# Regenerate the stage0 mirror that src/v1/04_infer.dag drives, for the PR-1 (producer-repair) tree.
#
# WHY A FRESH CLONE AND A PRINTED SHA. Everything measured in this lane is measured on a clone of a
# ref resolved inside the dispatch, at a SHA this script prints and checks. The shared runner
# workspace is dirty with other sessions' uncommitted .dag edits and moves between dispatches; a
# measurement taken there is not a measurement of this tree (that mistake produced a false
# "generated surface drift" report earlier in this lane).
#
# WHY ROUNDS. The compiler is written in the language it emits, so the mirror of 04_infer.dag is an
# input to what the NEXT round emits. Rounds continue until a round installs nothing AND the emitter
# itself reports no drift.
#
# THE STALEBINARY TRAP. The emitter binary is built from the mirrors. A binary built before the
# previous round installed its output would emit the PREVIOUS round's opinion, which converges to a
# fixed point of the wrong function. The emitter is therefore rebuilt at the top of every round.
#
# THE INSTALL RULE. Only a file in REGEN_TARGETS may be installed, and only if it already exists in
# src/v1/stage0/src/. A recursive find over the candidate tree descends into subdirectories
# (module_path_index/, bin/) that have no counterpart in the mirror dir and once dropped 27 generated
# files in the repo ROOT. And "the emitter produced it" is not a licence to install: a mirror can
# carry a DECLARED divergence where candidate != tree is expected.
set -uo pipefail

REF=${REF:?REF required, e.g. refs/heads/probe/pr1-producer-repair}
REPO=${REPO:-https://github.com/gunb-ai/gunbc.git}
# The ONE mirror this change drives. 04_infer.dag emits v1_compiler_infer.rs and nothing else; a
# wider set here would install the emitter's opinion about files this change does not touch.
REGEN_TARGETS=${REGEN_TARGETS:-"v1_compiler_infer.rs"}
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/root/gunbc_target_pr1}
export CARGO_TARGET_DIR CARGO_PROFILE_DEV_DEBUG=0
LOGS=${LOGS:-/tmp/pr1_regen_logs}
WORK=${WORK:-/tmp/pr1_regen_tree}
mkdir -p "$LOGS"
stamp(){ echo "[$(date -u +%H:%M:%S)] $*"; }

rm -rf "$WORK"
git clone --quiet --no-checkout "$REPO" "$WORK" 2>&1 | tail -2
SHA=$(git -C "$WORK" ls-remote origin "$REF" | awk '{print $1}')
[ ${#SHA} -ge 40 ] || { echo "FATAL cannot resolve $REF"; exit 4; }
git -C "$WORK" checkout -q --detach "$SHA"
cd "$WORK" || { echo "FATAL clone missing"; exit 4; }
echo "MEASURED_AT $SHA"
echo "REF $REF"
# GUARD, not a comment: if the tree the clone produced is not the tree I committed, everything below
# measures something else. The .dag must carry the repair and must NOT carry the accumulator.
DAGSUM=$(sha256sum src/v1/04_infer.dag | cut -c1-16)
FRESH=$(grep -c fresh_call_variables src/v1/04_infer.dag || true)
ACC=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag || true)
MIRROR_BEFORE=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
echo "GUARD infer.dag=$DAGSUM fresh_call_variables=$FRESH UnifyGenericsAcc=$ACC mirror_before=$MIRROR_BEFORE"
[ "$FRESH" -ge 2 ] || { echo "VOID: the tree has no producer repair"; exit 5; }
[ "$ACC" -eq 0 ] || { echo "VOID: the tree carries the accumulator; PR-1 must not"; exit 5; }
grep -q "fn minted_variables_are_fresh_per_call_site" src/v1/tests/src/generic_call_variable_freshness_test.rs \
  || { echo "VOID: the RED is not in the tree"; exit 5; }
grep -q "^mod generic_call_variable_freshness_test;" src/v1/tests/src/lib.rs \
  || { echo "VOID: the RED's module is not registered in lib.rs -- it would never run"; exit 5; }

CE_BIN="$CARGO_TARGET_DIR/release/claim_executor"
round=0
while :; do
  round=$((round+1))
  if [ "$round" -gt 6 ]; then echo "FATAL no fixed point after $round rounds"; exit 6; fi
  stamp "round $round: rebuilding the emitter from the CURRENT mirrors (StaleBinary guard)"
  ( cargo build --release --bin claim_executor ) > "$LOGS/emitter_build_$round.log" 2>&1
  brc=$?
  if [ $brc -ne 0 ]; then
    echo "FATAL emitter build rc=$brc round=$round"; grep -aE "^error" "$LOGS/emitter_build_$round.log" | head -8; exit 6
  fi
  rm -rf target/stage0-regen-candidate
  timeout -k 60 "${REGEN_TIMEOUT:-1800}" "$CE_BIN" --required-regen \
      --source-root dag --source-root src/v2 > "$LOGS/regen_$round.log" 2>&1
  erc=$?
  stamp "round $round: emitter rc=$erc (1 = drift found, the normal case while installing)"
  [ -d target/stage0-regen-candidate/src ] \
    || { echo "FATAL no candidate tree at target/stage0-regen-candidate/src (rc=$erc)"; tail -20 "$LOGS/regen_$round.log"; exit 6; }
  installed=0
  for f in $REGEN_TARGETS; do
    [ -f "src/v1/stage0/src/$f" ] || { echo "FATAL $f is not an existing mirror; refusing to invent it"; exit 6; }
    if [ ! -f "target/stage0-regen-candidate/src/$f" ]; then
      echo "FATAL emitter produced no candidate for $f"; exit 6
    fi
    if ! cmp -s "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"; then
      cp "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"
      installed=$((installed+1)); echo "  INSTALLED $f"
    fi
  done
  MIRROR_NOW=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
  # WHAT THE EMITTER SAID, not what I installed, decides convergence. installed=0 can mean "the file
  # I am allowed to install happens to match" while the emitter still reports drift in a file I am
  # NOT allowed to install -- that is a false fixed point, and it happened in this lane.
  DRIFT=$(grep -aoE "generated surface drift: [a-z_0-9./-]+" "$LOGS/regen_$round.log" | sed 's/.*: //' | sort -u | tr '\n' ',')
  DECL=$(grep -ac "declared_divergent" "$LOGS/regen_$round.log" || true)
  stamp "round $round: installed=$installed mirror=$MIRROR_NOW drift=[$DRIFT] declared_divergent_lines=$DECL"
  if [ "$round" -ge 2 ] && [ "$installed" -eq 0 ] && [ -z "$DRIFT" ]; then
    echo "FIXED POINT at round $round, mirror=$MIRROR_NOW"; break
  fi
  if [ "$installed" -eq 0 ] && [ -n "$DRIFT" ]; then
    # Drift outside the set this change may touch: report it, never install it. A PR that silently
    # reformats a mirror it does not own is unreviewable, and main may legitimately carry a declared
    # divergence that a naive install would destroy.
    echo "UNOWNED_DRIFT $DRIFT"
    echo "  (not installed; this change owns only: $REGEN_TARGETS)"
    if [ "$round" -ge 2 ]; then echo "FIXED POINT for the owned mirrors at round $round (with reported unowned drift)"; break; fi
  fi
done

# HAND THE ARTIFACT BACK AS A VERIFIABLE DIFF, NOT AS BASE64 OF THE WHOLE FILE.
# The base64 of a ~2 MB mirror does not survive the output path: a `tail -c` on the dispatch cut it
# and left an undecodable stub, so the round cost 25 minutes and yielded nothing. What is needed is
# (a) the emitter's own verdict, (b) the final hash, and (c) enough of the change to apply locally.
# Hash equality after applying the patch is the proof of byte-identity -- stronger than trusting a
# copy, because a truncated or corrupted hand-back shows up as a hash mismatch instead of a bad file.
FINAL=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
echo "FINAL_MIRROR_SHA256 $FINAL"
if [ "$FINAL" = "$MIRROR_BEFORE" ]; then
  echo "MIRROR_CHANGED no -- the committed mirror already IS the emitter's output"
else
  echo "MIRROR_CHANGED yes"
  git diff --stat -- src/v1/stage0/src/v1_compiler_infer.rs | tail -2
  DIFFLINES=$(git diff -U3 -- src/v1/stage0/src/v1_compiler_infer.rs | wc -l)
  echo "DIFF_LINES $DIFFLINES"
  if [ "$DIFFLINES" -le 4000 ]; then
    echo "DIFF_BEGIN"
    git diff -U3 -- src/v1/stage0/src/v1_compiler_infer.rs
    echo "DIFF_END"
  else
    # Too big to inline: gzip+base64 it and let the caller page through the persisted output file.
    echo "DIFF_TAR_GZ_BEGIN"
    git diff -U3 -- src/v1/stage0/src/v1_compiler_infer.rs | gzip -9 | base64 -w0
    echo
    echo "DIFF_TAR_GZ_END"
  fi
fi
stamp "done"
