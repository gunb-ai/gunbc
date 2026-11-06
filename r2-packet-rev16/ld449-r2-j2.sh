#!/bin/bash
# r2 restoration packet REVISION 16, job J2 (launcher job ld449-r2-j2). Author loyal-deer-449. NOT a grant request; byte review first.
# FRAME: #15967 packet_G.sh v4 (sha c781341a...), itself #15695 v4a (hardened frame #15931 v12 lineage): die2, scope gate, cgroup
#   bound gates, OOM parser/judge, verdict classifier, completion parser (REAL line verbatim, sha-checked), rc contract,
#   tree_clean, out_reusable, hash_of and their fixtures are copied BYTE FOR BYTE; only the job name in the scope gate and
#   its fixture, and the booked cgroup values, differ. Exit contract: 0 held, 1 deviation from a pre-registered verdict,
#   2 no observation (every refusal, timeout 124, signal >128, OOM, incoherent counter, survivors).
# LAUNCH: calmcrab-udrv.sh ld449-r2-j2 7200 16 14 4096 3200% <STAGED> <SHA> OUT <SERVE_UNIT_SHA>
# PINS: r2 head 26fe4b8f350a (tree b0ee022a6bf8) = merged integration abcbf27e4cc3 (#15990) + #15919 own commits + r2 1/2..5/5;
#   RED = r2 1/2 154afcd00667. One external process at a time; survivors in this job's cgroup are killed and verified gone.
# JOIN LAW ENFORCED: exactly one bare PASS|FAIL per selected name; no outcome line naming a function outside the roster; no
#   keyword other than PASS|FAIL for a roster name; batch rc cross-checked against that validated set. Jobs that run claim_batch
#   self-test this through the REAL run_entry (10 fixture logs) before anything else, and refuse if any case gives the wrong exit.
# AUTHORITY: calm-crab-645's launcher receipts (frozen.sha256, drv.log, printed SHAS) and the J3 receipt are what bind; the
#   markers written here are EVIDENCE for the next job's precheck only, never authority. Every job needs its own explicit grant.
set -uo pipefail
JOB=ld449-r2-j2
UNIT="$JOB.scope"
JOB_RT=7200
die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
# F3 scope gate: exactly one cgroup record, matched against a LITERAL suffix with a case glob (the quoted part is literal; no regex, no unescaped dot)
scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
[ "$(grep -c '' /proc/self/cgroup)" = 1 ] && scope_ok "$(cat /proc/self/cgroup)" || die2 "REFUSED_SETUP: /proc/self/cgroup is not exactly one 0:: line ending in /$UNIT"
CG="/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup)"
# PRE-WORK ACTUAL-BOUND CHECK (before any fetch): every bound read from the cgroup itself and compared to the booked value; mismatch or unreadable = exit 2
want() { local f="$CG/$1" v; v="$(cat "$f" 2>/dev/null)" || die2 "REFUSED_SETUP: $f unreadable"; [ "$v" = "$2" ] || die2 "REFUSED_SETUP: $1 is [$v], booked [$2]"; }
want memory.max 17179869184
want memory.high 15032385536
want memory.swap.max 0
want pids.max 4096
want cpu.max "3200000 100000"
oomread() { # CONTENT KEY -> prints digits, or returns 1
  local n m; n=$(printf '%s\n' "$1" | awk -v k="$2" '$1==k{c++} END{print c+0}'); m=$(printf '%s\n' "$1" | grep -acxE "$2 [0-9]+")
  [ "$n" = 1 ] && [ "$m" = 1 ] || return 1
  printf '%s\n' "$1" | grep -axE "$2 [0-9]+" | cut -d' ' -f2
}
mem() { echo "mem[$1] peak=$(cat "$CG/memory.peak" 2>&1) events=$(grep -aE '^oom' "$CG/memory.events" 2>&1 | tr '\n' ',') swap=$(cat "$CG/memory.swap.current" 2>&1)"; }
rawlog() { # TAG CONTENT: the RAW memory.events text behind a boundary decision, timestamped, appended under OUT ($R/oom-raw.log) so every OOM verdict re-derives from raw bytes
  printf '%s %s\n%s\n=====\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" "$2" >> "$R/oom-raw.log" || die2 "cannot write $R/oom-raw.log"; }
snap() { local E; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "REFUSED_SETUP: memory.events unreadable"
  rawlog "snap-before:${1:-?}" "$E"
  O0=$(oomread "$E" oom) && K0=$(oomread "$E" oom_kill) || die2 "REFUSED_SETUP: memory.events oom/oom_kill not each exactly one strict '<key> <digits>' line"; }
judge_oom() { # a RISE is OOM; a FALL cannot happen within one cgroup's life, so it is an incoherent counter -- both exit 2, labelled apart
  local E O1 K1; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "counter-incoherent: memory.events unreadable after $1"
  rawlog "judge-after:$1" "$E"
  O1=$(oomread "$E" oom) && K1=$(oomread "$E" oom_kill) || die2 "counter-incoherent: memory.events oom/oom_kill not strict after $1"
  { [ "$O1" -ge "$O0" ] && [ "$K1" -ge "$K0" ]; } || die2 "counter-incoherent (a FALL, not OOM) during $1: oom $O0->$O1, oom_kill $K0->$K1"
  { [ "$O1" -eq "$O0" ] && [ "$K1" -eq "$K0" ]; } || die2 "OOM (a RISE) during $1: oom $O0->$O1, oom_kill $K0->$K1"
}
# classify LINE NAME -> PASS|FAIL|NONE. ONLY the BARE forms are verdicts: "PASS <fn>" and "FAIL <fn>" (ClaimOutcome::Fail = a Bool-false claim).
# report_outcome prints the REASONED form "FAIL <fn> (<reason>)" for NON-claim outcomes (not-Bool return, runtime error, host tool unresolved, not-attempted, ExitFailure != 2),
# so a reasoned FAIL, "UNOBSERVED <fn> (...)" and "PANICKED <fn> (...)" are all NONE (exit 2, the line is printed in the leg's verdict lines). Exact on the name after stripping trailing whitespace.
classify() { local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"; if [ "$l" = "PASS $2" ]; then echo PASS; elif [ "$l" = "FAIL $2" ]; then echo FAIL; else echo NONE; fi; }
# verdict LOG NAME -> PASS|FAIL|NONE. EXACTLY ONE outcome line (any ALLCAPS keyword) may name the function; zero, a duplicate, a conflict (PASS+PANICKED), or an unknown keyword is NONE.
verdict() {
  local L; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  [ -n "$L" ] && [ "$(printf '%s\n' "$L" | grep -ac '')" = 1 ] || { echo NONE; return; }
  classify "$L" "$2"
}
# completion evidence: exactly one '[process-partition] total_ms=<digits>' line (claim_batch run() prints it at its end, Ok path only)
# Completion (keen-otter msg_a9e94d36): count EVERY record starting '[process-partition] total_ms' (the '[process-partition-exit]' line does not start so);
#   exactly one must exist, and it must match '^\[process-partition\] total_ms=[0-9]+( |$)'. A valid line beside a malformed competitor refuses.
completed() { local n m; n=$(grep -ac '^\[process-partition\] total_ms' "$1"); m=$(grep -acE '^\[process-partition\] total_ms=[0-9]+( |$)' "$1"); [ "$n" = 1 ] && [ "$m" = 1 ]; }
# hash_of FILE -> exactly one 64-hex sha256, or return 1: the sha256sum read must SUCCEED and yield one 64-hex token (two FAILED reads must never compare equal as two empty strings)
hash_of() { local o h; o="$(sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }
# exit-status contract: only 0/1/2 are in contract; 124 (timeout), >128 (signal) or anything else is out of contract
rc_in_contract() { case "$1" in 0|1|2) return 0;; *) return 1;; esac; }
out_reusable() { [ -e "$1" ] || [ -L "$1" ]; }   # true = path is taken in ANY form (dir, file, symlink incl. dangling)
tree_clean() { local o; o="$(git -C "$1" status --porcelain 2>/dev/null)" || return 1; [ -z "$o" ]; }   # a FAILED git status (non-repo, error) is NOT clean: capture, require rc 0, THEN test emptiness
# ---- FIXTURE self-checks (a focused failing input for each defect class), before any fetch ----
# The REAL completion line, copied verbatim from calm-crab-645's G1 cpr log (sha a0e1f1f6), with sha256 checked below.
REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
[ "$(printf %s "$REAL_COMPLETION" | sha256sum | cut -d' ' -f1)" = "7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2" ] || die2 "REFUSED_SETUP: the real-line fixture is not the verbatim bytes"
FX="$(mktemp -d)" || die2 "mktemp"
r() { if "$@"; then echo accepted; else echo refused; fi; }
# (1) parser
r_ev() { oomread "$1" oom >/dev/null && oomread "$1" oom_kill >/dev/null; }
[ "$(r r_ev $'oom 0\noom_kill 0')" = accepted ] || die2 "REFUSED_SETUP: fixture: valid-only events refused"
[ "$(r r_ev $'oom 0 garbage\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: extra field accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom bad')" = refused ] || die2 "REFUSED_SETUP: fixture: malformed duplicate accepted"
[ "$(r r_ev $'oom 0\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: valid duplicate accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom line accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom_kill')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom_kill line accepted"
[ "$(r r_ev $'oom\t0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom accepted"
[ "$(r r_ev $'oom 0\noom_kill\t0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom_kill accepted"
[ "$(r r_ev $'oom\tbad\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: oom<TAB>bad accepted"
[ "$(r r_ev $'oom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: absent key accepted"
# (2) verdict + completion
printf 'PASS x\nPASS x\n' > "$FX/v1"; printf 'PASS x\nPANICKED x (r)\n' > "$FX/v2"; printf 'PASS x\n' > "$FX/v3"; printf 'WEIRD x\n' > "$FX/v4"; printf 'FAIL x (runtime error [c]: m)\n' > "$FX/v5"; printf 'FAIL x\n' > "$FX/v7"; printf 'PASS xy\n' > "$FX/v6"
[ "$(verdict $FX/v1 x) $(verdict $FX/v2 x) $(verdict $FX/v3 x) $(verdict $FX/v4 x) $(verdict $FX/v5 x) $(verdict $FX/v6 x) $(verdict $FX/v7 x)" = "NONE NONE PASS NONE NONE NONE FAIL" ] || die2 "REFUSED_SETUP: verdict fixture misfire"
printf 'PASS x\n%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=340188\n' "$REAL_COMPLETION" > "$FX/c1"; printf 'PASS x\n' > "$FX/c2"; printf '[process-partition] total_ms=5\n' > "$FX/c3"; printf '[process-partition] total_ms=\n' > "$FX/c4"; printf '[process-partition] total_ms=abc pre_run_ms=0\n' > "$FX/c5"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_ms=6 a_ms=1\n' > "$FX/c6"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_msX\n' > "$FX/c7"; printf '[process-partitions] total_ms=5\n' > "$FX/c8"
[ "$(r completed $FX/c1) $(r completed $FX/c2) $(r completed $FX/c3) $(r completed $FX/c4) $(r completed $FX/c5) $(r completed $FX/c6) $(r completed $FX/c7) $(r completed $FX/c8)" = "accepted refused accepted refused refused refused refused refused" ] || die2 "REFUSED_SETUP: completion fixture misfire (real, absent, bare, missing total, non-numeric, duplicate, valid+malformed, wrong marker)"
# (3) exit contract
[ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
# (4) scope glob literal suffix, restore cleanliness
[ "$(r scope_ok '0::/a/ld449-r2-j2.scope') $(r scope_ok '0::/a/ld449-r2-j2Xscope') $(r scope_ok '0::/a/xld449-r2-j2.scope') $(r scope_ok '0::/a/ld449-r2-j2.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
[ "$A $B" = "accepted refused" ] || die2 "REFUSED_SETUP: restore-cleanliness fixture misfire ($A $B)"
# (5) output path reuse in any form
mkdir "$FX/d"; : > "$FX/f"; ln -s "$FX/nowhere" "$FX/dangling"
[ "$(r out_reusable $FX/d) $(r out_reusable $FX/f) $(r out_reusable $FX/dangling) $(r out_reusable $FX/absent)" = "accepted accepted accepted refused" ] || die2 "REFUSED_SETUP: output-path fixture misfire"
mkdir "$FX/nonrepo"; [ "$(r tree_clean $FX/nonrepo)" = refused ] || die2 "REFUSED_SETUP: fixture: tree_clean accepted a non-repo dir (failed git status read as clean)"
printf x > "$FX/hf"; H1="$(hash_of "$FX/hf")"; [ "${#H1}" = 64 ] || die2 "REFUSED_SETUP: fixture: hash_of did not yield 64-hex"
if hash_of "$FX/missing" >/dev/null; then die2 "REFUSED_SETUP: fixture: hash_of accepted a missing file"; fi
rm -rf "$FX"
echo "fixtures: all held"
# SELF identity, taken BEFORE any cd (works for a relative or absolute $0; the launcher runs `bash "$F"`).
SELF_SHA="$(hash_of "$0")" || die2 "REFUSED_SETUP: cannot hash own script $0"
[ "$#" = 2 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir> <serve-unit-sha256>"
if out_reusable "$1"; then die2 "REFUSED_SETUP: output path $1 already exists (dir, file or symlink); reuse is refused"; fi
mkdir -- "$1" || die2 "mkdir $1 (exclusive, no -p)"
cd -- "$1" || die2 "cd $1"
R="$PWD"
EXTRA_ARG="${2:-}"
roster_gate() { # $1 = file, $2... = roster names
  local f="$1"; shift
  local -a declared=(); local nm w d j found cnt
  while read -r nm; do declared+=("$nm"); done < <(grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$f" | awk '{print $NF}')
  for w in "$@"; do
    cnt=0; for d in "${declared[@]}"; do [ "$d" = "$w" ] && cnt=$((cnt+1)); done
    [ "$cnt" = 1 ] || return 1
  done
  j=0
  for w in "$@"; do
    found=0
    while [ "$j" -lt "${#declared[@]}" ]; do
      if [ "${declared[$j]}" = "$w" ]; then found=1; j=$((j+1)); break; fi
      j=$((j+1))
    done
    [ "$found" = 1 ] || return 1
  done
  return 0
}
# ==== r2 pins and shared work dir (identical in every job) ====
# TIME BUDGET: one per job. JOB_RT (emitted per job = the launcher RT) from scope start; JOB_END = start + JOB_RT - 60;
# RESERVE (default 300 s) is kept for finalization/stop paths. tcap N = min(N, JOB_END - RESERVE - now); every external
# operation's timeout is tcap of its own cap, and an operation with < 1 s left is not attempted (exit 2 "time budget"). So
# the sum of per-step caps can never exceed the job's runtime.
# ==== GATE HELPERS (rev15; sleek-owl/keen-otter rev13 reviews) ====
# The rule: a producer's exit status is checked BEFORE its content is compared or consumed; nothing that gates is read through
# `| head` or a process substitution whose status is lost; every grep distinguishes 0 match, 1 no match, >= 2 ERROR (refuse).
# capt VAR cmd...: runs cmd ONCE, the whole of its stdout into VAR, returns cmd's exact rc (no pipe, no head, no SIGPIPE).
capt() { local -n _cp=$1; shift; _cp="$("$@")"; }
# gcnt VAR grep-args...: VAR = grep -c count. rc 0 (some) and 1 (none) are counts; rc >= 2, or a non-numeric count, refuses.
gcnt() { local -n _gc=$1; shift; local _r; _gc="$(grep -c "$@")"; _r=$?; { [ $_r -le 1 ] && [[ $_gc =~ ^[0-9]+$ ]]; } || die2 "grep error (rc $_r) counting [$*]"; }
# gtest grep-args...: returns 0 on a match, 1 on no match; rc >= 2 refuses (a grep error is never read as "no match").
gtest() { grep -q "$@"; local _r=$?; [ $_r -le 1 ] || die2 "grep error (rc $_r) testing [$*]"; return $_r; }
# CLOCK: the builtin EPOCHSECONDS (bash >= 5), never `$(date +%s)`: a failed date makes an empty arithmetic operand.
[[ ${EPOCHSECONDS:-} =~ ^[0-9]+$ ]] || die2 "REFUSED_SETUP: this bash has no EPOCHSECONDS (bash >= 5 required)"
# SCOPE RE-CHECK (the frame's CG uses an unchecked sed): re-derive CG from /proc/self/cgroup with a builtin read and require
# the frame's value to equal it; a failed read refuses.
{ read -r _cgl < /proc/self/cgroup; } || die2 "REFUSED_SETUP: cannot read /proc/self/cgroup"
[ "$CG" = "/sys/fs/cgroup${_cgl#0::}" ] && [ "${_cgl:0:3}" = "0::" ] || die2 "REFUSED_SETUP: the frame's CG [$CG] is not this process's cgroup"
# ENVIRONMENT PRECONDITION (keen-otter msg_57af3d65): GUNBC_WORKSPACE_ROOT overrides the CLI's workspace walk, so every
# gunbc / claim_batch call would read a different tree than the pinned worktree. Generalized: the CLI at the pin reads at least
# 26 GUNBC_* variables by literal name (memo, cache root, memory budget, cost entries, CI diff base, ...) and others through
# generated constants (GUNBC_WORKSPACE_ROOT is one), so the set is not enumerable by grep. ANY exported GUNBC_* name refuses,
# before any CLI call. Names come from the builtin `compgen -e`; no value is read or printed.
GUNBC_ENV_SET=""; for _en in $(compgen -e); do [[ $_en == GUNBC_* ]] && GUNBC_ENV_SET+="$_en "; done   # builtins only
[ -z "${GUNBC_WORKSPACE_ROOT+set}" ] || die2 "REFUSED_SETUP: GUNBC_WORKSPACE_ROOT is set (value not shown); the packet requires every GUNBC_* variable unset"
[ -z "$GUNBC_ENV_SET" ] || die2 "REFUSED_SETUP: GUNBC_* variables are set: [ $GUNBC_ENV_SET] (names only, values not shown); the packet requires them unset"
T0=$(( EPOCHSECONDS - SECONDS )); JOB_END=$(( T0 + JOB_RT - 60 )); RESERVE=300   # fixed; not overridable from the environment
# OPDL: the ONE operation deadline (keen-otter, rev10 review). Every timed bookkeeping operation (wt_ok, ign_snapshot,
# shas_ok, receipt hashing in mark, containment waits) takes timeout = min(its own cap, OPDL - now) and FAILS when nothing is
# left, so a phase made of several such operations is bounded AS A WHOLE by OPDL, never by the sum of per-command caps.
OPDL=$(( JOB_END - 10 ))
# optc VAR N: VAR = min(N, OPDL - now), or return 1 when < 1 s remains (never 0: `timeout 0` would mean NO timeout).
optc() { local -n _oc=$1; local r=$(( OPDL - EPOCHSECONDS )); [ $r -ge 1 ] || return 1; if [ $r -lt $2 ]; then _oc=$r; else _oc=$2; fi; }
# hash_t FILE: hash_of under the operation deadline (cap 60 s). The frame's hash_of stays byte-for-byte (its own fixtures use it); EVERY hash on a job's real path (need, mark, J2, J4) uses hash_t.
hash_t() { local c o h; optc c 60 || return 1; o="$(timeout $c sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }
tcap() { local r=$(( JOB_END - RESERVE - EPOCHSECONDS )); [ $r -lt 1 ] && { echo 0; return; }; [ $r -lt $1 ] && echo $r || echo $1; }
# budget VAR N what: sets VAR to tcap N in the CURRENT shell, or exits the job (die2, which J2 redefines to finalize) when the
# budget is exhausted. Never call it inside $(...): exhaustion must end the job, not a subshell.
# (its local is _bc, so a caller's variable named c -- or anything but _bc -- is set, not shadowed)
budget() { local _bc; _bc=$(tcap $2); [ "$_bc" -ge 1 ] || die2 "time budget exhausted before $3"; printf -v "$1" '%s' "$_bc"; }
HEAD_EXPECT=26fe4b8f350a99bd57a9bc342d4625b0cb96804b; TREE_EXPECT=b0ee022a6bf8f9f9f35d496174afd681300c9847
RED_REV=154afcd00667d3c84609f551c45e0ab4f5b83bad; NO15175=f38a00cb75d85e3a7879c27f7288c2a8bba993ae
WD=/home/briansrls/runs/ld449-r2work; WT=$WD/wt; WR=$WD/wt-red; TG=$WD/target; MK=$WD/markers
DECLARED_SERVE_BINARY=/home/briansrls/runs/ld449-r2work/target/release/gunbc
OFF=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_record
SPW=dag/test/claim/roadmap/session_placement_wet_witness.dag
export PATH=$HOME/.cargo/bin:$PATH
# PATH-SHADOW GUARD (independent Y-SP review, rev16): $HOME/.cargo/bin is first on PATH (here and in the launcher), and the
# witnesses spawn bare program names through PATH. Every tool this packet or the reached witnesses run by bare name must
# resolve to /usr/bin or /bin; one found anywhere else (a user-writable directory first on PATH) refuses. `type -P` is a
# builtin; a tool that is absent is not refused here (its call fails loudly where it is used).
path_guard() { local n p; for n in mktemp mkdir date env sh bash git cat grep awk sed sort tee mv rm cp ln chmod chgrp chown stat readlink \
    find timeout sha256sum systemctl ss ssh curl comm cmp diff id seq dd xargs tr cut head tail wc sleep kill ls touch install; do
    p="$(type -P "$n")" || continue; case "$p" in /usr/bin/"$n"|/bin/"$n") ;; *) die2 "REFUSED_SETUP: '$n' resolves to [$p], outside /usr/bin and /bin (PATH shadow)";; esac; done; }
path_guard
shas_ok() { local c; optc c 120 || die2 "deadline before the binary sha check"; [ -f $WD/SHAS ] && timeout $c sha256sum -c --quiet $WD/SHAS >/dev/null 2>&1 || die2 "binary shas changed or missing (or the check timed out)"; }
shas_id() { hash_t $WD/SHAS; }
# Markers (B1). A marker is EVIDENCE for the next precheck, never authority (each job is its own grant; calm-crab verifies the
# launcher receipts -- frozen.sha256, drv.log, printed SHAS -- independently). It is one field per line:
#   HEAD, TREE, SCRIPT (the writing job's own sha, taken before cd), SHAS (binaries), OUT (the writing job's output dir),
#   PRIOR <name> <sha256 of that marker file>, and one RECEIPT <sha256> <absolute path> per retained receipt.
# need() re-reads reality before trusting a marker: the worktree's HEAD and tree (wt_ok), the binaries (shas_ok), and then the
# WHOLE chain back to J1: every marker's fields, every RECEIPT re-hashed, every PRIOR link re-hashed. Stated limit: a process
# running as the same user can still fabricate consistent files; the markers raise the cost of that, they do not prevent it.
# wfile <path> <text>: writes text + newline through ONE timed `tee` under OPDL (a hung write fails, never blocks the phase).
wfile() { local c; optc c 30 || return 1; printf '%s\n' "$2" | timeout $c tee -- "$1" > /dev/null; }
# mark: every id is computed and SHAPE-CHECKED in this shell BEFORE the body is built (keen-otter rev11: a failing
# "$(shas_id)" inside printf's arguments was masked by printf's own success and wrote an empty SHAS field), and every file
# operation (mkdir, write, readback, rename, final read) is a timed external command under OPDL.
mark() { local name=$1 prior=$2 ps=none sid f h t body c; shift 2
  if [ "$prior" != none ]; then ps="$(hash_t $MK/$prior)" || die2 "cannot hash prior marker $prior"; [[ $ps =~ ^[0-9a-f]{64}$ ]] || die2 "prior marker hash has a bad shape [$ps]"; fi
  sid="$(shas_id)" || die2 "marker $name: cannot hash SHAS (failed or deadline)"; [[ $sid =~ ^[0-9a-f]{64}$ ]] || die2 "marker $name: SHAS id has a bad shape [$sid]"
  optc c 30 || die2 "deadline before mkdir markers"; timeout $c mkdir -p -- $MK || die2 "mkdir markers"; t=$MK/.$name.tmp
  ign_snapshot $R/ignored-end.txt || die2 "ignored-set snapshot"
  printf -v body 'HEAD %s\nTREE %s\nSCRIPT %s\nSHAS %s\nOUT %s\nPRIOR %s %s' "$HEAD_EXPECT" "$TREE_EXPECT" "$SELF_SHA" "$sid" "$R" "$prior" "$ps"
  for f in "$@" $R/ignored-end.txt; do h="$(hash_t "$f")" || die2 "marker $name: cannot hash receipt $f"; [[ $h =~ ^[0-9a-f]{64}$ ]] || die2 "marker $name: receipt hash has a bad shape"; body="$body"$'\n'"RECEIPT $h $f"; done
  wfile $t "$body" || die2 "marker write $name (failed or deadline)"
  # checked readback (keen-otter rev12): the command's rc is checked BEFORE its output is compared, so a read that prints
  # the right bytes and then fails (rc 1, or timeout's 124) never publishes a marker.
  local rb; optc c 30 || die2 "deadline before marker readback $name"; rb="$(timeout $c cat -- $t)" || die2 "marker readback $name failed or timed out"; [ "$rb" = "$body" ] || die2 "marker readback $name differs"
  optc c 30 || die2 "deadline before marker rename $name"; timeout $c mv -f -T -- $t $MK/$name || die2 "marker rename $name"
  optc c 30 || die2 "deadline before marker final read $name"; timeout $c cat -- $MK/$name || die2 "marker final read $name"; }
mfield() { local _mf; _mf="$(awk -v k="$2" '$1==k{print substr($0,length(k)+2)}' "$1")" || return 1; printf '%s' "$_mf"; }   # rc-checked; callers assign and check
# Expected receipts per marker, by EXACT location: W:<path relative to the work dir> or O:<path relative to that marker's own
# OUT>. A receipt elsewhere, an extra one, or a missing one refuses (no basename-only matching).
EXPECTED_RECEIPTS="J1_GREEN=W:SHAS,O:build.log,O:claims-G-off.log,O:claims-R-off.log,O:oom-raw.log,O:ignored-end.txt
J1R_GREEN=W:render/serve.unit,W:render/belt.txt,W:render/review.txt,W:render/RENDER_SHAS,O:preflight.log,O:oom-raw.log,O:ignored-end.txt
J1B1_GREEN=O:claims-G-session_placement_wet_witness.log,O:claims-G-roadmap_belt_actuate_witness.log,O:claims-G-roadmap_dashboard_instance_witness.log,O:fabric-posts.txt,O:oom-raw.log,O:ignored-end.txt
J1B2_GREEN=O:claims-G-attempt_system_unit_launch_witness.log,O:claims-G-attempt_state_move_apply_wet_witness.log,O:claims-G-lab_apply_operator_grant_witness.log,O:oom-raw.log,O:ignored-end.txt
J1B3_GREEN=O:claims-G-worker_git_subtree_apply_spawn_wet_witness.log,O:claims-G-dispatch_worker_confinement_witness.log,O:claims-G-operator_endpoint_witness.log,O:claims-G-agent_door_witness.log,O:oom-raw.log,O:ignored-end.txt
J1B4_GREEN=O:claims-G-request_authentication_witness.log,O:claims-G-roadmap_event_log_witness.log,O:claims-G-harness_validate_worktree_witness.log,O:claims-G-dispatch_worker_traversal_witness.log,O:oom-raw.log,O:ignored-end.txt
J2_READY=W:identity.txt,O:serve.unit,O:k1-request.txt,O:admit.log,O:apply.log,O:units-before.txt,O:units-after-apply.txt,O:paths-before.txt,O:paths-after.txt,O:oom-raw.log,O:ignored-end.txt"
# OUT must lie under the launcher's own directory for the job that writes that marker (no foreign OUT).
MARKER_JOB="J1_GREEN:ld449-r2-j1 J1R_GREEN:ld449-r2-j1r J1B1_GREEN:ld449-r2-j1b1 J1B2_GREEN:ld449-r2-j1b2 J1B3_GREEN:ld449-r2-j1b3 J1B4_GREEN:ld449-r2-j1b4 J2_READY:ld449-r2-j2"
RUNS_ROOT=/home/briansrls/runs
# LAUNCHER OUT (assumption to be attested by calm-crab-645): calmcrab-udrv runs job <JOB> with OUT = $RUNS_ROOT/<JOB>/out.
# A marker's OUT must be EXACTLY that path for the job that writes it.
LAUNCHER_OUT_LEAF=out
need_one() { local m=$MK/$1 want_script=$2 want_prior=$3 v h f pn psha out exp got cf cout cwd
  [ -L $m ] && die2 "marker $1 is a symlink"; [ -f $m ] || die2 "marker $1 missing"
  local mh mt pr cnt; mh="$(mfield $m HEAD)" || die2 "marker $1: cannot read HEAD"; mt="$(mfield $m TREE)" || die2 "marker $1: cannot read TREE"
  [ "$mh" = $HEAD_EXPECT ] && [ "$mt" = $TREE_EXPECT ] || die2 "marker $1: head/tree"
  # both sides checked (the dual of keen-otter's rev11 mark finding): a failing shas_id and an EMPTY marker field must not
  # compare equal.
  local sid ms; sid="$(shas_id)" || die2 "marker $1: cannot hash SHAS (failed or deadline)"; [[ $sid =~ ^[0-9a-f]{64}$ ]] || die2 "marker $1: SHAS id has a bad shape"
  ms="$(mfield $m SHAS)" || die2 "marker $1: cannot read its SHAS field"; [ "$ms" = "$sid" ] || die2 "marker $1: binaries differ"
  v="$(mfield $m SCRIPT)" || die2 "marker $1: cannot read SCRIPT"; [[ $v =~ ^[0-9a-f]{64}$ ]] || die2 "marker $1: script field"; [ -z "$want_script" ] || [ "$v" = "$want_script" ] || die2 "marker $1 not written by reviewed script $want_script"
  gcnt cnt '^PRIOR ' $m; [ "$cnt" = 1 ] || die2 "marker $1: PRIOR"; pr="$(mfield $m PRIOR)" || die2 "marker $1: cannot read PRIOR"
  [[ $pr =~ ^([A-Za-z0-9_]+)\ (none|[0-9a-f]{64})$ ]] || die2 "marker $1: PRIOR field has a bad shape"; pn="${BASH_REMATCH[1]}"; psha="${BASH_REMATCH[2]}"
  [ "$pn" = "$want_prior" ] || die2 "marker $1 chains to $pn, expected $want_prior"
  [ "$pn" = none ] || { h="$(hash_t $MK/$pn)" || die2 "hash $pn"; [ "$h" = "$psha" ] || die2 "marker $1's prior $pn changed since it was written"; }
  gcnt cnt '^RECEIPT ' $m; [ "$cnt" -ge 1 ] || die2 "marker $1 binds no receipts"
  out="$(mfield $m OUT)" || die2 "marker $1: cannot read OUT"; gcnt cnt '^OUT ' $m; [ "$cnt" = 1 ] && [[ $out = /* ]] && [ -d "$out" ] && [ ! -L "$out" ] || die2 "marker $1: OUT [$out] is not a real absolute directory"
  cout="$(readlink -e -- "$out")" && [ "$cout" = "$out" ] || die2 "marker $1: OUT [$out] is not canonical"; cwd="$(readlink -e -- "$WD")" || die2 "work dir does not resolve"
  local recs; capt recs grep '^RECEIPT ' $m || die2 "marker $1: cannot read its RECEIPT lines"   # captured, rc checked (no process substitution)
  while read -r _ h f; do
    # canonicalize (resolve .., ., and symlinked directories) before the prefix check; the receipt must ALREADY be canonical.
    cf="$(readlink -e -- "$f")" || die2 "marker $1: receipt $f does not resolve"; [ "$cf" = "$f" ] || die2 "marker $1: receipt path $f is not canonical ($cf)"
    case "$cf" in "$cout"/*|"$cwd"/*) ;; *) die2 "marker $1: receipt $f is outside its OUT and the work dir";; esac
    [ -L "$f" ] && die2 "marker $1: receipt $f is a symlink"; local hh; hh="$(hash_t "$f")" || die2 "marker $1: cannot hash receipt $f"; [ "$hh" = "$h" ] || die2 "marker $1: receipt $f changed"; done <<< "$recs"
  local x job=""; for x in $MARKER_JOB; do [ "${x%%:*}" = "$1" ] && job="${x##*:}"; done; [ -n "$job" ] || die2 "marker $1: no writing job"
  [ "$cout" = "$RUNS_ROOT/$job/$LAUNCHER_OUT_LEAF" ] || die2 "marker $1: OUT [$cout] is not the launcher OUT $RUNS_ROOT/$job/$LAUNCHER_OUT_LEAF"
  gcnt cnt -vE '^(HEAD|TREE|SCRIPT|SHAS|OUT|PRIOR|RECEIPT) ' $m; [ "$cnt" = 0 ] || die2 "marker $1 has unknown lines"
  # expected and got sets, built with builtins from data already captured (no pipeline whose status could be lost)
  local row e line; local -a ex=() gt=(); while IFS= read -r row; do [ "${row%%=*}" = "$1" ] && { IFS=, read -ra ex <<< "${row#*=}"; }; done <<< "$EXPECTED_RECEIPTS"
  [ ${#ex[@]} -gt 0 ] || die2 "marker $1: no expected receipt set"
  for e in "${!ex[@]}"; do case "${ex[$e]}" in W:*) ex[$e]="$cwd/${ex[$e]#W:}";; O:*) ex[$e]="$cout/${ex[$e]#O:}";; *) die2 "marker $1: bad expected-receipt spec";; esac; done
  while read -r _ _ line; do gt+=("$line"); done <<< "$recs"
  exp="$(printf '%s\n' "${ex[@]}" | LC_ALL=C sort)" || die2 "marker $1: sort failed"; got="$(printf '%s\n' "${gt[@]}" | LC_ALL=C sort)" || die2 "marker $1: sort failed"
  [ "$got" = "$exp" ] || die2 "marker $1: receipts are not exactly the expected set at the expected locations"
  NEXT_PRIOR="$pn"; }
# need <marker> <EXACT reviewed script sha> <its prior marker>: verifies the named marker, then walks and verifies the chain to J1.
CHAIN_PRIORS="J1R_GREEN:J1_GREEN J1B1_GREEN:J1R_GREEN J1B2_GREEN:J1B1_GREEN J1B3_GREEN:J1B2_GREEN J1B4_GREEN:J1B3_GREEN J2_READY:J1B4_GREEN J1_GREEN:none"
reviewed_sha() { local x; for x in $REVIEWED_SCRIPTS; do [ "${x%%:*}" = "$1" ] && { echo "${x##*:}"; return 0; }; done; return 1; }
need() { local cur exp x rs f; wt_ok; shas_ok; need_one $1 $2 $3; cur="$NEXT_PRIOR"
  # ignored-path set must be exactly what the predecessor job recorded at its end (no change between jobs).
  local il; capt il grep '^RECEIPT .*/ignored-end.txt$' $MK/$1; case $? in 0) ;; 1) die2 "marker $1 has no ignored-set receipt";; *) die2 "marker $1: grep error reading it";; esac
  [[ $il =~ ^RECEIPT\ [0-9a-f]{64}\ (/[^[:space:]]+)$ ]] || die2 "marker $1: ignored-set receipt line has a bad shape (or more than one)"; f="${BASH_REMATCH[1]}"
  ign_snapshot $R/ignored-start.txt || die2 "ignored-set snapshot"; cmp -s "$f" $R/ignored-start.txt || die2 "worktree ignored-path set changed since $1 was written"
  while [ "$cur" != none ]; do exp=""; for x in $CHAIN_PRIORS; do [ "${x%%:*}" = "$cur" ] && exp="${x##*:}"; done
    [ -n "$exp" ] || die2 "chain: unknown marker $cur"; rs="$(reviewed_sha $cur)" || die2 "chain: no reviewed script sha for $cur"
    need_one $cur "$rs" $exp; cur="$NEXT_PRIOR"; done; echo "chain verified to J1 (every link pinned to its reviewed script)"; }
# fresh_own <marker>: a job refuses to run while its OWN success marker already exists (a stale success marker must never
# survive a failed rerun and be accepted by the next job). Removal of a stale marker is a keeper decision.
fresh_own() { { [ -e $MK/$1 ] || [ -L $MK/$1 ]; } && die2 "own marker $1 already exists (rerun over a prior success); calm-crab decides"; return 0; }
# wt_ok: the worktree is really at the pin and pristine, including ignored files (tree_clean cannot see a different commit).
wt_ok() { local o c1 c2 c3; optc c1 60 || die2 "deadline before the worktree pin check"; o="$(timeout $c1 git -C $WT rev-parse HEAD)" || die2 "worktree head read failed or timed out"; [ "$o" = $HEAD_EXPECT ] || die2 "worktree is not at the pin"
  optc c2 60 || die2 "deadline before the worktree tree check"; o="$(timeout $c2 git -C $WT rev-parse 'HEAD^{tree}')" || die2 "worktree tree read failed or timed out"; [ "$o" = $TREE_EXPECT ] || die2 "worktree is not at the pin"
  optc c3 120 || die2 "deadline before git status"; o="$(timeout $c3 git -C $WT status --porcelain --untracked-files=all)" || die2 "git status failed or timed out"; [ -z "$o" ] || die2 "worktree has tracked or untracked changes"; }
# IGNORED PATHS: .gitignore at the pin ignores /.gunbc/ and the two roster.dag paths among others, which runs may write. So the
# ignored set is not required empty after runs: J1 requires it EMPTY on the fresh clone; every job records it at its end as a
# marker receipt; the next job requires it UNCHANGED at its start (tampering between jobs refuses).
ign_snapshot() { local o c x; local -a L=(); optc c 120 || return 1; o="$(timeout $c git -C $WT status --porcelain --ignored --untracked-files=all)" || return 1
  while IFS= read -r x; do [[ $x == '!! '* ]] && L+=("$x"); done <<< "$o"   # an empty ignored set is valid
  optc c 30 || return 1; { if [ ${#L[@]} -gt 0 ]; then printf '%s\n' "${L[@]}"; fi; } | timeout $c sort | timeout $c tee -- "$1" > /dev/null; }
# Writes OUTSIDE OUT by wet witnesses (B3): they mktemp under FIXED /tmp templates, which other sessions running as the same
# user also use, so a new same-UID entry appearing during a run is NOT proof this run created it. This job therefore DELETES
# NOTHING under /tmp: it records every entry matching those templates that appeared during each wet run in
# $R/tmp-residue-candidates.txt (path, stat signature) as CANDIDATE residue of unproven provenance, retained for a separately
# reviewed cleanup.
TMP_GLOBS='/tmp/gunbc_g5_* /tmp/gunbc_pub_* /tmp/gunbc_attempt_state_move_wet_* /tmp/wre3-*'
# tmp_list: owned /tmp entries matching the templates. The filter's status is EXPLICIT (calm-crab M1, sleek-owl rev15: the
# loop's last [ -e ]/[ -O ] result leaked out as the function's rc, so a dangling or foreign LAST match refused a wet leg); the
# only failure that refuses is the sort itself.
tmp_list() { local g p; local -a hits=(); for g in $TMP_GLOBS; do for p in $g; do if [ -e "$p" ] && [ -O "$p" ]; then hits+=("$p"); fi; done; done
  [ ${#hits[@]} -gt 0 ] || return 0; printf '%s\n' "${hits[@]}" | LC_ALL=C sort; }
tmp_record() { local p now; now="$(tmp_list)" || die2 "cannot list /tmp residue candidates"; comm -13 <(printf '%s\n' "$1") <(printf '%s\n' "$now") | while read -r p; do printf '%s %s %s\n' "$2" "$(stat -c '%a %U:%G %i %Y' -- "$p" 2>&1)" "$p"; done >> $R/tmp-residue-candidates.txt || die2 "cannot record /tmp residue candidates"; }
# run_entry LEG TAG DIR ENTRY FUNCS(csv) WET(0|1) CAP : one claim_batch run, serial, under timeout CAP (timeout puts the run in
# its own process group and signals the group); every fn must have exactly one bare verdict; rc must equal this run's verdicts.
run_entry() { local LEG="$1" T="$2" DIR="$3" F="$4" FUNCS="$5" WET="$6" CAP="$7" L RC VE VD N A=""
  L="$R/claims-$LEG-$T.log"; [ "$WET" = 1 ] && A=--wet
  roster_gate "$DIR/$F" ${FUNCS//,/ } || die2 "leg $LEG $T: roster not declared exactly once, in order, in $F"
  [ "${CLOSED["$DIR|$F"]:-}" = 1 ] || die2 "leg $LEG $T: entry $F in $DIR was not admitted by this job's closure query"
  local TB=""; if [ "$WET" = 1 ]; then TB="$(tmp_list)" || die2 "leg $LEG $T: cannot list /tmp before the run"; fi
  snap "run $LEG $T"
  # FULL-CAP-OR-REFUSE (keen-otter rev11 budget review): a claim run starts only if its WHOLE cap still fits before
  # JOB_END - RESERVE; otherwise the leg refuses BEFORE starting (typed), never a run cut off before its roster.
  local CC; CC=$(tcap "$CAP"); [ "$CC" = "$CAP" ] || die2 "leg $LEG $T: remaining budget ${CC}s < its full cap ${CAP}s; not starting a run that could end before its roster"; CC=$((CC-30))
  ( cd "$DIR" && timeout --kill-after=30 "$CC" "$TG/release/claim_batch" $A --claim-run --source-root dag --source-root src/v2 --entry "$F" --functions "$FUNCS" ) > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; contain "run $LEG $T"; [ "$WET" = 1 ] && tmp_record "$TB" "$LEG.$T"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
  rc_in_contract "$RC" || die2 "leg $LEG $T: exit status $RC out of contract (0/1/2; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg $LEG $T: no single completion line; batch incomplete"
  gtest -aE "closure diagnostics|resolve failed|resolve did not reach" "$L" && die2 "leg $LEG $T: closure/resolve refusal (NO_OBSERVATION)"
  # rc 1 can also come from a Refused cost partition (claim_batch.rs batch_exit_code); require exactly one Reconciled one.
  local cpn; gcnt cpn -a '^\[cost-partition\] ' "$L"; local cpr; gcnt cpr -a '^\[cost-partition\] .*"verdict":{"state":"Reconciled"' "$L"; [ "$cpn" = 1 ] && [ "$cpr" = 1 ] || die2 "leg $LEG $T: cost partition not exactly one Reconciled line"
  VE=0
  for N in ${FUNCS//,/ }; do
    VD="$(verdict "$L" "$N")"
    [ "$VD" = ERROR ] && die2 "leg $LEG $T: grep error reading the log for '$N'"
    [ "$VD" = NONE ] && die2 "leg $LEG $T: '$N' has no SINGLE BARE PASS/FAIL verdict"
    [ "$VD" = FAIL ] && VE=1
    OBS["$LEG.$T.$N"]="$VD"
    [ "$VD" != "${EXPECT["$LEG.$T.$N"]:-NONE}" ] && { DEVIATIONS=$((DEVIATIONS+1)); echo "DEVIATION $LEG $T $N: observed $VD, pre-registered ${EXPECT["$LEG.$T.$N"]:-NONE}"; }
  done
  # JOIN LAW (keen-otter msg_b5063276, silent-bee msg_b43e5154; reference #15967 v6):
  #  (a) a CR byte anywhere refuses (a CR can hide a second outcome on one displayed line);
  #  (b) OUTCOME-LIKE lines: any known outcome keyword, CASE-INSENSITIVE, after optional leading whitespace, followed by
  #      any NON-WORD character ([^A-Za-z0-9_]) or end of line. ONLY this scan is case-insensitive;
  #  (c) CANONICAL lines: '^(PASS|FAIL|UNOBSERVED|...) <name>( |$)' exactly (no leading space, upper case, no colon);
  #  (d) UNKNOWN ALLCAPS tokens: '^[A-Z][A-Z0-9_]*' then a NON-WORD character or end of line (NOT [ \t:], which
  #      GNU grep reads as space, backslash, 't', colon -- rev10 counted 'Started ...' and missed 'WEDGED<TAB>x'), CASE-SENSITIVE (sleek-owl: 'TIMEOUT: a',
  #      'ABORTED', 'IGNORED: b' were accepted by the space-only form).
  #  Every outcome-like line must be canonical, and each of (b), (c), (d) must count exactly the roster size; so 'fail a',
  #  ' FAIL a', 'FAIL: a' or 'SKIPPED z' beside a full roster refuses, while ordinary diagnostics ('resolve completed',
  #  'claim_batch: ...', '[resolve] ...') match none of them.
  local KW='(pass|passed|fail|failed|skip|skipped|error|unobserved|panicked|abort|aborted|timeout|timedout|timed_out|ignored|xfail|notbool|runtimeerror)'
  local CK='(PASS|PASSED|FAIL|FAILED|SKIP|SKIPPED|ERROR|UNOBSERVED|PANICKED|ABORT|ABORTED|TIMEOUT|TIMEDOUT|TIMED_OUT|IGNORED|XFAIL|NOTBOOL|RUNTIMEERROR)'
  local NR c1 c2 c3 c4 c5 ol; local -a _fa; IFS=, read -ra _fa <<< "$FUNCS"; NR=${#_fa[@]}
  gcnt c1 -a $'\r' "$L"; [ "$c1" = 0 ] || die2 "leg $LEG $T: CR byte in the claim_batch log"
  # counted, not `| grep -q`: under pipefail an early-exiting grep -q can SIGPIPE its writer and fail the pipeline, which
  # would skip the refusal (`&& die2`) on a large log.
  # outcome-like lines captured ONCE with their rc (0 some, 1 none, >= 2 refuse); the non-canonical count is taken over that
  # captured text, not through a pipeline whose status could be lost
  capt ol grep -aiE "^[[:space:]]*$KW([^A-Za-z0-9_]|\$)" "$L"; case $? in 0) gcnt c2 -avE "^$CK [a-z0-9_]+( |\$)" <<< "$ol";; 1) c2=0;; *) die2 "leg $LEG $T: grep error reading the log";; esac
  [ "$c2" = 0 ] || die2 "leg $LEG $T: a malformed outcome-like line (case, leading space, colon)"
  gcnt c3 -aiE "^[[:space:]]*$KW([^A-Za-z0-9_]|\$)" "$L"; [ "$c3" = "$NR" ] || die2 "leg $LEG $T: outcome-like line count != roster size"
  gcnt c4 -aE "^$CK [a-z0-9_]+( |\$)" "$L"; [ "$c4" = "$NR" ] || die2 "leg $LEG $T: canonical outcome line count != roster size"
  gcnt c5 -aE '^[A-Z][A-Z0-9_]*([^A-Za-z0-9_]|$)' "$L"; [ "$c5" = "$NR" ] || die2 "leg $LEG $T: unknown ALLCAPS line (an ALLCAPS token followed by space, tab, colon or end of line) or count != roster size"
  [ "$RC" = "$VE" ] || die2 "leg $LEG $T: claim_batch returned $RC, its own verdict lines say $VE"
  echo "leg $LEG $T ok ($NR fns)"; }
# ROOTS (keen-otter msg_8cdaf17a, calm-crab msg_6627419e): claim runs use EXACTLY the authority's witness roots,
# gunbc.ci.ci_layer_roots witness_layer_roots = ["dag", "src/v2"], so the shared resolve slot claim_batch fills first is keyed
# on the same roots cli_run::resolve_virtual_source_with_imports demands (a different key refuses with
# SharedIndexSecondResidentPool). src/v1 is NOT a root. closure_gate_all <dir> <entry>..., once per job and worktree:
#  (1) the pinned authority row reads exactly that list (else refuse: the authority moved; this packet is stale);
#  (2) ONE `claim_batch --print-entry-closure` (non-evaluating) per job and worktree, over ALL its entries, with those
#      roots: rc 0 and exactly one complete summary for exactly that many entries ARE the evidence (a module findable only
#      under src/v1 makes the loader fail, rc 1; calm-crab msg_a019e137 verified this contract at 26fe4b8f). The row count
#      must match the summary and each entry must have rows; the dag|src/v2 path filter is defence in depth only (with
#      those roots it cannot otherwise fail). Output retained as $R/closure-<worktree>.txt. run_entry refuses an entry the
#      query did not admit.
WITNESS_ROOTS_ROW='data witness_layer_roots: List<String> = ["dag", "src/v2"]'
CQCAP=600   # one closure query per job and worktree; observed 130-160 s (calm-crab msg_a019e137); full cap or refuse
declare -A CLOSED
closure_gate_all() { local d=$1 C c n rows e bad tag; shift; tag="${d##*/}"; C="$R/closure-$tag.txt"; local -a args=()
  local g1 g2 g3 g4 g5; gcnt g1 -xF "$WITNESS_ROOTS_ROW" "$d/dag/gunbc/ci/ci_layer_roots.dag"; [ "$g1" = 1 ] || die2 "closure $tag: the authority witness_layer_roots row is not exactly [dag, src/v2] at the pin"
  c=$(tcap $CQCAP); [ "$c" = "$CQCAP" ] || die2 "closure $tag: remaining budget ${c}s < its full cap ${CQCAP}s; not starting"
  for e in "$@"; do args+=(--entry "$e"); done
  ( cd "$d" && timeout --kill-after=30 $((c-30)) "$TG/release/claim_batch" --print-entry-closure --source-root dag --source-root src/v2 "${args[@]}" ) > "$C" 2>&1 || die2 "closure $tag: entry-closure query failed or timed out (see $C)"
  contain "closure $tag"; gcnt g2 -a $'\r' "$C"; [ "$g2" = 0 ] || die2 "closure $tag: CR in closure output"
  local sl; capt sl grep -a '^\[entry-closure-summary\] ' "$C"; case $? in 0) ;; 1) die2 "closure $tag: no closure summary line";; *) die2 "closure $tag: grep error";; esac
  [[ $sl =~ ^\[entry-closure-summary\]\ state=complete\ entries=([0-9]+)\ path_rows=([1-9][0-9]*)$ ]] && [ "${BASH_REMATCH[1]}" = "$#" ] && n="${BASH_REMATCH[2]}" || die2 "closure $tag: summary is not complete for exactly $# entries"
  gcnt rows -a '^\[entry-closure\] ' "$C"; [ "$rows" = "$n" ] || die2 "closure $tag: $rows closure rows, summary says $n"
  # ONE pass, no pipe (sleek-owl rev11 B1: `| grep -q` under pipefail SIGPIPEd its writer on a real-sized closure and the
  # refusal never ran): the rows of the admitted shape must number EXACTLY the summary's path_rows.
  gcnt bad -aE "^\[entry-closure\] [^ ]+: file=\"(\./)?(dag|src/v2)/[^\"]+\.dag\"$" "$C"; [ "$bad" = "$n" ] || die2 "closure $tag: only $bad of $n closure rows are under dag/ or src/v2/ (see $C)"
  # no '..' path segment anywhere (sleek-owl rev12: dag/../src/v1/x.dag matched the prefix shape)
  gcnt g3 -aE 'file="([^"]*/)?\.\.(/|")' "$C"; [ "$g3" = 0 ] || die2 "closure $tag: a closure path has a '..' segment (see $C)"
  # each requested entry exactly once in argv, each with >= 1 row, and the per-entry row counts SUM to path_rows, so no row
  # belongs to an entry outside the request (keen-otter rev12: multi-entry usage).
  local k sum=0; declare -A seen=(); for e in "$@"; do [ -z "${seen[$e]:-}" ] || die2 "closure $tag: entry $e requested twice"; seen[$e]=1
    gcnt k -aF "[entry-closure] $e: file=" "$C"; [ "$k" -ge 1 ] || die2 "closure $tag: no closure rows for $e"; sum=$((sum+k)); done
  [ "$sum" = "$n" ] || die2 "closure $tag: per-entry rows sum to $sum, summary says $n (a row for an unrequested entry?)"
  for e in "$@"; do CLOSED["$d|$e"]=1; done
  echo "closure $tag: $# entries, $n files; rc 0 + complete summary = every closure resolves WITHOUT src/v1"; }
# fns_csv FILE: the declared test fns, comma-joined. grep rc checked (1 = no tests, >= 2 = error: both return 1); names by builtin.
fns_csv() { local o x; local -a n=(); capt o grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$1" || return 1
  while IFS= read -r x; do n+=("${x##* }"); done <<< "$o"; local IFS=,; printf '%s' "${n[*]}"; }
# roster_gate (r2 override of the frame's, same law): the declaration list is CAPTURED with its rc (the frame's version reads a
# process substitution whose status is lost); rc != 0 refuses.
roster_gate() { local f="$1" o nm w d j found cnt; shift; local -a declared=()
  capt o grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$f" || return 1
  while IFS= read -r nm; do declared+=("${nm##* }"); done <<< "$o"
  for w in "$@"; do cnt=0; for d in "${declared[@]}"; do [ "$d" = "$w" ] && cnt=$((cnt+1)); done; [ "$cnt" = 1 ] || return 1; done
  j=0; for w in "$@"; do found=0; while [ "$j" -lt "${#declared[@]}" ]; do if [ "${declared[$j]}" = "$w" ]; then found=1; j=$((j+1)); break; fi; j=$((j+1)); done; [ "$found" = 1 ] || return 1; done; return 0; }
# verdict (r2 override of the frame's, same law): the grep's rc is classified (0 lines, 1 none -> NONE, >= 2 -> ERROR, which
# run_entry refuses); the single-line count and the trim are builtins. classify is the frame's, unchanged.
verdict() { local L r; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"; r=$?; case $r in 0) ;; 1) echo NONE; return;; *) echo ERROR; return;; esac
  [[ $L == *$'\n'* ]] && { echo NONE; return; }; classify "$L" "$2"; }
declare -A EXPECT; declare -A OBS; DEVIATIONS=0
finish() { echo "OBSERVED:"; for k in "${!OBS[@]}"; do echo "  $k = ${OBS[$k]} (expected ${EXPECT[$k]})"; done | sort
  [ "${#OBS[@]}" = "${#EXPECT[@]}" ] || die2 "observed ${#OBS[@]} of ${#EXPECT[@]} pre-registered verdicts"
  echo "DEVIATIONS=$DEVIATIONS"; [ "$DEVIATIONS" = 0 ]; }
# ---- containment: the job's OWN cgroup is the containment boundary (the frame's scope gate proved $CG is this job's scope).
# B0 = the processes in this cgroup when the job began (this shell, and whatever the launcher placed here). After every
# external run, any OTHER process still in this cgroup is a survivor (a setsid'd descendant included): TERM, bounded wait
# (30 s), KILL, bounded wait (10 s), re-check; survivors remaining -> exit 2. Reads use bash builtins only (mapfile), so the
# check itself adds no process to the set it reads. A job is not complete while survivors exist, and the next job refuses to
# start while this job's scope unit is still active (prior_scope_gone).
shopt -s globstar nullglob
# subtree_procs: every pid in this job's cgroup AND all its sub-cgroups (a child that moved into a delegated sub-cgroup is
# still inside), read with bash builtins only (globstar + mapfile), so the reader adds no process to what it reads.
# subtree_procs_into <array-name>: fills the named array with every pid in this job's cgroup and all sub-cgroups, using bash
# builtins only, in the CURRENT shell (no process substitution or command substitution, so no reader process exists inside the
# cgroup while it is read, and a read failure is returned, never swallowed).
subtree_procs_into() { local -n _acc=$1; _acc=(); _walk_cg "$CG" _acc; }
# _walk_cg <dir> <array>: reads <dir>/cgroup.procs, then recurses into every sub-directory. A directory that is not readable
# AND searchable, or a cgroup.procs that cannot be read, returns 1 -- an unlistable sub-cgroup is never read as "empty".
# Each cgroup is visited exactly once, so each pid appears once.
_walk_cg() { local d=$1 sub; local -n _a=$2; local -a one
  [ -r "$d" ] && [ -x "$d" ] || return 1; mapfile -t one < "$d/cgroup.procs" || return 1; _a+=("${one[@]}")
  # dot-named sub-cgroups too (sleek-owl rev10: '*/' alone skips '.x/'); explicit patterns, so no global dotglob change.
  for sub in "$d"/*/ "$d"/.[!.]*/ "$d"/..?*/; do [ -L "${sub%/}" ] && continue; _walk_cg "${sub%/}" "$2" || return 1; done; return 0; }
subtree_procs_into B0 || die2 "REFUSED_SETUP: cannot read this job's cgroup subtree"
# survivors: sets SURV to the pids in this job's cgroup subtree other than this shell and B0; returns 1 if the read failed.
# Runs in the CURRENT shell (callers never use $(survivors)), so there is no reader subshell to exclude.
survivors() { local -a now; local p b keep; SURV=""; subtree_procs_into now || return 1
  for p in "${now[@]}"; do keep=0; [ "$p" = "$$" ] && keep=1
    for b in "${B0[@]}"; do [ "$p" = "$b" ] && keep=1; done; [ $keep = 0 ] && SURV="$SURV $p"; done; return 0; }
contain() { local i; survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0
  echo "SURVIVORS after $1:$SURV"; kill -TERM $SURV 2>/dev/null
  for i in {1..30}; do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; [ $EPOCHSECONDS -lt $OPDL ] || break; sleep 1; done; kill -KILL $SURV 2>/dev/null
  for i in {1..10}; do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; [ $EPOCHSECONDS -lt $OPDL ] || break; sleep 1; done; die2 "survivors remain in this job's cgroup after $1:$SURV"; }
# contain_selftest: run at the start of every job, INSIDE its real launcher scope (so on the real cgroup v2, not a fake file).
# It starts ONE known child, requires survivors() to report EXACTLY that pid (a reader process, or a missed sub-cgroup, would
# show up as a different or missing pid), kills it, and requires survivors() to report nothing. Any mismatch refuses before work.
contain_selftest() { local cp i; sleep 60 & cp=$!
  # up to 3 s for the listing to equal exactly [the child]; a persistent stray or a missing child still refuses.
  # (reads are bash builtins and add no process; between rounds the external `sleep` is waited for before the next read)
  local k ok=0; for i in 1 2 3 4 5 6; do for k in {1..200}; do survivors || die2 "REFUSED_SETUP: contain self-test: cgroup read failed"; [ "$SURV" = " $cp" ] && { ok=1; break; }; done; [ $ok = 1 ] && break; sleep 0.5; done
  [ "$SURV" = " $cp" ] || { kill $cp 2>/dev/null; die2 "REFUSED_SETUP: contain self-test: survivors() saw [$SURV], expected exactly [ $cp]"; }
  kill -KILL $cp 2>/dev/null; wait $cp 2>/dev/null
  for i in 1 2 3 4 5; do for k in {1..200}; do survivors || die2 "REFUSED_SETUP: contain self-test: re-read failed"; [ -z "$SURV" ] && { echo "contain self-test: held on this job's cgroup"; return 0; }; done; sleep 1; done
  die2 "REFUSED_SETUP: contain self-test: [$SURV] still present after kill"; }
# prior_scope_gone <job>: the prior job's launcher scope must be inactive (no survivors kept it alive) in BOTH managers.
prior_scope_gone() { local v m; for m in --user --system; do v="$(timeout 30 systemctl $m show "$1.scope" -p ActiveState --value)" || die2 "cannot read $1.scope ($m)"
  case "$v" in inactive|failed) ;; *) die2 "prior job scope $1.scope is [$v] in $m manager (survivors or still running)";; esac; done; }
# uprop <unit> <property>: a CHECKED systemctl read (rc 0 required); empty output stays empty and callers decide.
uprop() { local c; c=$(tcap 30); [ "$c" -ge 1 ] || return 124; timeout $c systemctl show "$1" -p "$2" --value; }
# ---- JOIN LAW SELF-TEST (keen-otter msg_2356fbe3). The REAL run_entry is exercised against fixture logs from a stub
# claim_batch, in a private temp dir, before this job does anything else. The join law it must enforce: exactly ONE bare
# PASS|FAIL line per selected name; NO outcome line (^[A-Z]+ ) naming a function outside the roster; NO keyword other than
# PASS|FAIL for a roster name; the batch rc cross-checked against THAT validated set. Each case must give its exit:
#   foreign PASS 2, foreign FAIL 2, foreign SKIPPED 2, duplicate PASS 2, reasoned FAIL 2, UNOBSERVED 2, missing name 2,
#   mixed PASS/FAIL with matching rc 1 -> held (0), mixed PASS/FAIL with rc 0 -> 2, all PASS rc 0 -> held (0).
# The whole self-test runs under ONE aggregate deadline (sleek-owl rev13 m4: "~20 s" was an observation, not a bound):
# JOB_END and CQCAP are shadowed for its callees so every stub run and closure query fits in 240 s from its start, plus at
# most one 30 s kill grace. Measured 3.2 s locally.
join_fixture() { local d c want got lines rc msg JOB_END=$(( EPOCHSECONDS + RESERVE + 240 )) CQCAP=40 OPDL=$(( EPOCHSECONDS + 240 )); d="$(mktemp -d)" || return 1
  mkdir -p "$d/w/dag/gunbc/ci" "$d/t/release" && printf 'test fn x() -> Bool { true }\ntest fn y() -> Bool { true }\n' > "$d/w/dag/e.dag" && printf '%s\n' "$WITNESS_ROOTS_ROW" > "$d/w/dag/gunbc/ci/ci_layer_roots.dag" || { rm -rf -- "$d"; return 1; }
  for c in "foreign-pass|2|PASS x\nPASS y\nPASS z|0|outcome-like line count" "foreign-fail|2|PASS x\nPASS y\nFAIL z|1|outcome-like line count" "foreign-skipped|2|PASS x\nPASS y\nSKIPPED z|0|outcome-like line count" \
           "duplicate-pass|2|PASS x\nPASS x\nPASS y|0|has no SINGLE BARE" "reasoned-fail|2|PASS x\nFAIL y (runtime error [c]: m)|1|has no SINGLE BARE" "unobserved|2|PASS x\nUNOBSERVED y (r)|2|has no SINGLE BARE" \
           "missing-name|2|PASS x|0|has no SINGLE BARE" "mixed-rc-match|0|PASS x\nFAIL y|1|" "mixed-rc-zero|2|PASS x\nFAIL y|0|its own verdict lines say" "all-pass|0|PASS x\nPASS y|0|" \
           "lower-fail|2|PASS x\nPASS y\nfail x|0|malformed outcome-like" "space-fail|2|PASS x\nPASS y\n FAIL x|0|malformed outcome-like" "colon-fail|2|PASS x\nPASS y\nFAIL: x|0|malformed outcome-like" \
           "timedout|2|PASS x\nPASS y\nTimedOut x|0|malformed outcome-like" "cr-byte|2|PASS x\rFAIL x\nPASS y|0|has no SINGLE BARE" "timeout-colon|2|PASS x\nPASS y\nTIMEOUT: x|0|malformed outcome-like" "aborted-bare|2|PASS x\nPASS y\nABORTED|0|malformed outcome-like" \
           "ignored-colon|2|PASS x\nPASS y\nIGNORED: y|0|malformed outcome-like" "unknown-caps-eol|2|PASS x\nPASS y\nWEDGED|0|unknown ALLCAPS" "unknown-caps-tab|2|PASS x\nPASS y\nWEDGED\tx|0|unknown ALLCAPS" \
           "noise-capital-t|0|Started resolve\nStatus: ok\nPASS x\nPASS y|0|" "timed-hyphen|2|PASS x\nPASS y\nTIMED-OUT x|0|unknown ALLCAPS" \
           "fail-paren|2|PASS x\nPASS y\nFAIL(x)|0|malformed outcome-like" "fail-hyphen|2|PASS x\nPASS y\nFAIL-x|0|malformed outcome-like" "failed-dot|2|PASS x\nPASS y\nFAILED.|0|malformed outcome-like" "closure-v1|2|PASS x\nPASS y|0|closure rows are under" "closure-nosummary|2|PASS x\nPASS y|0|no closure summary" "closure-big-v1|2|PASS x\nPASS y|0|closure rows are under" "closure-big-ok|0|PASS x\nPASS y|0|" "closure-foreign-entry|2|PASS x\nPASS y|0|per-entry rows sum" "closure-dotdot|2|PASS x\nPASS y|0|'..' segment" \
           "ignored-tab|2|PASS x\nPASS y\nIGNORED\tx|0|malformed outcome-like" "pass-tab|2|PASS x\tPASS y|0|has no SINGLE BARE" "cr-gate|2|PASS x\nPASS y\nnote\rtail|0|CR byte in the claim_batch log" \
           "noise-pass|0|resolve completed\nclaim_batch: closure subject for e: ok\nmemory-cgroup-bind: bound 1\n[resolve] 3 modules\n[witness] x\nPASS x\nPASS y|0|"; do
    IFS='|' read -r name want lines rc msg <<< "$c"
    cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=1'
    case $name in closure-v1) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure] dag/e.dag: file=\"src/v1/x.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=2';;
                  closure-nosummary) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"';;
                  closure-dotdot) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure] dag/e.dag: file=\"dag/../src/v1/x.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=2';;
                  closure-foreign-entry) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure] dag/other.dag: file=\"dag/o.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=2';;
                  closure-big-v1|closure-big-ok) cl="$(for i in $(seq 1000); do printf '[entry-closure] dag/e.dag: file=\\"dag/m%s.dag\\"\\n' $i; done
                      [ $name = closure-big-v1 ] && printf '[entry-closure] dag/e.dag: file=\\"src/v1/x.dag\\"\\n'
                      for i in $(seq 3000); do printf '[entry-closure] dag/e.dag: file=\\"src/v2/m%s.dag\\"\\n' $i; done
                      printf '[entry-closure-summary] state=complete entries=1 path_rows=%s' $([ $name = closure-big-v1 ] && echo 4001 || echo 4000))";; esac
    printf '#!/bin/bash\n[ "$1" = --print-entry-closure ] && { printf "%s\\n"; exit 0; }\nprintf "%s\\n"\necho '"'"'[cost-partition] {"verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":1}}'"'"'\necho "[process-partition] total_ms=5 a_ms=1"\nexit %s\n' "$cl" "$lines" "$rc" > "$d/t/release/claim_batch" && chmod +x "$d/t/release/claim_batch" || { rm -rf -- "$d"; return 1; }
    ( TG="$d/t"; R="$d"; EXPECT=(); declare -A EXPECT OBS; EXPECT["F.$name.x"]=PASS; EXPECT["F.$name.y"]=$([ $name = mixed-rc-match ] || [ $name = mixed-rc-zero ] && echo FAIL || echo PASS); DEVIATIONS=0
      { closure_gate_all "$d/w" dag/e.dag && run_entry F "$name" "$d/w" dag/e.dag x,y 0 40; } > "$d/$name.out" 2>&1 ) ; got=$?
    [ "$got" = "$want" ] || { echo "join fixture $name: exit $got, required $want"; tail -3 "$d/$name.out"; rm -rf -- "$d"; return 1; }
    # a budget refusal is NEVER a passing expected-refusal case (keen-otter msg_dc650b95), and each refusal must be for ITS reason
    gtest -aE 'remaining budget|time budget exhausted|deadline' "$d/$name.out" && { echo "join fixture $name: refused by the self-test budget, not by its own reason"; rm -rf -- "$d"; return 1; }
    if [ "$want" = 2 ]; then gtest -aF -- "$msg" "$d/$name.out" || { echo "join fixture $name: refused, but not for [$msg]: $(tail -1 "$d/$name.out")"; rm -rf -- "$d"; return 1; }; fi
  done
  # long real-style roster names ending in 't' (silent-bee msg_440d1499: a trailing-class bug strips a final 't' and refuses
  # a real name as outside the roster; short fixture names hide it). All-PASS, rc 0: must be ACCEPTED (exit 0).
  local n1=witness_teardown_dependency_arm_refuses_golden_independent n2=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_receipt
  printf 'test fn %s() -> Bool { true }\ntest fn %s() -> Bool { true }\n' $n1 $n2 > "$d/w/dag/e.dag" || { rm -rf -- "$d"; return 1; }
  printf '#!/bin/bash\n[ "$1" = --print-entry-closure ] && { printf "[entry-closure] dag/e.dag: file=\\"dag/e.dag\\"\\n[entry-closure-summary] state=complete entries=1 path_rows=1\\n"; exit 0; }\nprintf "PASS %s\\nPASS %s\\n"\necho '"'"'[cost-partition] {"verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":1}}'"'"'\necho "[process-partition] total_ms=5 a_ms=1"\nexit 0\n' $n1 $n2 > "$d/t/release/claim_batch" || { rm -rf -- "$d"; return 1; }
  ( TG="$d/t"; R="$d"; declare -A EXPECT OBS; EXPECT["F.long-t.$n1"]=PASS; EXPECT["F.long-t.$n2"]=PASS; DEVIATIONS=0
    { closure_gate_all "$d/w" dag/e.dag && run_entry F long-t "$d/w" dag/e.dag $n1,$n2 0 40; } > "$d/long-t.out" 2>&1 ); got=$?
  [ "$got" = 0 ] || { echo "join fixture long-t: exit $got, required 0"; tail -3 "$d/long-t.out"; rm -rf -- "$d"; return 1; }
  rm -rf -- "$d"; echo "join-law fixtures: all held (36 cases, each refusal checked for its own reason)"; return 0; }
# ---- r2 overrides of three frame helpers on the REAL path (rev15): same law, producer status no longer lost. The frame's
# originals stay byte-for-byte and are still exercised by the frame's own fixtures, which ran above, before these definitions.
# oomread CONTENT KEY -> digits, or 1: exactly one line whose first field is KEY, and it is strictly "KEY <digits>". Builtins.
oomread() { local x n=0 m=0 v=""; while IFS= read -r x; do read -r _f _ <<< "$x"; [ "${_f:-}" = "$2" ] && n=$((n+1)); [[ $x =~ ^$2\ ([0-9]+)$ ]] && { m=$((m+1)); v="${BASH_REMATCH[1]}"; }; done <<< "$1"
  [ $n = 1 ] && [ $m = 1 ] || return 1; printf '%s' "$v"; }
# classify LINE NAME: trailing whitespace trimmed with builtins (the frame's sed status was lost)
classify() { local l="$1"; l="${l%"${l##*[![:space:]]}"}"; if [ "$l" = "PASS $2" ]; then echo PASS; elif [ "$l" = "FAIL $2" ]; then echo FAIL; else echo NONE; fi; }
# completed LOG: exactly one process-partition total_ms record and it is strict; counts via gcnt (grep >= 2 refuses)
completed() { local n m; gcnt n -a '^\[process-partition\] total_ms' "$1"; gcnt m -aE '^\[process-partition\] total_ms=[0-9]+( |$)' "$1"; [ "$n" = 1 ] && [ "$m" = 1 ]; }
# the overrides' self-test: the frame's own oomread fixture inputs, and classify's, with the frame's expected results
oom_selftest() { local x
  for x in $'accepted|oom 0\noom_kill 0' $'refused|oom 0 garbage\noom_kill 0' $'refused|oom 0\noom_kill 0\noom bad' $'refused|oom 0\noom 0\noom_kill 0' \
           $'refused|oom 0\noom_kill 0\noom' $'refused|oom 0\noom_kill 0\noom_kill' $'refused|oom\t0\noom_kill 0' $'refused|oom 0\noom_kill\t0' \
           $'refused|oom\tbad\noom 0\noom_kill 0' $'refused|oom_kill 0'; do
    local want="${x%%|*}" ev="${x#*|}" got; if oomread "$ev" oom > /dev/null && oomread "$ev" oom_kill > /dev/null; then got=accepted; else got=refused; fi
    [ "$got" = "$want" ] || die2 "REFUSED_SETUP: r2 oomread override misfires on [${ev//$'\n'/|}]: $got, expected $want"; done
  [ "$(oomread $'oom 7\noom_kill 3' oom) $(oomread $'oom 7\noom_kill 3' oom_kill)" = "7 3" ] || die2 "REFUSED_SETUP: r2 oomread override value"
  [ "$(classify 'PASS x  ' x) $(classify 'FAIL x' x) $(classify 'FAIL x (r)' x) $(classify 'PASS y' x)" = "PASS FAIL NONE NONE" ] || die2 "REFUSED_SETUP: r2 classify override"
  echo "r2 helper overrides: self-test held"; }
oom_selftest
# ==== end r2 common ====
J1B4_SHA=e1b4094d0dcf223a6bb93328d79de645efa52dd14b171d55a6ec709f55fcbfb1
REVIEWED_SCRIPTS="J1_GREEN:b6bc04295c1dd9ec7ff7dc36a9eacbdaa5d3d387ee2fec7d3f290020be86c59e J1R_GREEN:102cf49082215391a64638ff9f8e1060f4e07b99b4283f03f581fd3132d8d2e0 J1B1_GREEN:8375193aabd9d65d8e85a152fa4139e6e6909627344613c73e41911d154bc104 J1B2_GREEN:2808f37e96c483f69f26021eb266d363798f4b947f9486d0825ab454b1033892 J1B3_GREEN:50a55f6ac0e8ff7eef6a3ac19fd8f9846bef8a732d66bddadec858b92220f07e J1B4_GREEN:e1b4094d0dcf223a6bb93328d79de645efa52dd14b171d55a6ec709f55fcbfb1"
# GRANT LINE REQUIRED: this job's grant must name the production probes explicitly -- `ssh srv1 -- /usr/bin/git -C
#   /opt/gunbc/gunbc rev-parse HEAD` and `ssh srv1 -- /usr/bin/curl -fsS http://127.0.0.1:8080/healthz` (read-only); the
#   `ssh -G srv1` precheck only verifies configuration and authorizes nothing.
# ==== J2 (rev16): LAB MUTATION WITHOUT ROOT. admit -> dashboard apply -> identity -> K1 request. NO install, NO reload, NO start. ====
# rev16 (keen-otter msg_bc8bc508, sleek-owl msg_af194434): every root act and every sudo read left this job. The install of
#   the reviewed serve unit and the daemon-reload are K1.sh (a separate, reviewed root act); start/observe/stop is K2.sh.
#   J2 ends by writing $R/k1-request.txt (the exact 3 lines K1 parses) and the J2_READY marker binding it.
# Arg 2 = sha256 of $WD/render/serve.unit as reviewed and authorized (a separate act after J1R).
# EFFECTS (exact; see APPLY-EFFECTS.md): admit and apply, both as briansrls. The apply's own root acts touch only
#   gunbc-srv1-lab-codex-feedback-v0.service's /etc fragment (stop, mv, daemon-reload) and only when that unit is LOADED; it is
#   not-found here (checked below), so the apply performs no root act in this state. J2 itself runs no sudo at all.
# BUDGET (RT 7200; JOB_END = start + 7140; OPDL = JOB_END - 10). Itemized caps, kill-after graces included:
#   pre-admit: frame fixtures (the frame's own, bounded only by RT), contain_selftest ~10, need chain (wt_ok 240, shas_ok 120,
#     ign snapshot 150, up to ~50 receipt hashes x 60, all under OPDL), ssh -G 30, cp 30+120, hashes 60 each, list-units 2 x 60,
#     inventory fixture (3 inventories, each <= 2 x (120 + 120)), the three inventories before apply (units 480, trigger units
#     30 x 10, provider 240), pre-snapshot 120 + 2 x 30.
#   ADMIT 900 + 30.  between: contain 40, git 30, shas_ok 120 = 190.  APPLY 2400 + 30.
#   post-apply (POST_CAPS = 2960): contain 40, known_hosts 2 x 60, provider 240, trigger units 300, units 480, identity 210,
#     k1-request write + readback 60, contain 40, wt_ok 240, shas_ok 120, mark 1110 (ign 150 + 14 hashes x 60 + 4 x 30).
#   FULL BUDGET BEFORE EACH MUTATION: admit starts only if OPDL - now >= 930 + 190 + 2430 + 2960 + 60 (finalize) = 6570;
#   apply starts only if OPDL - now >= 2430 + 2960 + 60 = 5450. Otherwise the job refuses BEFORE that mutation (pre-admit:
#   PRE_MUTATION, nothing changed; pre-apply: LAB_STATE_CHANGED step=admit, admit done, apply not started). Pre-admit work is
#   therefore limited to OPDL - 6570 = 560 s of real time; a slower start refuses before any mutation.
# PARTIAL-STATE DISPOSITION (admit and apply have NO rollback; nothing here restores anything):
#   pre-admit refusal       -> no change (PRE_MUTATION).
#   step=admit              -> the admitted ref, source checkout and FETCH_HEAD may have moved and roster.dag files may be
#                              deleted; J2_PARTIAL records it; lab down; calm-crab decides.
#   step=admit (pre-apply)  -> admit completed; apply NOT started (budget refusal); J2_PARTIAL; calm-crab decides.
#   step=apply              -> legs idempotent; bin/gunbc may be the witnessed copy or the old one ($R/pre-bin-gunbc holds the
#                              old bytes, NOT restored: they cannot read the moved attempt state); lab down.
#   step=apply-units / step=apply-trigger-units -> the unit inventory or the 5 trigger units changed during apply; lab down.
#   step=identity / step=request -> applied; no K1 request (or an unverified one); lab down.
#   step=done (rm J2_PARTIAL failed after the marker) -> J2_READY exists AND J2_PARTIAL exists: K1 refuses on J2_PARTIAL.
# FINALIZATION: ONE EXIT trap, armed at the first mutation, appends the step/rc to $WD/J2_PARTIAL and reports; it removes
#   NOTHING and reloads NOTHING (J2 holds no root).
# PROVIDER STATE (K5; APPLY-EFFECTS U1/U2): CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap, OUTSIDE the instance root. The
#   apply's ensure-owned-directory row may re-group/re-mode it to 02770 briansrls:gunbc-attempt, and its verify-provider legs
#   run `/usr/bin/env CODEX_HOME=<root> /opt/gunbc/provider-runtime/codex/current/node_modules/@openai/codex-linux-arm64/vendor/
#   aarch64-unknown-linux-musl/bin/codex --version` and `... login status`, which may write under it (unresolved in source).
#   So the directory's stat and every file's (sha256, size, mtime, path) are listed before and after the apply and the DIFF is
#   RECORDED, not refused: that is the explicit, bounded write surface of those legs. File CONTENTS are never printed.
# PATH (APPLY-EFFECTS U4): the apply looks up env, git, find, chgrp, chmod, mktemp, ln, rm and dd on PATH. This job runs admit
#   and apply with PATH=/usr/bin:/bin and records `command -v` for each, so those names resolve to fixed locations.
# PRINCIPAL: the launcher runs this job as briansrls; checked below (id -un).
# PRODUCTION DEPLOY WINDOW: the apply reads production (git rev-parse, /healthz) BEFORE and AFTER its plan; a production deploy
#   between them makes the after-read differ and the apply refuse AFTER mutating (LAB_STATE_CHANGED step=apply). J2's grant must
#   name a window with no production deploy, coordinated with the production owner.
# PRODUCTION PROBES (APPLY-EFFECTS row 0p): the apply (preflight inside it, and before/after the plan) makes read-only
#   production probes over ssh: `ssh srv1 -- /usr/bin/git -C /opt/gunbc/gunbc rev-parse HEAD` and `ssh srv1 -- /usr/bin/curl -fsS
#   http://127.0.0.1:8080/healthz`. No production write. UNRESOLVED: ssh config resolution of `srv1` and known_hosts appends.
# ADMIT (B8; APPLY-EFFECTS section A): repository_converge_owner_wet runs unprivileged as briansrls (whoami, git; no sudo, ssh,
#   systemd-run or network): git fetch of 26fe4b8f from $WT into /opt/gunbc-lab/codex-provider-feedback-v0/source (objects,
#   FETCH_HEAD); `git update-ref refs/gunbc/srv1-lab-admitted <new> <old>` (CAS on the value read at its start); `git reset
#   --hard` there (moves the checked-out branch, index, tracked files, ORIG_HEAD); deletes source/dag/gunbc/{recurring_failure_mode,
#   rung_drop}/roster.dag if present (retire_superseded_workspace_files: cleans files only OLDER binaries wrote; no current writer); writes the receipt non-atomically to $LAB/receipts/repository-convergence.json. It has NO
#   rollback. Nothing creates $LAB/receipts, so J2 requires it to exist and be writable BEFORE admit (else the git changes would
#   land and the receipt write fail).
mem start
contain_selftest
die_lab() { STEP="$1"; die2 "step=$1: $2"; }   # die2 is redefined at the first mutation to print LAB_STATE_CHANGED step=$STEP
SERVE_UNIT_SHA="$EXTRA_ARG"; [[ $SERVE_UNIT_SHA =~ ^[0-9a-f]{64}$ ]] || die2 "PRE_MUTATION: arg 2 must be the reviewed serve unit sha256"
prior_scope_gone ld449-r2-j1b4; need J1B4_GREEN "$J1B4_SHA" J1B3_GREEN
[ "$TG/release/gunbc" = "$DECLARED_SERVE_BINARY" ] || die2 "PRE_MUTATION: declared serve binary path"
# SSH RESOLUTION PRECHECK (non-secret, no connection): `ssh -G srv1` prints the configuration ssh WOULD use. The production
#   probes need it to resolve to a loopback host as user briansrls (srv1-live lives on this host); anything else refuses
#   before any probe. The resolved hostname, user, port, userknownhostsfile, stricthostkeychecking and batchmode are recorded.
sshg="$(timeout 30 ssh -G srv1 2>&1)" || die2 "PRE_MUTATION: ssh -G srv1 failed"
printf '%s\n' "$sshg" | grep -E '^(hostname|user|port|userknownhostsfile|stricthostkeychecking|batchmode) ' > $R/ssh-resolution.txt || die2 "PRE_MUTATION: ssh -G output unreadable"; cat $R/ssh-resolution.txt
# parsed with builtins from the rc-checked capture (no awk whose status is lost)
SG_HOST=""; SG_USER=""; SG_BATCH=""; SG_KH=""; while read -r _k _v _rest; do case "$_k" in hostname) SG_HOST="$_v";; user) SG_USER="$_v";; batchmode) SG_BATCH="$_v";; userknownhostsfile) SG_KH="$_v${_rest:+ $_rest}";; esac; done <<< "$sshg"
case "$SG_HOST" in localhost|127.0.0.1|::1) ;; *) die2 "PRE_MUTATION: ssh srv1 does not resolve to loopback";; esac
[ "$SG_USER" = briansrls ] || die2 "PRE_MUTATION: ssh srv1 user is not briansrls"
# READINESS (non-secret, no connection): the apply's production probes run plain `ssh srv1 -- ...` with no -o options, so their
# EFFECTIVE options are exactly what `ssh -G srv1` reports. Require batchmode yes: ssh then never prompts (no password,
# passphrase or host-key prompt) and fails instead, so an authentication problem is a bounded failure, not a stall. A
# known_hosts match does NOT establish non-interactive authentication; it is RECORDED only. known_hosts shas are recorded
# before and after the apply so an append is visible; the ssh/curl probes are recorded through $R/apply.log.
SSH_HOST="$SG_HOST"; KH="$SG_KH"
[ "$SG_BATCH" = yes ] || die2 "PRE_MUTATION: ssh srv1's effective BatchMode is not yes (the production probes could prompt)"
khok=no; for f in $KH; do f="${f/#\~/$HOME}"; [ -r "$f" ] && ssh-keygen -F "$SSH_HOST" -f "$f" >/dev/null 2>&1 && khok=yes; done; echo "known_hosts entry for $SSH_HOST: $khok (recorded only)"
for f in $KH; do f="${f/#\~/$HOME}"; printf '%s %s\n' "$(hash_t "$f" 2>/dev/null || echo ABSENT)" "$f"; done > $R/known-hosts-before.txt
{ [ -e $MK/J2_READY ] || [ -L $MK/J2_READY ] || [ -e $WD/J2_PARTIAL ]; } && die2 "PRE_MUTATION: J2 already ran (ready or partial); calm-crab decides"
_un="$(id -un)" || die2 "PRE_MUTATION: id -un failed"; [ "$_un" = briansrls ] || die2 "PRE_MUTATION: not running as briansrls"
for n in env git find chgrp chmod mktemp ln rm dd; do c="$(PATH=/usr/bin:/bin command -v $n)" || die2 "PRE_MUTATION: $n not found on PATH=/usr/bin:/bin"; case "$c" in /usr/bin/$n|/bin/$n) ;; *) die2 "PRE_MUTATION: $n resolves to [$c], not /usr/bin or /bin";; esac; printf '%s=%s ' $n "$c"; done > $R/path-resolution.txt; cat $R/path-resolution.txt; echo
LAB=/opt/gunbc-lab/codex-provider-feedback-v0; UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; GB=$TG/release/gunbc
PROVIDER_STATE_ROOT=/home/briansrls/.codex@gunbc-roadmap
# B7: K1 installs from J2's OUT copy (through its own private root copy, re-hashed): hashed here at copy time.
timeout 30 cp -- $WD/render/serve.unit $R/serve.unit || die2 "PRE_MUTATION: copy reviewed render into OUT"
_hs="$(hash_t $R/serve.unit)" || die2 "PRE_MUTATION: cannot hash the OUT copy"; [ "$_hs" = "$SERVE_UNIT_SHA" ] || die2 "PRE_MUTATION: the render is not the reviewed text"
utcnow() { local TZ=UTC0; printf '%(%Y-%m-%dT%H:%M:%SZ)T' -1; }   # builtin clock for records (no date process)
# lu (rev16; independent review BLOCKER 1): EVERY matching unit, in ANY state (--all), whose ACTIVE column is anything but
# inactive or failed counts. No substring exclusion: the old `belt|exit|drain` filter ran on the whole line (name and
# description), so a live attempt unit whose id, key or description contained one of those words read as absent.
lu() { local o x u ld act n=0; o="$(timeout 60 "$@" list-units --all --no-legend --plain --type=service "${UNITPATS[@]}")" || return 1
  while IFS= read -r x; do [ -n "$x" ] || continue; read -r u ld act _ <<< "$x"; case "$act" in inactive|failed) ;; *) n=$((n+1));; esac; done <<< "$o"; printf '%s' "$n"; }
UNITPATS=('gunbc-attempt-*'); SYS="$(lu systemctl)" || die2 "PRE_MUTATION: list-units (system) failed"
UNITPATS=('gunbc-obligation-attempt-*' 'gunbc-audit-*' 'gunbc-compute-*'); USR="$(lu systemctl --user)" || die2 "PRE_MUTATION: list-units (user) failed"
echo "LIVE system_attempt=$SYS user=$USR"; [ "$SYS" = 0 ] && [ "$USR" = 0 ] || die2 "PRE_MUTATION: live attempt/review/compute units present"
{ [ -e $DEST ] || [ -L $DEST ]; } && die2 "PRE_MUTATION: $DEST exists (K1 installs it; it must be absent before J2)"
v="$(uprop $UNIT LoadState)" || die2 "PRE_MUTATION: read LoadState"; [ "$v" = not-found ] || die2 "PRE_MUTATION: serve unit is [$v] before apply"
[ -d $LAB/receipts ] && [ -w $LAB/receipts ] || die2 "PRE_MUTATION: $LAB/receipts missing or not writable (admit would mutate git, then fail its receipt)"
timeout 120 cp -p $LAB/bin/gunbc $R/pre-bin-gunbc || die2 "PRE_MUTATION: preserve current bin/gunbc"
# inv_roots OUT ROOTSPEC...: ROOTSPEC = DIR or DIR:NAMEGLOB (basename filter at any depth). Per root: ONE `find` under one
# timeout lists every entry (D: mode owner; L: mode owner size mtime inode path -> target; F: mode owner size mtime inode path;
# X: other), then ONE timed `find -readable | xargs sha256sum` hashes the readable files (an unreadable file is identified by
# its stat line). Any error or stderr output returns 1; nothing here calls die2, so a refusal never lands in a data file
# (independent review rev15 #6). Caps: 120 + 120 per root, each min(cap, OPDL - now). OUT = the sorted listing + the sorted hashes.
inv_roots() { local out=$1 r root filt c; shift; : > "$out.l" && : > "$out.h" && : > "$out.err" || return 1
  for r in "$@"; do root="${r%%:*}"; filt=""; [ "$root" != "$r" ] && filt="${r#*:}"
    if [ ! -d "$root" ]; then echo "ABSENT $root" >> "$out.l" || return 1; continue; fi
    optc c 120 || return 1
    timeout $c bash -c 'f=(-true); [ -n "$2" ] && f=(-name "$2"); find "$1" -xdev "${f[@]}" \( \( -type d -printf "D %m %u:%g %p\n" \) -o \( -type l -printf "L %m %u:%g %s %T@ %i %p -> %l\n" \) -o \( -type f -printf "F %m %u:%g %s %T@ %i %p\n" \) -o -printf "X %m %u:%g %p\n" \)' _ "$root" "$filt" >> "$out.l" 2>> "$out.err" || return 1
    optc c 120 || return 1
    timeout $c bash -c 'set -o pipefail; f=(-true); [ -n "$2" ] && f=(-name "$2"); find "$1" -xdev "${f[@]}" -type f -readable -print0 | xargs -0r sha256sum --' _ "$root" "$filt" >> "$out.h" 2>> "$out.err" || return 1
  done
  [ -s "$out.err" ] && return 1; { LC_ALL=C sort "$out.l" && echo "--- sha256" && LC_ALL=C sort "$out.h"; } > "$out" || return 1; rm -f -- "$out.l" "$out.h" "$out.err"; }
# FIXTURE (before any mutation): the inventory end to end in a private temp tree: (a) an unchanged tree re-inventories
# IDENTICALLY; (b) a new entry, (c) a file mode change, (d) a directory mode change, (e) a new symlink (an enable), (f) a
# same-size content change each make it DIFFER. Compared with cmp (0 same, 1 differ, >= 2 error -> the fixture fails).
inv_fixture() { local d r=0 k; d="$(mktemp -d)" || return 1
  mkdir "$d/u" "$d/u/x.wants" && printf a > "$d/u/a.service" && ln -s ../a.service "$d/u/x.wants/a.service" || { rm -rf -- "$d"; return 1; }
  inv_roots "$d/b" "$d/u" || r=1; inv_roots "$d/c" "$d/u" || r=1; cmp -s "$d/b" "$d/c"; [ $? = 0 ] || { echo "fixture (a) an unchanged tree differs (or an error)"; r=1; }
  for k in b c d e f; do case $k in b) printf z > "$d/u/stray.conf";; c) chmod 600 "$d/u/a.service";; d) chmod 700 "$d/u/x.wants";; e) ln -s ../a.service "$d/u/x.wants/b.service";; f) printf b > "$d/u/a.service";; esac
    inv_roots "$d/n$k" "$d/u" || r=1; cmp -s "$d/b" "$d/n$k"; [ $? = 1 ] || { echo "fixture ($k) missed a change (or an error)"; r=1; }
    case $k in b) rm -f -- "$d/u/stray.conf";; c) chmod 644 "$d/u/a.service";; d) chmod 755 "$d/u/x.wants";; e) rm -f -- "$d/u/x.wants/b.service";; f) printf a > "$d/u/a.service";; esac; done
  rm -rf -- "$d"; return $r; }
unit_inv() { inv_roots "$1" /etc/systemd/system /etc/systemd/user /home/briansrls/.config/systemd/user "/run/systemd/system:gunbc-srv1-lab-codex-feedback-v0*"; }
PATHS="gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path gunbc-srv1-lab-codex-feedback-v0-publication.path gunbc-srv1-lab-codex-feedback-v0-belt-credential-recovery.timer"
# units_state: each read capped at 10 s (systemctl show is a local D-Bus read), 30 reads -> <= 300 s.
sprop() { local c; optc c 10 || return 124; timeout $c systemctl show "$1" -p "$2" --value; }
units_state() { local u p v; for u in $PATHS; do printf '%s' "$u"
    for p in Id LoadState UnitFileState Triggers FragmentPath; do v="$(sprop $u $p)" || return 1; printf ' %s=%s' $p "$v"; done
    case "$u" in *-publication.path|*-belt-credential-recovery.timer) ;; *) v="$(sprop $u ActiveState)" || return 1; printf ' ActiveState=%s' "$v";; esac; echo; done; }
# (config only: publication.path and the credential-recovery timer may fire during apply, so their volatile Active/Sub states
#  and the triggered services' states are NOT compared; the 3 calm-crab-disabled paths keep ActiveState, which must stay inactive.)
prov_inv() { [ -d "$PROVIDER_STATE_ROOT" ] || { echo "ABSENT $PROVIDER_STATE_ROOT" > "$1"; return 0; }; inv_roots "$1" "$PROVIDER_STATE_ROOT"; }   # hashes only, never contents
inv_fixture || die2 "PRE_MUTATION: install/inventory fixture failed"; echo "inventory fixture: held"
unit_inv $R/units-before.txt || die2 "PRE_MUTATION: unit inventory failed"; echo "unit inventory recorded: $R/units-before.txt"
units_state > $R/paths-before.txt || die2 "PRE_MUTATION: trigger unit state read failed"; cat $R/paths-before.txt
prov_inv $R/provider-before.txt || die2 "PRE_MUTATION: provider inventory failed"
{ timeout 120 sha256sum $LAB/bin/gunbc $LAB/bin/gunbc.prev; timeout 30 git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted HEAD; timeout 30 ls $LAB/attempt-state/attempt-identities | sha256sum; } > $R/pre-snapshot.txt 2>&1; cat $R/pre-snapshot.txt
# The three .path units calm-crab disabled must STILL be inactive+disabled before apply (they are re-enabled only by J3).
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path; do
  a="$(uprop $u ActiveState)" || die2 "PRE_MUTATION: read $u"; e="$(uprop $u UnitFileState)" || die2 "PRE_MUTATION: read $u"
  [ "$a" = inactive ] && [ "$e" = disabled ] || die2 "PRE_MUTATION: $u is [$a/$e]; must be inactive/disabled before apply"; done
# ---- finalization: armed at the first mutation ----
STEP=pre; finalize() { local rc=$?; [ "$STEP" = done ] && return; [ "$STEP" = pre ] && return
  # Records the step and rc in J2_PARTIAL and reports. It removes NOTHING and reloads NOTHING (J2 holds no root).
  echo "FINALIZE begin step=$STEP primary_rc=$rc"
  if printf 'J2_PARTIAL step=%s rc=%s at=%s out=%s\n' "$STEP" "$rc" "$(utcnow)" "$R" >> $WD/J2_PARTIAL; then echo "FINALIZE J2_PARTIAL updated"; else echo "UNRESOLVED_CLEANUP: J2_PARTIAL update failed (it was written before admit, so a rerun is still blocked)"; fi
  echo "FINALIZE end step=$STEP primary_rc=$rc (pre-apply bin/gunbc retained at $R/pre-bin-gunbc, NOT restored)"
  return $rc; }
trap 'exit 143' TERM
trap finalize EXIT
cd $WT || die2 "PRE_MUTATION: cd wt"
printf 'J2_PARTIAL step=begin at=%s out=%s (written BEFORE admit; removed only at DONE)\n' "$(utcnow)" "$R" > $WD/J2_PARTIAL || die2 "PRE_MUTATION: cannot write J2_PARTIAL ahead of admit"
# FULL BUDGET BEFORE EACH MUTATION (header BUDGET): refuse BEFORE admit / BEFORE apply when the rest cannot fit.
NEED_ADMIT=6570; NEED_APPLY=5450
_rem=$(( OPDL - EPOCHSECONDS )); [ $_rem -ge $NEED_ADMIT ] || die2 "PRE_MUTATION: ${_rem}s left before OPDL < ${NEED_ADMIT}s needed for admit + apply + post-apply caps + finalize; refused before admit"
STEP=admit; die2() { echo "LAB_STATE_CHANGED step=$STEP"; echo "NO OBSERVATION (exit 2): $*"; exit 2; }; snap admit
PATH=/usr/bin:/bin timeout --kill-after=30 900 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/live_deploy/repository_convergence_placement.dag --function repository_converge_owner_wet \
  --arg candidate=$HEAD_EXPECT --arg object_source=$WT --arg receipt=$LAB/receipts/repository-convergence.json --arg instance_id=srv1-lab-codex-feedback-v0 > $R/admit.log 2>&1
A=$?; contain admit; judge_oom admit; echo "ADMIT_RC=$A"; tail -5 $R/admit.log | cut -c1-400; [ $A = 0 ] || die_lab admit "admit rc=$A (900 s cap)"
v="$(timeout 30 git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted)" || die_lab admit "read admitted ref"; [ "$v" = $HEAD_EXPECT ] || die_lab admit "admitted ref reads $v"
shas_ok
_rem=$(( OPDL - EPOCHSECONDS )); [ $_rem -ge $NEED_APPLY ] || die_lab admit "admit done; ${_rem}s left < ${NEED_APPLY}s needed for apply + post-apply caps + finalize; apply NOT started"
STEP=apply; snap apply
PATH=/usr/bin:/bin timeout --kill-after=30 2400 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/roadmap/roadmap_dashboard_instance_apply.dag --function srv1_lab_dashboard_apply_wet > $R/apply.log 2>&1
A=$?; contain apply; judge_oom apply; echo "APPLY_RC=$A"; grep -vE "◐|✓|^\[pre-entry\]" $R/apply.log | tail -20 | cut -c1-400; [ $A = 0 ] || die_lab apply "apply rc=$A (2400 s cap)"
for f in $KH; do f="${f/#\~/$HOME}"; printf '%s %s\n' "$(hash_t "$f" 2>/dev/null || echo ABSENT)" "$f"; done > $R/known-hosts-after.txt; echo "KNOWN_HOSTS_DIFF (recorded, not refused):"; diff $R/known-hosts-before.txt $R/known-hosts-after.txt; true
prov_inv $R/provider-after.txt || die_lab apply "provider inventory after"; echo "PROVIDER_STATE_DIFF (recorded, not refused):"; diff $R/provider-before.txt $R/provider-after.txt; true
STEP=apply-units; unit_inv $R/units-after-apply.txt || die_lab apply-units "unit inventory after apply failed"
cmp -s $R/units-before.txt $R/units-after-apply.txt; case $? in 0) ;; 1) diff $R/units-before.txt $R/units-after-apply.txt | head -40; die_lab apply-units "the unit-config inventory changed during apply (the apply installs nothing in this state)";; *) die_lab apply-units "unit inventory compare error";; esac
STEP=apply-trigger-units; units_state > $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state read failed"; diff $R/paths-before.txt $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state changed during apply"
STEP=identity; capt IV timeout 30 $LAB/bin/gunbc --version || die_lab identity "served gunbc --version failed or timed out (rc $?)"; IV="${IV%%$'\n'*}"; IS="$(hash_t $LAB/bin/gunbc)" || die_lab identity "hash served"; JS="$(hash_t $GB)" || die_lab identity "hash witnessed"
SH="$(timeout 30 git -C $LAB/source rev-parse HEAD)" || die_lab identity "read source head"; ST="$(timeout 30 git -C $LAB/source rev-parse 'HEAD^{tree}')" || die_lab identity "read source tree"
echo "IDENTITY served_version=$IV served_sha=$IS witnessed_sha=$JS lab_source_head=$SH lab_source_tree=$ST" > $WD/identity.txt || die_lab identity "write identity.txt"; cat $WD/identity.txt
[ "$IS" = "$JS" ] || die_lab identity "served bytes are not the witnessed bytes"
[ "$IV" = "gunbc $HEAD_EXPECT" ] && [ "$SH" = $HEAD_EXPECT ] && [ "$ST" = $TREE_EXPECT ] || die_lab identity "version/source readback"
{ [ -e $DEST ] || [ -L $DEST ]; } && die_lab identity "$DEST appeared during apply"
# K1 REQUEST (the LAST act): exactly the 3 lines K1.sh parses; written, read back and compared (status before content).
STEP=request; _kr="$(readlink -e -- $R/serve.unit)" || die_lab request "cannot canonicalize the OUT unit copy"
_hs="$(hash_t "$_kr")" || die_lab request "cannot re-hash the OUT unit copy"; [ "$_hs" = "$SERVE_UNIT_SHA" ] || die_lab request "the OUT unit copy changed since its precheck"
printf -v K1REQ 'K1_REQUEST unit=%s dest=%s\nK1_UNIT_SHA %s\nK1_UNIT_TEXT %s' "$UNIT" "$DEST" "$SERVE_UNIT_SHA" "$_kr"
wfile $R/k1-request.txt "$K1REQ" || die_lab request "write k1-request.txt"; _rb="$(timeout 30 cat -- $R/k1-request.txt)" || die_lab request "read back k1-request.txt"; [ "$_rb" = "$K1REQ" ] || die_lab request "k1-request.txt readback differs"
_rs="$(hash_t $R/k1-request.txt)" || die_lab request "hash k1-request.txt"; echo "K1 REQUEST written: $R/k1-request.txt sha256=$_rs"; cat $R/k1-request.txt
contain final; wt_ok; shas_ok
mark J2_READY J1B4_GREEN $WD/identity.txt $R/serve.unit $R/k1-request.txt $R/admit.log $R/apply.log $R/units-before.txt $R/units-after-apply.txt $R/paths-before.txt $R/paths-after.txt $R/oom-raw.log
timeout 30 rm -f -- $WD/J2_PARTIAL || die_lab done "cannot remove J2_PARTIAL after success (J2_READY exists; K1 refuses while J2_PARTIAL exists)"
STEP=done; echo DONE; exit 0
