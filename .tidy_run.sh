cargo build --release --bin gunbc 2>&1 | tail -1
mkdir -p /sys/fs/cgroup/gb2 && echo 12884901888 > /sys/fs/cgroup/gb2/memory.max && echo $$ > /sys/fs/cgroup/gb2/cgroup.procs
T=src/v2/test/claim/floor/pure_producer_share_refusal_test.dag
for f in $(grep -o '^test fn [a-z_0-9]*' $T | awk '{print $3}'); do
  echo "CLAIM $f :: $(./target/release/gunbc run --source-root dag --source-root src/v2 --entry $T --function $f 2>&1 | grep -v '^\[' | grep -v '^\s*$' | tail -3 | tr '\n' ' ' | cut -c1-300)"
done
