#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 modeled TestClaim corpus runner host harness.
#
# 🟡 gated — feature:t38-testclaim-corpus-eval — bind src/v4/TASKS.md T-38 —
# dissolve-on-arrival: same trigger as scripts/v4-testclaim-corpus-gate.sh —
# this harness replaces the structural bridge when the modeled runner
# v4.test.claim.workflow.testclaim_corpus_runner.run_manual_testclaim_corpus_eval
# emits structured TestClaimRun verdicts from a v2-compiler emit-Rust subset
# build over the wedge roster (manual_corpus_node_runtime_value_rows). When the
# wedge expands to cover the full manual corpus and the report-emit path is
# end-to-end exercised in CI, this harness replaces the structural bridge in
# the dissolution PR (deletes scripts/v4-testclaim-corpus-gate.sh + ci.yml
# step + dsl/gunbc/ci_github_actions_workflow.dag paired RunStep).
#
# Authority: src/v4/test/claim/workflow/testclaim_corpus_runner.dag
# (CorpusEvalReport entry point) + src/v4/workflow/ci.dag
# (TestClaimCorpusEvalCommand command/job/gate already declared).
#
# Receipt path (wedge):
#   1. v2-compiler compile --target rust src/v4 — already proven clean by
#      scripts/v4-testclaim-corpus-gate.sh; emits v4_test_claim_workflow_
#      testclaim_corpus_runner.rs with run_manual_testclaim_corpus_eval().
#   2. Structural string-match receipt over the emitted runner.rs proving the
#      wedge runner function and the three TestClaimRun<Node, RuntimeValue>
#      row references survived emit (proxy for "modeled runner is wired";
#      same posture the existing eval_runtime_mvp witness check already uses).
#   3. JSON verdict surface (stdout) + GITHUB_STEP_SUMMARY markdown table
#      derived from the modeled CorpusEvalReport.entries / counts via the
#      emitted Rust corpus runner once the M1 cargo-clean wall lifts.
#
# Until step (3) lands cleanly, this harness FAILS-CLOSED on receipt absence
# and emits Deferred markers under V4_TESTCLAIM_CORPUS_EVAL_DEFERRED=1, so the
# CI step honestly reports modeled-runner status instead of green-on-stub.
#
# Env:
#   V2_COMPILER                     — v2-compiler binary (default: target/release/gunbc)
#   V4_TESTCLAIM_CORPUS_OUT         — rust emit output dir (default: $RUNNER_TEMP/v4-testclaim-corpus-eval or /tmp)
#   V4_TESTCLAIM_CORPUS_LOG         — compile log (default: ${OUT}.compile.log)
#   V4_TESTCLAIM_CORPUS_TIMEOUT_SECS — optional compile timeout (CI default: 240)
#   V4_TESTCLAIM_CORPUS_EVAL_DEFERRED — 1 to surface JSON Deferred verdicts when
#                                       cargo-clean wall blocks runtime invocation
#                                       (default: 1 until M1 subset strategy lands)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
tmp_root="${RUNNER_TEMP:-/tmp}"
out="${V4_TESTCLAIM_CORPUS_OUT:-${tmp_root}/v4-testclaim-corpus-eval-${run_suffix}}"
log="${V4_TESTCLAIM_CORPUS_LOG:-${out}.compile.log}"
rm -rf "$out"
mkdir -p "$out" "$(dirname "$log")"

compile_timeout="${V4_TESTCLAIM_CORPUS_TIMEOUT_SECS:-}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "$compile_timeout" ]]; then
  compile_timeout=240
fi

echo "=== T-38: v2-compiler compile --target rust src/v4 (corpus eval harness) ==="
set +e
if [[ -n "$compile_timeout" ]]; then
  timeout --preserve-status "$compile_timeout" \
    "$bin" compile --source-root src/v4 --output-dir "$out" --target rust 2>&1 | tee "$log"
else
  "$bin" compile --source-root src/v4 --output-dir "$out" --target rust 2>&1 | tee "$log"
fi
status=${PIPESTATUS[0]}
set -e

if [[ "$status" -ne 0 ]]; then
  echo "error: v4 corpus-eval compile --target rust exited $status (log: $log)" >&2
  exit "$status"
fi

if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
  echo "error: v4 corpus-eval compile --target rust did not emit a clean compiled receipt" >&2
  exit 1
fi

runner_rs="${out}/src/v4_test_claim_workflow_testclaim_corpus_runner.rs"
if [[ ! -s "$runner_rs" ]]; then
  echo "error: expected emitted modeled-runner file at $runner_rs" >&2
  exit 1
fi

# Structural witness — proves the modeled runner survived emit and the wedge
# roster references are intact. Same posture as
# scripts/v4-testclaim-corpus-gate.sh's eval_runtime_mvp witness check.
python3 - "$runner_rs" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path


class WedgeError(Exception):
    pass


required_present = [
    "pub fn run_manual_testclaim_corpus_eval",
    "manual_corpus_node_runtime_value_rows",
    "run_eval_mvp2_test_claim_route",
    "run_rust_language_model_emit_mechanical_reverification_claim",
    "run_rust_language_model_emit_subsumption_reverifies",
    "corpus_verdict_pass_tag",
    "corpus_verdict_fail_tag",
    "corpus_verdict_deferred_tag",
]


def check(path: Path) -> None:
    source = path.read_text()
    for needle in required_present:
        if needle not in source:
            raise WedgeError(f"{path}: missing modeled-runner symbol: {needle}")


try:
    check(Path(sys.argv[1]))
except WedgeError as err:
    raise SystemExit(f"error: {err}") from err

print("T-38 modeled-runner wedge structural receipt PASS: emitted runner.rs carries entry symbol + roster references.")
PY

deferred="${V4_TESTCLAIM_CORPUS_EVAL_DEFERRED:-1}"
if [[ "$deferred" == "1" ]]; then
  # Until the M1 cargo-clean subset strategy lands the runtime invocation,
  # surface honest Deferred verdicts so the CI step does not green-on-stub.
  # JSON shape mirrors CorpusEvalReport.entries; non-zero exit not yet
  # produced because all rows are Deferred (no Fail). Pass/Fail will appear
  # once the runner actually executes against the emitted Rust target.
  printf '%s\n' \
    '{' \
    '  "schema": "v4.test.claim.workflow.testclaim_corpus_runner.CorpusEvalReport",' \
    '  "deferred_reason": "M1 cargo-clean wall — modeled runner emitted, runtime invocation pending v2-compiler emit-Rust subset strategy",' \
    '  "entries": [' \
    '    { "label": "eval routes TestClaim through runtime interpretation", "verdict_tag": "corpus_verdict_deferred_tag" },' \
    '    { "label": "rust language model emit mechanical reverification", "verdict_tag": "corpus_verdict_deferred_tag" },' \
    '    { "label": "rust language model emit subsumption reverifies", "verdict_tag": "corpus_verdict_deferred_tag" }' \
    '  ],' \
    '  "pass_count": 0,' \
    '  "fail_count": 0,' \
    '  "deferred_count": 3,' \
    '  "total": 3' \
    '}'

  if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    {
      echo "### T-38 modeled TestClaim corpus runner (wedge)"
      echo ""
      echo "| label | verdict |"
      echo "| --- | --- |"
      echo "| eval routes TestClaim through runtime interpretation | Deferred |"
      echo "| rust language model emit mechanical reverification | Deferred |"
      echo "| rust language model emit subsumption reverifies | Deferred |"
      echo ""
      echo "Pass: 0 · Fail: 0 · Deferred: 3 · Total: 3"
      echo ""
      echo "_Deferred reason: modeled runner emitted from v2-compiler --target rust; runtime invocation pending emit-Rust subset strategy. See src/v4/test/claim/workflow/testclaim_corpus_runner.dag._"
    } >> "$GITHUB_STEP_SUMMARY"
  fi
  exit 0
fi

echo "error: V4_TESTCLAIM_CORPUS_EVAL_DEFERRED=0 set but runtime invocation path not yet wired" >&2
exit 1
