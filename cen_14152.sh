#!/bin/bash
# Census over the locally reproduced corpus + arm B's compile errors (the RED the fold
# prevents). Runs the COMMITTED instrument on the committed-artifact form.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "===== CENSUS over locally reproduced arm A (fold seed) ====="
bash tools/rebind_census.sh /tmp/shA.log 2>&1 | head -22
echo "===== CENSUS over CI artifact (run 38918165053) for comparison ====="
bash tools/rebind_census.sh /tmp/ci_v2boot.log 2>&1 | sed -n '3,18p'
echo "===== identical row sets? ====="
grep -o "expected 'type variable [^']*' ([a-z_0-9/.]*:[0-9-]*)" /tmp/shA.log | sort | uniq -c | sort -rn > /tmp/a_rows.txt
echo "local distinct rows: $(wc -l < /tmp/a_rows.txt)  total: $(awk '{s+=$1} END{print s}' /tmp/a_rows.txt)"
echo "===== ARM B (pre-fold seed): the 4 compile errors the fold prevents ====="
grep -n "^error" /tmp/shB.log | head -10 | cut -c1-260
echo "--- error detail blocks:"
grep -A6 "^error\[" /tmp/shB.log | head -50 | cut -c1-200
echo CEN_DONE
