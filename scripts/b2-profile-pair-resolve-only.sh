#!/usr/bin/env bash
# B2 acceptance: resolve-only timing for budget_roster vs fold_list twin.
# SCAFFOLD — dissolve-on: substrate-emitted resolve timings.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR
BIN="${CLAIM_BATCH:-$ROOT/target/release/claim_batch}"
if [[ ! -x "$BIN" ]]; then
  cargo build -p v1-compiler --release --bin claim_batch
fi
run_one() {
  local label="$1" entry="$2"
  local out rc
  set +e
  out="$("$BIN" --source-root src/v2 --source-root dag --entry "$entry" --function _profile_resolve_only_probe_ 2>&1)"
  rc=$?
  set -e
  echo "[pair:${label}] exit=${rc}"
  printf '%s\n' "$out" | rg -F "[resolve] ${entry}:" | tail -1 | sed "s/^/[pair:${label}] /" || true
}
echo "b2-profile-pair-resolve-only: cold resolve (no witness eval)"
run_one budget_roster src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag
run_one fold_list src/v2/test/claim/fold_list_generic_instantiation.dag
