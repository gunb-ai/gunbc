#!/bin/bash
# TC: run the throwaway same-text-refused field-axis probe against the SHIPPED SEED.
# NO git checkout / git clean on the runner (its HEAD is the main tip; it wiped the
# seed carriers once). lib.rs is edited in place and restored from a /tmp backup.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
cp src/v1/tests/src/lib.rs /tmp/lib.rs.bak.tc
cp tc_probe_body.rs src/v1/tests/src/tc_probe_test.rs
grep -q 'mod tc_probe_test;' src/v1/tests/src/lib.rs || echo 'mod tc_probe_test;' >> src/v1/tests/src/lib.rs
echo "TC: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
cargo test -p v1-compiler-tests --lib tc_ -- --nocapture 2>&1 | grep -E "^TC |error\[|error:|test result|panicked" | head -40
cp /tmp/lib.rs.bak.tc src/v1/tests/src/lib.rs
rm -f src/v1/tests/src/tc_probe_test.rs
echo "TC: restored lib.rs; tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo TC_DONE
