set -u
git fetch -q origin c4-enforcement-consistency; HEADSHA=$(git rev-parse FETCH_HEAD); echo "HEAD $HEADSHA"
git worktree add -q --detach /tmp/co5 $HEADSHA && cd /tmp/co5 && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16) GUNBC $(sha256sum target/release/gunbc | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-230; }
G=src/v2/test/claim/enforcement_live_witness.dag
echo "== HEAD"; cb $G enforcement_consistency_gate_holds,only_cost_contract_carries_red_control_probe
echo "== rfm-rung-census"; target/release/gunbc test //gunbc/instruments:rfm-rung-census > /tmp/rfm.out 2>&1; echo "exit=$?"; tail -15 /tmp/rfm.out | cut -c1-250
echo "== RED: Blocking contract consumer bound to a deleted file"
python3 - <<'P2'
p='src/v2/lens/enforcement/contract.dag'
s=open(p).read()
k='entry: "src/v2/test/claim/emitted_binary_effect_confinement_red_control.dag"'
assert k in s
open(p,'w').write(s.replace(k,'entry: "src/v2/test/claim/deleted_emitted_binary_effect_confinement_red_control.dag"'))
P2
cb $G enforcement_consistency_gate_holds
