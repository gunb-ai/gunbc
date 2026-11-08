#!/usr/bin/env bash
# FREEZE-2 PRE-RUN CHECKS — the three conditions registered in PRE_REG rev4 (sha b77f206d...) BEFORE any packet
# is granted. Any failure stops the attempt. Local git/grep only: it NEVER builds or runs gunbc, claim_batch or
# claim_executor, never touches a real worktree, and never touches the network beyond `git fetch` of refs you
# already have access to. It works in a private throwaway clone under a private TMPDIR.
#
# usage: prerun_checks_freeze2.sh <SEED_BASE_REV> [DIFF] [OUTDIR]
#   <SEED_BASE_REV>  the FINAL seed base (the commit the seed commit is cut from; it carries the main merge)
#   DIFF             default /tmp/packet/SEED_PROBE_SET_rebase_candidate.diff  (sha 2852848f...)
#   OUTDIR           default /var/tmp/freeze2-prerun.<rand>   (0700, private; receipt is printed)
set -u -o pipefail

BASE=${1:?usage: prerun_checks_freeze2.sh <SEED_BASE_REV> [DIFF] [OUTDIR]}
DIFF=${2:-/tmp/packet/SEED_PROBE_SET_rebase_candidate.diff}
WORK=${3:-}
REPO_DIR=${REPO_DIR:-/home/briansrls/.worktrees/gunbc/sleek-seal-226}
RTP_FILE=src/v2/test/claim/namespace_xl0/return_tail_position.dag
EXPECT_DIFF_SHA=2852848fa9c630f96a4050d7c2c9de5da9963665d8216fc0473c79f5aa12461b
EXPECT_BLOCK_SHA=af6f17477ac0f39591abb5345840e0eac827e388072afddb39f002b7eb17bd22
EXPECT_RTP_TESTFN=21

fail(){ printf 'FAIL %s\n' "$*" >&2; RECEIPT_STATUS=FAIL; }
note(){ printf 'OK   %s\n' "$*"; }
[ -n "$WORK" ] || WORK=$(mktemp -d "${TMPDIR:-/var/tmp}/freeze2-prerun.XXXXXX") || { echo "FATAL: no work dir"; exit 2; }
chmod 700 "$WORK" || true
mkdir -p "$WORK" && chmod 700 "$WORK"
RECEIPT="$WORK/prerun-receipt.txt"; RECEIPT_STATUS=PASS
exec > >(tee -a "$RECEIPT") 2>&1

echo "== FREEZE-2 pre-run checks  (base=$BASE diff=$DIFF)"
date -u +'%Y-%m-%dT%H:%M:%SZ'
echo "   work root: $WORK (0700, private; nothing outside it is written)"

# --- 0. the diff itself must be the artifact rev4 cites -----------------------------------------------
dsha=$(sha256sum "$DIFF" | cut -d' ' -f1)
if [ "$dsha" = "$EXPECT_DIFF_SHA" ]; then note "diff sha256 == $dsha (the re-anchor rev4 registers)"; else
  fail "diff sha256 $dsha != registered $EXPECT_DIFF_SHA -- do not proceed with a different artifact"; fi

# --- 1. a private clone at the FINAL seed base; NO fuzz anywhere ---------------------------------------
CLONE="$WORK/clone"
git clone --no-hardlinks -q -s "$REPO_DIR" "$CLONE" 2>/dev/null || { echo "FATAL: clone failed"; exit 2; }
git -C "$CLONE" fetch -q "$REPO_DIR" 2>/dev/null || true
git -C "$CLONE" checkout -q "$BASE" 2>/dev/null || { fail "cannot check out $BASE"; echo "RECEIPT: $RECEIPT"; echo "STATUS: $RECEIPT_STATUS"; exit 1; }
base_commit=$(git -C "$CLONE" rev-parse "$BASE^{commit}")
note "seed base = $base_commit  (tree $(git -C "$CLONE" rev-parse "$base_commit^{tree}"))"

# registered check 1: `git apply --check` with NO fuzz.  -v is informational; no --recount, no --fuzz, no -C.
if git -C "$CLONE" apply --check "$DIFF"; then note "git apply --check (NO fuzz) of $EXPECT_DIFF_SHA at $base_commit: CLEAN"
else fail "git apply --check FAILED at the final seed base -- STOP. Re-anchor (byte-identical probe block = not a content change); fuzz is never an accepted landing"; fi

# --- 2. apply for real (in the throwaway clone only) and count the declarations -------------------------
if git -C "$CLONE" apply "$DIFF"; then
  note "applied into the throwaway clone (the real worktree was never touched)"
else
  fail "git apply failed after --check said clean (or --check failed): STOP"
fi
n=$(grep -c '^test fn ' "$CLONE/$RTP_FILE" || true)
if [ "$n" = "$EXPECT_RTP_TESTFN" ]; then note "grep -c '^test fn ' on the seed's $RTP_FILE == $n (17 registered + 4 probes)"
else fail "seed declares $n 'test fn' in $RTP_FILE, expected $EXPECT_RTP_TESTFN -- the seed lost or duplicated the probe block"; fi

# the 21 names must be exactly the 17 registered + the 4 the frozen probe artifact declares
names_file="$WORK/seed_test_fns.txt"; grep '^test fn ' "$CLONE/$RTP_FILE" | sed 's/^test fn //; s/[ (].*$//' | sort > "$names_file"
dprobe(){ grep '^+test fn ' "$1" | sed 's/^+test fn //; s/[ (].*$//'; }
probes=$(dprobe "$DIFF" | sort)
echo "-- probe names declared by the artifact: $(echo "$probes" | tr '\n' ' ')"
for p in $probes; do
  grep -qx "$p" "$names_file" || fail "probe $p is not declared in the applied seed file"
done

# registered check 3: the probe block digest, recomputed from the APPLIED seed and digested by the SAME
# canonical rule used to freeze af6f1747 (probe_block_digest.py), so both sides are the identical function.
APPLIED="$WORK/applied.diff"
git -C "$CLONE" diff -- "$RTP_FILE" > "$APPLIED"
bout=$(python3 /tmp/packet/probe_block_digest.py "$APPLIED")
if printf '%s' "$bout" | grep -q "NO PROBE BLOCK"; then bsha=NONE; blines=0; else
  bsha=$(printf '%s' "$bout" | sed 's/.*sha256=//' | tr -d ' ')
  blines=$(printf '%s' "$bout" | sed 's/.*lines=//; s/ .*//')
fi
if [ "$bsha" = "$EXPECT_BLOCK_SHA" ]; then note "applied-seed probe block sha256 == $bsha ($blines lines, canonical rule)"
elif [ "$bsha" = NONE ]; then fail "the applied seed carries NO probe block at all (marker line missing) -- STOP"
else fail "applied-seed probe block sha256 $bsha ($blines lines) != registered $EXPECT_BLOCK_SHA -- content changed inside the block: this is a CONTENT change and comes back for contract review"; fi
# and the accepted artifact must still digest to the same value (the pairing is the point)
acc=$(python3 /tmp/packet/probe_block_digest.py /tmp/packet/SEED_PROBE_SET.diff | sed 's/.*sha256=//' | tr -d ' ')
[ "$acc" = "$EXPECT_BLOCK_SHA" ] && note "accepted SEED_PROBE_SET.diff still digests to $acc" || fail "accepted artifact drifted: $acc"

# --- 3. the executor's own shape, read from source (no execution) ---------------------------------------
CB="$CLONE/src/v1/stage0/src/bin/claim_batch.rs"
if [ -f "$CB" ]; then
  for probe in 'total_witnesses' 'for function in &group.functions' 'validate_explicit_functions'; do
    grep -q "$probe" "$CB" && note "claim_batch.rs still contains: $probe" || fail "claim_batch.rs no longer contains: $probe"
  done
else fail "claim_batch.rs missing at the seed base"; fi

echo
echo "RECEIPT: $RECEIPT"
echo "STATUS: $RECEIPT_STATUS"
# The receipt's sha256 is NEVER self-reported: the script's own output is still being streamed into the file
# when a self-hash would run, so it would hash a partial file. Take it EXTERNALLY after this process exits:
#     sha256sum "$RECEIPT"
# and record that value. (Rule: shas only from measured output, and never by the thing being hashed.)
[ "$RECEIPT_STATUS" = PASS ] || exit 1
exit 0
