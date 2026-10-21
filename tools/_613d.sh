set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @ d7bb6af37d92 -- parallel instrument queues. Arms: 17 witness rows by name; RED1 group
# revert; RED2 swap-read revert; NO-DRIFT through the DECLARED regen route (gunbc run main_wet_one),
# including a drift-repair control (corrupt the yml, the route must repair it byte-exact) and the
# typed unrostered refusal. Concurrency receipt is GitHub-side; not re-run here.
OID=34ad3659dbd0c9eda30a14604797f453e5bd7d5c  # gh pr view 14613 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12)"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613d
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613d 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14613d >/dev/null || { echo FETCH_FAILED; exit 9; }
[ "$(git rev-parse refs/verifier/14613d)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613d) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613d $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613d
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
AUTH=dag/gunbc/instruments/instrument_dispatch_workflow.dag
WIT=dag/test/claim/instruments/instrument_dispatch_workflow_witness.dag
YML=.github/workflows/instrument-dispatch.yml
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
cp $AUTH /tmp/613d_auth.orig; cp $YML /tmp/613d_yml.orig
echo "head-blobs auth=$(sha $AUTH|cut -c1-12) wit=$(sha $WIT|cut -c1-12) yml=$(sha $YML|cut -c1-12)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14613/release/claim_batch
GB=/root/workspace/repo-root/.t14613/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo "NO BINARIES - nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) GB_SHA=$(sha $GB|cut -c1-12) built $(date -u +%H:%M:%S)Z inside head worktree"
FNS=$(grep -aoE '^test fn [a-z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
echo "witness claim_count=$(echo $FNS | tr ',' '\n' | wc -l) (expect 17)"
run() { # tag
  local tag=$1
  echo "-------- [$tag] witness by name $(date -u +%H:%M:%S)Z"
  timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613d_$tag.out 2> /tmp/c613d_$tag.err; local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT EXCLUDED"; return 0; }
  echo "  rc=$rc out=$(wc -c < /tmp/c613d_$tag.out)B"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613d_$tag.out | sed 's/^/    V| /' | cut -c1-170
  echo "  counts: PASS=$(grep -ac '^PASS ' /tmp/c613d_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613d_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613d_$tag.out)"
  if grep -qa 'hermetic route has no arm' /tmp/c613d_$tag.err /tmp/c613d_$tag.out; then
    echo "  !! hermetic refused a host capability -> wet re-run:"
    timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613d_${tag}_wet.out 2> /tmp/c613d_${tag}_wet.err
    echo "     wet rc=$? PASS=$(grep -ac '^PASS ' /tmp/c613d_${tag}_wet.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613d_${tag}_wet.out)"
    grep -aE '^FAIL ' /tmp/c613d_${tag}_wet.out | sed 's/^/     wet FAIL| /' | cut -c1-170
  fi
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613d_$tag.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
  return 0
}
mutA() { python3 - "$1" "$2" "$3" <<'PY'
import sys
old,new,want=sys.argv[1],sys.argv[2],sys.argv[3],0
# anchor-count mode passed via want in argv[4] if present
PY
}
echo "=== A. clean at head ==="; run clean
echo "=== B. RED1: revert the concurrency group to ONE GLOBAL NAME for every label ==="
grep -c '^data instrument_dispatch_concurrency_group: String = join(' $AUTH | xargs echo "  anchor (expect 1):"
sed -i 's|^data instrument_dispatch_concurrency_group: String = join(.*|data instrument_dispatch_concurrency_group: String = "instrument-dispatch-queued"|' $AUTH
[ "$(grep -c 'instrument_dispatch_concurrency_group: String = "instrument-dispatch-queued"' $AUTH)" = "1" ] || { echo MUT1_FAILED; exit 9; }
run mut_group
git cat-file blob HEAD:$AUTH > $AUTH; cmp -s /tmp/613d_auth.orig $AUTH && echo "  RESTORED_BYTE_IDENTICAL auth" || echo "  !!RESTORE_FAILED auth"
echo "=== C. RED2: revert the swap bound read (assign + its two print words) ==="
grep -c 'bash_build_assign(name: "memory_swap_max", value: instrument_dispatch_cgroup_read(file: "memory.swap.max")),' $AUTH | xargs echo "  anchor assign (expect 1):"
grep -c 'bash_build_word_lit(text: " memory.swap.max="),' $AUTH | xargs echo "  anchor print (expect 1):"
sed -i '/bash_build_assign(name: "memory_swap_max", value: instrument_dispatch_cgroup_read(file: "memory.swap.max")),/d' $AUTH
sed -i '/bash_build_word_lit(text: " memory.swap.max="),/d; /bash_build_word_var(name: "memory_swap_max"),/d' $AUTH
[ "$(grep -c 'memory_swap_max' $AUTH)" -lt 4 ] && echo "  mutation applied (remaining memory_swap_max mentions: $(grep -c memory_swap_max $AUTH))" || { echo MUT2_NOT_APPLIED; exit 9; }
run mut_swap
git cat-file blob HEAD:$AUTH > $AUTH; cmp -s /tmp/613d_auth.orig $AUTH && echo "  RESTORED_BYTE_IDENTICAL auth" || echo "  !!RESTORE_FAILED auth"
echo "=== D. NO-DRIFT through the declared regen route ==="
echo "  D0 route rc on the head yml:"
timeout 1200 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$YML > /tmp/c613d_d0.out 2>&1; rc=$?
echo "    rc=$rc out=$(wc -c < /tmp/c613d_d0.out)B"; head -6 /tmp/c613d_d0.out | sed 's/^/    D| /' | cut -c1-180
if [ -z "$(git status --porcelain -- $YML)" ]; then echo "    D0 no-drift: route rewrote nothing or wrote identical bytes (git porcelain empty)"; else echo "    !! D0 route DRIFTED the yml:"; git diff --stat -- $YML | sed 's/^/    /'; fi
echo "  D1 DRIFT-REPAIR control: corrupt the yml, the route must repair it byte-exact"
printf '\n# verifier-stray-line (must be removed by the regen route)\n' >> $YML
echo "    corrupted to sha=$(sha $YML|cut -c1-12) (head sha=$(git cat-file blob HEAD:$YML | sha256sum | cut -d' ' -f1 | cut -c1-12))"
timeout 1200 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$YML > /tmp/c613d_d1.out 2>&1; rc=$?
echo "    rc=$rc"; head -4 /tmp/c613d_d1.out | sed 's/^/    D| /' | cut -c1-180
git cat-file blob HEAD:$YML | cmp -s - $YML && echo "    D1 REPAIRED_BYTE_EXACT (route actually writes)" || { echo "    !! D1 route did NOT repair the drift"; head -20 $YML | tail -3; }
git cat-file blob HEAD:$YML > $YML
echo "  D2 unrostered path must refuse, naming GeneratedArtifactPathUnrostered:"
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=.github/workflows/definitely_not_rostered_613.yml > /tmp/c613d_d2.out 2>&1; rc=$?
echo "    rc=$rc unrostered=$(grep -ac 'GeneratedArtifactPathUnrostered' /tmp/c613d_d2.out)"
head -5 /tmp/c613d_d2.out | sed 's/^/    D| /' | cut -c1-180
echo "=== E. witness after restores ==="; run final
cmp -s /tmp/613d_auth.orig $AUTH && echo FINAL_MATCH_auth || echo FINAL_MISMATCH_auth
cmp -s /tmp/613d_yml.orig $YML && echo FINAL_MATCH_yml || echo FINAL_MISMATCH_yml
echo DONE_613_D
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
