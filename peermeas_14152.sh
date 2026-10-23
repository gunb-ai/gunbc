#!/bin/bash
# MEASUREMENT FOR PR #14256 (quiet-eagle-395's node_type_equals_core arm), plus the split run my
# own PR needs. PROVENANCE IS THE HAZARD HERE, so it is stated up front and enforced below:
# peer's arm is applied IN THIS DISPATCH, ON THE RUNNER, from keep/patches/peer_equals_arm.patch
# (commit 92e2b987796). The mirrors this run regenerates therefore CONTAIN THEIR CHANGE and must
# NEVER be installed into #14152. Only two things come back from this run: the self-host log and
# the suite result. The mirrors are tarred solely so a reviewer can reproduce, and are named .peer.
#
# Measured so far, at the converged seed, same 26-case suite and same self-host instrument:
#   my relation (unfold + node_type_equals + declared text pair + bare-head-vs-application):
#     24 passed / 2 failed; 208 rebind refusals (census: UNFOLD_CANDIDATE 104, DISJOINT 38,
#     BOTH_APPLIED 10, SAME_TEXT 7, KERNEL_EMPTY_LEAF 7, FORMAL_VS_CONCRETE 0)
#   delegated to node_type_compatible instead: 22 passed / 4 failed -- the two NEW failures are the
#     reject twins (fold_refuses_two_distinct_settled_bindings_of_the_same_shape,
#     applied_constructor_same_head_differing_argument_list_refuses) -- and 30 refusals, which is
#     NOT a success: that run's log ends at src/v2/compiler/source_root_eval.dag, i.e. the corpus
#     aborted early, so most files were never reached. Fewer refusals there means fewer files typed.
# THIS run answers: with peer's arm, do my reject twins still refuse, and how deep does the corpus
# get? A drop in refusals counts only if the same file set is reached, so both are printed.
# NO git checkout / git clean. Restores 04_types.dag from keep/my_04_types_backup.dag at the end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
# my .dag, main's mirrors as the starting point
cp keep/my_04_infer.dag src/v1/04_infer.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
# peer's arm, applied here and now
cp keep/my_04_types_backup.dag src/v1/04_types.dag
# The peer patch does not apply to my tree (context drifted between bases), so the arm is carried
# as keep/peer_arm_04_types.dag: main's 04_types.dag with the two hunks of commit 92e2b987796
# hand-applied VERBATIM (anchors verified unique, line-by-line against git show 92e2b987796).
cp keep/peer_arm_04_types.dag src/v1/04_types.dag
# MARKERS ARE PRECISE ON PURPOSE. 'left_applied' ALONE IS NOT A PEER MARKER: main/HEAD already
# declares its OWN let left_applied at src/v1/04_types.dag:1274 with a different right-hand side
# ((left.children |> count) > 0), and my previous run grepped the loose name, believed the patch had
# landed when git apply had FAILED, and measured a tree that contained NEITHER the peer arm NOR a
# clean baseline. Markers must match the peer patch's exact text, and a mismatch must STOP the run.
M1=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag)
M2=$(grep -c 'left_applied && (right_leaf || right_struct)' src/v1/04_types.dag)
M3=$(grep -c 'node_type_compatible(left: left_u' src/v1/04_infer.dag)
M4=$(grep -c 'generic_head_matches_application(head: left_u' src/v1/04_infer.dag)
echo "markers: peerLet=$M1 peerArm=$M2 (want 1,1) foldDelegated=$M3 (want 0) foldMine=$M4 (want 1)"
if [ "$M1" != "1" ] || [ "$M2" != "1" ] || [ "$M3" != "0" ] || [ "$M4" != "1" ]; then
  echo "MARKER MISMATCH -- refusing to measure a tree that is not what this script claims."; exit 3
fi
manifest() { git ls-files -z "$SEED" | xargs -0 sha256sum 2>/dev/null | sort > "$1"; }
i=1
while [ $i -le 3 ]; do
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/pr_$i.log 2>&1
  echo "=== ROUND $i rc=$?  diag=$(grep -acE '^type mismatch|^call shape|arm decides' /tmp/pr_$i.log)"
  grep -aE "^type mismatch|^call shape|arm decides" /tmp/pr_$i.log | head -6 | cut -c1-200
  [ -d "$CAND" ] || { echo "no candidate tree"; break; }
  inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    [ -f "$SEED/$f" ] || continue
    cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "  installed $f"; }
  done
  echo "ROUND $i installed=$inst"
  [ "$inst" -eq 0 ] && break
  i=$((i+1))
done
echo "PEER-SEED: equals=$(grep -c 'left_applied && (right_leaf || right_struct)' $SEED/v1_compiler_infer_types.rs) myRelation=$(grep -c generic_head_matches_application $SEED/v1_compiler_infer.rs) fold=$(grep -o UnifyGenericsAcc $SEED/v1_compiler_infer.rs | wc -l) tvLeafRaw=$(grep -c 'binding_has_type_variable_leaf(c' $SEED/v1_compiler_infer.rs)"
# THE DISCRIMINATOR FOR PR #14256, stated as an observable rather than assumed.
# The suite runs once, against the seed this dispatch regenerated (which carries the peer arm).
# To call the new control 'RED because of the arm' I must show the ARM is what answers, so the
# seed is grepped for the arm's emitted signature and printed next to the control's result. If the
# signature is absent the arm did not reach the seed and the control's colour means nothing.
# Baseline comparison is already on the record from the previous dispatch at the same head with the
# same test file and no arm: the same control suite ran 25 passed / 1 failed (only the alias accept
# failed) -- so any control that changes colour here is changed by the arm and nothing else.
echo "=== emitted arm signature in the seed (proof the arm is in what I am testing) ==="
grep -oE "left_applied && \\(right_leaf|right_struct" $SEED/v1_compiler_infer_types.rs | head -2
echo "arm-in-seed=$(grep -c 'right_struct' $SEED/v1_compiler_infer_types.rs) (want >=1)"
echo "########## SUITE ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result" | head -34
echo "########## SELF-HOST ##########"
timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/pr_sh.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/pr_sh.log)"
echo "=== REACHED: distinct .dag files emitting any diagnostic (the denominator for 'fewer refusals') ==="
grep -aoE "src/[a-z0-9/_.]+\.dag" /tmp/pr_sh.log | sort -u | wc -l
grep -aoE "src/[a-z0-9/_.]+\.dag" /tmp/pr_sh.log | sort | uniq -c | sort -rn | head -6
echo "=== how did it end ==="
tail -6 /tmp/pr_sh.log | cut -c1-190
cp /tmp/pr_sh.log selfhost_peerarm2.log
echo "=== BEGIN-TAR-LOG ==="; ( cd . && tar -cz selfhost_peerarm2.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR-LOG ==="
echo "=== BEGIN-TAR-PEERMIRRORS (CONTAMINATED: carries PR #14256; never install into #14152) ==="
( cd $SEED && tar -cz $MIRRORS 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR-PEERMIRRORS ==="
cp keep/my_04_types_backup.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: 04_types=$(sha256sum src/v1/04_types.dag|cut -c1-12) peerarm=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag) (want 0) dag=$(sha256sum src/v1/04_infer.dag|cut -c1-12)"
echo PEERMEASURE_DONE
