set -e
cargo build --release -p gunbc 2>&1 | tail -2
./target/release/gunbc run gunbc.guarantee_stall.roster --function __no_such_function__ 2>&1 | tail -5 || true
