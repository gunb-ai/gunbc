CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 23622320128 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
F=src/v1/stage0/src/cli_run.rs; cp $F /tmp/keep.rs
python3 - <<'PY'
p='src/v1/stage0/src/cli_run.rs'; s=open(p).read()
a="""                | Some(RequiredFloorDisposition::DeclinedNoCiWetLane { .. })
                | Some(RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery {
                    ..
                }) => CostDebtRosterStanding::OutsideThisRunsUniverse,"""
assert s.count(a)==1, "anchor"
s=s.replace(a,""" => CostDebtRosterStanding::OutsideThisRunsUniverse,
                Some(RequiredFloorDisposition::DeclinedNoCiWetLane { .. })
                | Some(RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery { .. }) => CostDebtRosterStanding::DeclaredButNotWithheld,""")
open(p,'w').write(s)
PY
cargo test --release -p v1-compiler --lib changed_selection_declines_are_outside_this_runs_universe 2>&1 | grep -E "test result|error\[|FAILED" | head -3; echo "^^ REVERTED-ARM RUN (expect FAILED)"
cp /tmp/keep.rs $F
cargo clippy -p v1-compiler --lib --tests -- -D warnings 2>&1 | grep -E "^(error|warning)|-->" | head -20; echo CLIPPY_DONE
for t in changed_selection_declines_are_outside_this_runs_universe only_the_cost_debt_disposition changed_witness_sublane_join_tests; do cargo test --release -p v1-compiler --lib $t 2>&1 | grep -E "test result|FAILED|panicked" | head -2; done
