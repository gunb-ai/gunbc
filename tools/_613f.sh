set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Which tree owns the red? documentary_refs_witness :: ..._generated_document_renders_the_rows
# FAILed at #14613 head 34ad3659 during my delta-surface arm. This dispatch runs the same 5 claims,
# same head-built binary, at BOTH the PR head and origin/main -- a pre-existing red on an unenrolled
# witness is reportable but not this PR's; one introduced by the delta is this PR's.
H=34ad3659dbd0c9eda30a14604797f453e5bd7d5c
echo "H=$H START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613f /tmp/w613m
git fetch -q origin '+refs/pull/14613/head:refs/verifier/14613f' 2>&1 | tail -1
git fetch -q origin main 2>&1 | tail -1
M=$(git rev-parse origin/main); echo "M=$M"
[ "$(git rev-parse refs/verifier/14613f)" = "$H" ] || { echo "PR_REF_MOVED"; exit 9; }
git worktree add -q --detach /tmp/w613f $H || { echo WT_FAILED; exit 9; }
git worktree add -q --detach /tmp/w613m $M || { echo WT_FAILED_M; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w613f || exit 9
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14613/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
[ "$(git rev-parse HEAD)" = "$H" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha256sum $CB|cut -d' ' -f1|cut -c1-12) built at head"
DOC=dag/test/claim/documentary_refs_witness.dag
run_at() { # wt tag
  local wt=$1 tag=$2
  cd $wt || return 0
  local FNS
  FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $DOC | sed 's/^test fn //' | sort | paste -sd,)
  echo "-------- [$tag] documentary_refs claims by name (names read from THIS tree) $(date -u +%H:%M:%S)Z"
  echo "  tree=$(git rev-parse HEAD | cut -c1-12) fns=$FNS" | cut -c1-200
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $DOC --functions "$FNS" > /tmp/c613f_$tag.out 2>&1; local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT EXCLUDED"; return 0; }
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c613f_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613f_$tag.out)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613f_$tag.out | sed 's/^/    V| /' | cut -c1-160
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613f_$tag.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
  return 0
}
run_at /tmp/w613f head
run_at /tmp/w613m main
echo DONE_613_F
cd /root/workspace/repo-root
git worktree remove --force /tmp/w613f >/dev/null 2>&1 || true
git worktree remove --force /tmp/w613m >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
