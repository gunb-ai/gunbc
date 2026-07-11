#!/usr/bin/env bash
# Fast 2-call classification: name_resolution probe, else EmitSurfaceGap (0/27 self-emitted baseline).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
DISCOVER="${DISCOVER:-$ROOT/target/release/discover_source_root_ingest}"
PROBE_ROOT="$ROOT/target/frontier-probe"
RESULTS="$PROBE_ROOT/survey_results.tsv"
ENTRY="src/v2/test/claim/self_host/compiler_frontier_emit_probe_test.dag"

modules=(
  "src/v2/compiler/00_compile.dag" "src/v2/compiler/01_tokenize.dag" "src/v2/compiler/02_parse.dag"
  "src/v2/compiler/03_body_producer.dag" "src/v2/compiler/03_ingest.dag" "src/v2/compiler/03_name_resolve.dag"
  "src/v2/compiler/03_normalize.dag" "src/v2/compiler/03_resolve.dag" "src/v2/compiler/04_infer.dag"
  "src/v2/compiler/05_emit.dag" "src/v2/compiler/05_emit_orchestration.dag" "src/v2/compiler/05_eval.dag"
  "src/v2/compiler/06_translate.dag" "src/v2/compiler/07_target_carriers.dag" "src/v2/compiler/discovery_enumeration.dag"
  "src/v2/compiler/emit_host.dag" "src/v2/compiler/emit_module.dag" "src/v2/compiler/emit_produced.dag"
  "src/v2/compiler/emit_semantic_decl.dag" "src/v2/compiler/fold_lowering.dag" "src/v2/compiler/materialization_carriers.dag"
  "src/v2/compiler/parse_engine_hooks.dag" "src/v2/compiler/program_assembly.dag" "src/v2/compiler/program_partition.dag"
  "src/v2/compiler/self_host.dag" "src/v2/compiler/source_authority.dag" "src/v2/compiler/use_site_verdict.dag"
)

run_probe() {
  "$GUNBC" run --source-root dag --source-root src/v2 --source-root "$1" \
    --entry "$ENTRY" --function "$2" --claim-run 2>/dev/null | grep -E '^(true|false)$' | tail -1
}

mkdir -p "$PROBE_ROOT"
echo -e "module_path\tclosure_reads\tblocker_class" > "$RESULTS"

for mod in "${modules[@]}"; do
  slug="$(echo "$mod" | tr '/.' '_')"
  probe_dir="$PROBE_ROOT/$slug"
  mkdir -p "$probe_dir"
  [[ -f "$probe_dir/host_source_root_ingest_manifest.dag" ]] || \
    "$DISCOVER" --source-root src/v2 --source-root dag --entry "$mod" \
      --emit-dag-manifest "$probe_dir/host_source_root_ingest_manifest.dag" >/dev/null 2>&1
  reads="$(grep -o 'ingest_read_count: [0-9]*' "$probe_dir/host_source_root_ingest_manifest.dag" | awk '{print $2}')"
  if [[ "$(run_probe "$probe_dir" frontier_module_emit_probe_accepts_holds)" == "true" ]]; then
    blocker="SelfEmitReady"
  elif [[ "$(run_probe "$probe_dir" frontier_module_emit_probe_name_resolution_holds)" == "true" ]]; then
    blocker="NameResolutionGap"
  elif [[ "$(run_probe "$probe_dir" frontier_module_emit_probe_realization_holds)" == "true" ]]; then
    blocker="RealizationGap"
  else
    blocker="EmitSurfaceGap"
  fi
  echo -e "${mod}\t${reads}\t${blocker}" | tee -a "$RESULTS"
done
echo "done: $RESULTS"
