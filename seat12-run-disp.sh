set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
./target/release/gunbc test //gunbc/instruments:ambient-operation-disposition-census > all.out 2>&1; echo exit=$?
echo ---SUMMARY; grep -a "^disposition_summary" all.out
echo ---BASIS; grep -a "^disposition	" all.out | grep -ac "manager direction"; grep -ac "operator-ruling" all.out
echo ---TAIL; tail -c 400 all.out
