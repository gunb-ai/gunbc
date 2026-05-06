#!/usr/bin/env bash
# SG-0 net-shrink PR-body discipline (Director course-correction, 2026-05-05).
#
# When a pull request edits `sg0_census_test.rs`, require machine-checkable
# declarations in the PR body so net *adds* to `EXPECTED_HAND_AUTHORED_*`
# carry an explicit pairing class (retirement / Director budget / deferral).
#
# Authority: ROADMAP.md (SG-0 PR-window net-shrink discipline) +
# `.github/PULL_REQUEST_TEMPLATE.md` "SG-0 net-shrink discipline" section.
#
# Exit codes:
#   0 — not a pull_request event, census file unchanged, or body satisfies rules
#   1 — census changed on PR but body missing / invalid / positive delta without pairing

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

usage() {
  echo "usage: $0 [--self-test | --check-body-only]" >&2
  echo "  --self-test / --check-body-only take no additional arguments." >&2
  echo "  pull_request: requires GITHUB_EVENT_NAME=pull_request, PR_BODY," >&2
  echo "  and origin/main fetch so git diff origin/main...HEAD is meaningful." >&2
}

self_test() {
  local failed=0
  run_case() {
    local name="$1" body="$2" want="$3"
    set +e
    out=$(PR_BODY=$body bash "$ROOT/scripts/check-pr-sg0-net-shrink-discipline.sh" --check-body-only 2>&1)
    st=$?
    set -e
    if [ "$want" = pass ] && [ "$st" -ne 0 ]; then
      echo "::error::self-test FAIL: $name expected pass, got exit=$st: $out"
      failed=1
    elif [ "$want" = fail ] && [ "$st" -eq 0 ]; then
      echo "::error::self-test FAIL: $name expected fail, got pass"
      failed=1
    fi
  }

  # Positive delta requires pairing
  run_case "pairing (b) with +1" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b) https://example.com/budget' pass
  run_case "+1 missing pairing" $'SG-0 hand-path delta: +1\n(no pairing line)' fail
  run_case "+0 skips pairing" $'SG-0 hand-path delta: +0' pass
  run_case "zero skips pairing" $'SG-0 hand-path delta: 0' pass
  run_case "shrink skips pairing" $'SG-0 hand-path delta: -2' pass
  run_case "(a) pairing" $'SG-0 hand-path delta: +3\nSG-0 pairing: (a) removed foo.rs bar.rs' pass
  run_case "(c) pairing" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch: TM-0 lane' pass
  run_case "missing delta line" $'Summary only\nSG-0 pairing: (a) x' fail
  run_case "malformed negative delta token" $'SG-0 hand-path delta: -not-a-number' fail
  run_case "bare (b) without URL" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b)' fail
  run_case "(a) without path evidence" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) deferred only' fail
  run_case "(c) without dispatch" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) later' fail
  run_case "(b) URL on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b)\nhttps://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "mid-line SG-0 pairing mention ignored" $'SG-0 hand-path delta: +1\nNarrative: not SG-0 pairing: (b) https://trap.example\nSG-0 pairing: (b) https://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "only mid-line pairing substring" $'SG-0 hand-path delta: +1\nNote: not SG-0 pairing: (b) https://x.com' fail

  if [ "$failed" -ne 0 ]; then
    exit 1
  fi
  echo "check-pr-sg0-net-shrink-discipline.sh: self-test OK"
  exit 0
}

# Body-only check (invoked from --self-test harness).
if [ "${1:-}" = --check-body-only ]; then
  if [ -n "${2:-}" ]; then
    usage
    exit 2
  fi
  body=${PR_BODY:-}
  if ! printf '%s\n' "$body" | grep -qE '^SG-0 hand-path delta:'; then
    echo "::error::PR body missing required line starting with \`SG-0 hand-path delta:\`"
    exit 1
  fi
  delta_line=$(printf '%s\n' "$body" | grep -E '^SG-0 hand-path delta:' | head -1)
  token=${delta_line#SG-0 hand-path delta:}
  # shellcheck disable=SC2086
  token=$(echo "$token" | awk '{print $1; exit}')
  if [ -z "$token" ]; then
    echo "::error::SG-0 hand-path delta line has no numeric token"
    exit 1
  fi

  need_pairing=0
  if [[ "$token" =~ ^\+0$ ]] || [[ "$token" == "0" ]]; then
    need_pairing=0
  elif [[ "$token" =~ ^-([1-9][0-9]*)$ ]]; then
    need_pairing=0
  elif [[ "$token" =~ ^\+([1-9][0-9]*)$ ]]; then
    need_pairing=1
  else
    echo "::error::Unrecognized SG-0 hand-path delta token: $token (use signed integers: 0, +0, -N, +N)"
    exit 1
  fi

  if [ "$need_pairing" -eq 1 ]; then
    pairing_block=$(printf '%s\n' "$body" | awk '
      /^[[:space:]]*SG-0 pairing:/ {
        print
        if (getline > 0) print
        exit
      }
    ')
    if [ -z "$pairing_block" ]; then
      echo "::error::SG-0 hand-path delta is a strict net add ($token) but PR body lacks a \`SG-0 pairing:\` line"
      exit 1
    fi
    if printf '%s\n' "$pairing_block" | grep -qE '^[[:space:]]*SG-0 pairing:[[:space:]]*\(a\)'; then
      if ! printf '%s\n' "$pairing_block" | grep -qE '\(a\).*(\.[rR][sS]\>|[[:alnum:]_-]{2,}/[[:alnum:]_./-]+|removed[[:space:]]+[^[:space:]]+)'; then
        echo "::error::SG-0 pairing (a) must name removed paths (.rs paths, multi-segment / paths, or \"removed …\" on the pairing line or the line immediately after)"
        exit 1
      fi
    elif printf '%s\n' "$pairing_block" | grep -qE '^[[:space:]]*SG-0 pairing:[[:space:]]*\(b\)'; then
      if ! printf '%s\n' "$pairing_block" | grep -qE 'https://|http://'; then
        echo "::error::SG-0 pairing (b) must cite a Director-budget URL (http(s):// on the pairing line or the line immediately after)"
        exit 1
      fi
    elif printf '%s\n' "$pairing_block" | grep -qE '^[[:space:]]*SG-0 pairing:[[:space:]]*\(c\)'; then
      if ! printf '%s\n' "$pairing_block" | grep -qiE '\(c\).*dispatch'; then
        echo "::error::SG-0 pairing (c) must name follow-up dispatch (include \"dispatch\" on the pairing line or the line immediately after)"
        exit 1
      fi
    else
      echo "::error::SG-0 hand-path delta is a strict net add ($token) but PR body lacks \`SG-0 pairing: (a)\`, \`(b)\`, or \`(c)\`"
      exit 1
    fi
  fi
  exit 0
fi

if [ "${1:-}" = --self-test ]; then
  if [ -n "${2:-}" ]; then
    usage
    exit 2
  fi
  self_test
fi

if [ "${1:-}" != "" ]; then
  usage
  exit 2
fi

if [ "${GITHUB_EVENT_NAME:-}" != "pull_request" ]; then
  exit 0
fi

if ! git rev-parse -q --verify origin/main >/dev/null 2>&1; then
  echo "::notice::origin/main not available locally — skipping SG-0 PR-body discipline (CI fetches before this step)"
  exit 0
fi

if ! git diff --name-only origin/main...HEAD | grep -Fxq 'src/v3/compiler/tests/integration/sg0_census_test.rs'; then
  exit 0
fi

body=${PR_BODY:-}
export PR_BODY=$body
exec bash "$ROOT/scripts/check-pr-sg0-net-shrink-discipline.sh" --check-body-only
