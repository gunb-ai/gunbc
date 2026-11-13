set -u
RUSTFLAGS="-D warnings" cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > build.log 2>&1; rc=$?
echo "BUILD_RC=$rc"; [ $rc -eq 0 ] || { grep -E "^error" -A8 build.log | head -150; exit 3; }
grep -E "^warning" -A4 build.log | grep -E "never used|never constructed|never read|unused|-->" | head -200
B=$PWD/target/release/gunbc
git -c user.email=ci@local -c user.name=ci commit -q -a -m snapshot --no-verify >/dev/null 2>&1; echo "SNAPSHOT=$(git rev-parse --short HEAD) dirty=$(git status --porcelain=1 --untracked-files=no | wc -l)"
git cat-file -e fd458f74e474^{commit} 2>/dev/null || git fetch -q --depth=1 origin fd458f74e474d5bd048fdd928fe6bdb31a81a64b
git worktree add -q /tmp/base fd458f74e474 || echo "BASE_WORKTREE_FAILED"
run() { side=$1; dir=$2; shift 2; for e in "$@"; do out=$(mktemp -d); ( cd $dir && timeout 900 $B compile --source-root dag --source-root src/v2 --entry "$e" --output-dir "$out" --target dag >"$out.log" 2>&1 ); rc=$?; echo "$side rc=$rc $e :: $(grep -c '^error' $out.log) errors"; grep '^error' $out.log | cut -c1-220 | head -6; done; }
run HEAD $PWD "$@"
run BASE /tmp/base "$@"
# Finding D: grandfathered-roster membership cost, base vs head, same binary
for side in HEAD BASE; do dir=$PWD; [ $side = BASE ] && dir=/tmp/base
  for fn in every_declared_removal_names_a_cut_row_and_has_left_the_roster the_calibration_specimen_is_not_grandfathered the_roster_identity_join_is_exact_in_both_directions; do
    ( cd $dir && /usr/bin/time -f "WALL=%e RSS_KB=%M" $B run --source-root dag --source-root src/v2 --entry dag/test/claim/floor_eval_step_budget_witness.dag --claim-run --function $fn > /tmp/r.out 2>&1; echo "rc=$?" >> /tmp/r.out )
    echo "COST $side $fn :: $(grep -E 'WALL=|rc=|eval.step|steps|PASS|FAIL|true|false' /tmp/r.out | tr '\n' ' ' | cut -c1-400)"
  done
done

# Finding B: regenerate the stage0 mirrors exactly as heal does and show any drift from the committed tree
HEAL_BASE_MIRROR_COMMIT=fd458f74e474 ./target/release/claim_executor --regen-round-cost --base-mirror --regen-affected-scope --source-root dag --source-root src/v2 > regen.log 2>&1; echo "REGEN_RC=$?"
tail -15 regen.log
echo "MIRROR_DIFF_BEGIN"; git status --porcelain=1 --untracked-files=no | grep -v -E 'build.log|regen.log' ; git diff --stat | tail -20; git diff -- src/v1/stage0/src | head -120; echo "MIRROR_DIFF_END"
