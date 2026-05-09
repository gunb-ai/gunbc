#!/usr/bin/env bash
# R4-carve dissolution discipline ratchet (2026-05-09 carve-promotion).
#
# Per Director carve-promotion-IN-R3 ratification at gunbc#846 #issuecomment-4412330468 +
# (a) at #issuecomment-4412380947 + operator framing 2026-05-09 ("0 hand-Rust including
# tests AND stage0; bootstrap is data + self-generated"): R4 carves C1/C2/C3 (gates
# #81/#82/#95) DISSOLVED; gates are R3-load-bearing within Cluster F (T-LP-Retirement).
#
# This ratchet detects citations of the DISSOLVED carves that lack proper supersession
# annotations + counts them as drift candidates. Fail-closed if a NEW citation lands on
# a PR without DISSOLVED/AMENDED annotation.
#
# Authority sources:
#   - docs/r4-carve-out-routing.md (C1/C2/C3 DISSOLVED status)
#   - docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md (Cluster F sequencing)
#   - docs/r3-program-plan.md §1.5 (96 R3-load-bearing canonical authority)
#   - THESIS.md "Pure Bootstrap to Zero" + INVARIANTS.md P5
#
# Usage:
#   scripts/check-r4-carve-dissolution-discipline.sh           # check current tree
#   scripts/check-r4-carve-dissolution-discipline.sh --self-test  # run self-test cases

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Anti-patterns: phrases that indicate stale R4-carve framing without supersession.
# Each pattern is checked in scope (docs/) and lines must EITHER:
#   (a) include "DISSOLVED" / "AMENDED" / "carve-promot" / "formerly" / "prior" markers
#   (b) be in r4-carve-out-routing.md itself (which carries the dissolution record)
#   (c) be in r3-design-schedule-2026-05-06.md preamble or audit historical sections
ANTI_PATTERNS=(
  'carved to R4'
  'carve to R4'
  'R4-carved'
  'R4 carved'
  'R4-CARVED'
  'CARVED to R4'
  'CARVED \(C[123]\)'
)

# Permitted-context markers: lines containing any of these are allowed to cite the
# anti-pattern (historical/dissolved record, OR supersession annotation, OR
# legitimate post-2026-05-09 carve channel via Class P / past-R3 / R4+ partition
# per Debt-Paydown lane scope expansion at PR #2436 + Class P populated at PR #2437).
PERMITTED_MARKERS=(
  # Historical / dissolved record markers (pre-2026-05-09 R4-carved framings)
  'DISSOLVED'
  'dissolved'
  'AMENDED 2026-05-09'
  'carve-promot'
  'PROMOTED-IN-R3'
  'PROMOTED IN R3'
  'promoted-IN-R3'
  'promoted in R3'
  'formerly'
  'prior R4'
  'prior C[123]'
  'historical'
  'supersede'
  'preserved for traceability'
  'Original analysis'
  'pre-promotion'
  'pre-carve-promotion'
  'pre-2026-05-09'
  # Post-2026-05-09 legitimate carve channel — Class P / past-R3 routing per
  # Debt-Paydown lane scope expansion (PR #2436 + Class P populated PR #2437).
  # Going forward, R4+ work should route through Class P, not direct C1/C2/C3 carves.
  'Class P'
  'class P'
  'past-R3'
  'past R3'
  'R3 close cycle exit'
  'R4\+ via Class P'
  'Debt-Paydown'
  'debt-paydown'
)

# Canonical authority files: anti-patterns inside these files are allowed
# (these are the docs that record the carve-promotion itself).
AUTHORITY_FILES=(
  'docs/r4-carve-out-routing.md'
  'docs/audit/r3-r4-carve-substrate-readiness-2026-05-09.md'
  'docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md'
  'docs/audit/r3-pb0-velocity-walk-2026-05-09.md'
  'scripts/check-r4-carve-dissolution-discipline.sh'
)

is_authority_file() {
  local f=$1
  for af in "${AUTHORITY_FILES[@]}"; do
    [[ "$f" == "$af" ]] && return 0
  done
  return 1
}

line_has_permitted_marker() {
  local line=$1
  for m in "${PERMITTED_MARKERS[@]}"; do
    if echo "$line" | grep -qiE "$m"; then
      return 0
    fi
  done
  return 1
}

run_check() {
  local fail=0
  local checked=0
  local violations=()

  # Combine anti-patterns into a single regex
  local combined
  combined=$(IFS='|'; echo "${ANTI_PATTERNS[*]}")

  while IFS= read -r hit; do
    checked=$((checked + 1))
    local file line_num content
    file=$(echo "$hit" | cut -d: -f1)
    line_num=$(echo "$hit" | cut -d: -f2)
    content=$(echo "$hit" | cut -d: -f3-)

    # Skip authority files
    if is_authority_file "$file"; then
      continue
    fi

    # Skip if line contains a permitted-context marker
    if line_has_permitted_marker "$content"; then
      continue
    fi

    violations+=("$file:$line_num: $content")
    fail=1
  done < <(grep -rnE "$combined" docs/ 2>/dev/null || true)

  if [ $fail -eq 1 ]; then
    echo "::error::R4-carve dissolution discipline VIOLATIONS — ${#violations[@]} citation(s) of dissolved R4 carves lack supersession annotation:"
    for v in "${violations[@]}"; do
      echo "  $v"
    done
    echo ""
    echo "Per Director carve-promotion-IN-R3 ratification 2026-05-09 (gunbc#846 #issuecomment-4412330468):"
    echo "  R4 carves C1/C2/C3 (gates #81/#82/#95) DISSOLVED; reclassified R3-load-bearing within Cluster F."
    echo "  Citations of the carves must include 'DISSOLVED' / 'AMENDED 2026-05-09' / 'carve-promot' / 'formerly'"
    echo "  / 'prior' / 'historical' / 'supersede' marker, OR live in an authority file."
    echo ""
    echo "  Authority files: ${AUTHORITY_FILES[*]}"
    return 1
  fi
  echo "check-r4-carve-dissolution-discipline.sh: $checked citation(s) checked; all properly annotated or in authority files."
  return 0
}

run_self_test() {
  local fail=0

  # Self-test: a line with DISSOLVED marker should pass
  local pass_line='Gate #81 was R4-carved (C1) but is now DISSOLVED per Director ratification 2026-05-09'
  if line_has_permitted_marker "$pass_line"; then
    echo "self-test: ✓ DISSOLVED marker recognized"
  else
    echo "self-test: ✗ DISSOLVED marker NOT recognized in: $pass_line"
    fail=1
  fi

  # Self-test: a line with no marker should fail
  local fail_line='Gate #81 is R4-carved (C1) per docs/r4-carve-out-routing.md'
  if ! line_has_permitted_marker "$fail_line"; then
    echo "self-test: ✓ unannotated R4-carved citation flagged"
  else
    echo "self-test: ✗ unannotated R4-carved citation NOT flagged: $fail_line"
    fail=1
  fi

  # Self-test: AMENDED annotation should pass
  local amended_line='Gate #82 (AMENDED 2026-05-09): now R3-load-bearing within Cluster F'
  if line_has_permitted_marker "$amended_line"; then
    echo "self-test: ✓ AMENDED 2026-05-09 marker recognized"
  else
    echo "self-test: ✗ AMENDED marker NOT recognized: $amended_line"
    fail=1
  fi

  # Self-test: Class P (post-2026-05-09 legitimate carve channel) should pass
  local class_p_line='novel-substrate-introduction explicitly carved to R4+ via Class P partition (Debt-Paydown PR #2437)'
  if line_has_permitted_marker "$class_p_line"; then
    echo "self-test: ✓ Class P legitimate-carve channel recognized"
  else
    echo "self-test: ✗ Class P marker NOT recognized: $class_p_line"
    fail=1
  fi

  # Self-test: authority file should be skipped
  if is_authority_file 'docs/r4-carve-out-routing.md'; then
    echo "self-test: ✓ r4-carve-out-routing.md recognized as authority file"
  else
    echo "self-test: ✗ r4-carve-out-routing.md NOT recognized as authority"
    fail=1
  fi

  if [ $fail -eq 0 ]; then
    echo "check-r4-carve-dissolution-discipline.sh: self-test OK"
    return 0
  fi
  return 1
}

case "${1-}" in
  --self-test)
    run_self_test
    ;;
  *)
    run_check
    ;;
esac
