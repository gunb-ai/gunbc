#!/bin/bash
set -uo pipefail
echo "=== HEAD ==="; git rev-parse HEAD | cut -c1-11; git branch --show-current
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/build9.log 2>&1
echo "build rc=$?"
grep -aE '^error|^warning: unused|--> ' /tmp/build9.log | head -40 | cut -c1-200
echo "=== first error blocks ==="
grep -an 'error' /tmp/build9.log | head -8
sed -n "$(grep -an '^error' /tmp/build9.log | head -1 | cut -d: -f1),+6p" /tmp/build9.log | cut -c1-200
echo "=== DONE ==="
