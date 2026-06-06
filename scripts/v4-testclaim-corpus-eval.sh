#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 tracked-expectation corpus eval — THIN host transport (RR-A §5.2 / A.3b).
# Invokes gunbc and projects modeled witnesses from manual_corpus_eval_expected.dag.
# Pin table + drift comparison authority lives in the .dag layer (not bash). HostRejected CDV rows:
# transport observes `corpus_eval_cdv_host_rejection_stderr_marker` from the entry .dag (interim
# pre-TestClaimRun precondition); modeled witnesses gate pin alignment — not host-owned drift logic.
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

claim_run_required() {
  local gate_fn="$1" ratchet_fn="$2" label="$3" target="$4"
  set +e
  local out
  out="$(claim_run "$gate_fn" 2>&1)"
  local ex=$?
  set -e
  printf '%s\n' "$out"
  if [[ "$ex" -eq 0 ]]; then
    return 0
  fi

  set +e
  local ratchet_out
  ratchet_out="$(claim_run "$ratchet_fn" 2>&1)"
  local ratchet_ex=$?
  set -e
  if [[ "$ratchet_ex" -eq 0 ]]; then
    printf '%s\n' "$ratchet_out"
    echo "::error title=corpus eval ratchet required::row ${label} greened (dissolution_target ${target} achieved); flip its pin PinnedRed->MustPass in src/v4/test/claim/workflow/manual_corpus_eval_expected.dag in this PR to lock the win." >&2
    exit 1
  fi

  return "$ex"
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

host_rejected_or_transport_pass_required() {
  local label="$1" target="$2" row_entry="$3" row_fn="$4" host_gate_fn="$5" pass_gate_fn="$6" pass_ratchet_fn="$7"
  set +e
  local out
  out="$(transport_run "$row_entry" "$row_fn" 2>&1)"
  local ex=$?
  set -e
  printf '%s\n' "$out"

  if grep -Fq "$cdv_host_rejection_marker" <<< "$out"; then
    claim_run "$host_gate_fn"
    return 0
  fi

  if grep -Fq "TestClaimRun { verdict: Pass" <<< "$out"; then
    claim_run_required "$pass_gate_fn" "$pass_ratchet_fn" "$label" "$target"
    return 0
  fi

  echo "::error title=corpus eval regression::${label} row no longer matched its HostRejected pin and did not produce a passing TestClaimRun." >&2
  return "$ex"
}

cdv_host_rejection_marker="$(dag_string_data corpus_eval_cdv_host_rejection_stderr_marker)"
if [[ -z "$cdv_host_rejection_marker" ]]; then
  echo "error: missing corpus_eval_cdv_host_rejection_stderr_marker in $entry" >&2
  exit 2
fi

# Modeled pin-table frontier + alignment witness.
claim_run witness_corpus_eval_tracked_expectation_closed

# Executed rows: runtime drift gate projects from TestClaimRun via modeled witnesses.
claim_run_required \
  witness_corpus_eval_row_eval_mvp2_runtime_gate \
  witness_corpus_eval_row_eval_mvp2_ratchet_forward \
  claim_eval_mvp2_test_claim_route \
  dissolution_target_transform_eval
claim_run_required \
  witness_corpus_eval_row_mechanical_reverification_runtime_gate \
  witness_corpus_eval_row_mechanical_reverification_ratchet_forward \
  claim_rust_language_model_emit_mechanical_reverification \
  dissolution_target_transform_eval
claim_run_required \
  witness_corpus_eval_row_subsumption_reverifies_runtime_gate \
  witness_corpus_eval_row_subsumption_reverifies_ratchet_forward \
  claim_rust_language_model_emit_subsumption_reverifies \
  dissolution_target_subsumption_tree_eval

# HostRejected rows: if the row now produces a passing TestClaimRun, the modeled pass gate forces
# a PinnedRed->MustPass source edit instead of letting the improvement pass silently.
host_rejected_or_transport_pass_required \
  claim_parallelism_data_dependency_run_test_claim_receipt \
  dissolution_target_cdv_eager_moth_810 \
  "src/v4/test/claim/lens_parallelism/data_dependency.dag" \
  "run_parallelism_data_dependency_receipt" \
  witness_corpus_eval_row_parallelism_host_rejected_gate \
  witness_corpus_eval_row_parallelism_transport_pass_gate \
  witness_corpus_eval_row_parallelism_transport_pass_ratchet_forward

host_rejected_or_transport_pass_required \
  claim_lens_effect_depends_on_runtime_verdict \
  dissolution_target_cdv_eager_moth_810 \
  "src/v4/test/claim/lens_effect/effect_depends_on.dag" \
  "run_lens_effect_depends_on_runtime_verdict" \
  witness_corpus_eval_row_effect_host_rejected_gate \
  witness_corpus_eval_row_effect_transport_pass_gate \
  witness_corpus_eval_row_effect_transport_pass_ratchet_forward

echo "::notice title=corpus eval tracked-expectation gate::all pinned rows matched (0 drift)"
exit 0
