#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 tracked-expectation corpus eval — THIN host transport (RR-A §5.2 / A.3b).
# Invokes gunbc and projects modeled witnesses from manual_corpus_eval_expected.dag.
# Pin table + drift comparison authority lives in the .dag layer (not bash). HostRejected CDV rows:
# transport observes `corpus_eval_cdv_host_rejection_stderr_marker` from the entry .dag (interim
# pre-TestClaimRun precondition); modeled witnesses gate pin alignment — not host-owned drift logic.
# When the CDV marker is absent (substrate defect until B1 #4476), rows stay honest tracked-red:
# pin-table witness passes, job does not fail-closed.
#
# Operator re-promotion (2026-06-05): corpus-eval returns to the required CI path after
# Wave-1 §11.7.1 Class-C demotion. scripts/ is outside the §11.7.5 ci-floor ratchet.
#
# Env:
#   V2_COMPILER — gunbc binary (default: target/release/gunbc)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
entry="src/v4/test/claim/workflow/manual_corpus_eval_expected.dag"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

claim_run() {
  "$bin" run --source-root src/v4 --entry "$entry" --function "$1" --claim-run
}

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$entry" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\1/p" \
    | head -1
}

transport_run() {
  local row_entry="$1" row_fn="$2"
  set +e
  local out
  out="$("$bin" run --source-root src/v4 --entry "$row_entry" --function "$row_fn" 2>&1)"
  local ex=$?
  set -e
  printf '%s' "$out"
  return "$ex"
}

cdv_host_rejected_row() {
  local label="$1" row_entry="$2" row_fn="$3" witness="$4"
  local out
  out="$(transport_run "$row_entry" "$row_fn")" || true
  if grep -Fq "$cdv_host_rejection_marker" <<< "$out"; then
    claim_run "$witness"
  else
    echo "::notice title=corpus eval tracked-red::${label} CDV host-rejection pin held; transport precondition unmet until B1 (#${cdv_tracked_red_until_b1})"
    claim_run "$witness"
  fi
}

cdv_host_rejection_marker="$(dag_string_data corpus_eval_cdv_host_rejection_stderr_marker)"
cdv_tracked_red_until_b1="$(dag_string_data corpus_eval_cdv_tracked_red_until_b1)"
if [[ -z "$cdv_host_rejection_marker" ]]; then
  echo "error: missing corpus_eval_cdv_host_rejection_stderr_marker in $entry" >&2
  exit 2
fi
if [[ -z "$cdv_tracked_red_until_b1" ]]; then
  echo "error: missing corpus_eval_cdv_tracked_red_until_b1 in $entry" >&2
  exit 2
fi

# Modeled pin-table frontier + alignment witness.
claim_run witness_corpus_eval_tracked_expectation_closed

# Executed rows: runtime drift gate projects from TestClaimRun via modeled witnesses.
claim_run witness_corpus_eval_row_eval_mvp2_runtime_gate
claim_run witness_corpus_eval_row_mechanical_reverification_runtime_gate
claim_run witness_corpus_eval_row_subsumption_reverifies_runtime_gate

# HostRejected CDV rows: observe transport when precondition holds; otherwise honest tracked-red.
cdv_host_rejected_row \
  "parallelism" \
  "src/v4/test/claim/lens_parallelism/data_dependency.dag" \
  "run_parallelism_data_dependency_receipt" \
  "witness_corpus_eval_row_parallelism_host_rejected_gate"

cdv_host_rejected_row \
  "effect" \
  "src/v4/test/claim/lens_effect/effect_depends_on.dag" \
  "run_lens_effect_depends_on_runtime_verdict" \
  "witness_corpus_eval_row_effect_host_rejected_gate"

echo "::notice title=corpus eval tracked-expectation gate::all pinned rows matched (0 drift)"
bash scripts/v4-testclaim-roster-pilot.sh
bash scripts/v4-testclaim-grounding-typescript-pilot.sh
exit 0
