#!/usr/bin/env bash
# public-clone-smoke.sh — post-export defense-in-depth check for the public
# snapshot. Clones gunb-ai/daglang, re-runs the leak-grep patterns against
# the actually-shipped tree, builds gunbc from the public sources, and
# asserts the keep/strip inventory matches expectation.
#
# Lives under _internal/scripts/ so publish-snapshot.sh's STRIP_PATHS removes
# it from the public snapshot — this script must never ship publicly.
#
# Usage:
#   _internal/scripts/public-clone-smoke.sh
#
# Env:
#   PUBLIC_REPO_URL  default: git@github.com:gunb-ai/daglang.git
#   PUBLIC_BRANCH    default: main
set -euo pipefail

PUBLIC_REPO_URL="${PUBLIC_REPO_URL:-git@github.com:gunb-ai/daglang.git}"
PUBLIC_BRANCH="${PUBLIC_BRANCH:-main}"

# Allowlist: dissolve-comment substrate provenance is operator-approved to
# ship publicly. Keep in sync with publish-snapshot.sh.
ALLOWLIST_REGEX='🟡|dissolve-target|dissolve-on-arrival'

CONTENT_PATTERNS=(
  'msg_[a-f0-9-]+'
  'localhost:8787'
  'dashboard-ops'
  'dashboard-message'
  'operator-[a-z]+'
)

PATH_PATTERNS=(
  '_internal/'
  'docs/briefs/'
  'docs/audit/'
  'scripts/session-dashboard/'
  '\.cursor/'
)

EXPECTED_PRESENT=(
  'README.md'
  'dsl'
  'src/v2'
  'scripts/install-hooks.sh'
  'scripts/publish-snapshot.sh'
)

EXPECTED_ABSENT=(
  'docs/briefs'
  'docs/audit'
  'src/v3'
  'scripts/session-dashboard'
  '_internal'
)

CLONE_DIR="$(mktemp -d -t gunbc-public-smoke-XXXXXX)"
trap 'rm -rf "$CLONE_DIR"' EXIT

echo "[smoke] cloning ${PUBLIC_REPO_URL} (branch ${PUBLIC_BRANCH}) into ${CLONE_DIR}"
git clone --depth 1 --branch "$PUBLIC_BRANCH" "$PUBLIC_REPO_URL" "$CLONE_DIR"

CLONE_SHA="$(git -C "$CLONE_DIR" rev-parse HEAD)"
fail=0

echo "[smoke] leak-grep content patterns..."
for pat in "${CONTENT_PATTERNS[@]}"; do
  hits="$(git -C "$CLONE_DIR" grep -E -n -e "$pat" 2>/dev/null || true)"
  if [[ -n "$hits" ]]; then
    real_hits="$(echo "$hits" | grep -E -v "$ALLOWLIST_REGEX" || true)"
    if [[ -n "$real_hits" ]]; then
      echo "[smoke] LEAK: content /$pat/ (after allowlist):" >&2
      echo "$real_hits" | head -20 >&2
      fail=1
    fi
  fi
done

echo "[smoke] leak-grep path patterns..."
for glob in "${PATH_PATTERNS[@]}"; do
  hits="$(git -C "$CLONE_DIR" ls-files | grep -E -- "$glob" || true)"
  if [[ -n "$hits" ]]; then
    echo "[smoke] LEAK: path /${glob}/ (strip-list missed it):" >&2
    echo "$hits" | head -20 >&2
    fail=1
  fi
done

echo "[smoke] expected-paths-present..."
for p in "${EXPECTED_PRESENT[@]}"; do
  if [[ ! -e "${CLONE_DIR}/${p}" ]]; then
    echo "[smoke] MISSING expected: ${p}" >&2
    fail=1
  fi
done

echo "[smoke] expected-paths-absent..."
for p in "${EXPECTED_ABSENT[@]}"; do
  if [[ -e "${CLONE_DIR}/${p}" ]]; then
    echo "[smoke] PRESENT but should be stripped: ${p}" >&2
    fail=1
  fi
done

echo "[smoke] cargo build --release -p v2-compiler --bin gunbc..."
if ! ( cd "$CLONE_DIR" && cargo build --release -p v2-compiler --bin gunbc ); then
  echo "[smoke] BUILD FAILED" >&2
  fail=1
fi

if [[ -x "${CLONE_DIR}/target/release/gunbc" ]]; then
  echo "[smoke] ./target/release/gunbc --help..."
  if ! "${CLONE_DIR}/target/release/gunbc" --help >/dev/null 2>&1; then
    echo "[smoke] gunbc --help FAILED" >&2
    fail=1
  fi
else
  echo "[smoke] gunbc binary not produced by build" >&2
  fail=1
fi

echo
echo "==== public-clone-smoke receipt ===="
echo "remote:         ${PUBLIC_REPO_URL}#${PUBLIC_BRANCH}"
echo "clone_sha:      ${CLONE_SHA}"
echo "timestamp_utc:  $(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [[ "$fail" -eq 0 ]]; then
  echo "result:         PASS"
  exit 0
else
  echo "result:         FAIL"
  exit 1
fi
