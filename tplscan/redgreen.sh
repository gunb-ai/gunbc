set -e
CG=/sys/fs/cgroup/rg$$; mkdir -p $CG; echo $((40*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
B=$PWD/target/release/gunbc
run() { echo "=== $1"; $B run --claim-run --source-root dag --source-root src/v2 --entry src/v2/test/claim/compiler/string_template_hole_type_test.dag --function a_data_initializer_template_keeps_its_hole 2>&1 | grep -v "pre-entry\|typecheck" | tail -8; }
run FIXED
python3 - <<'PY'
p='src/v2/compiler/body_lowering_fold.dag'
s=open(p).read()
h="  } else if emitted == ^dag_string_template {\n    true\n"
assert h in s
open(p,'w').write(s.replace(h,"",1))
PY
run DEFERRAL_REVERTED
