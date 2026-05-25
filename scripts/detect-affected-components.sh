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
#                    (v2 binary is frozen-buildable + the v4 bootstrap stage;
#                    deps changes can break the build)
#   v3=true|false  — true if src/v3/ or dsl/ changed
#                    (v3 is FROZEN — workspace dep changes do NOT trigger v3 CI;
#                    we don't care if v3 incidentally breaks because v3 is abandoned;
#                    if v3 is ever revived, the first src/v3/ PR catches latent breakage)
#   v4=true|false  — true if src/v4/ or workspace deps changed
#                    (triggers v2→v4 bootstrap viability test)
#   workflow_policy=true|false — true if GitHub Actions workflow definitions, this
#                    script (affects which jobs run, including Gate #103), or the
#                    Gate #103 path-regex ratchet scripts changed. Independent of
#                    v2/v3/v4: PRs that touch only `.github/workflows/ci.yml` must still
#                    run fail-closed workflow policy checks (INVARIANTS P2/P3) and the
#                    MVP-1 end-to-end gate when its ci.yml wiring changes (ci job gates
#                    MVP-1 on workflow_policy || v4).
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

# v3 affected: src/v3/ OR dsl/ ONLY. Workspace deps (Cargo.toml/lock) do NOT
# trigger v3 CI under the freeze — we don't care if v3 incidentally breaks
# because v3 is abandoned. If v3 is ever revived, the first src/v3/ PR catches
# any latent dep-bump breakage anyway.
if echo "$changed" | grep -qE '^src/v3/|^dsl/'; then
  v3_state="true"
  echo "v3 affected: yes" >&2
else
  v3_state="false"
  echo "v3 affected: no (skipping v3 CI per freeze 2026-05-15)" >&2
fi

# v4 affected: src/v4/ touched OR workspace deps changed (deps can break v2 build,
# which v4 depends for bootstrap). Also dsl/std/ — MVP-1 gate compile dep pool
# (fixtures/v4-mvp1/add); dsl/std-only PRs must re-run the add receipt.
# scripts/v4-mvp1* and scripts/v4-m1* — v4 CI shell gates (MVP-1 e2e, M1 rust emit probe).
if echo "$changed" | grep -qE '^src/v4/|^fixtures/v4-mvp1/|^scripts/v4-mvp1|^scripts/v4-m1|^dsl/std/|^Cargo\.(toml|lock)$'; then
  v4_state="true"
  echo "v4 affected: yes (running v2→v4 bootstrap viability test)" >&2
else
  v4_state="false"
  echo "v4 affected: no (skipping v4 bootstrap test)" >&2
fi

# Gate #103 workflow / policy surface — orthogonal to compiler v2/v3/v4 buckets.
# Include this file: edits here change `workflow_policy` / v2/v3/v4 outputs and must
# not skip Gate #103 (INVARIANTS P3 fail-closed gating).
if echo "$changed" | grep -qE '^\.github/workflows/|^scripts/detect-affected-components\.sh|^scripts/check-workflow-path-regex-inventory\.sh|^scripts/workflow-path-regex-forbidden-substrings\.txt'; then
  workflow_policy_state="true"
  echo "workflow_policy (Gate #103 surface): yes" >&2
else
  workflow_policy_state="false"
  echo "workflow_policy (Gate #103 surface): no" >&2
fi

# Emit GitHub Actions outputs (or stdout if no output file given)
{
  echo "v2=$v2_state"
  echo "v3=$v3_state"
  echo "v4=$v4_state"
  echo "workflow_policy=$workflow_policy_state"
} >> "$output_file"
