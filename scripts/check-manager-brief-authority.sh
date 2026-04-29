#!/usr/bin/env bash
# Manager-brief authority consumer — verifies that R2 manager briefs
# cite live authorities and use consistent projections across briefs.
#
# Authority: gpt-5-5-pro meta-review on PR #1126 (2026-04-29) +
# `docs/r2-structure.md` §v2-guardrail-requirement-3.
#
# Scope: dissolves the "non-live authority consumed as live" pattern
# that recurred 5+ times during PR #1078 + #1126 review loops (codex
# tooling false-positives + cursor-flagged single-authority drift).
# v1 covers 4 of gpt-5-5-pro's 5 questions:
#   Q1 — cited parent authority file existence
#   Q2 — cited section anchor existence (§"section name" + path#anchor)
#   Q4 — `LANDED via #N` PR is reachable in this branch's history
#   Q5 — cross-brief lane-count / manager-count projections agree
# Q3 (controlled status vocabulary) deferred to v2 — too subjective
# for v1; named below as the next narrowing opportunity.
#
# Exit codes:
#   0 — no violations
#   1 — at least one violation

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Derive the GitHub repo slug for `gh pr view --repo`. Required in CI
# because `actions/checkout@v4`'s shallow clone may not expose the
# remote in a form `gh` auto-detects (CI ran `gh pr view 1080` and got
# nothing, even though locally the same call returns state=MERGED).
# Falls back to "gunb-ai/gunbc" if origin isn't readable (e.g.,
# self-test tmpdir with no remote set).
REPO_SLUG="$(git config --get remote.origin.url 2>/dev/null \
  | sed -E 's#.*github\.com[:/]##; s#\.git$##' \
  || echo "")"
if [ -z "$REPO_SLUG" ]; then
  REPO_SLUG="gunb-ai/gunbc"
fi

# 7 R2 manager briefs (mirrors r2-structure.md §"Manager structure"
# count of 7 standing managers post-Evaluator-add 2026-04-28).
# Listed explicitly (not glob) so renames surface as violations
# rather than silently passing.
MANAGER_BRIEFS=(
  "docs/briefs/r2-evaluator-manager.md"
  "docs/briefs/r2-grounding-manager.md"
  "docs/briefs/r2-impossible-bugs-manager.md"
  "docs/briefs/r2-modeling-manager.md"
  "docs/briefs/r2-pure-bootstrap-manager.md"
  "docs/briefs/r2-release-manager.md"
  "docs/briefs/r2-substrate-manager.md"
)

violations=0

# ---------------------------------------------------------------------
# Q1 — Cited file existence
# ---------------------------------------------------------------------
# Extract markdown links of form [text](path) where path is not a URL,
# not a fragment-only anchor, and not in a known excluded set; verify
# each path exists on disk relative to the brief's directory.

check_q1_file_existence() {
  local brief="$1"
  local brief_dir
  brief_dir="$(dirname "$brief")"
  local brief_violations=0

  # grep -oE extracts every (path) occurrence; we then strip wrapping
  # and drop URLs / pure-anchor / external fragments.
  while IFS= read -r raw_path; do
    [ -z "$raw_path" ] && continue

    # Strip wrapping parens already done by grep capture; raw_path is
    # path[#anchor] potentially.
    local path_only="${raw_path%%#*}"

    # Skip URLs.
    case "$path_only" in
      http://*|https://*|mailto:*) continue ;;
    esac

    # Skip pure-anchor (intra-doc) links.
    [ -z "$path_only" ] && continue

    # Resolve relative to brief's directory.
    local resolved
    if [[ "$path_only" = /* ]]; then
      resolved="$ROOT$path_only"
    else
      resolved="$brief_dir/$path_only"
    fi

    # Normalize ../ segments via realpath if available, else leave.
    if command -v realpath >/dev/null 2>&1; then
      resolved="$(realpath -m "$resolved" 2>/dev/null || echo "$resolved")"
    fi

    if [ ! -e "$resolved" ]; then
      echo "VIOLATION [Q1 file-existence]: $brief"
      echo "  cited path: $path_only"
      echo "  resolved:   $resolved"
      brief_violations=$((brief_violations + 1))
    fi
  done < <(grep -oE '\]\([^)]+\)' "$brief" | sed -E 's/^\]\(//; s/\)$//')

  return $brief_violations
}

# ---------------------------------------------------------------------
# Q2 — Cited section anchor existence (markdown form only)
# ---------------------------------------------------------------------
# For `path#anchor` markdown links, verify the anchor matches a heading
# in the target file (heading slugified to GitHub form: lowercase,
# spaces→dashes, punctuation stripped).
#
# Out of scope for v1: `§"section name"` prose-form citations. They're
# common in the briefs but parsing "what file does this prose §
# reference" requires more context than a regex can give. v2 candidate;
# tracked alongside Q3 status-vocabulary as the next narrowing.

slugify() {
  echo "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9 -]//g; s/  */-/g; s/--*/-/g; s/^-//; s/-$//'
}

check_q2_anchor_existence() {
  local brief="$1"
  local brief_dir
  brief_dir="$(dirname "$brief")"
  local brief_violations=0

  # Markdown anchor links: extract path#anchor pairs.
  while IFS= read -r raw_link; do
    [ -z "$raw_link" ] && continue
    case "$raw_link" in
      *"#"*) ;;
      *) continue ;;
    esac

    local path_only="${raw_link%%#*}"
    local anchor="${raw_link#*#}"

    case "$path_only" in
      http://*|https://*|mailto:*) continue ;;
    esac

    # Skip intra-doc anchors and Markdown linkrefs without a path.
    [ -z "$path_only" ] && continue

    local resolved
    if [[ "$path_only" = /* ]]; then
      resolved="$ROOT$path_only"
    else
      resolved="$brief_dir/$path_only"
    fi

    if command -v realpath >/dev/null 2>&1; then
      resolved="$(realpath -m "$resolved" 2>/dev/null || echo "$resolved")"
    fi

    [ ! -f "$resolved" ] && continue  # Q1 already reports missing files

    # Extract all heading slugs from target.
    local found=0
    while IFS= read -r heading; do
      heading="${heading#"${heading%%[![:space:]]*}"}"  # ltrim
      heading="${heading##\#* }"                          # strip leading #s + space
      local slug
      slug="$(slugify "$heading")"
      if [ "$slug" = "$anchor" ]; then
        found=1
        break
      fi
    done < <(grep -E '^#{1,6} ' "$resolved" || true)

    if [ "$found" = "0" ]; then
      echo "VIOLATION [Q2 anchor-existence]: $brief"
      echo "  cited link: $path_only#$anchor"
      echo "  target file: $resolved"
      echo "  no heading slug matches '$anchor'"
      brief_violations=$((brief_violations + 1))
    fi
  done < <(grep -oE '\]\([^)#]+#[^)]+\)' "$brief" | sed -E 's/^\]\(//; s/\)$//')

  return $brief_violations
}

# ---------------------------------------------------------------------
# Q4 — LANDED via #N PR-reachability check
# ---------------------------------------------------------------------
# Every `LANDED via #N` (or `landed via #N`) claim must correspond to
# a merge commit reachable from this branch's history.

check_q4_landed_pr_in_history() {
  local brief="$1"
  local brief_violations=0

  while IFS= read -r pr_num; do
    [ -z "$pr_num" ] && continue

    # Two-stage check ordered for CI compatibility (CI uses
    # fetch-depth=1 shallow clone, so git-log can't see merge history;
    # gh API works regardless of clone depth).
    #
    # Capture all output explicitly — don't pipe to grep -q under
    # pipefail (grep -q exits early on first match, causes SIGPIPE
    # on upstream git-log, pipefail reports pipeline failure).
    #
    # Stage 1 (primary): gh pr view — works in shallow clone, robust
    # to squash-merge subject variance (PR #900 was squash-merged
    # without "(#900)" suffix; pure git-grep would miss it).
    # Pass --repo explicitly: `actions/checkout@v4`'s shallow clone
    # exposes the remote in a form `gh` doesn't always auto-detect.
    # Capture stderr so auth/rate-limit failures surface in diagnostics.
    local pr_state="" gh_stderr=""
    if command -v gh >/dev/null 2>&1; then
      pr_state="$(gh pr view "$pr_num" --repo "$REPO_SLUG" --json state --jq '.state' 2>/tmp/gh-stderr-$$ || true)"
      gh_stderr="$(cat /tmp/gh-stderr-$$ 2>/dev/null || true)"
      rm -f /tmp/gh-stderr-$$
      if [ "$pr_state" = "MERGED" ]; then
        continue
      fi
    fi

    # Stage 2 (fallback): pure-git grep for "(#N)" merge subject.
    # Used when gh is unavailable (offline dev) or auth-blocked.
    # In CI with fetch-depth=1 this stage finds nothing — that's
    # why Stage 1 is primary.
    local matches
    matches="$(git log --all --oneline --grep="(#${pr_num})" 2>/dev/null || true)"
    if [ -n "$matches" ]; then
      continue
    fi

    echo "VIOLATION [Q4 landed-pr-in-history]: $brief"
    echo "  cited claim: 'LANDED via #${pr_num}'"
    echo "  PR is not in MERGED state via either:"
    echo "    - gh pr view #${pr_num} --repo ${REPO_SLUG} returning state=MERGED"
    echo "    - git log finding a merge subject '(#${pr_num})'"
    if [ -n "$pr_state" ] && [ "$pr_state" != "MERGED" ]; then
      echo "    - gh reports state=${pr_state}"
    fi
    if [ -n "$gh_stderr" ]; then
      echo "  gh stderr (truncated to 200 chars):"
      echo "    $(echo "$gh_stderr" | head -c 200)"
    fi
    brief_violations=$((brief_violations + 1))
  done < <(grep -oE '(LANDED|landed) via #[0-9]+' "$brief" | grep -oE '#[0-9]+' | tr -d '#' | sort -u)

  return $brief_violations
}

# ---------------------------------------------------------------------
# Q5 — Cross-brief projection consistency
# ---------------------------------------------------------------------
# Manager count, R2 lane count, R3 lane count, R3-Evaluator-gated count
# all should agree across the briefs that mention them. Drift here is
# exactly the class gpt-5-5-pro flagged ("only manager continues to R3"
# vs PB also continues; "5 R2-archiving managers" vs all 6 managers).

check_q5_cross_brief_projections() {
  local brief_violations=0

  # Pattern set: (label, canonical value, POSIX-extended regex).
  # Encoding is "label||canonical||regex" using "||" as field separator
  # to avoid colliding with the regex's own characters.
  # Canonical values match docs/r2-structure.md (manager count) and
  # docs/r3-structure.md (lane counts) authority docs.
  # Regex must use POSIX-extended (bash grep -E): [0-9]+, not \d+.
  #
  # Per claude-opus-4-7 review: empirical check confirms the bare
  # "standing managers" pattern does NOT substring-match
  # "standing R2 managers" — the regex requires "managers"
  # immediately after "standing" with one space, which "R2 " breaks.
  # The two patterns coexist as separate constraints; if a brief
  # uses either phrasing the count must match canonical (7).
  declare -a patterns=(
    'standing R2 managers||7||[0-9]+ standing R2 managers'
    'standing managers (no R2 qualifier)||7||[0-9]+ standing managers'
    'other managers||6||[0-9]+ other managers'
    'R3 lanes||10||[0-9]+ R3 lanes'
    'R3-Evaluator-gated lanes||7||[0-9]+ of 10 R3 lanes'
  )

  for entry in "${patterns[@]}"; do
    # Field separator is "||" (double-pipe) so single | in regex is safe.
    local label="${entry%%||*}"
    local rest="${entry#*||}"
    local canonical="${rest%%||*}"
    local pattern="${rest#*||}"

    declare -A counts_seen=()
    local mismatch=0
    local mismatch_details=""

    for brief in "${MANAGER_BRIEFS[@]}"; do
      while IFS= read -r match; do
        [ -z "$match" ] && continue
        # Extract the leading number from the matched substring.
        local n
        n="$(echo "$match" | grep -oE '^[0-9]+')"
        [ -z "$n" ] && continue
        counts_seen["$n"]="${counts_seen[$n]:-}${brief}; "
      done < <(grep -oE "$pattern" "$brief" 2>/dev/null || true)
    done

    if [ "${#counts_seen[@]}" -gt 1 ]; then
      mismatch=1
      for n in "${!counts_seen[@]}"; do
        mismatch_details+="    count=$n in: ${counts_seen[$n]}"$'\n'
      done
    fi

    if [ "$mismatch" = "1" ]; then
      echo "VIOLATION [Q5 cross-brief-projection]: '$label' inconsistent across briefs"
      echo "  canonical value: $canonical"
      echo "  observed counts:"
      printf "%s" "$mismatch_details"
      brief_violations=$((brief_violations + 1))
    elif [ "${#counts_seen[@]}" -eq 1 ]; then
      # Single value seen — verify it matches canonical.
      local seen
      seen="$(echo "${!counts_seen[@]}" | tr -d ' ')"
      if [ "$seen" != "$canonical" ]; then
        echo "VIOLATION [Q5 cross-brief-projection]: '$label' diverges from canonical"
        echo "  canonical value: $canonical"
        echo "  observed value:  $seen"
        echo "  in: ${counts_seen[$seen]}"
        brief_violations=$((brief_violations + 1))
      fi
    fi
    # If 0 counts seen, no brief mentions this projection — silent OK.
  done

  return $brief_violations
}

# ---------------------------------------------------------------------
# Driver: run all checks across all briefs.
# ---------------------------------------------------------------------

# Fail-closed precheck: every configured brief must exist.
missing_briefs=()
for brief in "${MANAGER_BRIEFS[@]}"; do
  if [ ! -f "$brief" ]; then
    missing_briefs+=("$brief")
  fi
done

if [ "${#missing_briefs[@]}" -gt 0 ]; then
  echo "Manager-brief authority check FAILED: ${#missing_briefs[@]} configured brief(s) missing."
  echo ""
  for brief in "${missing_briefs[@]}"; do
    echo "  MISSING: $brief"
  done
  echo ""
  echo "Each missing brief was declared in MANAGER_BRIEFS in this script."
  echo "Either the brief was renamed (update MANAGER_BRIEFS) or genuinely"
  echo "retired (remove from MANAGER_BRIEFS with the same review attention"
  echo "as adding a new manager)."
  exit 1
fi

# Drive the per-brief checks. Use `|| rc=$?` to prevent `set -e` from
# exiting on a function that returns non-zero — we WANT to keep going
# and report all violations across all briefs in a single run, not
# stop at the first failing one. (Without `||`, `set -e` treats the
# function call's non-zero return as a failed command.)
#
# The functions still return uint8 via `return $brief_violations`, but
# we read it via $? immediately and accumulate into the global
# `violations` int — which is plain bash arithmetic and not bounded
# to 8 bits. This sidesteps the 256-violation truncation foot-gun.
for brief in "${MANAGER_BRIEFS[@]}"; do
  rc=0
  check_q1_file_existence "$brief" || rc=$?
  violations=$((violations + rc))

  rc=0
  check_q2_anchor_existence "$brief" || rc=$?
  violations=$((violations + rc))

  rc=0
  check_q4_landed_pr_in_history "$brief" || rc=$?
  violations=$((violations + rc))
done

rc=0
check_q5_cross_brief_projections || rc=$?
violations=$((violations + rc))

if [ "$violations" -gt 0 ]; then
  echo ""
  echo "Manager-brief authority check FAILED: $violations violation(s) found."
  echo ""
  echo "Each violation is a claim in a manager brief that does not match"
  echo "live repo state. Either:"
  echo "  - the cited authority/PR/anchor genuinely doesn't exist → fix"
  echo "    the citation in the brief"
  echo "  - the canonical projection (manager count, lane count) drifted"
  echo "    → update the brief to match the authority doc"
  echo "  - the script's pattern set is wrong → update MANAGER_BRIEFS or"
  echo "    the patterns array with the same review attention as adding"
  echo "    a new authority claim"
  echo ""
  echo "Authority: gpt-5-5-pro meta-review on PR #1126;"
  echo "docs/r2-structure.md §v2-guardrail-requirement-3"
  exit 1
fi

echo "Manager-brief authority check passed: 7 briefs, no violations."
echo "  Q1 (file existence): all cited paths resolve"
echo "  Q2 (anchor existence): all path#anchor links match a heading"
echo "  Q4 (LANDED via #N): all cited PRs reachable from branch"
echo "  Q5 (cross-brief projection): manager/lane counts consistent"
