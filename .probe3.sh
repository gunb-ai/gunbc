unset GUNBC_BIND_MEMORY_CGROUP_BYTES
cargo build --release 2>&1 | tail -1
G=$(find . -path '*release/gunbc' -type f | head -1)
$G compile --source-root dag --source-root src/v2 --entry src/v2/compiler/body_lowering_fold.dag --output-dir /tmp/pout9 --target rust 2>&1 | grep -E "no field|compiled|blocking" | head -8 | cut -c1-250
