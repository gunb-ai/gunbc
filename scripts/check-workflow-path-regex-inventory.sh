#!/usr/bin/env bash
# T-WAD Slice 7 — post-dissolution ratchet: authoritative path-regex selection must
# NOT reappear in `.github/workflows/*.{yml,yaml}` after gate
# `ci_uses_affected_set_selection` (program row 103) removed the Layer-2 bridge.
#
# Fail-closed: forbidden fingerprints → non-zero exit.
# Pre-dissolution behavior (require inventoried bridges) lived in git history;
# see `docs/design-t-wad-slice-7-implementation-prequeue.md`.
#
# Usage: scripts/check-workflow-path-regex-inventory.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

violations=0
note() { echo "check-workflow-path-regex-inventory: $*" >&2; }
fail() { note "FAIL: $*"; violations=$((violations + 1)); }

workflow_files_nul() {
  git ls-files -z '.github/workflows/*.yml' '.github/workflows/*.yaml'
}

DOCS_ONLY_FILTER="grep -vE '^(docs/.*|[^/]+\.md)$'"

while IFS= read -r -d '' f; do
  if m=$(grep -nF "git diff --name-only" "$f" 2>/dev/null || true); [[ -n "$m" ]]; then
    fail "forbidden 'git diff --name-only' in $f — BinaryShim + lens receipts are authoritative:\n$m"
  fi
  if m=$(grep -nE "needs\.changes\.outputs" "$f" 2>/dev/null || true); [[ -n "$m" ]]; then
    fail "forbidden needs.changes.outputs bridge in $f:\n$m"
  fi
  if m=$(grep -nF "$DOCS_ONLY_FILTER" "$f" 2>/dev/null || true); [[ -n "$m" ]]; then
    fail "forbidden docs-only path-regex filter in $f:\n$m"
  fi
done < <(workflow_files_nul)

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
  fail "trigger-level paths:/paths-ignore: selection candidate: $match"
done < <(
  while IFS= read -r -d '' f; do
    awk "$trigger_paths_awk" "$f"
  done < <(workflow_files_nul)
)

while IFS= read -r match; do
  [[ -z "$match" ]] && continue
  fail "changed-files action wrapper candidate: $match"
done < <(
  workflow_files_nul \
    | xargs -0 grep -nHE "dorny/paths-filter|tj-actions/changed-files|paths-filter@" \
        2>/dev/null || true
)

if (( violations > 0 )); then
  note "$violations violation(s) — see messages above."
  exit 1
fi

echo "check-workflow-path-regex-inventory: ok (no authoritative path-regex selection in workflows)"
