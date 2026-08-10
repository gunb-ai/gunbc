#!/usr/bin/env bash
# Run one arm of the P3 ClassificationCostStanding A/B (#8055).
#
# Same-subject paired comparison:
#   1. Build on main:    ctrl-build -- cargo build -p v1-compiler --release --bin p3_cohort_probe
#      GUNBC_P3_ARM=control GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee control.log
#   2. Build on candidate (pr-8055 / merge SHA):
#      GUNBC_P3_ARM=candidate GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee candidate.log
#
# Compare RECEIPT lines: classification_production_ms + prep_exec_ms (candidate) vs prep_exec_ms
# at higher selected_entry_groups (control). Assert head_sha on each receipt matches the pinned tag.
#
# Subjects: incident (default, two-entry discriminating roster) | discovery (full scan).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SUBJECT="${GUNBC_P3_SUBJECT:-${1:-incident}}"
ARM="${GUNBC_P3_ARM:-candidate}"
BIN="${P3_COHORT_PROBE_BIN:-$ROOT/target/release/p3_cohort_probe}"

export GUNBC_P3_SUBJECT="$SUBJECT"
export GUNBC_P3_ARM="$ARM"

cd "$ROOT"
exec "$BIN"
