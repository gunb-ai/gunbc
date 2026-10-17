set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:$PATH"
export RUSTUP_HOME=/opt/rustup
export GITHUB_SHA="mergecheck"
export GITHUB_RUN_ID="ff568-mergecheck"
export GUNBC_MEMORY_BUDGET_BYTES=60000000000

HEAD_SHA=9d9fb95501b6cc5a0b8580a24a5bb0cb22b73aad
MAIN_SHA=5ef31256cc019c70bfdd6d6eebcd322e2a33eaaa

echo "=== ff568 MERGE-COMMIT REGEN PARITY check (NOTHING PUSHED; runner-side merge only) ==="
echo "pr_head=$HEAD_SHA  main=$MAIN_SHA"

rm -rf /tmp/ff568-mc
git clone -q --no-checkout /root/workspace/repo-root /tmp/ff568-mc 2>&1 | head -2
cd /tmp/ff568-mc
git fetch -q https://github.com/gunb-ai/gunbc "main:refs/heads/mcmain" 2>&1 | head -3
git fetch -q https://github.com/gunb-ai/gunbc "session/fierce-fox-568:refs/heads/mchead" 2>&1 | head -3
git checkout -qf mchead 2>&1 | head -2
echo "pr_head_check=$(git rev-parse HEAD)  (want $HEAD_SHA)"
[ "$(git rev-parse HEAD)" = "$HEAD_SHA" ] && echo "PIN_OK" || { echo "PIN_MISMATCH"; exit 9; }
git --no-pager log -1 --format='%H %s' mcmain
git merge --no-ff --no-edit -m "MERGCHECK merge main into pr head (throwaway, never pushed)" mcmain > /tmp/ff568_merge.log 2>&1
echo "merge_rc=$?"
tail -6 /tmp/ff568_merge.log
echo "merge_commit=$(git rev-parse HEAD)"
echo "conflicted_files=$(git diff --name-only --diff-filter=U | wc -l)"
echo "tracked_worktree_dirty=$(git status --porcelain | wc -l)"

echo "--- what the merge brought in, restricted to MY four files (want 0) ---"
git diff --name-only "$HEAD_SHA"..HEAD -- \
  src/v1/04_infer.dag \
  src/v1/stage0/src/v1_compiler_infer.rs \
  dag/gunbc/recurring_failure_mode/declared_type_wall_keyed_on_the_value_being_a_kernel.dag \
  dag/test/claim/declared_type_kernel_inhabitance_witness.dag
echo "(empty above = main moved none of my files)"
echo "files_brought_by_merge=$(git diff --name-only "$HEAD_SHA"..HEAD | wc -l)"

echo "--- verity of the merge commit's judge ---"
echo "gate_dag=$(grep -o 'actual_base.children |> count) > 0' src/v1/04_infer.dag | wc -l) (want 0)"
echo "fixmarker_dag=$(grep -c 'THE VALUE SHAPE IS NOT THE WALL' src/v1/04_infer.dag) (want 1)"
echo "gate_seed=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l) (want 0)"
echo "inhabited_join_types_seed=$(grep -o 'inhabited_join_types' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
md5sum src/v1/stage0/src/v1_compiler_infer.rs
echo "claim_file_sha256=$(sha256sum dag/test/claim/declared_type_kernel_inhabitance_witness.dag | cut -c1-16) (want 11f0db5bcf5bda65)"
echo "--- is the merge commit's mirror identical to the one I already certified at 9d9fb955? ---"
git cat-file -e "$HEAD_SHA":src/v1/stage0/src/v1_compiler_infer.rs && \
  if diff -q <(git show "$HEAD_SHA":src/v1/stage0/src/v1_compiler_infer.rs) src/v1/stage0/src/v1_compiler_infer.rs >/dev/null; then
    echo "MIRROR_IDENTICAL_TO_CERTIFIED_HEAD"
  else
    echo "MIRROR_DIFFERS_FROM_CERTIFIED_HEAD"
  fi

echo "=== BUILD claim_executor from the MERGE COMMIT's committed seed ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -3

echo "=== ROUND A: required-regen on the merge commit (installs nothing == parity holds) ==="
rm -rf target/stage0-regen-candidate
timeout 2400 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/ff568_regenMC.log 2>&1
echo "rcA=$?"
grep -E "refused:|drift|FAIL|first_generation_equal|adjudicated|declared_divergent" /tmp/ff568_regenMC.log | head -14
CHANGED=""
for f in target/stage0-regen-candidate/src/*.rs; do
  [ -e "$f" ] || continue
  b=$(basename "$f")
  if [ -f "src/v1/stage0/src/$b" ]; then
    cmp -s "$f" "src/v1/stage0/src/$b" || { echo "DIFF $b"; CHANGED="$CHANGED $b"; }
  fi
done
echo "CHANGED_A:[$CHANGED]"
if [ -z "$CHANGED" ]; then
  echo "VERDICT: MERGE COMMIT IS REGEN-CONVERGED (round A installed nothing) -> merging 9d9fb955 into current main needs NO re-regen"
else
  echo "VERDICT: MERGE COMMIT HAS GENERATED DRIFT -> the merge would need a regen commit"
  for b in $CHANGED; do
    echo "--- first hunk: $b ---"
    diff -u "src/v1/stage0/src/$b" "target/stage0-regen-candidate/src/$b" | head -40
  done
  echo "=== BEGIN-DIFF ==="
  diff -ru --src-label a/ --label b/ src/v1/stage0/src target/stage0-regen-candidate/src 2>/dev/null | gzip -9 | base64 -w0
  echo ""
  echo "=== END-DIFF ==="
fi
echo "=== MERGE-CHECK SCRIPT DONE ==="
