#!/usr/bin/env bash
# .github/ci-floor/v4-testclaim-corpus-eval.sh
#
# T-38 tracked-expectation corpus eval host transport (RR-A §5.2 / A.3b).
# Executes each manual roster row, compares (verdict arm, primary reason) against the
# modeled pin table in manual_corpus_eval_expected.dag — FAIL only on drift.
# All-red-but-pinned == PASS.
#
# Authority: v4.workflow.ci TestClaimCorpusEvalCommand + manual_corpus_eval_expected_pins.
# Pin table MUST stay aligned with src/v4/test/claim/workflow/manual_corpus_eval_expected.dag.
#
# Env:
#   V2_COMPILER — gunbc binary (default: target/release/gunbc)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

# claim_id|observation|primary_reason  (observation: ExecutedFail | HostRejected)
PIN_TABLE=$'claim_eval_mvp2_test_claim_route|ExecutedFail|eval_rejected_invalid_node\nclaim_parallelism_data_dependency_run_test_claim_receipt|HostRejected|interpreter_classified_dependency_view_non_exhaustive\nclaim_lens_effect_depends_on_runtime_verdict|HostRejected|interpreter_classified_dependency_view_non_exhaustive\nclaim_rust_language_model_emit_mechanical_reverification|ExecutedFail|eval_rejected_invalid_node\nclaim_rust_language_model_emit_subsumption_reverifies|ExecutedFail|eval_rejected_type_node'

# claim_id|entry_relpath|run_fn — mirrors manual_corpus_eval_host_run_specs
RUN_TABLE=$'claim_eval_mvp2_test_claim_route|src/v4/test/claim/manual/eval_runtime_mvp.dag|run_eval_mvp2_test_claim_route\nclaim_parallelism_data_dependency_run_test_claim_receipt|src/v4/test/claim/lens_parallelism/data_dependency.dag|run_parallelism_data_dependency_receipt\nclaim_lens_effect_depends_on_runtime_verdict|src/v4/test/claim/lens_effect/effect_depends_on.dag|run_lens_effect_depends_on_runtime_verdict\nclaim_rust_language_model_emit_mechanical_reverification|src/v4/test/claim/manual/dissolution_subsumption_reverification.dag|run_rust_language_model_emit_mechanical_reverification_claim\nclaim_rust_language_model_emit_subsumption_reverifies|src/v4/test/claim/manual/dissolution_subsumption_reverification.dag|run_rust_language_model_emit_subsumption_reverifies'

lookup_pin() { # $1=claim_id -> echoes observation|reason
  local cid="$1" line obs reason
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ "${line%%|*}" == "$cid" ]]; then
      obs="${line#*|}"; obs="${obs%%|*}"
      reason="${line##*|}"
      printf '%s|%s' "$obs" "$reason"
      return 0
    fi
  done <<< "$PIN_TABLE"
  return 1
}

lookup_run() { # $1=claim_id -> echoes entry|run_fn
  local cid="$1" line
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ "${line%%|*}" == "$cid" ]]; then
      printf '%s' "${line#*|}"
      return 0
    fi
  done <<< "$RUN_TABLE"
  return 1
}

classify_output() { # stdin=output -> prints observation|reason
  local out
  out="$(cat)"
  if grep -q "non-exhaustive pattern match on: ClassifiedDependencyView" <<< "$out"; then
    printf 'HostRejected|interpreter_classified_dependency_view_non_exhaustive'
    return 0
  fi
  if grep -q "runtime error:" <<< "$out"; then
    printf 'HostRejected|interpreter_runtime_error'
    return 0
  fi
  local verdict reason
  verdict="$(grep -oE 'verdict: (Pass|Fail|Deferred)' <<< "$out" | head -1 | awk '{print $2}')"
  reason="$(grep -oE 'reason: [a-z_0-9]+' <<< "$out" | head -1 | awk '{print $2}')"
  case "$verdict" in
    Pass) printf 'ExecutedPass|%s' "${reason:-corpus_eval_verdict_pass_reason}" ;;
    Fail) printf 'ExecutedFail|%s' "${reason:-unknown_fail_reason}" ;;
    Deferred) printf 'ExecutedDeferred|%s' "${reason:-unknown_deferred_reason}" ;;
    *) printf 'HostRejected|unclassified_transport_outcome' ;;
  esac
}

drift_count=0
receipt_json='['
first=1

while IFS= read -r pin_line; do
  [[ -z "$pin_line" ]] && continue
  claim_id="${pin_line%%|*}"
  rest="${pin_line#*|}"
  exp_obs="${rest%%|*}"
  exp_reason="${rest##*|}"

  run_spec="$(lookup_run "$claim_id")" || {
    echo "::error title=corpus eval config::missing run spec for $claim_id" >&2
    exit 2
  }
  entry="${run_spec%%|*}"
  run_fn="${run_spec##*|}"

  echo "== corpus eval: $claim_id ($entry::$run_fn) ==" >&2
  set +e
  out="$("$bin" run --source-root src/v4 --entry "$entry" --function "$run_fn" 2>&1)"
  run_ex=$?
  set -e

  actual="$(classify_output <<< "$out")"
  act_obs="${actual%%|*}"
  act_reason="${actual##*|}"

  drift=false
  if [[ "$act_obs" != "$exp_obs" || "$act_reason" != "$exp_reason" ]]; then
    drift=true
    drift_count=$((drift_count + 1))
    echo "::error title=corpus eval drift::$claim_id expected ${exp_obs}/${exp_reason} got ${act_obs}/${act_reason} (exit=$run_ex)" >&2
  else
    echo "::notice title=corpus eval pin ok::$claim_id ${act_obs}/${act_reason}" >&2
  fi

  if [[ "$first" -eq 1 ]]; then first=0; else receipt_json+=','; fi
  receipt_json+=$(printf '{"claim_id":"%s","expected":{"observation":"%s","reason":"%s"},"actual":{"observation":"%s","reason":"%s"},"drift":%s}' \
    "$claim_id" "$exp_obs" "$exp_reason" "$act_obs" "$act_reason" "$drift")
done <<< "$PIN_TABLE"

receipt_json+=']'
echo "corpus_eval_receipt=$receipt_json"

if [[ "$drift_count" -gt 0 ]]; then
  echo "::error title=corpus eval drift gate::${drift_count} row(s) drifted from tracked expectation pins" >&2
  exit 1
fi

echo "::notice title=corpus eval tracked-expectation gate::all pinned rows matched (0 drift)"
exit 0
