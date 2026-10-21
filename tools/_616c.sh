set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14616 corrected witness arm: dispatch A's W2 ran claim names that exist in NO file at this head
# (fabricated from a stale local note) -> rc=2 zero verdicts, voided. Run the 3 claims that DO exist.
OID=8f97fcaf55d57430f69ee0599c95bdaf6c79b45e
echo "HEADSHA=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w616c
git fetch -q origin +refs/pull/14616/head:refs/verifier/14616c 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14616c)" = "$OID" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w616c $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w616c
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=src/v2/test/claim/falsifier_rehomed_rows_unexecuted_witness.dag
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
echo "claims from the file: $FNS"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14616 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14616/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) (expect f083014aaef4 from dispatch A)"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c616c.out 2>&1; rc=$?
echo "rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c616c.out) FAIL=$(grep -ac '^FAIL ' /tmp/c616c.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c616c.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c616c.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c616c.out)" = "0" ] && echo "!! EXIT 0 HAVING OBSERVED NOTHING"
echo DONE_616_C
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
