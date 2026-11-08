#!/usr/bin/env bash
# FREEZE-2 SEED-COMMIT PRE-RUN CHECKS — the registered rev4 conditions evaluated AT the frozen seed commit,
# not merely at the base it was cut from. Local git/grep only: NEVER builds or runs gunbc, claim_batch or
# claim_executor; never touches a real worktree; works only in a private throwaway clone under a 0700 TMPDIR.
#
# usage: prerun_checks_freeze2_seed.sh <SEED_REV> [BASE_REV] [DIFF] [OUTDIR]
#   <SEED_REV>   the frozen seed commit (default c07e2b8b...)
#   [BASE_REV]   default 57ce2e78...  (the accepted seed base; tree 5a14ac7c...)
#   [DIFF]       default /tmp/packet/SEED_PROBE_SET_rebase_candidate.diff (sha 2852848f...)
# Companion to prerun_checks_freeze2.sh (sha b8182284...). It re-uses that script's OWN canonical block
# digester (probe_block_digest.py) so both receipts compute the same function of the same bytes.
set -u -o pipefail

SEED=${1:?usage: prerun_checks_freeze2_seed.sh <SEED_REV> [BASE_REV] [DIFF] [OUTDIR]}
BASE=${2:-57ce2e787aa5227fc1840961c607981f4abdde86}
DIFF=${3:-/tmp/packet/SEED_PROBE_SET_rebase_candidate.diff}
WORK=${4:-}
REPO_DIR=${REPO_DIR:-/var/tmp/freeze2-cut-15695/repo}
RTP_FILE=src/v2/test/claim/namespace_xl0/return_tail_position.dag
EXPECT_SEED_COMMIT=c07e2b8b666e0d2c529a4b0c44475f28db6cc129
EXPECT_SEED_TREE=d2bd176dc1ba1de4fff1387c145f91fb7ad5756c
EXPECT_BASE_COMMIT=57ce2e787aa5227fc1840961c607981f4abdde86
EXPECT_BASE_TREE=5a14ac7ca50f43312760f092c86ebee7c679ce37
EXPECT_DIFF_SHA=2852848fa9c630f96a4050d7c2c9de5da9963665d8216fc0473c79f5aa12461b
EXPECT_BLOCK_SHA=af6f17477ac0f39591abb5345840e0eac827e388072afddb39f002b7eb17bd22
EXPECT_RTP_TESTFN=21
DIGEST=/tmp/packet/probe_block_digest.py

fail(){ printf 'FAIL %s\n' "$*" >&2; RECEIPT_STATUS=FAIL; }
note(){ printf 'OK   %s\n' "$*"; }
[ -n "$WORK" ] || WORK=$(mktemp -d "${TMPDIR:-/var/tmp}/freeze2-seed-prerun.XXXXXX") || { echo "FATAL: no work dir"; exit 2; }
mkdir -p "$WORK" && chmod 700 "$WORK"
RECEIPT="$WORK/seed-prerun-receipt.txt"; RECEIPT_STATUS=PASS
exec > >(tee -a "$RECEIPT") 2>&1

echo "== FREEZE-2 SEED-COMMIT pre-run checks  (seed=$SEED base=$BASE diff=$DIFF)"
date -u +'%Y-%m-%dT%H:%M:%SZ'
echo "   work root: $WORK (0700, private; nothing outside it is written)"
echo "   REPO_DIR:  $REPO_DIR (local; never pushed to)"

# --- S0. the diff is the artifact rev4 registers ---------------------------------------------------------
dsha=$(sha256sum "$DIFF" | cut -d' ' -f1)
if [ "$dsha" = "$EXPECT_DIFF_SHA" ]; then note "diff sha256 == $dsha (the re-anchor rev4 registers)"; else
  fail "diff sha256 $dsha != registered $EXPECT_DIFF_SHA"; fi

CLONE="$WORK/clone"
git clone --no-hardlinks -q -s "$REPO_DIR" "$CLONE" 2>/dev/null || { echo "FATAL: clone failed"; exit 2; }
# NOTE (measured lesson): `git clone -s` COPIES the source's remote-tracking refs, which can be stale. Nothing
# here reads a branch or a remote ref: every object is addressed by a literal sha and resolved or rejected.

# --- S1. the seed commit is exactly what was frozen -----------------------------------------------------
git -C "$CLONE" cat-file -e "$SEED^{commit}" 2>/dev/null || { fail "seed $SEED is not present in $REPO_DIR"; echo "STATUS: $RECEIPT_STATUS"; exit 1; }
seed_commit=$(git -C "$CLONE" rev-parse "$SEED^{commit}")
seed_tree=$(git -C "$CLONE" rev-parse "$SEED^{tree}")
[ "$seed_commit" = "$EXPECT_SEED_COMMIT" ] && note "seed commit == $seed_commit" || fail "seed commit $seed_commit != frozen $EXPECT_SEED_COMMIT"
[ "$seed_tree" = "$EXPECT_SEED_TREE" ] && note "seed tree   == $seed_tree" || fail "seed tree $seed_tree != frozen $EXPECT_SEED_TREE"
npar=$(git -C "$CLONE" rev-list --parents -n1 "$seed_commit" | wc -w); npar=$((npar-1))
[ "$npar" = 1 ] && note "seed has exactly 1 parent (a seed commit is a single child of the base, never a merge)" \
                || fail "seed has $npar parents; expected exactly 1"
seed_parent=$(git -C "$CLONE" rev-parse "$seed_commit^")
[ "$seed_parent" = "$EXPECT_BASE_COMMIT" ] && note "seed parent == $seed_parent (the accepted base)" || fail "seed parent $seed_parent != base $EXPECT_BASE_COMMIT"

# --- S2. the base under the seed is still the accepted one ----------------------------------------------
base_commit=$(git -C "$CLONE" rev-parse "$BASE^{commit}"); base_tree=$(git -C "$CLONE" rev-parse "$base_commit^{tree}")
[ "$base_commit" = "$EXPECT_BASE_COMMIT" ] && note "base commit == $base_commit" || fail "base $base_commit != accepted $EXPECT_BASE_COMMIT"
[ "$base_tree" = "$EXPECT_BASE_TREE" ] && note "base tree   == $base_tree" || fail "base tree $base_tree != accepted $EXPECT_BASE_TREE"

# --- S3. the seed changes ONLY what the probe diff touches ------------------------------------------------
mapfile -t diff_paths < <(grep '^diff --git ' "$DIFF" | sed 's|^diff --git a/||; s| b/.*$||' | sort -u)
mapfile -t seed_paths < <(git -C "$CLONE" diff --name-only "$base_commit" "$seed_commit" | sort -u)
if [ "${#diff_paths[@]}" -eq "${#seed_paths[@]}" ] && [ "$(printf '%s\n' "${diff_paths[@]}")" = "$(printf '%s\n' "${seed_paths[@]}")" ]; then
  note "seed touches exactly the ${#diff_paths[@]} path(s) the probe diff touches: ${seed_paths[*]}"
else
  fail "seed path set != diff path set: seed={${seed_paths[*]}} diff={${diff_paths[*]}}"
fi

# --- S4. at the SEED commit: 21 declarations, and every added diff line present verbatim ------------------
git -C "$CLONE" checkout -q "$seed_commit" || { fail "cannot check out the seed"; echo "STATUS: $RECEIPT_STATUS"; exit 1; }
n=$(grep -c '^test fn ' "$CLONE/$RTP_FILE" || true)
[ "$n" = "$EXPECT_RTP_TESTFN" ] && note "at the SEED: grep -c '^test fn ' == $n (17 registered + 4 probes)" \
  || fail "seed declares $n 'test fn' in $RTP_FILE, expected $EXPECT_RTP_TESTFN"
# no constant is hard-coded here: the probe NAMES and LITERALS are read from the artifact, so this cannot
# certify a seed that drifted away from the artifact.
missing=0
while IFS= read -r line; do
  case "$line" in '+'*) body=${line#+} ;; *) continue ;; esac
  case "$body" in '++ '*) continue ;; esac           # diff header, not content
  [ -z "${body// /}" ] && continue                    # whitespace-only added line
  grep -qF -- "$body" "$CLONE/$RTP_FILE" || { fail "added diff line absent from the seed: $body"; missing=$((missing+1)); }
done < "$DIFF"
[ "$missing" = 0 ] && note "every added line of the artifact is present verbatim in the seed's $RTP_FILE" \
                   || fail "$missing added line(s) absent from the seed"

# --- S5. the probe block AT THE SEED digests to the registered value, by the SAME canonical rule ---------
SEEDDIFF="$WORK/seed-vs-base.diff"
git -C "$CLONE" diff "$base_commit" "$seed_commit" -- "$RTP_FILE" > "$SEEDDIFF"
bout=$(python3 "$DIGEST" "$SEEDDIFF")
if printf '%s' "$bout" | grep -q "NO PROBE BLOCK"; then bsha=NONE; blines=0; else
  bsha=$(printf '%s' "$bout" | sed 's/.*sha256=//' | tr -d ' '); blines=$(printf '%s' "$bout" | sed 's/.*lines=//; s/ .*//')
fi
[ "$bsha" = "$EXPECT_BLOCK_SHA" ] && note "at the SEED: probe block sha256 == $bsha ($blines lines, canonical rule)" \
  || fail "at the SEED: probe block sha256 $bsha ($blines lines) != registered $EXPECT_BLOCK_SHA"
# and the accepted artifact must still digest identically (the pairing is the point)
acc=$(python3 "$DIGEST" /tmp/packet/SEED_PROBE_SET.diff | sed 's/.*sha256=//' | tr -d ' ')
[ "$acc" = "$EXPECT_BLOCK_SHA" ] && note "accepted SEED_PROBE_SET.diff still digests to $acc" || fail "accepted artifact drifted: $acc"

# --- S6. THE LINK: the seed tree IS base+diff, byte for byte, with no fuzz -------------------------------
C2="$WORK/applied"; git clone --no-hardlinks -q -s "$REPO_DIR" "$C2" 2>/dev/null || { fail "second clone failed"; }
git -C "$C2" checkout -q "$base_commit" 2>/dev/null || fail "cannot check out the base in the second clone"
if git -C "$C2" apply --check "$DIFF"; then note "git apply --check (NO fuzz) of the diff at the base: CLEAN"; else
  fail "git apply --check FAILED at the base -- STOP"; fi
git -C "$C2" apply "$DIFF" || fail "git apply failed after --check said clean"
applied_tree=$(cd "$C2" && git add -A -- "$RTP_FILE" && git write-tree)
if [ "$applied_tree" = "$seed_tree" ]; then note "APPLIED TREE == SEED TREE == $applied_tree  (the seed is base+diff exactly; no fuzz, no hand edit)"
else fail "applied tree $applied_tree != seed tree $seed_tree -- the seed is NOT the registered diff applied to the base"; fi

# --- S7. the executor's own shape, read from source at the seed (no execution) ---------------------------
CB="$CLONE/src/v1/stage0/src/bin/claim_batch.rs"
if [ -f "$CB" ]; then
  for p in 'total_witnesses' 'for function in &group.functions' 'validate_explicit_functions'; do
    grep -q "$p" "$CB" && note "claim_batch.rs at the seed still contains: $p" || fail "claim_batch.rs no longer contains: $p"
  done
else fail "claim_batch.rs missing at the seed"; fi

echo
echo "RECEIPT: $RECEIPT"
echo "STATUS: $RECEIPT_STATUS"
# The receipt's sha256 is NEVER self-reported (the output is still streaming into the file when a self-hash
# would run). Take it EXTERNALLY after this process exits:   sha256sum "$RECEIPT"
[ "$RECEIPT_STATUS" = PASS ] || exit 1
exit 0
