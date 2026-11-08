#!/usr/bin/env bash
# hermetic_e2e_v1.sh -- ONE end-to-end hermetic run of the FROZEN packet frame, with three expected single-OUTCOME
# results. Written in response to the v4 review's blocking finding that no control exercised the frame as a whole:
# every earlier fixture tested extracted pieces (gates, argv, outcome machinery, reconstruction), and two frame
# defects (a top-level call before its definitions; a post-run predicate that can never hold) therefore survived
# every green receipt and only appeared when the frame was actually executed.
#
# What is real here and what is stood in for, stated precisely:
#   REAL      the frame itself, byte for byte. Its sha256 is checked before the run and again after every case; a
#            moved hash fails the control, so the fixture cannot quietly test an edited copy.
#   REAL      the reconstruction: git init, fetch, checkout, both embedded payload decodes, both `apply --index`
#            steps and the tree-identity gate that must land on the frozen seed tree.
#   STOOD IN  the cgroup. The frame refuses unless the cgroup it is IN carries the booked limits. This container
#            cannot create one (/sys/fs/cgroup is read-only, unshare is refused), so an LD_PRELOAD path mapping
#            presents the booked values as files -- the environment is faked, the frame is not. See cgroup_shim.c.
#   STOOD IN  cargo and claim_batch. No build and no claim executor may run outside the grant, and this control is
#            not the grant. The stub cargo writes a claim_batch that emits the same log SHAPE the frame parses
#            (verdict lines, one [process-partition] total_ms= completion line) for exactly the roster it is given.
#            The stub is therefore blind to program semantics: it proves the frame's plumbing, not the verdicts.
#   STOOD IN  the network. `url.<base>.insteadOf` in a private HOME rewrites the pinned GitHub URL to a local bare
#            repo, so the frame's own `git fetch --depth 1 --no-tags origin <sha>` runs unmodified against bytes
#            derived from a repo that contains the pinned commit.
#
# Cases (all three must produce EXACTLY ONE OUTCOME line, which is the contract a collector reads):
#   pass      stub says every witness PASS          -> OUTCOME: PASS rc=0, JOIN_MISSING=0 FAILS=0
#   fail      stub says one witness FAIL            -> OUTCOME: FAIL rc=1, JOIN_MISSING=0 FAILS=1
#   infra     cargo itself fails (exit 101)         -> OUTCOME: NO_OBSERVATION rc=2 (no binary to run)
# Inversions (each must refuse for its OWN named reason, with no work done):
#   i-gate    one booked limit wrong (memory.max)   -> REFUSED_SETUP before any fetch
#   i-sha     the frame bytes altered               -> the CONTROL refuses to run at all
#   i-fetch   the local remote lacks the pinned sha -> RECON_REFUSE at the fetch
#
# Usage: hermetic_e2e_v1.sh <frame> <frame-sha256> [case ...]     (no args = all six)
# Exit:  0 all expectations met, 1 a merit expectation failed, 2 a missing/unreadable input (never a merit claim).
set -uo pipefail

FRAME=${1:-}; WANT_SHA=${2:-}
if [ -z "$FRAME" ] || [ ! -f "$FRAME" ]; then echo "ABORT (exit 2): frame missing: '$FRAME'"; exit 2; fi
case "$WANT_SHA" in
  ''|*[!0-9a-f]*) echo "ABORT (exit 2): second argument must be the frame's sha256 (64 hex)"; exit 2 ;;
esac
[ "${#WANT_SHA}" = 64 ] || { echo "ABORT (exit 2): frame sha must be 64 hex, got ${#WANT_SHA} chars"; exit 2; }

SRC_REPO=${SRC_REPO:-/var/tmp/freeze2-cut-15695/repo}
[ -d "$SRC_REPO" ] || { echo "ABORT (exit 2): SRC_REPO ($SRC_REPO) is not a git checkout to derive the local remote from"; exit 2; }
SHIM_SRC=${SHIM_SRC:-}
if [ -z "$SHIM_SRC" ]; then
  here=$(cd "$(dirname "$0")" && pwd)
  for cand in "$here/build/cgroup_shim.c" "$here/cgroup_shim.c"; do
    [ -f "$cand" ] && { SHIM_SRC=$cand; break; }
  done
fi
[ -n "$SHIM_SRC" ] && [ -f "$SHIM_SRC" ] || { echo "ABORT (exit 2): shim source missing (looked next to the control and in ./build)"; exit 2; }
MAIN_COMMIT=7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9   # the frame's own pin; kept in sync by the packet builder
UNIT=cc1-15695-a0de-v18.scope

FRAME=$(cd "$(dirname "$FRAME")" && pwd)/$(basename "$FRAME")   # the cases run in their own cwd; the frame must be absolute
W=$(mktemp -d "${TMPDIR:-/var/tmp}/hermetic-e2e.XXXXXXXX") || { echo "ABORT (exit 2): mktemp failed"; exit 2; }
chmod 700 "$W"
cleanup() { [ "${KEEP:-0}" = 1 ] || rm -rf "$W"; }
trap cleanup EXIT
SHARE=$W/share; mkdir -p "$SHARE"

OK=0; BAD=0
ok()  { OK=$((OK+1));  echo "  OK   $1"; }
bad() { BAD=$((BAD+1)); echo "  BAD  $1"; }
chk() { if [ "$1" = 0 ]; then ok "$2"; else bad "$2"; fi; }

# ---------------------------------------------------------------- the frozen bytes are the thing under test
GOT_SHA=$(sha256sum -- "$FRAME" | cut -d' ' -f1)
echo "frame: $FRAME"
echo "frame sha256 $GOT_SHA (pinned $WANT_SHA)"
if [ "$GOT_SHA" != "$WANT_SHA" ]; then
  echo "FRAME-SHA-MISMATCH: refusing to run anything (exit 2). A control that runs whatever bytes it is handed"
  echo "cannot certify a frozen artifact."
  exit 2
fi

# ---------------------------------------------------------------- the fake cgroup: the booked values, as files
# The frame derives its cgroup directory as /sys/fs/cgroup<what /proc/self/cgroup says after 0::>, so the fake
# tree mirrors that shape exactly: the unit sits two levels down, under a parent, the way a keeper's slice does.
CG=$SHARE/cg; CGDIR=$CG/root/fake/$UNIT; mkdir -p "$CGDIR"
printf '0::/fake/%s\n' "$UNIT" > "$CG/proc-cgroup"
printf '%s\n' 34359738368      > "$CGDIR/memory.max"
printf '%s\n' 30064771072      > "$CGDIR/memory.high"
printf '%s\n' 0                > "$CGDIR/memory.swap.max"
printf '%s\n' 4096             > "$CGDIR/pids.max"
printf '%s\n' '3200000 100000' > "$CGDIR/cpu.max"
printf '%s\n' 1073741824       > "$CGDIR/memory.current"
printf '%s\n' 5368709120       > "$CGDIR/memory.peak"
printf '%s\n' 0                > "$CGDIR/memory.swap.current"
{ printf 'low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n'; } > "$CGDIR/memory.events"
printf '%s\n' 'memory pids cpu io' > "$CG/root/cgroup.controllers"

gcc -O2 -fPIC -shared -o "$SHARE/cgroup_shim.so" "$SHIM_SRC" -ldl || { echo "ABORT (exit 2): cannot build $SHIM_SRC"; exit 2; }
echo "shim: $(sha256sum -- "$SHARE/cgroup_shim.so" | cut -d' ' -f1) from $SHIM_SRC ($(wc -c < "$SHIM_SRC") bytes)"

# ---------------------------------------------------------------- the local stand-in for GitHub
git clone -q --bare "$SRC_REPO" "$SHARE/fake-remote.git" || { echo "ABORT (exit 2): cannot clone $SRC_REPO to make the local remote"; exit 2; }
git --git-dir="$SHARE/fake-remote.git" cat-file -e "$MAIN_COMMIT^{commit}" 2>/dev/null \
  || { echo "ABORT (exit 2): the source repo does not contain the pinned main commit $MAIN_COMMIT"; exit 2; }
git --git-dir="$SHARE/fake-remote.git" update-ref refs/heads/pinned "$MAIN_COMMIT"
git --git-dir="$SHARE/fake-remote.git" config uploadpack.allowReachableSHA1InWant true
HOME_DIR=$SHARE/home; mkdir -p "$HOME_DIR"
cat > "$HOME_DIR/.gitconfig" <<EOF
[user]
	name = hermetic-e2e
	email = hermetic-e2e@invalid
[url "file://$SHARE/fake-remote.git"]
	insteadOf = https://github.com/gunb-ai/gunbc
EOF
echo "local remote: file://$SHARE/fake-remote.git (rewritten from the pinned URL by insteadOf)"

# ---------------------------------------------------------------- the stubs: cargo, and a claim_batch it manufactures
BIN=$SHARE/bin; mkdir -p "$BIN"
cat > "$BIN/cargo" <<'STUB'
#!/usr/bin/env bash
# Stub cargo for the hermetic end-to-end control. It does NOT build anything: a build would run the program under
# test outside its grant. It manufactures a claim_batch that emits the log SHAPE the frame parses -- one verdict
# line per function it is handed, exactly one [process-partition] total_ms= completion line -- for the roster in
# its own argv. So the frame's reconstruction, gates, caps, exit-code contract and verdict JOIN are exercised;
# program semantics deliberately are not.
case "${STUB_MODE:-pass}" in
  infra)
    echo "error[E0433]: cannot find module (stub: the toolchain itself failed)" >&2
    exit 101 ;;
esac
TARGET=${CARGO_TARGET_DIR:?stub cargo expects CARGO_TARGET_DIR}
mkdir -p "$TARGET/release"
cat > "$TARGET/release/claim_batch" <<'CLAIM'
#!/usr/bin/env bash
FUNCS=""; ENTRY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --functions) FUNCS=${2:-}; shift 2 ;;
    --entry)     ENTRY=${2:-};  shift 2 ;;
    *) shift ;;
  esac
done
IFS=, read -r -a NAMES <<< "$FUNCS"
echo "memory-cgroup-bind: not requested"
echo "[floor-phase] phase=bare-reference-edge-index-warm state=completed cpu_ms=1 wall_ms=1 rss_growth_bytes=0 source_files=1 stub=1"
echo "claim_batch: stub entry=$ENTRY group(s), ${#NAMES[@]} witness(es) total"
i=0
# The FAIL case refuses exactly ONE witness in the rtp entry only. Refusing the first name of EVERY entry (an early
# draft) produced two FAILs and a JOIN of FAILS=2, which is a different expectation, not a frame defect -- the point
# of the case is that a single refused witness yields exactly one OUTCOME: FAIL rc=1 line.
REFUSE=0
if [ "${STUB_MODE:-pass}" = fail ] && [ "${#NAMES[@]}" = 21 ]; then REFUSE=1; fi
for n in "${NAMES[@]}"; do
  [ -n "$n" ] || continue
  if [ "$REFUSE" = 1 ] && [ "$i" = 0 ]; then echo "FAIL $n"; else echo "PASS $n"; fi
  i=$((i+1))
done
echo "[process-partition] total_ms=1234 pre_run_ms=0 args_ms=1 index_build_ms=1 edge_index_warm_ms=1 output_policy_ms=1 roster_ms=1 stub=1"
# the real executor exits 1 when a witness refused and 0 when everything held; the contract allows 0/1/2 only
if [ "$REFUSE" = 1 ]; then exit 1; fi
exit 0
CLAIM
chmod +x "$TARGET/release/claim_batch"
echo "cargo(stub): manufactured claim_batch for CARGO_TARGET_DIR=$TARGET"
exit 0
STUB
chmod +x "$BIN/cargo"

# ---------------------------------------------------------------- one case = one whole run of the frozen frame
run_case() { # run_case <name> <stub-mode> [--cg-file <file> <content>] ...
  local name=$1 mode=$2; shift 2
  local cw=$SHARE/case-$name; mkdir -p "$cw"
  LAST_CASE=$name
  local cg=$CG
  if [ "$name" = i-gate ]; then           # inversion: the host carries the WRONG booked limit
    cg=$cw/cg; cp -r "$CG" "$cw/cg"; printf '%s\n' 25769803776 > "$cw/cg/root/fake/$UNIT/memory.max"
  fi
  local work=$cw/work; mkdir -p "$work"
  local log=$cw/run.log
  local home=$HOME_DIR
  [ "$name" = i-fetch ] && home=$HOME2      # the empty-remote inversion points the same URL somewhere else
  echo "--- case $name (stub=$mode)"
  ( cd "$work" && env -i \
      HOME="$home" PATH="$BIN:/usr/bin:/bin" LANG=C LC_ALL=C \
      FAKE_CG_ROOT="$cg" LD_PRELOAD="$SHARE/cgroup_shim.so" \
      STUB_MODE="$mode" TMPDIR="$cw" \
      bash "$FRAME" "$work/out" ) > "$log" 2>&1
  local rc=$?
  local n_out; n_out=$(grep -c '^OUTCOME: ' "$log")
  local line;  line=$(grep -m1 '^OUTCOME: ' "$log")
  echo "  rc=$rc outcomelines=$n_out ${line:+| $line}"
  LAST_LOG=$log; LAST_RC=$rc; LAST_N=$n_out; LAST_LINE=$line
  # the frame bytes must survive the run untouched
  local after; after=$(sha256sum -- "$FRAME" | cut -d' ' -f1)
  [ "$after" = "$WANT_SHA" ] || bad "$name: the frame's sha256 MOVED during the run ($after)"
  return 0
}

expect_outcome() { # expect_outcome <name> <expected full line prefix>
  local name=$1 want=$2
  [ "$LAST_N" = 1 ] && chk 0 "$name: exactly one OUTCOME line" || bad "$name: $LAST_N OUTCOME lines (contract wants exactly 1)"
  case "$LAST_LINE" in
    "$want"*) chk 0 "$name: $want" ;;
    *) bad "$name: expected '$want', got '${LAST_LINE:-<none>}'" ;;
  esac
}
has()     { grep -aq "$1" "$LAST_LOG" && chk 0 "$2" || bad "$2 (no line matching '$1')"; }
hasnt()   { grep -aq "$1" "$LAST_LOG" && bad "$2 (found '$1')" || chk 0 "$2"; }
count_is(){ local n; n=$(grep -ac "$1" "$LAST_LOG"); [ "$n" = "$2" ] && chk 0 "$3 (= $2)" || bad "$3: got $n, want $2"; }
count_dir(){ # count_dir <ere> <expected> <label>: count across EVERY file the case produced, because phases that
             # redirect into build.log / claims-*.log keep their own receipts there
  local n; n=$(grep -rahE "$1" "$SHARE/case-$LAST_CASE" 2>/dev/null | wc -l)
  [ "$n" = "$2" ] && chk 0 "$3 (= $2)" || bad "$3: got $n, want $2"
}

CASES=("${@:3}")
[ ${#CASES[@]} -gt 0 ] || CASES=(pass fail infra i-gate i-sha i-fetch)

for c in "${CASES[@]}"; do
  case "$c" in
    pass)
      run_case pass pass
      expect_outcome pass "OUTCOME: PASS rc=0"
      has '^main ok commit='        'pass: the pinned main commit and tree verified'
      has 'precheck: group_kill=1'  'pass: the kill ladder proved it can reach a whole process group BEFORE any work ran'
      # Every bounded phase must name the mechanism in its receipt. Three of them (build, entry-rtp, entry-ce)
      # redirect their output into their own log files, so their receipts live there and not in run.log: the count
      # is taken across the case directory, which is what a collector reading the artifacts would do.
      count_dir 'rc=[0-9]* (.*group-kill=1)' 11 'pass: all 11 bounded phases report the group-kill mechanism'
      count_dir 'rc=[0-9]* (.*group-kill=0)' 0  'pass: no phase silently degraded to a single-pid kill'
      has '^base ok tree='          'pass: the accepted base tree was reconstructed'
      has '^seed ok tree='          'pass: the frozen seed tree was reconstructed'
      has 'claim_batch sha'         'pass: the executor was hashed'
      count_is '^PASS ' 34          'pass: 34 verdict lines produced (21 rtp + 13 ce)'
      has 'JOIN_MISSING=0 FAILS=0'  'pass: the verdict JOIN saw nothing missing and nothing refused'
      ;;
    fail)
      run_case fail fail
      expect_outcome fail "OUTCOME: FAIL rc=1"
      count_is '^FAIL ' 1           'fail: exactly one refused witness'
      has 'JOIN_MISSING=0 FAILS=1'  'fail: the JOIN counted the refusal'
      hasnt 'REFUSED_SETUP'         'fail: no setup refusal (a merit FAIL, not an infra one)'
      ;;
    infra)
      run_case infra infra
      expect_outcome infra "OUTCOME: NO_OBSERVATION rc=2"
      has 'cargo_exit=101'          'infra: the build failure is recorded with its own rc'
      hasnt 'claim_batch sha'       'infra: no executor was hashed, so no verdicts can be claimed'
      count_is '^(PASS|FAIL) ' 0    'infra: zero verdict lines'
      ;;
    i-gate)
      run_case i-gate pass
      expect_outcome i-gate "OUTCOME: NO_OBSERVATION rc=2"
      has 'REFUSED_SETUP'           'i-gate: the scope gate refused'
      hasnt '^main ok commit='      'i-gate: refused BEFORE any fetch or build'
      ;;
    i-sha)
      echo "--- case i-sha (inversion: the frame bytes do not match the pin)"
      cp "$FRAME" "$SHARE/alt.sh"
      printf '\n# altered\n' >> "$SHARE/alt.sh"
      out=$(bash "$0" "$SHARE/alt.sh" "$WANT_SHA" 2>&1); rc=$?
      [ "$rc" = 2 ] && chk 0 'i-sha: the control refuses an altered frame with exit 2' \
                     || bad "i-sha: exit $rc, want 2 -- output: $(printf '%s' "$out" | head -1)"
      printf '%s\n' "$out" | grep -aq 'FRAME-SHA-MISMATCH' && chk 0 'i-sha: the refusal names the reason' \
                                                           || bad 'i-sha: the refusal is unattributed'
      ;;
    i-fetch)
      echo "--- case i-fetch (inversion: the remote does not hold the pinned commit at all)"
      # A second, EMPTY bare remote with its own HOME: the same pinned URL is rewritten to a repository with no
      # objects at all, so the frame's own fetch must refuse. Deleting only the ref would prove nothing -- with
      # allowReachableSHA1InWant the commit stays reachable from the clone's other refs and would still fetch.
      git init -q --bare "$SHARE/empty-remote.git"
      HOME2=$SHARE/home2; mkdir -p "$HOME2"
      sed "s#$SHARE/fake-remote.git#$SHARE/empty-remote.git#" "$HOME_DIR/.gitconfig" > "$HOME2/.gitconfig"
      run_case i-fetch pass
      expect_outcome i-fetch "OUTCOME: NO_OBSERVATION rc=2"
      has 'RECON_REFUSE'            'i-fetch: the reconstruction refused at the fetch'
      hasnt 'base ok tree='         'i-fetch: refused before any patch was applied'
      ;;
    *) echo "ABORT (exit 2): unknown case '$c'"; exit 2 ;;
  esac
done

echo
echo "HERMETIC OK=$OK BAD=$BAD  (work dir ${KEEP:+kept at }$W)"
[ "$BAD" = 0 ] || exit 1
exit 0
