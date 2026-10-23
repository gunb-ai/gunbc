#!/usr/bin/env bash
# #14805 (keen-owl-704): delete Int? census selectors; 23 witnesses read typed census (stacked on #14783).
# Usage: _14805.sh <full-head-sha>   (head sent by manager; PR_REF_MOVED guard against refs/pull/14805/head)
# Arms:
#  S  statics at head: whole-tree grep for the 3 deleted selectors (expect 0 code hits, prose only);
#     fleet_fan uses clean_exit; sole_constructor uses sc_has_zero_blocking+census_exit.
#  F  fail-open pair on fleet_fan witness_a_broken_outside_module_is_not_clean:
#     base overlay + unterm plant in broken fixture source => PASS at base (clean()==false on refusal);
#     head + same plant => ExitFailure 2 (CensusUnobserved), head clean => PASS.
#  C  per-file claim status base==head spot-check: fleet_fan_mint_seal_witness all claims, head vs base overlay,
#     SAME head-built binary; plus guarantee_probe_corpus_witness 65+4 profile at head (author claims 65 PASS 4 FAIL combined head).
#  D  control on the other shape: plant a source that COMPILES CLEAN (add a trivial extra fn) => claim must still PASS (refusal was the cause, not any plant).
set -u
HEAD_SHA=${1:-}
[ -n "$HEAD_SHA" ] || { echo "NEED_HEAD_SHA -- STOP"; exit 9; }
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver805 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver805
echo 51539607552 > /sys/fs/cgroup/ver805/memory.max
RB=$(cat /sys/fs/cgroup/ver805/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver805 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14805
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14805/head:refs/verifier/f14805 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14805)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14805) want=$HEAD_SHA -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14805/release/claim_batch
if [ ! -x $CB ]; then
  rm -rf /tmp/w805gb; git worktree add -q --detach /tmp/w805gb $HEAD_SHA || { echo WT_FAIL; exit 9; }
  cd /tmp/w805gb && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
  BUILD_EXIT=$?; [ $BUILD_EXIT -eq 0 ] || { echo "BUILD_FAIL $BUILD_EXIT -- STOP"; exit 9; }
  cd $REPO || exit 9
fi
[ -x $CB ] || { echo "NO_BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA CB_SHA=$(sha $CB)"
[ -f tools/base14805.tgz ] || { echo "NO_BASE_TGZ -- build overlay first (tools/base14805.tgz: PR-diff files at base commit) -- STOP"; exit 9; }
FF=dag/test/claim/fleet_fan_mint_seal_witness.dag
GPC=dag/test/claim/guarantee_probe_corpus_witness.dag
rm -rf /tmp/w805h /tmp/w805b
git worktree add -q --detach /tmp/w805h $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --detach /tmp/w805b $HEAD_SHA || { echo WTB_FAIL; exit 9; }
tar xzf tools/base14805.tgz -C /tmp/w805b || exit 9
# overlay sanity: at least FF differs; every file in tgz matches its base blob (checked locally at pack time)
[ "$(sha /tmp/w805h/$FF)" != "$(sha /tmp/w805b/$FF)" ] || { echo "OVERLAY_NOOP -- STOP"; exit 9; }
echo "=== S statics: $(date -u +%H:%M:%S)Z"
HITS=$(cd /tmp/w805h && grep -rn 'census_blocking_count_for_class\|census_blocking_count_for_subject\|census_any_count_for_class' dag src 2>/dev/null | grep -v '^\s*//' | wc -l)
echo "  [static] deleted-selector code refs at head = $HITS (expect 0)"
grep -c 'clean_exit' /tmp/w805h/$FF | sed 's/^/  [static] fleet clean_exit = /'
grep -c 'census_exit' /tmp/w805h/dag/test/claim/sole_constructor_type_head_exposure_witness.dag | sed 's/^/  [static] sole census_exit = /'
grep -n 'fn clean(' /tmp/w805b/$FF | sed 's/^/  [static] base had fn clean(: /' | head -2
run() { local tag=$1 tree=$2 entry=$3; shift 3
  cd $tree || exit 9
  timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions "$*" > /tmp/w805_$tag.out 2>&1
  echo "  [$tag] rc=$? PASS=$(grep -ac '^PASS ' /tmp/w805_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/w805_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/w805_$tag.out) BYTES=$(wc -c < /tmp/w805_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/w805_$tag.out | sed 's/^/    V| /' | cut -c1-200 | head -12
  cd $REPO || exit 9
}
echo "=== F/C fleet arms (names enumerated from head blob at run time): $(date -u +%H:%M:%S)Z"
FFNAMES=$(grep -oE '^test fn [a-z_0-9]+' /tmp/w805h/$FF | awk '{print $3}' | paste -sd, -)
echo "  fleet claims: $FFNAMES"
run ff_head_clean /tmp/w805h $FF $FFNAMES
run ff_base_clean /tmp/w805b $FF $FFNAMES
# fail-open pair: plant unterm string in the broken fixture source (escaped \n, witness parse survives)
python3 - /tmp/w805b/$FF <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
import re
m=re.search(r'data broken_[a-z_]*source: String = "module [^"]*', s)
if not m: print("  NO broken_*source ANCHOR at base -- STOP"); sys.exit(1)
a=m.group(0)
open(p,'w').write(s[:m.start()]+a[:-1]+'fn broken() -> Int { let s = \\"abc\\n'+s[m.end():]); print("  plant unterm applied (base)")
PY
run ff_base_plant /tmp/w805b $FF witness_a_broken_outside_module_is_not_clean
cp -f /tmp/w805b/$FF /tmp/x_base_plant.ff
python3 - /tmp/w805h/$FF <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
import re
m=re.search(r'data broken_[a-z_]*source: String = "module [^"]*', s)
if not m: print("  NO broken_*source ANCHOR at head -- STOP"); sys.exit(1)
a=m.group(0)
open(p,'w').write(s[:m.start()]+a[:-1]+'fn broken() -> Int { let s = \\"abc\\n'+s[m.end():]); print("  plant unterm applied (head)")
PY
run ff_head_plant /tmp/w805h $FF witness_a_broken_outside_module_is_not_clean
git -C /tmp/w805h checkout -q -- $FF
echo "=== D control on other shape: head + CLEAN-compiling extra fn plant => claim still PASS (refusal, not any-plant, was the cause): $(date -u +%H:%M:%S)Z"
python3 - /tmp/w805h/$FF <<'PY' || exit 9
import sys,re
p=sys.argv[1]; s=open(p).read()
m=re.search(r'data broken_[a-z_]*source: String = "module [^"]*', s)
a=m.group(0)
open(p,'w').write(s[:m.start()]+a[:-1]+'fn harmless() -> Int { 1 }\n'+s[m.end():]); print("  clean-fn plant applied (head)")
PY
run ff_head_cleanplant /tmp/w805h $FF witness_a_broken_outside_module_is_not_clean
echo DONE_14805
