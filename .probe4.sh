unset GUNBC_BIND_MEMORY_CGROUP_BYTES
cargo build --release 2>&1 | tail -1
G=$(find . -path '*release/gunbc' -type f | head -1)
$G compile --source-root dag --source-root src/v2 --entry src/v2/compiler/source_root_eval.dag --output-dir /tmp/pout9 --target rust > /tmp/p4.txt 2>&1
echo P4OUT; grep -vn "^field 'value'" /tmp/p4.txt | head -70 | cut -c1-260
echo P4UNIQ; sort /tmp/p4.txt | uniq -c | sort -rn | head -12 | cut -c1-200
