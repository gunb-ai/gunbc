set -u
cd /root/workspace/repo-root 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w671dbg
git worktree add -q --detach /tmp/w671dbg 8b6fa692557687874a554955092b279ed7a1698d || { echo WT_FAIL; exit 9; }
cd /tmp/w671dbg || exit 9
grep -c '<<<<<<<\|>>>>>>>' src/v2/workflow/local_repo_wet_terminal.dag | sed 's/^/CONFLICT_MARKERS=/'
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/w671dbg/.t
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -E '^error|error\[' | head -12
echo BUILD_TAIL_DONE
cd /root/workspace/repo-root
git worktree remove --force /tmp/w671dbg >/dev/null 2>&1 || true
