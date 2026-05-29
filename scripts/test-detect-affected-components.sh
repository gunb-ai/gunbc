#!/usr/bin/env bash
# Self-test for scripts/detect-affected-components.sh path buckets.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
DETECT="$ROOT/scripts/detect-affected-components.sh"

run_case() {
  local name="$1" changed="$2" expect="$3"
  local out
  out=$(DETECT_AFFECTED_CHANGED_OVERRIDE="$changed" bash "$DETECT" pull_request /dev/stdout)
  if [ "$out" != "$expect" ]; then
    echo "::error::case $name: expected:" >&2
    echo "$expect" | sed 's/^/  /' >&2
    echo "got:" >&2
    echo "$out" | sed 's/^/  /' >&2
    return 1
  fi
}

failed=0

# Docs-only PR: no compiler or discipline buckets (except false across board).
run_case docs_only $'docs/readme-note.md\n' \
  'v2=false
v3=false
v4=false
workflow_policy=false
ci_sg0=false
ci_r4_carve=true
ci_fabrication=false
ci_release_doc=false
ci_manager_brief=false
ci_test_timeout=false
ci_rust_toolchain=false
ci_t19=false
ci_fmt=false' || failed=1

# Rust source touches fabrication + fmt; not v3 freeze paths unless under src/v3.
run_case rust_only $'src/v2/stage0/src/lib.rs\n' \
  'v2=true
v3=false
v4=false
workflow_policy=false
ci_sg0=false
ci_r4_carve=false
ci_fabrication=true
ci_release_doc=false
ci_manager_brief=false
ci_test_timeout=false
ci_rust_toolchain=false
ci_t19=false
ci_fmt=true' || failed=1

# Workflow edit forces workflow_policy and several discipline buckets.
run_case workflow_edit $'.github/workflows/ci.yml\n' \
  'v2=false
v3=false
v4=false
workflow_policy=true
ci_sg0=true
ci_r4_carve=false
ci_fabrication=false
ci_release_doc=false
ci_manager_brief=false
ci_test_timeout=false
ci_rust_toolchain=true
ci_t19=false
ci_fmt=false' || failed=1

if [ "$failed" -ne 0 ]; then
  echo "test-detect-affected-components.sh: FAILED" >&2
  exit 1
fi
echo "test-detect-affected-components.sh: ok"
