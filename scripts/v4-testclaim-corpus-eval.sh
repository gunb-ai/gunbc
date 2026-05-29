#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 modeled TestClaim corpus runner host harness (T-38-PR1 SCAFFOLD).
#
# 🟡 gated — feature:t38-testclaim-corpus-eval — bind src/v4/TASKS.md T-38 —
# This harness covers the model-PR1 slice of T-38: it proves the modeled
# corpus runner compiles (dag + rust 0-diagnostic) and its emitted Rust
# entry symbols survived emit. It does NOT yet execute the runner — the
# wedge-real execution boundary is gated on M1 cargo-clean emit-Rust
# (full-tree cargo check on the emitted crate fails ~3660 errors as of
# 2026-05-29, dominated by FreeMonoid drop-check cycles + Nat cata Fn-clone
# patterns owned by the v2 emitter, NOT by this PR's modeled runner). The
# harness reports execution_status="blocked_m1_subset" with both compile
# receipts attached so CI surfaces honest fail-closed state rather than
# fake Pass verdicts.
#
# Dissolve-on-arrival: scripts/v4-testclaim-corpus-gate.sh and the paired
# ci.yml + ci_github_actions_workflow.dag step are NOT deleted by this PR
# (T-38-PR1 is the scaffold; the structural bridge keeps existing receipt
# coverage live). The dissolution PR (T-38-PR2) lands the runtime
# invocation, replaces "blocked_m1_subset" with real per-row Verdict
# emission, deletes the gate script + paired step, and wires the live
# `TestClaimCorpusEvalCommand` job step in ci.yml.
#
# Authority: src/v4/test/claim/workflow/testclaim_corpus_runner.dag
# (CorpusEvalReport entry point) + src/v4/workflow/ci.dag
# (TestClaimCorpusEvalCommand job/gate already declared).
#
# Env:
#   V2_COMPILER                     — v2-compiler binary (default: target/release/gunbc)
#   V4_TESTCLAIM_CORPUS_OUT         — rust emit output dir (default: $RUNNER_TEMP/v4-testclaim-corpus-eval or /tmp)
#   V4_TESTCLAIM_CORPUS_LOG         — compile log (default: ${OUT}.compile.log)
#   V4_TESTCLAIM_CORPUS_TIMEOUT_SECS — optional compile timeout (CI default: 240)

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

echo "=== T-38: v2-compiler compile --target rust src/v4 (corpus runner scaffold) ==="
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

compiled_receipt="$(grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" | tail -1 || true)"
if [[ -z "$compiled_receipt" ]]; then
  echo "error: v4 corpus-eval compile --target rust did not emit a clean compiled receipt" >&2
  exit 1
fi

runner_rs="${out}/src/v4_test_claim_workflow_testclaim_corpus_runner.rs"
roster_rs="${out}/src/v4_test_claim_manual_manual_corpus_roster.rs"
for path in "$runner_rs" "$roster_rs"; do
  if [[ ! -s "$path" ]]; then
    echo "error: expected emitted modeled-runner file at $path" >&2
    exit 1
  fi
done

# Structural witness — proves the modeled runner survived emit with the
# faithful Verdict<RuntimeValue> carrier (Codex review #3902 points 2/3/4
# acceptance) and the wedge roster references are intact. Same posture as
# the existing scripts/v4-testclaim-corpus-gate.sh eval_runtime_mvp witness.
python3 - "$runner_rs" "$roster_rs" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path


class WedgeError(Exception):
    pass


runner_required = [
    "pub fn run_manual_testclaim_corpus_eval",
    "pub struct CorpusEvalReport",
    "pub struct CorpusEvalEntry",
    "pub fn test_claim_label_of",
    "pub fn corpus_entry_label",
    "pub fn corpus_report_tally",
    "pub fn corpus_report_pass_count",
    "pub fn corpus_report_fail_count",
    "pub fn corpus_report_deferred_count",
    "pub fn corpus_report_total",
    "manual_corpus_node_runtime_value_rows",
    "ManualCorpusRow",
]

roster_required = [
    "manual_corpus_node_runtime_value_rows",
    "ManualCorpusRow",
    "run_eval_mvp2_test_claim_route",
    "run_rust_language_model_emit_mechanical_reverification_claim",
    "run_rust_language_model_emit_subsumption_reverifies",
    "claim_eval_mvp2_test_claim_route",
    "claim_rust_language_model_emit_mechanical_reverification",
    "claim_rust_language_model_emit_subsumption_reverifies",
]


def check(path: Path, needles: list[str]) -> None:
    source = path.read_text()
    for needle in needles:
        if needle not in source:
            raise WedgeError(f"{path}: missing modeled-runner symbol: {needle}")


try:
    check(Path(sys.argv[1]), runner_required)
    check(Path(sys.argv[2]), roster_required)
except WedgeError as err:
    raise SystemExit(f"error: {err}") from err

print("T-38 modeled-runner scaffold structural receipt PASS: emitted runner.rs + roster.rs carry entry symbol + roster row references.")
PY

# Honest report: scaffold landed and emit-clean; runtime invocation
# blocked on M1 subset crate strategy. JSON shape is forward-compatible
# with the eventual real per-row verdict surface (entries: [{label,
# verdict_tag}, ...]); empty `entries` here is the explicit, non-Deferred
# signal that no row has been executed yet.
files_emitted="$(echo "$compiled_receipt" | sed -n 's/^compiled: \([0-9]*\) files emitted, 0 diagnostics$/\1/p')"

cat <<JSON
{
  "schema": "v4.test.claim.workflow.testclaim_corpus_runner.CorpusEvalReport",
  "execution_status": "blocked_m1_subset",
  "blocked_reason": "wedge-real runtime invocation of run_manual_testclaim_corpus_eval requires a cargo-clean subset of the v2-compiler emit-Rust output; full-tree cargo check on the emitted crate currently fails (FreeMonoid drop-check cycle + Nat cata Fn-clone in v2_compiler_emit_rust — M1 emitter lane, not T-38 scope). Tracked as T-38-PR2.",
  "scaffold_receipts": {
    "rust_emit": { "status": "0-diagnostic", "files_emitted": ${files_emitted} },
    "emitted_runner_structural_witness": "PASS"
  },
  "entries": [],
  "pass_count": 0,
  "fail_count": 0,
  "deferred_count": 0,
  "total": 0
}
JSON

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### T-38 modeled TestClaim corpus runner — scaffold (PR1)"
    echo ""
    echo "**execution_status:** \`blocked_m1_subset\` — modeled runner emit-clean (\`gunbc compile --target rust\` 0-diagnostic), runtime invocation gated on M1 emit-Rust subset strategy (T-38-PR2 follow-on)."
    echo ""
    echo "| receipt | status |"
    echo "| --- | --- |"
    echo "| \`gunbc compile --source-root src/v4 --target rust\` | 0-diagnostic, ${files_emitted} files emitted |"
    echo "| emitted runner.rs + roster.rs structural witness (entry symbols + roster row refs) | PASS |"
    echo ""
    echo "_No per-row Pass/Fail/Deferred verdicts emitted by this PR — the JSON \`entries\` list is intentionally empty under \`execution_status=blocked_m1_subset\` rather than fake-Pass or fake-Deferred. See \`src/v4/test/claim/workflow/testclaim_corpus_runner.dag\` header and src/v4/TASKS.md §T-38._"
  } >> "$GITHUB_STEP_SUMMARY"
fi

exit 0
