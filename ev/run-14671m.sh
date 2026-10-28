set -u
ROOT=$(pwd)
git fetch -q origin c4g-ledger-enrol 2>&1 | tail -2
GOT=$(git rev-parse FETCH_HEAD)
echo "FETCHED_HEAD $GOT (expected 0b1884cdbbf2b6420106b2e2ec3a29ae3acc968d)"
git -C "$ROOT" worktree add -q --detach /tmp/coB FETCH_HEAD && cd /tmp/coB && echo "CO_HEAD $(git rev-parse HEAD)"
echo "CHECKOUT_TREE $(git -C "$ROOT" rev-parse FETCH_HEAD^{tree})"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^(error|warning: build)|-->|could not|No such" | head -12
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-200; }
echo "== wet-schedule rows of this PR present at head: $(grep -c 'module_path: "test.claim.ledger_row_coherence"' src/v2/workflow/local_repo_wet_terminal.dag) (expect 18)"
echo '== head claims (18)'; cb dag/test/claim/ledger_row_coherence.dag a_fully_rendered_ledger_is_coherent,a_withheld_row_is_named_as_missing,the_rung_drop_ledger_is_covered_without_a_projection_index,a_non_ledger_artifact_cannot_report_coherent,the_gate_diagnostic_names_the_missing_identity,a_difference_that_drops_no_row_is_not_reported_incoherent,a_rendered_declaration_parses_back_to_the_same_identities,a_coherent_artifact_declares_no_repair,a_declaration_matching_the_inference_agrees,a_repair_that_under_declares_is_caught_in_the_inferred_direction,a_declaration_that_over_claims_is_caught_in_the_declared_direction,a_trailer_missing_its_identity_yields_no_declaration,a_coherent_artifact_still_declares_that_it_was_judged,an_absent_declaration_judges_nothing,a_repairing_declaration_reports_both_numerator_and_denominator,an_unjudgeable_ledger_refuses_rather_than_skipping,one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one,an_unjudgeable_non_ledger_artifact_does_not_stop_the_line
echo '== MUTATED: withheld-row fixture no longer withholds; a_withheld_row_is_named_as_missing must FAIL'
python3 - <<'P'
p='dag/test/claim/ledger_row_coherence.dag'
s=open(p).read()
i=s.index('test fn a_withheld_row_is_named_as_missing')
s=s[:i]+s[i:].replace('committed_content: all_rows_except(a: a, skip: id)','committed_content: all_rows_rendered(a: a)',1)
open(p,'w').write(s)
P
cb dag/test/claim/ledger_row_coherence.dag a_withheld_row_is_named_as_missing,a_fully_rendered_ledger_is_coherent
