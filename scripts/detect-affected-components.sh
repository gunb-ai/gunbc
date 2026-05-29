#!/usr/bin/env bash
# scripts/detect-affected-components.sh
#
# Detects which v* components are affected by the current PR / push so CI
# can gate heavy v2 / v3 jobs accordingly (frozen-buildable discipline).
#
# Usage:
#   scripts/detect-affected-components.sh <event_name> <output_file>
#   scripts/detect-affected-components.sh --self-test
#
# event_name:  "pull_request" | "push"
# output_file: a file to write GitHub Actions outputs into (one per line:
#              key=value). Pass "$GITHUB_OUTPUT" from the workflow.
#
# Writes:
#   v2=true|false  — true if src/v2/ or workspace deps (Cargo.toml/lock) changed
#   v3=true|false  — true if src/v3/ or dsl/ changed
#   v4=true|false  — true if src/v4/ or workspace deps / v4 gate scripts changed
#   workflow_policy=true|false — Gate #103 workflow surface (ci.yml, this script, …)
#   ci_sg0, ci_r4_carve, ci_fabrication, ci_release_doc, ci_manager_brief,
#   ci_test_timeout, ci_rust_toolchain, ci_t19, ci_fmt — path buckets for the
#   `ci` / `fmt` jobs' shell-discipline steps (Gate #103: selection lives here,
#   not in workflow YAML path-regex).
#
# Self-test: DETECT_AFFECTED_CHANGED_OVERRIDE='<one path per line>' \
#   scripts/detect-affected-components.sh pull_request /dev/stdout
#
# Why this lives in a script (not inline in ci.yml):
# Gate #103 (`ci_uses_affected_set_selection`) policy forbids path-selection
# substrings in workflow YAML — see scripts/workflow-path-regex-forbidden-
# substrings.txt.

set -euo pipefail

if [ "${1:-}" = "--self-test" ]; then
  exec bash "$(cd "$(dirname "$0")" && pwd)/test-detect-affected-components.sh"
fi

event_name="${1:-pull_request}"
output_file="${2:-/dev/stdout}"

if [ "$event_name" = "pull_request" ]; then
  diff_range="origin/main...HEAD"
else
  diff_range="HEAD~1..HEAD"
fi

if [ -n "${DETECT_AFFECTED_CHANGED_OVERRIDE:-}" ]; then
  changed="$DETECT_AFFECTED_CHANGED_OVERRIDE"
else
  changed=$(git diff --name-only "$diff_range" 2>/dev/null || true)
fi

echo "Changed files in $diff_range:" >&2
if [ -n "$changed" ]; then
  echo "$changed" | sed 's/^/  /' >&2
else
  echo "  (none detected)" >&2
fi
echo "" >&2

changed_matches() {
  echo "$changed" | grep -qE "$1"
}

# Any tracked *.rs / *.dag outside docs/ (consumer: check-fabrication-sentinels.sh).
fabrication_affected() {
  local f
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    [[ "$f" == docs/* ]] && continue
    case "$f" in
      *.rs | *.dag) return 0 ;;
    esac
  done <<<"$changed"
  changed_matches '^scripts/check-fabrication-sentinels\.sh$'
}

# .rs outside docs/, or workspace toolchain manifests (consumer: fmt job).
fmt_affected() {
  local f
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    [[ "$f" == docs/* ]] && continue
    case "$f" in
      *.rs) return 0 ;;
    esac
  done <<<"$changed"
  changed_matches '^rust-toolchain\.toml$|Cargo\.(toml|lock)$'
}

log_bucket() {
  local val="$1" yes_msg="$2" no_msg="$3"
  if [ "$val" = true ]; then
    echo "$yes_msg" >&2
  else
    echo "$no_msg" >&2
  fi
}

# v2 affected: src/v2/ touched, OR workspace deps changed.
if changed_matches '^src/v2/|^Cargo\.(toml|lock)$'; then
  v2_state=true
  echo "v2 affected: yes" >&2
else
  v2_state=false
  echo "v2 affected: no (skipping v2 fixed-point)" >&2
fi

# v3 affected: src/v3/ OR dsl/ ONLY.
if changed_matches '^src/v3/|^dsl/'; then
  v3_state=true
  echo "v3 affected: yes" >&2
else
  v3_state=false
  echo "v3 affected: no (skipping v3 CI per freeze 2026-05-15)" >&2
fi

# v4 affected.
if changed_matches '^src/v4/|^fixtures/v4-mvp1/|^scripts/v4-mvp1|^scripts/v4-m1|^scripts/v4-testclaim-|^dsl/std/|^Cargo\.(toml|lock)$'; then
  v4_state=true
  echo "v4 affected: yes (running v2→v4 bootstrap viability test)" >&2
else
  v4_state=false
  echo "v4 affected: no (skipping v4 bootstrap test)" >&2
fi

# Gate #103 workflow / policy surface.
if changed_matches '^\.github/workflows/|^scripts/detect-affected-components\.sh|^scripts/test-detect-affected-components\.sh|^scripts/check-workflow-path-regex-inventory\.sh|^scripts/workflow-path-regex-forbidden-substrings\.txt'; then
  workflow_policy_state=true
  echo "workflow_policy (Gate #103 surface): yes" >&2
else
  workflow_policy_state=false
  echo "workflow_policy (Gate #103 surface): no" >&2
fi

# --- ci job discipline path buckets (orthogonal to v2/v3/v4) ---

if changed_matches '^src/v3/compiler/tests/integration/sg0_census_test\.rs$|^scripts/check-pr-sg0-net-shrink-discipline\.sh$|^scripts/ci-merge/sg0-|^\.github/(workflows/ci\.yml|PULL_REQUEST_TEMPLATE\.md)$'; then
  ci_sg0_state=true
else
  ci_sg0_state=false
fi
log_bucket "$ci_sg0_state" \
  "ci_sg0 bucket: yes" \
  "ci_sg0 bucket: no (skipping SG-0 PR-body discipline)"

if changed_matches '^docs/|^scripts/check-r4-carve-dissolution-discipline\.sh$'; then
  ci_r4_carve_state=true
else
  ci_r4_carve_state=false
fi
log_bucket "$ci_r4_carve_state" \
  "ci_r4_carve bucket: yes" \
  "ci_r4_carve bucket: no (skipping R4-carve dissolution discipline)"

if fabrication_affected; then
  ci_fabrication_state=true
else
  ci_fabrication_state=false
fi
log_bucket "$ci_fabrication_state" \
  "ci_fabrication bucket: yes" \
  "ci_fabrication bucket: no (skipping fabrication sentinel ratchet)"

# Keep in sync with RELEASE_DOCS in scripts/check-release-doc-authority.sh.
if changed_matches '^docs/r2-structure\.md$|^docs/r3-structure\.md$|^docs/thesis/r2-r3-thesis-mapping\.md$|^scripts/check-release-doc-authority\.sh$|^scripts/test-check-release-doc-authority\.sh$'; then
  ci_release_doc_state=true
else
  ci_release_doc_state=false
fi
log_bucket "$ci_release_doc_state" \
  "ci_release_doc bucket: yes" \
  "ci_release_doc bucket: no (skipping release-doc authority)"

if changed_matches '^docs/briefs/|^docs/r2-structure\.md$|^scripts/check-manager-brief-authority\.sh$|^scripts/test-check-manager-brief-authority\.sh$'; then
  ci_manager_brief_state=true
else
  ci_manager_brief_state=false
fi
log_bucket "$ci_manager_brief_state" \
  "ci_manager_brief bucket: yes" \
  "ci_manager_brief bucket: no (skipping manager-brief authority)"

if changed_matches '^scripts/check-test-timeout\.sh$|^scripts/test-check-test-timeout\.sh$|^dsl/gunbc/test_node_wall_clock_ratchet\.dag$'; then
  ci_test_timeout_state=true
else
  ci_test_timeout_state=false
fi
log_bucket "$ci_test_timeout_state" \
  "ci_test_timeout bucket: yes" \
  "ci_test_timeout bucket: no (skipping test-timeout ratchet self-test)"

if changed_matches '^rust-toolchain\.toml$|^dsl/extdeps/rustup\.dag$|^\.github/workflows/|^scripts/check-rust-toolchain-single-authority\.sh$'; then
  ci_rust_toolchain_state=true
else
  ci_rust_toolchain_state=false
fi
log_bucket "$ci_rust_toolchain_state" \
  "ci_rust_toolchain bucket: yes" \
  "ci_rust_toolchain bucket: no (skipping rust-toolchain single-authority)"

if changed_matches '^src/v4/(lens/testgen|std/effects|std/verification|test/claim/)|^scripts/check_t19_testgen_activation\.py$|^scripts/test_check_t19_testgen_activation\.py$'; then
  ci_t19_state=true
else
  ci_t19_state=false
fi
log_bucket "$ci_t19_state" \
  "ci_t19 bucket: yes" \
  "ci_t19 bucket: no (skipping T-19 testgen self-test)"

if fmt_affected; then
  ci_fmt_state=true
else
  ci_fmt_state=false
fi
log_bucket "$ci_fmt_state" \
  "ci_fmt bucket: yes" \
  "ci_fmt bucket: no (skipping cargo fmt job)"

{
  echo "v2=$v2_state"
  echo "v3=$v3_state"
  echo "v4=$v4_state"
  echo "workflow_policy=$workflow_policy_state"
  echo "ci_sg0=$ci_sg0_state"
  echo "ci_r4_carve=$ci_r4_carve_state"
  echo "ci_fabrication=$ci_fabrication_state"
  echo "ci_release_doc=$ci_release_doc_state"
  echo "ci_manager_brief=$ci_manager_brief_state"
  echo "ci_test_timeout=$ci_test_timeout_state"
  echo "ci_rust_toolchain=$ci_rust_toolchain_state"
  echo "ci_t19=$ci_t19_state"
  echo "ci_fmt=$ci_fmt_state"
} >>"$output_file"
