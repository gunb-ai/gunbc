#!/usr/bin/env bash
# freeze2.sh — FREEZE-2 helper for packet-evidence-v16 (sleek-seal-226).
# Fires when #15992 has merged and main has been merged into PR #15999 (merge commit => head moves).
# Re-cuts SEED_A on the NEW head as ONE commit adding ONLY the probe file, with identical blobs,
# gates everything v14/v15-style, and PRINTS the packet/PRE-REG/description edits. Does NOT edit
# the frozen files itself (one reviewed delta principle).
#
# Usage: freeze2.sh <NEW_HEAD_SHA> [rehearse]      # 'rehearse' skips push and prints planned actions
set -euo pipefail
REPO=/home/briansrls/.worktrees/gunbc/sleek-seal-226
NEW_HEAD="${1:?usage: freeze2.sh <NEW_HEAD_SHA> [rehearse]}"
MODE="${2:-real}"

OLD_PARENT_A=a4a21da46284e64a31efff08585c42aaf589089a
SEED_A_OLD=d92fd54d1e2ff8b838246bb974715ce7d3f8cf29
BLOB_PROBE=$(grep -m1 '^BLOB_PROBE=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2)
BLOB_WC_A=$(grep -m1 '^BLOB_WC_A=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2 | cut -d\; -f1)
BLOB_RTP_A=$(grep -m1 '^BLOB_RTP_A=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2 | cut -d\; -f1)
for v in BLOB_PROBE BLOB_WC_A BLOB_RTP_A; do eval "len=\${#$v}"; [ "$len" = 40 ] || { echo "FATAL: $v not a full sha (len=$len)"; exit 1; }; done
# claim/probe paths are read FROM THE PACKET itself to avoid drift
P_WC=$(grep -m1 '^WC_PATH=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2)
P_RTP=$(grep -m1 '^RTP_PATH=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2)
P_PROBE=$(grep -m1 '^PROBE_PATH=' /tmp/packet/packet-evidence-v16.sh | cut -d= -f2)
[ -n "$P_PROBE" ] || { echo "FATAL: PROBE_PATH not found in packet"; exit 1; }

cd "$REPO"
git fetch -q origin "+refs/pull/15999/head:refs/freeze2/head" || true
git fetch -q origin main || true

# --- GATE 0: #15992 merged; NEW_HEAD == live PR head; NEW_HEAD contains origin/main (merge happened)
# In rehearse mode (before the merge exists) these are WARNINGS only; the cut logic still runs against the current head.
gh pr view 15992 --repo gunb-ai/gunbc --json state --jq .state | grep -qx MERGED && echo "GATE0 ok: #15992 MERGED" || { [ "$MODE" = rehearse ] && echo "GATE0 WARN(rehearse): #15992 not MERGED yet" || { echo "GATE0 FAIL: #15992 not MERGED — FREEZE-2 must not run"; exit 1; }; }
LIVE=$(gh pr view 15999 --repo gunb-ai/gunbc --json headRefOid --jq .headRefOid)
[ "$LIVE" = "$NEW_HEAD" ] || { echo "GATE0 FAIL: NEW_HEAD $NEW_HEAD != live PR head $LIVE"; exit 1; }
git cat-file -e "$NEW_HEAD^{commit}" 2>/dev/null || git fetch -q origin "$NEW_HEAD"
[ "$(git rev-list --parents -n1 "$NEW_HEAD" | wc -w)" -ge 3 ] && echo "GATE0 ok: NEW_HEAD is a merge commit (2+ parents)" || { [ "$MODE" = rehearse ] && echo "GATE0 WARN(rehearse): NEW_HEAD not a merge commit (expected pre-merge)" || { echo "GATE0 FAIL: NEW_HEAD is not a merge commit"; exit 1; }; }
git merge-base --is-ancestor origin/main "$NEW_HEAD" && echo "GATE0 ok: origin/main is an ancestor of NEW_HEAD" || { [ "$MODE" = rehearse ] && echo "GATE0 WARN(rehearse): main not merged in yet" || { echo "GATE0 FAIL: main not merged in"; exit 1; }; }

# --- GATE 1: A-leg claim blobs unchanged at NEW_HEAD (the merge must not have touched them)
for pair in "$P_WC:$BLOB_WC_A" "$P_RTP:$BLOB_RTP_A"; do
  p=${pair%%:*}; want=${pair##*:}
  got=$(git rev-parse "$NEW_HEAD:$p")
  [ "$got" = "$want" ] || { echo "GATE1 FAIL: $p at NEW_HEAD is $got, expected $want — stop, re-derive A blobs, tell parent"; exit 1; }
  echo "GATE1 ok: $p blob ${got:0:8}"
done
git cat-file -e "$BLOB_PROBE" 2>/dev/null || { echo "GATE1 FAIL: probe blob $BLOB_PROBE unretrievable — fetch from SEED_A old tree"; git cat-file -e "$SEED_A_OLD:$P_PROBE" && echo "(available at old seed: $(git rev-parse $SEED_A_OLD:$P_PROBE))"; exit 1; }
got=$(git rev-parse "$SEED_A_OLD:$P_PROBE"); [ "$got" = "$BLOB_PROBE" ] || { echo "GATE1 FAIL: old-seed probe blob $got != $BLOB_PROBE"; exit 1; }

# --- GATE 2: non-.dag Rust inputs identical NEW_HEAD vs SEED_B base (one-build gate preview, pre-fetch)
git diff --quiet "$NEW_HEAD" 3c14fdabfb145a8388455faf754bebaedcbc372a -- src/v1 Cargo.toml Cargo.lock \
  && echo "GATE2 ok: src/v1+Cargo identical NEW_HEAD vs SEED_B base (one build still valid)" \
  || { echo "GATE2 WARN: Rust inputs now differ across legs — packet one-build gate will die2 at setup; tell parent BEFORE re-posting"; }

# --- CUT: new tree = NEW_HEAD tree + probe blob only; ONE commit on NEW_HEAD
if git cat-file -e "$NEW_HEAD:$P_PROBE" 2>/dev/null; then echo "CUT FAIL: probe path already exists at NEW_HEAD"; exit 1; fi
rm -f /tmp/freeze2/idx
export GIT_INDEX_FILE=/tmp/freeze2/idx
git read-tree "$NEW_HEAD"
git update-index --add --cacheinfo 100644,"$BLOB_PROBE","$P_PROBE"
NEWTREE=$(git write-tree)
unset GIT_INDEX_FILE
NEW_SEED=$(git commit-tree "$NEWTREE" -p "$NEW_HEAD" -m "evidence v16 seed A on post-#15992 merge head (probe blob $BLOB_PROBE only)")
# gates on the cut, as v14/v15 did:
[ "$(git rev-list --parents -n1 "$NEW_SEED" | wc -w)" -eq 2 ] || { echo "CUT FAIL: seed not one-parent"; exit 1; }
git diff --quiet "$NEW_HEAD" "$NEW_SEED" || true
D=$(git diff --name-only "$NEW_HEAD" "$NEW_SEED"); [ "$D" = "$P_PROBE" ] || { echo "CUT FAIL: delta is [$D], expected only probe path"; exit 1; }
[ "$(git rev-parse "$NEW_SEED:$P_PROBE")" = "$BLOB_PROBE" ] || { echo "CUT FAIL: probe blob mismatch"; exit 1; }
TREE=$(git rev-parse "$NEW_SEED^{tree}")
echo "CUT ok: SEED_A=$NEW_SEED TREE_A=$TREE PARENT_A=$NEW_HEAD"

if [ "$MODE" = "rehearse" ]; then
  echo "REHEARSE: skipping push. Planned branch: session/sleek-seal-226-evidence-v16-A (no force-push; fresh branch name)."
  git branch -f freeze2-rehearse "$NEW_SEED" && echo "REHEARSE: rehearsal commit kept at refs/heads/freeze2-rehearse (delete after: git branch -D freeze2-rehearse)"
else
  git push -q origin "$NEW_SEED":refs/heads/session/sleek-seal-226-evidence-v16-A
  echo "PUSHED session/sleek-seal-226-evidence-v16-A"
fi

# --- PRINT the edits (never applied here)
cat <<EOF

FREEZE-2 EDITS TO APPLY (one delta, then re-gzip/re-sha/re-post per freeze-2 plan):
  packet-evidence-v16.sh line  74: PARENT_A=$OLD_PARENT_A  ->  PARENT_A=$NEW_HEAD
  packet-evidence-v16.sh line  76: SEED_A=$SEED_A_OLD -> SEED_A=$NEW_SEED ; TREE_A=fcba0d8da22642a6b6baa871ff7bdff953ae891f -> TREE_A=$TREE
  (plus the previously approved removal of the vestigial line 1041 echo 'claim_batch_exit[A probe]' >/dev/null)
  then: re-freeze packet sha, update PRE-REG header pins + description comment shas, re-post, re-verify live bodies.
EOF
