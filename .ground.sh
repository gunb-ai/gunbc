export CARGO_TARGET_DIR=$PWD/target
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
CG=/sys/fs/cgroup/gr; mkdir -p $CG; echo 25769803776 > $CG/memory.max; echo $$ > $CG/cgroup.procs
echo "HEAD $(git rev-parse HEAD)"; git status --short | grep -v '^?? [0-9a-f]\{64\}$'
./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/lens/grounding.dag --function __no_such_function__ 2>&1 | grep -v "started\|done in" | tail -8 | cut -c1-300
