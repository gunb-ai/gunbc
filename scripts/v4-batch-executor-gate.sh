#!/usr/bin/env bash
# Must-pass v4 batch-executor CI gate — the near-term dogfood where the
# v4.workflow.executor .dag is the BATCHING AUTHORITY for the claim suite.
#
# `claim_executor` evaluates the executor-decided plan
# (batch_runner.dag::bre_claim_batches → List<List<ClaimRef>>) and runs the
# suite batch-by-batch, claims-in-parallel. This gate asserts:
#
#   green:   the plan yields 2 batches and every claim passes (exit 0). The
#            executor places the gating claim in batch 1 and the two derived
#            claims in batch 2 (a later readiness layer).
#
#   --perturb-check adds two discriminating receipts:
#     authority:  empty the .dag dependency list in a temp tree → the plan must
#                 collapse to 1 batch. Proves the host CONSUMES the .dag's
#                 batching (it is not a hand-coded mirror): change the model,
#                 the batches change, with zero host edit.
#     fail-closed: rewrite the batch-1 (gating) witness body to `false` in a temp
#                 tree → the run must exit 1 AND never reach batch 2. Proves the
#                 executor's dependency ordering actually gates execution.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CLAIM_EXECUTOR:-target/release/claim_executor}"
plan_entry="src/v4/test/claim/workflow/batch_runner.dag"
perturb=0

case "${1:-}" in
  --perturb-check) perturb=1 ;;
  "") ;;
  *) echo "usage: $0 [--perturb-check]" >&2; exit 2 ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: claim_executor not found at $bin" >&2
  exit 2
fi

echo "== green: executor-decided batches, every claim passes =="
out="$("$bin" --source-root src/v4 --plan-entry "$plan_entry" 2>&1)"
echo "$out"
if ! grep -q "executor plan = 2 batch(es)" <<<"$out"; then
  echo "FAIL: expected 2 executor-decided batches" >&2
  exit 1
fi
if grep -q '^FAIL' <<<"$out"; then
  echo "FAIL: a claim failed in the green pass" >&2
  exit 1
fi
echo "green OK"

if [[ "$perturb" -eq 0 ]]; then
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "== perturb/authority: emptying the .dag dependency list must rebatch 2 -> 1 =="
cp -r src/v4 "$tmp/v4"
python3 - "$tmp/v4/test/claim/workflow/batch_runner.dag" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
needle = """data bre_suite_dependencies: List<DependencyView> = [
  bre_core_to_a_dependency,
  bre_core_to_b_dependency
]"""
if needle not in s:
    raise SystemExit("perturb: dependency-list anchor not found")
open(p, "w").write(s.replace(needle, "data bre_suite_dependencies: List<DependencyView> = []"))
PY
out_auth="$("$bin" --source-root "$tmp/v4" --plan-entry "$tmp/v4/test/claim/workflow/batch_runner.dag" 2>&1)"
echo "$out_auth"
if ! grep -q "executor plan = 1 batch(es)" <<<"$out_auth"; then
  echo "FAIL: removing the .dag dependency did NOT rebatch to 1 — host is not consuming the executor" >&2
  exit 1
fi
echo "authority OK (model drives batching)"

echo "== perturb/fail-closed: a false batch-1 gating claim must stop before batch 2 =="
# The claim entry path inside batch_runner.dag is repo-relative, so the host runs
# the real affected_set file. Perturb it in place and restore unconditionally.
gating_file="src/v4/test/claim/workflow/affected_set_ci_runner.dag"
cp "$gating_file" "$tmp/gating.orig"
restore_gating() { cp "$tmp/gating.orig" "$gating_file"; }
trap 'restore_gating; rm -rf "$tmp"' EXIT
python3 - "$gating_file" <<'PY'
import sys
p = sys.argv[1]
fn = "ci_runner_narrow_selection_holds"
s = open(p).read()
start = s.find(f"fn {fn}(")
if start < 0:
    raise SystemExit(f"perturb: function {fn} not found")
brace = s.find("{", start)
depth = 0
i = brace
while i < len(s):
    if s[i] == "{":
        depth += 1
    elif s[i] == "}":
        depth -= 1
        if depth == 0:
            break
    i += 1
open(p, "w").write(s[:brace] + "{\n  false\n}" + s[i + 1:])
PY
set +e
out_fc="$("$bin" --source-root src/v4 --plan-entry "$plan_entry" 2>&1)"
code=$?
set -e
restore_gating
trap 'rm -rf "$tmp"' EXIT
echo "$out_fc"
if [[ "$code" -eq 0 ]]; then
  echo "FAIL: a false gating claim still exited 0" >&2
  exit 1
fi
if grep -q "batch 2" <<<"$out_fc"; then
  echo "FAIL: batch 2 ran despite a failed gating batch 1 (ordering not enforced)" >&2
  exit 1
fi
echo "fail-closed OK (executor ordering gates execution)"

echo "== perturb/degenerate: an empty suite (0 batches) must fail closed, not pass on 0 claims =="
rm -rf "$tmp/v4"
cp -r src/v4 "$tmp/v4"
python3 - "$tmp/v4/test/claim/workflow/batch_runner.dag" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
needle = """data bre_suite_nodes: List<Node> = [
  bre_core_node,
  bre_a_node,
  bre_b_node
]"""
if needle not in s:
    raise SystemExit("perturb: suite-nodes anchor not found")
open(p, "w").write(s.replace(needle, "data bre_suite_nodes: List<Node> = []"))
PY
set +e
out_empty="$("$bin" --source-root "$tmp/v4" --plan-entry "$tmp/v4/test/claim/workflow/batch_runner.dag" 2>&1)"
code_empty=$?
set -e
echo "$out_empty"
if [[ "$code_empty" -eq 0 ]]; then
  echo "FAIL: a 0-batch executor plan exited 0 (vacuous pass)" >&2
  exit 1
fi
echo "degenerate OK (empty plan fails closed)"

echo "ALL OK"
