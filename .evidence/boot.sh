set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
echo "STATE:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' '|')]"
# (b) BOTH compilations the gate builds: the seed crate, and the v2-bootstrap emitted closure.
set +e
cargo check --release -p v1-compiler --all-targets > /tmp/seedcheck.log 2>&1
echo "SEED-CHECK-EXIT=$?"
set -e
grep -aE "^error" /tmp/seedcheck.log | head -20 || true
set +e
cargo build --release -p v1-compiler --bin bootstrap_witness > /tmp/bootb.log 2>&1
echo "BOOT-BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/bootb.log | head -10 || true
set +e
./target/release/bootstrap_witness --suite expensive > /tmp/boot.log 2>&1
echo "BOOTSTRAP-EXIT=$?"
set -e
grep -aE "PASS|FAIL|error|stage1 cargo check|errors \(ratchet|closure" /tmp/boot.log | tail -30
echo "ZZ-DONE"
