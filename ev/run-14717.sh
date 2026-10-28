set -u
git fetch -q origin scratch/keen-owl-704-bootref-merge; H=$(git rev-parse FETCH_HEAD); echo "FETCHED $H (expected 6e7134997d41c70b3705c5ff4451adce65cf777f)"
git worktree add -q --detach /tmp/co17 $H && cd /tmp/co17 && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
for f in dag/test/claim/fleet/fleet_converge_checkout_trust.dag dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag dag/test/claim/namecheap_workflow_witness.dag dag/test/claim/printer_workflow_binding_witness.dag; do
 fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
 $B $R --entry $f --functions $fns 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-200 > /tmp/r.txt
 echo "== $f: $(grep -c ^PASS /tmp/r.txt) PASS $(grep -c ^FAIL /tmp/r.txt) FAIL $(grep -c ^UNOBS /tmp/r.txt) UNOBS"; grep -v ^PASS /tmp/r.txt
done
