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

# Single source of truth for "GitHub Actions workflow file" universe. Both
# `.yml` and `.yaml` are honored by GitHub Actions (per
# https://docs.github.com/actions). Every detector below uses this helper
# instead of inlining a glob, so a new file extension cannot create a
# fail-open extension gap.
workflow_files_nul() {
  git ls-files -z '.github/workflows/*.yml' '.github/workflows/*.yaml'
}

# Stray .yaml warning: today the repo has zero workflow .yaml files. If one
# appears in future, that is fine on its own — but the bigger picture, per
# §3 inventory + Slice 7 dissolution path, is that *all* workflow files
# (regardless of extension) feed both detectors below.

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
done < <(workflow_files_nul | xargs -0 grep -nH "git diff --name-only" 2>/dev/null || true)

# Independent anchor-host check: the §3 row #1 invariant says the inventoried
# diff anchor LIVES IN ci.yml. The earlier PATH_REGEX_FILTER grep at site #1
# pins a DIFFERENT literal (the docs-only allowlist), so a refactor that
# removed the `git diff --name-only origin/main...HEAD` line but kept the
# allowlist regex would slip past the earlier check. Enforce the anchor-host
# invariant explicitly here so cursor 10271's failure modes are both closed.
inventoried_present=0
if grep -qF "$INVENTORIED_DIFF_ANCHOR" "$CI_YML" 2>/dev/null; then
  inventoried_present=1
fi
if (( inventoried_present == 0 )); then
  fail "§3 row #1 anchor missing from $CI_YML:
        expected literal: $INVENTORIED_DIFF_ANCHOR
        per $INVENTORY_DOC §3 + §4 fail-closed contract, this anchor must
        live in ci.yml until the BinaryShim runner replacement (post–Slice 5)
        is wired. If you're landing the Slice 7 implementation PR, retire
        this script as part of the dissolution receipt — do not just remove
        the anchor."
fi

# Count invariant: actual_diff_count must equal expected_diff_count exactly.
# Both directions fail-closed (per §4 + INVARIANTS P3): overflow = candidate
# new bridge; underflow = inventoried bridge removed before replacement.
if (( actual_diff_count > expected_diff_count )); then
  if (( ${#extra_locations[@]} == 0 )); then
    # Every overflow match still contains the inventoried anchor verbatim —
    # i.e., a DUPLICATE copy of the anchored invocation. Treat as a new
    # bridge.
    fail "duplicate-anchor count overflow: found $actual_diff_count occurrences of
        'git diff --name-only' but expected $expected_diff_count. Every overflow
        line still matches the inventoried anchor verbatim, suggesting a copy
        of the §3 row #1 invocation was added (same literal, second site).
        Either route selection through the BinaryShim runner (post–Slice 5)
        or, if event-orthogonal, document in $INVENTORY_DOC §3 and bump
        'expected_diff_count' in this script."
  else
    for loc in "${extra_locations[@]}"; do
      fail "new path-regex selection candidate (un-inventoried 'git diff --name-only'): $loc
        If this is genuinely orthogonal to affected-set selection, add it to
        $INVENTORY_DOC §3 with rationale and bump 'expected_diff_count' in
        this script. Otherwise route selection through the BinaryShim runner
        (post–Slice 5)."
    done
  fi
elif (( actual_diff_count < expected_diff_count )); then
  fail "count underflow: found $actual_diff_count occurrence(s) of
        'git diff --name-only' across all workflow files but expected
        $expected_diff_count (the §3 row #1 inventoried anchor). The
        inventoried bridge appears to have been removed before BinaryShim
        replacement is wired. Per $INVENTORY_DOC §4 fail-closed contract,
        do not just delete the anchor — retire this script as part of the
        Slice 7 dissolution receipt."
fi

# ---- Drift detector: OTHER path-regex selection mechanisms -----------------
# A second class of authoritative path-regex selection that does NOT route
# through `git diff --name-only`:
#
#   * Trigger-level `paths:` / `paths-ignore:` filters on `on: push:` /
#     `on: pull_request:` (skip the workflow when no matching files change).
#   * `dorny/paths-filter` action usage (popular changed-files filter action).
#   * `tj-actions/changed-files` action usage (alternative changed-files
#     action).
#   * The literal substring `paths-filter` used in step uses-clauses.
#
# Baseline today (`.github/workflows/*.yml`): zero occurrences of any of
# these. The Slice 7 dissolution surface is meant to consume the PR #2713
# affected-set lens via the BinaryShim runner — not to grow a second
# parallel selection authority. Any introduction → fail.
#
# If a legitimate event-orthogonal use of `paths:` is needed in future,
# document it in $INVENTORY_DOC §3 and add the specific file:line to an
# explicit allowlist here (mirroring the `expected_diff_count` pattern
# above). The default stance is fail-closed.
# Trigger-level `paths:` / `paths-ignore:` detector (awk state-machine).
# Only flags `paths:` / `paths-ignore:` keys that appear UNDER
# `on:` → `push:` / `pull_request:` / `pull_request_target:` (the only YAML
# locations where these keys carry authoritative path-regex selection
# semantics). Step-input `paths:` keys (e.g., `with:` blocks under
# `dorny/paths-filter`) are intentionally NOT flagged here — they are step
# inputs, not workflow-trigger filters; the dorny action itself is caught by
# the separate action-name detector below.
#
# This precise scoping resolves the doc-vs-impl mismatch raised by cursor
# review 10280 (P2 / Practice 5 single-authority alignment).
trigger_paths_awk='
function indent_of(s,   l) { l = match(s, /[^ ]/); return l > 0 ? l - 1 : -1 }
{
  raw = $0
  if (raw ~ /^[[:space:]]*$/ || raw ~ /^[[:space:]]*#/) next
  ind = indent_of(raw)
  content = raw; sub(/^[[:space:]]+/, "", content)
  if (ind == 0) {
    in_on  = (content ~ /^on:/) ? 1 : 0
    in_trig = 0
    next
  }
  if (!in_on) next
  if (!in_trig) {
    if (content ~ /^(push|pull_request|pull_request_target):/) {
      in_trig = 1; trig_indent = ind
    }
    next
  }
  if (ind <= trig_indent) {
    if (content ~ /^(push|pull_request|pull_request_target):/) {
      in_trig = 1; trig_indent = ind
    } else {
      in_trig = 0
    }
    next
  }
  if (content ~ /^paths(-ignore)?:/) {
    printf("%s:%d:%s\n", FILENAME, NR, raw)
  }
}'

while IFS= read -r match; do
  [[ -z "$match" ]] && continue
  fail "new trigger-level paths:/paths-ignore: selection candidate: $match
        Authoritative path-regex selection under on:push / on:pull_request /
        on:pull_request_target. Either dissolve via the BinaryShim runner
        (post–Slice 5) or, if event-orthogonal, document in $INVENTORY_DOC §3
        and add an explicit allowlist entry to this script."
done < <(
  while IFS= read -r -d '' f; do
    awk "$trigger_paths_awk" "$f"
  done < <(workflow_files_nul)
)

# Action-name detector for non-trigger-level non-diff path-regex selection:
# popular changed-files action wrappers that read changed paths and emit
# outputs which downstream jobs gate on. Uses-clause substrings are a tight
# fingerprint with no plausible false positives.
while IFS= read -r match; do
  [[ -z "$match" ]] && continue
  fail "new changed-files action use detected: $match
        Matches one of: dorny/paths-filter, tj-actions/changed-files, or
        a 'paths-filter@' uses-clause substring. Either dissolve via the
        BinaryShim runner (post–Slice 5) or, if event-orthogonal, document
        in $INVENTORY_DOC §3 and add an explicit allowlist entry to this
        script."
done < <(
  workflow_files_nul \
    | xargs -0 grep -nHE "dorny/paths-filter|tj-actions/changed-files|paths-filter@" \
        2>/dev/null || true
)

if (( violations > 0 )); then
  note "$violations violation(s) — see messages above."
  exit 1
fi

echo "check-workflow-path-regex-inventory: ok (2/2 inventoried sites present, no new bridges)"
