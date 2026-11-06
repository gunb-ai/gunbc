#!/bin/bash
# r2 restoration packet REVISION 5, job J4 (launcher job ld449-r2-j4). Author loyal-deer-449. NOT a grant request; byte review first.
# FRAME: #15967 packet_G.sh v4 (sha c781341a...), itself #15695 v4a (hardened frame #15931 v12 lineage): die2, scope gate, cgroup
#   bound gates, OOM parser/judge, verdict classifier, completion parser (REAL line verbatim, sha-checked), rc contract,
#   tree_clean, out_reusable, hash_of and their fixtures are copied BYTE FOR BYTE; only the job name in the scope gate and
#   its fixture, and the booked cgroup values, differ. Exit contract: 0 held, 1 deviation from a pre-registered verdict,
#   2 no observation (every refusal, timeout 124, signal >128, OOM, incoherent counter).
# LAUNCH: calmcrab-udrv.sh ld449-r2-j4 2400 4 3 256 100% <STAGED> <SHA> OUT
# PINS: r2 head 26fe4b8f350a (tree b0ee022a6bf8) = merged integration abcbf27e4cc3 (#15990) + #15919 own commits + r2 1/2..5/5;
#   RED = r2 1/2 154afcd00667. Every job runs one claim_batch / gunbc invocation at a time (no parallel children).
set -uo pipefail
JOB=ld449-r2-j4
UNIT="$JOB.scope"
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
[ "$#" = 1 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir>"
if out_reusable "$1"; then die2 "REFUSED_SETUP: output path $1 already exists (dir, file or symlink); reuse is refused"; fi
mkdir -- "$1" || die2 "mkdir $1 (exclusive, no -p)"
cd "$1" || die2 "cd $1"
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
HEAD_EXPECT=26fe4b8f350a99bd57a9bc342d4625b0cb96804b; TREE_EXPECT=b0ee022a6bf8f9f9f35d496174afd681300c9847
RED_REV=154afcd00667d3c84609f551c45e0ab4f5b83bad; NO15175=f38a00cb75d85e3a7879c27f7288c2a8bba993ae
WD=/home/briansrls/runs/ld449-r2work; WT=$WD/wt; WR=$WD/wt-red; TG=$WD/target; MK=$WD/markers
DECLARED_SERVE_BINARY=/home/briansrls/runs/ld449-r2work/target/release/gunbc
OFF=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_record
SPW=dag/test/claim/roadmap/session_placement_wet_witness.dag
export PATH=$HOME/.cargo/bin:$PATH
# SELF identity: the launcher runs `bash "$F"` on the frozen copy, so $0 is that file.
SELF_SHA="$(hash_of "$0")" || die2 "REFUSED_SETUP: cannot hash own script $0"
shas_ok() { [ -f $WD/SHAS ] && sha256sum -c --quiet $WD/SHAS >/dev/null 2>&1 || die2 "binary shas changed or missing"; }
shas_id() { hash_of $WD/SHAS; }
# Markers: written ONLY after every gate of the job, atomically, and FATAL on failure. A marker binds head, tree, the
# writing script's sha, the SHAS sha and the prior marker (name + sha). It is evidence for the next precheck, never authority.
mark() { local name=$1 prior=${2:-none} ps=none; [ "$prior" != none ] && { ps="$(hash_of $MK/$prior)" || die2 "cannot hash prior marker $prior"; }
  mkdir -p $MK || die2 "mkdir markers"
  printf 'HEAD=%s TREE=%s SCRIPT=%s SHAS=%s PRIOR=%s:%s\n' $HEAD_EXPECT $TREE_EXPECT $SELF_SHA "$(shas_id)" $prior $ps > $MK/.$name.tmp || die2 "marker write $name"
  mv -f $MK/.$name.tmp $MK/$name || die2 "marker rename $name"; cat $MK/$name; }
# need <marker> <EXACT reviewed sha256 of the script that must have written it> [<prior marker it must chain to>]
need() { local m=$MK/$1 line pf pn psha
  [ -f $m ] || die2 "prerequisite marker $1 missing"
  line="$(cat $m)"; case "$line" in "HEAD=$HEAD_EXPECT TREE=$TREE_EXPECT SCRIPT=$2 SHAS=$(shas_id) PRIOR="*) ;; *) die2 "marker $1 not written by reviewed script $2 for this head/tree/build";; esac
  pf="${line##*PRIOR=}"; pn="${pf%%:*}"; psha="${pf##*:}"
  [ "${3:-none}" = "$pn" ] || die2 "marker $1 chains to $pn, expected ${3:-none}"
  [ "$pn" = none ] || [ "$(hash_of $MK/$pn)" = "$psha" ] || die2 "marker $1's prior $pn changed since it was written"; }
# run_entry LEG TAG DIR ENTRY FUNCS(csv) WET(0|1) CAP : one claim_batch run, serial, under timeout CAP (timeout puts the run in
# its own process group and signals the group); every fn must have exactly one bare verdict; rc must equal this run's verdicts.
run_entry() { local LEG="$1" T="$2" DIR="$3" F="$4" FUNCS="$5" WET="$6" CAP="$7" L RC VE VD N A=""
  L="$R/claims-$LEG-$T.log"; [ "$WET" = 1 ] && A=--wet
  roster_gate "$DIR/$F" $(echo "$FUNCS" | tr ',' ' ') || die2 "leg $LEG $T: roster not declared exactly once, in order, in $F"
  snap "run $LEG $T"
  ( cd "$DIR" && timeout --kill-after=30 "$CAP" "$TG/release/claim_batch" $A --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$F" --functions "$FUNCS" ) > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
  rc_in_contract "$RC" || die2 "leg $LEG $T: exit status $RC out of contract (0/1/2; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg $LEG $T: no single completion line; batch incomplete"
  grep -qE "closure diagnostics|resolve failed|resolve did not reach" "$L" && die2 "leg $LEG $T: closure/resolve refusal (NO_OBSERVATION)"
  VE=0
  for N in $(echo "$FUNCS" | tr ',' ' '); do
    VD="$(verdict "$L" "$N")"
    [ "$VD" = NONE ] && die2 "leg $LEG $T: '$N' has no SINGLE BARE PASS/FAIL verdict"
    [ "$VD" = FAIL ] && VE=1
    OBS["$LEG.$T.$N"]="$VD"
    [ "$VD" != "${EXPECT["$LEG.$T.$N"]:-NONE}" ] && { DEVIATIONS=$((DEVIATIONS+1)); echo "DEVIATION $LEG $T $N: observed $VD, pre-registered ${EXPECT["$LEG.$T.$N"]:-NONE}"; }
  done
  [ "$(grep -acE '^(PASS|FAIL|UNOBSERVED|PANICKED|ERROR)( |$)' "$L")" = "$(echo "$FUNCS" | tr ',' '\n' | grep -c .)" ] || die2 "leg $LEG $T: outcome line count != roster size (foreign or extra outcome)"
  [ "$RC" = "$VE" ] || die2 "leg $LEG $T: claim_batch returned $RC, its own verdict lines say $VE"
  echo "leg $LEG $T ok ($(echo "$FUNCS" | tr ',' '\n' | grep -c .) fns)"; }
fns_csv() { grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$1" | awk '{print $NF}' | paste -sd,; }
declare -A EXPECT; declare -A OBS; DEVIATIONS=0
finish() { echo "OBSERVED:"; for k in "${!OBS[@]}"; do echo "  $k = ${OBS[$k]} (expected ${EXPECT[$k]})"; done | sort
  [ "${#OBS[@]}" = "${#EXPECT[@]}" ] || die2 "observed ${#OBS[@]} of ${#EXPECT[@]} pre-registered verdicts"
  echo "DEVIATIONS=$DEVIATIONS"; [ "$DEVIATIONS" = 0 ]; }
# ==== end r2 common ====
J2_SHA=86358d794fbf50026a538883e757276169bd22373caff6075273ee1dcd9ea05f
# ==== J4: start the unit J2 installed (exits draining per J3) and read back. Start only; no enable. ====
# RESTART POLICY (render checked in J1R): Restart=on-failure, no StartLimit directive (systemd defaults: 5 starts / 10 s).
# INVOCATION OUTCOMES (stop authority below must be EXPLICIT in the J4 grant; nothing outside this unit is touched):
#   same InvocationID throughout                     -> normal; any stop targets this unit only.
#   new InvocationID AND NRestarts rose since start  -> restart-born invocation of the unit THIS job started: stop the unit
#                                                       (by name; it is the only unit touched) and read back inactive.
#   new InvocationID, NRestarts unchanged            -> started by someone else: NOT stopped; exit 2; calm-crab decides.
mem start
shas_ok; need J2_INSTALLED "$J2_SHA" J1B4_GREEN
[ -e $MK/J4_UP ] && die2 "J4 already ran"
UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; LAB=/opt/gunbc-lab/codex-provider-feedback-v0
sha256sum -c --quiet $WD/INSTALLED_SHAS >/dev/null 2>&1 || die2 "installed unit or bin/gunbc changed since J2"
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path; do
  [ "$(systemctl show $u -p ActiveState --value)" = active ] && [ "$(systemctl show $u -p UnitFileState --value)" = enabled ] || die2 "$u not active+enabled (J3 not done)"; done
[ "$(systemctl show $UNIT -p LoadState --value)" = loaded ] && [ "$(systemctl show $UNIT -p ActiveState --value)" = inactive ] || die2 "unit not loaded/inactive before start"
ss -Hltn 'sport = :18081' | grep -q . && die2 ":18081 already has a listener before start"
NR0="$(systemctl show $UNIT -p NRestarts --value)"
sudo -n /usr/bin/systemctl start $UNIT; r=$?; INV="$(systemctl show $UNIT -p InvocationID --value)"; echo "START_RC=$r INVOCATION=$INV NRESTARTS0=$NR0"
systemctl show $UNIT -p LoadState -p ActiveState -p SubState -p Result -p ExecMainPID -p ExecMainStatus -p NRestarts -p ControlGroup -p InvocationID -p User -p CapabilityBoundingSet -p NoNewPrivileges -p RestrictSUIDSGID | tee $R/serve-props.txt
stop_unit() { sudo -n /usr/bin/systemctl stop $UNIT; systemctl show $UNIT -p ActiveState -p Result -p InvocationID -p NRestarts | tr '\n' ' '; echo; }
settle_fail() { local cur nr; cur="$(systemctl show $UNIT -p InvocationID --value)"; nr="$(systemctl show $UNIT -p NRestarts --value)"
  if [ "$cur" = "$INV" ] || [ -z "$cur" ]; then stop_unit; die2 "$1; stopped this invocation (lab down)"
  elif [ "${nr:-0}" -gt "${NR0:-0}" ]; then echo "RESTART_BORN invocation=$cur nrestarts=$nr"; stop_unit; die2 "$1; stopped the restart-born invocation of this job's start (lab down)"
  else die2 "$1; invocation $cur was NOT started by this job (NRestarts unchanged); not stopped"; fi; }
[ $r = 0 ] && [ -n "$INV" ] || settle_fail "start rc=$r or no InvocationID"
LP=""
for i in $(seq 1 30); do
  [ "$(systemctl show $UNIT -p InvocationID --value)" = "$INV" ] || settle_fail "INVOCATION_CHANGED"
  [ "$(systemctl show $UNIT -p ActiveState --value)" = active ] || settle_fail "STARTUP_ERROR $(systemctl show $UNIT -p Result -p ExecMainStatus -p NRestarts | tr '\n' ' ')"
  LP="$(ss -Hltnp 'sport = :18081' 2>/dev/null | grep -oE 'pid=[0-9]+' | head -1 | cut -d= -f2)"; [ -n "$LP" ] && break; sleep 60
done
CG2="$(systemctl show $UNIT -p ControlGroup --value)"
[ -n "$LP" ] && [ -n "$CG2" ] && [ "$(systemctl show $UNIT -p InvocationID --value)" = "$INV" ] && grep -qx "$LP" "/sys/fs/cgroup$CG2/cgroup.procs" || settle_fail "NOT_OURS_OR_NOT_LISTENING pid=${LP:-none} cgroup=${CG2:-none}"
echo "LISTENING pid=$LP cgroup=$CG2 invocation=$INV"
mark J4_UP J2_INSTALLED; echo DONE; exit 0
