#!/usr/bin/env bash
# #14788 @ 7e36ad457b55062c4e825f9fff70df3a99c79f7c (base = 644069c197591c09215c8e0122a495036421df8c)
# Arms: HEAD 33/33 by name; BASE same binary 31/2 (the two claims FAIL); MUT A (claim1 conjunct back
# to literal "cursor-agent") -> claim1 FAILs; MUT B (claim2 abandoned branch back to fx_take) -> claim2 FAILs;
# restore byte-exact + post. Author did NOT establish 33/33 (their whole-file run exited 2) -- my full run settles it.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEADSHA=7e36ad457b55062c4e825f9fff70df3a99c79f7c
BASESHA=644069c197591c09215c8e0122a495036421df8c
[ ${#HEADSHA} -eq 40 ] || { echo "LEN STOP"; exit 9; }
WIT=dag/test/claim/dispatch_credential_snapshot_wet_witness.dag
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver788 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver788
echo 51539607552 > /sys/fs/cgroup/ver788/memory.max
RB=$(cat /sys/fs/cgroup/ver788/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver788 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver788/cgroup.procs && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (proceeding)"
sha(){ sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w788h /tmp/w788b
git fetch -q origin +refs/pull/14788/head:refs/verifier/f788 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f788)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f788) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w788h $HEADSHA || { echo WT_FAIL; exit 9; }
cd /tmp/w788h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
[ "$(git cat-file blob $HEADSHA:$WIT | sha256sum | cut -d' ' -f1)" = "$(sha $WIT)" ] || { echo "NOT_HEAD_BLOB -- STOP"; exit 9; }
cp -f $WIT /tmp/wit788_orig
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t788 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t788/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NAMES=$(grep -oE '^test fn [a-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l) (33 expected)"
C1=a_cursor_turn_on_srv2_binds_the_taken_snapshot_through_the_env_wrapper
C2=an_abandoned_cursor_attempt_leaves_a_tombstone
C3=a_cursor_turn_off_its_custody_host_refuses
runall() { # $1 tag $2 tree-dir
  cd $2 || return 1
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/f788_${1}.out 2>&1
  local rc=$?
  echo "  [$1] rc=$rc bytes=$(wc -c < /tmp/f788_${1}.out) PASS=$(grep -ac '^PASS ' /tmp/f788_${1}.out) FAIL=$(grep -ac '^FAIL ' /tmp/f788_${1}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f788_${1}.out)"
  for f in $C1 $C2 $C3; do echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/f788_${1}.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/f788_${1}.out)"; done
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f788_${1}.out | head -6 | sed 's/^/    V| /' | cut -c1-220
}
if [ "${SKIP_HEAD:-0}" != "1" ]; then echo "=== HEAD full 33 by name: $(date -u +%H:%M:%S)Z"; runall head /tmp/w788h; else echo "HEAD arm skipped (already recorded 33/33 rc=0 in 14788-b.txt)"; fi
echo "=== BASE same binary (expect FAIL exactly C1+C2): $(date -u +%H:%M:%S)Z"
git worktree add -q --detach /tmp/w788b $BASESHA || { echo BASE_WT_FAIL; exit 9; }
[ "$(git -C /tmp/w788b rev-parse HEAD)" = "$BASESHA" ] || { echo BASE_NOT_AT_BASE; exit 9; }
runall base /tmp/w788b
echo "=== MUT A: claim1 conjunct back to bare literal (expect C1 FAIL, C2 PASS):"
cd /tmp/w788h
A=$(grep -acF 'a == cursor_agent_executable_path(version: cursor_agent_pinned_version)' $WIT || true)
[ "$A" = "1" ] || echo "MUTA_ANCHOR_UNEXPECTED got=$A -- NOT a mutation run"
python3 - <<'PY'
p='dag/test/claim/dispatch_credential_snapshot_wet_witness.dag'
s=open(p).read()
o='a == cursor_agent_executable_path(version: cursor_agent_pinned_version)'
n='a == "cursor-agent"'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  mutA applied")
PY
runall mutA /tmp/w788h
cp -f /tmp/wit788_orig $WIT
[ "$(sha $WIT)" = "$(sha /tmp/wit788_orig)" ] && echo "  RESTORE_A=YES" || { echo "RESTORE_A=NO -- STOP"; exit 9; }
echo "=== MUT B: claim2 abandoned branch back to fx_take (no register path) (expect C2 FAIL, C1 PASS):"
python3 - <<'PY'
p='dag/test/claim/dispatch_credential_snapshot_wet_witness.dag'
s=open(p).read()
o='fx_register_take(dir: abandoned_dir'
n='fx_take(dir: abandoned_dir'
assert s.count(o)==1, f"anchor {s.count(o)}"
open(p,'w').write(s.replace(o,n)); print("  mutB applied")
PY
runall mutB /tmp/w788h
cp -f /tmp/wit788_orig $WIT
[ "$(sha $WIT)" = "$(sha /tmp/wit788_orig)" ] && echo "  RESTORE_B=YES" || echo "RESTORE_B=NO"
echo "=== statics: claim1 no bare literal / import of product authority:"
grep -acF 'cursor_agent_executable_path(version: cursor_agent_pinned_version)' /tmp/wit788_orig | sed 's/^/  head_conjunct_count=/'
sed -n "/^test fn $C1/,/^}/p" /tmp/wit788_orig | grep -acF '"cursor-agent"' | sed 's/^/  claim1_bare_cursor_agent_literals=/'
grep -aE "import .*cursor_agent_executable_path|cursor_agent_pinned_version" /tmp/wit788_orig | grep -a import | head -2 | sed 's/^/  imp| /'
git status --porcelain | head -3
echo DONE_788 $(date -u +%H:%M:%S)Z
