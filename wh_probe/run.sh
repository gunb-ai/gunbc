set -x
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
G=./target/release/gunbc
$G run --source-root dag --source-root wh_probe --entry wh_probe/emit_probe.dag 2>&1 | tail -5
rm -rf /tmp/pout; $G compile --source-root dag --source-root wh_probe --entry wh_probe/emit_probe.dag --output-dir /tmp/pout --target rust 2>&1 | tail -10
ls -R /tmp/pout | head -20
(cd /tmp/pout/* 2>/dev/null || cd /tmp/pout; cargo build --release 2>&1 | grep -E "^error|-->|warning: unused" -A3 | head -60; ls target/release | head)
find /tmp/pout -maxdepth 3 -name '*.rs' | head
grep -n "bytes_octets\|fn ts_before" -A6 $(find /tmp/pout -name '*.rs') | head -40
for b in $(find /tmp/pout -path '*target/release/*' -maxdepth 4 -type f -executable); do echo RUN $b; $b; done
