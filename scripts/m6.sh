W=/home/briansrls/.worktrees/gunbc/royal-newt-820-mainbuild; R=/home/briansrls/.worktrees/gunbc/royal-newt-820
git -C $W checkout -qf --detach $(git -C $R rev-parse MERGE_HEAD); echo "main=$(git -C $W rev-parse --short HEAD)"
cd $W && CARGO_TARGET_DIR=/tmp/claude-1000/main-target CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin claim_batch 2>&1 | grep -E "^error|Finished"
cd $R; bash /tmp/claude-1000/mregen.sh > /tmp/claude-1000/mregen.log 2>&1; grep -E "first_generation|NO_CAND|^error" /tmp/claude-1000/mregen.log | tail -3
H=/tmp/claude-1000/royal-target/release; M=/tmp/claude-1000/main-target/release
timeout 900 $H/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=docs/design-rung-drops.md 2>&1 | tail -1 | cut -c1-80
run(){ f=$1; timeout 1500 $2/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $(grep -oP '(?<=^test fn )\w+' $f | paste -sd,) 2>&1 | grep -E "^PASS|^FAIL|THREW|REFUS" | cut -c1-120; }
for f in dag/test/claim/text_boundary_identity_wall_witness_test.dag dag/test/claim/bare_name_ambiguity_wall_witness_test.dag dag/test/claim/guarantee_stall_witness_test.dag; do echo "== $f"; run $f $H | cut -d' ' -f1 | sort | uniq -c; run $f $H | grep -v ^PASS; done
echo "== new-importer receipt (base = main binary on main tree; head = merged)"
for f in dag/gunbc/bmc_model.dag dag/gunbc/runner/wet_host_premise_readback.dag; do
  (cd $W && timeout 1500 $M/gunbc compile --source-root dag --source-root src/v2 --entry $f --target dag --output-dir /tmp/claude-1000/n4b 2>&1 | tee /tmp/claude-1000/n4b.log | grep blocking); grep -E "^error\[" /tmp/claude-1000/n4b.log | sort -u > /tmp/claude-1000/n4eb
  (timeout 1500 $H/gunbc compile --source-root dag --source-root src/v2 --entry $f --target dag --output-dir /tmp/claude-1000/n4h 2>&1 | tee /tmp/claude-1000/n4h.log | grep blocking); grep -E "^error\[" /tmp/claude-1000/n4h.log | sort -u > /tmp/claude-1000/n4eh
  echo "COMPILE $f head-only error rows: $(comm -13 /tmp/claude-1000/n4eb /tmp/claude-1000/n4eh | wc -l)"
done
for f in $(grep -rlE "^import (gunbc\.bmc_model|gunbc\.runner\.wet_host_premise_readback)\b" dag src/v2 | xargs grep -l "^test fn"); do
  (cd $W; run $f $M | sort) > /tmp/claude-1000/n4cb; run $f $H | sort > /tmp/claude-1000/n4ch
  echo "CLAIMS $f base $(cut -d' ' -f1 /tmp/claude-1000/n4cb|sort|uniq -c|tr '\n' ' ') head $(cut -d' ' -f1 /tmp/claude-1000/n4ch|sort|uniq -c|tr '\n' ' ') base-PASS-not-head: $(comm -23 <(grep ^PASS /tmp/claude-1000/n4cb) <(grep ^PASS /tmp/claude-1000/n4ch) | wc -l)"
done
echo "== skip redundancy test"
cp src/v1/stage0/src/cli_run.rs /tmp/claude-1000/cli_run.keep
python3 - <<'P'
p='src/v1/stage0/src/cli_run.rs'; s=open(p).read()
a=s.index("                // A kernel or container spelling binds the substrate, never a declaring module\n")
b=s.index("                if is_substrate_vocabulary(name) {\n                    continue;\n                }\n",a)+len("                if is_substrate_vocabulary(name) {\n                    continue;\n                }\n")
open(p,'w').write(s[:a]+s[b:])
P
CARGO_TARGET_DIR=/tmp/claude-1000/royal-target CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -E "^error|Finished"
echo "without skip:"; run dag/test/claim/bare_name_ambiguity_wall_witness_test.dag $H
cargo fmt --all --check >/dev/null 2>&1; echo fmt=$?
echo M6DONE
