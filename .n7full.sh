SHA=$1
git fetch -q --depth=1 origin $SHA && git -c advice.detachedHead=false checkout -q -f $SHA || { echo FETCHFAIL; exit 3; }
echo "rev=$(git rev-parse HEAD)"
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -5
mkdir -p /sys/fs/cgroup/swn && echo 21474836480 > /sys/fs/cgroup/swn/memory.max && echo $BASHPID > /sys/fs/cgroup/swn/cgroup.procs
GITHUB_SHA=$(git rev-parse HEAD) target/release/gunbc test //v2/test/parse/expression_bodied_fn_decl_parse:all > /tmp/n7.txt 2>&1; echo "N7 rc=$?"
grep -a "native-verdict" /tmp/n7.txt | head -1 | grep -oE "chain: .*" | sed 's/ -> /\n/g' | sed 's/<node occurrence #[0-9]*>//' | uniq -c | head -60
echo "--- node/node_query/count lines"
grep -aE "v2\.std\.node(_query)?\b|bare.*count|count\(|unbound_symbol|resolve_reason" /tmp/n7.txt | cut -c1-400 | sort | uniq -c | head -30
