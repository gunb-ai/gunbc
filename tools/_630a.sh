set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14630 std.optional witness as a parameter. Manager scope: (1) 40/40 tpb claims at BOTH heads,
# (2) the eval_steps drop on the 3 assembly-heavy claims, (3) the bla T? RED (base refuses, head
# green), (4) the 12 consumer files build at head. Statics (local): delta base..head = 1 file
# (src/v2/test/claim/type_param_binder_frame.dag, +50/-5); tpb_assemble/tpb_accepts defined there;
# 12 consumer files with paren-attached calls outside the frame file.
HEADSHA=ca960003d3dd367d354a0aec973fdc9d9b496c25   # gh pr view 14630 --json headRefOid
BASESHA=647d770c78c3489d5430dfeb3f3399e3d9cd9312   # git merge-base head origin/main (fetched)
F=src/v2/test/claim/type_param_binder_frame.dag
BLA=src/v2/test/claim/body_let_annotation.dag
CONSUMERS="src/v2/compiler/type_scheme_classify src/v2/test/claim/stage0_kernel_type_spelling_witness src/v2/test/claim/body_let_annotation src/v2/compiler/kernel_type_spelling src/v2/test/claim/canonical_type_param_source_witness src/v2/test/claim/kernel_type_spelling_capture_witness src/v2/compiler/type_param_shape src/v2/compiler/shape_kernel_type_spelling src/v2/test/claim/stage0_body_let_annotation_witness src/v2/compiler/type_class_of_value src/v2/compiler/kernel_type_eraser src/v2/compiler/type_argument_shape"
echo "HEADSHA=$HEADSHA BASESHA=$BASESHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w630h /tmp/w630b
git fetch -q origin +refs/pull/14630/head:refs/verifier/14630 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14630)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14630) STOP"; exit 9; }
git cat-file -e "$BASESHA^{commit}" || git fetch -q origin $BASESHA
git worktree add -q --detach /tmp/w630h $HEADSHA || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w630b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch at HEAD ==="
( cd /tmp/w630h && CARGO_TARGET_DIR=/tmp/w630h/.t630h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w630h/.t630h/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "CBH_SHA=$(sha $CBH | cut -c1-12)"
echo "=== build claim_batch at BASE ==="
( cd /tmp/w630b && CARGO_TARGET_DIR=/tmp/w630b/.t630b cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBB=/tmp/w630b/.t630b/release/claim_batch; [ -x $CBB ] || { echo NO_CBB; exit 9; }
echo "CBB_SHA=$(sha $CBB | cut -c1-12)"
runw () { # entry cb out fns wt  (source roots: src/v2 for v2 trees, dag too)
  local extra=""; [ "$5" = "wet" ] && extra="--wet"
  timeout 1500 $2 --claim-run --source-root dag --source-root src/v2 --source-root src/v1 $extra --entry "$1" --functions "$4" > "$3" 2>&1; echo $?
}
steps () { # claim-name out -> eval_steps value
  grep -a "^.*$1" "$2" | grep -aoE 'eval_steps=[0-9]+' | head -1
}
for REV in h b; do
  [ $REV = h ] && { WT=/tmp/w630h; CB=$CBH; } || { WT=/tmp/w630b; CB=$CBB; }
  cd $WT || exit 9
  echo "=== TPB 40-by-name @ rev=$REV ==="
  FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | sort | paste -sd,)
  N=$(echo $FNS | tr ',' '\n' | wc -l); echo "  fns_from_file=$N"
  rc=$(runw $F $CB /tmp/c630_tpb_$REV.out "$FNS" hermetic)
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630_tpb_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630_tpb_$REV.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c630_tpb_$REV.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c630_tpb_$REV.out | sed 's/^/    V| /' | cut -c1-160
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c630_tpb_$REV.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  for c in w_tpb_fingerprint_stable_across_rounds w_tpb_checkers_reject_toplevel_tvar w_tpb_checkers_reject_illegal_shape; do
    echo "  steps[$REV,$c]=$(steps $c /tmp/c630_tpb_$REV.out) (PR claims head 16044/16086/16242, base 67538/67580/67736)"
  done
  echo "=== BLA T? claim @ rev=$REV ==="
  rc=$(runw $BLA $CB /tmp/c630_bla_$REV.out "bla_optional_at_a_required_let" hermetic)
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630_bla_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630_bla_$REV.out)"
  grep -aE '^(PASS|FAIL) ' /tmp/c630_bla_$REV.out | sed 's/^/    V| /' | cut -c1-140
  echo "  T?-refusal-cause=$(grep -aci "T'?\|'? is not supported\|nested_optional\|not supported at a parameter position" /tmp/c630_bla_$REV.out)"
  grep -aE 'refus|Refus|not supported|error' /tmp/c630_bla_$REV.out | head -4 | sed 's/^/    E| /' | cut -c1-190
done
echo "=== CONSUMER BUILD CHECK at HEAD: first test fn of each consumer entry ==="
cd /tmp/w630h || exit 9
for e in $CONSUMERS; do
  [ -f "$e.dag" ] || { echo "  $e MISSING(disclose)"; continue; }
  n=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $e.dag | sed 's/^test fn //' | head -1)
  [ -z "$n" ] && { echo "  $e NO_TEST_FN(disclose)"; continue; }
  rc=$(runw $e.dag $CBH /tmp/c630_cons.out "$n" hermetic)
  echo "  $e first=$n rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630_cons.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630_cons.out) refused=$(grep -aci 'refus\|not found\|no declaration' /tmp/c630_cons.out)"
done
echo DONE_630_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w630h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w630b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
