#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell M=11 probe loop; until then it
# projects the curated_cargo_probe_one.sh spine across partition §11.14 modules and banks
# per-site B1 E0369 classification TSV (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: dag/tools/self_host_curated_probe_cargo.dag via docs/probes/curated_cargo_probe_one.sh
# (docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh — no parallel Cargo.toml heredoc);
# per-site rules via docs/probes/e0369_b1_operator_classify.py; partition bucket definition
# docs/plans/self-host-cargo-refusal-root-partition.md §18 / §18.4.
# Frozen output receipt (not authority): docs/probes/e0369_b1_operator_classification_2026-08-17.md.
#
# M=11 E0369 B1 operator-on-carrier classification receipt.
# Route: curated_cargo_probe_one.sh (§11.1 instrument) → cargo.log parse → classify.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

STAMP_DIR="$SCRIPT_DIR/e0369_b1_classification_$(date -u +%Y-%m-%d)"
LOG_DIR="$STAMP_DIR/logs"
mkdir -p "$LOG_DIR"

export CSSL_STD_SEED_LINK=1
export GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
export PROBE_KEEP_LOG_DIR="$LOG_DIR"

MODULES=(
  "05_emit:src/v2/compiler/05_emit.dag"
  "06_translate:src/v2/compiler/06_translate.dag"
  "04_infer:src/v2/compiler/04_infer.dag"
  "03_ingest:src/v2/compiler/03_ingest.dag"
  "emit_host:src/v2/compiler/emit_host.dag"
  "01_tokenize:src/v2/compiler/01_tokenize.dag"
  "materialization_carriers:src/v2/compiler/materialization_carriers.dag"
  "emit_module:src/v2/compiler/emit_module.dag"
  "03_normalize:src/v2/compiler/03_normalize.dag"
  "program_partition:src/v2/compiler/program_partition.dag"
  "05_eval:src/v2/compiler/05_eval.dag"
)

echo "stamp_dir=$STAMP_DIR"
echo "git_sha=$(git rev-parse HEAD)"

fail=0
for spec in "${MODULES[@]}"; do
  name="${spec%%:*}"
  path="${spec#*:}"
  echo "=== probe $name ==="
  if ! "$SCRIPT_DIR/curated_cargo_probe_one.sh" "$path" >/dev/null; then
    fail=1
    echo "WARN: probe failed for $name" >&2
  fi
done

python3 "$SCRIPT_DIR/e0369_b1_operator_classify.py" \
  --log-dir "$LOG_DIR" \
  --out-tsv "$STAMP_DIR/sites_classified.tsv" \
  --summary-md "$STAMP_DIR/summary.md" \
  --git-sha "$(git rev-parse HEAD)"

echo "done: $STAMP_DIR (probe_fail=$fail)"
exit "$fail"
