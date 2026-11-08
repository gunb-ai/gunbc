#!/usr/bin/env bash
# hermetic_e2e_v2.sh -- the whole-frame control, revised for the v6 review findings. What v1 could NOT see, one
# line each: (N1) v1 counted receipt strings and no stub ever overran a cap, so a cap that killed nothing still
# printed 'group-kill=1' -- v2 runs the FROZEN caps against a toolchain that hangs on purpose, measures the wall,
# and then asks whether the RECORDED pids (the toolchain's own and a descendant it spawned) are actually dead;
# (N2) v1's pass case always had the executor exit 0, so an all-PASS batch exiting 1 or 2 read as PASS -- v2 forces
# those statuses; (m1) a gate refusal must be attributed to the gates phase with its real code, never as a signal
# death. Every case keeps its RAW log, and the overrun case keeps the pid record it asserts on.
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
# hermetic_e2e_v4.sh -- v3 plus gate G2's whole-frame PIPE case (`pipe-quiet`), which asserts that the OUTCOME
# line is the frame's LAST output. v3's sig-pipe case is kept: it covers the single line, the rc agreement and the
# dead phase tree, and it is NOT evidence for termination -- measured, deleting on_pipe's `exit 2` leaves the v3
# suite at OK=103 BAD=0 (tidy-ant-251 msg_c1243b39). The G2 receipt is the one that certifies termination, and the
# mutant that must turn pipe-quiet RED is produced by build/mk_pipe_mutant.py and driven by build/pipe_g2_negctl.sh.
set -uo pipefail

HARNESS_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)   # for the helpers this suite ships with
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
  overrun)
    # N1 control: the toolchain hangs. It records its OWN pid and the pid of a process it SPAWNS, because the
    # assertion that matters is not what the frame printed about the kill but whether BOTH are dead afterwards.
    echo "cargo(stub): hanging on purpose; pids recorded for the liveness check" >&2
    echo $$ > "$TMPDIR/overrun.pids"
    # The GROUP id, not just the pids: the v7 signal cases assert that NOTHING in the phase's process group
    # survives the handler, and a pgid is the only thing that names a tree.
    awk '{print $5}' "/proc/$$/stat" > "$TMPDIR/overrun.pgid"
    sleep 900 &
    echo $! >> "$TMPDIR/overrun.pids"
    # The signal cases need to know the phase is really running before they signal it; signalling a frame
    # that has not launched a phase yet would test the empty case and pass for the wrong reason.
    echo started > "$TMPDIR/stub.started"
    wait ;;
  infra)
    echo "error[E0433]: cannot find module (stub: the toolchain itself failed)" >&2
    exit 101 ;;
  killfail)
    # i-r125: the COMMAND exits 125 on its own. 125 is also what the frame's own kill ladder returns when it
    # cannot clear a tree, so the receipt must say which of the two happened -- and here it is the command,
    # with no KILL_FAILED line above it.
    echo "cargo(stub): exiting 125 on purpose, without any kill ladder involved" >&2
    exit 125 ;;
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
# N2 control: CLAIM_RC forces the executor exit status while every verdict line still reads PASS. A batch that
# refuses to account for its own cost (1) or reports no observation (2) prints good-looking text, and the frame
# must not turn that text into a PASS.
if [ -n "${CLAIM_RC:-}" ]; then exit "$CLAIM_RC"; fi
exit 0
CLAIM
chmod +x "$TARGET/release/claim_batch"
echo "cargo(stub): manufactured claim_batch for CARGO_TARGET_DIR=$TARGET"
exit 0
STUB
chmod +x "$BIN/cargo"
cat > "$BIN/git" <<'GITSHIM'
#!/usr/bin/env bash
# A git stand-in for the i-ignterm case only. Every subcommand passes through to the real git EXCEPT
# `checkout` when GIT_SHIM_IGNTERM=1, where it ignores SIGTERM and hangs, recording its pid, a descendant
# and its process-group id. The point: the checkout site is bounded by CAP_CHECKOUT with GRACE_LOCAL=3, and
# the only way to prove the ladder escalates group-TERM -> group-KILL inside 3 seconds is a LOCAL process
# that refuses to stop on TERM. Everything else in the run is real git.
if [ "${GIT_SHIM_IGNTERM:-0}" = 1 ] && [ "${1:-}" = checkout ]; then
  trap '' TERM
  echo "$$" > "$TMPDIR/ignterm.pids"
  sleep 900 &
  echo $! >> "$TMPDIR/ignterm.pids"
  awk '{print $5}' "/proc/$$/stat" > "$TMPDIR/ignterm.pgid"
  echo "git(shim): checkout refuses SIGTERM; pids recorded" >&2
  wait
fi
exec /usr/bin/git "$@"
GITSHIM
chmod +x "$BIN/git"

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
  local t0=$(date +%s)
  # The forced executor status for the N2 cases. It has to be decided BEFORE the launch: the case-dispatch
  # block below runs after the frame has already run, so setting it there would be setting it too late.
  local claim_rc=""
  case "$name" in pass-rc1) claim_rc=1 ;; pass-rc2) claim_rc=2 ;; esac
  local ignterm=0
  [ "$name" = i-ignterm ] && ignterm=1
  local log=$cw/run.log
  local home=$HOME_DIR
  [ "$name" = i-fetch ] && home=$HOME2      # the empty-remote inversion points the same URL somewhere else
  echo "--- case $name (stub=$mode)"
  ( cd "$work" && env -i \
      HOME="$home" PATH="$BIN:/usr/bin:/bin" LANG=C LC_ALL=C \
      FAKE_CG_ROOT="$cg" LD_PRELOAD="$SHARE/cgroup_shim.so" \
      STUB_MODE="$mode" CLAIM_RC="$claim_rc" GIT_SHIM_IGNTERM="$ignterm" TMPDIR="$cw" \
      bash "$FRAME" "$work/out" ) > "$log" 2>&1
  local rc=$?
  LAST_WALL=$(( $(date +%s) - t0 ))
  local n_out; n_out=$(grep -c '^OUTCOME: ' "$log")
  local line;  line=$(grep -m1 '^OUTCOME: ' "$log")
  echo "  rc=$rc outcomelines=$n_out ${line:+| $line}"
  LAST_LOG=$log; LAST_RC=$rc; LAST_N=$n_out; LAST_LINE=$line
  # the frame bytes must survive the run untouched
  local after; after=$(sha256sum -- "$FRAME" | cut -d' ' -f1)
  [ "$after" = "$WANT_SHA" ] || bad "$name: the frame's sha256 MOVED during the run ($after)"
  return 0
}
# local remote), but in the background, and the signal is sent only once the build phase is demonstrably
# running -- the stub announces itself by writing stub.started, and the recorded pid/pgid files exist.
run_case_sig() { # run_case_sig <name> <signal>   (signal as kill accepts it: PIPE, TERM, INT, HUP)
  local name=$1 sig=$2
  local cw=$SHARE/case-$name; mkdir -p "$cw"
  LAST_CASE=$name
  local work=$cw/work; mkdir -p "$work"; local log=$cw/run.log
  local t0=$(date +%s)
  echo "--- case $name (signal=$sig, stub=overrun: a build phase that is still running when the signal lands)"
  # exec_reset.py first, and this is not decoration: a suite launched under nohup (which is how a sweep starts)
  # has SIGHUP IGNORED, bash inherits that into the frame, and bash cannot trap a signal it entered ignored --
  # measured, the SIGTERM/INT/PIPE cases passed under nohup while the HUP case did nothing at all until the build
  # cap fired. A signal case must make the disposition testable before it sends the signal.
  ( cd "$work" && exec python3 "$HARNESS_DIR/build/exec_reset.py" -- env -i \
      HOME="$HOME_DIR" PATH="$BIN:/usr/bin:/bin" LANG=C LC_ALL=C \
      FAKE_CG_ROOT="$CG" LD_PRELOAD="$SHARE/cgroup_shim.so" \
      STUB_MODE=overrun TMPDIR="$cw" bash "$FRAME" "$work/out" ) > "$log" 2>&1 &
  local fpid=$! w=0
  until [ -f "$cw/stub.started" ] || [ "$w" -ge 150 ]; do sleep 1; w=$((w+1)); done
  [ -f "$cw/stub.started" ] || { echo "  BAD the phase never started; the signal would have tested nothing"; }
  # WHO IS BEING SIGNALLED, and what hangs off it at that moment. Without this a signal case cannot even say what
  # it tested: if the pid behind $! were not the frame's own shell, "no handler ran" would be a harness bug
  # reading as a frame bug. The snapshot is part of the receipt.
  echo "  signal target: $(ps -o pid=,pgid=,ppid=,stat=,comm= -p "$fpid" 2>/dev/null || echo 'the pid is gone already')"
  echo "  children of the target at signal time: $(ps -eo pid=,pgid=,ppid=,comm= | awk -v p="$fpid" '$3==p' | tr '\n' ';')"
  kill -"$sig" "$fpid" 2>/dev/null || echo "  BAD could not send SIG$sig to pid $fpid"
  wait "$fpid"; local rc=$?
  LAST_WALL=$(( $(date +%s) - t0 ))
  local n_out; n_out=$(grep -c '^OUTCOME: ' "$log")
  local line; line=$(grep -m1 '^OUTCOME: ' "$log")
  echo "  rc=$rc outcomelines=$n_out ${line:+| $line}"
  LAST_LOG=$log; LAST_RC=$rc; LAST_N=$n_out; LAST_LINE=$line
  local after; after=$(sha256sum -- "$FRAME" | cut -d' ' -f1)
  [ "$after" = "$WANT_SHA" ] || bad "$name: the frame's sha256 MOVED during the run ($after)"
  return 0
}

rc_agrees() { # rc_agrees <name> <rc-in-the-OUTCOME-line> -- the process status and the receipt must agree
  local name=$1 want=$2
  if [ "$LAST_RC" = "$want" ]; then
    chk 0 "$name: the process exit code is $want, equal to the rc its OUTCOME line states"
  else
    bad "$name: process exit $LAST_RC disagrees with the OUTCOME line, which states rc $want"
  fi
}
outcome_final() { # outcome_final <name> -- G2: the OUTCOME line is the LAST thing the frame wrote
  local name=$1 log=$LAST_LOG n after body
  n=$(grep -n '^OUTCOME: ' "$log" | tail -1 | cut -d: -f1)
  [ -n "$n" ] || { bad "$name: no OUTCOME line to locate; the assertion below cannot mean anything"; return; }
  after=$(awk -v n="$n" 'NR>n' "$log" | grep -v '^[[:space:]]*$' || true)
  if [ -z "$after" ]; then
    ok "$name: the OUTCOME line is the final output -- the run ended where its own receipt said it ended"
  else
    body=$(printf '%s\n' "$after" | head -6 | tr '\n' '|')
    bad "$name: $(printf '%s\n' "$after" | grep -c '') line(s) of frame output AFTER the OUTCOME line: $body"
  fi
}
pids_gone() { # pids_gone <name> <pids-file> -- no RECORDED pid may still be running
  local name=$1 f=$SHARE/case-$1/$2
  if [ ! -f "$f" ]; then bad "$name: no recorded pids at $f -- the case never proved a tree existed"; return; fi
  if rec_alive "$f"; then bad "$name: a recorded phase pid is STILL RUNNING after the handler ran"; else
    chk 0 "$name: every recorded phase pid is dead ($(wc -l < "$f") recorded)"; fi
}
group_gone() { # group_gone <name> <pgid-file> -- nothing in the whole system may still carry that group id
  local name=$1 f=$SHARE/case-$1/$2 pg p pp
  [ -f "$f" ] || { bad "$name: no recorded pgid at $f"; return; }
  pg=$(cat "$f")
  for p in /proc/[0-9]*; do
    pp=$(awk '{print $5}' "$p/stat" 2>/dev/null) || continue
    if [ "$pp" = "$pg" ]; then
      case "$(awk '{print $3}' "$p/stat" 2>/dev/null)" in Z|X) continue ;; esac
      bad "$name: pid ${p#/proc/} still carries process group $pg after the handler ran"; return
    fi
  done
  chk 0 "$name: no live process carries process group $pg -- the whole tree the phase spawned is gone"
}

# ---------------------------------------------------------------- the assertions, all against the raw log

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

rec_alive() { # rec_alive <file of recorded pids> -- true if any RECORDED pid is still running (a zombie is not)
  local f=$1 p st
  [ -f "$f" ] || { echo "    (no recorded pids at $f)"; return 1; }
  while read -r p; do
    case "$p" in ''|*[!0-9]*) continue ;; esac
    st=$(awk '{print $3}' "/proc/$p/stat" 2>/dev/null) || continue
    case "$st" in Z|X) continue ;; *) echo "    recorded pid $p is still running (state $st)"; return 0 ;; esac
  done < "$f"
  return 1
}
CASES=("${@:3}")
[ ${#CASES[@]} -gt 0 ] || CASES=(pass fail infra i-gate i-sha i-fetch o-build pass-rc1 pass-rc2 sig-pipe pipe-quiet sig-term sig-int sig-hup i-ignterm i-r125)

# A control that can silently shrink is not a control.  Every known case must be in the declared list, and an
# unknown name aborts the whole run (below), so the list can neither lose a case nor invent one.
KNOWN=(pass fail infra i-gate i-sha i-fetch o-build pass-rc1 pass-rc2 sig-pipe pipe-quiet sig-term sig-int sig-hup i-ignterm i-r125)
SUITE=1
if [ "${#CASES[@]}" != "${#KNOWN[@]}" ]; then
  if [ "${PARTIAL_OK:-0}" = 1 ]; then SUITE=0
    echo "PARTIAL RUN: ${#CASES[@]} of ${#KNOWN[@]} cases -- this receipt is NOT a suite receipt, and no
    summary line may be read as one. A full suite runs with PARTIAL_OK unset."
  else echo "BAD  the case list has ${#CASES[@]} entries, want ${#KNOWN[@]}: a dropped case is not a smaller suite"; BAD=$((BAD+1)); fi
fi
# Every declared case must have been dispatched -- this is what stops a case vanishing from a SUITE receipt.
# In a PARTIAL run the omissions are explicit, announced, and the summary says NOT a suite receipt, so the
# check would only be noise; the guarantee that survives is that a partial receipt can never be read as one.
if [ "$SUITE" = 1 ]; then
  for k in "${KNOWN[@]}"; do f=0; for c in "${CASES[@]}"; do [ "$c" = "$k" ] && f=1; done; [ "$f" = 1 ] || { echo "BAD  case '$k' is missing from the declared list"; BAD=$((BAD+1)); }; done
fi
RAN=0
for c in "${CASES[@]}"; do
  RAN=$((RAN+1))
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
      has 'EXECUTOR_STATUS: every entry must exit 0 for a PASS' 'pass: the executor statuses are stated beside the verdict text'
      hasnt 'unspecified'           'pass: no terminal line carries a placeholder reason'
      ;;
    fail)
      run_case fail fail
      expect_outcome fail "OUTCOME: FAIL rc=1"
      count_is '^FAIL ' 1           'fail: exactly one refused witness'
      has 'JOIN_MISSING=0 FAILS=1'  'fail: the JOIN counted the refusal'
      hasnt 'REFUSED_SETUP'         'fail: no setup refusal (a merit FAIL, not an infra one)'
      ;;
    o-build)
      # N1, AT THE FROZEN CAPS. The build phase is bounded by CAP_BUILD (454 s) with GRACE_BUILD (15 s); the stub
      # toolchain hangs for 900 s and spawns a descendant of its own. The frame must cut the WHOLE TREE inside
      # cap+grace, say so exactly once, and leave nothing running. The wall and the liveness of the recorded pids
      # are the evidence; the receipt text only has to agree with them. This case is slow BY CONSTRUCTION -- it is
      # the real cap being enforced, which is the only honest way to test enforcement in the frozen bytes.
      run_case o-build overrun
      expect_outcome o-build 'OUTCOME: NO_OBSERVATION'
      hasnt 'KILL_FAILED'                     'o-build: the kill did not fail -- the tree was reached'
      hasnt 'claim_batch_exit'                'o-build: no entry ran, because the build never finished'
      hasnt 'OUTCOME: PASS'                   'o-build: an unfinished build is never a result'
      # THE CAUSE LABEL IS PART OF THE VERDICT. The stage-6 head labelled a real cap expiry as "the COMMAND itself
      # exited 124 -- the cap never fired", because the ladder cleared PHASE_RC_ORIGIN before the caller read it.
      has 'exceeded its 454 s cap and the whole tree it spawned was killed' 'o-build: the OUTCOME names the KILL LADDER as the producer of rc=124, which is what the phase log shows'
      hasnt 'the cap never fired'              'o-build: no clause may say the cap never fired while the phase log carries its TIMEOUT_KILL receipt'
      # NOT `local`: these case branches run at the top level, and a `local` there is an error, not a
      # declaration -- which is what made the two assertions below read as failures on an earlier run.
      receipt=""; fel=""
      receipt=$(grep -ar '\[cap\] phase=build' "$SHARE/case-o-build" 2>/dev/null | tail -1)
      echo "    build-phase receipt: ${receipt:-MISSING}"
      fel=$(printf '%s' "$receipt" | sed -n 's/.*elapsed=\([0-9]*\)s.*/\1/p')
      if [ -n "$fel" ] && [ "$fel" -ge 454 ] && [ "$fel" -le 505 ]; then
        OK=$((OK+1)); echo "OK   o-build: the frame's own elapsed receipt for the build phase is ${fel}s -- inside cap 454 + grace 15 + scheduling slack, NOT the toolchain's 900 s"
      else
        BAD=$((BAD+1)); echo "BAD  o-build: build-phase elapsed is '${fel:-unreadable}', want 454..505 s. A cap that kills nothing reports the CHILD's length instead, which is exactly the v5 defect."
      fi
      if [ "$(grep -arc 'TIMEOUT_KILL' "$SHARE/case-o-build" 2>/dev/null | awk -F: '{t+=$NF} END {print t+0}')" = 1 ]; then
        OK=$((OK+1)); echo "OK   o-build: exactly one kill receipt across the case artifacts"
      else
        BAD=$((BAD+1)); echo "BAD  o-build: the kill receipt count is not 1"
      fi
      if rec_alive "$SHARE/case-o-build/overrun.pids"; then
        BAD=$((BAD+1)); echo "BAD  o-build: a recorded pid is STILL RUNNING after a receipt claiming the tree was killed"
      else
        OK=$((OK+1)); echo "OK   o-build: LIVENESS on the recorded pids -- the hung toolchain AND the descendant it spawned are both gone"
      fi
      echo "    recorded pids: $(tr '\n' ' ' < "$SHARE/case-o-build/overrun.pids" 2>/dev/null || echo none)"
      echo "    case wall: ${LAST_WALL}s"
      ;;
    sig-pipe)
      # v6 BLOCKER: the PIPE handler printed its OUTCOME line and RETURNED, so the frame kept working and could
      # finish with a status that contradicted its own receipt. The case asserts the line, the agreement, and that
      # nothing from the running phase is left.
      run_case_sig sig-pipe PIPE
      expect_outcome sig-pipe "OUTCOME: NO_OBSERVATION rc=2"
      rc_agrees sig-pipe 2
      has 'WALL_KILL: exit'            'sig-pipe: the wall/pipe path is announced before the OUTCOME line'
      pids_gone sig-pipe overrun.pids
      group_gone sig-pipe overrun.pgid
      hasnt 'OUTCOME: PASS'            'sig-pipe: a broken pipe is never a result'
      ;;
    pipe-quiet)
      # GATE G2 (keen-otter msg_2622a1fe; mutant measurement msg_c1243b39). THREE assertions, all on the WHOLE
      # frame: (1) exactly one OUTCOME line and it is NO_OBSERVATION rc=2; (2) the process status equals it;
      # (3) NO frame output follows the OUTCOME line. (3) is the assertion that discriminates deleting
      # on_pipe's `exit 2`: the handler returns, the frame resumes, and its continuation output lands after the
      # receipt it has already announced. v3's sig-pipe case passes under that mutant (measured), so v3 does not
      # cover this and this case is not a restatement of it.
      run_case_sig pipe-quiet PIPE
      expect_outcome pipe-quiet "OUTCOME: NO_OBSERVATION rc=2"
      rc_agrees pipe-quiet 2
      outcome_final pipe-quiet
      pids_gone pipe-quiet overrun.pids
      group_gone pipe-quiet overrun.pgid
      hasnt 'OUTCOME: PASS'            'pipe-quiet: a broken pipe is never a result'
      ;;
    sig-term)
      run_case_sig sig-term TERM
      expect_outcome sig-term 'OUTCOME: NO_OBSERVATION rc=143'
      rc_agrees sig-term 143
      has 'SIGNAL: received TERM'      'sig-term: the handler ran and said which signal it was'
      has 'REAP: phase=build'          'sig-term: the handler named the phase it reaped'
      has 'the phase tree is gone'     'sig-term: the reap verified the tree instead of claiming one'
      pids_gone sig-term overrun.pids
      group_gone sig-term overrun.pgid
      hasnt 'OUTCOME: PASS'            'sig-term: an interrupted run is no observation'
      ;;
    sig-int)
      run_case_sig sig-int INT
      expect_outcome sig-int 'OUTCOME: NO_OBSERVATION rc=130'
      rc_agrees sig-int 130
      has 'REAP: phase=build'          'sig-int: the running phase was reaped, not left running'
      pids_gone sig-int overrun.pids
      group_gone sig-int overrun.pgid
      ;;
    sig-hup)
      run_case_sig sig-hup HUP
      expect_outcome sig-hup 'OUTCOME: NO_OBSERVATION rc=129'
      rc_agrees sig-hup 129
      has 'REAP: phase=build'          'sig-hup: the running phase was reaped, not left running'
      pids_gone sig-hup overrun.pids
      group_gone sig-hup overrun.pgid
      ;;
    i-ignterm)
      # THE RULING'S FIXTURE: a LOCAL operation that ignores SIGTERM. The checkout site carries GRACE_LOCAL=3, so
      # the ladder must be inside cap+3+slack seconds. If the frame still used the old 10 s grace the wall would
      # land near cap+10, and the assertion below would fail -- this is what pins GRACE_LOCAL to the code, not to
      # the table.
      run_case i-ignterm pass
      expect_outcome i-ignterm 'OUTCOME: NO_OBSERVATION'
      receipt=""; receipt=$(grep -ar '\[cap\] phase=checkout' "$SHARE/case-i-ignterm" 2>/dev/null | tail -1)
      echo "    checkout-phase receipt: ${receipt:-MISSING}"
      case "$receipt" in *'cap=45'*'grace=3'*) chk 0 'i-ignterm: the checkout site ran under CAP_CHECKOUT with GRACE_LOCAL=3' ;;
        *) bad "i-ignterm: the checkout receipt does not show cap=45 grace=3: ${receipt:-MISSING}" ;;
      esac
      el=0; el=$(printf '%s' "$receipt" | sed -n 's/.*elapsed=\([0-9]*\)s.*/\1/p')
      if [ -z "$el" ]; then bad 'i-ignterm: no elapsed= in the checkout receipt'; else
        # cap 45 + one 3 s grace + slack, and NOT cap+10: the escalation to KILL is what is being timed.
        if [ "$el" -ge 45 ] && [ "$el" -le 53 ]; then
          chk 0 "i-ignterm: the TERM-ignoring local operation was killed at ${el}s, inside cap 45 + GRACE_LOCAL 3 + slack"
        else
          bad "i-ignterm: killed at ${el}s -- outside cap+GRACE_LOCAL+slack (45..53); the ladder is not spending 3 s"
        fi
      fi
      count_dir 'KILL_FAILED: phase=' 0 'i-ignterm: no failed-kill receipt: the tree was reached'
      has 'TIMEOUT_KILL: phase=checkout cap=45s grace=3s rc=124 killed=yes' 'i-ignterm: the cap receipt states the kill worked, proven by the post-kill scan'
      has 'RECON_REFUSE: checkout of 7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9 failed' 'i-ignterm: the refusal names the local operation the cap cut'
      hasnt 'timeout(124'              'i-ignterm: nothing here is timeout(1)'
      has 'exceeded its 45 s cap and the whole tree it spawned was killed' 'i-ignterm: the OUTCOME names the KILL LADDER as the producer at the local site, and the tree it means includes the process that ignored TERM'
      hasnt 'COMMAND itself exited'          'i-ignterm: the local operation did not exit by itself -- it was killed for ignoring TERM'
      pids_gone i-ignterm ignterm.pids
      group_gone i-ignterm ignterm.pgid
      ;;
    i-r125)
      # The other half of the cause label: the COMMAND exited 125, so the receipt must NOT blame the kill ladder.
      run_case i-r125 killfail
      expect_outcome i-r125 'OUTCOME: NO_OBSERVATION'
      has 'build rc=125'               'i-r125: the status is carried through, not swallowed'
      has 'the COMMAND itself exited 125' 'i-r125: the label says the command produced it, not the kill ladder'
      count_dir 'KILL_FAILED: phase=' 0 'i-r125: no failed-kill receipt exists'
      count_dir 'TIMEOUT_KILL: phase=build' 0 'i-r125: no kill receipt either: the ladder never ran'
      hasnt 'the kill ladder FAILED'   'i-r125: the ladder is not falsely accused'
      ;;
    pass-rc1)
      # N2. Every verdict line reads PASS and the JOIN is clean, but the executor refuses to account for its cost
      # and exits 1. The previous revision decided the verdict from the text alone and called that a PASS. The
      # forced status is set from the case name BEFORE the launch (the assertions below run after the run).
      run_case pass-rc1 pass
      expect_outcome pass-rc1 'OUTCOME: NO_OBSERVATION'
      count_is '^PASS ' 34                     'pass-rc1: every expected claim really did print PASS (the text is clean)'
      has 'JOIN_MISSING=0 FAILS=0'             'pass-rc1: the text scan itself found nothing wrong'
      has 'claim_batch_exit\[rtp\]=1'          'pass-rc1: the executor status is recorded where a collector can see it'
      has 'did not exit 0: rtp=1'              'pass-rc1: the single OUTCOME line names the executor and its status'
      ;;
    pass-rc2)
      # N2, second half: exit 2 is the executor's OWN 'no observation'. The text is still spotless.
      run_case pass-rc2 pass
      expect_outcome pass-rc2 'OUTCOME: NO_OBSERVATION'
      count_is '^PASS ' 34                     'pass-rc2: every expected claim really did print PASS'
      has 'did not exit 0: rtp=2'              'pass-rc2: the single OUTCOME line names the executor status 2'
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
      has 'phase gates refused'     'i-gate: attributed to the gates phase with its real code (2), not masked as a signal death'
      hasnt 'rc=137'                'i-gate: no 137 anywhere -- the EXIT-trap status write means a deliberate refusal is never reported as a signal'
      hasnt 'STATUS_UNRECORDED'     'i-gate: the status WAS recorded'
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
if [ "$SUITE" = 1 ]; then
  echo "CASES: suite of ${#KNOWN[@]}, dispatched $RAN (an unknown name aborts with exit 2, so neither list can drift)"
else
  echo "CASES: PARTIAL ${#CASES[@]} of ${#KNOWN[@]}, dispatched $RAN -- NOT a suite receipt"
fi
echo "HERMETIC OK=$OK BAD=$BAD  (work dir ${KEEP:+kept at }$W)"
[ "$BAD" = 0 ] || exit 1
exit 0
