#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this orchestration shell; until then it
# sequences curated_cargo_probe_one.sh over the frontier roster (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe sweep transport in .dag.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
REPORT="${ROOT}/docs/probes/curated_cargo_frontier_probe_sweep.tsv"
CLASSIFIER_STAMP="rule1-first-error-plus-residual-histogram-v3"
FORCE_REPROBE="${CSSL_FORCE_REPROBE:-0}"
mkdir -p "$(dirname "$REPORT")"
PROBE="$ROOT/scripts/curated_cargo_probe_one.sh"

if [[ "$FORCE_REPROBE" == "1" ]] || [[ ! -f "$REPORT" ]] || ! grep -qF "$CLASSIFIER_STAMP" "$REPORT" 2>/dev/null; then
  {
    echo "# classifier_stamp: $CLASSIFIER_STAMP"
    echo -e "module\temit\tcargo\tfirst_error\tmapped_gate\tverdict\tresidual_histogram"
  } >"$REPORT"
fi

probe() {
  local path="$1"
  local shim="${2:-}"
  echo "==> probing $path" >&2
  if [[ -n "$shim" ]]; then
    "$PROBE" "$path" "$shim"
  else
    "$PROBE" "$path"
  fi
}

run_tier() {
  local path="$1"
  local shim="${2:-}"
  local row
  if [[ "$FORCE_REPROBE" != "1" ]] && grep -qF "$path" "$REPORT"; then
    echo "skip (already probed): $path" >&2
    return 0
  fi
  row="$(probe "$path" "$shim")"
  printf '%s\n' "$row" >>"$REPORT"
}

# TIER 1 — highest information value
run_tier src/v2/compiler/02_parse.dag
run_tier src/v2/compiler/03_ingest.dag
run_tier src/v2/compiler/06_translate.dag
run_tier src/v2/compiler/05_eval.dag
run_tier src/v2/compiler/materialization_carriers.dag
run_tier src/v2/compiler/05_emit.dag
run_tier src/v2/compiler/05_emit_orchestration.dag
run_tier src/v2/compiler/emit_module.dag
run_tier src/v2/compiler/emit_produced.dag
run_tier src/v2/compiler/emit_semantic_decl.dag
run_tier src/v2/compiler/emit_host.dag
run_tier src/v2/compiler/fold_lowering.dag
run_tier src/v2/compiler/03_name_resolve.dag
run_tier src/v2/compiler/03_resolve.dag

# TIER 2 — expected Gate A / namespace confirmations
run_tier src/v2/compiler/00_compile.dag
run_tier src/v2/compiler/01_tokenize.dag
run_tier src/v2/compiler/03_normalize.dag
run_tier src/v2/compiler/04_infer.dag
run_tier src/v2/compiler/program_assembly.dag
run_tier src/v2/compiler/program_partition.dag
run_tier src/v2/compiler/source_authority.dag

echo "sweep complete: $REPORT" >&2
