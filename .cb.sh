cargo build --release -q -p v1-compiler --bin claim_batch 2>&1 | tail -5
mkdir -p /sys/fs/cgroup/swcb && echo 21474836480 > /sys/fs/cgroup/swcb/memory.max && echo $BASHPID > /sys/fs/cgroup/swcb/cgroup.procs
for m in compiler/free_monoid_structure_bound_test type_application_kind_test binder_node_test; do
f=src/v2/test/claim/$m.dag; fns=$(grep -oP '^test fn \K\w+' $f | paste -sd,)
target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" > /tmp/o.txt 2>&1; echo "== $m rc=$?"
grep -vE '^\[(floor-phase|interp-stats|data-profile|pre-entry|process-partition)' /tmp/o.txt | cut -c1-200 | tail -30
done
