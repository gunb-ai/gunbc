set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Attribution, honest on BOTH sides: main-built binary at the main tree (a main arm run with the
# HEAD-built binary gave rc=1 with ZERO verdicts -- condition (c), excluded, not counted).
H=34ad3659dbd0c9eda30a14604797f453e5bd7d5c
echo "H=$H START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613g /tmp/w613gm
git fetch -q origin '+refs/pull/14613/head:refs/verifier/14613h' 2>&1 | tail -1
git fetch -q origin main 2>&1 | tail -1
M=$(git rev-parse origin/main); echo "M=$M"
[ "$(git rev-parse refs/verifier/14613h)" = "$H" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613h)"; exit 9; }
git worktree add -q --detach /tmp/w613gm $M || { echo WT_FAILED_M; exit 9; }
DOC=dag/test/claim/documentary_refs_witness.dag
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
echo "=== build claim_batch AT MAIN into .tmain613 ==="
CARGO_TARGET_DIR=/root/workspace/repo-root/.tmain613 cargo build --manifest-path /tmp/w613gm/Cargo.toml --release -p v1-compiler --bin claim_batch --offline 2>&1 | tail -1 || CARGO_TARGET_DIR=/root/workspace/repo-root/.tmain613 cargo build --manifest-path /tmp/w613gm/Cargo.toml --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CBM=/root/workspace/repo-root/.tmain613/release/claim_batch
[ -x $CBM ] || { echo NO_BINARY_MAIN; exit 9; }
[ "$(git -C /tmp/w613gm rev-parse HEAD)" = "$M" ] || { echo MAIN_WT_WRONG; exit 9; }
echo "CBM_SHA=$(sha256sum $CBM|cut -d' ' -f1|cut -c1-12) built $(date -u +%H:%M:%S)Z at $(git -C /tmp/w613gm rev-parse HEAD | cut -c1-12)"
FNS=$(cd /tmp/w613gm && grep -aoE '^test fn [a-zA-Z0-9_]+' $DOC | sed 's/^test fn //' | sort | paste -sd,)
echo "-------- [main-main] main-built binary at main tree, 5 claims by name $(date -u +%H:%M:%S)Z"
( cd /tmp/w613gm && timeout 900 $CBM --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $DOC --functions "$FNS" > /tmp/c613g_main.out 2> /tmp/c613g_main.err )
rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c613g_main.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613g_main.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613g_main.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613g_main.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613g_main.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
if [ "$(grep -acE '^PASS |^FAIL ' /tmp/c613g_main.out)" = "0" ]; then
  echo "  zero-verdict: err tail:"; tail -8 /tmp/c613g_main.err | sed 's/^/    E| /' | cut -c1-190
  echo "  out tail:"; tail -6 /tmp/c613g_main.out | sed 's/^/    O| /' | cut -c1-190
fi
echo "  the missing symbol at head was reported as a design_ruling symbol absent from the rendered document; which one?"
echo DONE_613_G
cd /root/workspace/repo-root
git worktree remove --force /tmp/w613gm >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
