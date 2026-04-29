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
# Q3 (controlled status vocabulary) deferred to v2 with concrete
# dissolution trigger: implement Q3 when ANY status-string drift
# slips past Q1/Q2/Q4/Q5 in a real review (i.e., a brief edit
# introduces an unrecognized status keyword that none of the
# existing checks catch). At that point Q3 has a known unhandled
# class to encode against. Until then, status vocabulary is
# captured indirectly by Q5 (count-projection consistency catches
# the most common drift class — wrong manager/lane counts).
# DISSOLUTION TRIGGER: first reviewer-flagged status-string drift
# class that Q1/Q2/Q4/Q5 don't catch.
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
#
# Known limitation (per claude-opus-4-7 review on 91b5274fc): every
# `](path)` is treated as a filesystem reference. Markdown
# reference-style link definitions (`[ref]: path`) and code-block
# examples containing `](foo)` would false-positive. Currently no
# briefs use either form. DISSOLUTION TRIGGER: first false-positive
# from a brief that introduces a code-block example or reference-style
# link, at which point Q1 needs context-aware extraction (skip lines
# inside fenced code blocks; skip reference definitions).

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
# Q2 — Cited section anchor existence (markdown links + prose §)
# ---------------------------------------------------------------------
# Two forms covered (per gpt-5-5-pro review on 91b5274fc; deferring
# prose form left load-bearing INVARIANTS/r2-structure/design
# authority claims uncheckable):
#
# Form A — markdown `path#anchor`: verify the anchor matches a heading
# in the target file (heading slugified to GitHub form: lowercase,
# spaces→dashes, punctuation stripped).
#
# Form B — prose `§"section name"` or `§AnchorToken`: identify the
# cited file via the most recent markdown link / bare `.md` reference
# on the same line, then grep -F for the section text in the target.
# Permissive vs heading-only Form A — accepts the section text
# appearing anywhere in the file (since prose citations may use
# paraphrased section names) — but still catches the "section deleted"
# class, which is the load-bearing failure mode.

slugify() {
  echo "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9 -]//g; s/  */-/g; s/--*/-/g; s/^-//; s/-$//'
}

# Strip leading "# " / "## " / ... markdown heading marker from a line
# and return the heading text. Don't use Bash's `##` glob — it's
# greedy through the LAST space, so "## Goal 7 — Evaluator XL"
# becomes "XL" instead of "Goal 7 — Evaluator XL". sed regex with
# explicit hash-count + whitespace bound avoids that.
strip_heading_marker() {
  echo "$1" | sed -E 's/^#{1,6}[[:space:]]+//'
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

    # Extract all heading slugs from target. Use strip_heading_marker
    # (sed-based) — Bash's `##\#* ` glob is greedy through the LAST
    # space, dropping multi-word heading text.
    local found=0
    while IFS= read -r heading; do
      heading="${heading#"${heading%%[![:space:]]*}"}"  # ltrim
      heading="$(strip_heading_marker "$heading")"
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

# Resolve a bare filename like "INVARIANTS.md" or "r2-structure.md" to
# a filesystem path by trying known authority-doc locations.
# Returns empty string if unresolvable.
resolve_authority_file() {
  local name="$1"
  local candidates=(
    "$ROOT/$name"
    "$ROOT/docs/$name"
    "$ROOT/docs/thesis/$name"
    "$ROOT/docs/briefs/$name"
  )
  for candidate in "${candidates[@]}"; do
    if [ -f "$candidate" ]; then
      echo "$candidate"
      return
    fi
  done
  echo ""
}

# Q2-prose: verify §"section" / §Anchor citations resolve in the
# most-recently-cited file on the same line.
check_q2_prose_section_existence() {
  local brief="$1"
  local brief_violations=0

  # Read brief line-by-line so we can pair each § citation with the
  # cited file from the same line.
  while IFS= read -r line || [ -n "$line" ]; do
    # Skip lines without § citations.
    case "$line" in
      *§*) ;;
      *) continue ;;
    esac

    # Process § citations in order, advancing a running prefix so that
    # the Nth citation's prefix is everything up to (but not including)
    # the Nth §. Each citation pairs with the most recent markdown
    # link or bare `.md` token in its OWN prefix, not the global
    # before-first-§ prefix.
    local remaining="$line"

    # Find every §-citation in the line. Two forms:
    #   §"quoted section text"
    #   §AnchorToken (alphanumeric-leading; hyphens, dots, no spaces —
    #                 anchors, P1/P5/Q6/Q8 tokens, "v2-guardrail-
    #                 requirement-3", numeric "§4"/"§6a"/"§0.7", etc.)
    # Per gpt-5-5-pro review on b9f7a1c1: digit-leading tokens
    # (`§4`, `§6a`, `§0.7`) were silently skipped by the prior regex
    # which required a leading [A-Za-z]. Now `[A-Za-z0-9]`-leading.
    # Note: short digit-only tokens like `§4` resolve permissively
    # (substring match against bare "4" always finds something) —
    # tracked as known limitation; multi-character tokens like
    # `§6a`/`§0.7` are discriminating.
    while IFS= read -r citation; do
      [ -z "$citation" ] && continue

      local section_text="$citation"
      # Strip leading § and quotes.
      section_text="${section_text#§}"
      section_text="${section_text#\"}"
      section_text="${section_text%\"}"

      # Compute the prefix for THIS citation: everything in `remaining`
      # before the next occurrence of this citation, then advance
      # `remaining` past it for the next iteration.
      local prefix="${remaining%%${citation}*}"
      # Advance remaining past the citation.
      remaining="${remaining#*${citation}}"

      # Identify the cited file: scan the prefix backward for either:
      #   1. A markdown link [text](path) — extract path.
      #   2. A bare `<NAME>.md` token — try resolving via known
      #      authority-doc locations.
      local cited_file=""

      # Try markdown link form: extract last [...](path).
      local md_path
      md_path="$(echo "$prefix" | grep -oE '\]\([^)]+\)' | tail -1 | sed -E 's/^\]\(//; s/\)$//')"
      if [ -n "$md_path" ]; then
        # Strip anchor portion if present.
        md_path="${md_path%%#*}"
        case "$md_path" in
          http://*|https://*|mailto:*) md_path="" ;;
        esac
        if [ -n "$md_path" ]; then
          local brief_dir
          brief_dir="$(dirname "$brief")"
          if [[ "$md_path" = /* ]]; then
            cited_file="$ROOT$md_path"
          else
            cited_file="$brief_dir/$md_path"
          fi
          if command -v realpath >/dev/null 2>&1; then
            cited_file="$(realpath -m "$cited_file" 2>/dev/null || echo "$cited_file")"
          fi
        fi
      fi

      # Fallback: bare `<NAME>.md` token. Look for the last
      # whitespace-or-backtick-bounded `*.md` in the prefix.
      if [ -z "$cited_file" ] || [ ! -f "$cited_file" ]; then
        local bare_md
        bare_md="$(echo "$prefix" | grep -oE '[A-Za-z][A-Za-z0-9_./-]*\.md' | tail -1)"
        if [ -n "$bare_md" ]; then
          # Strip leading directory if present (e.g., docs/r2-structure.md)
          local resolved
          resolved="$(resolve_authority_file "$(basename "$bare_md")")"
          if [ -n "$resolved" ]; then
            cited_file="$resolved"
          fi
        fi
      fi

      # If we couldn't pair the § citation with a cited file, skip
      # silently — ambiguous prose; don't false-positive.
      [ -z "$cited_file" ] && continue
      [ ! -f "$cited_file" ] && continue

      # Verify the section text appears in the cited file.
      # Permissive match: grep -F (literal substring), not slug match.
      # This catches the load-bearing class ("section deleted") while
      # tolerating paraphrased prose citations.
      if ! grep -F -q "$section_text" "$cited_file" 2>/dev/null; then
        echo "VIOLATION [Q2 prose-section-existence]: $brief"
        echo "  prose citation: §\"$section_text\""
        echo "  resolved cited file: $cited_file"
        echo "  section text not found in target file"
        brief_violations=$((brief_violations + 1))
      fi
    done < <(echo "$line" | grep -oE '§"[^"]+"|§[A-Za-z0-9][A-Za-z0-9._-]*[A-Za-z0-9]|§[A-Za-z0-9]')
  done < "$brief"

  return $brief_violations
}

# ---------------------------------------------------------------------
# Q4 — LANDED via #N PR-reachability check
# ---------------------------------------------------------------------
# Every `LANDED via #N`, `landed via #N`, or `Landed via #N` claim
# must correspond to a merge commit reachable from this branch's
# history. Case-insensitive extraction (per gpt-5-5-pro review on
# 91b5274f): live briefs use all three case forms — UPPERCASE for
# emphasized status-table claims, title-case for sentence-leading
# headings (e.g., r2-release-manager.md "Landed via #1078:"), and
# lowercase for inline prose.

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
      # mktemp avoids the $$-PID approach: a crashed run on a shared
      # dev box could leak /tmp/gh-stderr-$$ files; mktemp gives unique
      # path + cleanup is paired in this scope.
      local gh_stderr_file
      gh_stderr_file="$(mktemp)"
      pr_state="$(gh pr view "$pr_num" --repo "$REPO_SLUG" --json state --jq '.state' 2>"$gh_stderr_file" || true)"
      gh_stderr="$(cat "$gh_stderr_file" 2>/dev/null || true)"
      rm -f "$gh_stderr_file"
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
  done < <(grep -oEi 'landed via #[0-9]+' "$brief" | grep -oE '#[0-9]+' | tr -d '#' | sort -u)

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
  # Markdown bold (`**7**`) handling — per gpt-5-5-pro review on
  # ea33aeb9d: live briefs write counts with markdown emphasis
  # (e.g., "Names this manager one of **7** standing R2 managers").
  # Earlier patterns required a bare leading digit and so missed
  # every live count claim — the check passed silently because no
  # matches at all means "0 counts seen → silent OK" per the
  # mismatch logic below. Patterns now optionally accept "**" before
  # and after the digit; extraction strips asterisks before
  # parsing the integer.
  declare -a patterns=(
    'standing R2 managers||7||\*?\*?[0-9]+\*?\*? standing R2 managers'
    'standing managers (no R2 qualifier)||7||\*?\*?[0-9]+\*?\*? standing managers'
    'other managers||6||\*?\*?[0-9]+\*?\*? other managers'
    'R3 lanes||10||\*?\*?[0-9]+\*?\*? R3 lanes'
    'R3-Evaluator-gated lanes||7||\*?\*?[0-9]+\*?\*? of 10 R3 lanes'
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
        # Strip markdown bold asterisks first (live briefs use **N**).
        local n
        n="$(echo "$match" | tr -d '*' | grep -oE '^[0-9]+')"
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
  check_q2_prose_section_existence "$brief" || rc=$?
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
echo "  Q2 (markdown anchors): all path#anchor links match a heading"
echo "  Q2 (prose §): all §\"section\" / §Anchor citations resolve in cited files"
echo "  Q4 (LANDED via #N): all cited PRs reachable from branch"
echo "  Q5 (cross-brief projection): manager/lane counts consistent"
