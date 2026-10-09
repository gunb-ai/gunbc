SHA=$1
git fetch -q --depth=1 origin $SHA && git -c advice.detachedHead=false checkout -q -f $SHA || { echo FETCHFAIL; exit 3; }
echo "rev=$(git rev-parse HEAD)"
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -5
mkdir -p /sys/fs/cgroup/swn && echo 21474836480 > /sys/fs/cgroup/swn/memory.max && echo $BASHPID > /sys/fs/cgroup/swn/cgroup.procs
GITHUB_SHA=$(git rev-parse HEAD) target/release/gunbc test //v2/test/parse/expression_bodied_fn_decl_parse:all > /tmp/n7.txt 2>&1; echo "N7 rc=$?"
grep -iE 'reason|refus|PASS|FAIL|held|member|census|node_query|concat' /tmp/n7.txt | grep -v '^\[floor-phase\|^\[interp' | cut -c1-700 | sort | uniq -c | sort -rn | head -60
