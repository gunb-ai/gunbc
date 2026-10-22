set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14668 RFM row 'the emitter renders an ambiguous literal silently' + two emit-route refusal
# claims enrolled. Manager scope: the 2 claims PASS at head; FAIL when the ambiguity diagnostic
# is SUPPRESSED; row parses + roster-joins; enrolment rows present. Suppression = the two
# 'count > 1' refusal guards in src/v1/05_emit_rust.dag (lines ~15905 single_candidates,
# ~15948 resolution_candidates) made unreachable -> ambiguous literals fall to tuple pass-through
# and the expected compile_error! strings never appear. Emitter is COMPILED INTO claim_batch
# (05_emit_rust.dag embedded) -> rebuild needed for the mutation AND after restore.
HEADSHA=96efb37a7dc62919c198c60d193b0446456b91b6   # gh pr view 14668 --json headRefOid
WIT=dag/test/claim/record_emission_candidate_set_witness.dag
PAR=src/v2/test/claim/recurring_failure_mode_parity.dag
RFM=dag/gunbc/recurring_failure_mode/registry.dag
TERM=src/v2/workflow/local_repo_wet_terminal.dag
EMIT=src/v1/05_emit_rust.dag
CLAIMS=single_field_ambiguous_literal_emits_the_refusal,multi_field_ambiguous_literal_refuses_with_located_diagnostic
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w668h
git fetch -q origin +refs/pull/14668/head:refs/verifier/14668 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14668)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14668) STOP"; exit 9; }
git worktree add -q --detach /tmp/w668h $HEADSHA || { echo WT_H_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w668h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "=== build claim_batch at HEAD ==="
( CARGO_TARGET_DIR=/tmp/w668h/.t668h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=/tmp/w668h/.t668h/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12)"
echo "=== A. the two claims PASS at head (author's wet route, names verified in file) ==="
for c in single_field_ambiguous_literal_emits_the_refusal multi_field_ambiguous_literal_refuses_with_located_diagnostic; do
  grep -q "^test fn $c$" $WIT || echo "  !! NAME_NOT_IN_FILE $c"
done
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$CLAIMS" > /tmp/c668_pass.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c668_pass.out) FAIL=$(grep -ac '^FAIL ' /tmp/c668_pass.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c668_pass.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c668_pass.out | sed 's/^/    V| /' | cut -c1-150
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c668_pass.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== A2. the two NON-refusal siblings also PASS (mutation must break only the two) ==="
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct" > /tmp/c668_sib.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c668_sib.out) FAIL=$(grep -ac '^FAIL ' /tmp/c668_sib.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c668_sib.out | sed 's/^/    V| /' | cut -c1-150
echo "=== B. RFM row (updated in this PR) + enrolment statics ==="
ls -la dag/gunbc/recurring_failure_mode/anonymous_record_resolved_by_field_names_guesses_on_ambiguity.dag | sed 's/^/    R| /'
grep -c 'C4-G row 10' dag/gunbc/recurring_failure_mode/anonymous_record_resolved_by_field_names_guesses_on_ambiguity.dag | sed 's/^/  B| new_receipt_hits(expect 1)=/'
grep -c '11d6c58f7525' dag/gunbc/recurring_failure_mode/anonymous_record_resolved_by_field_names_guesses_on_ambiguity.dag | sed 's/^/  B| entry_receipt_sha_hits(expect 1)=/'
# roster join (rostered_row_join.dag:69 RecurringFailureModeRows; declared_type_name RecurringFailureMode)
grep -n 'RecurringFailureModeRows' dag/gunbc/rostered_row_join.dag | head -3 | sed 's/^/    RJ| /' | cut -c1-170
echo "  population rows:"; ls dag/gunbc/recurring_failure_mode/ | sed 's/^/    POP| /'
echo "  row identity declared once (count 'identity:' =1) and unique:"
grep -c 'identity:' dag/gunbc/recurring_failure_mode/anonymous_record_resolved_by_field_names_guesses_on_ambiguity.dag | sed 's/^/  B| identity_field_count=/;s/^/  /'
# enrolment rows for the two claims
grep -B2 -A6 'single_field_ambiguous_literal_emits_the_refusal\|multi_field_ambiguous_literal_refuses_with_located_diagnostic' $TERM | sed 's/^/    ROW| /' | cut -c1-170
echo "=== C. SUPPRESSION MUTATION on 05_emit_rust.dag (2 anchors) -> the two claims FAIL ==="
cp $EMIT /tmp/c668_e.orig; HB=$(sha $EMIT)
python3 - <<'PY'
p="src/v1/05_emit_rust.dag"
s=open(p).read()
a1="} else if single_candidates |> count > 1 {"
a2="} else if resolution_candidates |> count > 1 {"
assert s.count(a1)==1 and s.count(a2)==1, "anchors %d/%d -- SKIP LOUDLY" % (s.count(a1), s.count(a2))
s=s.replace(a1,"} else if single_candidates |> count > 999999 {").replace(a2,"} else if resolution_candidates |> count > 999999 {")
open(p,'w').write(s)
print("SUPPRESSED anchors=1/1")
PY
cmp -s $EMIT /tmp/c668_e.orig && { echo "  !! MUTATION_NOOP"; echo DONE_668_A; exit 0; }
echo "  mutated sha=$(sha $EMIT|cut -c1-12); rebuilding (emitter compiled into claim_batch)..."
( CARGO_TARGET_DIR=/tmp/w668h/.t668h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$CLAIMS" > /tmp/c668_red.out 2>&1; rc=$?
echo "  RED rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c668_red.out) FAIL=$(grep -ac '^FAIL ' /tmp/c668_red.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c668_red.out | sed 's/^/    V| /' | cut -c1-150
echo "  siblings-while-suppressed:"
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct" > /tmp/c668_sibred.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c668_sibred.out) FAIL=$(grep -ac '^FAIL ' /tmp/c668_sibred.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c668_sibred.out | sed 's/^/    V| /' | cut -c1-150
echo "=== restore + rebuild ==="
cp /tmp/c668_e.orig $EMIT
cmp -s $EMIT /tmp/c668_e.orig && [ "$(sha $EMIT)" = "$HB" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO"
( CARGO_TARGET_DIR=/tmp/w668h/.t668h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$CLAIMS" > /tmp/c668_post.out 2>&1; rc=$?
echo "  POST-restore rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c668_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c668_post.out)"
git status --porcelain | grep -v '^??' | sed 's/^/    PORCELAIN| /'
echo DONE_668_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w668h >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
