set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
export RUST_BACKTRACE=1
echo "=== mirrors state check:"
for f in std_algebra.rs std_types.rs std_computation.rs; do echo "$f scaffold?=$(cmp -s src/v1/stage0/src/$f .valiant-scaffold/$f && echo SCAFFOLD || echo candidate)"; done
echo "=== build claim_executor (raw errors):"
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b.log 2>&1
echo "build_exit=$?"
grep -E "^error" -A6 /tmp/b.log | head -80
echo "=== run regen:"
rm -rf target/rc2
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rc2 --regen-receipt target/r2.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/r2full.log 2>&1
echo "regen_exit=$?"
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/r2full.log > /tmp/r2clean.log
grep -vE "^\[|Uploaded|Starting|ctrl-build|Checking patch|Applied patch|git apply|exec bash|wait for run|Warning:|Waiting for|Streaming|remote:|Receiving|Resolving|From |warning:|Reset branch|Updating files|Removing " /tmp/r2clean.log | grep -vE "^\s*$" | head -60
