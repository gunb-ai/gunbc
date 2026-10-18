#!/usr/bin/env bash
# Execute the contract-stamp guard against the real bytes, and fail if it does not refuse what it
# is supposed to refuse.
#
# WHY THIS EXISTS AS A FILE, AND WHY IT RUNS THINGS. The witness in
# dag/test/claim/compiler_gate_contract_stamp_witness.dag asserts the guard's SHAPE; this asserts
# its BEHAVIOUR by running it. The distinction earned its keep: the fused `-v key=` defect made
# the awk option token `'-v key='` with a trailing space, mawk died with
#   awk: fatal: ` key' is not a legal variable name
# the capture came back empty on EVERY input, and the guard refused a correctly stamped tree with
# StampAbsent. The four RED cases still exited 1 with StampAbsent and so still "passed" -- for a
# reason that had nothing to do with the tree. A RED that passes for the wrong reason is the same
# shape as the silent green recorded in
# dag/gunbc/recurring_failure_mode/instruction_and_subject_resolved_from_different_revisions.dag.
# Only the cases that must PROCEED distinguish a working guard from a dead one, so every case here
# asserts exit code AND diagnostic.
#
# WHAT IT RUNS IS THE COMMITTED ARTIFACT, NOT A COPY OF IT. extract_guard.py reads the script out
# of .github/workflows/witnesses.yml byte-exactly; it never retypes it. Each case runs under
# `bash -e -o pipefail`, which is GitHub's default `run:` shell, and `${{ env.* }}` is substituted
# the way Actions would substitute it.
#
# Usage:  ./test/compiler_gate_contract_stamp/run_evidence.sh
# Exit:   0 only if every assertion holds.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
committed="$repo/.github/workflows/witnesses.yml"
fixture="$here/fixtures/silent-green-head-witnesses.yml"
guard_template="$(mktemp)"
tmpdir=""
trap 'rm -f "$guard_template"; [ -n "$tmpdir" ] && rm -rf "$tmpdir"; true' EXIT

fail=0

# The tree under test must be the file GitHub would have read, so refuse to invent it.
[ -f "$committed" ] || { echo "FAIL: no committed $committed"; exit 1; }
python3 "$here/extract_guard.py" "$committed" > "$guard_template" || {
  echo "FAIL: could not extract the guard from the committed workflow"; exit 1; }
[ -s "$guard_template" ] || { echo "FAIL: extracted guard is empty"; exit 1; }

if ! command -v awk >/dev/null 2>&1; then echo "FAIL: awk is required"; exit 1; fi
echo "== awk: $(awk --version 2>&1 | head -1)"
echo "== guard extracted from the committed artifact: $(grep -c '' "$guard_template") lines"

# The running side is what the generated workflow actually declares at workflow level. Actions
# expands ${{ env.X }} from that mapping; a literal that is not a hex stamp is no evidence at all,
# so an unreadable value here aborts rather than silently producing a garbage comparison.
RUNNING="$(grep -o 'GUNBC_COMPILER_GATE_CONTRACT_STAMP: [0-9a-f]\{16\}' "$committed" | head -1 | awk '{print $2}')"
if [ -z "$RUNNING" ]; then
  echo "FAIL: the committed workflow declares no readable 16-hex contract stamp"
  exit 1
fi
echo "== running-side stamp (workflow-level literal): $RUNNING"

emit_guard() { # <running-value> <dest>
  python3 - "$1" "$2" <<'PY'
import io, os, sys
s = io.open(os.environ['GUARD_TEMPLATE'], encoding='utf-8').read()
io.open(sys.argv[2], 'w', encoding='utf-8').write(
    s.replace('${{ env.GUNBC_COMPILER_GATE_CONTRACT_STAMP }}', sys.argv[1]))
PY
}

tmpdir="$(mktemp -d)"

run_case() { # <label> <tree-yml> <running-value> <expect-rc> <expect-substring...>
  local label="$1" tree="$2" running="$3" expect_rc="$4"; shift 4
  local d="$tmpdir/$(printf '%s' "$label" | tr -c '[:alnum:]' '_')"
  rm -rf "$d"; mkdir -p "$d/.github/workflows"
  cp "$tree" "$d/.github/workflows/witnesses.yml"
  GUARD_TEMPLATE="$guard_template" emit_guard "$running" "$d/guard.sh"
  local out rc
  out="$(cd "$d" && bash -e -o pipefail ./guard.sh 2>&1)"; rc=$?
  printf '%-46s rc=%s expect=%s | %s\n' "$label" "$rc" "$expect_rc" \
    "$(printf '%s' "$out" | tr '\n' ' ' | cut -c1-118)"
  if [ "$rc" != "$expect_rc" ]; then
    echo "   ^^ ASSERTION FAILED (exit code)"; fail=1; return
  fi
  local needle
  for needle in "$@"; do
    if ! printf '%s' "$out" | grep -q -- "$needle"; then
      echo "   ^^ ASSERTION FAILED (diagnostic lacks '$needle')"; fail=1
    fi
  done
}

echo ""
echo "== RED: the silent-green head must now be REFUSED, not green =="
# The four measured runs shared one committed workflow artifact, so this one fixture stands for
# all four (see fixtures/README.md for the run ids and the per-run measurements).
run_case "RED silent-green head 5003782716" "$fixture" "$RUNNING" 1 "StampAbsent"

echo ""
echo "== CONTROLS =="
# Without these, the RED above is indistinguishable from a guard that refuses everything -- which
# is what the fused `-v key=` render did, and it passed the RED.
run_case "CONTROL paired (head carries the stamp)" "$committed" "$RUNNING" 0 \
  "paired running=$RUNNING tree=$RUNNING"
run_case "CONTROL mismatch (running != tree)" "$committed" "deadbeefcafe0123" 1 \
  "StampMismatch" "deadbeefcafe0123" "$RUNNING"
run_case "CONTROL unreadable (no running stamp)" "$committed" "" 1 "StampUnreadable"
# A PR that LEGITIMATELY changes the invoked contract moves both sides at once: the head's own
# committed workflow carries the new stamp, and the merge-ref workflow the lane runs carries it too.
# The guard must proceed, or it is a wall and not a guard.
d="$tmpdir/contract_pr"; mkdir -p "$d/.github/workflows"
python3 - "$committed" "$d/.github/workflows/witnesses.yml" <<'PY'
import io, re, sys
src, dst = sys.argv[1], sys.argv[2]
new = "0123456789abcdef"
y = io.open(src, encoding='utf-8').read()
if len(re.findall(r"GUNBC_COMPILER_GATE_CONTRACT_STAMP: [0-9a-f]+", y)) < 1:
    sys.exit("FAIL: no workflow-level stamp member to rewrite")
y = re.sub(r"GUNBC_COMPILER_GATE_CONTRACT_STAMP: [0-9a-f]+",
           "GUNBC_COMPILER_GATE_CONTRACT_STAMP: " + new, y)
io.open(dst, "w", encoding="utf-8").write(y)
PY
run_case "CONTROL contract-changing PR (both new)" "$d/.github/workflows/witnesses.yml" "0123456789abcdef" 0 \
  "paired running=0123456789abcdef tree=0123456789abcdef"

# A step-level env: of the same name must NOT satisfy the workflow-level read: the tree is not
# allowed to hand the judge its own evidence. Built by removing the workflow-level member and
# hanging the same key off the first step's env, so it is textually present and structurally absent.
d="$tmpdir/step_env"; mkdir -p "$d/.github/workflows"
python3 - "$committed" "$d/.github/workflows/witnesses.yml" "$RUNNING" <<'PY'
import io, sys
src, dst, run = sys.argv[1], sys.argv[2], sys.argv[3]
y = io.open(src, encoding='utf-8').read().split("\n")
y = [l for l in y if not l.strip().startswith("GUNBC_COMPILER_GATE_CONTRACT_STAMP:")]
for i, l in enumerate(y):
    if l.strip().startswith("- name:"):
        y[i:i + 1] = [l, "          env:", "            GUNBC_COMPILER_GATE_CONTRACT_STAMP: " + run]
        break
io.open(dst, "w", encoding="utf-8").write("\n".join(y))
PY
run_case "CONTROL step-level env is not the read" "$d/.github/workflows/witnesses.yml" "$RUNNING" 1 "StampAbsent"

echo ""
if [ "$fail" = "0" ]; then
  echo "== EVIDENCE VERDICT: ALL ASSERTIONS HELD (1 RED x 4 measured runs + 5 controls) =="
else
  echo "== EVIDENCE VERDICT: AT LEAST ONE ASSERTION FAILED =="
fi
exit "$fail"
