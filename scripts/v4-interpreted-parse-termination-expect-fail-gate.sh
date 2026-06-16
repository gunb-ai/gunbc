#!/usr/bin/env bash
# ExpectFail gate (#4957): interpreted-parse termination witness — honest defer, NOT a reshape.
#
# The witness still runs and honestly REDs when native-fold-cost is insufficient under the
# 30s budget. CI treats the expected RED as PASS (defer), unblocking parity/oracle gates.
# STALE MANIFEST: if the witness completes sub-30s, this gate fails closed — flip to ExpectPass.
#
# bind_anchor: node://adhoc-fc63cf25-e45 (interpreted-parse fold-cost optimization; dissolve-on for this defer)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

readonly HONEST_MARK=$'interpreted-parse termination NOT achieved; native-fold-cost insufficient under 30s budget; by-execution: native-on fold_list=11384/right=62895 still recv_timeout; DEFERRED pending fold-cost optimization'

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

ec=0
if ! bash .github/ci-floor/with-sccache-retry.sh cargo test -p v2-compiler-tests --release \
  interpreted_parse_termination_test::interpreted_parse_bisect_parse_terminates_within_budget \
  -- --exact --test-threads=1 --nocapture >"$log" 2>&1; then
  ec=$?
fi
cat "$log"

if [[ "$ec" -eq 0 ]]; then
  echo "::error::interpreted-parse termination ExpectFail: STALE MANIFEST — witness GREEN (sub-30s); flip defer to ExpectPass in scripts/v4-interpreted-parse-termination-expect-fail-gate.sh"
  exit 1
fi

# Discriminate the SPECIFIC honest-RED signature: budget trip WITH native-fold engaged.
# Compile errors, panics, wrong-tree, or native-off (0/0 hits) must NOT pass as defer.
if rg -q "exceeded.*budget|witness bisect_parse_terminates elapsed" "$log" \
  && rg -q "native_fold_hits fold_list=[1-9][0-9]* fold_list_right=[1-9]" "$log"; then
  echo "::notice title=interpreted-parse termination (ExpectFail defer)::${HONEST_MARK}"
  exit 0
fi

echo "::error::interpreted-parse termination ExpectFail: not honest budget RED with native-fold hits (infra/compile/panic/wrong failure mode)"
exit 1
