#!/usr/bin/env bash
# Execute the contract-stamp guard against the real bytes, and fail if it does not refuse what it
# is supposed to refuse.
#
# WHY THIS FILE EXISTS, AND WHY IT RUNS THINGS. The witness in
# dag/test/claim/compiler_gate_contract_stamp_witness.dag asserts the guard's SHAPE; this asserts
# its BEHAVIOUR by running it. The distinction earned its keep: the fused `-v key=` defect made the
# awk option token `'-v key='` with a trailing space, mawk died with
#     awk: fatal: ` key' is not a legal variable name
# the capture came back empty on EVERY input, and the guard refused a correctly stamped tree with
# StampAbsent. The four RED cases still exited 1 with StampAbsent -- still "passed" -- while the
# guard was dead, refusing every tree including correct ones. A RED that passes for the wrong reason
# is the same shape as the silent green recorded in
# dag/gunbc/recurring_failure_mode/instruction_and_subject_resolved_from_different_revisions.dag.
# Only the cases that must PROCEED separate a working guard from a dead one, so every case below
# asserts exit code AND diagnostic.
#
# WHAT IT RUNS IS THE COMMITTED ARTIFACT, NOT A COPY OF IT. extract_guard reads the script out of
# .github/workflows/witnesses.yml byte-exactly; nothing is retyped, because a retyped copy can pass
# while the real bytes fail. Each case runs under `bash -e -o pipefail`, GitHub's default `run:`
# shell, and `${{ env.* }}` is substituted the way Actions would substitute it.
#
# bash and awk only, no python: .gitignore puts *.py "outside the committed substrate", so a
# committed test cannot depend on a file the repo refuses to track. The runner image's awk is mawk,
# the same implementation the fleet's runners use.
#
# Usage:  ./test/compiler_gate_contract_stamp/run_evidence.sh
# Exit:   0 only if every assertion holds.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
committed="$repo/.github/workflows/witnesses.yml"
fixture="$here/fixtures/silent-green-head-witnesses.yml"
step_name='- name: Refuse a subject older than the instruction running it'
placeholder='${{ env.GUNBC_COMPILER_GATE_CONTRACT_STAMP }}'

fail=0
guard_template="$(mktemp)"
tmpdir="$(mktemp -d)"
cleanup() { rm -f "$guard_template"; [ -n "$tmpdir" ] && rm -rf "$tmpdir"; }
trap 'cleanup' EXIT

bail() { echo "FAIL: $*"; exit 1; }
command -v awk >/dev/null 2>&1 || bail "awk is required"
[ -f "$committed" ] || bail "no committed workflow at $committed"
[ -f "$fixture" ] || bail "no silent-green fixture at $fixture"

# ---------------------------------------------------------------------------------------------
# extract_guard <file>  --  print the guard step's run body, byte exactly.
# A structural scan whose edges are asserted rather than assumed:
#   * locate the step by its exact `- name:` line; anything other than exactly one is a failure;
#   * the next line must be a `run:` block scalar (|- or |+ style);
#   * the block's indentation is READ from its own first non-blank line, and every deeper line is
#     re-emitted dedented by exactly that many spaces;
#   * the scan stops at the first non-blank line shallower than the block, so body lines cannot be
#     truncated early;
#   * an empty body, a missing `run:` header, or a body not deeper than its own step name is a loud
#     failure -- never a silently short script.
# ---------------------------------------------------------------------------------------------
extract_guard() {
  awk -v step="$step_name" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    function indent(s) { if (match(s, /[^ \t]/) == 0) return -1; return match(s, /[^ \t]/) - 1 }
    function strip(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    BEGIN { state = "seek"; found = 0; printed = 0; pending = 0 }
    state == "seek" {
      if (strip($0) == step) { found++; name_indent = indent($0); state = "want_run" }
      next
    }
    state == "want_run" {
      if (strip($0) == "") next
      if (strip($0) !~ /^run: [-+]?[0-9]*[|>][-+]?$/)
        die("guard step is not a `run:` block scalar (got: " strip($0) ")")
      state = "body"; body_indent = -1
      next
    }
    state == "body" {
      # a blank line -- truly empty or spaces-only -- is part of the block; only a NON-blank
      # line at a shallower indent terminates it. (mawk has no // comments; # is the form.)
      if ($0 == "" || indent($0) < 0) { pending++; next }
      ind = indent($0)
      if (body_indent < 0) {
        body_indent = ind
        if (body_indent <= name_indent) die("guard body is not deeper than its own step name")
      } else if (ind < body_indent) { state = "done"; next }
      while (pending > 0) { print ""; pending-- }
      print substr($0, body_indent + 1)
      printed++
      next
    }
    END {
      if (found != 1) die("expected exactly one guard step, found " found)
      if (state != "done") die("guard block never terminated")
      if (printed == 0) die("guard body is empty")
    }
  ' "$1"
}

# emit_guard <running-value> <dest>  --  substitute ${{ env.X }} the way Actions would.
emit_guard() {
  awk -v ph="$placeholder" -v run="$1" '
    {
      out = ""
      while ((p = index($0, ph)) > 0) { out = out substr($0, 1, p - 1) run; $0 = substr($0, p + length(ph)) }
      print out $0
    }
  ' "$guard_template" > "$2"
}

# rewrite_stamp <new-stamp> <in> <out>  --  move the workflow-level stamp member, as a PR that
# legitimately changes the invoked contract would. Refuses if the member is not there to move.
rewrite_stamp() { # <value> <in> <out>
  awk -v new="$1" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ {
      sub(/:.*/, ": " new); moved++
    }
    { print }
    END { if (moved != 1) die("expected exactly one workflow-level stamp member, rewrote " moved) }
  ' "$2" > "$3"
}

# demote_stamp <running-value> <in> <out>  --  remove the workflow-level member and hang the same
# key off the FIRST step's env, so it is textually present and structurally absent. The tree is not
# allowed to hand the judge its own evidence.
demote_stamp() { # <value> <in> <out>
  awk -v run="$1" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ { removed++; next }
    {
      print
      if (!inserted && $0 ~ /^[ \t]*-[ \t]+name:/) {
        printf "          env:\n"
        printf "            GUNBC_COMPILER_GATE_CONTRACT_STAMP: %s\n", run
        inserted = 1
      }
    }
    END {
      if (removed != 1) die("expected exactly one workflow-level stamp member, removed " removed)
      if (inserted != 1) die("no step to hang the demoted env: off of")
    }
  ' "$2" > "$3"
}

# The running side is what the generated workflow actually declares at workflow level. A value that
# is not a 16-hex stamp is no evidence at all, so this aborts rather than silently comparing garbage.
# STRUCTURE FIRST, THEN CONTENT. If the step is missing, is not a block scalar, or never
# terminates, the extractor says so; reporting "no readable stamp" for a workflow whose guard
# step has gone missing would blame the wrong fact and send a reader to the wrong line.
extract_guard "$committed" > "$guard_template" || bail "could not extract the guard from the committed workflow"
[ -s "$guard_template" ] || bail "extracted guard is empty"

RUNNING="$(grep -o 'GUNBC_COMPILER_GATE_CONTRACT_STAMP: [0-9a-f]\{16\}' "$committed" | head -1 | awk '{print $2}')"
[ -n "$RUNNING" ] || bail "the committed workflow declares no readable 16-hex contract stamp"

echo "== awk: $(awk --version 2>&1 | head -1)"
echo "== guard extracted from the committed artifact: $(grep -c '' < "$guard_template") lines, sha256 $(sha256sum < "$guard_template" | cut -d' ' -f1 | cut -c1-16)"
echo "== running-side stamp (workflow-level literal): $RUNNING"
run_case() { # <label> <tree-yml> <running-value> <expect-rc> <expect-substring>...
  local label="$1" tree="$2" running="$3" expect_rc="$4"; shift 4
  local d="$tmpdir/$(printf '%s' "$label" | tr -c '[:alnum:]' '_')"
  rm -rf "$d"; mkdir -p "$d/.github/workflows"
  cp "$tree" "$d/.github/workflows/witnesses.yml"
  emit_guard "$running" "$d/guard.sh"
  local out rc needle
  out="$(cd "$d" && bash -e -o pipefail ./guard.sh 2>&1)"; rc=$?
  printf '%-46s rc=%s expect=%s | %s\n' "$label" "$rc" "$expect_rc" \
    "$(printf '%s' "$out" | tr '\n' ' ' | cut -c1-118)"
  if [ "$rc" != "$expect_rc" ]; then
    echo "   ^^ ASSERTION FAILED (exit code, expected $expect_rc)"; fail=1
  fi
  for needle in "$@"; do
    if ! printf '%s' "$out" | grep -q -- "$needle"; then
      echo "   ^^ ASSERTION FAILED (diagnostic lacks '$needle')"; fail=1
    fi
  done
}

echo ""
echo "== RED: the silent-green head must now be REFUSED, not green =="
# The four measured runs shared ONE committed workflow artifact, so this fixture stands for all
# four; fixtures/README.md carries the run ids and each run's measurements.
run_case "RED silent-green head 5003782716" "$fixture" "$RUNNING" 1 "StampAbsent"

echo ""
echo "== CONTROLS =="
# Without these, the RED above is indistinguishable from a guard that refuses everything -- which is
# exactly what the fused `-v key=` render did while still passing every RED case.
run_case "CONTROL paired (head carries the stamp)" "$committed" "$RUNNING" 0 \
  "paired running=$RUNNING tree=$RUNNING"
run_case "CONTROL mismatch (running != tree)" "$committed" "deadbeefcafe0123" 1 \
  "StampMismatch" "deadbeefcafe0123" "$RUNNING"
run_case "CONTROL unreadable (no running stamp)" "$committed" "" 1 "StampUnreadable"

# A PR that LEGITIMATELY changes the invoked contract moves both sides at once: the head's own
# committed workflow carries the new stamp, and the merge-ref workflow the lane runs carries it too.
# The guard must proceed, or it is a wall and not a guard.
rewrite_stamp "0123456789abcdef" "$committed" "$tmpdir/contract_pr.yml" \
  || bail "could not build the contract-changing fixture"
run_case "CONTROL contract-changing PR (both new)" "$tmpdir/contract_pr.yml" "0123456789abcdef" 0 \
  "paired running=0123456789abcdef tree=0123456789abcdef"

# A step-level env: of the same name must NOT satisfy the workflow-level read.
demote_stamp "$RUNNING" "$committed" "$tmpdir/step_env.yml" \
  || bail "could not build the step-level-env fixture"
run_case "CONTROL step-level env is not the read" "$tmpdir/step_env.yml" "$RUNNING" 1 "StampAbsent"

echo ""
if [ "$fail" = "0" ]; then
  echo "== EVIDENCE VERDICT: ALL ASSERTIONS HELD (1 RED x 4 measured runs + 5 controls) =="
else
  echo "== EVIDENCE VERDICT: AT LEAST ONE ASSERTION FAILED =="
fi
exit "$fail"
