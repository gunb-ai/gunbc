set -u
ROOT=$(pwd)
git fetch -q origin c4g-rfm-ambig 2>&1 | tail -2
GOT=$(git rev-parse FETCH_HEAD)
echo "FETCHED_HEAD $GOT (expected 4a77abc8c6f994902d4e74a56037181c07689ec4)"
git -C "$ROOT" worktree add -q --detach /tmp/coA FETCH_HEAD && cd /tmp/coA && echo "CO_HEAD $(git rev-parse HEAD)"
echo "CHECKOUT_TREE $(git -C "$ROOT" rev-parse FETCH_HEAD^{tree})"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^(error|warning: build)|-->|could not|No such" | head -12
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-200; }
echo "== wet-schedule rows of this PR present at head: $(grep -c "module_path: \"test.claim.record_emission_candidate_set_witness\"" src/v2/workflow/local_repo_wet_terminal.dag) (expect 2)"
echo "== head claims"; cb dag/test/claim/record_emission_candidate_set_witness.dag single_field_ambiguous_literal_emits_the_refusal,multi_field_ambiguous_literal_refuses_with_located_diagnostic,single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct
echo "== MUTATED (ambiguous providers swapped for single-candidate): the two REDs must FAIL, controls PASS"
sed -i "s/laneg_multi_amb_provider/laneg_multi_res_provider/; s/laneg_single_amb_provider/laneg_single_res_provider/" dag/test/claim/record_emission_candidate_set_witness.dag
cb dag/test/claim/record_emission_candidate_set_witness.dag single_field_ambiguous_literal_emits_the_refusal,multi_field_ambiguous_literal_refuses_with_located_diagnostic,single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct
