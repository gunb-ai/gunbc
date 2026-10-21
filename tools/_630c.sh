set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14630 FIX dispatch. Prior arms GOOD: tpb 40/40 PASS both revs. Void arms (disclosed): bla ran a
# fabricated name (rc=2 zero verdicts); consumer list was brief-stale; steps grep pattern wrong.
# Body truth: eval_steps claims = tpb_type_param_shadowing_a_visible_name_refuses (296785->184966),
# tpb_same_spelling_in_two_declarations_does_not_alias (445886->326327); bla claim
# bla_optional_at_a_required_let_refuses_as_a_return_and_a_field_do PASS both (1,276,563 vs
# 1,276,593); RED = witness source replaced by a module lacking Optional -> same claim FAILS.
HEADSHA=ca960003d3dd367d354a0aec973fdc9d9b496c25
BASESHA=647d770c78c3489d5430dfeb3f3399e3d9cd9312
F=src/v2/test/claim/type_param_binder_frame.dag
BLA=src/v2/test/claim/body_let_annotation.dag
BLA_CLAIM=bla_optional_at_a_required_let_refuses_as_a_return_and_a_field_do
CONSUMERS="src/v2/test/claim/algebra_operator_derivation src/v2/test/claim/binder_default_judgment src/v2/test/claim/body_cast_node src/v2/test/claim/body_let_annotation src/v2/test/claim/body_lowering/plain_type_decl_lowering src/v2/test/claim/body_type_annotation_refusal src/v2/test/claim/execution/infer_declaration_formation src/v2/test/claim/normalize/service_spine src/v2/test/claim/parse/population_grammar_gaps src/v2/test/claim/parse/record_field_pun src/v2/test/claim/value_base_projection src/v2/test/infer_facts/key_collision_ref_and_int"
echo "FIX2 START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w630h /tmp/w630b
git fetch -q origin +refs/pull/14630/head:refs/verifier/14630c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14630c)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14630c) STOP"; exit 9; }
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
echo "=== BLA real claim, BOTH revs (body: PASS both, 1276563 head / 1276593 base) ==="
for REV in h b; do
  [ $REV = h ] && { WT=/tmp/w630h; CB=$CBH; } || { WT=/tmp/w630b; CB=$CBB; }
  cd $WT || exit 9
  rc=$(runw $BLA $CB /tmp/c630c_bla_$REV.out "$BLA_CLAIM"); echo "  rev=$REV rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_bla_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_bla_$REV.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c630c_bla_$REV.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c630c_bla_$REV.out | sed 's/^/    V| /' | cut -c1-160
  grep -a 'witness\]' /tmp/c630c_bla_$REV.out | sed 's/^/    W| /' | cut -c1-200
done
echo "=== eval_steps, the TWO body-named claims, both revs ==="
for REV in h b; do
  [ $REV = h ] && { WT=/tmp/w630h; CB=$CBH; } || { WT=/tmp/w630b; CB=$CBB; }
  cd $WT || exit 9
  rc=$(runw $F $CB /tmp/c630c_steps_$REV.out "tpb_type_param_shadowing_a_visible_name_refuses,tpb_same_spelling_in_two_declarations_does_not_alias"); echo "  rev=$REV rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_steps_$REV.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_steps_$REV.out)"
  grep -a 'witness\]' /tmp/c630c_steps_$REV.out | sed 's/^/    W| /' | cut -c1-210
  echo "  body claims: shadowing 296785(base)->184966(head); same_spelling 445886->326327"
done
echo "=== RED (PR's own): mutate tpb_std_optional_witness source to lack Optional -> bla claim FAILs ==="
cd /tmp/w630h || exit 9
cp $F /tmp/c630c_f.orig
A=$(grep -c 'carried: "module std.optional' $F); echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip"; echo DONE_630_C; exit 0; }
python3 - <<'PY'
p="src/v2/test/claim/type_param_binder_frame.dag"
s=open(p).read()
old='carried: "module std.optional\\n\\ntype Optional<T>\\n  = Absent\\n  | Present { value: T }\\n"'
n=s.count(old)
assert n==1, "anchor count %d -- skip mutation loudly" % n
open(p,'w').write(s.replace(old,'carried: "module std.optional\\n"'))
print("MUTATED_BY_PY anchor=1")
PY
cmp -s $F /tmp/c630c_f.orig && echo "  !! MUTATION_NOOP (disclose)" || echo "  mutated: $(sha $F | cut -c1-12)"
rc=$(runw $BLA $CBH /tmp/c630c_red.out "$BLA_CLAIM"); echo "  RED rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_red.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_red.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c630c_red.out | sed 's/^/    V| /' | cut -c1-160
grep -aE 'ResolveGeneratedReferenceDeclarationAbsent|not found|refus' /tmp/c630c_red.out | head -3 | sed 's/^/    E| /' | cut -c1-190
echo "  control-while-mutated: a NON-T? tpb claim should still PASS (witness only matters for T?):"
CTL=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $F | sed 's/^test fn //' | grep -v optional | head -1)
echo "  control claim from file: $CTL"
rc=$(runw $F $CBH /tmp/c630c_ctlmut.out "$CTL"); echo "    rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_ctlmut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_ctlmut.out)"
cp /tmp/c630c_f.orig $F
cmp -s $F /tmp/c630c_f.orig && echo "  RESTORE_IDENTICAL=YES" || echo "  RESTORE_IDENTICAL=NO"
rc=$(runw $BLA $CBH /tmp/c630c_post.out "$BLA_CLAIM"); echo "  POST-restore rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_post.out)"
git status --porcelain -- $F | sed 's/^/    PORCELAIN| /'
echo "=== CONSUMER first-claims at HEAD (PR did NOT run these; my extra) ==="
for e in $CONSUMERS; do
  [ -f "$e.dag" ] || { echo "  $e MISSING(disclose)"; continue; }
  n=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $e.dag | sed 's/^test fn //' | head -1)
  [ -z "$n" ] && { echo "  $e NO_TEST_FN(disclose)"; continue; }
  rc=$(runw $e.dag $CBH /tmp/c630c_cons.out "$n")
  echo "  $(basename $e) first=$n rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c630c_cons.out) FAIL=$(grep -ac '^FAIL ' /tmp/c630c_cons.out) refused=$(grep -aci 'no declaration is indexed\|variant .* not found\|resolve failed' /tmp/c630c_cons.out)"
done
echo DONE_630_C
cd /root/workspace/repo-root
git worktree remove --force /tmp/w630h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w630b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
