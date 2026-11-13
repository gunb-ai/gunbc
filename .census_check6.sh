set -u
jobs=$1
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > build.log 2>&1; rc=$?; echo "BUILD_RC=$rc"; [ $rc -eq 0 ] || exit 3
B=$PWD/target/release/gunbc
git add -A . ':!build.log' && git -c user.email=ci@local -c user.name=ci commit -q -m snapshot --no-verify >/dev/null 2>&1
git fetch -q --shallow-since=2026-11-08 origin fd458f74e474d5bd048fdd928fe6bdb31a81a64b >/dev/null 2>&1; git worktree add -q /tmp/base fd458f74e474 || echo BASE_WORKTREE_FAILED
while read side kind e fn; do dir=$PWD; [ $side = BASE ] && dir=/tmp/base; [ -e $dir/$e ] || { echo "$side $kind absent $e"; continue; }
  s=$(date +%s)
  if [ $kind = compile ]; then out=$(mktemp -d); ( cd $dir && timeout 1800 $B compile --source-root dag --source-root src/v2 --entry $e --output-dir $out --target dag > $out.log 2>&1 ); rc=$?; echo "$side COMPILE rc=$rc t=$(( $(date +%s)-s ))s $e :: $(grep '^error' $out.log | head -3 | cut -c1-200 | tr '\n' '|')"
  elif [ $kind = run ]; then ( cd $dir && timeout 1800 $B run --source-root dag --source-root src/v2 --entry $e --claim-run --function $fn > /tmp/o 2>&1 ); rc=$?; echo "$side RUN rc=$rc t=$(( $(date +%s)-s ))s $e::$fn :: $(grep -E 'PASS|FAIL|error' /tmp/o | tail -2 | cut -c1-200 | tr '\n' '|')"
  elif [ $kind = regen ]; then git fetch -q --shallow-since=2026-11-05 origin main census-retire-instruments 2>&1 | tail -1; git merge-base HEAD 435252e2d6276c847e25999d7c2801b0181e28f9 && echo MB_OK; HEAL_BASE_MIRROR_COMMIT=435252e2d6276c847e25999d7c2801b0181e28f9 ./target/release/claim_executor --regen-round-cost --base-mirror --source-root dag --source-root src/v2 > regen.log 2>&1; echo "REGEN_RC=$?"; grep -E 'refus|wrote|regenerat|drift' regen.log | tail -8 | cut -c1-300; echo MIRROR_DIFF_BEGIN; git status --porcelain=1 | grep -v -E 'build.log|regen.log'; git diff --stat | tail -5; git diff -- src/v1/stage0/src | head -100; echo MIRROR_DIFF_END
  else ( cd $dir && timeout 1800 $B run --source-root dag --source-root src/v2 --entry $e --function where_are_we --arg out=/tmp/standing.txt > /tmp/w 2>&1 ); echo "$side REPORT rc=$?"; echo REPORT_BEGIN; cat /tmp/standing.txt; echo REPORT_END; fi
done < $jobs
