#!/usr/bin/env bash
# R3 debt velocity tripwire reporter.
#
# Authority:
# - `docs/r3-structure.md` standing R3 Debt-Paydown Manager
# - `INVARIANTS.md` §P5(c) velocity tripwire
# - `ROADMAP.md` `### Post-merge debt (...)` sections
#
# This script is reporting-only. It does not gate CI.
# Usage:
#   scripts/r3-debt-velocity.sh 2026-04-20 2026-05-02
# Optional third argument: path to ROADMAP.md (defaults to ./ROADMAP.md).

set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 START_DATE END_DATE [ROADMAP.md]" >&2
  exit 2
fi

START_DATE="$1"
END_DATE="$2"
ROADMAP_PATH="${3:-ROADMAP.md}"

if [ ! -f "$ROADMAP_PATH" ]; then
  echo "error: missing roadmap file: $ROADMAP_PATH" >&2
  exit 2
fi

awk -v start="$START_DATE" -v end="$END_DATE" '
function flush_section(   ratio) {
  if (current_date == "") {
    return
  }
  if (current_date < start || current_date > end) {
    return
  }
  sections++
  introduced_total += introduced
  retired_total += retired
  printf("section %s | introduced=%d retired=%d\n", current_heading, introduced, retired)
  for (i = 1; i <= receipt_count; i++) {
    print "  resolved: " receipt_lines[i]
  }
}

function reset_section() {
  introduced = 0
  retired = 0
  receipt_count = 0
  delete receipt_lines
}

BEGIN {
  sections = 0
  introduced_total = 0
  retired_total = 0
  current_date = ""
  current_heading = ""
  reset_section()
}

index($0, "### Post-merge debt (") == 1 {
  flush_section()
  reset_section()
  current_date = substr($0, length("### Post-merge debt (") + 1, 10)
  current_heading = $0
  next
}

current_date != "" {
  if ($0 ~ /^- /) {
    introduced++
    if (index($0, "~~") > 0 || index($0, "**RESOLVED ") > 0) {
      retired++
      receipt_lines[++receipt_count] = $0
    }
  }
}

END {
  flush_section()
  print "aggregate introduced=" introduced_total " retired=" retired_total
  if (retired_total == 0) {
    print "ratio=inf"
  } else {
    ratio = introduced_total / retired_total
    printf("ratio=%.2f:1\n", ratio)
  }
  print "sections=" sections
  if (retired_total > 0) {
    if (introduced_total >= 3 * retired_total) {
      exit 1
    }
    exit 0
  }
  if (introduced_total > 0) {
    exit 1
  }
  exit 0
}
' "$ROADMAP_PATH"
