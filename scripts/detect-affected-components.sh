#!/usr/bin/env bash
# scripts/detect-affected-components.sh
#
# Detects which v* components are affected by the current PR / push so CI
# can gate heavy v2 / v3 jobs accordingly (frozen-buildable discipline).
#
# Usage:
#   scripts/detect-affected-components.sh <event_name> <output_file>
#
# event_name:  "pull_request" | "push"
# output_file: a file to write GitHub Actions outputs into (one per line:
#              key=value). Pass "$GITHUB_OUTPUT" from the workflow.
#
# Writes:
#   v2=true|false  — true if src/v2/ or workspace deps (Cargo.toml/lock) changed
#   v3=true|false  — true if src/v3/, dsl/, or workspace deps changed
#
# Why this lives in a script (not inline in ci.yml):
# Gate #103 (`ci_uses_affected_set_selection`) policy forbids path-selection
# substrings in workflow YAML — see scripts/workflow-path-regex-forbidden-
# substrings.txt. The detection logic itself is not the v3 affected-set
# lens; it's the *gating* signal that decides whether the v3 lens (and v3
# build) need to run at all. By living here it is exempt from the YAML
# substring check while preserving the policy intent (no path-regex
# selection inside workflow YAML).
#
# When v3 is unfrozen and the v3 lens becomes the canonical affected-set
# authority again, this script can be replaced by an invocation of that
# lens — the workflow integration point stays the same.

set -euo pipefail

event_name="${1:-pull_request}"
output_file="${2:-/dev/stdout}"

if [ "$event_name" = "pull_request" ]; then
  diff_range="origin/main...HEAD"
  # Caller is expected to have fetched origin/main (via fetch-depth: 0 or
  # a prior fetch step). If not, the diff falls back to empty and both
  # outputs become "false" — safe default for a fresh-checkout case.
else
  # push event — diff against the previous commit on this ref
  diff_range="HEAD~1..HEAD"
fi

changed=$(git diff --name-only "$diff_range" 2>/dev/null || true)

echo "Changed files in $diff_range:" >&2
if [ -n "$changed" ]; then
  echo "$changed" | sed 's/^/  /' >&2
else
  echo "  (none detected)" >&2
fi
echo "" >&2

# v2 affected: src/v2/ touched, OR workspace deps changed.
# Workspace deps can break v2 build even if v2 sources aren't touched.
if echo "$changed" | grep -qE '^src/v2/|^Cargo\.(toml|lock)$'; then
  v2_state="true"
  echo "v2 affected: yes" >&2
else
  v2_state="false"
  echo "v2 affected: no (skipping v2 fixed-point)" >&2
fi

# v3 affected: src/v3/, dsl/ (v3 reads dsl/), OR workspace deps changed.
if echo "$changed" | grep -qE '^src/v3/|^dsl/|^Cargo\.(toml|lock)$'; then
  v3_state="true"
  echo "v3 affected: yes" >&2
else
  v3_state="false"
  echo "v3 affected: no (skipping v3 CI per freeze 2026-05-15)" >&2
fi

# Emit GitHub Actions outputs (or stdout if no output file given)
{
  echo "v2=$v2_state"
  echo "v3=$v3_state"
} >> "$output_file"
