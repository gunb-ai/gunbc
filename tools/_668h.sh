#!/usr/bin/env bash
# #14668 @ 85f8bc3c268b5d2bbe97eb7347fd9334a1020c66 (moved head) — mutation re-run.
# A: suppress emitted refusal text (both .to_string() sites -> ()); expect FAIL exactly single_field_ambiguous_literal_emits_the_refusal.
# B: suppress census node make_error_node(AmbiguousAnonymousRecordLiteral{...})->Rc::new(vec![]); expect FAIL exactly multi_field_ambiguous_literal_refuses_with_located_diagnostic.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver668h 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver668h
echo 51539607552 > /sys/fs/cgroup/ver668h/memory.max
RB=$(cat /sys/fs/cgroup/ver668h/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver668h memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver668h/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t668h
sha() { sha256sum "$1" | cut -d' ' -f1; }
OID=85f8bc3c268b5d2bbe97eb7347fd9334a1020c66
[ ${#OID} -eq 40 ] || { echo "OID LEN STOP"; exit 9; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w668h
git fetch -q origin +refs/pull/14668/head:refs/verifier/f668h || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f668h)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f668h) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w668h $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w668h || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo WT_NOT_AT_HEAD; exit 9; }
WIT=dag/test/claim/record_emission_candidate_set_witness.dag
EMIT=src/v1/stage0/src/v1_compiler_emit_rust.rs
[ "$(git cat-file blob $OID:$WIT | sha256sum | cut -d' ' -f1)" = "$(sha $WIT)" ] || { echo "NOT_HEAD_BLOB witness -- STOP"; exit 9; }
[ "$(git cat-file blob $OID:$EMIT | sha256sum | cut -d' ' -f1)" = "$(sha $EMIT)" ] || { echo "NOT_HEAD_BLOB emit -- STOP"; exit 9; }
cp -f $EMIT /tmp/emit668_orig
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t668h/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) $(date -u +%H:%M:%S)Z"
NAMES="single_field_literal_passes_through_bare_value,single_field_ambiguous_literal_emits_the_refusal,multi_field_literal_emits_nominal_struct,multi_field_ambiguous_literal_refuses_with_located_diagnostic"
run4() { # $1 tag $2 entry
  timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/h668_${1}.out 2>&1
  echo "  [$1] rc=$? PASS=$(grep -ac '^PASS ' /tmp/h668_${1}.out) FAIL=$(grep -ac '^FAIL ' /tmp/h668_${1}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/h668_${1}.out)"
  for f in single_field_literal_passes_through_bare_value single_field_ambiguous_literal_emits_the_refusal multi_field_literal_emits_nominal_struct multi_field_ambiguous_literal_refuses_with_located_diagnostic; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/h668_${1}.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/h668_${1}.out)"
  done
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/h668_${1}.out | head -3 | sed 's/^/    V| /' | cut -c1-230
}
echo "=== baseline head: $(date -u +%H:%M:%S)Z"; run4 base
echo "=== MUT A: suppress emitted refusal text (2 sites -> ())"
PATTERN='compile_error!(\"ambiguous anonymous record literal shape; add a nominal type\")'
A=$(grep -acF "$PATTERN" $EMIT || true)
echo "  MUTA anchors before=$A (2 expected)"
[ "$A" = "2" ] || { echo "MUTA_ANCHOR_FAIL -- NOT a mutation run"; }
perl -pi -e "s/\Q$PATTERN\E/()/g" $EMIT
AFTER=$(grep -acF "$PATTERN" $EMIT || true)
echo "  sites after=$AFTER (0 expected)"
cargo build --release -p v1-compiler --bin claim_batch > /tmp/mutA_build.log 2>&1; BC=$?
tail -1 /tmp/mutA_build.log
[ $BC -eq 0 ] || { echo "MUTA_BUILD_FAIL -- the mutA run would be the OLD binary: NOT a result"; }
echo "  mutA CB_SHA=$(sha $CB)"; run4 mutA
cp -f /tmp/emit668_orig $EMIT
[ "$(sha $EMIT)" = "$(sha /tmp/emit668_orig)" ] && echo "  RESTORE_A=YES" || { echo "RESTORE_A=NO -- STOP"; exit 9; }
echo "=== MUT B: suppress census node (make_error_node(AmbiguousAnonymousRecordLiteral{...}) -> Rc::new(vec![]))"
B=$(grep -acF 'Rc::new(vec![crate::v1_std_core::make_error_node(Rc::new(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral {' $EMIT || true)
[ "$B" = "1" ] || echo "MUTB_ANCHOR_UNEXPECTED got=$B (1 expected) -- NOT a mutation run"
perl -0777 -pi -e 's/Rc::new\(vec!\[crate::v1_std_core::make_error_node\(Rc::new\(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral \{.*?module_name\.clone\(\)\)\]\)/Rc::new(vec![])/s' $EMIT
AFTERB=$(grep -acF 'Rc::new(vec![crate::v1_std_core::make_error_node(Rc::new(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral {' $EMIT || true)
echo "  anchors before=$B after=$AFTERB"
[ "$AFTERB" = "0" ] || { echo "MUTB_REPLACE_FAIL after=$AFTERB -- the mutB run would be the OLD binary: NOT a result"; }
cargo build --release -p v1-compiler --bin claim_batch > /tmp/mutB_build.log 2>&1; BC=$?
tail -1 /tmp/mutB_build.log
[ $BC -eq 0 ] || { echo "MUTB_BUILD_FAIL -- the mutB run would be the OLD binary: NOT a result"; }
echo "  mutB CB_SHA=$(sha $CB)"; run4 mutB
cp -f /tmp/emit668_orig $EMIT
[ "$(sha $EMIT)" = "$(sha /tmp/emit668_orig)" ] && echo "  RESTORE_B=YES" || echo "RESTORE_B=NO"
git status --porcelain | head -3
echo "=== post baseline (rebuild clean):"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  post CB_SHA=$(sha $CB)"; run4 post
echo DONE_668H $(date -u +%H:%M:%S)Z
