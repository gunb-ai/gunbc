set -u
echo "budget=${GUNBC_MEMORY_BUDGET_BYTES:-UNSET}"
R="--source-root dag --source-root src/v2"
git log --oneline -1
cargo build --release --bin claim_executor 2>&1 | tail -1
./target/release/claim_executor --required-regen $R > /tmp/r1.log 2>&1; echo "regen rc=$?"; grep "required-regen:" /tmp/r1.log | cut -c1-2500
sha256sum src/v1/stage0/src/v1_compiler_emit_rust.rs target/stage0-regen-candidate/src/v1_compiler_emit_rust.rs
cmp src/v1/stage0/src/v1_compiler_emit_rust.rs target/stage0-regen-candidate/src/v1_compiler_emit_rust.rs && echo "PER-FILE FIXED POINT: IDENTICAL"
