#!/usr/bin/env bash
# Scaffold wrapper — authority: dag/tools/hand_import_gate.dag
# Gate-internal dissolution triggers: hand_import_allowed_prefixes_mirror_dissolution_trigger.
# Shell-carrier dissolution: hand_import_gate_shell_emit_dissolution_trigger /
# hand_import_gate_shell_runner_scaffold (orchestration-to-shell bash-emission capability).
# CI enrolls HandImportGate on GithubActionsCiJob (batch-0 RunnableSingleClaim via floor_effect_gate_witness).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
if [[ ! -x "${ROOT}/target/release/claim_batch" ]]; then
  echo "hand-import gate: claim_batch not built (run cargo build --release --bin claim_batch)" >&2
  exit 1
fi
"$ROOT/target/release/claim_batch" \
  --source-root "$ROOT/dag" \
  --source-root "$ROOT/src/v2" \
  --entry dag/test/claim/hand_import_gate_witness_test.dag \
  --function hand_import_gate_passes_on_branch \
  --claim-run
