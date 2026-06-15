#!/usr/bin/env bash
# scripts/v4-layering-imports-discover.sh
#
# Host transport for layering-imports gate. Enumerates import facts from std/ and
# extdeps/ layer roots into an ephemeral never-committed manifest consumed by
# modeled witnesses in layering_imports/clean_tree.dag.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

scan_bin="${LAYERING_IMPORTS_SCAN:-target/release/layering_imports_scan}"
scan_repo_root="${LAYERING_IMPORTS_REPO_ROOT:-$root}"
manifest="${V4_LAYERING_IMPORTS_MANIFEST:-target/v4-layering-imports-manifest.dag}"

if [[ ! -x "$scan_bin" ]]; then
  echo "error: layering_imports_scan not found at $scan_bin" >&2
  exit 2
fi

"$scan_bin" \
  --repo-root "$scan_repo_root" \
  --emit-dag-manifest "$manifest"

if git ls-files --error-unmatch "$manifest" >/dev/null 2>&1; then
  echo "error: ephemeral layering-imports manifest is tracked in git: $manifest" >&2
  exit 2
fi

printf '%s' "$manifest"
