set -u
G=/sys/fs/cgroup/ks$$; mkdir -p $G && echo 48G > $G/memory.max && echo $$ > $G/cgroup.procs
git fetch -q origin scratch/keen-owl-704-14657-probe
d=/tmp/sp; git worktree add -q --detach $d bf84073261464a0d4bdeda72ea7576e104ab7904; cd $d; echo "TREE $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
RS="where_refinements_retire_for_checked_constructors literal_only_refinement_judgment_is_interim instrument_measurement_queues_run_in_parallel approval_enrolment_is_google_sign_in drain_wait_bound_is_derived_from_declared_job_timeouts emit_build_subjects_are_the_two_compiler_products"
F=dag/test/claim/probe_standing.dag
{ echo "module test.claim.probe_standing"; echo "import gunbc.documentary_refs { documentary_ref_symbol_text }"; echo "import gunbc.design_document { expected_design_md }"; echo "import gunbc.design_ruling { $(echo $RS | sed 's/ /, /g') }"
  for r in $RS; do echo "test fn cited_$r() -> Bool {"; echo "  string_contains(s: expected_design_md(), pattern: documentary_ref_symbol_text(reference: $r.reference))"; echo "}"; done; } > $F
FN=$(for r in $RS; do printf "cited_%s," $r; done | sed 's/,$//')
echo "== instrument cited-reading per new row (PASS = the generated document cites it)"
$B --wet --source-root dag --source-root src/v2 --entry $F --functions $FN > /tmp/cb.out 2>&1; echo "EXIT $?"; grep -aE '^(PASS|FAIL|UNOBS)' /tmp/cb.out | cut -c1-400
echo "== documentary_refs_every_ruling_standing_agrees_with_the_document with placeholders"
$B --wet --source-root dag --source-root src/v2 --entry dag/test/claim/documentary_refs_witness.dag --functions documentary_refs_every_ruling_standing_agrees_with_the_document > /tmp/cb.out 2>&1; grep -aE '^(PASS|FAIL|UNOBS)' /tmp/cb.out | cut -c1-400
