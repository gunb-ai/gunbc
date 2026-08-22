set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
run() { echo "=== $2 ==="; $B run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1 | grep -E "returned|cause: |error:" | head -4 || true; }
E=dag/test/claim/live_deploy/emit_test.dag
for f in emit_RED_unit_files_are_not_written_through_here_documents emit_staged_unit_body_is_quoted_by_the_posix_authority witness_apply_script_contains_systemd_and_tailscale; do run $E $f; done
