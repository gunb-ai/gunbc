#!/usr/bin/env bash
# RECONSTRUCTION FIXTURE for freeze-2 MODE=B. Proves the seed-rebuild phase refuses, with its OWN named reason, on
# each way it can go wrong -- and only succeeds when every identity holds.
#
# Scope, and why it is narrow: keen-otter's authorisation (msg_81c240d4) is LOCAL preparation and review only, so
# this fixture touches nothing but git and the network fetch of one published commit. It never invokes cargo and
# never invokes claim_batch: it extracts the frame's reconstruction region verbatim and runs exactly that. The
# extraction is structural with BOTH ends asserted, so the fixture cannot quietly drift into executing some other
# part of the frame (that exact mistake -- a stale line range running the live cgroup preflight -- is what this
# pattern was created to prevent).
#
# What tree equality does and does not prove (stated here because it is easy to over-claim): matching
# tree a0de248f proves the CONTENT of the working tree matches the frozen seed content. It does NOT prove the
# original seed COMMIT identity -- a rebuild does not reproduce the parent chain, author, committer or message --
# and it does not prove anything about execution. Those are separate claims with separate evidence.
set -u
SELF_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)   # inputs resolve next to me, never at a build-time absolute path
FRAME=${FRAME:-$SELF_DIR/packet-evidence-freeze2.sh}
# MISSING/UNREADABLE INPUT IS AN ABORT (exit 2), NOT A CONTENT VERDICT (exit 1): a reviewer who forgets to attach
# the frame must see an abort, and the collector must not read exit 1 as "a control failed on merit".
[ -r "$FRAME" ] || { echo "ABORT (exit 2): frame not readable: $FRAME"; exit 2; }
[ -s "$FRAME" ] || { echo "ABORT (exit 2): frame is empty: $FRAME"; exit 2; }
[ -f "$FRAME" ] || { echo "ABORT: no frame at $FRAME"; exit 2; }
[ -s "$FRAME" ] || { echo "ABORT: frame is zero bytes"; exit 2; }
W="${TMPDIR:-/var/tmp}/recon-fixture.$$"; mkdir -p "$W"; chmod 700 "$W"
cleanup(){ [ -n "${KEEP:-}" ] || chmod -R u+rwX "$W" 2>/dev/null; [ -n "${KEEP:-}" ] || rm -rf "$W" 2>/dev/null; }
trap cleanup EXIT

pass=0; bad=0
# A fixture that runs FEWER cases than it claims is not a smaller suite -- it is a control that stopped running. The
# negctl fixture once printed OK=12 BAD=0 with seven cases never executed, so the declared count is checked here too.
EXPECTED_CASES=10
say(){ printf '%s\n' "$*"; }
ok(){ pass=$((pass+1)); say "OK   $*"; }
no(){ bad=$((bad+1)); say "BAD  $*"; }

# ---- structural extraction, both ends asserted -----------------------------
first(){ grep -n -m1 -F "$1" "$FRAME" | cut -d: -f1; }
A1=$(first 'PAYLOAD_REPAIR='); A2=$(first 'SEEDP_BYTES=4026')   # the payload block ends at its registered-bytes line
B1=$(first '# --- MODE B: reconstruct'); B2=$(first 'echo "seed ok tree=')
# THE HELPERS ARE TAKEN BY DEFINITION, NOT BY LINE WINDOW. An earlier revision lifted the
# frame lines from 'CAP_SETUP=' to 'ENTRIES="', which worked only while every definition sat
# inside that window and every top-level COMMAND outside it. Reordering the frame (the caps had
# to move above their first use) put phase invocations and the usage check inside the window, so
# the harness started executing frame code it was never meant to run and failed with rc=127 on a
# helper defined above the window -- a fixture that breaks on a harmless reorder is testing line
# numbers. extract_helpers.py copies function DEFINITIONS verbatim (a definition has no effect
# until called) plus the literal cap assignments, and refuses if a required one is missing.
HELPERS="$W/helpers_block.sh.raw"
python3 "$SELF_DIR/build/extract_helpers.py" "$FRAME" --caps \
   --require=caprun,bounded,bcall,plan,phase_done,deadline_check,unpickle,index_agrees_with_worktree,tree_clean,run \
   > "$HELPERS" || { say "BAD  helper extraction refused by extract_helpers.py"; exit 2; }
[ -s "$HELPERS" ] || { say "BAD  helper extraction produced nothing"; exit 2; }
for pair in "A1:$A1" "A2:$A2" "B1:$B1" "B2:$B2"; do
  # an unfindable anchor means the FRAME artifact is missing or malformed: INPUT class, exit 2, never 1
  [ -n "${pair#*:}" ] || { say "BAD  extraction anchor missing: ${pair%%:*}"; exit 2; }
done
[ "$A1" -lt "$A2" ] && [ "$B1" -lt "$B2" ] || { say "BAD  anchors out of order"; exit 2; }
sed -n "${A1},${A2}p"      "$FRAME" > "$W/payloads_block.sh"
sed -n "${B1},${B2}p"      "$FRAME" > "$W/recon_block.sh"
cp "$HELPERS" "$W/helpers_block.sh"
# the identity constants the reconstruction refuses against, taken line-for-line from the frame so a fixture can
# never pass against an id the frame does not actually use. EXTRA_SETS re-assigns AFTER this, so inversions win.
: > "$W/identity.sh"
for V in PIN PIN_TREE MAIN_COMMIT MAIN_TREE BASE_COMMIT BASE_TREE REPO_URL; do
  _l=$(grep -n -m1 "^$V=" "$FRAME" | cut -d: -f1)
  [ -n "$_l" ] || { say "BAD  identity constant $V is not defined in the frame"; exit 2; }
  sed -n "${_l}p" "$FRAME" >> "$W/identity.sh"
done
say "identity constants taken verbatim: $(wc -l < "$W/identity.sh") of 7"

DIE2_LINE=$(grep -n -m1 '^die2() {' "$FRAME" | cut -d: -f1)
[ -n "$DIE2_LINE" ] || { say "BAD  no die2() in the frame"; exit 2; }
sed -n "${DIE2_LINE}p" "$FRAME" > "$W/die2.sh"
grep -q '^unpickle()' "$W/helpers_block.sh"  || { say "BAD  helpers block lost unpickle()"; exit 2; }
grep -q '^PAYLOAD_SEED=' "$W/payloads_block.sh" || { say "BAD  payloads block lost the seed payload"; exit 2; }
grep -q '^caprun()'     "$W/helpers_block.sh"  || { say "BAD  helpers block lost caprun()"; exit 2; }
grep -q 'git write-tree' "$W/recon_block.sh"   || { say "BAD  recon block lost the tree check"; exit 2; }
say "extracted frame lines: payloads $A1..$A2 ; helpers by DEFINITION ($(wc -l < "$W/helpers_block.sh") lines, verbatim) ; recon $B1..$B2 (of $(wc -l < "$FRAME"))"
CAPS_SEEN=$(grep -oE "\bCAP_[A-Z_]+=[0-9]+" "$W/helpers_block.sh" | wc -l)   # two per line is legal bash
[ "$CAPS_SEEN" -ge 8 ] || { say "BAD  helpers block carries only $CAPS_SEEN cap constants; the harness would run with unset caps and a control could pass because a cap was missing"; exit 2; }
say "   harness sees $CAPS_SEEN cap constants, taken verbatim from the frame (no cap is invented by the fixture)"

# the frame's own tree_clean, taken verbatim so the fixture exercises the same predicate the frame does
TREE_CLEAN_LINE=$(grep -n -m1 '^tree_clean() {' "$FRAME" | cut -d: -f1)
IAW_LINE=$(grep -n -m1 '^index_agrees_with_worktree() {' "$FRAME" | cut -d: -f1)
[ -n "$IAW_LINE" ] || { say "BAD  no index_agrees_with_worktree() in the frame"; exit 2; }
[ -n "$TREE_CLEAN_LINE" ] || { say "BAD  no tree_clean() in the frame"; exit 2; }
sed -n "${TREE_CLEAN_LINE}p" "$FRAME" > "$W/tree_clean.sh"
sed -n "${IAW_LINE},$((IAW_LINE+4))p" "$FRAME" > "$W/iaw.sh"   # the whole multi-line predicate, verbatim
grep -q 'ls-files -o --exclude-standard' "$W/iaw.sh" || { say "BAD  the post-apply predicate lost its untracked check"; exit 2; }
cat "$W/iaw.sh" >> "$W/tree_clean.sh"
say "tree_clean taken verbatim from frame L$TREE_CLEAN_LINE"

mkharness(){ # mkharness <name>  -> harness path. ONE assembly for every case, so no case can test a script the
             # others do not: RECON_SRC overrides the reconstruction block only (used by the reorder inversion).
  local d="$W/$1"; mkdir -p "$d"; shift
  local recon=${RECON_SRC:-$W/recon_block.sh}
  { echo 'set -u'
    echo 'OUTCOME_WHY=""'
    echo 'R="$PWD"'
    cat "$W/tree_clean.sh"
    cat "$W/identity.sh"              # the frozen ids and the repo url, verbatim from the frame
    cat "$W/die2.sh"                 # the frame's own refusal primitive, verbatim
    cat "$W/helpers_block.sh"        # the frame's cap constants and every function definition, verbatim
    # the frame initialises the wall-accounting counter at top level ('plan "$PLANNED_TOTAL"'), which the
    # definitions-only extraction cannot carry; the harness reproduces that initialisation call verbatim,
    # otherwise phase_done()/deadline_check() hit 'REMAINING: unbound variable' under set -u.
    echo 'plan "$PLANNED_TOTAL"' 
    cat "$W/payloads_block.sh"
    printf '%s\n' "${EXTRA_SETS[@]}"
    [ -s "$recon" ] || { echo "FIXTURE ABORT: recon block is empty ($recon)"; exit 2; }
    grep -q "git apply --index" "$recon" || { echo "FIXTURE ABORT: recon block lost the applies ($recon)"; exit 2; }
    grep -q "git write-tree" "$recon" || { echo "FIXTURE ABORT: recon block lost the tree check ($recon)"; exit 2; }
    cat "$recon"
    echo 'echo "RECON_SUCCEEDED"'
  } > "$d/harness.sh"
  printf '%s' "$d/harness.sh"
}

runh(){ # runh <label> <expected-substring|RECON_SUCCEEDED> <want-refusal:yes|no> <harness>
  local label=$1 want=$2 refus=$3 h=$4 out rc
  # the run is captured to a file as well as tested, so a case that wants to look at the output again reads the
  # SAME run. Re-invoking the harness is not innocent: a second apply into an already-patched work tree refuses for
  # a reason that has nothing to do with what is being asserted (this fixture's first version did exactly that).
  out=$(cd "$(dirname "$h")" && bash "$h" 2>&1); rc=$?
  printf '%s\n' "$out" > "${h%.sh}.out"
  if [ "$refus" = no ]; then
    if [ $rc = 0 ] && printf '%s' "$out" | grep -q "$want"; then ok "$label: accepted, rc=0, matched '$want'"
    else no "$label: expected acceptance but rc=$rc ; tail: $(printf '%s' "$out" | tail -1)"; fi
  else
    if [ $rc != 0 ] && printf '%s' "$out" | grep -q "$want"; then ok "$label: refused as required -- $(printf '%s' "$out" | grep -m1 "$want" | cut -c1-118)"
    else no "$label: expected refusal naming '$want' but rc=$rc ; tail: $(printf '%s' "$out" | tail -1)"; fi
  fi
}

export MAIN_COMMIT_MAIN=7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9

say "== P-genuine: the reconstruction as frozen must succeed"
EXTRA_SETS=()
H=$(mkharness genuine); runh "P-genuine" 'RECON_SUCCEEDED' no "$H"
# verify the genuine run actually printed the frozen tree -- from the run already made above, not a second run
out=$(cat "$W/genuine/harness.out")
printf '%s' "$out" | grep -q 'seed ok tree=a0de248f1918aff315145392e2057db6ecc66c1a' \
  && ok "P-genuine printed the frozen seed tree a0de248f..." || no "P-genuine did not print the frozen tree"

say "-- I1 wrong base: fetch a different published commit"
EXTRA_SETS=("MAIN_COMMIT=a4a21da46284e64a31efff08585c42aaf589089a")
H=$(mkharness wrong-base); runh "I1 wrong base" 'RECON_REFUSE' yes "$H"

say "-- I2 altered patch: flip one base64 character of the seed payload"
EXTRA_SETS=("PAYLOAD_SEED=\$(printf '%s' \"\$PAYLOAD_SEED\" | sed 's/./X/1')")
H=$(mkharness altered); runh "I2 altered patch" 'RECON_REFUSE: payload PAYLOAD_SEED' yes "$H"

say "-- I3 omitted patch: the seed payload is absent entirely"
EXTRA_SETS=("PAYLOAD_SEED=")
H=$(mkharness omitted); runh "I3 omitted patch" 'did not decompress' yes "$H"

say "-- I4 reordered patch: seed applied before the repair"
EXTRA_SETS=(); mkdir -p "$W/reordered"
python3 - "$W/recon_block.sh" "$W/reordered/recon_block.sh" <<'PY'
import sys,re
src=open(sys.argv[1]).read()
# Locate the two apply legs by what they ARE, not by the cap/grace arguments they happen to carry: a fixture that
# breaks when a grace VARIABLE is renamed is testing its own string, not the frame's gate.  The leg boundaries are the
# two `caprun apply-*` calls and the seed-ok echo that closes the seed leg.
a=src.index("caprun apply-repair ")
b=src.index("caprun apply-seed ", a)
c=src.index("\necho \"seed ok tree=", b)
assert src.count("caprun apply-repair ")==1 and src.count("caprun apply-seed ")==1, "expected exactly one apply leg of each kind"
repair=src[a:b]; seed=src[b:c]
assert 'repair.patch' in repair and 'seed.patch' in seed and 'BASE_TREE' in repair and 'PIN_TREE' in seed
assert 'git apply --index' in repair and 'git apply --index' in seed
# swap the two apply legs, keeping their tree assertions attached to their own leg
out=src[:a]+seed.replace('$SEEDP_SHA','$SEEDP_SHA')+repair+src[c:]
assert out != src and out.count("caprun apply-seed ")==1 and out.count("caprun apply-repair ")==1, "the swap did not happen"
assert out.index("caprun apply-seed ") < out.index("caprun apply-repair "), "the seed leg is not first after the swap"
# the two verification lines must travel with their patch: strip them from the moved seed leg and re-attach
open(sys.argv[2],'w').write(out)
PY
RECON_SRC="$W/reordered/recon_block.sh"; H=$(mkharness reordered); RECON_SRC=""
runh "I4 reordered patch" 'RECON_REFUSE' yes "$H"

say "-- I8 the sha gate itself: a WELL-FORMED gzip payload whose bytes are not the registered patch"
# I2/I7 corrupt the base64 stream, so they legitimately die at the DECOMPRESS gate: that proves the decompress
# check, not the sha256 assertion. This case re-packs a patch that decompresses fine (valid patch content, one
# comment line longer), so ONLY the registered sha256 gate can refuse it -- the gate that matters when a
# plausible-looking patch is swapped in. Generator: build/bad_seed_payload.py (deterministic gzip, mtime=0).
BADPAY=$(python3 "$SELF_DIR/build/bad_seed_payload.py")
[ -n "$BADPAY" ] || { say "BAD  could not build the well-formed wrong-bytes payload"; exit 2; }
EXTRA_SETS=("PAYLOAD_SEED=$BADPAY")
H=$(mkharness sha-gate); runh "I8 sha gate (well-formed gzip, wrong bytes)" 'RECON_REFUSE: payload PAYLOAD_SEED sha256' yes "$H"

say "-- I5 wrong tree expectation: claim the base tree is the seed tree"
EXTRA_SETS=("PIN_TREE=5a14ac7ca50f43312760f092c86ebee7c679ce37")
H=$(mkharness wrong-tree); runh "I5 wrong tree" 'RECONSTRUCTED TREE MISMATCH' yes "$H"

say "-- I6 wrong main-tree expectation (the first gate must refuse, before any patch is read)"
EXTRA_SETS=("MAIN_TREE=0000000000000000000000000000000000000000")
H=$(mkharness wrong-maintree); runh "I6 wrong main tree" 'RECON_REFUSE: tree of' yes "$H"

say "-- I7 the repair payload alone altered (both payloads are independently load-bearing)"
EXTRA_SETS=("PAYLOAD_REPAIR=\$(printf '%s' \"\$PAYLOAD_REPAIR\" | sed 's/./Q/1')")
H=$(mkharness altered-repair); runh "I7 altered repair" 'RECON_REFUSE: payload PAYLOAD_REPAIR' yes "$H"

say ""
# THE CASE LEDGER: what ran must equal what is declared.
[ "$((pass+bad))" = "$EXPECTED_CASES" ] || { bad=$((bad+1)); say "BAD  CASES: ran $((pass+bad)) of $EXPECTED_CASES declared cases -- a shortfall is a control failure, not a smaller suite"; }
say "CASES: declared $EXPECTED_CASES, ran $((pass+bad))"
say "RECONSTRUCTION FIXTURE: OK=$pass BAD=$bad"
[ "$bad" = 0 ] || exit 1
