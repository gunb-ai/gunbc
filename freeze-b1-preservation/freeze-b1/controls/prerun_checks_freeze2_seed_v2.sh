#!/usr/bin/env bash
# FREEZE-2 SEED-COMMIT PRE-RUN CHECKS, v2 (M2 rework of de333026).
#
# The registered rev5 conditions evaluated AT the frozen seed commit. Local git/grep only: it NEVER builds or
# runs gunbc, claim_batch or claim_executor, never touches a real worktree, and writes only inside a private
# 0700 work root it creates.
#
# What M2 required, and where it is honoured:
#   trap + OUTCOME line          -> every exit path prints `OUTCOME: PASS|FAIL|ABORT rc=<n> <reason>`
#   missing inputs exit 2        -> verify_inputs(); 1 is reserved for a genuine check failure, 2 for bad input
#   no `exec > >(tee ...)`       -> log() appends to the receipt and echoes; no async redirect can truncate it
#   hermetic mode                -> every input is an explicit path/sha, verified present before any check;
#                                  no /tmp/packet path is baked in (the digester is supplied by the caller)
#   every input attached         -> the expectations live in EXPECT_FILE, not in this script's constants
#
# usage: prerun_checks_freeze2_seed_v2.sh <WORK_ROOT> <CANDIDATE_COMMIT>
#   The CANDIDATE is the artifact actually about to be run, and it is a REQUIRED argument with NO default. An
#   earlier version had no candidate channel at all: every identity check read its value from the expectations
#   file, so the script proved 'the env still describes a commit in the repo' rather than 'the thing I am about
#   to run is that commit'. Handing it the base instead of the seed passed every check. A default of
#   $SEED_COMMIT would recreate exactly that circularity, so its absence is load-bearing.
#   env REPO_DIR=<clone source>   the repo that must contain BASE and SEED
#   env EXPECT_FILE=<path>        the attached expectations (see prerun_expectations_freeze2_v2.env)
#   env DIGEST_TOOL=<path>        the canonical probe-block digester
set -u

WANT_RC=0
OUTCOME='ABORT'
REASON='not started'
WORK=''
RECEIPT=''

log(){ [ -n "$RECEIPT" ] && printf '%s\n' "$*" >> "$RECEIPT"; printf '%s\n' "$*"; }
fail(){ REASON="$*"; OUTCOME='FAIL'; WANT_RC=1; log "FAIL $*"; }
note(){ log "OK   $*"; }
abort(){ OUTCOME='ABORT'; WANT_RC=${1:-2}; REASON="${2:-aborted}"; log "ABORT $REASON"; finish; }
finish(){ log "RECEIPT: ${RECEIPT:-<none>}"; log "OUTCOME: $OUTCOME rc=$WANT_RC $REASON"; exit "$WANT_RC"; }
on_exit(){ local e=$?; if [ "$OUTCOME" = 'ABORT' ] && [ "$e" != 2 ]; then OUTCOME='ABORT'; WANT_RC=2; REASON="unexpected exit $e"; log "OUTCOME: ABORT rc=2 $REASON"; fi; }
trap on_exit EXIT

# ---- inputs, all explicit, all verified (M2: missing inputs are exit 2, never 1) -------------------------
EXPECT_FILE=${EXPECT_FILE:-}; DIGEST_TOOL=${DIGEST_TOOL:-}; REPO_DIR=${REPO_DIR:-}
[ $# -ge 2 ] || abort 2 'usage: prerun_checks_freeze2_seed_v2.sh <WORK_ROOT> <CANDIDATE_COMMIT> (the candidate has no default by design)'
WORK=$1
CAND=$2
[ -n "$CAND" ] || abort 2 'empty candidate argument'
for v in EXPECT_FILE DIGEST_TOOL REPO_DIR FRAME_SCRIPT; do
  [ -n "${!v}" ] || abort 2 "required env $v is unset (hermetic mode: no defaults)"
  [ -e "${!v}" ] || abort 2 "required env $v does not exist: ${!v}"
done
[ -d "$REPO_DIR" ] || abort 2 "REPO_DIR is not a directory: $REPO_DIR"
[ -x "$DIGEST_TOOL" ] || [ -f "$DIGEST_TOOL" ] || abort 2 "DIGEST_TOOL is not readable"

# shellcheck disable=SC1090
source "$EXPECT_FILE" || abort 2 "cannot source EXPECT_FILE"
# RELOCATABILITY: a relative path in the expectations resolves next to the EXPECTATIONS FILE, never against the
# caller's cwd -- the whole artifact set may be placed anywhere, and a reviewer must not have to cd first. A value
# that still carries a build-time /tmp/packet path is refused outright, so this property cannot rot back.
EXPECT_DIR=$(cd -- "$(dirname -- "$EXPECT_FILE")" >/dev/null 2>&1 && pwd)
for _v in PROBE_DIFF ACCEPTED_DIFF RTP_ARTIFACT CE_ARTIFACT; do
  _val=${!_v:-}
  [ -n "$_val" ] || continue
  case $_val in
    /tmp/packet/*) abort 2 "$_v still holds a build-time absolute path: $_val" ;;
    /*) : ;;
    *) printf -v "$_v" '%s' "$EXPECT_DIR/$_val" ;;
  esac
done

for k in BASE_COMMIT BASE_TREE SEED_COMMIT SEED_TREE DIFF_SHA BLOCK_SHA RTP_FILE EXPECT_TESTFN; do
  [ -n "${!k:-}" ] || abort 2 "EXPECT_FILE is missing $k"
done
[ -f "$PROBE_DIFF" ] || abort 2 "PROBE_DIFF missing: ${PROBE_DIFF:-<unset>}"
# A zero-byte registered artifact is an INPUT failure, not a mismatch: it cannot express the frozen 45-line
# block, so reporting it as "content differs" would let a truncated download or a failed write look like a
# verdict about the seed. It aborts (exit 2), the same class as a missing file.
[ -s "$PROBE_DIFF" ] || abort 2 "PROBE_DIFF is empty (0 bytes): ${PROBE_DIFF} -- input class, not a content verdict"
[ -f "$ACCEPTED_DIFF" ] || abort 2 "ACCEPTED_DIFF missing: ${ACCEPTED_DIFF:-<unset>}"

mkdir -p "$WORK" || abort 2 "cannot create work root"
chmod 700 "$WORK" || abort 2 "cannot chmod 700 work root"
RECEIPT="$WORK/seed-prerun-receipt.txt"; : > "$RECEIPT" || abort 2 "cannot write receipt"

log "== FREEZE-2 seed pre-run checks v2  $(date -u +%FT%TZ)"
log "   work root   $WORK (0700, private; nothing outside it is written)"
log "   REPO_DIR    $REPO_DIR"
log "   expectations $EXPECT_FILE sha256 $(sha256sum "$EXPECT_FILE" | cut -d' ' -f1)"
log "   digester    $DIGEST_TOOL sha256 $(sha256sum "$DIGEST_TOOL" | cut -d' ' -f1)"
log "   candidate   ${CAND:-?}"
log "   frozen      seed ${SEED_COMMIT:-?} tree ${SEED_TREE:-?}"

CLONE="$WORK/clone"
git clone --no-hardlinks -q -s "$REPO_DIR" "$CLONE" 2>/dev/null || abort 2 "cannot clone $REPO_DIR"
# NOTE (measured lesson): `git clone -s` COPIES the source's remote-tracking refs, which can be stale. Nothing
# here reads a branch or a remote ref: every object is addressed by a literal sha.

# S0 the probe diff is the registered artifact
dsha=$(sha256sum "$PROBE_DIFF" | cut -d' ' -f1)
if [ "$dsha" = "$DIFF_SHA" ]; then note "diff sha256 == $dsha"; else fail "diff sha256 $dsha != registered $DIFF_SHA"; fi

# S1 the CANDIDATE is exactly what was frozen
if ! git -C "$CLONE" cat-file -e "${CAND}^{commit}" 2>/dev/null; then
  fail "candidate ${CAND} is not a resolvable commit in $REPO_DIR -- every later identity check would be vacuous, so they are skipped"
  REASON="${REASON} (unresolvable candidate)"; finish
fi
sc=$(git -C "$CLONE" rev-parse "$CAND^{commit}"); st=$(git -C "$CLONE" rev-parse "$CAND^{tree}")
[ "$sc" = "$CAND" ] && note "candidate resolves to $sc" || fail "candidate $CAND does not resolve to itself ($sc)"
[ "$sc" = "$SEED_COMMIT" ] && note "candidate commit == frozen $sc" || fail "candidate commit $sc != frozen seed $SEED_COMMIT"
[ "$st" = "$SEED_TREE" ] && note "candidate tree   == frozen $st" || fail "candidate tree $st != frozen seed tree $SEED_TREE"
npar=$(( $(git -C "$CLONE" rev-list --parents -n1 "$sc" | wc -w) - 1 ))
[ "$npar" = 1 ] && note "seed has exactly 1 parent" || fail "seed has $npar parents; expected 1"
sp=$(git -C "$CLONE" rev-parse "$sc^")
[ "$sp" = "$BASE_COMMIT" ] && note "seed parent == $sp" || fail "seed parent $sp != base $BASE_COMMIT"

# S2 the base is still the accepted one
bc=$(git -C "$CLONE" rev-parse "$BASE_COMMIT^{commit}"); bt=$(git -C "$CLONE" rev-parse "$bc^{tree}")
[ "$bc" = "$BASE_COMMIT" ] && note "base commit == $bc" || fail "base $bc != $BASE_COMMIT"
[ "$bt" = "$BASE_TREE" ] && note "base tree   == $bt" || fail "base tree $bt != $BASE_TREE"

# S3 the CANDIDATE changes only what the diff touches
mapfile -t dp < <(grep '^diff --git ' "$PROBE_DIFF" | sed 's|^diff --git a/||; s| b/.*$||' | sort -u)
mapfile -t sp_ < <(git -C "$CLONE" diff --name-only "$bc" "$sc" | sort -u)
if [ "${#dp[@]}" -eq "${#sp_[@]}" ] && [ "${dp[*]}" = "${sp_[*]}" ]; then
  note "seed touches exactly the ${#dp[@]} path(s) the diff touches: ${sp_[*]}"
else fail "seed path set != diff path set: seed={${sp_[*]}} diff={${dp[*]}}"; fi

# S4 at the CANDIDATE: the registered declaration count
git -C "$CLONE" checkout -q "$sc" || abort 2 "cannot check out the candidate"
n=$(grep -c '^test fn ' "$CLONE/$RTP_FILE" || true)
[ "$n" = "$EXPECT_TESTFN" ] && note "at the seed: '^test fn ' count == $n" || fail "seed declares $n 'test fn', expected $EXPECT_TESTFN"

# S5 the probe block AT THE CANDIDATE digests to the registered value, by the same canonical rule
SD="$WORK/seed-vs-base.diff"
git -C "$CLONE" diff "$bc" "$sc" -- "$RTP_FILE" > "$SD"
bout=$(python3 "$DIGEST_TOOL" "$SD" 2>&1) || true
if printf '%s' "$bout" | grep -q 'NO PROBE BLOCK'; then bsha=NONE; blines=0
else bsha=$(printf '%s' "$bout" | sed -n 's/.*sha256=\([0-9a-f]\{64\}\).*/\1/p'); blines=$(printf '%s' "$bout" | sed -n 's/.*lines=\([0-9]*\).*/\1/p'); fi
[ "$bsha" = "$BLOCK_SHA" ] && note "at the seed: probe block sha256 == $bsha ($blines lines)" \
  || fail "at the seed: probe block sha256 $bsha ($blines lines) != registered $BLOCK_SHA"
acc=$(python3 "$DIGEST_TOOL" "$ACCEPTED_DIFF" 2>&1 | sed -n 's/.*sha256=\([0-9a-f]\{64\}\).*/\1/p')
# THE COMPARISON (review MINOR): printing the accepted artifact's digest proved only that a digest exists. An
# ACCEPTED_DIFF swapped for some other probe set passed unnoticed, because nothing tied it to what is being
# frozen. It must yield the SAME block digest as the candidate's own diff, which is itself pinned to BLOCK_SHA.
# THE COMPARISON (review MINOR): printing the accepted artifact's digest proved only that SOME digest exists, so an
# ACCEPTED_DIFF swapped for any other probe set passed unnoticed. The accepted file is the HISTORICAL artifact --
# the probe set as accepted before the B2 canonical-arm re-form -- so it is checked against ITS OWN registered
# digests, not against the candidate's: BLOCK_SHA belongs to the candidate, and the two generations must stay
# distinguishable. Both the file bytes and the extracted block are pinned, and a missing registration is an input
# abort, never a silent pass.
[ -n "${ACCEPTED_SHA:-}" ]       || abort 2 "ACCEPTED_SHA missing: the accepted artifact's bytes are then uncheckable"
[ -n "${ACCEPTED_BLOCK_SHA:-}" ] || abort 2 "ACCEPTED_BLOCK_SHA missing: the accepted artifact's probe block is then uncheckable"
accfile=$(sha256sum "$ACCEPTED_DIFF" | cut -d' ' -f1)
if [ "$accfile" != "$ACCEPTED_SHA" ]; then
  fail "accepted artifact is $accfile, not the registered $ACCEPTED_SHA -- the file the checker holds is not the file that was accepted"
elif [ -z "$acc" ]; then
  fail "accepted artifact does not yield a block digest"
elif [ "$acc" != "$ACCEPTED_BLOCK_SHA" ]; then
  fail "accepted artifact block digests to $acc, not the registered $ACCEPTED_BLOCK_SHA"
elif [ "$acc" = "$bsha" ]; then
  fail "accepted and candidate probe blocks are the SAME generation ($acc): the registered supersession is not real"
else
  note "accepted artifact is the registered historical generation: file=$accfile block=$acc (superseded by the candidate's $bsha)"
fi

# S6 the link: the CANDIDATE tree IS base+diff, no fuzz, no hand edit
C2="$WORK/applied"; git clone --no-hardlinks -q -s "$REPO_DIR" "$C2" 2>/dev/null || fail "second clone failed"
git -C "$C2" checkout -q "$bc" 2>/dev/null || fail "cannot check out the base in the second clone"
if git -C "$C2" apply --check "$PROBE_DIFF"; then note "git apply --check (NO fuzz) at the base: CLEAN"
else fail "git apply --check FAILED at the base"; fi
git -C "$C2" apply "$PROBE_DIFF" || fail "git apply failed after --check said clean"
git -C "$C2" add -A -- "$RTP_FILE"
at=$(git -C "$C2" write-tree)
if [ "$at" = "$st" ]; then note "APPLIED TREE == CANDIDATE TREE == $at"
else fail "applied tree $at != candidate tree $st"; fi

# S7 the executor's shape, read from source (no execution)
CB="$CLONE/src/v1/stage0/src/bin/claim_batch.rs"
if [ -f "$CB" ]; then
  for p in 'total_witnesses' 'for function in &group.functions' 'validate_explicit_functions'; do
    grep -q "$p" "$CB" && note "claim_batch.rs still contains: $p" || fail "claim_batch.rs no longer contains: $p"
  done
else fail "claim_batch.rs missing at the seed"; fi

# S8 the B2 invariant, so the canonical-arm repair cannot silently regress
if grep -qE '^[[:space:]]*optional_(present|absent).*=>' "$CLONE/$RTP_FILE"; then
  fail "a lexeme match PATTERN is back in $RTP_FILE (B2 regression)"
else note "no lexeme match patterns (B2 canonical arms hold)"; fi

# S9 every blob the RUN FRAME pins must be the blob git actually has AT THE CANDIDATE.
# This check exists because a fabricated rtp blob pin (first 8 hex right, remainder invented) survived two review
# rounds. It is not cosmetic: the frame's own preflight compares `git hash-object` against these pins and dies with
# REFUSED_SETUP (exit 2, no verdict lines), so a wrong pin spends the single granted attempt producing nothing.
# A hash that git does not know cannot be a measurement, whatever prose is written about it.
[ -f "$FRAME_SCRIPT" ] || abort 2 "FRAME_SCRIPT is not a readable file: $FRAME_SCRIPT"
ents=$(grep -m1 '^ENTRIES=' "$FRAME_SCRIPT" | sed 's/^ENTRIES="//; s/"$//')
[ -n "$ents" ] || abort 2 "no ENTRIES= line found in $FRAME_SCRIPT -- nothing to cross-check"
nf=0
for E in $ents; do
  nf=$((nf+1))
  IFS=: read -r T F B V <<< "$E"
  case "$B" in ''|*[!0-9a-f]*) fail "frame entry $T carries a non-hex blob pin: '$B'"; continue;; esac
  [ "${#B}" = 40 ] || { fail "frame entry $T blob pin has ${#B} characters, not 40: $B"; continue; }
  gb=$(git -C "$CLONE" ls-tree "$sc" -- "$F" | awk '{print $3}')
  if [ -z "$gb" ]; then fail "frame entry $T pins $F but that path does not exist at the candidate"
  elif [ "$gb" = "$B" ]; then note "frame pin for $T matches git ls-tree: $gb ($(basename "$F"))"
  else fail "frame pins $T at $B but git ls-tree at the candidate gives $gb -- the pin is not a measurement"; fi
done
[ "$nf" -gt 0 ] || fail "no frame entries parsed from ENTRIES"

[ "$OUTCOME" = 'FAIL' ] && { REASON="${REASON} (see FAIL lines above)"; finish; }
OUTCOME='PASS'; REASON="all checks passed"; finish
