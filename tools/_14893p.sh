#!/usr/bin/env bash
# probe: what does `gunbc test //gunbc/instruments:rfm-rung-census` actually print? (134B rc=2 mystery)
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=ab3279b68044af1af59311cb6e1f0307f88a0d92
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893p || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893p)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893p
T=/tmp/w893p; git worktree prune >/dev/null 2>&1; rm -rf $T
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
(cd $T && cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GUNBC=$CARGO_TARGET_DIR/release/gunbc
cd $T || exit 9
echo "== try1 exact label"
timeout 900 $GUNBC test '//gunbc/instruments:rfm-rung-census' > /tmp/p1.out 2>&1; echo "rc=$? BYTES=$(wc -c </tmp/p1.out)"; cat /tmp/p1.out | head -12
echo "== try2 bare label no slashes"
timeout 900 $GUNBC test 'gunbc/instruments:rfm-rung-census' > /tmp/p2.out 2>&1; echo "rc=$? BYTES=$(wc -c </tmp/p2.out)"; head -12 /tmp/p2.out
echo "== try3 pattern with wildcard"
timeout 900 $GUNBC test '*rfm-rung-census*' > /tmp/p3.out 2>&1; echo "rc=$? BYTES=$(wc -c </tmp/p3.out)"; head -12 /tmp/p3.out
echo "== how CI/wet lane invokes this instrument target"
grep -rn 'rfm-rung-census' --include='*.rs' src/v2 src/v1/stage0/src 2>/dev/null | grep -v test_ | head -8
grep -rn 'instruments:' --include='*.rs' src/v2/workflow | grep -i 'test\|verb\|target' | head -8
echo DONE_PROBE
