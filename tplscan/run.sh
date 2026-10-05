set -e
cargo build --release -q --bin gunbc 2>&1 | tail -3
B=$PWD/target/release/gunbc
run() { $B run --source-root dag --source-root "$1" --source-root tplscan --entry tplscan/drive.dag --function run; }
echo "=== AFTER"; run src/v2 > tplscan/after.txt 2>&1 || echo "exit $?"; cat tplscan/after.txt | tail -100
rm -rf /tmp/v2main && cp -r src/v2 /tmp/v2main && cp tplscan/dag_main.dag.txt /tmp/v2main/extdeps/languages/dag.dag
echo "=== BEFORE"; run /tmp/v2main > tplscan/before.txt 2>&1 || echo "exit $?"; cat tplscan/before.txt | tail -100
