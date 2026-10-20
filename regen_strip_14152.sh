#!/usr/bin/env bash
# Regenerate the stage0 mirrors that src/v1/04_types.dag drives, for the STRIP tree
# (scratch/acc-without-equality-widening = PR-2 head with node_type_equals_core's equality widening
# reverted to main's spelling). Answers stern-bat-401's consumer question: does PR-2's Acc need it?
#
# WHY REGEN IS MANDATORY HERE: the gate builds gunbc from the REF and the crate compiles the stage0
# MIRRORS. A .dag-only strip would measure the head's own compiled comparator and report "the
# widening has no consumer" for the wrong reason. Mirrors are emitter output only — never hand-edited,
# never --ours/--theirs (the #14287 silent-merge incident is the standing lesson).
#
# SKELETON: copied from regen_pr1_14152.sh, which converged, keeping its three lessons:
#  - StaleBinary guard: the emitter is rebuilt from the CURRENT mirrors at the top of every round, and
#    the ctrl-build cargo wrapper has been seen to report Finished while the requested target dir held
#    nothing, so the binary is LOCATED, not assumed.
#  - WHAT THE EMITTER SAID decides convergence, not what I installed: installed=0 with drift still
#    reported elsewhere is a false fixed point.
#  - The artifact comes back as a gzip+base64 PATCH, and byte-identity is proved by the hash AFTER
#    applying. No colourised whole-file blob, no bare git diff (pager hangs the dispatch).
set -uo pipefail
export GIT_PAGER=cat PAGER=cat GIT_TERMINAL_PROMPT=0

REF=${REF:-refs/heads/scratch/acc-without-equality-widening}
REPO=${REPO:-https://github.com/gunb-ai/gunbc.git}
# The mirrors this .dag is allowed to own. Anything else the emitter wants to rewrite is REPORTED as
# unowned drift and never installed, so a strip cannot smuggle in an unrelated mirror rewrite.
OWNED=${OWNED:-"v1_compiler_infer_types.rs"}
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/root/gunbc_target_strip}
export CARGO_TARGET_DIR CARGO_PROFILE_DEV_DEBUG=0
LOGS=${LOGS:-/tmp/strip_regen_logs}
WORK=${WORK:-/tmp/strip_regen_tree}
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

# --- GUARDS: assert the tree is the strip I think it is, in both directions. ---
# Measuring a tree that silently kept the widening is the exact failure this lane keeps hitting, so the
# strip shape is ASSERTED (not skipped) and the head shape is rejected.
DAGSUM=$(sha256sum src/v1/04_types.dag | cut -c1-16)
CALLS=$(grep -c 'type_heads_agree(left:' src/v1/04_types.dag || true)   # defs also match; expected 1 = the def line
APPLIED=$(grep -c 'left_applied = left.connective == NoConnective && (left.children |> count) > 0' src/v1/04_types.dag || true)
PEEL=$(grep -c 'peel_alias_rhs_once(n: left' src/v1/04_types.dag || true)
DEFPEEL=$(grep -c 'fn peel_alias_rhs_once' src/v1/04_types.dag || true)
GUA=$(grep -c 'fn type_heads_agree' src/v1/04_types.dag || true)
MIRROR_BEFORE=$(sha256sum src/v1/stage0/src/v1_compiler_infer_types.rs | cut -c1-16)
MCALLS=$(grep -c 'type_heads_agree(' src/v1/stage0/src/v1_compiler_infer_types.rs || true)
echo "GUARD types.dag=$DAGSUM type_heads_agree_lines=$CALLS defs=$GUA applied_arm=$APPLIED peel_call=$PEEL peel_def=$DEFPEEL mirror_before=$MIRROR_BEFORE mirror_type_heads_agree_lines=$MCALLS"
# The strip must have NO call sites: total lines minus the one definition.
[ $((CALLS - GUA)) -eq 0 ] || { echo "VOID: $((CALLS-GUA)) type_heads_agree CALL SITES survive; this is not the strip"; exit 5; }
# The alias machinery must SURVIVE: dropping it here would confound the strip with parent's other variable.
[ "$PEEL" -ge 1 ] && [ "$DEFPEEL" -ge 1 ] || { echo "VOID: the alias peel machinery is gone; the strip must keep it (parent's 3-head discriminator separates it)"; exit 5; }
# The widening's second half must be gone with the first: an applied/applied arm left calling the
# comparator is not "the widening stripped".
[ "$APPLIED" -eq 0 ] || { echo "VOID: the applied/applied declaration-identity arm survives ($APPLIED); strip is half-done"; exit 5; }
# And the drift must EXIST, or the whole exercise is vacuous: the committed mirror still calls it.
[ "$MCALLS" -ge 1 ] || { echo "VOID: the committed mirror already has no call sites; nothing to regenerate — report instead of running"; exit 5; }

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
  if [ ! -x "$CE_BIN" ]; then
    echo "PRE-X absent at $CE_BIN (umask=$(umask)); searching for what the wrapper built"
    tail -6 "$LOGS/emitter_build_$round.log" | sed 's/^/    /'
    FOUND=$(find "$CARGO_TARGET_DIR" /root/workspace -maxdepth 6 -type f -name claim_executor 2>/dev/null | head -3)
    echo "  found: ${FOUND:-NOTHING}"
    if [ -n "$FOUND" ]; then
      CE_BIN=$(echo "$FOUND" | head -1); echo "  USING $CE_BIN"
      chmod +x "$CE_BIN" 2>/dev/null || true
    fi
    [ -x "$CE_BIN" ] || { echo "FATAL no runnable claim_executor near $CARGO_TARGET_DIR"; exit 6; }
  fi
  rm -rf target/stage0-regen-candidate
  timeout -k 60 "${REGEN_TIMEOUT:-1800}" "$CE_BIN" --required-regen \
      --source-root dag --source-root src/v2 > "$LOGS/regen_$round.log" 2>&1
  erc=$?
  stamp "round $round: emitter rc=$erc (1 = drift found, the normal case while installing)"
  [ -d target/stage0-regen-candidate/src ] \
    || { echo "FATAL no candidate tree (rc=$erc)"; tail -20 "$LOGS/regen_$round.log"; exit 6; }
  installed=0
  for f in $OWNED; do
    [ -f "src/v1/stage0/src/$f" ] || { echo "FATAL $f is not an existing mirror; refusing to invent it"; exit 6; }
    [ -f "target/stage0-regen-candidate/src/$f" ] || { echo "FATAL emitter produced no candidate for $f"; exit 6; }
    if ! cmp -s "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"; then
      cp "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"
      installed=$((installed+1)); echo "  INSTALLED $f"
    fi
  done
  MIRROR_NOW=$(sha256sum src/v1/stage0/src/v1_compiler_infer_types.rs | cut -c1-16)
  DRIFT=$(grep -aoE "generated surface drift: [a-z_0-9./-]+" "$LOGS/regen_$round.log" | sed 's/.*: //' | sort -u | tr '\n' ',')
  DECL=$(grep -ac "declared_divergent" "$LOGS/regen_$round.log" || true)
  stamp "round $round: installed=$installed mirror=$MIRROR_NOW drift=[$DRIFT] declared_divergent_lines=$DECL"
  if [ "$round" -ge 2 ] && [ "$installed" -eq 0 ] && [ -z "$DRIFT" ]; then
    echo "FIXED POINT at round $round, mirror=$MIRROR_NOW"; break
  fi
  if [ "$installed" -eq 0 ] && [ -n "$DRIFT" ]; then
    echo "UNOWNED_DRIFT $DRIFT"
    echo "  (not installed; this change owns only: $OWNED)"
    if [ "$round" -ge 2 ]; then echo "FIXED POINT for the owned mirrors at round $round (with reported unowned drift)"; break; fi
  fi
done

FINAL=$(sha256sum src/v1/stage0/src/v1_compiler_infer_types.rs | cut -c1-16)
FCALLS=$(grep -c 'type_heads_agree(' src/v1/stage0/src/v1_compiler_infer_types.rs || true)
echo "FINAL_MIRROR_SHA256 $FINAL"
echo "MIRROR_BEFORE $MIRROR_BEFORE"
echo "S2_MIRROR_CALL_SITES before=$MCALLS after=$FCALLS  (S2 wants after=1: the surviving DEFINITION only)"
git --no-pager diff --no-color -- src/v1/stage0/src/v1_compiler_infer_types.rs > "$LOGS/mirror.patch" 2>/dev/null
echo "PATCH_LINES $(wc -l < "$LOGS/mirror.patch")"
if [ "$FINAL" = "$MIRROR_BEFORE" ]; then
  echo "MIRROR_CHANGED no -- the committed mirror already IS the emitter's output for this .dag"
else
  echo "MIRROR_CHANGED yes"
  echo "WHAT_REMAINS_OF_THE_COMPARATOR"
  grep -n "type_heads_agree\|fn node_type_equals_core\|peel_alias_rhs_once" src/v1/stage0/src/v1_compiler_infer_types.rs | head -12
  echo "PATCH_GZIP_B64_BEGIN"; gzip -9 < "$LOGS/mirror.patch" | base64 -w0; echo; echo "PATCH_GZIP_B64_END"
fi
stamp "done"
