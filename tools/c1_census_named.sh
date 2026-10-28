set -uo pipefail
echo "SUBJECT_SHA=$(git rev-parse HEAD)"
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_BIND_MEMORY_CGROUP_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES
export CARGO_TARGET_DIR="$PWD/target"; export GITHUB_ACTIONS=true
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
mkdir -p /sys/fs/cgroup/gunbc.bound 2>/dev/null || true
echo 51539607552 > /sys/fs/cgroup/gunbc.bound/memory.max 2>/dev/null || true
echo $$ > /sys/fs/cgroup/gunbc.bound/cgroup.procs 2>/dev/null || true
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
rm -rf target/c1-cn && mkdir -p target/c1-cn
./target/release/gunbc compile --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir target/c1-cn > /tmp/cn.txt 2>&1
echo "census_rc=$? bytes=$(wc -c </tmp/cn.txt)"
echo "invalid_cast_lines=$(grep -ac 'invalid cast' /tmp/cn.txt || true)"
echo "invalid_cast_as_String=$(grep -ac "invalid cast.*as String" /tmp/cn.txt || true)"
echo "=== the lines ==="
grep -a "invalid cast" /tmp/cn.txt | head -20
echo "=== blocking error count ==="
grep -aiE "blocking error" /tmp/cn.txt | tail -2
echo MARKER_CENSUS_NAMED_DONE
