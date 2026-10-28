set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency; HEADSHA=$(git rev-parse FETCH_HEAD)
PRE=115143a4d8a1e870124e7848fa5084c3fcfe1c38
echo "HEAD $HEADSHA PRE $PRE"
git worktree add -q --detach /tmp/co3s $HEADSHA && cd /tmp/co3r && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-230; }
G=src/v2/test/claim/enforcement_live_witness.dag
FILES=$(grep -rlE "import v2\.lens\.(registry|enforcement)" --include=*.dag src dag | grep -E "test|claim|witness" | xargs grep -l '^test fn' | sort)
sweep(){ for f in $FILES; do fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,); [ -f $f ] && cb $f $fns | sed "s#^#$f #"; done; }
echo "== HEAD"; cb $G enforcement_consistency_gate_holds,only_cost_contract_carries_red_control_probe
echo "== RED: plant a Blocking contract consumer bound to a deleted file"
python3 - <<'P2'
p='src/v2/lens/enforcement/contract.dag'
s=open(p).read()
k='entry: "src/v2/test/claim/emitted_binary_effect_confinement_red_control.dag"'
assert k in s
open(p,'w').write(s.replace(k,'entry: "src/v2/test/claim/deleted_emitted_binary_effect_confinement_red_control.dag"'))
P2
cb $G enforcement_consistency_gate_holds
