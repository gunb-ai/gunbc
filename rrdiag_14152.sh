#!/bin/bash
# WHY DID THE REGEN PRODUCE NO CANDIDATE TREE? The previous config-B dispatch is void, and I am
# recording why rather than re-running it and hoping: --required-regen returned rc=1 with 0 lines
# matching my hard-diagnostic grep, target/stage0-regen-candidate/src did not exist, so nothing was
# installed, so the "suite at config B" was in fact CONFIG A again (same 25/3, and the same three
# rows failing at the same file:line 242/257 -- identical output is the proof it was the same seed).
# Nothing about the peer's arm was measured.
#
# The candidate tree is what converge_14152.sh installs from, and that route DID work (round 2 rc=0,
# it installed v1_compiler_infer.rs / _infer_types.rs / _claim_generic_identity_census.rs and
# REPORTED two candidate files not in the seed, cli_run + module_path_index). The only difference
# here is the source: quiet-eagle-395's 04_types.dag at 835cbd38e1e plus my three branch-local
# helpers. So the live question is narrow: does their arm make SELF-HOST EMISSION refuse (no
# candidate, because emission failed) or merely drift (candidate present, rc=1 by drift)? Those are
# different findings and only the log distinguishes them, so this dispatch keeps the WHOLE log and
# tars it FIRST, before anything else can eat the remote time cap.
#
# NO git checkout / git clean (it wiped carriers mid-run once). Restores by cp at the end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
REF=keep/converged3
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src

echo "########## baseline = MY converged seed (never main's mirrors: the test crate needs my\nTextJudgment argument on unify_generics, so main's mirrors do not even compile) ##########"
for f in $MIRRORS; do cp "$REF/$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "baseline seed: bindAs=$(grep -ac 'bound as' $SEED/v1_compiler_infer.rs) U=$(grep -ac UnifyGenericsAcc $SEED/v1_compiler_infer.rs) (want >0 both) sha=$(sha256sum $SEED/v1_compiler_infer.rs | cut -c1-16)"

echo "########## apply the arm to the SOURCE ##########"
cp keep/peer_arm_04_types.dag src/v1/04_types.dag
echo "source: lockstepWalker=$(grep -ac arguments_agree_in_lockstep src/v1/04_types.dag) appliedSpelling=$(grep -ac applied_spelling_arguments_agree src/v1/04_types.dag) declIdentity=$(grep -ac type_declaration_identity_key src/v1/04_types.dag) myHelpers=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag) totalLines=$(grep -c '' src/v1/04_types.dag)"

echo "########## build the regenerator ##########"
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -3

rm -rf target/stage0-regen-candidate 2>/dev/null
echo "########## ROUND 1: --required-regen, whole log kept ##########"
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_full.log 2>&1
rc=$?
echo "ROUND1 rc=$rc"
echo "--- the receipt lines (these carry the verdict):"
grep -a "^required-regen:" /tmp/rr_full.log | head -8 | cut -c1-230
echo "--- first 18 lines:"; head -18 /tmp/rr_full.log | cut -c1-200
echo "--- last 25 lines:"; tail -25 /tmp/rr_full.log | cut -c1-200
echo "--- counts: totalLines=$(grep -c '' /tmp/rr_full.log) refus=$(grep -aci 'refus' /tmp/rr_full.log) mismatch=$(grep -ac 'type mismatch' /tmp/rr_full.log) panic=$(grep -ac 'panic' /tmp/rr_full.log) hard=$(grep -acE '^type mismatch|^call shape|arm decides' /tmp/rr_full.log)"
echo "--- candidate dir:"; ls -l $CAND 2>&1 | head -12
if [ -d "$CAND" ]; then
  echo "########## candidate PRESENT -> install + converge to a fixed point ##########"
  inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    if [ -f "$SEED/$f" ]; then
      cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "  installed $f"; }
    else
      echo "  CANDIDATE FILE NOT IN SEED (reported, not committed): $f"
    fi
  done
  echo "round1 installed=$inst"
  j=2
  while [ $j -le 3 ] && [ "$inst" != "0" ]; do
    cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
    target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_full_$j.log 2>&1
    echo "=== round $j rc=$? ; receipt:"; grep -a "^required-regen:" /tmp/rr_full_$j.log | head -4 | cut -c1-220
    inst=0
    for f in $(ls "$CAND" 2>/dev/null); do
      [ -f "$SEED/$f" ] || continue
      cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "  installed $f"; }
    done
    echo "round $j installed=$inst"; j=$((j+1))
  done
  echo "SEED-LEVEL PROOF OF THE ARM (emitted form): lockstep=$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs) appliedSpelling=$(grep -ac applied_spelling_arguments_agree $SEED/v1_compiler_infer_types.rs) declIdentity=$(grep -ac type_declaration_identity_key $SEED/v1_compiler_infer_types.rs) myHelpers=$(grep -ac 'fn type_head_carrier' $SEED/v1_compiler_infer_types.rs)"
else
  echo "CANDIDATE ABSENT -- emission did not produce a tree. The log above is the answer; do not read anything from a suite run after this."
fi

echo "=== BEGIN-TAR (log FIRST, it is the artifact this dispatch exists for) ==="
cp /tmp/rr_full.log rr_full.log 2>/dev/null
tar -c rr_full.log 2>/dev/null | base64 -w0; echo
echo "=== END-TAR ==="

echo "########## restore ##########"
cp keep/my_04_types_backup.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "$REF/$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: peerMarkers=$(grep -ac applied_spelling_arguments_agree src/v1/04_types.dag) (want 0)"
echo RRDIAG_DONE
