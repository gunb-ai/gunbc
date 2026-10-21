set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14699 @ 540449ac80adce5210c79162c5e2122499c4f1b5 (sunny-bear-433, parse-skip flip; head from
# gh pr view 14699 --json headRefOid; manager moved it from 484ec19d for citation fixes).
# Arms: (1) witness claims by name wet -> 2 PASS rc 0; (2) manual child RED (unparsable pool file
# planted) -> rc 1, located refusal, no verdict line; control (healthy twin) -> rc 0 PASS;
# (3) MUTATION: delete the refusal call line in coproduct_reflection.rs -> RED claim FAILs,
#     control still PASSes; restore byte-exact; (4) grep: files_scanned/files_parsed deleted from
#     the Rust surface (base has them, head does not outside docs/RFM prose); (5) RFM row lists the
#     3 deferred skippers with triggers; (6) one decl_facts consumer over [dag, src/v2] at head
#     does NOT refuse.
# ALL claim_batch runs execute inside this dispatch's bounded cgroup subtree (read-back asserted).
OID=540449ac80adce5210c79162c5e2122499c4f1b5
BASE=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1
WIT=dag/test/claim/decl_facts_walk_refusal_witness.dag
CP=src/v1/stage0/src/coproduct_reflection.rs
echo "HEADSHA_hardcoded_from_gh=$OID"
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver699 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver699
echo 51539607552 > /sys/fs/cgroup/ver699/memory.max
RB=$(cat /sys/fs/cgroup/ver699/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver699 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver699/cgroup.procs; echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w699
git fetch -q origin +refs/pull/14699/head:refs/verifier/14699 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14699)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/14699) want=$OID STOP"; exit 9; }
git worktree add -q --detach /tmp/w699 $OID || { echo WT_FAIL; exit 9; }
WT=/tmp/w699
cd $WT || exit 9
[ "$(git cat-file blob $OID:$WIT | sha256sum | cut -d' ' -f1)" = "$(sha $WIT)" ] || { echo "WIT_NOT_HEAD_BLOB"; exit 9; }
[ "$(git cat-file blob $OID:$CP | sha256sum | cut -d' ' -f1)" = "$(sha $CP)" ] || { echo "CP_NOT_HEAD_BLOB"; exit 9; }
echo "STAGED_OK (witness + coproduct_reflection == head blobs)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t699 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t699/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) $(date -u +%H:%M:%S)Z"
mkdir -p $WT/target/release && cp -f $CB $WT/target/release/claim_batch
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
echo "claim names: $FNS"

# ---- manual child reproduction of what the witness pins ----
plant() { # $1=dir $2=pooltag(healthy|broken)
  rm -rf "$1"; mkdir -p "$1/src" "$1/pool"
  printf 'module probe.decl_facts_walk_entry\n\nimport std.types { Bool }\nimport std.algebra { length }\nimport v2.std.decl_index { decl_facts }\n\ntest fn decl_facts_walk_entry_holds() -> Bool {\n  length(decl_facts(pool_roots: ["%s/pool"])) > 0\n}\n' "$1" > "$1/src/decl_facts_walk_entry.dag"
  printf 'module probe.decl_facts_walk_pool\n\nfn decl_facts_walk_pool_value() -> Bool {\n  true\n}\n' > "$1/pool/decl_facts_walk_pool.dag"
  if [ "$2" = broken ]; then
    printf 'module probe.decl_facts_walk_pool_broken\n\nfn decl_facts_walk_pool_broken_value() -> Bool {\n  true\n' > "$1/pool/decl_facts_walk_second.dag"
  else
    printf 'module probe.decl_facts_walk_pool_broken\n\nfn decl_facts_walk_pool_broken_value() -> Bool {\n  true\n}\n' > "$1/pool/decl_facts_walk_second.dag"
  fi
}
manual() { # label pooltag
  local dir=$WT/target/decl-facts-walk-manual-$1
  plant "$dir" $2
  echo "-------- [child-$1] pool=$2 $(date -u +%H:%M:%S)Z"
  ( cd $WT && timeout 600 ./target/release/claim_batch --claim-run --source-root dag --source-root src/v2 --source-root "$dir/src" --entry "$dir/src/decl_facts_walk_entry.dag" --function decl_facts_walk_entry_holds ) > /tmp/c699_$1.out 2>&1
  local rc=$?
  echo "  rc=$rc output=$(wc -c < /tmp/c699_$1.out)B"
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/c699_$1.out) FAIL=$(grep -ac '^FAIL ' /tmp/c699_$1.out) refusal=$(grep -ac 'decl_facts corpus walk refused' /tmp/c699_$1.out) located=$(grep -ac 'decl_facts_walk_second.dag' /tmp/c699_$1.out)"
  grep -aE 'refused|PASS |FAIL |panic|REFUSAL' /tmp/c699_$1.out | head -6 | sed 's/^/    O| /' | cut -c1-190
  rm -rf "$dir"
}
echo "=== A. MANUAL child: RED (broken pool file) + control (healthy) ==="
manualared=""; manual red broken
manual ctl healthy
echo "=== B. WITNESS claims wet, by name (head) ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c699_wit_base.out 2>&1
RC=$?
echo "  rc=$RC ($(date -u +%H:%M:%S)Z)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c699_wit_base.out | sed 's/^/    V| /' | cut -c1-190
[ "$RC" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c699_wit_base.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== C. MUTATION: delete the refusal call (the walk skips again) ==="
grep -c 'crate::cli_run::refuse_unparseable_corpus_sources("decl_facts corpus walk refused", &refusals);' $CP | xargs echo "  anchor count (must be 1):"
sed -i 's|crate::cli_run::refuse_unparseable_corpus_sources("decl_facts corpus walk refused", &refusals);|// MUTATION refusal-line deleted|' $CP
[ "$(grep -c 'MUTATION refusal-line deleted' $CP)" = "1" ] || { echo "MUTATION_NOT_APPLIED"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
cp -f $CB $WT/target/release/claim_batch
echo "  rebuilt mutated $(date -u +%H:%M:%S)Z CB_SHA=$(sha $CB | cut -c1-12)"
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c699_wit_mut.out 2>&1
echo "  rc=$? verdicts:"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c699_wit_mut.out | sed 's/^/    V| /' | cut -c1-190
manual mred broken
echo "  restore:"
git cat-file blob HEAD:$CP > $CP
[ "$(sha $CP)" = "$(git cat-file blob $OID:$CP | sha256sum | cut -d' ' -f1)" ] && echo "  RESTORED_OK byte-exact" || { echo "RESTORE_FAIL"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
cp -f $CB $WT/target/release/claim_batch
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c699_wit_rest.out 2>&1
echo "  restored rc=$? ($(date -u +%H:%M:%S)Z)"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c699_wit_rest.out | sed 's/^/    V| /' | cut -c1-190
echo "=== D. counters deleted grep ==="
cd $WT
echo "  base(201fe0bd) occurrences in rust:"; ( cd /root/workspace/repo-root && git grep -c 'files_scanned\|files_parsed' $BASE -- 'src/v1/stage0/src/cli_run.rs' 'src/v1/stage0/src/cli_run/complexity_gates.rs' 2>/dev/null )
echo "  head: decl_facts-walk counters gone? (non-doc hits below are unrelated locals named module_count-log; the STRUCT fields must be gone)"
grep -n 'files_scanned' $CP src/v1/stage0/src/cli_run.rs src/v1/stage0/src/cli_run/complexity_gates.rs 2>/dev/null | head -5
[ "$(grep -c 'files_scanned' $CP)" = "0" ] && echo "  decl_facts walk: files_scanned GONE from coproduct_reflection" || echo "  STILL PRESENT in coproduct_reflection"
[ "$(grep -c 'pub files_scanned' src/v1/stage0/src/cli_run.rs)" = "0" ] && echo "  DeclFactsWalkSummary struct fields GONE from cli_run.rs" || { echo "  struct fields STILL PRESENT"; grep -n 'pub files_scanned\|pub files_parsed' src/v1/stage0/src/cli_run.rs | head -4 | sed 's/^/    /'; }
grep -n 'files_parsed' src/v1/stage0/src/coproduct_reflection.rs | head -3 | sed 's/^/  leftover-coprod: /'
echo "=== E. RFM row: 3 deferred skippers with triggers ==="
RFM=dag/gunbc/recurring_failure_mode/a_corpus_walk_skips_the_file_it_cannot_parse.dag
for w in parse_module_heads_tolerant importer_roots_have_importless_dag_files project_layer_import_syntax_facts; do
  c=$(grep -c "$w" $RFM); t=$(grep -A2 "$w" $RFM | grep -c 'trigger:' || true)
  echo "  $w: mentioned=$c"
done
grep -c 'trigger:' $RFM | xargs echo "  total trigger: lines:"
grep -n 'trigger:' $RFM | head -8 | cut -c1-150 | sed 's/^/    /'
echo "=== F. one decl_facts consumer over [dag, src/v2] at head: must NOT refuse ==="
CONS=$(grep -rl 'decl_facts(pool_roots' src/v2/test/claim | head -3)
echo "  consumer files: $CONS"
for f in $CONS; do
  N=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $f | sed 's/^test fn //' | sort | head -6 | paste -sd,)
  [ -n "$N" ] || continue
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $f --functions "$N" > /tmp/c699_cons.out 2>&1
  rc=$?
  echo "  [$f] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c699_cons.out) FAIL=$(grep -ac '^FAIL ' /tmp/c699_cons.out) refuse_text=$(grep -ac 'decl_facts corpus walk refused' /tmp/c699_cons.out) (corpus-refusal must be 0)"
  grep -aE '^(PASS|FAIL) ' /tmp/c699_cons.out | head -3 | sed 's/^/    V| /' | cut -c1-150
done
git status --porcelain | sed 's/^/  dirty: /'
echo DONE_699
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
