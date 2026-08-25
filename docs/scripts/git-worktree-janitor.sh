#!/bin/bash
# git-worktree-janitor — reap orphaned git worktree checkouts and stale admin entries
# on the srv1/srv2 fleet.
#
# THE LOAD-BEARING DISTINCTION (do not "simplify" this away):
#
#   Class A  gitdir -> a HOST path that no longer exists.  Genuine garbage. Reap.
#   Class B  gitdir -> a CONTAINER-NAMESPACE path (/session-home/...).  VALID AND LIVE.
#            It only *looks* dangling from the host because the host has no such mount
#            point. The review containers bind-mount
#                /home/briansrls/.local/share  ->  /session-home/.local/share
#            i.e. the SAME directory under two paths. Reaping Class B destroys in-flight
#            code reviews.
#
# A host-only "does the target exist?" test cannot tell A from B. That exact test
# misclassified 379 live review worktrees on 2026-08-25 (restored; verified by running
# git inside a container). Hence CONTAINER_PREFIXES: any gitdir under one of those is
# skipped unconditionally — never reaped, never pruned.
#
# Likewise `git worktree prune` is NEVER run against a repo holding any container-scoped
# worktree: from the host every one of its entries looks missing, so a single prune would
# delete the entire live set at once.
#
# Git behaviours this encodes, learned the hard way:
#   - bare `worktree prune` silently skips entries inside a ~3-month grace window
#     (gc.worktreePruneExpire) -> needs --expire=now
#   - `worktree prune` refuses LOCKED entries; a `worktree add` that dies mid-init leaves
#     locked="initializing" + a stale index.lock -> unlock first, but only when the
#     checkout is provably gone
#   - `git worktree repair` does NOT help once the admin dir is gone (re-add or remove)
#
# Usage:
#   git-worktree-janitor              dry run (default; prints WOULD-*)
#   git-worktree-janitor --apply      act
#   git-worktree-janitor --self-test  prove Class A/B discrimination, then exit

set -uo pipefail

CONTAINER_PREFIXES=(/session-home)
SCAN_ROOTS=(/home/briansrls /opt /tmp)
MAXDEPTH=8
# Prune scan must reach the review workspace repo (depth 5 under $HOME) so the
# container-scoped guard is genuinely exercised rather than merely out of range.
PRUNE_MAXDEPTH=8
QUARANTINE_BASE="$HOME/.worktree-janitor-quarantine"
LOG="$HOME/.worktree-janitor.log"
LOCK="$HOME/.worktree-janitor.lock"
LOG_MAX_LINES=5000
# Never rescan our own quarantines (full of intentionally-broken stubs) or object storage.
EXCLUDE_RE="/\.worktree-janitor-quarantine/|/\.stub-quarantine-|/\.git/objects/"

MODE=dry
case "${1:-}" in
  --apply)     MODE=apply ;;
  --self-test) MODE=selftest ;;
  ""|--dry-run) MODE=dry ;;
  *) echo "usage: $(basename "$0") [--apply|--self-test]" >&2; exit 2 ;;
esac

ts()  { date -u +%Y-%m-%dT%H:%M:%SZ; }
log() { printf '%s %s\n' "$(ts)" "$*" | tee -a "$LOG"; }

is_container_path() {
  local p="$1" pre
  for pre in "${CONTAINER_PREFIXES[@]}"; do
    case "$p" in "$pre"/*|"$pre") return 0 ;; esac
  done
  return 1
}

# Resolve a .git pointer file to an absolute gitdir, or empty if not a pointer.
resolve_gitdir() {
  local gitfile="$1" d target
  d=$(dirname "$gitfile")
  target=$(sed -n 's/^gitdir: //p' "$gitfile" 2>/dev/null)
  [ -z "$target" ] && return 1
  case "$target" in
    /*) printf '%s' "$target" ;;
    *)  (cd "$d" 2>/dev/null && readlink -m "$target") ;;
  esac
}

# ------------------------------------------------------------------ self-test
# Discriminating control: a genuine orphan MUST be flagged, a container-namespace
# stub MUST NOT be. Both targets are absent on the host, so only the namespace
# rule can separate them.
if [ "$MODE" = selftest ]; then
  T=$(mktemp -d)
  trap 'rm -rf "$T"' EXIT
  mkdir -p "$T/genuine" "$T/containerish"
  echo "gitdir: $T/NOPE/.git/worktrees/a" > "$T/genuine/.git"
  echo "gitdir: /session-home/.local/share/gunbc-review/workspace/.git/worktrees/b" \
       > "$T/containerish/.git"

  a=$(resolve_gitdir "$T/genuine/.git");     b=$(resolve_gitdir "$T/containerish/.git")
  rc=0
  if is_container_path "$a"; then echo "FAIL: genuine orphan treated as container path"; rc=1
  elif [ -e "$a" ];          then echo "FAIL: self-test fixture target unexpectedly exists"; rc=1
  else                             echo "PASS: genuine orphan would be reaped"; fi
  if is_container_path "$b"; then echo "PASS: container stub protected"
  else                             echo "FAIL: container stub would be REAPED (destroys live reviews)"; rc=1; fi
  exit $rc
fi

# ------------------------------------------------------- single-instance guard
# cron and a human running --apply concurrently would race over the same moves.
exec 9>"$LOCK"
if ! flock -n 9; then
  log "SKIP: another janitor run holds $LOCK"
  exit 0
fi

APPLY=0; [ "$MODE" = apply ] && APPLY=1
n_reap=0; n_container=0; n_ok=0; n_pruned=0; n_skipped_repo=0

log "=== janitor start (apply=$APPLY) on $(hostname) ==="

# ---------------------------------------------------------------- Class A reap
QDIR="$QUARANTINE_BASE/$(date -u +%Y%m%d-%H%M%S)"
MANIFEST="$QDIR/MANIFEST.txt"

while IFS= read -r gitfile; do
  d=$(dirname "$gitfile")
  abs=$(resolve_gitdir "$gitfile") || continue
  [ -z "$abs" ] && continue

  if is_container_path "$abs"; then
    n_container=$((n_container+1)); continue      # Class B — hands off
  fi
  if [ -e "$abs" ]; then
    n_ok=$((n_ok+1)); continue                    # healthy
  fi

  n_reap=$((n_reap+1))                            # Class A — genuine orphan
  if [ "$APPLY" -eq 1 ]; then
    mkdir -p "$QDIR"
    echo "$d" >> "$MANIFEST"
    dest="$QDIR/$(echo "$d" | sed 's#^/##; s#/#__#g')"
    if mv "$d" "$dest" 2>/dev/null; then
      log "REAPED $d (gitdir gone: $abs)"
    else
      log "REAP-FAILED $d"
    fi
  else
    log "WOULD-REAP $d (gitdir gone: $abs)"
  fi
done < <(find "${SCAN_ROOTS[@]}" -maxdepth "$MAXDEPTH" -name .git -type f 2>/dev/null \
         | grep -Ev "$EXCLUDE_RE")

# ------------------------------------------------- stale admin entries (prune)
# Only for repos with NO container-scoped worktrees. See header.
while IFS= read -r gitdir; do
  repo=$(dirname "$gitdir")
  git -C "$repo" rev-parse --git-dir >/dev/null 2>&1 || continue

  has_container=0
  while IFS= read -r wt; do
    is_container_path "$wt" && { has_container=1; break; }
  done < <(git -C "$repo" worktree list --porcelain 2>/dev/null | awk '/^worktree /{print $2}')

  if [ "$has_container" -eq 1 ]; then
    n_skipped_repo=$((n_skipped_repo+1))
    log "PRUNE-SKIP $repo (has container-scoped worktrees)"
    continue
  fi

  # Unlock only entries whose checkout is provably gone, else prune refuses them.
  while IFS= read -r wt; do
    [ -n "$wt" ] && [ ! -e "$wt" ] && git -C "$repo" worktree unlock "$wt" >/dev/null 2>&1
  done < <(git -C "$repo" worktree list --porcelain 2>/dev/null | awk '/^worktree /{print $2}')

  if [ "$APPLY" -eq 1 ]; then
    out=$(git -C "$repo" worktree prune -v --expire=now 2>&1)
    [ -n "$out" ] && { n_pruned=$((n_pruned+1)); log "PRUNED $repo: $(echo "$out" | tr '\n' ';')"; }
  else
    out=$(git -C "$repo" worktree prune -v -n --expire=now 2>&1)
    [ -n "$out" ] && { n_pruned=$((n_pruned+1)); log "WOULD-PRUNE $repo: $(echo "$out" | tr '\n' ';')"; }
  fi
done < <(find "${SCAN_ROOTS[@]}" -maxdepth "$PRUNE_MAXDEPTH" -name .git -type d 2>/dev/null \
         | grep -Ev "$EXCLUDE_RE")

log "=== done: reap=$n_reap container-skipped=$n_container healthy=$n_ok repos-pruned=$n_pruned repos-prune-skipped=$n_skipped_repo ==="
[ "$APPLY" -eq 1 ] && [ -d "$QDIR" ] && log "quarantine: $QDIR"

# keep the log bounded (it is append-only and runs daily)
if [ -f "$LOG" ] && [ "$(wc -l < "$LOG")" -gt "$LOG_MAX_LINES" ]; then
  tail -n "$LOG_MAX_LINES" "$LOG" > "$LOG.tmp" && mv "$LOG.tmp" "$LOG"
fi
exit 0
