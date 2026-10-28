set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
# main baseline (control known CLEAN) + only the r1 regen, FULL log
cp -f .valiant-main-mirrors/*.rs src/v1/stage0/src/
for f in std_algebra.rs std_types.rs std_computation.rs; do cp -f .valiant-main-mirrors/$f src/v1/stage0/src/$f; done
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b0.log 2>&1 || { echo BUILDFAIL; grep -E "^error" -A6 /tmp/b0.log|head -30; exit 1; }
rm -rf target/rcX
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rcX --regen-receipt target/rX.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/rcX.log 2>&1
echo "regen_exit=$?"
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/rcX.log > /tmp/rcX.clean
echo "total_lines=$(wc -l < /tmp/rcX.clean)"
echo "=== verdict block (everything after first required-regen line):"
grep -n "required-regen\|hard diagnostic" /tmp/rcX.clean | head
echo "=== ALL distinct diagnostic shapes:"
grep -E "^\s{2,}- |^\s{4}" /tmp/rcX.clean | sed -E 's/\([a-z_0-9/.]+\.[a-z]+:[0-9]+:[0-9]+\)/(LOC)/; s/'"'"'[^'"'"']*'"'"'/'"'"'X'"'"'/g; s/[0-9]+/N/g' | sort | uniq -c | sort -rn | head -25
echo "=== sample 30 raw refusal lines:"
awk '/hard diagnostic/{f=1;next} f' /tmp/rcX.clean | head -30
echo "=== candidate present?"
ls target/rcX 2>/dev/null | head -3; ls target/rcX 2>/dev/null | wc -l
