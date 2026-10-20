unset GUNBC_BIND_MEMORY_CGROUP_BYTES
cargo build --release 2>&1 | tail -1
G=$(find . -path '*release/gunbc' -type f | head -1)
$G compile --source-root zzprobe --source-root dag --source-root src/v2 --entry zzprobe/use.dag --output-dir /tmp/pout9 --target rust 2>&1 | grep -v "^$" | cut -c1-300 | tail -6
