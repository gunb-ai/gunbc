set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14630 FIX dispatch (previous arms: tpb 40/40 PASS both revs GOOD; bla arm ran a FABRICATED
# claim name from the brief -> rc=2 zero verdicts, void; consumer list brief-stale -> MISSING;
# eval_steps extraction empty -> re-grep raw [witness] lines).
HEADSHA=ca960003d3dd367d354a0aec973fdc9d9b496c25
BASESHA=647d770c78c3489d5430dfeb3f3399e3d9cd9312
F=src/v2/test/claim/type_param_binder_frame.dag
BLA=src/v2/test/claim/body_let_annotation.dag
BLA_CLAIM=bla_optional_at_a_required_let_refuses_as_a_return_and_a_field_do
CONSUMERS="src/v2/test/claim/algebra_operator_derivation src/v2/test/claim/binder_default_judgment src/v2/test/claim/body_cast_node src/v2/test/claim/body_let_annotation src/v2/test/claim/body_lowering/plain_type_decl_lowering src/v2/test/claim/body_type_annotation_refusal src/v2/test/claim/execution/infer_declaration_formation src/v2/test/claim/normalize/service_spine src/v2/test/claim/parse/population_grammar_gaps src/v2/test/claim/parse/record_field_pun src/v2/test/claim/value_base_projection src/v2/test/infer_facts/key_collision_ref_and_int"
echo "FIX START $(date -u +%H:%M:%S)Z HEADSHA=$HEADSHA"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w630h /tmp/w630b
git fetch -q origin +refs/pull/14630/head:refs/verifier/14630b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14630b)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14630b) STOP"; exit 9; }
git worktree add -q --detach /tmp/w630h $HEADSHA || { echo WT_H_FAIL; exit 9; }
git worktree add -q --detach /tmp/w630b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
( cd /tmp/w630h && CARGO_TARGET_DIR=/tmp/w630h/.t630h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBH=/tmp/w630h/.t630h/release/claim_batch; [ -x $CBH ] || { echo NO_CBH; exit 9; }
echo "CBH_SHA=$(sha $CBH | cut -c1-12)"
( cd /tmp/w630b && CARGO_TARGET_DIR=/tmp/w630b/.t630b cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBB=/tmp/w630b/.t630b/release/claim_batch; [ -x $CBB ] || { echo NO_CBB; exit 9; }
echo "CBB_SHA=$(sha $CBB | cut -c1-12)"
runw () { # entry cb out fns
  timeout 1200 $2 --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$4" > "$3" 2>&1; echo $?
}
for REV in h b; do
  [ $REV = h ] && { WT=/tmp/w630h; CB=$CBH; } || { WT=/tmp/w630b; CB=$CBB; }
  cd $WT || exit 9
  echo "=== BLA real claim @ rev=$REV (expect PASS both; PR: base refused for the WRONG reason -> check FAIL at base) ==="
  rc=$(runw $BLA $CB /tmp/c630b_bla_$REV.out "$BLA_CLAIM"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630b_bla_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630b_bla_$REV.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c630b_bla_$REV.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c630b_bla_$REV.out | sed 's/^/    V| /' | cut -c1-160
  grep -a 'witness\]' /tmp/c630b_bla_$REV.out | head -2 | sed 's/^/    W| /' | cut -c1-200
done
echo "=== eval_steps lines for the 3 assembly-heavy claims, both revs ==="
for REV in h b; do
  [ $REV = h ] && { WT=/tmp/w630h; CB=$CBH; } || { WT=/tmp/w630b; CB=$CBB; }
  cd $WT || exit 9
  rc=$(runw $F $CB /tmp/c630b_tpb3_$REV.out "w_tpb_fingerprint_stable_across_rounds,w_tpb_checkers_reject_toplevel_tvar,w_tpb_checkers_reject_illegal_shape"); echo "  rev=$REV rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630b_tpb3_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630b_tpb3_$REV.out)"
  grep -a 'witness\]' /tmp/c630b_tpb3_$REV.out | sed 's/^/    W| /' | cut -c1-200
  echo "  (PR claims: head 16044/16086/16242; base 67538/67580/67736)"
done
echo "=== CONSUMER checks at HEAD (paths from the TREE at run time) ==="
cd /tmp/w630h || exit 9
for e in $CONSUMERS; do
  [ -f "$e.dag" ] || { echo "  $e MISSING(disclose)"; continue; }
  n=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $e.dag | sed 's/^test fn //' | head -1)
  [ -z "$n" ] && { echo "  $e NO_TEST_FN(disclose)"; continue; }
  rc=$(runw $e.dag $CBH /tmp/c630b_cons.out "$n")
  echo "  $(basename $e) first=$n rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630b_cons.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630b_cons.out) refused=$(grep -aci 'no declaration is indexed\|variant .* not found\|resolve failed' /tmp/c630b_cons.out)"
done
echo "=== INTERMEDIATE RED @ 23a8a95130 (parameterized tpb_assemble, witness NOT yet injected): expect the claim FAILs ==="
MID=23a8a95130
git cat-file -e "$MID^{commit}" 2>/dev/null || { echo "MID_MISS fetch"; git fetch -q origin $MID || echo MID_FETCH_FAIL; }
if git cat-file -e "$MID^{commit}" 2>/dev/null; then
  rm -rf /tmp/w630m; git worktree add -q --detach /tmp/w630m $MID || echo WT_M_FAIL
  if [ -d /tmp/w630m ]; then
    cd /tmp/w630m || exit 9
    echo "  MID_HEAD=$(git rev-parse HEAD)"
    grep -n 'tpb_assemble_ingest(ingest:' $BLA | head -1 | sed 's/^/    BLA-arm| /' | cut -c1-120
    grep -c 'std_optional' $F | sed 's/^/    frame_std_optional_hits(expect 0)=/;s/^/    MID| /'
    ( CARGO_TARGET_DIR=/tmp/w630m/.t630m cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
    CBM=/tmp/w630m/.t630m/release/claim_batch
    rc=$(runw $BLA $CBM /tmp/c630b_bla_mid.out "$BLA_CLAIM"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630b_bla_mid.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630b_bla_mid.out)"
    grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c630b_bla_mid.out | sed 's/^/    V| /' | cut -c1-160
    grep -aE 'error|refus|not found' /tmp/c630b_bla_mid.out | head -3 | sed 's/^/    E| /' | cut -c1-190
  fi
else echo "  MID_UNAVAILABLE (disclose)"; fi
echo DONE_630_B
cd /root/workspace/repo-root
git worktree remove --force /tmp/w630h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w630b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
