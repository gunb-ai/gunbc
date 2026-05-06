#!/usr/bin/env bash
# SG-0 net-shrink PR-body discipline (Director course-correction, 2026-05-05).
#
# When a pull request edits `sg0_census_test.rs`, require machine-checkable
# declarations in the PR body so net *adds* to `EXPECTED_HAND_AUTHORED_*`
# carry an explicit pairing class (retirement / Director budget / deferral).
# Class **(a)** requires the literal `removed` plus a census-shaped path token.
#
# On pull_request, the declared `SG-0 hand-path delta:` must match the net
# change to **hand-authored** census path literals only:
# `EXPECTED_HAND_AUTHORED_NON_TEST`, `EXPECTED_HAND_AUTHORED_TEST`, and
# `EXPECTED_HAND_AUTHORED_FRAGMENTS` (not `EXPECTED_GENERATED_*` or other
# string inventories in the same file). Counts are derived from
# `git show origin/main:…` vs `git show HEAD:…` (see `sg0_count_hand_expect_paths_at`).
#
# Authority: ROADMAP.md (SG-0 PR-window net-shrink discipline) +
# `.github/PULL_REQUEST_TEMPLATE.md` "SG-0 net-shrink discipline" section.
#
# Exit codes:
#   0 — not a pull_request event, census file unchanged, or body satisfies rules
#   1 — pull_request but origin/main missing; census changed on PR but body missing /
#       invalid / positive delta without pairing; or other gate failure

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SG0_CENSUS='src/v3/compiler/tests/integration/sg0_census_test.rs'

usage() {
  echo "usage: $0 [--self-test | --check-body-only]" >&2
  echo "  --self-test / --check-body-only take no additional arguments." >&2
  echo "  pull_request: requires GITHUB_EVENT_NAME=pull_request, PR_BODY," >&2
  echo "  and origin/main fetch so git diff origin/main...HEAD is meaningful." >&2
}

# Sets SG0_DECLARED_INT (signed net paths claimed in PR body).
sg0_validate_pr_body_format() {
  local body="$1"
  local delta_line token need_pairing pairing_block pairing_flat

  # Use here-strings (not `printf … | grep`) so `pipefail` + early `grep -q`
  # exit cannot SIGPIPE the writer on large PR bodies.
  if ! grep -qE '^SG-0 hand-path delta:' <<<"$body"; then
    echo "::error::PR body missing required line starting with \`SG-0 hand-path delta:\`"
    return 1
  fi
  delta_line=$(grep -E '^SG-0 hand-path delta:' <<<"$body" | head -1)
  token=${delta_line#SG-0 hand-path delta:}
  read -r token _ <<<"$token"
  if [ -z "$token" ]; then
    echo "::error::SG-0 hand-path delta line has no numeric token"
    return 1
  fi

  need_pairing=0
  if [[ "$token" =~ ^\+0$ ]] || [[ "$token" == "0" ]]; then
    need_pairing=0
    SG0_DECLARED_INT=0
  elif [[ "$token" =~ ^-([1-9][0-9]*)$ ]]; then
    need_pairing=0
    SG0_DECLARED_INT=$((0 - BASH_REMATCH[1]))
  elif [[ "$token" =~ ^\+([1-9][0-9]*)$ ]]; then
    need_pairing=1
    SG0_DECLARED_INT=${BASH_REMATCH[1]}
  else
    echo "::error::Unrecognized SG-0 hand-path delta token: $token (use signed integers: 0, +0, -N, +N)"
    return 1
  fi

  if [ "$need_pairing" -eq 1 ]; then
    pairing_block=$(awk '
      /^SG-0 pairing:/ {
        print
        if (getline > 0) print
        exit
      }
    ' <<<"$body")
    if [ -z "$pairing_block" ]; then
      echo "::error::SG-0 hand-path delta is a strict net add ($token) but PR body lacks a column-0 \`SG-0 pairing:\` line (no leading whitespace — see PR template)"
      return 1
    fi
    # Flatten pairing line + continuation so evidence may sit on the next line
    # (template: "that line or immediately after") while grep(1) does not span '\n' with '.'.
    pairing_flat=$(printf '%s\n' "$pairing_block" | tr '\n' ' ')
    if grep -qE '^SG-0 pairing:[[:space:]]*\(a\)' <<<"$pairing_block"; then
      # (a) requires the literal "removed" plus a census-shaped path token (no
      # standalone .rs / slash tokens — those are not same-PR retirement evidence).
      if ! grep -qE '\(a\).*removed[[:space:]]+(src/v3/compiler/[[:alnum:]_./-]+|[[:alnum:]_./-]+\.[rR][sS]|[[:alnum:]_./-]+\.[tT][xX][tT])' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (a) must cite same-PR retirements: include \`removed\` and a census-shaped path (\`src/v3/compiler/…\`, or a *.rs / *.txt token) on the pairing line or the line immediately after"
        return 1
      fi
    elif grep -qE '^SG-0 pairing:[[:space:]]*\(b\)' <<<"$pairing_block"; then
      if ! grep -qE 'https://|http://' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (b) must cite a Director-budget URL (http(s):// on the pairing line or the line immediately after)"
        return 1
      fi
    elif grep -qE '^SG-0 pairing:[[:space:]]*\(c\)' <<<"$pairing_block"; then
      if ! grep -qiE '\(c\).*dispatch' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (c) must name follow-up dispatch (include \"dispatch\" on the pairing line or the line immediately after)"
        return 1
      fi
    else
      echo "::error::SG-0 hand-path delta is a strict net add ($token) but PR body lacks \`SG-0 pairing: (a)\`, \`(b)\`, or \`(c)\`"
      return 1
    fi
  fi
  return 0
}

# Count path-string rows in EXPECTED_HAND_AUTHORED_{NON_TEST,TEST,FRAGMENTS}
# only (ROADMAP / PR-template authority). stdin = full `sg0_census_test.rs`.
sg0_count_hand_expect_paths_from_stdin() {
  awk '
    BEGIN { hand = 0; count = 0 }
    /^const EXPECTED_HAND_AUTHORED_FRAGMENTS:/ {
      if (/\];/) {
        line = $0
        while (match(line, /"src\/v3\/compiler\/[^"]+"/)) {
          count++
          line = substr(line, RSTART + RLENGTH)
        }
        next
      }
      hand = 1
      next
    }
    /^const EXPECTED_HAND_AUTHORED_NON_TEST:/ { hand = 1; next }
    /^const EXPECTED_HAND_AUTHORED_TEST:/ { hand = 1; next }
    hand == 1 {
      if ($0 ~ /^];/) {
        hand = 0
        next
      }
      if ($0 ~ /^    "src\/v3\/compiler\/[^"]+",[[:space:]]*$/) count++
      next
    }
    END { print count + 0 }
  '
}

sg0_count_hand_expect_paths_at() {
  local treeish=$1
  local content
  if ! content=$(git show "${treeish}:${SG0_CENSUS}" 2>/dev/null); then
    echo "::error::cannot read ${SG0_CENSUS} at ${treeish}" >&2
    return 1
  fi
  if [ -z "$content" ]; then
    echo "::error::empty ${SG0_CENSUS} at ${treeish}" >&2
    return 1
  fi
  printf '%s\n' "$content" | sg0_count_hand_expect_paths_from_stdin
}

# Net hand-authored path rows: HEAD minus origin/main (see header).
sg0_net_path_delta_from_git_diff() {
  local base head
  base=$(sg0_count_hand_expect_paths_at "origin/main") || return 1
  head=$(sg0_count_hand_expect_paths_at "HEAD") || return 1
  echo $((head - base))
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
  run_case "(a) removed without path-shaped token" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) removed not-a-path' fail
  run_case "(a) path-shaped token without removed keyword" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) src/v3/compiler/src/foo.rs' fail
  run_case "(c) without dispatch" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) later' fail
  run_case "(b) URL on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b)\nhttps://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "(a) path evidence on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a)\nremoved src/v3/compiler/src/foo.rs' pass
  run_case "(c) dispatch on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c)\nfollow-up dispatch: TM-0 lane' pass
  run_case "mid-line SG-0 pairing mention ignored" $'SG-0 hand-path delta: +1\nNarrative: not SG-0 pairing: (b) https://trap.example\nSG-0 pairing: (b) https://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "only mid-line pairing substring" $'SG-0 hand-path delta: +1\nNote: not SG-0 pairing: (b) https://x.com' fail
  run_case "indented SG-0 pairing rejected" $'SG-0 hand-path delta: +1\n    SG-0 pairing: (b) https://example.com/budget' fail

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
  sg0_validate_pr_body_format "${PR_BODY:-}" || exit 1
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
  echo "::error::SG-0 PR-body discipline requires \`origin/main\` on pull_request (fail-closed). Fetch \`main\` to \`refs/remotes/origin/main\` before this script — see \`.github/workflows/ci.yml\` step \"Fetch main for PR discipline diffs\"."
  exit 1
fi

# Census unchanged vs origin/main — path-limited diff only (avoid
# `git diff --name-only … | grep -q` + pipefail + SIGPIPE false-negative).
if git diff --quiet origin/main...HEAD -- "$SG0_CENSUS"; then
  exit 0
fi

body=${PR_BODY:-}
if ! sg0_validate_pr_body_format "$body"; then
  exit 1
fi

computed_net=$(sg0_net_path_delta_from_git_diff) || exit 1
if [ "${SG0_DECLARED_INT:-}" -ne "$computed_net" ]; then
  echo "::error::SG-0 hand-path delta mismatch: PR body declares net ${SG0_DECLARED_INT} hand-authored path(s) in EXPECTED_HAND_AUTHORED_{NON_TEST,TEST,FRAGMENTS}, but \`git show origin/main:${SG0_CENSUS}\` vs \`git show HEAD:${SG0_CENSUS}\` counts ${computed_net} net change. Fix the PR description or the census edit; if the edit is a legitimate edge case this counter misses, escalate to Director (see script header)."
  exit 1
fi

exit 0
