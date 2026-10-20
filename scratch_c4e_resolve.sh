set -e
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/guarantee_stall/roster.dag --function __no_such_function__ 2>&1 | tail -5 || true
