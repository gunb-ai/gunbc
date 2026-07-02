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

# Run one pair entry. resolve_only=1: witness may fail after resolve; still require [resolve] line.
# Otherwise require claim_batch exit 0 (fail-closed for eval witnesses).
run_pair_entry() {
  local label="$1" entry="$2" fn="$3" resolve_only="${4:-0}"
  local -a cmd=( "$BIN" --source-root src/v2 --source-root dsl --entry "$entry" --function "$fn" )
  if [[ "$resolve_only" == "0" ]]; then
    cmd+=( --claim-run --wet )
  fi
  local out rc
  set +e
  out="$("${cmd[@]}" 2>&1)"
  rc=$?
  set -e
  printf '%s\n' "$out" >>"$log"
  if ! printf '%s\n' "$out" | rg -qF "[resolve] ${entry}:"; then
    echo "profile-cold-resolve: FAIL [pair:${label}] no [resolve] line (exit ${rc})" >&2
    printf '%s\n' "$out" >&2
    exit 1
  fi
  if [[ "$resolve_only" == "0" ]] && [[ "$rc" -ne 0 ]]; then
    echo "profile-cold-resolve: FAIL [pair:${label}] witness run failed (exit ${rc})" >&2
    printf '%s\n' "$out" >&2
    exit 1
  fi
  printf '%s\n' "$out" | rg -F "[resolve] ${entry}:" | tail -1 | sed "s/^/[pair:${label}] /"
  printf '%s\n' "$out" | rg '\[resolve-summary\]' | tail -1 | sed "s/^/[pair:${label}] /" || true
}

if [[ "$MODE" == "pair" ]]; then
  echo "profile-cold-resolve: pathological pair (cold per-process resolve)" >&2
  # resolve_only entries: no test fn in module; claim_batch resolves then fails witness lookup.
  run_pair_entry budget_roster \
    src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag \
    complexity_budget_roster_family_gate_holds 0
  run_pair_entry structural_twin_fold_list \
    src/v2/test/claim/fold_list_generic_instantiation.dag \
    fold_list_generic_instantiation_holds 0
  run_pair_entry roster_module \
    src/v2/test/claim/complexity_gate/subject_complexity_budget_roster.dag \
    _profile_resolve_only_probe_ 1
  run_pair_entry twin_single_row_eval \
    src/v2/test/claim/complexity_gate/source_bridged_add_budget_test.dag \
    source_bridged_add_budget_claim_holds 0
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
