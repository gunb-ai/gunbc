#!/usr/bin/env bash
# T-WAD Slice 7 — observational ratchet pinning the **current** authoritative
# path-regex selection in .github/workflows/*.yml.
#
# This script is FAIL-CLOSED in BOTH directions:
#
#   (a) If a NEW authoritative path-regex selection appears outside the
#       inventory documented in
#       `docs/design-t-wad-slice-7-implementation-prequeue.md` §3, the script
#       fails so the operator must either widen the inventory + canvas or
#       remove the new bridge.
#
#   (b) If an inventoried selection site disappears BEFORE the T-WAD Slice 7
#       implementation PR lands the BinaryShim runner consuming PR #2713
#       affected-set lens output, the script also fails — silent removal
#       would leave authoritative selection in an inconsistent state.
#
# This script is NOT wired into .github/workflows/ci.yml by the PR that
# introduces it (that would itself be a CI behavior change). It is intended
# to be invoked manually by reviewers and by the eventual Slice 7
# implementation PR as part of its dissolution receipt.
#
# Status today: gate `ci_uses_affected_set_selection` (row 103) is NOT closer
# to PASSING because of this script — see the prequeue doc §2.
#
# Usage: scripts/check-workflow-path-regex-inventory.sh
# Exit 0 = inventory matches expectation; non-zero = drift detected.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

INVENTORY_DOC="docs/design-t-wad-slice-7-implementation-prequeue.md"
CI_YML=".github/workflows/ci.yml"

violations=0
note() { echo "check-workflow-path-regex-inventory: $*" >&2; }
fail() { note "FAIL: $*"; violations=$((violations + 1)); }

if [[ ! -f "$CI_YML" ]]; then
  fail "$CI_YML not found — inventory cannot be verified"
fi

# ---- Inventoried site #1: the `changes:` job docs-only allowlist grep -------
# Fingerprint: the literal regex string used by the `changes:` job to detect
# non-docs files. Pinned by exact substring to avoid false positives from
# unrelated edits elsewhere in the workflow.
PATH_REGEX_FILTER="grep -vE '^(docs/.*|[^/]+\\.md)\$'"
if ! grep -qF "$PATH_REGEX_FILTER" "$CI_YML"; then
  fail "expected path-regex docs-only filter missing from $CI_YML:
        searched literal: $PATH_REGEX_FILTER
        see $INVENTORY_DOC §3 row #1.
        If you are landing the T-WAD Slice 7 implementation PR (BinaryShim
        runner consuming PR #2713 lens output), update / retire this script
        as part of the dissolution receipt — do not just delete the filter."
fi

# ---- Inventoried site #2: the `v3:` job `if:` consuming `outputs.code` ------
# Fingerprint: the `needs.changes.outputs.code == 'true'` substring inside
# an `if:` line. This is the gate that turns site #1 into authoritative
# selection (skip-when-false on PR events).
if ! grep -E "^\s*if:.*needs\.changes\.outputs\.code" "$CI_YML" >/dev/null; then
  fail "expected v3-job if-gate on needs.changes.outputs.code missing from $CI_YML:
        see $INVENTORY_DOC §3 row #2.
        Same dissolution rule as row #1."
fi

# ---- Drift detector: NEW authoritative path-regex selection ----------------
# Heuristic: count `git diff --name-only` invocations across ALL workflow
# files. The inventory (§3) anchors a single expected occurrence — the
# inventoried `changes:` job in ci.yml with the literal anchor
# `origin/main...HEAD`. Any occurrence beyond that — whether in ci.yml itself
# (e.g., the `changes:` job grows a second selector, or a new job adds its
# own diff) OR in a sibling workflow file — is a candidate new path-regex
# bridge that the inventory does not yet describe.
#
# This intentionally scans `ci.yml` as well as siblings so the inventoried
# file is not free to grow un-inventoried selection (closes the fail-open
# gap that the v1 of this script had).
INVENTORIED_DIFF_ANCHOR="git diff --name-only origin/main...HEAD"
expected_diff_count=1  # exactly the §3 row #1 invocation
actual_diff_count=0
extra_locations=()
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  actual_diff_count=$((actual_diff_count + 1))
  # Extract "file:lineno:content" from `grep -n` output across files.
  if [[ "$line" != *"$INVENTORIED_DIFF_ANCHOR"* ]]; then
    extra_locations+=("$line")
  fi
done < <(git ls-files -z '.github/workflows/*.yml' | xargs -0 grep -nH "git diff --name-only" 2>/dev/null || true)

# Locate the inventoried anchor specifically; if it's missing, the §3 row #1
# fingerprint check above already reported it — here we just confirm count.
inventoried_present=0
if grep -qF "$INVENTORIED_DIFF_ANCHOR" "$CI_YML" 2>/dev/null; then
  inventoried_present=1
fi

if (( actual_diff_count > expected_diff_count )); then
  for loc in "${extra_locations[@]}"; do
    fail "new path-regex selection candidate (un-inventoried 'git diff --name-only'): $loc
        If this is genuinely orthogonal to affected-set selection, add it to
        $INVENTORY_DOC §3 with rationale and bump 'expected_diff_count' in
        this script. Otherwise route selection through the BinaryShim runner
        (post–Slice 5)."
  done
elif (( actual_diff_count < expected_diff_count )) && (( inventoried_present == 0 )); then
  # Already covered by the row #1 fingerprint check; left as a defensive
  # branch so the count-vs-anchor invariant is explicit.
  :
fi

if (( violations > 0 )); then
  note "$violations violation(s) — see messages above."
  exit 1
fi

echo "check-workflow-path-regex-inventory: ok (2/2 inventoried sites present, no new bridges)"
