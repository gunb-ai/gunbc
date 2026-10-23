#!/usr/bin/env bash
# #14805 dispatch B (part A done: statics + fleet head batch 8/8 PASS, base batch 7/8 then timeout, head-only RED 1/1).
# Arms here:
#  P1 fail-open RED: base overlay + BROKEN-source unterm plant (escape-safe insert) => witness_a_broken... still PASS (defect)
#  P2 repair:        head + same plant => FAIL rc=1 (census_exit ExitFailure 2, claimed cause CensusUnobserved)
#  P3 other-shape:   head + CLEAN-fn plant (fixture compiles, histogram equal) => same claim still PASS (plant alone isn't the cause)
#  P4 head-only claim at BASE => rc=2 zero-verdict (fn absent at base; proves the RED claim is head-new)
#  P5 base 8th claim single-name run (A's base batch was timeout-killed before it printed)
#  P6 alias_cycle head batch (7) vs base batch (7), SAME head-built binary => base==head parity
#  P7 sole_constructor head batch (6) vs base batch (6) => base==head parity; sc fail-closed at base both green
#  P8 GPC profile check at head: the 4 author-declared reds + 2 green controls by name => expect FAIL=4 PASS=2
set -u
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD_SHA=dfe7025c3903e98f988530e523f0a47e945faf19
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver805b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver805b
echo 51539607552 > /sys/fs/cgroup/ver805b/memory.max
RB=$(cat /sys/fs/cgroup/ver805b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver805b memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14805
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14805/head:refs/verifier/f14805b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14805b)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14805b) want=$HEAD_SHA -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14805/release/claim_batch
[ -x $CB ] || { echo NO_BINARY_BUILDING; rm -rf /tmp/w805gb2; git worktree add -q --detach /tmp/w805gb2 $HEAD_SHA && cd /tmp/w805gb2 && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2; cd $REPO; }
[ -x $CB ] || { echo "NO_BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA CB_SHA=$(sha $CB) (expect 7dedb0e848f4... if warm)"
FF=dag/test/claim/fleet_fan_mint_seal_witness.dag
AC=dag/test/claim/alias_cycle_refusal_witness.dag
SC=dag/test/claim/sole_constructor_type_head_exposure_witness.dag
GPC=dag/test/claim/guarantee_probe_corpus_witness.dag
rm -rf /tmp/w805bh /tmp/w805bb
git worktree add -q --detach /tmp/w805bh $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --detach /tmp/w805bb $HEAD_SHA || { echo WTB_FAIL; exit 9; }
tar xzf tools/base14805.tgz -C /tmp/w805bb || exit 9
[ "$(sha /tmp/w805bh/$FF)" != "$(sha /tmp/w805bb/$FF)" ] || { echo "OVERLAY_NOOP -- STOP"; exit 9; }
echo BASE_OVERLAY_OK
BROK=fn\ broken\(\)\ -\>\ Int\ \{\ let\ s\ =\ \\\\\"\\
run() { local tag=$1 tree=$2 entry=$3 to=$4; shift 4
  cd $tree || exit 9
  timeout $to $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions "$*" > /tmp/w805b_$tag.out 2>&1
  echo "  [$tag] rc=$? PASS=$(grep -ac '^PASS ' /tmp/w805b_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/w805b_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/w805b_$tag.out) BYTES=$(wc -c < /tmp/w805b_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE '^(PASS|FAIL|UNOBSERVED) |Census|cause|exit' /tmp/w805b_$tag.out | grep -aiv 'pass claim' | head -10 | sed 's/^/    V| /' | cut -c1-235
  cd $REPO || exit 9
}
plant_brk() { python3 - "$1" <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
BS=chr(92); anchor='data broken_outside_source: String = "module probe_fan_legal'+BS+'n'
assert s.count(anchor)==1, f"anchor={s.count(anchor)}"
ins='fn broken() -> Int { let s = '+BS+chr(34)+'abc'+BS+'n'
open(p,'w').write(s.replace(anchor, anchor+ins)); print("  plant BROKEN applied (escaped, witness stays parseable)")
PY
}
plant_ok() { python3 - "$1" <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
BS=chr(92); anchor='data broken_outside_source: String = "module probe_fan_legal'+BS+'n'
assert s.count(anchor)==1, f"anchor={s.count(anchor)}"
ins='fn harmless_'+str(abs(hash(p))%997)+'() -> Int { 1 }'+BS+'n'
open(p,'w').write(s.replace(anchor, anchor+ins)); print("  plant OK applied (fixture still compiles)")
PY
}
echo "=== P1 base + broken plant (expect PASS = fail-open defect): $(date -u +%H:%M:%S)Z"
plant_brk /tmp/w805bb/$FF || exit 9
run p1_bbroken /tmp/w805bb $FF 850 witness_a_broken_outside_module_is_not_clean
echo "=== P2 head + same plant (expect FAIL rc=1 with CensusUnobserved/exit-2): $(date -u +%H:%M:%S)Z"
plant_brk /tmp/w805bh/$FF || exit 9
run p2_hbroken /tmp/w805bh $FF 850 witness_a_broken_outside_module_is_not_clean
echo "=== P3 head + clean-fn plant (expect PASS = other-shape control): $(date -u +%H:%M:%S)Z"
git -C /tmp/w805bh checkout -q -- $FF
plant_ok /tmp/w805bh/$FF || exit 9
run p3_hok /tmp/w805bh $FF 850 witness_a_broken_outside_module_is_not_clean
echo "=== P4 head-only claim at base (expect rc=2 zero-verdict; fn absent at base): $(date -u +%H:%M:%S)Z"
run p4_baseonly /tmp/w805bb $FF 300 a_census_that_cannot_run_is_unobserved_not_clean
echo "=== P5 base 8th claim single-name (A batch timed out before it): $(date -u +%H:%M:%S)Z"
git -C /tmp/w805bb checkout -q -- $FF
run p5_b8th /tmp/w805bb $FF 850 witness_indirect_subject_through_variant_subject_is_refused
echo "=== P6 alias parity (names from blobs): $(date -u +%H:%M:%S)Z"
ACN=$(grep -oE '^test fn [a-z_0-9]+' /tmp/w805bh/$AC | awk '{print $3}' | paste -sd, -)
run p6_ac_head /tmp/w805bh $AC 1400 $ACN
run p6_ac_base /tmp/w805bb $AC 1400 $ACN
echo "=== P7 sole parity: $(date -u +%H:%M:%S)Z"
SCN=$(grep -oE '^test fn [a-z_0-9]+' /tmp/w805bh/$SC | awk '{print $3}' | paste -sd, -)
run p7_sc_head /tmp/w805bh $SC 1400 $SCN
run p7_sc_base /tmp/w805bb $SC 1400 $SCN
echo "=== P8 GPC profile: 4 author-reds + 2 greens by name (expect FAIL=4 PASS=2): $(date -u +%H:%M:%S)Z"
GPCN="bare_none_generic_parameter_field_is_outside_the_wall_boundary,sole_constructor_sanctioned_mint_green_control_has_zero_diagnostics,sole_constructor_admitted_caller_green_control_has_zero_diagnostics,dark_suite_dispositions_cover_migrated_v1_probes,bare_none_green_control_has_zero_diagnostics,identity_registry_is_unique"
run p8_gpc /tmp/w805bh $GPC 1500 $GPCN
echo DONE_14805B
