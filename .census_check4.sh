set -u
mode=$1; shift
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > build.log 2>&1; rc=$?
echo "BUILD_RC=$rc"; [ $rc -eq 0 ] || { grep -E "^error" -A8 build.log | head -50; exit 3; }
B=$PWD/target/release/gunbc; export B
git -c user.email=ci@local -c user.name=ci commit -q -a -m snapshot --no-verify >/dev/null 2>&1
git fetch -q --shallow-since=2026-11-08 origin fd458f74e474d5bd048fdd928fe6bdb31a81a64b 2>&1 | tail -2; git merge-base --is-ancestor b5a924921eaa8b564a7d3e511b3514c09c9b83c9 fd458f74e474 && echo ANCESTRY_OK || echo ANCESTRY_MISSING
git worktree add -q /tmp/base fd458f74e474 || echo BASE_WORKTREE_FAILED
one() { side=$1; e=$2; dir=$PWD; [ $side = BASE ] && dir=/tmp/base; [ -f "$dir/$e" ] || { echo "$side rc=absent $e"; return; }; out=$(mktemp -d); ( cd $dir && timeout ${T:-900} $B compile --source-root dag --source-root src/v2 --entry "$e" --output-dir "$out" --target dag >"$out.log" 2>&1 ); rc=$?; echo "$side rc=$rc $e :: $(grep -c '^error' $out.log) errors $(grep '^error' $out.log | head -3 | cut -c1-200 | tr '\n' '|')"; }
export -f one
case $mode in
  compile) for e in "$@"; do echo "HEAD $e"; done | xargs -P 4 -L1 bash -c 'one $0 $1' ;;
  both) for e in "$@"; do echo "HEAD $e"; echo "BASE $e"; done | xargs -P 4 -L1 bash -c 'one $0 $1' ;;
  cost)
  for side in HEAD BASE; do dir=$PWD; [ $side = BASE ] && dir=/tmp/base
    for fn in every_declared_removal_names_a_cut_row_and_has_left_the_roster the_calibration_specimen_is_not_grandfathered the_roster_identity_join_is_exact_in_both_directions; do
      ( cd $dir && s=$(date +%s.%N); timeout 1500 $B run --source-root dag --source-root src/v2 --entry dag/test/claim/floor_eval_step_budget_witness.dag --claim-run --function $fn > /tmp/r.out 2>&1; rc=$?; echo "rc=$rc WALL=$(echo "$(date +%s.%N) - $s" | bc)" >> /tmp/r.out )
      echo "COST $side $fn :: $(tail -c 600 /tmp/r.out | tr '\n' ' ')"
    done
  done ;;
  regen)
  HEAL_BASE_MIRROR_COMMIT=fd458f74e474 ./target/release/claim_executor --regen-round-cost --base-mirror --regen-affected-scope --source-root dag --source-root src/v2 > regen.log 2>&1; echo "REGEN_RC=$?"
  tail -15 regen.log
  echo MIRROR_DIFF_BEGIN; git status --porcelain=1 --untracked-files=no | grep -v -E 'build.log|regen.log'; git diff --stat | tail -20; git diff -- src/v1/stage0/src | head -150; echo MIRROR_DIFF_END ;;
esac
