#!/usr/bin/env bash
# Banked-dissolutions ratchet (Lane 1 Stage 1a).
#
# Scans lane and phase docs for shapes that the design-blocker set
# (DB-1…DB-10) has formally rejected. These rejected names become a
# forbidden-string list — the master authority is
# docs/post-l15-phase-plan.md § "Banked dissolutions — rejected shapes".
#
# Exempt from the scan:
#   docs/design-*.md         — DB docs record the rejections
#   docs/post-l15-phase-plan.md — master plan carries the ratchet itself
#
# Usage: scripts/check-banked-dissolutions.sh
#   Exits 0 when clean; exits 1 with a report on any match.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

FORBIDDEN=(
  "port_by_id"
  "node_by_id"
  "RestTransport"
  "ShellTransport"
  "GrpcTransport"
  "TransportKind"
  "target_language: TargetLanguageId"
  "StructFieldRule"
  "AllowAttributeOnStructDecl"
  "MutualLoop"
)

# Lane/phase docs only. DB docs and the master plan are exempt because
# they legitimately enumerate the rejected names.
shopt -s nullglob
FILES=()
for pattern in docs/lane*.md docs/phase*.md; do
  for file in $pattern; do
    case "$file" in
      docs/post-l15-phase-plan.md) ;;
      docs/design-*.md) ;;
      *) FILES+=("$file") ;;
    esac
  done
done
shopt -u nullglob

if [ ${#FILES[@]} -eq 0 ]; then
  echo "banked-dissolutions: no lane/phase docs to scan"
  exit 0
fi

violations=0
for pat in "${FORBIDDEN[@]}"; do
  hits=$(grep -nHF -- "$pat" "${FILES[@]}" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    if [ "$violations" -eq 0 ]; then
      echo "banked-dissolutions ratchet: forbidden shapes found in lane/phase docs."
      echo "Authority: docs/post-l15-phase-plan.md § Banked dissolutions."
      echo
    fi
    echo "--- forbidden: $pat ---"
    echo "$hits"
    echo
    violations=$((violations + 1))
  fi
done

if [ "$violations" -gt 0 ]; then
  echo "banked-dissolutions ratchet: $violations forbidden shape(s) found."
  echo "Fix: delete the restatement and reference the DB doc instead."
  exit 1
fi

echo "banked-dissolutions ratchet: clean (${#FILES[@]} docs scanned)"
