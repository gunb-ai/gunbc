#!/usr/bin/env bash
# Fail if __BUG_NO_PROFILE_ is reintroduced into tracked *.rs / *.dag sources (P0-C).
# Docs and markdown are not scanned — only compiler-facing sources.
#
# Usage: ./scripts/check-fabrication-sentinels.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

violations=0
while IFS= read -r f; do
  [[ "$f" =~ ^docs/ ]] && continue
  if grep -q '__BUG_NO_PROFILE_' "$f" 2>/dev/null; then
    echo "error: __BUG_NO_PROFILE_ found in $f" >&2
    violations=$((violations + 1))
  fi
done < <(git ls-files '*.rs' '*.dag')

if (( violations > 0 )); then
  echo "check-fabrication-sentinels: failed ($violations file(s))" >&2
  exit 1
fi

echo "check-fabrication-sentinels: ok"
