set -e
CG=/sys/fs/cgroup/rg$$; mkdir -p $CG; echo $((40*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
for f in an_int_hole_refuses_at_the_hole a_data_initializer_template_keeps_its_hole an_unbound_hole_in_a_data_initializer_refuses; do
./target/release/gunbc run --claim-run --source-root dag --source-root src/v2 --entry src/v2/test/claim/compiler/string_template_hole_type_test.dag --function $f 2>&1 | grep -E "^(PASS|FAIL)"
done
