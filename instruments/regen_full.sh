set -u
echo "HEAD=$(git rev-parse HEAD)"
echo "unmerged=$(git diff --name-only --diff-filter=U | tr '\n' ' ')"
cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -2
echo "########## --required-regen (verify form) ##########"
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr.log 2>&1; echo "RR_EXIT=$?"
grep -vE "^◐|^\[floor|memory-cgroup" /tmp/rr.log | tail -25
echo "########## --regen-round-cost (the heal bot's write form) ##########"
./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/rc.log 2>&1; echo "RC_EXIT=$?"
grep -vE "^◐|^\[floor|memory-cgroup" /tmp/rc.log | tail -25
echo "== after: mirror fold refs=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) candidates:"
ls target/stage0-regen-candidate/src/v1/stage0/src 2>/dev/null | wc -l
echo REGEN_FULL_DONE
