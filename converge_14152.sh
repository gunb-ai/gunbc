#!/bin/bash
# CONVERGE-ONLY, and the TRANSPORT COMES FIRST IN THE SCRIPT ORDER OF NECESSITY: the last dispatch
# ran out of its 45-minute remote cap in phase 3, after phases 1-2 had spent 40 minutes, and the
# base64 tar of the converged seed sat AFTER phase 3 -- so the artifact was lost while the answer was
# kept. Here the only work is the convergence, and the tar is emitted as soon as it converges.
#
# What is already measured (previous dispatch, three_14152.sh):
#  P1 clean main: --required-regen rc=1, ZERO hard diagnostics, first_generation_equal=false,
#     'FAIL generated surface drift: v1_compiler_infer.rs' -- i.e. main's committed mirror is not what
#     main's own .dag emits. That is the pre-existing X2-class drift, not a refusal.
#  P2 my .dag + main's mirrors, after my 3 fixes: round 2 rc=0, ZERO hard diagnostics. All 6 gone.
#     Rounds installed v1_compiler_infer.rs, v1_compiler_infer_types.rs,
#     v1_tests_claim_generic_identity_census.rs, and reported TWO candidate files not in the seed at
#     all (cli_run, module_path_index) -- reported, not committed.
# NO git checkout / git clean. Restores by cp at the end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
# NOTE: keep/my_04_infer.dag must be RESTAGED locally before each dispatch. A run that used a copy
# staged at 03:25 emitted a seed WITHOUT the fixes made at 06:20-06:24, and printed a clean rc=0 --
# a converged seed that measured the old source. Verify the marker counts below name what you changed.
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "start: dag relation=$(grep -c generic_types_unify src/v1/04_infer.dag) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag) seedfold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs) (want 0 = main's base)"
manifest() { git ls-files -z "$SEED" | xargs -0 sha256sum 2>/dev/null | sort > "$1"; }
i=1
while [ $i -le 4 ]; do
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  manifest /tmp/cm_before_$i
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/cm_$i.log 2>&1
  rc=$?
  echo "=== ROUND $i rc=$rc"
  grep -a "^required-regen:" /tmp/cm_$i.log | grep -aE "first_generation_equal|FAIL|drift" | head -4 | cut -c1-220
  echo "--- hard diagnostics:"; grep -acE "^type mismatch|^call shape|arm decides" /tmp/cm_$i.log
  grep -aE "^type mismatch|^call shape|arm decides" /tmp/cm_$i.log | head -8 | cut -c1-230
  manifest /tmp/cm_after_$i
  echo "--- seed files this round moved:"; comm -3 /tmp/cm_before_$i /tmp/cm_after_$i | head -10
  [ -d "$CAND" ] || { echo "no candidate tree"; break; }
  inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    if [ -f "$SEED/$f" ]; then
      cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; echo "  installed $f"; inst=$((inst+1)); }
    else
      echo "  CANDIDATE FILE NOT IN SEED (reported, not committed): $f"
    fi
  done
  echo "ROUND $i installed=$inst"
  [ "$inst" -eq 0 ] && break
  i=$((i+1))
done
echo "=== CONVERGED seed: relation=$(grep -c generic_types_unify $SEED/v1_compiler_infer.rs) formalGuard=$(grep -c binding_is_unsettled_formal_annotation $SEED/v1_compiler_infer.rs) tvLeaf=$(grep -c binding_has_type_variable_leaf $SEED/v1_compiler_infer.rs) fold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)"
for f in $MIRRORS; do echo "sha256 $(sha256sum $SEED/$f | cut -c1-16) $(wc -c < $SEED/$f) $f"; done
echo "=== BEGIN-TAR ==="
( cd $SEED && tar -c $MIRRORS 2>/dev/null | base64 -w0 )
echo "=== END-TAR ==="
cp /tmp/cm_1.log cm1.log 2>/dev/null; cp /tmp/cm_2.log cm2.log 2>/dev/null
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: $(for f in $MIRRORS; do echo -n "$(basename $f)=$(sha256sum $SEED/$f|cut -c1-10) "; done)"
echo CONVERGE_DONE
