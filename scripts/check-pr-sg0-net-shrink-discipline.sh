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
# Fallback: when census changed vs origin/main yet PR_BODY lacks `SG-0 hand-path delta:`, prefix
# **only** the mechanically correct delta token (from `git diff` counts). **`SG-0 pairing:` lines
# are never fabricated** — strict net adds (`+N`) still require an author-authored pairing class
# (a)/(b)/(c); see `sg0_validate_pr_body_format`.
#
# Exit codes:
#   0 — not a pull_request event, census file unchanged, or body satisfies rules
#   1 — pull_request but origin/main missing; census changed on PR but body invalid /
#       positive declared delta without pairing; declared vs computed mismatch; or other gate failure

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SG0_SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"

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

  # GitHub `pull_request.body` can be CRLF-shaped; strip `\r` so column-0 anchors
  # and numeric tokens match LF-only self-tests and typical Unix tooling.
  body=${body//$'\r'/}

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
      # `.txt` fragments must stay under `src/v3/compiler/`; short `*.rs` remains allowed.
      if ! grep -qE '\(a\).*removed[[:space:]]+(src/v3/compiler/[[:alnum:]_./-]+|[[:alnum:]_./-]+\.[rR][sS]|src/v3/compiler/[[:alnum:]_./-]+\.[tT][xX][tT])' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (a) must cite same-PR retirements: include \`removed\` and a census-shaped path (\`src/v3/compiler/…\`, short \`*.rs\`, or \`src/v3/compiler/…*.txt\`) on the pairing line or the line immediately after"
        return 1
      fi
    elif grep -qE '^SG-0 pairing:[[:space:]]*\(b\)' <<<"$pairing_block"; then
      if ! grep -qE 'https://|http://' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (b) must cite a Director-budget URL (http(s):// on the pairing line or the line immediately after)"
        return 1
      fi
    elif grep -qE '^SG-0 pairing:[[:space:]]*\(c\)' <<<"$pairing_block"; then
      # Tightened 2026-05-09 per PB-0 velocity walk (PR #2358 / docs/audit/r3-pb0-velocity-walk-2026-05-09.md):
      # SG-0 census grew +30/9days because (c) deferrals were paper-trailed without dispatch enforcement.
      # (c) now requires concrete dispatch evidence: a qualified tracker issue ref (gunbc#NNNN or
      # gunb-ai/gunbc#NNNN), a full GitHub issue URL, OR a brief path (docs/briefs/*.md). Bare #NNNN
      # refs (could be gate-numbers in prose) are NOT accepted — see regex below + codex BLOCKING fix.
      # Generic "dispatch" word alone no longer satisfies.
      if ! grep -qiE '\(c\).*dispatch' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (c) must name follow-up dispatch (include \"dispatch\" on the pairing line or the line immediately after)"
        return 1
      fi
      # Full-URL alternative tightened 2026-05-09 per openai-pro REQUEST_CHANGES:
      # prior pattern `https?://github\.com/[[:alnum:]_./-]+/issues/[0-9]+` accepted ANY GitHub
      # issue URL — unrelated repos (e.g. github.com/other/repo/issues/1234) could satisfy the
      # SG-0 deferral gate. ROADMAP.md option (c) is "dispatch-tracker issue URL", which means
      # the gunbc tracker. Tightened to gunb-ai/gunbc only.
      if ! grep -qE '(gunbc#|gunb-ai/gunbc#)[0-9]+|docs/briefs/[[:alnum:]_./-]+\.md|https?://github\.com/gunb-ai/gunbc/issues/[0-9]+' <<<"$pairing_flat"; then
        echo "::error::SG-0 pairing (c) must cite concrete dispatch evidence — a qualified tracker issue ref (gunbc#NNNN or gunb-ai/gunbc#NNNN), a gunb-ai/gunbc issue URL (https://github.com/gunb-ai/gunbc/issues/NNNN), OR a queued brief path (docs/briefs/*.md). Bare #NNNN refs + external-repo URLs (could be from any GitHub project) no longer accepted. Tightened 2026-05-09 per codex BLOCKING + openai-pro REQUEST_CHANGES."
        return 1
      fi
      # Tightened 2026-05-09 per codex BLOCKING on PR #2361 sha b925b174:
      # cited brief paths must exist in the checkout — string-pattern match alone is escapable.
      # Issue refs / URLs are external; not file-checkable here. Brief paths get file existence verification.
      #
      # Tightened further 2026-05-09 per codex BLOCKING on PR #2361 sha 296f72fa8 (line 119):
      # path must be canonical (no `..` segments that escape `docs/briefs/`); existence check alone
      # is insufficient — `docs/briefs/../foo.md` would resolve to `docs/foo.md` which could exist
      # as a non-brief file. Reject path-traversal tokens; verify resolved file is under docs/briefs/.
      cited_brief_paths=$(grep -oE 'docs/briefs/[[:alnum:]_./-]+\.md' <<<"$pairing_flat" || true)
      if [ -n "$cited_brief_paths" ]; then
        while IFS= read -r p; do
          [ -z "$p" ] && continue
          # Reject path-traversal tokens: any `..` segment escapes docs/briefs/ scope.
          if grep -qE '(^|/)\.\.(/|$)' <<<"$p"; then
            echo "::error::SG-0 pairing (c) cited brief path '$p' contains '..' path-traversal segment; brief paths must be canonical under docs/briefs/. Tightened 2026-05-09 per codex BLOCKING."
            return 1
          fi
          # Reject paths not actually under docs/briefs/ (defense-in-depth — prefix already enforced
          # by capture pattern, but explicit canonical-prefix check guards against future regex relaxation).
          if [[ "$p" != docs/briefs/* ]]; then
            echo "::error::SG-0 pairing (c) cited brief path '$p' is not under docs/briefs/; must be a queued brief in the canonical briefs directory."
            return 1
          fi
          if [ ! -f "$ROOT/$p" ]; then
            echo "::error::SG-0 pairing (c) cited brief path '$p' does not exist in the checkout. Either author the brief in this PR, fix the path, or use an issue ref instead. Tightened 2026-05-09 per codex BLOCKING."
            return 1
          fi
        done <<<"$cited_brief_paths"
      fi
    else
      echo "::error::SG-0 hand-path delta is a strict net add ($token) but PR body lacks \`SG-0 pairing: (a)\`, \`(b)\`, or \`(c)\`"
      return 1
    fi
  fi
  return 0
}

# If `body` lacks a column-0 `SG-0 hand-path delta:` line, prepend the single line described by
# `computed_net` (signed path count). Never injects `SG-0 pairing:` — positive net adds remain
# gated on author pairing in `sg0_validate_pr_body_format`.
sg0_maybe_prepend_synthetic_delta_line() {
  local body=$1
  local computed_net=$2
  local delta_line

  body=${body//$'\r'/}

  if grep -qE '^SG-0 hand-path delta:' <<<"$body"; then
    printf '%s\n' "$body"
    return 0
  fi

  if [ "$computed_net" -ge 1 ]; then
    delta_line="SG-0 hand-path delta: +${computed_net}"
  elif [ "$computed_net" -eq 0 ]; then
    delta_line="SG-0 hand-path delta: +0"
  else
    delta_line="SG-0 hand-path delta: ${computed_net}"
  fi

  printf '%s\n\n%s\n' "$delta_line" "$body"
}

# Count path-string rows in EXPECTED_HAND_AUTHORED_{NON_TEST,TEST,FRAGMENTS}
# only (ROADMAP / PR-template authority). stdin = full `sg0_census_test.rs`.
# Assumptions: `rustfmt` keeps a trailing comma on each multi-line path row (the
# block counter matches `,` before optional EOL `//` comment). `EXPECTED_HAND_AUTHORED_FRAGMENTS` today is
# a one-line `&[...];` const (inline scan); a multi-line reformatted block would
# fall through to the same `hand==1` arm as NON_TEST/TEST — extend explicitly if
# both shapes must diverge. Counter **fail-closes** if any of the three `const`
# blocks is missing, duplicated, or left unclosed (see awk `END`).
sg0_count_hand_expect_paths_from_stdin() {
  awk '
    BEGIN { hand = 0; count = 0; saw_nt = 0; saw_t = 0; saw_f = 0 }
    /^const EXPECTED_HAND_AUTHORED_FRAGMENTS:/ {
      saw_f++
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
    /^const EXPECTED_HAND_AUTHORED_NON_TEST:/ { saw_nt++; hand = 1; next }
    /^const EXPECTED_HAND_AUTHORED_TEST:/ { saw_t++; hand = 1; next }
    hand == 1 {
      if ($0 ~ /^];/) {
        hand = 0
        next
      }
      line = $0
      sub(/\/\/.*$/, "", line)
      gsub(/[[:space:]]+$/, "", line)
      if (line ~ /^[[:space:]]*"src\/v3\/compiler\/[^"]+",[[:space:]]*$/) {
        count++
        next
      }
      if (line ~ /"src\/v3\/compiler\//) {
        print "sg0_census_counter: hand-authored path row does not match supported shape (rustfmt path + trailing comma, optional // comment)" > "/dev/stderr"
        exit 1
      }
      next
    }
    END {
      if (saw_nt != 1 || saw_t != 1 || saw_f != 1) {
        print "sg0_census_counter: expected exactly one const each EXPECTED_HAND_AUTHORED_NON_TEST, EXPECTED_HAND_AUTHORED_TEST, EXPECTED_HAND_AUTHORED_FRAGMENTS (missing, renamed, or duplicated)" > "/dev/stderr"
        exit 1
      }
      if (hand != 0) {
        print "sg0_census_counter: unclosed hand-authored EXPECTED_HAND_AUTHORED_* array (no closing ] line)" > "/dev/stderr"
        exit 1
      }
      print count + 0
    }
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

sg0_assert_ci_wiring() {
  local workflow=".github/workflows/ci.yml"
  local self_test_line body_check_line bootstrap_verify_line

  if [ ! -f "$workflow" ]; then
    echo "::error::missing $workflow; gate #75 requires CI wiring for SG-0 PR-body discipline"
    return 1
  fi

  self_test_line=$(grep -nE 'bash[[:space:]]+scripts/check-pr-sg0-net-shrink-discipline\.sh[[:space:]]+--self-test([[:space:]]|$)' "$workflow" | head -1 | cut -d: -f1 || true)
  body_check_line=$(grep -nE 'bash[[:space:]]+scripts/check-pr-sg0-net-shrink-discipline\.sh([[:space:]]|$)' "$workflow" | grep -v -- '--self-test' | head -1 | cut -d: -f1 || true)
  bootstrap_verify_line=$(grep -nE '^[[:space:]]*run:[[:space:]]+.*regen_bootstrap[[:space:]].*--verify' "$workflow" | head -1 | cut -d: -f1 || true)

  if [ -z "$self_test_line" ]; then
    echo "::error::$workflow no longer runs check-pr-sg0-net-shrink-discipline.sh --self-test"
    return 1
  fi
  if [ -z "$body_check_line" ]; then
    echo "::error::$workflow no longer runs check-pr-sg0-net-shrink-discipline.sh for pull_request PR-body enforcement"
    return 1
  fi
  if [ -z "$bootstrap_verify_line" ]; then
    echo "::error::$workflow no longer has the bootstrap verify step used as the gate-order anchor"
    return 1
  fi
  if [ "$self_test_line" -ge "$body_check_line" ] || [ "$body_check_line" -ge "$bootstrap_verify_line" ]; then
    echo "::error::$workflow must run SG-0 discipline self-test, then PR-body enforcement, before bootstrap verify (gate #75 CI-active receipt)"
    return 1
  fi
}

self_test() {
  local failed=0
  run_case() {
    local name="$1" body="$2" want="$3"
    set +e
    out=$(PR_BODY=$body bash "$SG0_SCRIPT" --check-body-only 2>&1)
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

  run_census_case() {
    local name="$1" snippet="$2" want="$3" want_n="${4:-}"
    set +e
    out=$(printf '%s\n' "$snippet" | sg0_count_hand_expect_paths_from_stdin 2>&1)
    st=$?
    set -e
    if [ "$want" = pass ]; then
      if [ "$st" -ne 0 ] || [ "$out" != "$want_n" ]; then
        echo "::error::self-test FAIL census $name: exit=$st out=$out want_count=$want_n"
        failed=1
      fi
    else
      if [ "$st" -eq 0 ]; then
        echo "::error::self-test FAIL census $name expected counter fail, got pass out=$out"
        failed=1
      fi
    fi
  }

  # --- census counter (stdin synthetic snapshots; fail-closed shape) ---
  local census_ok census_frag_ml census_miss_nt census_miss_f census_dup census_unclosed census_row_comment census_bad_row
  census_ok=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n    "src/v3/compiler/a.rs",\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n    "src/v3/compiler/tests/b.rs",\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/c.txt"];\n'
  run_census_case "census counter minimal valid" "$census_ok" pass 3

  census_row_comment=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n    "src/v3/compiler/x.rs", // ratchet note\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/c.txt"];\n'
  run_census_case "census counter path row with trailing // comment" "$census_row_comment" pass 2

  census_bad_row=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n    "src/v3/compiler/x.rs"\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/c.txt"];\n'
  run_census_case "census counter non-matching path row fails closed" "$census_bad_row" fail

  census_frag_ml=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &[\n    "src/v3/compiler/z.txt",\n];\n'
  run_census_case "census counter multiline FRAGMENTS block" "$census_frag_ml" pass 1

  census_miss_nt=$'const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/c.txt"];\n'
  run_census_case "census counter missing NON_TEST" "$census_miss_nt" fail

  census_miss_f=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\n'
  run_census_case "census counter missing FRAGMENTS" "$census_miss_f" fail

  census_dup=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/c.txt"];\n'
  run_census_case "census counter duplicate NON_TEST" "$census_dup" fail

  census_unclosed=$'const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[\n];\nconst EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &[\n    "src/v3/compiler/c.txt",\n'
  run_census_case "census counter unclosed FRAGMENTS array" "$census_unclosed" fail

  # --- PR-body pairing (existing) ---
  run_case "pairing (b) with +1" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b) https://example.com/budget' pass
  run_case "CRLF +1 (b) pairing" $'SG-0 hand-path delta: +1\r\nSG-0 pairing: (b) https://example.com/budget\r\n' pass
  run_case "+1 missing pairing" $'SG-0 hand-path delta: +1\n(no pairing line)' fail
  run_case "+0 skips pairing" $'SG-0 hand-path delta: +0' pass
  run_case "zero skips pairing" $'SG-0 hand-path delta: 0' pass
  run_case "CRLF +0 skips pairing" $'SG-0 hand-path delta: +0\r\n' pass
  run_case "shrink skips pairing" $'SG-0 hand-path delta: -2' pass
  run_case "(a) pairing" $'SG-0 hand-path delta: +3\nSG-0 pairing: (a) removed foo.rs bar.rs' pass
  run_case "(c) pairing with existing brief path" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via docs/briefs/r3-v-tests-as-data-v1-worker.md' pass
  run_case "(c) pairing with issue ref" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via gunbc#1234' pass
  run_case "(c) pairing with full GitHub issue URL" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via https://github.com/gunb-ai/gunbc/issues/1234' pass
  run_case "(c) without dispatch evidence (tightened 2026-05-09)" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch: vague-lane-name' fail
  run_case "(c) cited brief path that does not exist (tightened 2026-05-09 codex BLOCKING)" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via docs/briefs/r3-nonexistent-brief-2026.md' fail
  run_case "(c) cited brief path with path-traversal (.. segment)" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via docs/briefs/../r3-program-plan.md' fail
  run_case "(c) bare #NNNN gate-number-in-prose (tightened 2026-05-09 codex BLOCKING)" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch for gate #84' fail
  run_case "(c) external-repo GitHub issue URL (tightened 2026-05-09 openai-pro REQUEST_CHANGES — must be gunb-ai/gunbc tracker)" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) follow-up dispatch via https://github.com/other-org/other-repo/issues/1234' fail
  run_case "missing delta line" $'Summary only\nSG-0 pairing: (a) x' fail
  run_case "malformed negative delta token" $'SG-0 hand-path delta: -not-a-number' fail
  run_case "bare (b) without URL" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b)' fail
  run_case "(a) without path evidence" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) deferred only' fail
  run_case "(a) removed without path-shaped token" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) removed not-a-path' fail
  run_case "(a) path-shaped token without removed keyword" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) src/v3/compiler/src/foo.rs' fail
  run_case "(a) removed bare .txt rejected" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) removed other.txt' fail
  run_case "(a) removed fragment txt under census root" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a) removed src/v3/compiler/parse_parser_body.txt' pass
  run_case "(c) without dispatch" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c) later' fail
  run_case "(b) URL on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (b)\nhttps://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "CRLF (b) URL on following line" $'SG-0 hand-path delta: +1\r\nSG-0 pairing: (b)\r\nhttps://github.com/gunb-ai/gunbc/issues/1\r\n' pass
  run_case "(a) path evidence on following line" $'SG-0 hand-path delta: +1\nSG-0 pairing: (a)\nremoved src/v3/compiler/src/foo.rs' pass
  run_case "(c) dispatch on following line with existing brief path" $'SG-0 hand-path delta: +1\nSG-0 pairing: (c)\nfollow-up dispatch via docs/briefs/r3-v-tests-as-data-v1-worker.md' pass
  run_case "mid-line SG-0 pairing mention ignored" $'SG-0 hand-path delta: +1\nNarrative: not SG-0 pairing: (b) https://trap.example\nSG-0 pairing: (b) https://github.com/gunb-ai/gunbc/issues/1' pass
  run_case "only mid-line pairing substring" $'SG-0 hand-path delta: +1\nNote: not SG-0 pairing: (b) https://x.com' fail
  run_case "indented SG-0 pairing rejected" $'SG-0 hand-path delta: +1\n    SG-0 pairing: (b) https://example.com/budget' fail

  # --- Synthetic delta-prefix (positive net adds must still ship author pairing; see header) ---
  run_synthetic_delta_then_validate() {
    local name="$1" raw_body="$2" computed_net="$3" want="$4"
    local merged=""
    merged=$(sg0_maybe_prepend_synthetic_delta_line "$raw_body" "$computed_net")
    set +e
    local out=""
    local st=""
    out=$(PR_BODY="$merged" bash "$SG0_SCRIPT" --check-body-only 2>&1)
    st=$?
    set -e
    if [ "$want" = pass ] && [ "$st" -ne 0 ]; then
      echo "::error::self-test FAIL synth-delta $name expected pass, got exit=$st: $out merged=$merged"
      failed=1
    elif [ "$want" = fail ] && [ "$st" -eq 0 ]; then
      echo "::error::self-test FAIL synth-delta $name expected fail, got pass merged=$merged"
      failed=1
    fi
  }

  run_synthetic_delta_then_validate \
    "+N delta-only prepend without pairing rejects" \
    $'Summary (template)\nMore text' \
    2 \
    fail
  run_synthetic_delta_then_validate \
    "+N prepend passes when pairing already authored" \
    $'Intro\nSG-0 pairing: (c) structural deferral; follow-up dispatch via docs/briefs/r3-v-tests-as-data-v1-worker.md\n' \
    2 \
    pass
  run_synthetic_delta_then_validate \
    "+0 prepend without pairing accepts" \
    $'Intro only\n' \
    0 \
    pass
  run_synthetic_delta_then_validate \
    "negative net prepend without pairing accepts" \
    $'Removed paths summary\n' \
    -3 \
    pass

  if ! sg0_assert_ci_wiring; then
    failed=1
  fi

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

computed_net=$(sg0_net_path_delta_from_git_diff) || exit 1

body=${PR_BODY:-}
body=${body//$'\r'/}

if ! grep -qE '^SG-0 hand-path delta:' <<<"$body"; then
  echo "::notice::SG-0 net-shrink: PR body lacked a column-0 \`SG-0 hand-path delta:\` while editing ${SG0_CENSUS} — inserting delta-only line computed_net=${computed_net}. Positive net increases still require an author \`SG-0 pairing:\` line (never auto-synthesized)." >&2
  body=$(sg0_maybe_prepend_synthetic_delta_line "${PR_BODY:-}" "$computed_net")
fi

if ! sg0_validate_pr_body_format "$body"; then
  exit 1
fi

if [ "${SG0_DECLARED_INT:-}" -ne "$computed_net" ]; then
  echo "::error::SG-0 hand-path delta mismatch: PR body declares net ${SG0_DECLARED_INT} hand-authored path(s) in EXPECTED_HAND_AUTHORED_{NON_TEST,TEST,FRAGMENTS}, but \`git show origin/main:${SG0_CENSUS}\` vs \`git show HEAD:${SG0_CENSUS}\` counts ${computed_net} net change. Fix the PR description or the census edit; if the edit is a legitimate edge case this counter misses, escalate to Director (see script header)."
  exit 1
fi

exit 0
