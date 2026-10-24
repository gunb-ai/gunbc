set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
./target/release/gunbc test //gunbc/instruments:ambient-operation-disposition-census > dc.out 2>&1; echo dc_exit=$?
grep -a "refused_rewritten" dc.out | cut -c1-600 | head -5
echo ---DISP; grep -a "^disposition" dc.out | cut -c1-520
echo ---TAIL; tail -n 3 dc.out | cut -c1-300; tail -n 5 dc.err | cut -c1-300
