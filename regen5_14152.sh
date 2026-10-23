#!/bin/bash
# Declared repair route for a conflicted generated mirror, executed IN ONE DISPATCH.
#   1. build claim_executor from the CURRENT installed mirrors (main's, pre-fold)
#   2. --required-regen: writes the candidate tree under target/stage0-regen-candidate/src
#      BEFORE judging, prints the files it names; install them
#   3. because this changes the emitter's OWN pipeline (v1.compiler infer), rebuild from the
#      installed seed and re-run until it installs nothing -- convergence
# Every round prints refusal lines verbatim (no filtering), and every file a round moves is
# named by a sha256 manifest diff (the runner's 'git status' is useless: its checkout HEAD is a
# different commit than the synced working tree, so it listed 270 files main moved ahead by).
# NO git checkout / git clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
CAND=target/stage0-regen-candidate/src
manifest() { git ls-files -z "$SEED" | xargs -0 sha256sum 2>/dev/null | sort > "$1"; }
round() {
  local tag=$1
  CAND=target/stage0-regen-candidate/src
  manifest /tmp/man_before_$tag
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_$tag.log 2>&1
  echo "=== ROUND $tag rc=$?"
  grep -E "^required-regen:" /tmp/rr_$tag.log | head -6 | cut -c1-300
  echo "--- refusals/failures, verbatim (first 40):"
  grep -iE "refus|fail|mismatch|diagnostic|error" /tmp/rr_$tag.log | grep -v "^required-regen: elapsed" | head -40 | cut -c1-230
  manifest /tmp/man_after_$tag
  echo "--- files this round moved:"
  comm -3 /tmp/man_before_$tag /tmp/man_after_$tag | head -30
  echo "--- moved count=$(comm -3 /tmp/man_before_$tag /tmp/man_after_$tag | wc -l)"
  # discover the candidate tree: the host names it itself, do not hard-guess its layout
  if [ ! -d "$CAND" ]; then
    found=$(find target -maxdepth 2 -type d -name "*regen-candidate*" 2>/dev/null | head -1)
    if [ -n "$found" ]; then echo "  (candidate tree found at $found)"; CAND="$found"; [ -d "$CAND/src" ] && CAND="$CAND/src"; fi
  fi
  local inst=0
  if [ -d "$CAND" ]; then
    for f in $(ls "$CAND" 2>/dev/null); do
      if [ -f "$SEED/$f" ]; then
        if ! cmp -s "$CAND/$f" "$SEED/$f"; then cp "$CAND/$f" "$SEED/$f"; echo "  installed $f"; inst=$((inst+1)); fi
      else
        echo "  NEW candidate file, not in seed: $f"
      fi
    done
  fi
  echo "--- installed $inst"
  [ "$inst" -gt 0 ]
}
echo "start: seedfold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs) dag_relation=$(grep -c generic_types_unify src/v1/04_infer.dag) dag_tvleaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag)"
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -3
# NOTE: capture the status into a variable -- an assignment or echo between the call and the
# test clobbers $?, and a loop that always ran its rounds would silently cost 3 full
# self-compiles (~4 min each).
round r1; rc=$?
i=2
while [ "$rc" -eq 0 ] && [ $i -le 4 ]; do
  echo "### convergence round $i (the emitter's own pipeline changed)"
  cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -3
  round r$i; rc=$?
  i=$((i+1))
done
echo "=== AFTER: seed carries relation=$(grep -c generic_types_unify $SEED/v1_compiler_infer.rs) formalGuard=$(grep -c binding_is_unsettled_formal_annotation $SEED/v1_compiler_infer.rs) tvLeaf=$(grep -c binding_has_type_variable_leaf $SEED/v1_compiler_infer.rs) fold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)  (main's base fold=0)"
cp /tmp/rr_*.log . 2>/dev/null
ls rr_*.log 2>/dev/null
# Transport: the runner's edits do NOT come back, so the 4 regenerated mirrors are shipped as a
# base64 tar for local extraction. Only the mirrors (the whole seed dir is tens of MB).
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
echo "=== BEGIN-TAR ==="
( cd $SEED && tar -c $MIRRORS 2>/dev/null | base64 -w0 )
echo "=== END-TAR ==="
for f in $MIRRORS; do echo "sha $f $(sha256sum $SEED/$f | cut -c1-16) $(wc -c < $SEED/$f) bytes"; done
echo REGEN5_DONE
