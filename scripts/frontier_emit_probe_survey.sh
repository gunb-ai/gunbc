#!/usr/bin/env bash
# Wave 2: execution-measured per-module emit probe survey over the 27 compiler frontier modules.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
DISCOVER="${DISCOVER:-$ROOT/target/release/discover_source_root_ingest}"
PROBE_ROOT="$ROOT/target/frontier-probe"
RESULTS="$PROBE_ROOT/survey_results.tsv"

if [[ ! -x "$GUNBC" ]]; then
  echo "frontier_emit_probe_survey: build gunbc first (cargo build -p v1-compiler --release --bin gunbc --bin discover_source_root_ingest)" >&2
  exit 1
fi

modules=(
  "src/v2/compiler/00_compile.dag"
  "src/v2/compiler/01_tokenize.dag"
  "src/v2/compiler/02_parse.dag"
  "src/v2/compiler/03_body_producer.dag"
  "src/v2/compiler/03_ingest.dag"
  "src/v2/compiler/03_name_resolve.dag"
  "src/v2/compiler/03_normalize.dag"
  "src/v2/compiler/03_resolve.dag"
  "src/v2/compiler/04_infer.dag"
  "src/v2/compiler/05_emit.dag"
  "src/v2/compiler/05_emit_orchestration.dag"
  "src/v2/compiler/05_eval.dag"
  "src/v2/compiler/06_translate.dag"
  "src/v2/compiler/07_target_carriers.dag"
  "src/v2/compiler/discovery_enumeration.dag"
  "src/v2/compiler/emit_host.dag"
  "src/v2/compiler/emit_module.dag"
  "src/v2/compiler/emit_produced.dag"
  "src/v2/compiler/emit_semantic_decl.dag"
  "src/v2/compiler/fold_lowering.dag"
  "src/v2/compiler/materialization_carriers.dag"
  "src/v2/compiler/parse_engine_hooks.dag"
  "src/v2/compiler/program_assembly.dag"
  "src/v2/compiler/program_partition.dag"
  "src/v2/compiler/self_host.dag"
  "src/v2/compiler/source_authority.dag"
  "src/v2/compiler/use_site_verdict.dag"
)

mkdir -p "$PROBE_ROOT"
echo -e "module_path\tclosure_reads\tgap4_ingest\temit_accepts\tlocated_reason" > "$RESULTS"

for mod in "${modules[@]}"; do
  slug="$(echo "$mod" | tr '/.' '_')"
  probe_dir="$PROBE_ROOT/$slug"
  mkdir -p "$probe_dir"

  "$DISCOVER" \
    --source-root src/v2 \
    --source-root dag \
    --entry "$mod" \
    --emit-dag-manifest "$probe_dir/host_source_root_ingest_manifest.dag" \
    >/dev/null 2>&1

  reads="$(grep -o 'ingest_read_count: [0-9]*' "$probe_dir/host_source_root_ingest_manifest.dag" | head -1 | awk '{print $2}')"

  gap4_out="$("$GUNBC" run \
    --source-root dag \
    --source-root src/v2 \
    --source-root "$probe_dir" \
    --entry src/v2/test/claim/self_host/compiler_closure_emit_from_ingest_test.dag \
    --function gap4_probe_all_ingest_reads_accept_holds \
    --claim-run 2>&1 || true)"
  gap4="$(echo "$gap4_out" | grep -E '^(true|false)$' | tail -1 || echo "error")"

  emit_out="$("$GUNBC" run \
    --source-root dag \
    --source-root src/v2 \
    --source-root "$probe_dir" \
    --entry src/v2/test/claim/self_host/compiler_frontier_emit_probe_test.dag \
    --function frontier_module_emit_probe_accepts_holds \
    --claim-run 2>&1 || true)"
  emit="$(echo "$emit_out" | grep -E '^(true|false)$' | tail -1 || echo "error")"
  reason="$(echo "$emit_out" | grep -o 'located_reason=[^ ]*' | tail -1 | cut -d= -f2 || echo "unknown")"

  echo -e "${mod}\t${reads}\t${gap4}\t${emit}\t${reason}"
  echo -e "${mod}\t${reads}\t${gap4}\t${emit}\t${reason}" >> "$RESULTS"
done

echo "frontier_emit_probe_survey: wrote $RESULTS"
