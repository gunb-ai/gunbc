#!/usr/bin/env bash
# publish-snapshot.sh — export a clean public snapshot of the internal repo.
#
# Per RELEASE_TODO.md §2: builds a stripped worktree from the current HEAD,
# commits it as a single snapshot, and (with --publish) force-pushes to the
# public remote. The public repo is treated as a force-pushed mirror of the
# latest snapshot — no internal history travels with it.
#
# Target repo: gunb-ai/daglang (separate public repo; internal gunb-ai/gunbc
# stays unchanged). 'daglang' is the public language name; 'gunbc' remains
# the compiler binary name. Configure the remote once with:
#   git remote add public git@github.com:gunb-ai/daglang.git
#
# Defaults to dry-run (no push). To actually publish:
#   PUBLISH_CONFIRM=yes scripts/publish-snapshot.sh --publish
#
# Usage:
#   scripts/publish-snapshot.sh [--publish] [--remote NAME] [--branch NAME]
#                               [--export-dir PATH] [--keep-export]
#
# Defaults:
#   --remote public
#   --branch main
#   --export-dir /tmp/gunbc-pub-export
set -euo pipefail

REMOTE="public"
BRANCH="main"
EXPORT_DIR="/tmp/gunbc-pub-export"
PUBLISH=0
KEEP_EXPORT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --publish)      PUBLISH=1 ;;
    --remote)       REMOTE="$2"; shift ;;
    --branch)       BRANCH="$2"; shift ;;
    --export-dir)   EXPORT_DIR="$2"; shift ;;
    --keep-export)  KEEP_EXPORT=1 ;;
    -h|--help)      sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
  shift
done

# Run from the repo root.
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Refuse to run on a dirty tree — the snapshot must reflect a committed state.
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "ERROR: working tree is dirty. Commit or stash first." >&2
  exit 1
fi

SNAPSHOT_SHA="$(git rev-parse --short HEAD)"
SNAPSHOT_REF="$(git rev-parse HEAD)"
SNAPSHOT_BRANCH="snapshot-${SNAPSHOT_SHA}"

# Verify the public remote exists when we intend to publish.
if [[ "$PUBLISH" -eq 1 ]]; then
  if ! git remote get-url "$REMOTE" >/dev/null 2>&1; then
    echo "ERROR: git remote '$REMOTE' is not configured." >&2
    echo "  Add it with: git remote add $REMOTE <public-repo-url>" >&2
    exit 1
  fi
  if [[ "${PUBLISH_CONFIRM:-}" != "yes" ]]; then
    echo "ERROR: --publish requires PUBLISH_CONFIRM=yes in the environment." >&2
    echo "  Force-push to '$REMOTE/$BRANCH' is destructive and rewrites public history." >&2
    exit 1
  fi
fi

# Strip-list per RELEASE_TODO §2. Paths are relative to the export worktree
# root. Adding to this list is the normal way to extend the snapshot policy.
STRIP_PATHS=(
  # Maintainer-facing planning docs (not for public).
  "RELEASE_TODO.md"
  "WISHLIST.md"
  "docs/RELEASE_v0.1.0.md"

  # Internal docs (agent briefs, history, debt, audits, proposals, perf, decisions)
  "docs/briefs"
  "docs/history"
  "docs/debt"
  "docs/review-findings"
  "docs/admin"
  "docs/db-history"
  "docs/postmortems"
  "docs/audit"
  "docs/r3"
  "docs/proposals"
  "docs/perf"
  "docs/decisions"

  # Internal process docs at docs/ root (design DBs, planning, rung specs, modeling).
  "docs/design-*.md"
  "docs/planning/"
  "docs/r3-*.md"
  "docs/r4-*.md"
  "docs/regroup-*.md"
  "docs/v4-*.md"
  "docs/modeling/"

  # v3 + v4 substrate SHIP public in v0.1.0 labeled alpha / WIP per
  # the D-REL-1 (iv) flip (2026-05-30, docs/RELEASE_v0.1.0.md). They are
  # not on the supported contract; SUPPORTED.md per-surface labels what
  # is/isn't claimed. This supersedes both the earlier "strip src/v3"
  # legacy and the previous "strip src/v4 wholesale" ruling, and
  # overrides the older RELEASE_TODO.md §6 housecleaning notes.
  #
  # Per the (iv) reconciliation rule ("process docs / agent traffic =
  # stripped; substrate in-progress = alpha-labeled"), the v4 *process*
  # markdown files (TASKS / BRIEF_TEMPLATE / CULTURE / DECISIONS) remain
  # stripped — they are agent-process traffic, not substrate.
  "src/v4/TASKS.md"
  "src/v4/BRIEF_TEMPLATE.md"
  "src/v4/CULTURE.md"
  "src/v4/DECISIONS.md"

  # v2 internal design docs.
  "src/v2/CM.md"
  "src/v2/CM-inventory.md"
  "src/v2/cx-violation-triage.md"

  # Work-in-progress and internal tooling.
  "wip"
  "RELEASE_TODO.md"
  "scripts/session-dashboard"
  "scripts/_internal"

  # Editor/agent metadata.
  ".cursor"
  "_internal"

  # Internal interp_test fixtures (not user-facing demo material).
  "dsl/examples/interp_test/rest_test.dag"
  "dsl/examples/interp_test/shell_test.dag"
)

# Clean any prior export dir/worktree.
if [[ -e "$EXPORT_DIR" ]]; then
  # If it's a registered worktree, prune it properly; otherwise rm.
  if git worktree list --porcelain | grep -q "worktree $EXPORT_DIR$"; then
    git worktree remove --force "$EXPORT_DIR"
  else
    rm -rf "$EXPORT_DIR"
  fi
fi

# Also drop any leftover snapshot-<sha> branch from a prior dry run on the
# same HEAD — the branch outlives the worktree, and the orphan checkout
# below would refuse to recreate it. Safe: snapshot branches are throwaway.
git branch -D "$SNAPSHOT_BRANCH" >/dev/null 2>&1 || true

# Build the snapshot in an isolated worktree based on HEAD.
git worktree add --detach "$EXPORT_DIR" "$SNAPSHOT_REF"

pushd "$EXPORT_DIR" >/dev/null

# Apply strip-list. Entries may be literal paths or globs; missing paths are ok.
shopt -s nullglob
for pattern in "${STRIP_PATHS[@]}"; do
  for path in $pattern; do
    rm -rf "$path"
    echo "stripped: $path"
  done
done
shopt -u nullglob

# Workspace members must match the stripped tree — otherwise `cargo fmt` and
# other metadata commands fail on missing manifests (public snapshot CI).
snapshot_patch_workspace_cargo() {
  local cargo_toml="Cargo.toml"
  if [[ ! -f "$cargo_toml" ]]; then
    echo "ERROR: missing $cargo_toml in export worktree" >&2
    return 1
  fi
  awk '
    /"src\/v3\// { next }
    { print }
  ' "$cargo_toml" > "${cargo_toml}.snapshot" \
    && mv "${cargo_toml}.snapshot" "$cargo_toml"
  echo "patched: $cargo_toml (removed workspace members absent from public snapshot)"
}

snapshot_patch_workspace_cargo

# Stage everything (including deletions) on a fresh orphan branch so the
# public history is a single root commit per snapshot — no internal SHAs leak.
git checkout --orphan "$SNAPSHOT_BRANCH"
git add -A
# Commit message must not embed the internal SHA — that would leak the
# private repo's identity into the public history (Boundary Discipline).
# A UTC date stamp is enough to identify the snapshot externally; the
# internal correspondence is recorded only in the publisher's own records,
# never in the published commit.
SNAPSHOT_LABEL="$(date -u +%Y-%m-%d)"
git -c user.name="gunbc-release" -c user.email="release@gunb.ai" \
    commit -m "snapshot ${SNAPSHOT_LABEL}"

EXPORT_SHA="$(git rev-parse HEAD)"

popd >/dev/null

# Release receipt: emit public-export-manifest.txt as a sibling of the export
# dir so it doesn't get committed into the public snapshot or matched by the
# leak-grep gate. Operators paste this into release notes / audit trails to
# record what shipped, what was stripped, and the SHA correspondence.
MANIFEST_PATH="$(dirname "$EXPORT_DIR")/public-export-manifest.txt"
MANIFEST_TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
  echo "# public-export-manifest"
  echo "timestamp_utc: ${MANIFEST_TIMESTAMP}"
  echo "snapshot_source_sha: ${SNAPSHOT_REF}"
  echo "export_sha: ${EXPORT_SHA}"
  echo "snapshot_branch: ${SNAPSHOT_BRANCH}"
  echo
  echo "## stripped_paths"
  for p in "${STRIP_PATHS[@]}"; do
    echo "${p}"
  done
  echo
  echo "## included_paths"
  git -C "$EXPORT_DIR" ls-files
} > "$MANIFEST_PATH"
echo "manifest: ${MANIFEST_PATH}"

# Leak-grep gate. Runs AFTER strip+commit against the exported tree — this
# defends the export, not the internal repo. Allowlist exempts dissolve-
# comment substrate provenance (sanctioned by the operator verdict on
# adhoc-e7966a73-c38) plus self-referential lines tagged leak-gate-self.
ALLOWLIST_REGEX='🟡|dissolve-target|dissolve-on-arrival|leak-gate-self'

LEAK_CONTENT_PATTERNS=(  # leak-gate-self
  'msg_[a-f0-9-]+'       # leak-gate-self
  'localhost:8787'       # leak-gate-self
  'dashboard-ops'        # leak-gate-self
  'dashboard-message'    # leak-gate-self
  'operator-[a-z]+'      # leak-gate-self
)

# Path patterns mirror STRIP_PATHS — any stripped path present in the export
# means strip-list failed to remove an internal-only path. Derived directly
# from STRIP_PATHS so the two stay in sync as the strip-list grows.

echo "leak-grep gate: scanning export..."
leak_fail=0
for pat in "${LEAK_CONTENT_PATTERNS[@]}"; do
  hits="$(git -C "$EXPORT_DIR" grep -E -n -e "$pat" 2>/dev/null || true)"
  if [[ -n "$hits" ]]; then
    real_hits="$(echo "$hits" | grep -E -v "$ALLOWLIST_REGEX" || true)"
    if [[ -n "$real_hits" ]]; then
      echo "LEAK: content pattern /$pat/ matched (after allowlist):" >&2
      echo "$real_hits" | head -20 >&2
      leak_fail=1
    fi
  fi
done
EXPORT_FILES="$(git -C "$EXPORT_DIR" ls-files)"
for p in "${STRIP_PATHS[@]}"; do
  # Match exact file or anything under the stripped prefix.
  pat_escaped="${p//./\\.}"
  hits="$(echo "$EXPORT_FILES" | grep -E "^${pat_escaped}(/|$)" || true)"
  if [[ -n "$hits" ]]; then
    echo "LEAK: stripped path '${p}' present in export (strip-list missed it):" >&2
    echo "$hits" | head -20 >&2
    leak_fail=1
  fi
done
if [[ "$leak_fail" -ne 0 ]]; then
  echo "ERROR: leak-grep gate failed; refusing to publish." >&2
  exit 1
fi
echo "leak-grep gate: PASS"

if [[ "$PUBLISH" -eq 1 ]]; then
  echo "force-pushing snapshot to ${REMOTE}/${BRANCH}..."
  git -C "$EXPORT_DIR" push --force "$REMOTE" "${SNAPSHOT_BRANCH}:${BRANCH}"
  echo "published: ${REMOTE}/${BRANCH} now points at snapshot from internal@${SNAPSHOT_SHA}"
else
  echo
  echo "DRY RUN: snapshot built at ${EXPORT_DIR} (branch ${SNAPSHOT_BRANCH})."
  echo "  Inspect with:   git -C ${EXPORT_DIR} log -1 --stat"
  echo "  Publish with:   PUBLISH_CONFIRM=yes $0 --publish"
fi

# Post-export defense-in-depth: re-grep the actually-shipped tree and verify
# it builds. Runs on both dry-run and real publish per the brief — on dry-run
# it validates whatever is currently published (the operator's pre-flight
# check against drift); on publish it validates what we just pushed.
SMOKE_SCRIPT="${REPO_ROOT}/_internal/scripts/public-clone-smoke.sh"
if [[ -x "$SMOKE_SCRIPT" ]]; then
  echo "running public-clone-smoke..."
  if ! "$SMOKE_SCRIPT"; then
    echo "ERROR: public-clone-smoke failed" >&2
    exit 1
  fi
fi

# Auto-remove the export only after a real publish. On dry-run we leave it
# in place so the export can be inspected via the command printed above —
# otherwise the "DRY RUN: inspect with…" instructions would race a
# silent teardown. Pass --keep-export to preserve it past a publish as well.
if [[ "$PUBLISH" -eq 1 && "$KEEP_EXPORT" -eq 0 ]]; then
  git worktree remove --force "$EXPORT_DIR"
fi
