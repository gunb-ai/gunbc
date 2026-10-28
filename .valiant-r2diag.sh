set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -c "^error" | xargs echo "build_errors="
rm -rf target/rc2
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rc2 --regen-receipt target/r2.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/r2full.log 2>&1
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/r2full.log > /tmp/r2clean.log
echo "regen_exit=$?"
grep -c "effect summary incomplete" /tmp/r2clean.log | xargs echo "effect_lines="
echo "=== distinct diagnostic shapes (normalized):"
grep -oE "(effect summary incomplete|unresolved [a-z]+|no registry row|unknown|does not exist)[^:]*: [^;]*" /tmp/r2clean.log | sed -E "s/'[^']*'/'X'/g; s/(v1|std)\.[a-z_.]+/MODULE/g" | sort | uniq -c | sort -rn | head -20
echo "=== 'names no registry row' callees:"
grep -oE "MODULE calls MODULE, whose identity" /tmp/r2clean.log | sort | uniq -c | sort -rn | head -20
echo "=== first 25 raw diagnostic lines:"
grep -E "effect summary incomplete|unresolved|no registry row" /tmp/r2clean.log | head -25
