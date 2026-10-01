set -u
SHA=2237a1095cba810b57d54e39ece8da9045d45bb7; SIDE=HEAD
A=/root/workspace/artifacts/command-0; mkdir -p $A
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null; mkdir -p /sys/fs/cgroup/census && echo 24G > /sys/fs/cgroup/census/memory.max && echo $$ > /sys/fs/cgroup/census/cgroup.procs && echo "cgroup memory.max=$(cat /sys/fs/cgroup/census/memory.max)"
export GUNBC_MEMORY_BUDGET_BYTES=23622320128
git fetch -q origin $SHA; git checkout -q --detach -f $SHA; export GITHUB_SHA=$(git rev-parse HEAD); echo "$SIDE rev=$GITHUB_SHA"
cargo build --release -p v1-compiler --bin gunbc > /tmp/b.log 2>&1 || { echo BUILD_FAILED; tail -5 /tmp/b.log; exit 1; }
echo "$SIDE gunbc_sha256=$(sha256sum target/release/gunbc | cut -c1-64)"
target/release/gunbc test //gunbc/instruments:v2-native-frontier > /tmp/f.log 2>&1; echo "$SIDE exit=$?"
grep '^\[native-verdict\]' /tmp/f.log | sort | gzip -9 > $A/verdicts_$SIDE.gz
gzip -9c /tmp/f.log > $A/frontier_$SIDE.log.gz
echo "$SIDE verdict_lines=$(zcat $A/verdicts_$SIDE.gz | wc -l)"
grep -E '^v2-native-(route|frontier):' /tmp/f.log | cut -c1-300
for f in verdicts_$SIDE.gz frontier_$SIDE.log.gz; do echo "DIGEST $f $(sha256sum $A/$f | cut -c1-64)/$(stat -c%s $A/$f)"; done
