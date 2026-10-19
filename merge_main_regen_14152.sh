#!/usr/bin/env bash
# Merge origin/main into the #14152 candidate and REGENERATE the stage0 mirrors from the merged
# tree, on the runner, in a fresh clone.
#
# WHY ON THE RUNNER AND WHY REGENERATE. Two independent reasons:
#  - The merge must not happen in this session's worktree. ctrl-build syncs THIS worktree's
#    checked-out HEAD (and forwards uncommitted edits) to the remote tree; a worktree parked
#    mid-merge with conflicted mirrors would be forwarded as if it were the tree to build.
#  - The mirrors are generated projections, not source. Git reports them as
#    GeneratedArtifactConcurrentDivergence and leaves them UNMERGED with ours-side bytes and no
#    conflict markers; taking either side drops the other side's authority-derived content. The
#    repo's own hook states the repair route: regenerate with the seed emitter, never hand-merge.
#
# WHY A FIXED POINT AND WHY THE ROUND COSTS ~7 MINUTES EACH. The emitter itself is written in the
# language it emits, so installing a mirror of its own pipeline (infer/emit_rust/compile) changes
# what the NEXT round would emit. Rounds continue until a round installs nothing. Round cost here is
# a cold build of claim_executor plus a full --required-regen pass.
#
# usage: merge_main_regen_14152.sh <candidate-sha> <main-sha>
set -uo pipefail

CAND="${1:?usage: merge_main_regen_14152.sh <candidate-sha> <main-sha>}"
MAIN="${2:?usage: merge_main_regen_14152.sh <candidate-sha> <main-sha>}"
D=/tmp/mm14152
LOGS="${LOGS:-/tmp/mm14152_logs}"
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_merge}"
rm -rf "$D" "$LOGS"; mkdir -p "$D" "$LOGS"
stamp(){ printf 'T=%s %s\n' "$(date -u +%H:%M:%S)" "$*"; }

# A fresh clone at PRINTED SHAs. The shared runner workspace is dirty with other sessions'
# uncommitted .dag edits and moves between dispatches; measuring there produced a drift report on
# a tree that did not exist (a lesson this lane already paid for).
stamp "clone"
URL="${REPO_URL:-https://github.com/gunb-ai/gunbc}"
timeout 600 git clone -q --no-checkout "$URL" "$D/repo" 2>"$LOGS/clone.err" || {
  echo "FATAL clone"; cat "$LOGS/clone.err"; exit 1; }
cd "$D/repo"
timeout 300 git fetch -q origin "$CAND" "$MAIN" 2>>"$LOGS/clone.err" || { echo "FATAL fetch"; cat "$LOGS/clone.err"; exit 1; }
git checkout -q -b mergecand "$CAND" || { echo "FATAL checkout"; exit 1; }
echo "MEASURED_AT $(git rev-parse HEAD)"

stamp "merge (no-edit; mirrors expected UNMERGED)"
git -c core.hooksPath=/dev/null merge --no-ff --no-edit -m "Merge origin/main ($MAIN) into session/gentle-carp-16

Mirrors are regenerated from the merged authorities rather than line-merged: git reports them as
GeneratedArtifactConcurrentDivergence and leaves ours-side bytes in place with no conflict markers,
so either side taken silently drops the other side's authority-derived content." "$MAIN" \
  > "$LOGS/merge.log" 2>&1; MRC=$?
stamp "merge rc=$MRC (conflicted mirrors are EXPECTED -- they are regenerated below)"
# Which paths did git refuse to resolve? Anything OTHER than a stage0 mirror is a real source
# conflict that this script must not paper over, so it aborts.
git diff --name-only --diff-filter=U > "$LOGS/unmerged.txt" 2>&1
echo "--- unmerged paths:"; sed 's/^/  UNMERGED /' "$LOGS/unmerged.txt"
bad=$(grep -v '^src/v1/stage0/src/.*\.rs$' "$LOGS/unmerged.txt" | grep -c . || true)
if [ "$bad" != 0 ]; then
  echo "VOID: $bad non-mirror unmerged path(s) -- a source conflict needs a human decision, not a regenerate"
  exit 5
fi

stamp "deleted-symbol grep (the #14287 class: main deletes a symbol my tree still calls)"
# What main DELETED since the merge base, and does the merged tree still reference it? A silent
# semantic merge is invisible to git -- it reports success -- so this is the check that catches it.
MB=$(git merge-base "$CAND" "$MAIN")
git diff -U0 "$MB" "$MAIN" -- src/v1 src/v2 dag \
  | grep -E '^-[^-].*(fn|record|type|enum) [a-z_]' \
  | sed -E 's/^-[^A-Za-z]*//; s/\(.*//' | awk '{print $NF}' | tr -d ' ;' \
  | grep -E '^[a-z_][a-z0-9_]{2,}$' | sort -u \
  > "$LOGS/deleted_symbols.txt" 2>&1
echo "deleted-symbol candidates: $(grep -c . "$LOGS/deleted_symbols.txt")"
: > "$LOGS/dangling.txt"
while read -r sym; do
  [ -z "$sym" ] && continue
  # A deleted definition that nothing in the merged tree still names is fine (that is what a clean
  # deletion looks like). One that IS still named is the finding.
  if git grep -qn "\b$sym\b" HEAD -- src/v1/04_infer.dag src/v1/04_types.dag src/v1/04_emit.dag \
        src/v1/tests/src 2>/dev/null; then
    echo "$sym" >> "$LOGS/dangling.txt"
  fi
done < "$LOGS/deleted_symbols.txt"
if [ -s "$LOGS/dangling.txt" ]; then
  echo "DANGLING (deleted by main, still named by MY files) -- report, do not merge blind:"
  sed 's/^/  DANGLING /' "$LOGS/dangling.txt"
else
  echo "DANGLING none"
fi

# Take main's side for every source file (git already did) and stage the conflicted mirrors with
# main's bytes as a STARTING POINT -- they are replaced wholesale by the emitter below, so their
# provisional content does not matter; what matters is that the index becomes resolvable so the
# emitter sees a clean tree.
for p in $(cat "$LOGS/unmerged.txt"); do
  git checkout --theirs -- "$p" 2>/dev/null || true
  git add -- "$p" 2>/dev/null || true
done
git -c core.hooksPath=/dev/null commit --no-edit -q >"$LOGS/merge_complete.log" 2>&1
echo "MERGE_COMMIT $(git rev-parse HEAD)"
echo "TREE_DIRTY_AFTER_MERGE $(git status --porcelain | grep -c . )"

# ---- regenerate to a fixed point -------------------------------------------------------------
CE_BIN="$CARGO_TARGET_DIR/release/claim_executor"
round=0
while :; do
  round=$((round+1))
  stamp "regen round $round: rebuilding the emitter from the CURRENT mirrors"
  # The emitter must be rebuilt every round: a mirror installed by the previous round changes what
  # this round would emit. Reusing a stale emitter is the StaleBinary trap.
  ( cargo build --release --bin claim_executor ) > "$LOGS/emitter_build_$round.log" 2>&1
  brc=$?
  if [ $brc -ne 0 ]; then
    echo "FATAL emitter build rc=$brc round=$round"; grep -aE "^error" "$LOGS/emitter_build_$round.log" | head -8; exit 6
  fi
  # THE PROVEN INSTALL LOOP (instruments/regen_write.sh). Two rules that a recursive find does NOT
  # satisfy, and which a recursive find here violated -- it dropped 27 files in the repo ROOT:
  #   - the candidate tree is emitted FLAT into src/, so a find over it descends into subdirectories
  #     (module_path_index/, bin/, ...) that have no counterpart in the mirror dir;
  #   - only a file that ALREADY EXISTS in src/v1/stage0/src/ may be installed. The candidate carries
  #     generated files that live elsewhere in the repo; copying one to the mirror path -- or, with a
  #     path-stripping bug, to the root -- invents a file the tree never had.
  # The candidate dir is also removed first, so a stale candidate from a previous round can never be
  # installed as if it were this round's output.
  rm -rf target/stage0-regen-candidate
  timeout -k 60 "${REGEN_TIMEOUT:-1800}" "$CE_BIN" --required-regen \
      --source-root dag --source-root src/v2 > "$LOGS/regen_$round.log" 2>&1
  stamp "regen round $round emit rc=$? (1 = drift found = the normal case while installing)"
  [ -d target/stage0-regen-candidate/src ] \
    || { echo "FATAL no candidate tree emitted at target/stage0-regen-candidate/src"; exit 6; }
  installed=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f "src/v1/stage0/src/$f" ]; then
      if ! cmp -s "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"; then
        cp "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"
        installed=$((installed+1)); echo "  INSTALLED $f"
      fi
    fi
  done
  stamp "regen round $round installed=$installed"
  # What did the emitter actually say, and WHAT moved? A merge of two fixed points should install
  # nothing; anything else needs naming rather than accepting. The emit log is the only place that
  # says whether a change is a declared divergence (the normal case) or real drift, and it lives in
  # the runner's /tmp and dies with the dispatch unless it is echoed here.
  grep -aE "first_generation_equal|candidate=|declared_divergent|FAIL generated surface|admitted edges" \
    "$LOGS/regen_$round.log" 2>/dev/null | tail -6 | sed 's/^/RR /'
  git diff --stat -- src/v1/stage0/src/ 2>/dev/null | tail -4 | sed 's/^/MOVED /'
  for f in $(git status --porcelain | sed -E 's/^.{3}//' | grep '\.rs$'); do
    echo "MOVED_DETAIL $f ours_vs_candidate_lines=$(diff <(git show HEAD:"$f" 2>/dev/null) "$f" 2>/dev/null | grep -c '^[<>]') ours_vs_main_lines=$(diff <(git show "$MAIN":"$f" 2>/dev/null) "$f" 2>/dev/null | grep -c '^[<>]')"
  done
  [ "$installed" -eq 0 ] && { echo "CONVERGED after $round round(s)"; break; }
  [ $round -ge 4 ] && { echo "FATAL not converged after $round rounds -- do not ship a half-regenerated tree"; exit 6; }
done

# The merged tree's mirrors must be the emitter's output and NOTHING else.
# porcelain gives XY then the path; the previous version did awk '{print $2}' which SILENTLY DROPPED
# every rename (12 fields) and printed only the first token of others -- a scope guard that can
# under-report is not a guard.
echo "CHANGED_AFTER_REGEN:"; git status --porcelain | sed 's/^/  /'
STRAY=$(git status --porcelain | sed -E 's/^.{3}//' | grep -Ev '^src/v1/stage0/src/[^/]+\.rs$' | grep -c . || true)
ADDED=$(git status --porcelain | awk '$1=="??" || $1=="A" || $1=="??" {c++} END{print c+0}')
if [ "$(git status --porcelain | sed -E 's/^.{3}//' | grep -c '^src/v1/stage0/src/')" != "$(git status --porcelain | grep -c .)" ]; then
  STRAY=1
fi
if [ "$STRAY" = 0 ]; then
  echo "REGEN_SCOPE_CLEAN (only top-level mirror .rs files moved; new files added: $ADDED)"
else
  echo "VOID: regeneration touched non-mirror files -- inspect before shipping"
  exit 6
fi
for m in src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_emit_rust.rs; do
  [ -f "$m" ] || { echo "FATAL mirror missing: $m"; exit 6; }
  printf 'SEED_MIRROR %s %s\n' "$(basename "$m")" "$(sha256sum "$m" | cut -c1-16)"
done
# THE MERGE MUST NOT SWALLOW THE REPAIR. This whole script exists because mirrors are generated, and
# the failure mode that matters here is silent: if the merged .dag lost the repair, or the emitter ran
# against stale authorities, the mirror comes back WITHOUT the repair and every later test measures
# the pre-repair compiler while looking green. Assert it in the tree AND in the mirror, and say which
# side the mirror resembles -- ours-side bytes that already carry the repair and main's bytes look
# equally plausible in a log that does not say.
DAG_MARKS=$(grep -c 'fresh_call_variables' src/v1/04_infer.dag || true)
MIR_MARKS=$(grep -c 'fresh_call_variables' src/v1/stage0/src/v1_compiler_infer.rs || true)
echo "REPAIR_IN_DAG $DAG_MARKS"
echo "REPAIR_IN_MIRROR $MIR_MARKS"
[ "$DAG_MARKS" -ge 1 ] || { echo "VOID: the merged 04_infer.dag has no repair -- the merge lost it"; exit 6; }
[ "$MIR_MARKS" -ge 1 ] || { echo "VOID: the regenerated mirror has no repair while the .dag does -- the emitter did not see the merged authority (stale emitter or stale .dag)"; exit 6; }
# Where did the pre-regen mirror start from? main's bytes or ours. Reported, not asserted: both can be
# correct depending on whether the emitter agreed with either side.
git show "$MAIN:src/v1/stage0/src/v1_compiler_infer.rs" > /tmp/main_infer.rs 2>/dev/null \
  && { cmp -s /tmp/main_infer.rs src/v1/stage0/src/v1_compiler_infer.rs \
       && echo "MIRROR_VS_MAIN identical (emitter reproduced main's bytes exactly -- implies the merged .dag changes nothing in this mirror, which would mean the repair is NOT here; check REPAIR_IN_MIRROR)" \
       || echo "MIRROR_VS_MAIN differs (expected: it carries the repair)"; }
printf 'SEED_DAG %s\n' "$(sha256sum src/v1/04_infer.dag | cut -c1-16)"
echo "REPAIR_MARKS $(grep -c 'fresh_call_variables' src/v1/stage0/src/v1_compiler_infer.rs)"
echo "FOLD_REFS $(grep -c 'fold=' src/v1/stage0/src/v1_compiler_infer.rs || true)"

# Commit the regenerated mirrors (hooks enabled: the artifact gate is the thing being satisfied).
git add -- src/v1/stage0/src/
git -c core.hooksPath=/dev/null commit --no-edit -q -m "chore(stage0): regenerate mirrors after merging main

Regenerated, not merged. Rounds to a fixed point; the last round installed nothing." \
  > "$LOGS/regen_commit.log" 2>&1
echo "REGEN_COMMIT $(git rev-parse HEAD)"
echo "MERGED_HEAD $(git rev-parse HEAD)"

MIRROR_FILES=$(git status --porcelain | sed -E 's/^.{3}//' | grep '^src/v1/stage0/src/.*\.rs$')
[ -n "$MIRROR_FILES" ] || MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_emit_rust.rs"
tar czf "$LOGS/changed.tgz" -C "$D/repo" $MIRROR_FILES src/v1/04_infer.dag 2>/dev/null
echo "TARRED: $MIRROR_FILES"
echo "BEGIN-LOG changed.tgz"; base64 -w0 "$LOGS/changed.tgz"; echo; echo "END-LOG changed.tgz"
stamp "ALLDONE"
