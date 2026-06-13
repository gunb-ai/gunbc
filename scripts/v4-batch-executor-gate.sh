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

phase_name=""
phase_started=0
phase_begin() {
  phase_name="$1"
  phase_started=$SECONDS
  echo "::group::${phase_name}"
}
phase_end() {
  echo "::endgroup::"
  echo "::notice title=gate timing::${phase_name} took $((SECONDS - phase_started))s"
}

gate_started=$SECONDS

phase_begin "executor green: 2-batch plan, every claim passes"
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
phase_end

if [[ "$perturb" -eq 0 ]]; then
  echo "::notice title=gate timing::executor gate total took $((SECONDS - gate_started))s"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

phase_begin "executor perturb/authority: empty dependency list rebatches 2 -> 1"
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
phase_end

phase_begin "executor perturb/fail-closed: false batch-1 gating claim stops before batch 2"
# Hermetic like the other two perturbs: work entirely in a tmp/v4 copy. The
# claim entry inside batch_runner.dag is repo-relative, so we also repoint
# bre_suite_entry at the tmp copy — then the host resolves the perturbed file,
# never the tracked one (no in-place mutation of the working tree).
rm -rf "$tmp/v4"
cp -r src/v4 "$tmp/v4"
tmp_gating="$tmp/v4/test/claim/workflow/affected_set_ci_runner.dag"
python3 - "$tmp/v4/test/claim/workflow/batch_runner.dag" "$tmp_gating" <<'PY'
import sys
br, gating = sys.argv[1], sys.argv[2]
# Repoint the suite entry at the tmp copy.
s = open(br).read()
needle = 'data bre_suite_entry: String = "src/v4/test/claim/workflow/affected_set_ci_runner.dag"'
if needle not in s:
    raise SystemExit("perturb: bre_suite_entry anchor not found")
open(br, "w").write(s.replace(needle, f'data bre_suite_entry: String = "{gating}"'))
# Rewrite the gating witness body to `false` in the tmp copy.
fn = "ci_runner_narrow_selection_holds"
g = open(gating).read()
start = g.find(f"fn {fn}(")
if start < 0:
    raise SystemExit(f"perturb: function {fn} not found")
brace = g.find("{", start)
depth = 0
i = brace
while i < len(g):
    if g[i] == "{":
        depth += 1
    elif g[i] == "}":
        depth -= 1
        if depth == 0:
            break
    i += 1
open(gating, "w").write(g[:brace] + "{\n  false\n}" + g[i + 1:])
PY
set +e
out_fc="$("$bin" --source-root "$tmp/v4" --plan-entry "$tmp/v4/test/claim/workflow/batch_runner.dag" 2>&1)"
code=$?
set -e
echo "$out_fc"
if [[ "$code" -eq 0 ]]; then
  echo "FAIL: a false gating claim still exited 0" >&2
  exit 1
fi
if grep -q "batch 2" <<<"$out_fc"; then
  echo "FAIL: batch 2 ran despite a failed gating batch 1 (ordering not enforced)" >&2
  exit 1
fi
phase_end

phase_begin "executor perturb/degenerate: empty suite (0 batches) fails closed"
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
phase_end

echo "::notice title=gate timing::executor gate total took $((SECONDS - gate_started))s"
echo "ALL OK"
