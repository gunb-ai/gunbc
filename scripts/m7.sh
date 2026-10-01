W=/home/briansrls/.worktrees/gunbc/royal-newt-820-mainbuild; R=/home/briansrls/.worktrees/gunbc/royal-newt-820
git -C $W checkout -qf --detach $(git -C $R rev-parse MERGE_HEAD); echo "main=$(git -C $W rev-parse --short HEAD)"
cd $W && CARGO_TARGET_DIR=/tmp/claude-1000/main-target CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin claim_batch 2>&1 | grep -E "^error|Finished"
cd $R; bash /tmp/claude-1000/mregen.sh > /tmp/claude-1000/mregen.log 2>&1; grep -E "first_generation|NO_CAND|^error" /tmp/claude-1000/mregen.log | tail -3
H=/tmp/claude-1000/royal-target/release; M=/tmp/claude-1000/main-target/release
timeout 900 $H/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=docs/design-rung-drops.md 2>&1 | tail -1 | cut -c1-80
run(){ f=$1; timeout 1500 $2/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $(grep -oP '(?<=^test fn )\w+' $f | paste -sd,) 2>&1 | grep -E "^PASS|^FAIL|THREW|REFUS" | cut -c1-120; }
for f in dag/test/claim/text_boundary_identity_wall_witness_test.dag dag/test/claim/bare_name_ambiguity_wall_witness_test.dag dag/test/claim/guarantee_stall_witness_test.dag $(git ls-files | grep alias_head_identity_probe_witness_test.dag); do echo "== $f"; run $f $H > /tmp/claude-1000/r.txt; cut -d' ' -f1 /tmp/claude-1000/r.txt | sort | uniq -c; grep -v ^PASS /tmp/claude-1000/r.txt; done
echo "== receipt for new importers"
for f in $(cat /tmp/claude-1000/new5.txt); do
  mod=$(grep -m1 -oP '(?<=^module )\S+' $f)
  (cd $W && timeout 1500 $M/gunbc compile --source-root dag --source-root src/v2 --entry $f --target dag --output-dir /tmp/claude-1000/n5b 2>&1) > /tmp/claude-1000/n5b.log; grep -E "^error\[" /tmp/claude-1000/n5b.log | sort -u > /tmp/claude-1000/n5eb
  timeout 1500 $H/gunbc compile --source-root dag --source-root src/v2 --entry $f --target dag --output-dir /tmp/claude-1000/n5h > /tmp/claude-1000/n5h.log 2>&1; grep -E "^error\[" /tmp/claude-1000/n5h.log | sort -u > /tmp/claude-1000/n5eh
  echo "COMPILE $mod base[$(grep -oE '[0-9]+ blocking' /tmp/claude-1000/n5b.log)] head[$(grep -oE '[0-9]+ blocking' /tmp/claude-1000/n5h.log)] head-only rows: $(comm -13 /tmp/claude-1000/n5eb /tmp/claude-1000/n5eh | wc -l)"
  for c in $(grep -rlE "^import ${mod//./\\.}\b" dag src/v2 | xargs -r grep -l "^test fn"); do
    (cd $W; [ -f $c ] && run $c $M | sort) > /tmp/claude-1000/n5cb; run $c $H | sort > /tmp/claude-1000/n5ch
    echo "  CLAIMS $c base $(cut -d' ' -f1 /tmp/claude-1000/n5cb|sort|uniq -c|tr '\n' ' ') head $(cut -d' ' -f1 /tmp/claude-1000/n5ch|sort|uniq -c|tr '\n' ' ') base-PASS-not-head: $(comm -23 <(grep ^PASS /tmp/claude-1000/n5cb) <(grep ^PASS /tmp/claude-1000/n5ch) | wc -l)"
  done
done
cargo fmt --all --check >/dev/null 2>&1; echo fmt=$?
echo M7DONE
