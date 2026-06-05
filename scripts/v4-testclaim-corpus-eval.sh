#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 tracked-expectation corpus eval — THIN host transport (RR-A §5.2 / A.3b).
# Invokes gunbc and projects modeled witnesses from manual_corpus_eval_expected.dag.
# Pin table + drift comparison authority lives in the .dag layer (not bash).
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

# Modeled pin-table frontier + alignment witness.
claim_run witness_corpus_eval_tracked_expectation_closed

# Executed rows: runtime drift gate projects from TestClaimRun via modeled witnesses.
claim_run witness_corpus_eval_row_eval_mvp2_runtime_gate
claim_run witness_corpus_eval_row_mechanical_reverification_runtime_gate
claim_run witness_corpus_eval_row_subsumption_reverifies_runtime_gate

# HostRejected rows: transport boundary observes interpreter failure; gate is modeled.
out="$(transport_run \
  "src/v4/test/claim/lens_parallelism/data_dependency.dag" \
  "run_parallelism_data_dependency_receipt")" || true
if ! grep -q "non-exhaustive pattern match on: ClassifiedDependencyView" <<< "$out"; then
  echo "::error title=corpus eval transport::parallelism row expected CDV host rejection" >&2
  exit 1
fi
claim_run witness_corpus_eval_row_parallelism_host_rejected_gate

out="$(transport_run \
  "src/v4/test/claim/lens_effect/effect_depends_on.dag" \
  "run_lens_effect_depends_on_runtime_verdict")" || true
if ! grep -q "non-exhaustive pattern match on: ClassifiedDependencyView" <<< "$out"; then
  echo "::error title=corpus eval transport::effect row expected CDV host rejection" >&2
  exit 1
fi
claim_run witness_corpus_eval_row_effect_host_rejected_gate

echo "::notice title=corpus eval tracked-expectation gate::all pinned rows matched (0 drift)"
exit 0
