set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency; HEADSHA=$(git rev-parse FETCH_HEAD)
PRE=115143a4d8a1e870124e7848fa5084c3fcfe1c38
echo "HEAD $HEADSHA PRE $PRE"
git worktree add -q --detach /tmp/co3 $HEADSHA && cd /tmp/co3 && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-230; }
G=src/v2/test/claim/enforcement_live_witness.dag
FILES=$(grep -rlE "import v2\.lens\.(registry|enforcement)" --include=*.dag src dag | grep -E "test|claim|witness" | xargs grep -l '^test fn' | sort)
sweep(){ for f in $FILES; do fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,); [ -f $f ] && cb $f $fns | sed "s#^#$f #"; done; }
echo "== HEAD: gate + control"
cb $G enforcement_consistency_gate_holds,only_cost_contract_carries_red_control_probe
echo "== HEAD sweep of every claim file importing v2.lens.registry/enforcement"
sweep > /tmp/head.sweep; echo "head: $(grep -c ' PASS ' /tmp/head.sweep) PASS $(grep -c ' FAIL ' /tmp/head.sweep) FAIL $(grep -c ' UNOBS ' /tmp/head.sweep) UNOBS"; grep -a ' FAIL \| UNOBS ' /tmp/head.sweep
echo "== PRE-deletion tree (115143a4d8a) sweep for comparison"
git checkout -q $PRE -- src dag; git clean -fdq src dag
sweep > /tmp/pre.sweep; echo "pre: $(grep -c ' PASS ' /tmp/pre.sweep) PASS $(grep -c ' FAIL ' /tmp/pre.sweep) FAIL"
echo "== claims FAILing at head but not at pre (new refusals):"; diff <(grep -a ' FAIL ' /tmp/pre.sweep|sort) <(grep -a ' FAIL ' /tmp/head.sweep|sort) | grep -a '^>' || echo none
git checkout -q $HEADSHA -- src dag; git clean -fdq src dag
echo "== RED: plant a Blocking contract consumer bound to a deleted file"
python3 - <<'P'
p='src/v2/lens/enforcement/contract.dag'
s=open(p).read()
k='entry: "src/v2/test/claim/long/vacuity_consumer_witness_long.dag"'
assert k in s
open(p,'w').write(s.replace(k,'entry: "src/v2/test/claim/long/deleted_vacuity_consumer_witness_long.dag"'))
P
cb $G enforcement_consistency_gate_holds
