#!/usr/bin/env bash
# #14893 @ab3279b680 E3: census head vs base, each arm with a gunbc BUILT FROM ITS OWN TREE
# (BinaryFreshnessUndecided gate: a binary may only run from the worktree whose paths its dep-info names).
# Both trees share IDENTICAL Rust source (only the .dag row file differs) => deterministic builds
# must print the SAME sha256: that equality is the one-binary proof. Expect census_below_ceiling 37 (head) vs 38 (base).
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=ab3279b68044af1af59311cb6e1f0307f88a0d92
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893e3 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893e3)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$HEAD_SHA"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
T=/tmp/w893e3_h; B=/tmp/w893e3_b
for d in $T $B; do git worktree prune >/dev/null 2>&1; rm -rf $d; done
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
ROW=dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag
grep -q AtCeiling $T/$ROW || { echo HEAD_ROW_UNEXPECTED; exit 9; }
cp $REPO/tools/stage893/a_refusal_prints_and_the_line_continues.base.dag $B/$ROW || exit 9
grep -q BelowCeiling $B/$ROW || { echo RESTORE_NOOP; exit 9; }
grep -q AtCeiling $T/$ROW || { echo HEAD_ROW_LOST; exit 9; }
diff <(cd $T && git status --porcelain) <(cd $B && git status --porcelain) | head -4
echo TREES_OK "(row file is the only diff; Rust source identical)"
(cd $T && CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893e3h cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GH=/root/workspace/repo-root/.t14893e3h/release/gunbc
[ -x $GH ] || { echo NO_GUNBC_H; exit 9; }
(cd $B && CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893e3b cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GB=/root/workspace/repo-root/.t14893e3b/release/gunbc
[ -x $GB ] || { echo NO_GUNBC_B; exit 9; }
SH_H=$(sha $GH); SH_B=$(sha $GB)
echo "GUNBC_HEAD_SHA=$SH_H"
echo "GUNBC_BASE_SHA=$SH_B"
[ "$SH_H" = "$SH_B" ] && echo "EXECUTOR_IDENTICAL=YES (same sha from both trees — deterministic, Rust source identical)" || echo "EXECUTOR_IDENTICAL=NO (source differs beyond .dag — interpret accordingly)"
TARGET='//gunbc/instruments:rfm-rung-census'
census() { # tag tree g
  local tag=$1 tree=$2 g=$3
  local out=/tmp/w893e3_$tag.out
  cd $tree || exit 9
  timeout 1500 $g test "$TARGET" > $out 2>&1; local rc=$?
  echo "  [$tag] rc=$rc BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
  if [ "$(wc -c < $out)" -lt 500 ]; then echo "    RAWDUMP:"; sed 's/^/    R| /' $out | cut -c1-200; fi
  grep -aE 'census_read|census_below|case |refus|REFUSED|closure=' $out | head -14 | sed 's/^/    C| /' | cut -c1-190
  grep -ac 'a_refusal_prints_and_the_line_continues' $out | sed 's/^/    ROWMENTIONS=/'
  cd $REPO || exit 9
}
echo "=== census HEAD $(date -u +%H:%M:%S)Z"; census head $T $GH
echo "=== census BASE $(date -u +%H:%M:%S)Z"; census base $B $GB
echo DONE_14893E3
