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
export GIT_PAGER=cat PAGER=cat GIT_TERMINAL_PROMPT=0

REF=${REF:?REF required, e.g. refs/heads/probe/pr1-producer-repair}
REPO=${REPO:-https://github.com/gunb-ai/gunbc.git}
# The ONE mirror this change drives. 04_infer.dag emits v1_compiler_infer.rs and nothing else; a
# wider set here would install the emitter's opinion about files this change does not touch.
REGEN_TARGETS=${REGEN_TARGETS:-"v1_compiler_infer.rs"}
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/tmp/gunbc_target_pr1_regen}
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
# measures something else. TREE_KIND selects which tree is expected, and the two modes are mutually
# exclusive: measuring a mutant while believing you are measuring the repair is the exact failure
# this lane keeps hitting, so the guard asserts the MUTANT shape too rather than merely skipping.
DAGSUM=$(sha256sum src/v1/04_infer.dag | cut -c1-16)
FRESH=$(grep -c fresh_call_variables src/v1/04_infer.dag || true)
ACC=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag || true)
NK=$(grep -c 'map_insert(acc, g, type_variable_node(id: g))' src/v1/04_infer.dag || true)
SPAN=$(grep -c 'type_variable_node(id: concat(g, "@", span.file' src/v1/04_infer.dag || true)
# The narrowing (stern-bat-401 msg_c4d4f3e1): the mint fires ONLY for a call with no argument nodes.
# Asserted, not assumed -- a regen from a tree that lost it would emit a mirror that minted at calls
# WITH actuals, which is what broke the self-host product with error[E0277] `S: Clone`.
NARROW=$(grep -c "mint_only_without_actuals" src/v1/04_infer.dag || true)
PIN=$(grep -c "fn arguments_present_do_not_trigger_the_call_site_mint" src/v1/tests/src/generic_call_variable_freshness_test.rs || true)
MIRROR_BEFORE=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
echo "GUARD infer.dag=$DAGSUM fresh=$FRESH namekeyed=$NK span=$SPAN narrow=$NARROW pin=$PIN UnifyGenericsAcc=$ACC mirror_before=$MIRROR_BEFORE"
if [ "${TREE_KIND:-repair}" = repair ]; then
  [ "$NARROW" -ge 2 ] || { echo "VOID: the call-site mint is not narrowed to argument-less calls (found $NARROW, need 2: the binding and the branch)"; exit 5; }
  [ "$PIN" -ge 1 ] || { echo "VOID: the narrowing pin row is missing; the mirror would ship an unpinned widening"; exit 5; }
fi
[ "$ACC" -eq 0 ] || { echo "VOID: the tree carries the accumulator; neither PR-1 nor its mutant may"; exit 5; }
if [ "${TREE_KIND:-repair}" = repair ]; then
  [ "$FRESH" -ge 2 ] || { echo "VOID: the tree has no producer repair"; exit 5; }
  [ "$SPAN" -ge 1 ] || { echo "VOID: the repair does not mint span-keyed ids at the call site"; exit 5; }
  [ "$NK" -le 1 ] || { echo "VOID: the call site mints by the callee's PARAMETER NAME -- this is the MUTANT, not the repair (TREE_KIND=name-keyed if that is what you meant)"; exit 5; }
else
  # The mutant must differ from the repair in exactly one respect, and the guard says which: the
  # call-site fold must mint by parameter name. TWO occurrences of that text is what means it -- one
  # is the pre-existing unbound_owner_param_variables fold, so a bare grep would pass on a tree with
  # no mutation at all.
  [ "$NK" -ge 2 ] || { echo "VOID: name-keyed mutation is not present at the call site (found $NK, need 2)"; exit 5; }
  [ "$SPAN" -eq 0 ] || { echo "VOID: the span-keyed id survived; this is not the mutant"; exit 5; }
fi
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
  # Executability is ASSERTED, not assumed. Two rounds of this instrument died with
  # `timeout: failed to run command '<target>/release/claim_executor': Permission denied` after cargo
  # reported Finished rc=0: the runner's /root persistence is per-container and a directory freshly
  # created there this dispatch can yield a binary that will not exec. Print the mode, chmod, retest;
  # only then run --required-regen, or fail with a reason instead of an rc=126 that looks like drift.
  if [ ! -x "$CE_BIN" ]; then
    echo "PRE-X not executable: $(ls -l "$CE_BIN" 2>&1 | cut -c1-120)  umask=$(umask)"
    chmod +x "$CE_BIN" 2>/dev/null || echo "CHMOD_FAILED"
    [ -x "$CE_BIN" ] || { echo "FATAL emitter binary not executable at $CE_BIN (target dir on this runner cannot exec what cargo wrote; run with CARGO_TARGET_DIR under /tmp)"; exit 6; }
    echo "POST-X chmod repaired it"
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

# HAND THE ARTIFACT BACK AS A VERIFIABLE PATCH.
# Never a colourised whole-file base64 blob: the first attempt filled the output tail, cut every
# verdict line, and left an undecodable stub; and a bare `git diff` in a non-interactive dispatch
# OPENS A PAGER and hangs the job at the client cap. So: GIT_PAGER=cat, --no-pager, --no-color, the
# patch goes to a file, and what is printed is the verdict lines plus a gzipped base64 of the patch.
# Byte-identity is proved by the HASH AFTER APPLYING, not by trusting a copy.
FINAL=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
git --no-pager diff --no-color -- src/v1/stage0/src/v1_compiler_infer.rs > "$LOGS/mirror.patch" 2>/dev/null
PLINES=$(wc -l < "$LOGS/mirror.patch")
echo "FINAL_MIRROR_SHA256 $FINAL"
echo "MIRROR_BEFORE $MIRROR_BEFORE"
echo "PATCH_LINES $PLINES"
if [ "$FINAL" = "$MIRROR_BEFORE" ]; then
  echo "MIRROR_CHANGED no -- the committed mirror already IS the emitter's output"
else
  echo "MIRROR_CHANGED yes"
  # The few lines that matter, printed plainly so a human can eyeball the emitted repair without
  # decoding anything: the minting statement and the merged substitution.
  echo "EMITTED_REPAIR_MARKERS"
  grep -n "uninstantiated_call_generics\|fresh_call_variables\|map_merge(call_subst\|type_variable_node(id: g)" src/v1/stage0/src/v1_compiler_infer.rs | head -14
  echo "PATCH_GZIP_B64_BEGIN"
  gzip -9 < "$LOGS/mirror.patch" | base64 -w0; echo
  echo "PATCH_GZIP_B64_END"
fi
stamp "done"
