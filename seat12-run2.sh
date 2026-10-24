set -o pipefail
cargo build --release -p v1-compiler --bin gunbc && /usr/bin/time -v ./target/release/gunbc test //gunbc/instruments:ambient-operation-disposition-census > disp.out 2> disp.time
echo exit=$?
wc -c disp.out; grep -E "Maximum resident|Elapsed" disp.time; tail -5 disp.time | cut -c1-300
head -c 3000 disp.out; grep -E "^disposition_summary" disp.out
