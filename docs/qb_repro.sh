set -x
export CARGO_TARGET_DIR=$PWD/target/qb
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
OUT=$PWD/target/qb-out; rm -rf $OUT; mkdir -p $OUT
./target/qb/release/gunbc compile --source-root dag --entry dag/extdeps/units/iso8601_calendar.dag --output-dir $OUT --target rust 2>&1 | tail -20
grep -rn "unsupported cast" $OUT | head
grep -rn "fn iso8601_seconds_per_hour" -A4 $OUT | head -20
grep -rn "fn iso8601_seconds_per_minute" -A3 $OUT | head
