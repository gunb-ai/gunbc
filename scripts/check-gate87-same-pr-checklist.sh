#!/usr/bin/env bash
set -euo pipefail

# Gate #87 Band-C same-PR checklist ratchet.
#
# If a PR touches the lens capability/register/cementing surfaces where a
# BEHAVIORALLY COMPLETE flip or regen registry change can happen, require the PR
# body to make the TESTING.md Band-C same-PR checklist disposition explicit.

usage() {
  cat <<'USAGE'
usage:
  bash scripts/check-gate87-same-pr-checklist.sh [--self-test]

On pull_request CI, set PR_BODY to the GitHub PR description, PR_BASE_SHA to the
pull request base commit, and PR_HEAD_SHA to the pull request head commit. The
script diffs that explicit PR range and requires a column-0 line:

  Gate-87 same-PR checklist: complete - <lens/receipt summary>
  Gate-87 same-PR checklist: n/a - <why this touched surface is not a COMPLETE flip>
USAGE
}

TRIGGER_PATHS=(
  ".github/PULL_REQUEST_TEMPLATE.md"
  "TESTING.md"
  "docs/v3-lens-capability-register.md"
  "src/v3/std/verification.dag"
  "src/v3/compiler/regen.dag"
  "src/v3/compiler/tests/dag/cementing_dispatch.dag"
  "src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs"
  "src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs"
)

changed_files() {
  if [[ -n "${GATE87_CHANGED_FILES:-}" ]]; then
    printf '%s\n' "${GATE87_CHANGED_FILES}"
    return 0
  fi

  local base=${PR_BASE_SHA:-}
  local head=${PR_HEAD_SHA:-HEAD}
  if [[ -z "$base" ]]; then
    echo "::error::PR_BASE_SHA is required so the Gate-87 same-PR ratchet diffs against the actual pull request base." >&2
    return 1
  fi
  git diff --name-only "${base}...${head}"
}

is_trigger_path() {
  local path=$1
  local trigger
  for trigger in "${TRIGGER_PATHS[@]}"; do
    if [[ "$path" == "$trigger" ]]; then
      return 0
    fi
  done
  [[ "$path" == src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag ]]
}

requires_checklist() {
  local path
  while IFS= read -r path; do
    if is_trigger_path "$path"; then
      return 0
    fi
  done
  return 1
}

validate_body() {
  local body=$1
  local line rest
  if ! grep -qE '^Gate-87 same-PR checklist:' <<<"$body"; then
    echo "::error::PR touches gate-87 lens/register/cementing surfaces but is missing a column-0 \`Gate-87 same-PR checklist:\` line. Use \`complete - ...\` or \`n/a - ...\` per .github/PULL_REQUEST_TEMPLATE.md." >&2
    return 1
  fi

  line=$(grep -E '^Gate-87 same-PR checklist:' <<<"$body" | head -1)
  rest=${line#Gate-87 same-PR checklist:}
  rest=${rest#"${rest%%[![:space:]]*}"}

  if [[ ! "$rest" =~ ^(complete|n/a)[[:space:]]+-[[:space:]]+[^[:space:]].* ]]; then
    echo "::error::Gate-87 same-PR checklist line must be \`complete - <summary>\` or \`n/a - <reason>\`; got: $line" >&2
    return 1
  fi
}

self_test() {
  validate_body $'Gate-87 same-PR checklist: complete - cost row flipped with DifferentialEquals receipt' >/dev/null
  validate_body $'Gate-87 same-PR checklist: n/a - docs-only clarification, no COMPLETE flip' >/dev/null

  if validate_body $'Gate-87 same-PR checklist: complete' >/dev/null 2>&1; then
    echo "expected incomplete checklist line to fail" >&2
    return 1
  fi
  if validate_body $'- Gate-87 same-PR checklist: complete - hidden behind bullet' >/dev/null 2>&1; then
    echo "expected non-column-0 checklist line to fail" >&2
    return 1
  fi

  GATE87_CHANGED_FILES=$'README.md\nsrc/v3/compiler/regen.dag'
  requires_checklist < <(changed_files)
  GATE87_CHANGED_FILES=$'README.md\ndocs/other.md'
  if requires_checklist < <(changed_files); then
    echo "expected non-trigger changed-file set to skip checklist" >&2
    return 1
  fi
  unset GATE87_CHANGED_FILES

  if (unset PR_BASE_SHA PR_HEAD_SHA GATE87_CHANGED_FILES; changed_files >/dev/null 2>&1); then
    echo "expected changed_files without PR_BASE_SHA to fail closed" >&2
    return 1
  fi

  local tmp base parent head range
  tmp=$(mktemp -d)
  (
    cd "$tmp"
    git init -q
    git config user.email gate87@example.invalid
    git config user.name gate87
    printf 'base\n' > README.md
    git add README.md
    git commit -q -m base
    base=$(git rev-parse HEAD)
    mkdir -p src/v3/compiler
    printf 'inherited\n' > src/v3/compiler/regen.dag
    git add src/v3/compiler/regen.dag
    git commit -q -m parent-trigger-change
    parent=$(git rev-parse HEAD)
    printf 'feature\n' >> README.md
    git add README.md
    git commit -q -m feature-change
    head=$(git rev-parse HEAD)
    range=$(PR_BASE_SHA="$parent" PR_HEAD_SHA="$head" changed_files)
    if [[ "$range" != "README.md" ]]; then
      echo "expected explicit PR base range to exclude inherited trigger change, got: $range" >&2
      return 1
    fi
  )
  rm -rf "$tmp"

  echo "check-gate87-same-pr-checklist.sh: self-test OK"
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ "${1:-}" == "--self-test" ]]; then
  self_test
  exit 0
fi

changed=$(changed_files)
if ! requires_checklist <<<"$changed"; then
  exit 0
fi

validate_body "${PR_BODY:-}"
