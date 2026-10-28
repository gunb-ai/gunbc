export GUNBC_MEMORY_BUDGET_BYTES=20000000000
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -E "^error|Finished"
B=$PWD/target/release/claim_batch; F=dag/test/claim/v1_required_lanes_withdrawn_stage0_mirrors_witness.dag; D=dag/gunbc/rung_drop/v1_required_lanes_withdrawn.dag
FN=$(grep -oP '^test fn \K[a-zA-Z0-9_]+' $F | paste -sd,)
run(){ echo "== $1"; timeout 1500 $B --wet --source-root dag --source-root src/v2 --entry $F --functions $FN 2>&1 | grep -aE '^(PASS|FAIL|UNOBSERVED)|refus' | cut -c1-200; }
cp $D /tmp/d.orig
run unmutated
python3 - <<'P'
import re
d='dag/gunbc/rung_drop/v1_required_lanes_withdrawn.dag';s=open(d).read()
a=s.index('fn v1_required_lanes_withdrawn_stage0_partition_mirrors()');b=s.index('\n}\n',a)
open(d,'w').write(s[:a]+'fn v1_required_lanes_withdrawn_stage0_partition_mirrors() -> List<WithdrawnStage0Mirror> {\n  []'+s[b:])
P
run derivation-emptied
cp /tmp/d.orig $D
python3 - <<'P'
d='dag/gunbc/rung_drop/v1_required_lanes_withdrawn.dag';s=open(d).read()
s2=s.replace('  distinct_by_key(\n    flat_map(generated_partition_crate_rows, r => map(r.modules, m => WithdrawnStage0Mirror { repo_path: concat(stage0_src_repo_prefix, concat(m, ".rs")) })),\n    w => w.repo_path\n  )','  flat_map(generated_partition_crate_rows, r => map(r.modules, m => WithdrawnStage0Mirror { repo_path: concat(stage0_src_repo_prefix, concat(m, ".rs")) }))')
assert s!=s2; open(d,'w').write(s2)
P
run no-distinct
cp /tmp/d.orig $D
echo ==END==
