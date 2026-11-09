set -uo pipefail
cd repo-root 2>/dev/null || true
st=0
cargo check -p v1-compiler --lib --tests > /tmp/check1.log 2>&1; r=$?; echo "CHECK1=$r"; tail -3 /tmp/check1.log; st=$((st+r))
echo 'fn __must_fail_control__( { broken' >> src/v1/stage0/src/cli_run.rs
cargo check -p v1-compiler --lib --tests > /tmp/check2.log 2>&1; r=$?; echo "MUSTFAIL_CONTROL_exit=$r (expect nonzero)"
git checkout -- src/v1/stage0/src/cli_run.rs
cargo check -p v1-compiler --lib --tests > /tmp/check3.log 2>&1; r=$?; echo "CHECK3=$r"; tail -3 /tmp/check3.log; st=$((st+r))
cargo build --release -p v1-compiler --bin gunbc > /tmp/build.log 2>&1; r=$?; echo "BUILD=$r"; tail -2 /tmp/build.log; st=$((st+r))
if [ $r -eq 0 ]; then
while read f; do out=$(./target/release/gunbc run --source-root dag --source-root src/v2 --entry "$f" --function definitely_not_a_function 2>&1 | tail -1); case "$out" in *NoSuchFunction*) echo "OK $f";; *) echo "BAD $f :: $out";; esac; done < ./seedcheck-list-tmp.txt
fi
echo "TOTAL=$st"
