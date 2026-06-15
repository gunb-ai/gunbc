#!/usr/bin/env bash
# scripts/ci-lens-lane-required.sh — fail-closed lens-lane classifier for CI dispatch.
#
# Authority: markdown/docs-only diffs cannot affect .dag lens witnesses; path-filter
# them OUT of the v4_lens_* lane (fleet crisis 2026-06-15, bold-crane-680).
# Uses git diff-tree (not workflow path-regex) — see workflow_no_path_regex_policy_ci_yml.
#
# Writes GITHUB_OUTPUT key lens_lane_required=true|false when GITHUB_OUTPUT is set;
# otherwise prints lens_lane_required=... for eval.

set -euo pipefail

emit() {
  local value="$1"
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    echo "lens_lane_required=${value}" >>"$GITHUB_OUTPUT"
  else
    echo "lens_lane_required=${value}"
  fi
}

# Fail-closed: unknown event / missing base → require lens lane.
if [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]; then
  base="${GITHUB_EVENT_PULL_REQUEST_BASE_SHA:-}"
  head="${GITHUB_SHA:-}"
elif [[ "${GITHUB_EVENT_NAME:-}" == "push" ]]; then
  base="${GITHUB_EVENT_BEFORE:-}"
  head="${GITHUB_SHA:-}"
else
  emit true
  exit 0
fi

if [[ -z "$base" || -z "$head" || "$base" =~ ^0+$ ]]; then
  emit true
  exit 0
fi

mapfile -d '' files < <(git diff-tree -r --name-only --no-commit-id -z "$base" "$head" 2>/dev/null || true)

if [[ ${#files[@]} -eq 0 ]]; then
  emit false
  exit 0
fi

docs_only_path() {
  local f="$1"
  [[ "$f" == docs/* ]] && return 0
  [[ "$f" == *.md && "$f" != */* ]] && return 0
  return 1
}

for f in "${files[@]}"; do
  [[ -z "$f" ]] && continue
  # CI / lens transport edits can change enrollment or gate behavior.
  case "$f" in
    .github/workflows/* | scripts/v4-* | scripts/check_ci_* | src/v4/*)
      emit true
      exit 0
      ;;
  esac
  if ! docs_only_path "$f"; then
    emit true
    exit 0
  fi
done

emit false
