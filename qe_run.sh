cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
M=src/v1/stage0/src/v1_compiler_infer.rs
for which in new base; do
  if [ $which = base ]; then git show origin/main:$M > $M; fi
  /opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -1
  target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/qe_scratch.dag --function qe_main > /tmp/qe_run_$which.out 2>&1; echo "rc_$which=$?"
  tail -3 /tmp/qe_run_$which.out | cut -c1-300
done
git checkout $M
