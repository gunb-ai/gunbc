#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: substrate-emitted resolve-phase timings (realization_measurement_loop
# carrier / GUNBC_RESOLVE_PROFILE=1) retire this bash runner; until then it is the reproducible
# receipt entrypoint documented in docs/plans/resolver-pathology-profile-receipt.md.
# dissolve-on: gunbc bash-emit capability (#5828 / ROADMAP shell-emission) realizes profile
# orchestration through host_effect_apply transport handlers instead of hand-rolled bash.
#
# Reproducible cold-resolve profiling for the CI floor witness corpus.
# Profile receipt: docs/plans/resolver-pathology-profile-receipt.md
#
# Usage:
#   ./scripts/profile-cold-resolve.sh              # per-entry resolve log (warm typed_module_cache)
#   ./scripts/profile-cold-resolve.sh --top 25     # print top-N offenders after run
#   ./scripts/profile-cold-resolve.sh --pair       # budget_roster vs structural twin only
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Cross-process resolve cache OFF: unset only (do not export empty string — var_os treats "" as set).
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR

BIN="${CLAIM_BATCH:-$ROOT/target/release/claim_batch}"
if [[ ! -x "$BIN" ]]; then
  echo "profile-cold-resolve: building claim_batch (release)…" >&2
  CTRL_BUILD_BYPASS_SHIMS=1 cargo build -p v1-compiler --release --bin claim_batch
fi

TOP_N=15
MODE="corpus"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --top) TOP_N="${2:-15}"; shift 2 ;;
    --pair) MODE="pair"; shift ;;
    -h|--help)
      sed -n '2,9p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

log="$(mktemp "${TMPDIR:-/tmp}/profile-cold-resolve.XXXXXX.log")"
trap 'rm -f "$log"' EXIT

if [[ "$MODE" == "pair" ]]; then
  echo "profile-cold-resolve: pathological pair (cold per-process resolve)" >&2
  for spec in \
    "budget_roster:src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag" \
    "structural_twin_fold_list:src/v2/test/claim/fold_list_generic_instantiation.dag" \
    "roster_module:src/v2/test/claim/complexity_gate/subject_complexity_budget_roster.dag" \
    "twin_single_row_eval:src/v2/test/claim/complexity_gate/source_bridged_add_budget_test.dag:source_bridged_add_budget_claim_holds"; do
    label="${spec%%:*}"
    rest="${spec#*:}"
    entry="${rest%%:*}"
    fn="${rest#*:}"
    if [[ "$entry" == "$fn" ]]; then
      "$BIN" --source-root src/v2 --source-root dsl --entry "$entry" --function __noop__ >>"$log" 2>&1 || true
    else
      "$BIN" --source-root src/v2 --source-root dsl --entry "$entry" --function "$fn" --claim-run --wet >>"$log" 2>&1 || true
    fi
    rg "\[resolve\] $entry:" "$log" | tail -1 | sed "s/^/[pair:$label] /"
    rg "\[resolve-summary\]" "$log" | tail -1 | sed "s/^/[pair:$label] /" || true
  done
  exit 0
fi

echo "profile-cold-resolve: discovery roster — per-entry resolve (typed_module_cache warms across entries)" >&2
echo "profile-cold-resolve: log → $log" >&2

"$BIN" \
  --source-root src/v2 --source-root dsl \
  --scan-dir src/v2/test/claim \
  --scan-dir dsl/test/claim \
  --roster-from-discovery \
  --claim-run --wet \
  2>&1 | tee "$log"

echo >&2
echo "profile-cold-resolve: resolve serial-sum (unique entries)" >&2
rg '\[resolve\]' "$log" | sed -E 's/.*\[resolve\] ([^:]+): ([0-9]+)ms.*\(([0-9]+) modules, ([0-9]+) resolved items.*/\2\t\4\t\3\t\1/' \
  | awk '{sum+=$1; n++} END {printf "  entries=%d  serial_sum_ms=%d  avg_ms=%.1f\n", n, sum, (n?sum/n:0)}'

echo "profile-cold-resolve: top-$TOP_N entry resolve offenders (ms, items, modules, path)" >&2
rg '\[resolve\]' "$log" | sed -E 's/.*\[resolve\] ([^:]+): ([0-9]+)ms.*\(([0-9]+) modules, ([0-9]+) resolved items.*/\2\t\4\t\3\t\1/' \
  | sort -rn | head -"$TOP_N"
