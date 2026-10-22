set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14652 @ d5648d9fc43 (supersedes 1331788; PR delta = schedule file only, claims live on main).
# Manager per-head asks: (1) head-built COMPILE of src/v2/workflow/local_repo_wet_terminal.dag
#   (via its fold claims file, which parses + audits every row: duplicate field or empty row
#   would land all-UNOBSERVED parse-failed, proven in the #14601 dup-probe);
# (2) schedule diff vs main = exactly the PR's own 2 rows (done locally, re-assert here);
# (3) the 4 claims by name wet, head-built; (4) infer-revert mutation (#14185 hunk): EXACTLY the
#   two w_an_equality_* claims FAIL, both census controls PASS, wet + hermetic on author roots
#   (dag+src/v2); (5) restore byte-exact, rebuild, POST 4/4.
OID=d5648d9fc43899d0a80d298eefe375aea0aa9f34   # gh pr view 14652 --json headRefOid (manager prefix d5648d9fc43)
F=dag/test/claim/record_escapes_declared_optional_return_witness.dag
LCW=src/v2/test/claim/local_repo_wet_terminal.dag
LWT=src/v2/workflow/local_repo_wet_terminal.dag
INFER=src/v1/stage0/src/v1_compiler_infer.rs
ALL4="w_an_equality_between_a_coproduct_and_a_record_is_refused,w_an_equality_between_unrelated_records_is_refused,w_the_census_refuses_a_blatant_mismatch,w_the_one_axis_green_sources_are_admitted"
echo "HEADSHA=$OID START $(date -u +%H:%M:%S)Z"
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver652d 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver652d
echo 51539607552 > /sys/fs/cgroup/ver652d/memory.max
RB=$(cat /sys/fs/cgroup/ver652d/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver652d memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver652d/cgroup.procs; echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w652d
git fetch -q origin +refs/pull/14652/head:refs/verifier/14652d || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14652d)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14652d) STOP"; exit 9; }
git worktree add -q --detach /tmp/w652d $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w652d || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo WRONG_COMMIT; exit 9; }
for f in $F $LCW $LWT $INFER; do [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "STAGED_FAIL $f"; exit 9; }; done
echo "STAGED_OK (4 files == head blobs)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t652d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t652d/release/claim_batch
[ -x $CB ] || { echo NO_CB; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) $(date -u +%H:%M:%S)Z"
cat > /tmp/ih.patch <<'INFERPATCH_EOF'
commit eb0b1dc8d133dedb5cee0eb314aaa3ffbc47502e
Author: gunbai-bot[bot] <289086189+gunbai-bot[bot]@users.noreply.github.com>
Date:   Tue Oct 20 17:18:32 2026 +0000

    == operand judgment: refuse operands that cannot inhabit together (classes C, C-prime) (#14185)
    
    * wip
    
    * Install regenerated stage0 infer mirror for the declared-type fix
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Climb classes A and B: leave the expected-red roster, record the climb
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Fix closer count in the narrowed expected-red chunk
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * wip: refuse == between operands that cannot inhabit together
    
    * Install regenerated stage0 infer mirror for the == operand judgment
    
    * Climb classes C and C-prime: delete the expected-red chunk, record the climb
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Fix doubled comma in the cross-type comparison row receipt
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Declared-type fix: plain-T? payload only; nominal Optional abstains; mirror regenerated on merged main
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Update the climb receipt: plain T? boundary; covering compile now OOMs
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Install regenerated stage0 infer mirror on merged main for the == operand judgment
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Correct the climb receipt to the exact differential result: 57 of 60, three named unresolved
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * == product pair consumes nominal_product_inhabitance_refusal instead of re-deriving it (review 81296)
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Install regenerated stage0 infer mirror
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Install regenerated stage0 infer mirror on merged main
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Install regenerated stage0 infer mirror on main with #14205
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * Install regenerated stage0 infer mirror on main with #14219
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * #14185: retire the stale expected-red citation in the RFM row (review 82876)
    
    The chunk it cited is deleted by this PR; the two REDs stay enrolled as
    permanent regression controls (DESIGN 4b meta-obligation 4).
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>
    
    * chore: regenerate drifted generated artifacts (ci auto-heal)
    
    Regenerated by the stage0 mirror convergence round and the seed-build artifact writer:
    - src/v1/stage0/src/std_decimal.rs
    - src/v1/stage0/src/v1_compiler_infer.rs
    
    Heal-Candidate-Run: 39307716375
    
    * WIP #14185: ledger attribution (14178 climbed A/B; 14185 adds C/C'), drop expected-red preamble
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * #14185: regenerate stage0 mirror v1_compiler_infer.rs from the merged source (round 1 writes it, round 2 converges with 0 stages)
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
    
    * RFM record_value_escapes: credit #14178 for A/B; #14185 adds C/C-prime
    
    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>
    
    ---------
    
    Co-authored-by: Brian Searls <briansearls1@gmail.com>
    Co-authored-by: Claude Sonnet 5.5 <noreply@anthropic.com>
    Co-authored-by: gunbai-bot[bot] <289086189+gunbai-bot[bot]@users.noreply.github.com>

diff --git a/src/v1/stage0/src/v1_compiler_infer.rs b/src/v1/stage0/src/v1_compiler_infer.rs
index 0d545bba71..371e09eaeb 100644
--- a/src/v1/stage0/src/v1_compiler_infer.rs
+++ b/src/v1/stage0/src/v1_compiler_infer.rs
@@ -5435,6 +5435,21 @@ pub fn equality_admission_refusal_diag(
     }
 }
 
+pub fn equality_operands_never_inhabit_together(
+    left: Rc<Node>,
+    right: Rc<Node>,
+    scope: Rc<InferScope>,
+) -> bool {
+    if product_versus_coproduct_head_mismatch(left.clone(), right.clone(), scope.clone()) {
+        true
+    } else {
+        match nominal_product_inhabitance_refusal(left.clone(), right.clone(), scope.clone()) {
+            Some(_) => true,
+            std::option::Option::None => false,
+        }
+    }
+}
+
 pub fn equality_admission_diags(
     op: BinOp,
     left_typed: Rc<Node>,
@@ -5515,18 +5530,26 @@ pub fn equality_admission_diags(
                             )])
                         }
                     } else {
-                        match equality_operand_admission(
+                        if equality_operands_never_inhabit_together(
                             lt.clone(),
+                            rt.clone(),
                             scope.clone(),
-                            v1_rt::rc_empty_map::<String, bool>(),
-                            0,
                         ) {
-                            Some(r) => Rc::new(vec![crate::v1_std_core::make_error_node(
-                                equality_admission_refusal_diag(r.clone(), span.clone()),
+                            Rc::new(vec![type_mismatch_error(
+                                crate::v1_compiler_infer_types::node_type_shape(
+                                    lt.clone(),
+                                    source_indices.clone(),
+                                ),
+                                crate::v1_compiler_infer_types::node_type_shape(
+                                    rt.clone(),
+                                    source_indices.clone(),
+                                ),
+                                span.clone(),
                                 scope.module_name.clone(),
-                            )]),
-                            std::option::Option::None => match equality_operand_admission(
-                                rt.clone(),
+                            )])
+                        } else {
+                            match equality_operand_admission(
+                                lt.clone(),
                                 scope.clone(),
                                 v1_rt::rc_empty_map::<String, bool>(),
                                 0,
@@ -5535,8 +5558,19 @@ pub fn equality_admission_diags(
                                     equality_admission_refusal_diag(r.clone(), span.clone()),
                                     scope.module_name.clone(),
                                 )]),
-                                std::option::Option::None => Rc::new(vec![]),
-                            },
+                                std::option::Option::None => match equality_operand_admission(
+                                    rt.clone(),
+                                    scope.clone(),
+                                    v1_rt::rc_empty_map::<String, bool>(),
+                                    0,
+                                ) {
+                                    Some(r) => Rc::new(vec![crate::v1_std_core::make_error_node(
+                                        equality_admission_refusal_diag(r.clone(), span.clone()),
+                                        scope.module_name.clone(),
+                                    )]),
+                                    std::option::Option::None => Rc::new(vec![]),
+                                },
+                            }
                         }
                     }
                 }

INFERPATCH_EOF
run2 () { # out fns wetflag entry
  local extra=""; [ "$3" = "wet" ] && extra="--wet"
  timeout 1500 $CB --claim-run $extra --source-root dag --source-root src/v2 --entry "$4" --functions "$2" > "$1" 2>&1; echo $?
}
echo "=== A0. fold claims = head-built compile+audit of the schedule file, wet ==="
FOLDN=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $LCW | sed 's/^test fn //' | sort | paste -sd,)
echo "  fold names: $FOLDN"
rc=$(run2 /tmp/c652d_fold.out "$FOLDN" wet $LCW)
echo "  fold rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652d_fold.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652d_fold.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c652d_fold.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c652d_fold.out | head -4 | sed 's/^/    V| /' | cut -c1-200
echo "=== A. baseline 4 claims wet, by name ==="
rc=$(run2 /tmp/c652d_base.out "$ALL4" wet $F)
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652d_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652d_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c652d_base.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c652d_base.out | sed 's/^/    V| /' | cut -c1-150
echo "=== B. MUTATION: revert #14185 infer hunk ==="
cp $INFER /tmp/c652d_infer.orig; HB=$(sha $INFER)
git apply -R /tmp/ih.patch; echo "  APPLY_R rc=$?"
cmp -s $INFER /tmp/c652d_infer.orig && { echo "  !! MUTATION_NOOP -- STOP"; exit 0; } || echo "  mutated $(sha $INFER|cut -c1-12) was ${HB:0:12}"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
rc=$(run2 /tmp/c652d_mutw.out "$ALL4" wet $F); echo "  MUT-wet rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652d_mutw.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652d_mutw.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c652d_mutw.out | sed 's/^/    V| /' | cut -c1-150
rc=$(run2 /tmp/c652d_muth.out "$ALL4" hermetic $F); echo "  MUT-herm rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652d_muth.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652d_muth.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c652d_muth.out | sed 's/^/    V| /' | cut -c1-150
echo "=== C. restore byte-exact + rebuild + POST ==="
cp /tmp/c652d_infer.orig $INFER
cmp -s $INFER /tmp/c652d_infer.orig && echo "  RESTORE_BYTE_EXACT=YES" || { echo "  RESTORE_BYTE_EXACT=NO -- STOP"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
rc=$(run2 /tmp/c652d_post.out "$ALL4" wet $F)
echo "  POST-wet rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652d_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652d_post.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c652d_post.out | sed 's/^/    V| /' | cut -c1-150
git status --porcelain | grep -v '^??' | sed 's/^/    PORCELAIN| /'
echo DONE_652_D
cd /root/workspace/repo-root
git worktree remove --force /tmp/w652d >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
