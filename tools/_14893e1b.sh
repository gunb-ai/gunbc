#!/usr/bin/env bash
# #14893 @ab3279b680 E1b: the BASE-DEFECT arm done right. G2 = head + plant893-R + gate893b-R + gate893-R
# (base = same tree, gating diff reverse-applied, WITH the planted over-attribution present).
# Expect: refusal PRINTED ([cost-partition] Refused kind=OverAttributed), claim PASSes, rc=0  (base defect).
# P re-verified from head with the SAME builds as E1 (CB shas must match: same-binary proof across dispatches).
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=ab3279b68044af1af59311cb6e1f0307f88a0d92
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893eb 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893eb
echo 51539607552 > /sys/fs/cgroup/ver893eb/memory.max
RB=$(cat /sys/fs/cgroup/ver893eb/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893eb memory.max=$RB (read-back, bounded)"
exec 11>/sys/fs/cgroup/ver893eb/cgroup.procs; echo $BASHPID >&11
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893eb || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893eb)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$HEAD_SHA"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893eb
T=/tmp/w893eb_h; G=/tmp/w893eb_g; M=/tmp/w893eb_m
for d in $T $G $M; do git worktree prune >/dev/null 2>&1; rm -rf $d; done
for d in $T $G $M; do git worktree add -q --force --detach $d $HEAD_SHA || { echo WT_FAIL $d; exit 9; }; done
# M: mutation worktree (NoSpans arm) for the mutation unit run — E1 already showed the red; rerun to confirm determinism cheaply? skip.
rm -rf $M
# G2: plant first, THEN gates
(cd $G && git apply -R $REPO/tools/plant893.patch) || { echo PLANT_APPLY_FAIL; exit 9; }
grep -q 'pub out_of_span' $G/src/v1/stage0/src/cli_run.rs && { echo PLANT_NOOP; exit 9; }
(cd $G && git apply -R $REPO/tools/gate893b.patch && git apply -R $REPO/tools/gate893.patch) || { echo GATE_APPLY_FAIL; exit 9; }
[ "$(grep -c 'CostAccountingRefusal\|partition_exit_flags' $G/src/v1/stage0/src/bin/claim_batch.rs)" = "0" ] || { echo GATE_INCOMPLETE; exit 9; }
grep -q 'out_of_span' $T/src/v1/stage0/src/cli_run.rs || { echo HEAD_PRECHECK_FAIL; exit 9; }
echo TREES_OK
build() { # tag tree bin
  local tag=$1 tree=$2 bin=$3
  (cd $tree && cargo build --release -p v1-compiler --bin $bin 2>&1 | tail -1)
  local b=$CARGO_TARGET_DIR/release/$bin
  [ -x $b ] || { echo NO_BINARY $tag; exit 9; }
  cp $b /tmp/cb893eb_$tag; sha /tmp/cb893eb_$tag | sed "s/^/CB_$tag=/"; rm -f $b
}
build H $T claim_batch
echo "CB_H_MATCHES_E1=$( [ "$(sha /tmp/cb893eb_H)" = "1dba8ce124dd51349818760527b66205f8f25130ee6374349e85034f9f7da2b2" ] && echo YES || echo NO )"
build G $G claim_batch
echo "CB_G_NEW_SHA (base driver, plant+no-gate)"
claim() { # tag fn entry
  local tag=$1 fn=$2 entry=$3
  cd $T || exit 9
  timeout 900 /tmp/cb893eb_$tag --claim-run --source-root dag --source-root src/v2 --entry $entry --functions $fn > /tmp/w893eb_${tag}_$fn.out 2>&1
  local rc=$?
  local cp=$(grep -aE '\[cost-partition\]' /tmp/w893eb_${tag}_$fn.out | tail -1)
  echo "  [$tag $fn] rc=$rc PASS=$(grep -ac "^PASS $fn$" /tmp/w893eb_${tag}_$fn.out) FAIL=$(grep -ac "^FAIL $fn$" /tmp/w893eb_${tag}_$fn.out) verdict=$(printf '%s' "$cp" | grep -oE '\"state\":\"[A-Za-z]+\"' | head -1) kind=$(printf '%s' "$cp" | grep -oE '\"kind\":\"[A-Za-z]+\"' | head -1) oos=$(printf '%s' "$cp" | grep -oc 'out_of_span') refused_printed=$(grep -ac 'Refused' /tmp/w893eb_${tag}_$fn.out) BYTES=$(wc -c < /tmp/w893eb_${tag}_$fn.out) $(date -u +%H:%M:%S)Z"
  cd $REPO || exit 9
}
for pair in "string_family_cast_holds dag/test/claim/string_family_cast.dag" "unicode_ident_xid_grounding_holds dag/test/claim/unicode_ident_xid_grounding.dag"; do
  set -- $pair; fn=$1; entry=$2
  for tag in H G; do claim $tag $fn $entry; done
done
echo DONE_14893E1B
