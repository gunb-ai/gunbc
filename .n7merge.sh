BASE=$1; HEAD=$2; MODE=$3
git fetch -q origin $BASE $HEAD || { echo FETCHFAIL; exit 3; }
git -c advice.detachedHead=false checkout -q -f $BASE
if [ "$MODE" = merged ]; then git -c user.email=x@x -c user.name=x merge -q --no-edit $HEAD >/tmp/m.txt 2>&1 || { echo "MERGE CONFLICT"; git diff --name-only --diff-filter=U; exit 4; }; fi
echo "rev=$(git rev-parse HEAD) mode=$MODE"
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -5
mkdir -p /sys/fs/cgroup/swm && echo 21474836480 > /sys/fs/cgroup/swm/memory.max && echo $BASHPID > /sys/fs/cgroup/swm/cgroup.procs
GITHUB_SHA=$(git rev-parse HEAD) target/release/gunbc test //v2/test/parse/expression_bodied_fn_decl_parse:all > /tmp/n7.txt 2>&1; echo "N7 rc=$?"
grep -a "native-verdict" /tmp/n7.txt | head -1 | grep -oE "chain: .*" | sed 's/ -> /\n/g' | sed 's/<node occurrence #[0-9]*>//' | grep -E "resolve_" | sort | uniq -c
