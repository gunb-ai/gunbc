#!/bin/bash
# r2 restoration packet REVISION 11, job J4 (launcher job ld449-r2-j4). Author loyal-deer-449. NOT a grant request; byte review first.
# FRAME: #15967 packet_G.sh v4 (sha c781341a...), itself #15695 v4a (hardened frame #15931 v12 lineage): die2, scope gate, cgroup
#   bound gates, OOM parser/judge, verdict classifier, completion parser (REAL line verbatim, sha-checked), rc contract,
#   tree_clean, out_reusable, hash_of and their fixtures are copied BYTE FOR BYTE; only the job name in the scope gate and
#   its fixture, and the booked cgroup values, differ. Exit contract: 0 held, 1 deviation from a pre-registered verdict,
#   2 no observation (every refusal, timeout 124, signal >128, OOM, incoherent counter, survivors).
# LAUNCH: calmcrab-udrv.sh ld449-r2-j4 2400 4 3 256 100% <STAGED> <SHA> OUT <J3_RECEIPT_PATH> <J3_RECEIPT_SHA>
# PINS: r2 head 26fe4b8f350a (tree b0ee022a6bf8) = merged integration abcbf27e4cc3 (#15990) + #15919 own commits + r2 1/2..5/5;
#   RED = r2 1/2 154afcd00667. One external process at a time; survivors in this job's cgroup are killed and verified gone.
# JOIN LAW ENFORCED: exactly one bare PASS|FAIL per selected name; no outcome line naming a function outside the roster; no
#   keyword other than PASS|FAIL for a roster name; batch rc cross-checked against that validated set. Jobs that run claim_batch
#   self-test this through the REAL run_entry (10 fixture logs) before anything else, and refuse if any case gives the wrong exit.
# AUTHORITY: calm-crab-645's launcher receipts (frozen.sha256, drv.log, printed SHAS) and the J3 receipt are what bind; the
#   markers written here are EVIDENCE for the next job's precheck only, never authority. Every job needs its own explicit grant.
set -uo pipefail
JOB=ld449-r2-j4
UNIT="$JOB.scope"
JOB_RT=2400
die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
# F3 scope gate: exactly one cgroup record, matched against a LITERAL suffix with a case glob (the quoted part is literal; no regex, no unescaped dot)
scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
[ "$(grep -c '' /proc/self/cgroup)" = 1 ] && scope_ok "$(cat /proc/self/cgroup)" || die2 "REFUSED_SETUP: /proc/self/cgroup is not exactly one 0:: line ending in /$UNIT"
CG="/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup)"
# PRE-WORK ACTUAL-BOUND CHECK (before any fetch): every bound read from the cgroup itself and compared to the booked value; mismatch or unreadable = exit 2
want() { local f="$CG/$1" v; v="$(cat "$f" 2>/dev/null)" || die2 "REFUSED_SETUP: $f unreadable"; [ "$v" = "$2" ] || die2 "REFUSED_SETUP: $1 is [$v], booked [$2]"; }
want memory.max 4294967296
want memory.high 3221225472
want memory.swap.max 0
want pids.max 256
want cpu.max "100000 100000"
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
[ "$(r scope_ok '0::/a/ld449-r2-j4.scope') $(r scope_ok '0::/a/ld449-r2-j4Xscope') $(r scope_ok '0::/a/xld449-r2-j4.scope') $(r scope_ok '0::/a/ld449-r2-j4.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
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
[ "$#" = 3 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir> <j3-receipt-path> <j3-receipt-sha256>"
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
T0=$(( $(date +%s) - SECONDS )); JOB_END=$(( T0 + JOB_RT - 60 )); RESERVE=300   # fixed; not overridable from the environment
# OPDL: the ONE operation deadline (keen-otter, rev10 review). Every timed bookkeeping operation (wt_ok, ign_snapshot,
# shas_ok, receipt hashing in mark, containment waits) takes timeout = min(its own cap, OPDL - now) and FAILS when nothing is
# left, so a phase made of several such operations is bounded AS A WHOLE by OPDL, never by the sum of per-command caps.
OPDL=$(( JOB_END - 10 ))
# optc VAR N: VAR = min(N, OPDL - now), or return 1 when < 1 s remains (never 0: `timeout 0` would mean NO timeout).
optc() { local -n _oc=$1; local r=$(( OPDL - $(date +%s) )); [ $r -ge 1 ] || return 1; if [ $r -lt $2 ]; then _oc=$r; else _oc=$2; fi; }
# hash_t FILE: hash_of under the operation deadline (cap 60 s). The frame's hash_of stays byte-for-byte; mark and shas_id use this.
hash_t() { local c o h; optc c 60 || return 1; o="$(timeout $c sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }
tcap() { local r=$(( JOB_END - RESERVE - $(date +%s) )); [ $r -lt 1 ] && { echo 0; return; }; [ $r -lt $1 ] && echo $r || echo $1; }
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
shas_ok() { local c; optc c 120 || die2 "deadline before the binary sha check"; [ -f $WD/SHAS ] && timeout $c sha256sum -c --quiet $WD/SHAS >/dev/null 2>&1 || die2 "binary shas changed or missing (or the check timed out)"; }
shas_id() { hash_t $WD/SHAS; }
# Markers (B1). A marker is EVIDENCE for the next precheck, never authority (each job is its own grant; calm-crab verifies the
# launcher receipts -- frozen.sha256, drv.log, printed SHAS -- independently). It is one field per line:
#   HEAD, TREE, SCRIPT (the writing job's own sha, taken before cd), SHAS (binaries), OUT (the writing job's output dir),
#   PRIOR <name> <sha256 of that marker file>, and one RECEIPT <sha256> <absolute path> per retained receipt.
# need() re-reads reality before trusting a marker: the worktree's HEAD and tree (wt_ok), the binaries (shas_ok), and then the
# WHOLE chain back to J1: every marker's fields, every RECEIPT re-hashed, every PRIOR link re-hashed. Stated limit: a process
# running as the same user can still fabricate consistent files; the markers raise the cost of that, they do not prevent it.
mark() { local name=$1 prior=$2 ps=none f h t body; shift 2; [ "$prior" != none ] && { ps="$(hash_t $MK/$prior)" || die2 "cannot hash prior marker $prior"; }
  mkdir -p $MK || die2 "mkdir markers"; t=$MK/.$name.tmp
  ign_snapshot $R/ignored-end.txt || die2 "ignored-set snapshot"
  body="$(printf 'HEAD %s\nTREE %s\nSCRIPT %s\nSHAS %s\nOUT %s\nPRIOR %s %s' $HEAD_EXPECT $TREE_EXPECT $SELF_SHA "$(shas_id)" "$R" $prior $ps)" || die2 "marker body $name"
  for f in "$@" $R/ignored-end.txt; do h="$(hash_t "$f")" || die2 "marker $name: cannot hash receipt $f"; body="$body"$'\n'"RECEIPT $h $f"; done
  printf '%s\n' "$body" > $t || die2 "marker write $name"; [ "$(cat $t)" = "$body" ] || die2 "marker readback $name"
  local c; optc c 30 || die2 "deadline before marker rename $name"; timeout $c mv -f -T -- $t $MK/$name || die2 "marker rename $name"; cat $MK/$name; }
mfield() { awk -v k="$2" '$1==k{print substr($0,length(k)+2)}' "$1"; }
# Expected receipts per marker, by EXACT location: W:<path relative to the work dir> or O:<path relative to that marker's own
# OUT>. A receipt elsewhere, an extra one, or a missing one refuses (no basename-only matching).
EXPECTED_RECEIPTS="J1_GREEN=W:SHAS,O:build.log,O:claims-G-off.log,O:claims-R-off.log,O:oom-raw.log,O:ignored-end.txt
J1R_GREEN=W:render/serve.unit,W:render/belt.txt,W:render/review.txt,W:render/RENDER_SHAS,O:preflight.log,O:oom-raw.log,O:ignored-end.txt
J1B1_GREEN=O:claims-G-session_placement_wet_witness.log,O:claims-G-roadmap_belt_actuate_witness.log,O:claims-G-roadmap_dashboard_instance_witness.log,O:oom-raw.log,O:ignored-end.txt
J1B2_GREEN=O:claims-G-attempt_system_unit_launch_witness.log,O:claims-G-attempt_state_move_apply_wet_witness.log,O:claims-G-lab_apply_operator_grant_witness.log,O:oom-raw.log,O:ignored-end.txt
J1B3_GREEN=O:claims-G-worker_git_subtree_apply_spawn_wet_witness.log,O:claims-G-dispatch_worker_confinement_witness.log,O:claims-G-operator_endpoint_witness.log,O:claims-G-agent_door_witness.log,O:oom-raw.log,O:ignored-end.txt
J1B4_GREEN=O:claims-G-request_authentication_witness.log,O:claims-G-roadmap_event_log_witness.log,O:claims-G-harness_validate_worktree_witness.log,O:claims-G-dispatch_worker_traversal_witness.log,O:oom-raw.log,O:ignored-end.txt
J2_INSTALLED=W:INSTALLED_SHAS,W:identity.txt,O:serve.unit,O:admit.log,O:apply.log,O:units-before.txt,O:units-pre-reload.txt,O:units-after.txt,O:paths-before.txt,O:paths-after-reload.txt,O:oom-raw.log,O:ignored-end.txt"
# OUT must lie under the launcher's own directory for the job that writes that marker (no foreign OUT).
MARKER_JOB="J1_GREEN:ld449-r2-j1 J1R_GREEN:ld449-r2-j1r J1B1_GREEN:ld449-r2-j1b1 J1B2_GREEN:ld449-r2-j1b2 J1B3_GREEN:ld449-r2-j1b3 J1B4_GREEN:ld449-r2-j1b4 J2_INSTALLED:ld449-r2-j2 J4_UP:ld449-r2-j4"
RUNS_ROOT=/home/briansrls/runs
# LAUNCHER OUT (assumption to be attested by calm-crab-645): calmcrab-udrv runs job <JOB> with OUT = $RUNS_ROOT/<JOB>/out.
# A marker's OUT must be EXACTLY that path for the job that writes it.
LAUNCHER_OUT_LEAF=out
need_one() { local m=$MK/$1 want_script=$2 want_prior=$3 v h f pn psha out exp got cf cout cwd
  [ -L $m ] && die2 "marker $1 is a symlink"; [ -f $m ] || die2 "marker $1 missing"
  [ "$(mfield $m HEAD)" = $HEAD_EXPECT ] && [ "$(mfield $m TREE)" = $TREE_EXPECT ] || die2 "marker $1: head/tree"
  [ "$(mfield $m SHAS)" = "$(shas_id)" ] || die2 "marker $1: binaries differ"
  v="$(mfield $m SCRIPT)"; [[ $v =~ ^[0-9a-f]{64}$ ]] || die2 "marker $1: script field"; [ -z "$want_script" ] || [ "$v" = "$want_script" ] || die2 "marker $1 not written by reviewed script $want_script"
  [ "$(grep -c '^PRIOR ' $m)" = 1 ] || die2 "marker $1: PRIOR"; pn="$(mfield $m PRIOR | cut -d' ' -f1)"; psha="$(mfield $m PRIOR | cut -d' ' -f2)"
  [ "$pn" = "$want_prior" ] || die2 "marker $1 chains to $pn, expected $want_prior"
  [ "$pn" = none ] || { h="$(hash_of $MK/$pn)" || die2 "hash $pn"; [ "$h" = "$psha" ] || die2 "marker $1's prior $pn changed since it was written"; }
  [ "$(grep -c '^RECEIPT ' $m)" -ge 1 ] || die2 "marker $1 binds no receipts"
  out="$(mfield $m OUT)"; [ "$(grep -c '^OUT ' $m)" = 1 ] && [[ $out = /* ]] && [ -d "$out" ] && [ ! -L "$out" ] || die2 "marker $1: OUT [$out] is not a real absolute directory"
  cout="$(readlink -e -- "$out")" && [ "$cout" = "$out" ] || die2 "marker $1: OUT [$out] is not canonical"; cwd="$(readlink -e -- "$WD")" || die2 "work dir does not resolve"
  while read -r _ h f; do
    # canonicalize (resolve .., ., and symlinked directories) before the prefix check; the receipt must ALREADY be canonical.
    cf="$(readlink -e -- "$f")" || die2 "marker $1: receipt $f does not resolve"; [ "$cf" = "$f" ] || die2 "marker $1: receipt path $f is not canonical ($cf)"
    case "$cf" in "$cout"/*|"$cwd"/*) ;; *) die2 "marker $1: receipt $f is outside its OUT and the work dir";; esac
    [ -L "$f" ] && die2 "marker $1: receipt $f is a symlink"; [ "$(hash_of "$f")" = "$h" ] || die2 "marker $1: receipt $f changed or missing"; done < <(grep '^RECEIPT ' $m)
  local x job=""; for x in $MARKER_JOB; do [ "${x%%:*}" = "$1" ] && job="${x##*:}"; done; [ -n "$job" ] || die2 "marker $1: no writing job"
  [ "$cout" = "$RUNS_ROOT/$job/$LAUNCHER_OUT_LEAF" ] || die2 "marker $1: OUT [$cout] is not the launcher OUT $RUNS_ROOT/$job/$LAUNCHER_OUT_LEAF"
  grep -vE '^(HEAD|TREE|SCRIPT|SHAS|OUT|PRIOR|RECEIPT) ' $m | grep -q . && die2 "marker $1 has unknown lines"
  exp="$(printf '%s\n' "$EXPECTED_RECEIPTS" | awk -F= -v k="$1" '$1==k{print $2}' | tr ',' '\n' | sed -e "s#^W:#$cwd/#" -e "s#^O:#$cout/#" | sort)"; [ -n "$exp" ] || die2 "marker $1: no expected receipt set"
  got="$(grep '^RECEIPT ' $m | awk '{print $3}' | sort)"; [ "$got" = "$exp" ] || die2 "marker $1: receipts are not exactly the expected set at the expected locations"
  NEXT_PRIOR="$pn"; }
# need <marker> <EXACT reviewed script sha> <its prior marker>: verifies the named marker, then walks and verifies the chain to J1.
CHAIN_PRIORS="J1R_GREEN:J1_GREEN J1B1_GREEN:J1R_GREEN J1B2_GREEN:J1B1_GREEN J1B3_GREEN:J1B2_GREEN J1B4_GREEN:J1B3_GREEN J2_INSTALLED:J1B4_GREEN J1_GREEN:none"
reviewed_sha() { local x; for x in $REVIEWED_SCRIPTS; do [ "${x%%:*}" = "$1" ] && { echo "${x##*:}"; return 0; }; done; return 1; }
need() { local cur exp x rs f; wt_ok; shas_ok; need_one $1 $2 $3; cur="$NEXT_PRIOR"
  # ignored-path set must be exactly what the predecessor job recorded at its end (no change between jobs).
  f="$(grep '^RECEIPT .*/ignored-end.txt$' $MK/$1 | awk '{print $3}')"; [ -n "$f" ] || die2 "marker $1 has no ignored-set receipt"
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
ign_snapshot() { local o c; optc c 120 || return 1; o="$(timeout $c git -C $WT status --porcelain --ignored --untracked-files=all)" || return 1; { printf '%s\n' "$o" | grep '^!! ' || true; } | sort > "$1"; }   # an empty set is valid (grep's no-match rc is not a failure)
# Writes OUTSIDE OUT by wet witnesses (B3): they mktemp under FIXED /tmp templates, which other sessions running as the same
# user also use, so a new same-UID entry appearing during a run is NOT proof this run created it. This job therefore DELETES
# NOTHING under /tmp: it records every entry matching those templates that appeared during each wet run in
# $R/tmp-residue-candidates.txt (path, stat signature) as CANDIDATE residue of unproven provenance, retained for a separately
# reviewed cleanup.
TMP_GLOBS='/tmp/gunbc_g5_* /tmp/gunbc_pub_* /tmp/gunbc_attempt_state_move_wet_* /tmp/wre3-*'
tmp_list() { local g p; for g in $TMP_GLOBS; do for p in $g; do [ -e "$p" ] && [ -O "$p" ] && echo "$p"; done; done | sort; }
tmp_record() { local p; comm -13 <(printf '%s\n' "$1") <(tmp_list) | while read -r p; do printf '%s %s %s\n' "$2" "$(stat -c '%a %U:%G %i %Y' -- "$p" 2>&1)" "$p"; done >> $R/tmp-residue-candidates.txt || die2 "cannot record /tmp residue candidates"; }
# run_entry LEG TAG DIR ENTRY FUNCS(csv) WET(0|1) CAP : one claim_batch run, serial, under timeout CAP (timeout puts the run in
# its own process group and signals the group); every fn must have exactly one bare verdict; rc must equal this run's verdicts.
run_entry() { local LEG="$1" T="$2" DIR="$3" F="$4" FUNCS="$5" WET="$6" CAP="$7" L RC VE VD N A=""
  L="$R/claims-$LEG-$T.log"; [ "$WET" = 1 ] && A=--wet
  roster_gate "$DIR/$F" $(echo "$FUNCS" | tr ',' ' ') || die2 "leg $LEG $T: roster not declared exactly once, in order, in $F"
  closure_gate "$LEG $T" "$DIR" "$F"
  local TB=""; [ "$WET" = 1 ] && TB="$(tmp_list)"
  snap "run $LEG $T"
  local CC; CC=$(tcap "$CAP"); [ "$CC" -ge 31 ] || die2 "time budget exhausted before $LEG $T"; CC=$((CC-30))
  ( cd "$DIR" && timeout --kill-after=30 "$CC" "$TG/release/claim_batch" $A --claim-run --source-root dag --source-root src/v2 --entry "$F" --functions "$FUNCS" ) > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; contain "run $LEG $T"; [ "$WET" = 1 ] && tmp_record "$TB" "$LEG.$T"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
  rc_in_contract "$RC" || die2 "leg $LEG $T: exit status $RC out of contract (0/1/2; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg $LEG $T: no single completion line; batch incomplete"
  grep -qE "closure diagnostics|resolve failed|resolve did not reach" "$L" && die2 "leg $LEG $T: closure/resolve refusal (NO_OBSERVATION)"
  # rc 1 can also come from a Refused cost partition (claim_batch.rs batch_exit_code); require exactly one Reconciled one.
  [ "$(grep -ac '^\[cost-partition\] ' "$L")" = 1 ] && grep -aq '^\[cost-partition\] .*"verdict":{"state":"Reconciled"' "$L" || die2 "leg $LEG $T: cost partition not exactly one Reconciled line"
  VE=0
  for N in $(echo "$FUNCS" | tr ',' ' '); do
    VD="$(verdict "$L" "$N")"
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
  local NR; NR="$(echo "$FUNCS" | tr ',' '\n' | grep -c .)"
  grep -aq $'\r' "$L" && die2 "leg $LEG $T: CR byte in the claim_batch log"
  grep -aiE "^[[:space:]]*$KW([^A-Za-z0-9_]|\$)" "$L" | grep -avE "^$CK [a-z0-9_]+( |\$)" | grep -aq . && die2 "leg $LEG $T: a malformed outcome-like line (case, leading space, colon)"
  [ "$(grep -aciE "^[[:space:]]*$KW([^A-Za-z0-9_]|\$)" "$L")" = "$NR" ] || die2 "leg $LEG $T: outcome-like line count != roster size"
  [ "$(grep -acE "^$CK [a-z0-9_]+( |\$)" "$L")" = "$NR" ] || die2 "leg $LEG $T: canonical outcome line count != roster size"
  [ "$(grep -acE '^[A-Z][A-Z0-9_]*([^A-Za-z0-9_]|$)' "$L")" = "$NR" ] || die2 "leg $LEG $T: unknown ALLCAPS line (an ALLCAPS token followed by space, tab, colon or end of line) or count != roster size"
  [ "$RC" = "$VE" ] || die2 "leg $LEG $T: claim_batch returned $RC, its own verdict lines say $VE"
  echo "leg $LEG $T ok ($(echo "$FUNCS" | tr ',' '\n' | grep -c .) fns)"; }
# ROOTS (keen-otter msg_8cdaf17a, calm-crab msg_6627419e): claim runs use EXACTLY the authority's witness roots,
# gunbc.ci.ci_layer_roots witness_layer_roots = ["dag", "src/v2"], so the shared resolve slot claim_batch fills first is keyed
# on the same roots cli_run::resolve_virtual_source_with_imports demands (a different key refuses with
# SharedIndexSecondResidentPool). src/v1 is NOT a root. closure_gate <what> <dir> <entry>, before every claim run:
#  (1) the pinned authority row reads exactly that list (else refuse: the authority moved; this packet is stale);
#  (2) `claim_batch --print-entry-closure` (non-evaluating) with those roots answers rc 0, exactly one complete summary line
#      for one entry, as many [entry-closure] rows as it reports, and every row's file under dag/ or src/v2/ -- i.e. the
#      entry's closure resolves WITHOUT src/v1, executed, not inferred. Output retained as $R/closure-<leg>-<tag>.txt.
WITNESS_ROOTS_ROW='data witness_layer_roots: List<String> = ["dag", "src/v2"]'
closure_gate() { local what=$1 d=$2 f=$3 C c n rows; C="$R/closure-${what// /-}.txt"
  [ "$(grep -cxF "$WITNESS_ROOTS_ROW" "$d/dag/gunbc/ci/ci_layer_roots.dag")" = 1 ] || die2 "$what: the authority witness_layer_roots row is not exactly [dag, src/v2] at the pin"
  budget c 900 "closure query $what"
  ( cd "$d" && timeout --kill-after=30 "$c" "$TG/release/claim_batch" --print-entry-closure --source-root dag --source-root src/v2 --entry "$f" ) > "$C" 2>&1 || die2 "$what: entry-closure query failed or timed out (see $C)"
  contain "closure $what"; grep -aq $'\r' "$C" && die2 "$what: CR in closure output"
  [ "$(grep -ac '^\[entry-closure-summary\] ' "$C")" = 1 ] || die2 "$what: not exactly one closure summary line"
  n="$(sed -nE 's/^\[entry-closure-summary\] state=complete entries=1 path_rows=([1-9][0-9]*)$/\1/p' "$C")"; [ -n "$n" ] || die2 "$what: closure summary is not complete for exactly one entry"
  rows="$(grep -ac '^\[entry-closure\] ' "$C")"; [ "$rows" = "$n" ] || die2 "$what: $rows closure rows, summary says $n"
  grep -a '^\[entry-closure\] ' "$C" | grep -avqE "^\[entry-closure\] [^ ]+: file=\"(\./)?(dag|src/v2)/[^\"]+\.dag\"$" && die2 "$what: a closure file is outside dag/ and src/v2/ (see $C)"
  echo "closure $what: $n files, all under dag/ or src/v2/"; }
fns_csv() { grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$1" | awk '{print $NF}' | paste -sd,; }
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
  for i in {1..30}; do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; [ $(date +%s) -lt $OPDL ] || break; sleep 1; done; kill -KILL $SURV 2>/dev/null
  for i in {1..10}; do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; [ $(date +%s) -lt $OPDL ] || break; sleep 1; done; die2 "survivors remain in this job's cgroup after $1:$SURV"; }
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
join_fixture() { local d c want got lines rc; d="$(mktemp -d)" || return 1
  mkdir -p "$d/w/dag/gunbc/ci" "$d/t/release" && printf 'test fn x() -> Bool { true }\ntest fn y() -> Bool { true }\n' > "$d/w/dag/e.dag" && printf '%s\n' "$WITNESS_ROOTS_ROW" > "$d/w/dag/gunbc/ci/ci_layer_roots.dag" || { rm -rf -- "$d"; return 1; }
  for c in "foreign-pass|2|PASS x\nPASS y\nPASS z|0" "foreign-fail|2|PASS x\nPASS y\nFAIL z|1" "foreign-skipped|2|PASS x\nPASS y\nSKIPPED z|0" \
           "duplicate-pass|2|PASS x\nPASS x\nPASS y|0" "reasoned-fail|2|PASS x\nFAIL y (runtime error [c]: m)|1" "unobserved|2|PASS x\nUNOBSERVED y (r)|2" \
           "missing-name|2|PASS x|0" "mixed-rc-match|0|PASS x\nFAIL y|1" "mixed-rc-zero|2|PASS x\nFAIL y|0" "all-pass|0|PASS x\nPASS y|0" \
           "lower-fail|2|PASS x\nPASS y\nfail x|0" "space-fail|2|PASS x\nPASS y\n FAIL x|0" "colon-fail|2|PASS x\nPASS y\nFAIL: x|0" \
           "timedout|2|PASS x\nPASS y\nTimedOut x|0" "cr-byte|2|PASS x\rFAIL x\nPASS y|0" "timeout-colon|2|PASS x\nPASS y\nTIMEOUT: x|0" "aborted-bare|2|PASS x\nPASS y\nABORTED|0" \
           "ignored-colon|2|PASS x\nPASS y\nIGNORED: y|0" "unknown-caps-eol|2|PASS x\nPASS y\nWEDGED|0" "unknown-caps-tab|2|PASS x\nPASS y\nWEDGED\tx|0" \
           "noise-capital-t|0|Started resolve\nStatus: ok\nPASS x\nPASS y|0" "timed-hyphen|2|PASS x\nPASS y\nTIMED-OUT x|0" \
           "fail-paren|2|PASS x\nPASS y\nFAIL(x)|0" "fail-hyphen|2|PASS x\nPASS y\nFAIL-x|0" "failed-dot|2|PASS x\nPASS y\nFAILED.|0" "closure-v1|2|PASS x\nPASS y|0" "closure-nosummary|2|PASS x\nPASS y|0" \
           "ignored-tab|2|PASS x\nPASS y\nIGNORED\tx|0" "pass-tab|2|PASS x\tPASS y|0" \
           "noise-pass|0|resolve completed\nclaim_batch: closure subject for e: ok\nmemory-cgroup-bind: bound 1\n[resolve] 3 modules\n[witness] x\nPASS x\nPASS y|0"; do
    IFS='|' read -r name want lines rc <<< "$c"
    cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=1'
    case $name in closure-v1) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"\n[entry-closure] dag/e.dag: file=\"src/v1/x.dag\"\n[entry-closure-summary] state=complete entries=1 path_rows=2';;
                  closure-nosummary) cl='[entry-closure] dag/e.dag: file=\"dag/e.dag\"';; esac
    printf '#!/bin/bash\n[ "$1" = --print-entry-closure ] && { printf "%s\\n"; exit 0; }\nprintf "%s\\n"\necho '"'"'[cost-partition] {"verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":1}}'"'"'\necho "[process-partition] total_ms=5 a_ms=1"\nexit %s\n' "$cl" "$lines" "$rc" > "$d/t/release/claim_batch" && chmod +x "$d/t/release/claim_batch" || { rm -rf -- "$d"; return 1; }
    ( TG="$d/t"; R="$d"; EXPECT=(); declare -A EXPECT OBS; EXPECT["F.$name.x"]=PASS; EXPECT["F.$name.y"]=$([ $name = mixed-rc-match ] || [ $name = mixed-rc-zero ] && echo FAIL || echo PASS); DEVIATIONS=0
      run_entry F "$name" "$d/w" dag/e.dag x,y 0 60 > "$d/$name.out" 2>&1 ) ; got=$?
    [ "$got" = "$want" ] || { echo "join fixture $name: exit $got, required $want"; cat "$d/$name.out" | tail -3; rm -rf -- "$d"; return 1; }
  done
  # long real-style roster names ending in 't' (silent-bee msg_440d1499: a trailing-class bug strips a final 't' and refuses
  # a real name as outside the roster; short fixture names hide it). All-PASS, rc 0: must be ACCEPTED (exit 0).
  local n1=witness_teardown_dependency_arm_refuses_golden_independent n2=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_receipt
  printf 'test fn %s() -> Bool { true }\ntest fn %s() -> Bool { true }\n' $n1 $n2 > "$d/w/dag/e.dag" || { rm -rf -- "$d"; return 1; }
  printf '#!/bin/bash\n[ "$1" = --print-entry-closure ] && { printf "[entry-closure] dag/e.dag: file=\\"dag/e.dag\\"\\n[entry-closure-summary] state=complete entries=1 path_rows=1\\n"; exit 0; }\nprintf "PASS %s\\nPASS %s\\n"\necho '"'"'[cost-partition] {"verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":1}}'"'"'\necho "[process-partition] total_ms=5 a_ms=1"\nexit 0\n' $n1 $n2 > "$d/t/release/claim_batch" || { rm -rf -- "$d"; return 1; }
  ( TG="$d/t"; R="$d"; declare -A EXPECT OBS; EXPECT["F.long-t.$n1"]=PASS; EXPECT["F.long-t.$n2"]=PASS; DEVIATIONS=0
    run_entry F long-t "$d/w" dag/e.dag $n1,$n2 0 60 > "$d/long-t.out" 2>&1 ); got=$?
  [ "$got" = 0 ] || { echo "join fixture long-t: exit $got, required 0"; tail -3 "$d/long-t.out"; rm -rf -- "$d"; return 1; }
  rm -rf -- "$d"; echo "join-law fixtures: all held (31 cases)"; return 0; }
# ==== end r2 common ====
J2_SHA=5b0fe82c21bd97a7b674103c403674809d82c12adc07c159eecfd043bf6f43e6
REVIEWED_SCRIPTS="J1_GREEN:64974aa7151dd8024fc767c5102bd403836ab3989e30d273863cd9a291aff71f J1R_GREEN:c66165d83aab75748cad5fad9ac4aff63903be8e21ea2dc39b09690cdc5410e2 J1B1_GREEN:40bc077422a6abbe229b845407e7a8144dfbfb2fea032fe8b2aabe2625fe1050 J1B2_GREEN:2cee29f0592d836551a87ab97673c8688ab75f5cbae92b1d562b69a4c6ff3754 J1B3_GREEN:a996eec597b8dfcfbb2460d9d876aa4cc27f08d0a0a69af1acf1300da37d96c0 J1B4_GREEN:3d5d1a8ede46506709b374a4dfdfe42a4ef04269539ab56c57f9483ed0b46b0c J2_INSTALLED:5b0fe82c21bd97a7b674103c403674809d82c12adc07c159eecfd043bf6f43e6"
# ==== J4: start the unit J2 installed (exits draining per J3) and read back. Start only; no enable. ====
# RESTART POLICY (checked in J1R from the rendered text): Restart=on-failure, no StartLimit directive (systemd defaults:
#   5 starts per 10 s), [Install] WantedBy present, so the unit reads disabled until a separate enable.
# OWNERSHIP: J4 makes NO concurrency or ownership inference. It identifies its start ONLY by the InvocationID read
#   immediately after its own `start`. The only stop it may perform (authority to be stated explicitly in J4's grant) is a stop
#   of this unit while the unit still carries THAT InvocationID. If the InvocationID changes for ANY reason (a Restart=on-failure
#   crash-restart or another actor), J4 does NOT stop: it reports UNRESOLVED with the readings and exits 2; the decision to stop
#   the unit is then calm-crab-645's, under its own act. Stated risk: with Restart=on-failure and a slow-failing serve
#   (~11 min to bind), a crash loop can continue until that decision.
# J3 RECEIPT (B9): J4 gates on calm-crab-645's J3 RECEIPT, not on live state alone. Args: OUT <j3-receipt-path> <its sha256>.
#   The receipt format is fixed by J3-PLAN.txt: one line per unit, "J3_OK <unit> ActiveState=active UnitFileState=enabled
#   Triggers=<service>", for exactly the three units. The live state is then re-read and must agree.
# AFTER J4 EXITS 0 the serve is the restored lab service. Its Restart=on-failure restarts from then on are normal service
#   behaviour, owned by loyal-deer-449 as lab owner; a crash loop observed later is escalated to calm-crab-645 for a stop
#   under its own act. J4 claims nothing about invocations after its own exit.
# POST-START BOOKKEEPING (after LISTENING: contain, wt_ok, shas_ok, mark incl. ign_snapshot and every receipt hash) is bounded
#   AS A WHOLE by the one operation deadline OPDL = JOB_END - 10 (body-common): every operation takes min(its cap, OPDL - now)
#   and fails when nothing is left. A failure there is labelled "serve is UP ... bookkeeping failed" (die2 below) -- honest,
#   because the stop path is not taken after a verified listener.
# ROOT ACTS: `sudo -n -l` shows only what the live sudoers PERMITS; it is NOT operator authorization. The start and the
#   invocation-scoped stop need their own explicit scope in J4's grant.
# EVERY read is checked (rc 0 required); a failed read is UNRESOLVED, never "no listener" or "down".
# A stop is reported as "lab down" ONLY when verified: stop rc 0, ActiveState inactive|failed, the unit's cgroup empty or
#   gone across its WHOLE subtree (M3), and no listener on :18081 (checked ss). The wait loop has a deadline 300 s before the
#   job RT (M2), so J4 always reaches an outcome line inside its scope.
mem start
contain_selftest
# TIME BUDGET (M2; keen-otter): one overall deadline. RT is 2400 s from scope start; JOB_END = scope start + 2400 - 60 (margin for
#   the frame's exit). STOP_RESERVE = 360 s is held back for the bounded stop (120) + verification reads (<=6 x 30) + slack.
#   WORK_END = JOB_END - STOP_RESERVE. Every operation before the stop path takes timeout = min(its own cap, WORK_END - now);
#   if that is < 1 s the operation is not attempted and the path goes to fail_ours (which runs inside the reserve).
# POST_RESERVE = 600 s is SIZING, not the bound: it is held back so a listener accepted at WORK_END leaves >= 950 s to OPDL for
#   the post-listen phase (whose per-operation caps sum to ~880 s: contain 40, wt_ok 240, shas_ok 120, mark: ign_snapshot 120
#   + up to 6 hashes x 60 + rename 30). The BOUND is OPDL.
T0=$(( $(date +%s) - SECONDS )); JOB_END=$(( T0 + 2400 - 60 )); STOP_RESERVE=360; POST_RESERVE=600; WORK_END=$(( JOB_END - STOP_RESERVE - POST_RESERVE ))
cap() { local r=$(( WORK_END - $(date +%s) )); [ $r -ge 1 ] || { echo 0; return; }; [ $r -lt $1 ] && echo $r || echo $1; }
prior_scope_gone ld449-r2-j2; need J2_INSTALLED "$J2_SHA" J1B4_GREEN
J3R="$EXTRA_ARG"; J3S="${3:-}"; [[ $J3R = /* ]] && [ -f "$J3R" ] && [[ $J3S =~ ^[0-9a-f]{64}$ ]] || die2 "usage: OUT <j3-receipt-path> <j3-receipt-sha256>"
[ "$(hash_of "$J3R")" = "$J3S" ] || die2 "J3 receipt is not the cited bytes"
# J3 receipt (J4-4): EXACTLY three lines and nothing else, each exactly
#   J3_OK <unit>.path ActiveState=active UnitFileState=enabled Triggers=<unit>.service InvocationID=<id> ActiveEnterTimestampMonotonic=<n>
# where the last two are calm-crab's per-act readings, which J4 re-reads LIVE and requires equal (a receipt that does not
# match the units' actual current activation refuses). The receipt is evidence; calm-crab's grant is the authority.
[ "$(wc -l < "$J3R")" = 3 ] || die2 "J3 receipt is not exactly 3 lines"
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit gunbc-srv1-lab-codex-feedback-v0-belt-discovery gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain; do
  iv="$(uprop $u.path InvocationID)" || die2 "read $u.path InvocationID"; ts="$(uprop $u.path ActiveEnterTimestampMonotonic)" || die2 "read $u.path timestamp"
  [ -n "$iv" ] && [ -n "$ts" ] || die2 "$u.path has no live InvocationID/activation timestamp"
  want="J3_OK $u.path ActiveState=active UnitFileState=enabled Triggers=$u.service InvocationID=$iv ActiveEnterTimestampMonotonic=$ts"
  [ "$(grep -cxF "$want" "$J3R")" = 1 ] || die2 "J3 receipt line for $u.path does not equal the live reading"; done
timeout 30 cp -- "$J3R" $R/j3-receipt.txt || die2 "copy J3 receipt into OUT"
[ -e $WD/J2_PARTIAL ] && die2 "J2_PARTIAL present"
fresh_own J4_UP
UNIT=gunbc-srv1-lab-codex-feedback-v0.service
timeout 120 sha256sum -c --quiet $WD/INSTALLED_SHAS >/dev/null 2>&1 || die2 "installed unit or bin/gunbc changed since J2 (or unreadable)"
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path; do
  a="$(uprop $u ActiveState)" || die2 "read $u ActiveState"; e="$(uprop $u UnitFileState)" || die2 "read $u UnitFileState"
  [ "$a" = active ] && [ "$e" = enabled ] || die2 "$u is [$a/$e], need active/enabled (J3 not done)"; done
a="$(uprop $UNIT LoadState)" || die2 "read LoadState"; b="$(uprop $UNIT ActiveState)" || die2 "read ActiveState"
fp="$(uprop $UNIT FragmentPath)" || die2 "read FragmentPath"; [ "$fp" = /etc/systemd/system/$UNIT ] || die2 "FragmentPath is [$fp]"
dp="$(uprop $UNIT DropInPaths)" || die2 "read DropInPaths"; [ -z "$dp" ] || die2 "unit has drop-ins [$dp] not reviewed"
nr="$(uprop $UNIT NeedDaemonReload)" || die2 "read NeedDaemonReload"; [ "$nr" = no ] || die2 "NeedDaemonReload=[$nr]"
# sudo_rule_ok <exact argv>: parses `sudo -n -ll` (one block per sudoers rule) and requires a rule whose RunAsUsers includes
# root (or ALL), whose Options include !authenticate (NOPASSWD), and whose Commands include the EXACT argv, the bare program
# (sudoers: no args = any args), or ALL. Wrong runas, a PASSWD rule, a prefix-only match, or the same program with other args
# do not satisfy it, and a rule that carries ANY negated command ('!/usr/bin/x', e.g. beside ALL) never counts: this precheck
# does not evaluate sudoers' negation semantics, so it refuses them (sleek-owl rev10). This reads the live sudoers as listed; it is a precheck, NOT authorization (the grant authorizes).
sudo_rule_ok() { local want="$1" prog="${1%% *}"
  printf '%s\n' "$SLL" | awk -v want="$want" -v prog="$prog" '
    function flush() { if (inrule && runas && noauth && cmdok && !neg) found=1; inrule=0; runas=0; noauth=0; cmdok=0; incmd=0; neg=0 }
    /^Sudoers entry/ { flush(); inrule=1; next }
    inrule && /^[[:space:]]*RunAsUsers:/ { incmd=0; n=split(substr($0, index($0,":")+1), u, ","); for (i=1;i<=n;i++) { gsub(/^[ \t]+|[ \t]+$/,"",u[i]); if (u[i]=="root"||u[i]=="ALL") runas=1 } next }
    inrule && /^[[:space:]]*Options:/ { incmd=0; if ($0 ~ /!authenticate/) noauth=1; next }
    inrule && /^[[:space:]]*Commands:/ { incmd=1; next }
    inrule && incmd && /^[[:space:]]+[^[:space:]]/ { c=$0; gsub(/^[ \t]+|[ \t]+$/,"",c); gsub(/,$/,"",c); if (c ~ /^!/) neg=1; if (c==want || c==prog || c=="ALL") cmdok=1; next }
    END { flush(); exit(found ? 0 : 1) }'; }
SLL="$(timeout 30 sudo -n -ll 2>&1)" || die2 "sudo -n -ll failed"; printf '%s\n' "$SLL" > $R/sudo-ll.txt
for sa in "/usr/bin/systemctl start $UNIT" "/usr/bin/systemctl stop $UNIT"; do sudo_rule_ok "$sa" || die2 "no sudoers rule permits [$sa] as root without a password"; done
printf 'start/stop: root,!authenticate,matching\n' > $R/sudo-authority.txt
[ "$a" = loaded ] && [ "$b" = inactive ] || die2 "unit is [$a/$b] before start, need loaded/inactive"
listen() { timeout 30 ss -Hltn 'sport = :18081'; }   # stop-path read (inside the reserve); rc != 0 => the read failed
wprop() { local c; c=$(cap 30); [ "$c" -ge 1 ] || return 124; timeout $c systemctl show "$1" -p "$2" --value; }   # work-path read
l="$(listen)" || die2 "ss read failed before start"; [ -z "$l" ] || die2 ":18081 already has a listener before start"
# ONE checked cgroup-subtree reader, used for BOTH listener membership and stopped-state verification: every cgroup.procs
# under the unit's cgroup; a find or read error fails the read (rc 1), never reads as "empty".
# The WHOLE traversal (find + every cgroup.procs read) is ONE subprocess under ONE timeout <to> (keen-otter rev10: a per-read
# timeout multiplies with the number of sub-cgroups). Timeout, a find error, any unreadable cgroup.procs, or any stderr -> rc 1.
# Anything the timeout leaves behind is inside this job's own cgroup and is handled by contain.
cg_procs() { local d=${CGROOT:-/sys/fs/cgroup}$1 to=${2:-30}; [ -d "$d" ] || return 2; [ "$to" -ge 1 ] || return 1
  timeout --kill-after=5 "$to" bash -c 'set -o pipefail; find "$1" -name cgroup.procs -print0 | xargs -0r cat --' _ "$d" > $R/.cgp 2> $R/.cgp.err || return 1
  [ -s $R/.cgp.err ] && return 1; grep -E '^[0-9]+$' $R/.cgp; return 0; }
stop_verified() { local a cg l p
  cg="$(uprop $UNIT ControlGroup)" || return 1   # read BEFORE the stop: systemd may empty it once inactive
  timeout 120 sudo -n /usr/bin/systemctl stop $UNIT || { echo "stop rc failed"; return 1; }
  a="$(uprop $UNIT ActiveState)" || return 1; case "$a" in inactive|failed) ;; *) echo "after stop ActiveState=$a"; return 1;; esac
  if [ -n "$cg" ]; then p="$(cg_procs "$cg")"; case $? in 0) [ -z "$p" ] || { echo "cgroup subtree still has: $p"; return 1; };; 2) ;; *) return 1;; esac; fi
  l="$(listen)" || return 1; [ -z "$l" ] || { echo "listener remains"; return 1; }; return 0; }
unresolved() { echo "UNRESOLVED: $1"; timeout 30 systemctl show $UNIT -p ActiveState -p SubState -p Result -p InvocationID -p NRestarts -p ExecMainStatus 2>&1 | tr '\n' ' '; echo; die2 "$1 (lab state UNRESOLVED; calm-crab-645 decides)"; }
fail_ours() { local cur; cur="$(uprop $UNIT InvocationID)" || unresolved "$1; InvocationID read failed"
  [ "$cur" = "$INV" ] || unresolved "$1; InvocationID is now [$cur], not this start's [$INV]; not stopped"
  if stop_verified; then die2 "$1; this start's invocation stopped and verified down"; else unresolved "$1; stop failed or could not be verified"; fi; }
NR0="$(uprop $UNIT NRestarts)" || die2 "read NRestarts"
c=$(cap 300); [ "$c" -ge 1 ] || die2 "no time budget left to start"; timeout $c sudo -n /usr/bin/systemctl start $UNIT; r=$?
INV="$(uprop $UNIT InvocationID)" || unresolved "start rc=$r; InvocationID read failed"
echo "START_RC=$r INVOCATION=[$INV] NRESTARTS0=$NR0"
t=$(cap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE before the post-start property read"
timeout $t systemctl show $UNIT -p LoadState -p ActiveState -p SubState -p Result -p ExecMainPID -p ExecMainStatus -p NRestarts -p ControlGroup -p InvocationID -p User -p CapabilityBoundingSet -p NoNewPrivileges -p RestrictSUIDSGID > $R/serve-props.txt || fail_ours "post-start property read failed or timed out"; cat $R/serve-props.txt
if [ -z "$INV" ]; then b="$(uprop $UNIT ActiveState)" || unresolved "start rc=$r; empty InvocationID; ActiveState read failed"
  case "$b" in inactive|failed) die2 "start rc=$r produced no invocation; unit [$b]; nothing to stop";; *) unresolved "start rc=$r; empty InvocationID while unit is [$b]";; esac; fi
[ $r = 0 ] || fail_ours "start rc=$r"
LP=""
while :; do
  [ "$(cap 60)" -ge 1 ] || fail_ours "DEADLINE: work budget exhausted before a listener appeared"
  c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out while waiting"; [ "$c" = "$INV" ] || fail_ours "INVOCATION_CHANGED"
  b="$(wprop $UNIT ActiveState)" || fail_ours "ActiveState read failed or timed out while waiting"; case "$b" in active|activating) ;; *) fail_ours "STARTUP_ERROR ActiveState=$b";; esac
  t=$(cap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE before ss"; o="$(timeout $t ss -Hltnp 'sport = :18081')" || fail_ours "ss read failed or timed out while waiting"
  LP="$(printf '%s' "$o" | grep -oE 'pid=[0-9]+' | cut -d= -f2 | sort -u | tr '\n' ' ')"
  if [ -n "$o" ]; then [ "$(printf '%s\n' "$o" | grep -c .)" = "$(printf '%s\n' "$o" | grep -c 'pid=[0-9]')" ] || fail_ours "a :18081 listener line carries no pid (not attributable)"; break; fi; t=$(cap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE"; sleep $t
done
CG2="$(wprop $UNIT ControlGroup)" || fail_ours "ControlGroup read failed or timed out"
c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out"
[ -n "$CG2" ] && [ -d "${CGROOT:-/sys/fs/cgroup}$CG2" ] || fail_ours "unit cgroup missing"
t=$(cap 30); procs="$(cg_procs "$CG2" "$t")" || fail_ours "cannot read the unit cgroup subtree (failed or out of budget)"
[ -n "$LP" ] && [ "$c" = "$INV" ] || fail_ours "NOT_LISTENING_OR_NOT_IDENTIFIED pids=[${LP:-none}] (ss -p shows pids only for this user's processes)"
for p in $LP; do printf '%s\n' "$procs" | grep -qx "$p" || fail_ours "listener pid $p is not in $UNIT's cgroup subtree"; done
echo "LISTENING pid=$LP cgroup=$CG2 invocation=$INV"
die2() { echo "UNRESOLVED: the serve is UP and listening (invocation $INV) but post-start bookkeeping failed: $*"; echo "NO OBSERVATION (exit 2): $*"; exit 2; }
contain final; wt_ok; shas_ok
mark J4_UP J2_INSTALLED $R/serve-props.txt $R/j3-receipt.txt $R/sudo-authority.txt; echo DONE; exit 0
