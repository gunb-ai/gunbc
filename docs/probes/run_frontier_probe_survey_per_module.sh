#!/usr/bin/env bash
# SCAFFOLD — probe one-off beside docs/probes/curated_cargo_probe_one.sh (not a general scripts/ home).
# dissolve-on: ^migrate_when_frontier_per_module_probe_receipt_binds — floor-enrolled
# v2.workflow.frontier_probe_survey_transport runs the survey without this shell loop; delete
# this script when the transport + single-process survey path is green on CI hosts without OOM.
# Runtime-present: invokes frontier_probe_survey seed bin per module (not bash-emitted workflow).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "frontier_probe_survey: SurveyPinRefused: dirty working tree — run from a clean detached worktree at the selected commit" >&2
  exit 1
fi

SOURCE_COMMIT="$(git rev-parse HEAD)"
echo "frontier_probe_survey: building release bin at source_commit=${SOURCE_COMMIT}" >&2
/opt/cargo/bin/cargo build --release -p v1-compiler --bin frontier_probe_survey

BIN="$ROOT/target/release/frontier_probe_survey"
if [[ ! -x "$BIN" ]]; then
  echo "frontier_probe_survey: release bin missing after build: $BIN" >&2
  exit 1
fi

SURVEY_DIR="$ROOT/target/frontier-probe-survey"
TSV="$SURVEY_DIR/frontier_probe_survey.tsv"
MANIFEST="$SURVEY_DIR/host_frontier_probe_survey_manifest.dag"
mkdir -p "$SURVEY_DIR"

MODULES=(
  "src/v2/compiler/03_normalize.dag"
  "src/v2/compiler/03_resolve.dag"
  "src/v2/compiler/03_name_resolve.dag"
  "src/v2/compiler/emit_module.dag"
  "src/v2/compiler/05_emit_orchestration.dag"
  "src/v2/compiler/emit_semantic_decl.dag"
  "src/v2/compiler/emit_host.dag"
  "src/v2/compiler/emit_produced.dag"
  "src/v2/compiler/03_body_producer.dag"
  "src/v2/compiler/program_assembly.dag"
  "src/v2/compiler/source_authority.dag"
  "src/v2/compiler/00_compile.dag"
  "src/v2/compiler/03_ingest.dag"
  "src/v2/compiler/parse_engine_hooks.dag"
  "src/v2/compiler/use_site_verdict.dag"
  "src/v2/compiler/discovery_enumeration.dag"
  "src/v2/compiler/01_tokenize.dag"
  "src/v2/compiler/self_host.dag"
  "src/v2/compiler/materialization_carriers.dag"
  "src/v2/compiler/07_target_carriers.dag"
  "src/v2/compiler/fold_lowering.dag"
  "src/v2/compiler/04_infer.dag"
  "src/v2/compiler/05_eval.dag"
  "src/v2/compiler/06_translate.dag"
  "src/v2/compiler/05_emit.dag"
  "src/v2/compiler/program_partition.dag"
  "src/v2/compiler/02_parse.dag"
)

echo "module	source_commit	source_tree	survey_executable	source_roots_digest	probe_policy_revision	self_emit_ready	blocker_class	located_stage	located_reason	rejection_chain	overlap_roster_detail	probe_error" > "$TSV"
failures=0
for mod in "${MODULES[@]}"; do
  echo "=== probing $mod ===" >&2
  rm -f "$SURVEY_DIR/tmp_row.tsv"
  if ! "$BIN" \
    --source-root src/v2 \
    --source-root dag \
    --module "$mod" \
    --survey-dir "$SURVEY_DIR" \
    --emit-survey-manifest "$SURVEY_DIR/tmp_manifest.dag" \
    --emit-tsv "$SURVEY_DIR/tmp_row.tsv" >"$SURVEY_DIR/tmp_probe.log" 2>&1
  then
    echo "frontier_probe_survey refused for $mod (exit $?)" >&2
    tail -20 "$SURVEY_DIR/tmp_probe.log" >&2
    rm -f "$SURVEY_DIR/tmp_row.tsv"
    failures=$((failures + 1))
    continue
  fi
  if [[ ! -f "$SURVEY_DIR/tmp_row.tsv" ]]; then
    echo "frontier_probe_survey produced no row file for $mod" >&2
    failures=$((failures + 1))
    continue
  fi
  tail -n +2 "$SURVEY_DIR/tmp_row.tsv" >> "$TSV"
done

if [[ failures -ne 0 ]]; then
  echo "survey refused: $failures module probe(s) failed — TSV not promoted to manifest" >&2
  exit 1
fi

"$BIN" \
  --source-root src/v2 \
  --source-root dag \
  --emit-survey-manifest "$MANIFEST" \
  --emit-manifest-only \
  --receipt-tsv "$TSV"

echo "Survey complete: $MANIFEST (source_commit=${SOURCE_COMMIT})"
