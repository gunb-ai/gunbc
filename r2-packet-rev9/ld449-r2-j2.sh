#!/bin/bash
# r2 restoration packet REVISION 9, job J2 (launcher job ld449-r2-j2). Author loyal-deer-449. NOT a grant request; byte review first.
# FRAME: #15967 packet_G.sh v4 (sha c781341a...), itself #15695 v4a (hardened frame #15931 v12 lineage): die2, scope gate, cgroup
#   bound gates, OOM parser/judge, verdict classifier, completion parser (REAL line verbatim, sha-checked), rc contract,
#   tree_clean, out_reusable, hash_of and their fixtures are copied BYTE FOR BYTE; only the job name in the scope gate and
#   its fixture, and the booked cgroup values, differ. Exit contract: 0 held, 1 deviation from a pre-registered verdict,
#   2 no observation (every refusal, timeout 124, signal >128, OOM, incoherent counter, survivors).
# LAUNCH: calmcrab-udrv.sh ld449-r2-j2 6000 16 14 4096 3200% <STAGED> <SHA> OUT <SERVE_UNIT_SHA>
# PINS: r2 head 26fe4b8f350a (tree b0ee022a6bf8) = merged integration abcbf27e4cc3 (#15990) + #15919 own commits + r2 1/2..5/5;
#   RED = r2 1/2 154afcd00667. One external process at a time; survivors in this job's cgroup are killed and verified gone.
# JOIN LAW ENFORCED: exactly one bare PASS|FAIL per selected name; no outcome line naming a function outside the roster; no
#   keyword other than PASS|FAIL for a roster name; batch rc cross-checked against that validated set. Jobs that run claim_batch
#   self-test this through the REAL run_entry (10 fixture logs) before anything else, and refuse if any case gives the wrong exit.
set -uo pipefail
JOB=ld449-r2-j2
UNIT="$JOB.scope"
JOB_RT=6000
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
T0=$(( $(date +%s) - SECONDS )); JOB_END=$(( T0 + JOB_RT - 60 )); RESERVE=${RESERVE:-300}
tcap() { local r=$(( JOB_END - RESERVE - $(date +%s) )); [ $r -lt 1 ] && { echo 0; return; }; [ $r -lt $1 ] && echo $r || echo $1; }
# budget VAR N what: sets VAR to tcap N in the CURRENT shell, or exits the job (die2, which J2 redefines to finalize) when the
# budget is exhausted. Never call it inside $(...): exhaustion must end the job, not a subshell.
budget() { local c; c=$(tcap $2); [ "$c" -ge 1 ] || die2 "time budget exhausted before $3"; printf -v "$1" '%s' "$c"; }
HEAD_EXPECT=26fe4b8f350a99bd57a9bc342d4625b0cb96804b; TREE_EXPECT=b0ee022a6bf8f9f9f35d496174afd681300c9847
RED_REV=154afcd00667d3c84609f551c45e0ab4f5b83bad; NO15175=f38a00cb75d85e3a7879c27f7288c2a8bba993ae
WD=/home/briansrls/runs/ld449-r2work; WT=$WD/wt; WR=$WD/wt-red; TG=$WD/target; MK=$WD/markers
DECLARED_SERVE_BINARY=/home/briansrls/runs/ld449-r2work/target/release/gunbc
OFF=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_record
SPW=dag/test/claim/roadmap/session_placement_wet_witness.dag
export PATH=$HOME/.cargo/bin:$PATH
shas_ok() { [ -f $WD/SHAS ] && timeout 120 sha256sum -c --quiet $WD/SHAS >/dev/null 2>&1 || die2 "binary shas changed or missing"; }
shas_id() { hash_of $WD/SHAS; }
# Markers (B1). A marker is EVIDENCE for the next precheck, never authority (each job is its own grant; calm-crab verifies the
# launcher receipts -- frozen.sha256, drv.log, printed SHAS -- independently). It is one field per line:
#   HEAD, TREE, SCRIPT (the writing job's own sha, taken before cd), SHAS (binaries), OUT (the writing job's output dir),
#   PRIOR <name> <sha256 of that marker file>, and one RECEIPT <sha256> <absolute path> per retained receipt.
# need() re-reads reality before trusting a marker: the worktree's HEAD and tree (wt_ok), the binaries (shas_ok), and then the
# WHOLE chain back to J1: every marker's fields, every RECEIPT re-hashed, every PRIOR link re-hashed. Stated limit: a process
# running as the same user can still fabricate consistent files; the markers raise the cost of that, they do not prevent it.
mark() { local name=$1 prior=$2 ps=none f h t body; shift 2; [ "$prior" != none ] && { ps="$(hash_of $MK/$prior)" || die2 "cannot hash prior marker $prior"; }
  mkdir -p $MK || die2 "mkdir markers"; t=$MK/.$name.tmp
  ign_snapshot $R/ignored-end.txt || die2 "ignored-set snapshot"
  body="$(printf 'HEAD %s\nTREE %s\nSCRIPT %s\nSHAS %s\nOUT %s\nPRIOR %s %s' $HEAD_EXPECT $TREE_EXPECT $SELF_SHA "$(shas_id)" "$R" $prior $ps)" || die2 "marker body $name"
  for f in "$@" $R/ignored-end.txt; do h="$(hash_of "$f")" || die2 "marker $name: cannot hash receipt $f"; body="$body"$'\n'"RECEIPT $h $f"; done
  printf '%s\n' "$body" > $t || die2 "marker write $name"; [ "$(cat $t)" = "$body" ] || die2 "marker readback $name"
  mv -f $t $MK/$name || die2 "marker rename $name"; cat $MK/$name; }
mfield() { awk -v k="$2" '$1==k{print substr($0,length(k)+2)}' "$1"; }
EXPECTED_RECEIPTS="J1_GREEN=SHAS,build.log,claims-G-off.log,claims-R-off.log,oom-raw.log,ignored-end.txt
J1R_GREEN=serve.unit,belt.txt,review.txt,RENDER_SHAS,preflight.log,oom-raw.log,ignored-end.txt
J1B1_GREEN=claims-G-session_placement_wet_witness.log,claims-G-roadmap_belt_actuate_witness.log,claims-G-roadmap_dashboard_instance_witness.log,oom-raw.log,ignored-end.txt
J1B2_GREEN=claims-G-attempt_system_unit_launch_witness.log,claims-G-attempt_state_move_apply_wet_witness.log,claims-G-lab_apply_operator_grant_witness.log,oom-raw.log,ignored-end.txt
J1B3_GREEN=claims-G-worker_git_subtree_apply_spawn_wet_witness.log,claims-G-dispatch_worker_confinement_witness.log,claims-G-operator_endpoint_witness.log,claims-G-agent_door_witness.log,oom-raw.log,ignored-end.txt
J1B4_GREEN=claims-G-request_authentication_witness.log,claims-G-roadmap_event_log_witness.log,claims-G-harness_validate_worktree_witness.log,claims-G-dispatch_worker_traversal_witness.log,oom-raw.log,ignored-end.txt
J2_INSTALLED=INSTALLED_SHAS,identity.txt,serve.unit,admit.log,apply.log,units-before.txt,units-after.txt,paths-before.txt,paths-after-reload.txt,oom-raw.log,ignored-end.txt"
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
  exp="$(printf '%s\n' "$EXPECTED_RECEIPTS" | awk -F= -v k="$1" '$1==k{print $2}' | tr ',' '\n' | sort)"; [ -n "$exp" ] || die2 "marker $1: no expected receipt set"
  got="$(grep '^RECEIPT ' $m | awk '{print $3}' | xargs -n1 basename | sort)"; [ "$got" = "$exp" ] || die2 "marker $1: receipt set is not exactly the expected set"
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
wt_ok() { local o; [ "$(timeout 60 git -C $WT rev-parse HEAD)" = $HEAD_EXPECT ] && [ "$(timeout 60 git -C $WT rev-parse 'HEAD^{tree}')" = $TREE_EXPECT ] || die2 "worktree is not at the pin"
  o="$(timeout 120 git -C $WT status --porcelain --untracked-files=all)" || die2 "git status failed"; [ -z "$o" ] || die2 "worktree has tracked or untracked changes"; }
# IGNORED PATHS: .gitignore at the pin ignores /.gunbc/ and the two roster.dag paths among others, which runs may write. So the
# ignored set is not required empty after runs: J1 requires it EMPTY on the fresh clone; every job records it at its end as a
# marker receipt; the next job requires it UNCHANGED at its start (tampering between jobs refuses).
ign_snapshot() { local o; o="$(timeout 120 git -C $WT status --porcelain --ignored --untracked-files=all)" || return 1; { printf '%s\n' "$o" | grep '^!! ' || true; } | sort > "$1"; }   # an empty set is valid (grep's no-match rc is not a failure)
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
  local TB=""; [ "$WET" = 1 ] && TB="$(tmp_list)"
  snap "run $LEG $T"
  local CC; CC=$(tcap "$CAP"); [ "$CC" -ge 31 ] || die2 "time budget exhausted before $LEG $T"; CC=$((CC-30))
  ( cd "$DIR" && timeout --kill-after=30 "$CC" "$TG/release/claim_batch" $A --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$F" --functions "$FUNCS" ) > "$L" 2>&1
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
  [ "$(grep -aciE '^[[:space:]]*(pass|fail|unobserved|panicked|error|skipped)([[:space:]]|$)' "$L")" = "$(echo "$FUNCS" | tr ',' '\n' | grep -c .)" ] || die2 "leg $LEG $T: a near-miss outcome line (case, leading space or tab) or an extra outcome"
  [ "$(grep -acE '^[A-Z]+ ' "$L")" = "$(echo "$FUNCS" | tr ',' '\n' | grep -c .)" ] || die2 "leg $LEG $T: outcome line count (any '^[A-Z]+ ' line, as verdict() reads them) != roster size"
  [ "$RC" = "$VE" ] || die2 "leg $LEG $T: claim_batch returned $RC, its own verdict lines say $VE"
  echo "leg $LEG $T ok ($(echo "$FUNCS" | tr ',' '\n' | grep -c .) fns)"; }
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
subtree_procs_into() { local f; local -a one; local -n _acc=$1; _acc=(); for f in "$CG"/cgroup.procs "$CG"/**/cgroup.procs; do mapfile -t one < "$f" || return 1; _acc+=("${one[@]}"); done; return 0; }
subtree_procs_into B0 || die2 "REFUSED_SETUP: cannot read this job's cgroup subtree"
# survivors: sets SURV to the pids in this job's cgroup subtree other than this shell and B0; returns 1 if the read failed.
# Runs in the CURRENT shell (callers never use $(survivors)), so there is no reader subshell to exclude.
survivors() { local -a now; local p b keep; SURV=""; subtree_procs_into now || return 1
  for p in "${now[@]}"; do keep=0; [ "$p" = "$$" ] && keep=1
    for b in "${B0[@]}"; do [ "$p" = "$b" ] && keep=1; done; [ $keep = 0 ] && SURV="$SURV $p"; done; return 0; }
contain() { local i; survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0
  echo "SURVIVORS after $1:$SURV"; kill -TERM $SURV 2>/dev/null
  for i in $(seq 1 30); do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; sleep 1; done; kill -KILL $SURV 2>/dev/null
  for i in $(seq 1 10); do survivors || die2 "cannot read own cgroup after $1"; [ -z "$SURV" ] && return 0; sleep 1; done; die2 "survivors remain in this job's cgroup after $1:$SURV"; }
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
  mkdir -p "$d/w/dag" "$d/t/release" && printf 'test fn x() -> Bool { true }\ntest fn y() -> Bool { true }\n' > "$d/w/dag/e.dag" || { rm -rf -- "$d"; return 1; }
  for c in "foreign-pass|2|PASS x\nPASS y\nPASS z|0" "foreign-fail|2|PASS x\nPASS y\nFAIL z|1" "foreign-skipped|2|PASS x\nPASS y\nSKIPPED z|0" \
           "duplicate-pass|2|PASS x\nPASS x\nPASS y|0" "reasoned-fail|2|PASS x\nFAIL y (runtime error [c]: m)|1" "unobserved|2|PASS x\nUNOBSERVED y (r)|2" \
           "missing-name|2|PASS x|0" "mixed-rc-match|0|PASS x\nFAIL y|1" "mixed-rc-zero|2|PASS x\nFAIL y|0" "all-pass|0|PASS x\nPASS y|0"; do
    IFS='|' read -r name want lines rc <<< "$c"
    printf '#!/bin/bash\nprintf "%s\\n"\necho '"'"'[cost-partition] {"verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":1}}'"'"'\necho "[process-partition] total_ms=5 a_ms=1"\nexit %s\n' "$lines" "$rc" > "$d/t/release/claim_batch" && chmod +x "$d/t/release/claim_batch" || { rm -rf -- "$d"; return 1; }
    ( TG="$d/t"; R="$d"; EXPECT=(); declare -A EXPECT OBS; EXPECT["F.$name.x"]=PASS; EXPECT["F.$name.y"]=$([ $name = mixed-rc-match ] || [ $name = mixed-rc-zero ] && echo FAIL || echo PASS); DEVIATIONS=0
      run_entry F "$name" "$d/w" dag/e.dag x,y 0 60 > "$d/$name.out" 2>&1 ) ; got=$?
    [ "$got" = "$want" ] || { echo "join fixture $name: exit $got, required $want"; cat "$d/$name.out" | tail -3; rm -rf -- "$d"; return 1; }
  done; rm -rf -- "$d"; echo "join-law fixtures: all held (10 cases)"; return 0; }
# ==== end r2 common ====
J1B4_SHA=793f62af80347ce857201597a05e36a7c1c530136b53087052ff8629053a40ab
REVIEWED_SCRIPTS="J1_GREEN:72b0860435a229c3c2abadafd9c9fb75cb10f36d06cdc3f701619ff29d9306b6 J1R_GREEN:f2b6a927947fa02fbce64d8337b752579ea253906c5990a8344648430763ef26 J1B1_GREEN:bc1051023fecd327fdff2960c985a1b7c4490498f2edf64b6842f9f90931802a J1B2_GREEN:7c2fd51bf28daec84362bb78b08fa289e711a0f6e0827a0da3c5cb67871b4628 J1B3_GREEN:6a29986dc6ee08d25e983b7313824489f21a675a4a2ac3a6ca9ea3a99f59586a J1B4_GREEN:793f62af80347ce857201597a05e36a7c1c530136b53087052ff8629053a40ab"
# ==== J2: LAB MUTATION. admit -> dashboard apply -> identity -> install the REVIEWED rendered serve unit -> daemon-reload. NO start. ====
# Arg 2 = sha256 of $WD/render/serve.unit as reviewed and authorized (a separate act after J1R).
# BUDGET (RT 6000): admit cap 900 + apply cap 2400 + 2x30 kill-after = 3360 s; INVENTORY ALLOWANCE 1200 s (4 unit inventories +
#   2 provider listings, each capped at 300 s by `timeout`); systemctl/sudo calls capped at 30-120 s each; ~1000 s margin.
# EFFECT SCOPE (exact; see APPLY-EFFECTS.md): the apply's root acts touch only gunbc-srv1-lab-codex-feedback-v0.service's /etc
#   fragment (stop, mv, daemon-reload, and only when that unit is loaded -- it is not-found here, checked below); this job adds
#   one install into /etc/systemd/system and one daemon-reload. The unit-config inventory therefore covers:
#   (1) /etc/systemd/system RECURSIVELY: every entry by type -- regular file (mode, owner, size, mtime, inode, sha256 when
#       readable; a root-only file is identified by its stat signature, never by a constant), symlink (enablement *.wants/
#       *.requires links and masks -> /dev/null, with target), directory (mode, owner);
#   (2) /run/systemd/system restricted to names matching gunbc-srv1-lab-codex-feedback-v0* (runtime overrides of this
#       instance's units, incl. *.d drop-ins and *.wants links);
#   (3) /home/briansrls/.config/systemd/user RECURSIVELY (the operator's user units; static).
#   EXCLUDED, with reason: /usr/lib/systemd/system and /lib/systemd/system (package-managed; no reachable effect writes there);
#   the rest of /run/systemd (transient units of live sessions/scopes; volatile by design; no reachable effect writes there);
#   generator output (regenerated by daemon-reload by design). The diff is taken BEFORE daemon-reload (after install) and
#   again after it; any change other than the one installed serve unit refuses -- before the reload makes it live.
#   No host-wide read beyond this scope is made.
# PROVIDER STATE (K5; APPLY-EFFECTS U1/U2): CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap, OUTSIDE the instance root. The
#   apply's ensure-owned-directory row may re-group/re-mode it to 02770 briansrls:gunbc-attempt, and its verify-provider legs
#   run `/usr/bin/env CODEX_HOME=<root> /opt/gunbc/provider-runtime/codex/current/node_modules/@openai/codex-linux-arm64/vendor/
#   aarch64-unknown-linux-musl/bin/codex --version` and `... login status`, which may write under it (unresolved in source).
#   So the directory's stat and every file's (sha256, size, mtime, path) are listed before and after the apply and the DIFF is
#   RECORDED, not refused: that is the explicit, bounded write surface of those legs. File CONTENTS are never printed.
# PATH (APPLY-EFFECTS U4): the apply looks up env, git, find, chgrp, chmod, mktemp, ln, rm and dd on PATH. This job runs admit
#   and apply with PATH=/usr/bin:/bin and records `command -v` for each, so those names resolve to fixed locations.
# PRINCIPAL: the launcher runs this job as briansrls; checked below (id -un).
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
# FINALIZATION (B4): ONE path, an EXIT trap armed at the first mutation. On any post-mutation exit it (a) writes
#   $WD/J2_PARTIAL (step, time, readings) so a rerun of J2 refuses until calm-crab-645 decides; (b) if the serve unit file is
#   present at $DEST with a sha other than the reviewed one, or a staged temp remains, removes it (sudo -n rm -f --, the only
#   finalization privilege) and reloads; (c) does NOT restore $R/pre-bin-gunbc: those bytes are 5592b17b code that cannot read
#   the moved attempt state, so restoring them is not safe (they are retained for calm-crab's decision). Install is atomic:
#   copy to $DEST.ld449-staging (not a unit name; systemd ignores it), then `mv -T` onto $DEST, so a half-written unit file
#   can never be loaded.
# FAILURE DISPOSITIONS (the serve is down throughout; nothing here starts it). Every refusal AFTER the first mutation is
#   labelled LAB_STATE_CHANGED step=<step>, runs the finalization, and exits 2:
#   pre-admit refusal              -> no change (label PRE_MUTATION).
#   step=admit                     -> admit has no rollback: the admitted ref, source checkout and FETCH_HEAD may have moved
#                                     and roster.dag files may be deleted; J2_PARTIAL records it; lab down; calm-crab decides.
#   step=apply                     -> legs idempotent; bin/gunbc may be the witnessed copy or the old one ($R/pre-bin-gunbc
#                                     holds the old bytes); old unit fragment already retired; lab down.
#   step=apply-trigger-units       -> the 5 trigger units' state changed during apply; calm-crab-645 reads and decides; lab down.
#   step=identity                  -> applied, no unit installed; lab down.
#   step=install                   -> no file, or a file whose sha != reviewed; calm-crab-645 removes it under its own act; lab down.
#   step=reload / step=post-reload -> file present; loaded state or inventory not as expected; calm-crab-645 decides; lab down.
mem start
die_lab() { STEP="$1"; die2 "step=$1: $2"; }   # die2 is redefined at the first mutation to print LAB_STATE_CHANGED step=$STEP
SERVE_UNIT_SHA="$EXTRA_ARG"; [[ $SERVE_UNIT_SHA =~ ^[0-9a-f]{64}$ ]] || die2 "PRE_MUTATION: arg 2 must be the reviewed serve unit sha256"
prior_scope_gone ld449-r2-j1b4; need J1B4_GREEN "$J1B4_SHA" J1B3_GREEN
[ "$TG/release/gunbc" = "$DECLARED_SERVE_BINARY" ] || die2 "PRE_MUTATION: declared serve binary path"
# SSH RESOLUTION PRECHECK (non-secret, no connection): `ssh -G srv1` prints the configuration ssh WOULD use. The production
#   probes need it to resolve to a loopback host as user briansrls (srv1-live lives on this host); anything else refuses
#   before any probe. The resolved hostname, user, port, userknownhostsfile, stricthostkeychecking and batchmode are recorded.
sshg="$(timeout 30 ssh -G srv1 2>&1)" || die2 "PRE_MUTATION: ssh -G srv1 failed"
printf '%s\n' "$sshg" | grep -E '^(hostname|user|port|userknownhostsfile|stricthostkeychecking|batchmode) ' > $R/ssh-resolution.txt || die2 "PRE_MUTATION: ssh -G output unreadable"; cat $R/ssh-resolution.txt
case "$(printf '%s\n' "$sshg" | awk '$1=="hostname"{print $2}')" in localhost|127.0.0.1|::1) ;; *) die2 "PRE_MUTATION: ssh srv1 does not resolve to loopback";; esac
[ "$(printf '%s\n' "$sshg" | awk '$1=="user"{print $2}')" = briansrls ] || die2 "PRE_MUTATION: ssh srv1 user is not briansrls"
# READINESS (non-secret, no connection): the apply's production probes run plain `ssh srv1 -- ...` with no -o options, so their
# EFFECTIVE options are exactly what `ssh -G srv1` reports. Require batchmode yes: ssh then never prompts (no password,
# passphrase or host-key prompt) and fails instead, so an authentication problem is a bounded failure, not a stall. A
# known_hosts match does NOT establish non-interactive authentication; it is RECORDED only. known_hosts shas are recorded
# before and after the apply so an append is visible; the ssh/curl probes are recorded through $R/apply.log.
SSH_HOST="$(printf '%s\n' "$sshg" | awk '$1=="hostname"{print $2}')"; KH="$(printf '%s\n' "$sshg" | awk '$1=="userknownhostsfile"{print $2}')"
[ "$(printf '%s\n' "$sshg" | awk '$1=="batchmode"{print $2}')" = yes ] || die2 "PRE_MUTATION: ssh srv1's effective BatchMode is not yes (the production probes could prompt)"
khok=no; for f in $KH; do f="${f/#\~/$HOME}"; [ -r "$f" ] && ssh-keygen -F "$SSH_HOST" -f "$f" >/dev/null 2>&1 && khok=yes; done; echo "known_hosts entry for $SSH_HOST: $khok (recorded only)"
for f in $KH; do f="${f/#\~/$HOME}"; printf '%s %s\n' "$(hash_of "$f" 2>/dev/null || echo ABSENT)" "$f"; done > $R/known-hosts-before.txt
{ [ -e $MK/J2_INSTALLED ] || [ -e $WD/J2_PARTIAL ]; } && die2 "PRE_MUTATION: J2 already ran (installed or partial); calm-crab decides"
[ "$(id -un)" = briansrls ] || die2 "PRE_MUTATION: not running as briansrls"
for n in env git find chgrp chmod mktemp ln rm dd; do printf '%s=%s ' $n "$(PATH=/usr/bin:/bin command -v $n)"; done > $R/path-resolution.txt; cat $R/path-resolution.txt; echo
LAB=/opt/gunbc-lab/codex-provider-feedback-v0; UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; STAGE=$DEST.ld449-staging; GB=$TG/release/gunbc
PROVIDER_STATE_ROOT=/home/briansrls/.codex@gunbc-roadmap
# B7: install from a copy in OUT, hashed at copy time, not from the user-writable work dir.
cp -- $WD/render/serve.unit $R/serve.unit || die2 "PRE_MUTATION: copy reviewed render into OUT"
[ "$(hash_of $R/serve.unit)" = "$SERVE_UNIT_SHA" ] || die2 "PRE_MUTATION: the render is not the reviewed text"
lu() { local o; o="$(timeout 60 "$@" list-units --no-legend --plain --state=active,activating --type=service "${UNITPATS[@]}")" || return 1; printf '%s' "$o" | grep -v -E 'belt|exit|drain' | grep -c . ; return 0; }
UNITPATS=('gunbc-attempt-*'); SYS="$(lu systemctl)" || die2 "PRE_MUTATION: list-units (system) failed"
UNITPATS=('gunbc-obligation-attempt-*' 'gunbc-audit-*' 'gunbc-compute-*'); USR="$(lu systemctl --user)" || die2 "PRE_MUTATION: list-units (user) failed"
echo "LIVE system_attempt=$SYS user=$USR"; [ "$SYS" = 0 ] && [ "$USR" = 0 ] || die2 "PRE_MUTATION: live attempt/review/compute units present"
{ [ -e $DEST ] || [ -L $DEST ] || [ -e $STAGE ]; } && die2 "PRE_MUTATION: $DEST or its staging name exists"
v="$(uprop $UNIT LoadState)" || die2 "PRE_MUTATION: read LoadState"; [ "$v" = not-found ] || die2 "PRE_MUTATION: serve unit is [$v] before apply"
[ -d $LAB/receipts ] && [ -w $LAB/receipts ] || die2 "PRE_MUTATION: $LAB/receipts missing or not writable (admit would mutate git, then fail its receipt)"
cp -p $LAB/bin/gunbc $R/pre-bin-gunbc || die2 "PRE_MUTATION: preserve current bin/gunbc"
inv_dir() { local root=$1 filt=${2:-} f t h m; [ -d "$root" ] || { echo "ABSENT $root"; return 0; }
  budget BUD1 120 inventory; timeout $BUD1 find "$root" -xdev -print0 > $R/.inv.find 2> $R/.inv.err || return 1; [ -s $R/.inv.err ] && return 1
  sort -z $R/.inv.find | while IFS= read -r -d '' f; do
    [ -n "$filt" ] && case "${f#$root/}" in $filt|*/$filt) ;; *) [ "$f" = "$root" ] || continue;; esac
    m="$(stat -c '%a %U:%G %s %Y %i' -- "$f")" || exit 1
    if [ -L "$f" ]; then t="$(readlink -- "$f")" || exit 1; printf 'L %s %s -> %s\n' "$m" "$f" "$t"
    elif [ -d "$f" ]; then m="$(stat -c '%a %U:%G' -- "$f")" || exit 1; printf 'D %s %s\n' "$m" "$f"
    elif [ -f "$f" ]; then if [ -r "$f" ]; then h="$(hash_of "$f")" || exit 1; else h="unreadable-statsig"; fi; printf 'F %s %s %s\n' "$m" "$h" "$f"
    else printf 'X %s %s\n' "$m" "$f"; fi; done; }
# FIXTURE (before any mutation): the install-plus-inventory comparison, end to end, with the exact inv_dir helper, in a
# private temp tree. (a) Adding only the permitted file is NOT a diff once that file's line is excluded (the parent dir's
# bookkeeping is not recorded). (b) An unexpected new entry IS a diff. (c) A mode change on an existing file IS a diff.
# (d) An owner/mode change on a directory IS a diff. (e) A new symlink (an enable) IS a diff.
inv_fixture() { local d e r=0 o; d="$(mktemp -d)" || return 1
  mkdir "$d/u" "$d/u/x.wants" && printf a > "$d/u/a.service" && ln -s ../a.service "$d/u/x.wants/a.service" || { rm -rf -- "$d"; return 1; }
  inv_dir "$d/u" > "$d/b" || r=1
  sleep 1; printf n > "$d/u/new.service"; inv_dir "$d/u" > "$d/c" || r=1
  diff <(grep -v " $d/u/new.service\$" "$d/b") <(grep -v " $d/u/new.service\$" "$d/c") > /dev/null || { echo "fixture (a) refused the permitted install"; r=1; }
  printf z > "$d/u/stray.conf"; inv_dir "$d/u" > "$d/e1" || r=1; diff <(grep -v " $d/u/new.service\$" "$d/b") <(grep -v " $d/u/new.service\$" "$d/e1") > /dev/null && { echo "fixture (b) missed an unexpected entry"; r=1; }; rm -f -- "$d/u/stray.conf"
  chmod 600 "$d/u/a.service"; inv_dir "$d/u" > "$d/e2" || r=1; diff <(grep -v " $d/u/new.service\$" "$d/b") <(grep -v " $d/u/new.service\$" "$d/e2") > /dev/null && { echo "fixture (c) missed a file mode change"; r=1; }; chmod 644 "$d/u/a.service"
  chmod 700 "$d/u/x.wants"; inv_dir "$d/u" > "$d/e3" || r=1; diff <(grep -v " $d/u/new.service\$" "$d/b") <(grep -v " $d/u/new.service\$" "$d/e3") > /dev/null && { echo "fixture (d) missed a directory mode change"; r=1; }; chmod 755 "$d/u/x.wants"
  ln -s ../new.service "$d/u/x.wants/new.service"; inv_dir "$d/u" > "$d/e4" || r=1; diff <(grep -v " $d/u/new.service\$" "$d/b") <(grep -v " $d/u/new.service\$" "$d/e4") > /dev/null && { echo "fixture (e) missed a new enable symlink"; r=1; }
  rm -rf -- "$d"; return $r; }
unit_inv() { inv_dir /etc/systemd/system && inv_dir /etc/systemd/user && inv_dir /run/systemd/system 'gunbc-srv1-lab-codex-feedback-v0*' && inv_dir /home/briansrls/.config/systemd/user; }
# not_dest: drop ONLY the line whose PATH field is exactly $DEST (F/D/X lines: last field; L lines: the field before "->").
# A symlink whose TARGET is $DEST (an enablement) is kept and therefore diffs.
not_dest() { awk -v d="$DEST" '($1=="L" && $(NF-2)==d) {next} ($1!="L" && $NF==d) {next} {print}' "$1"; }
PATHS="gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path gunbc-srv1-lab-codex-feedback-v0-publication.path gunbc-srv1-lab-codex-feedback-v0-belt-credential-recovery.timer"
units_state() { local u p v; for u in $PATHS; do printf '%s' "$u"
    for p in Id LoadState UnitFileState Triggers FragmentPath; do v="$(uprop $u $p)" || return 1; printf ' %s=%s' $p "$v"; done
    case "$u" in *-publication.path|*-belt-credential-recovery.timer) ;; *) v="$(uprop $u ActiveState)" || return 1; printf ' ActiveState=%s' "$v";; esac; echo; done; }
# (config only: publication.path and the credential-recovery timer may fire during apply, so their volatile Active/Sub states
#  and the triggered services' states are NOT compared; the 3 calm-crab-disabled paths keep ActiveState, which must stay inactive.)
prov_inv() { [ -d "$PROVIDER_STATE_ROOT" ] || { echo "ABSENT $PROVIDER_STATE_ROOT"; return 0; }; stat -c "DIR %a %U:%G %n" "$PROVIDER_STATE_ROOT" || return 1
  budget BUD2 120 provider-inventory; timeout $BUD2 find "$PROVIDER_STATE_ROOT" -xdev -type f -printf '%s %T@ %p\n' > $R/.prov.find 2>/dev/null || return 1
  sort -k3 $R/.prov.find | while read -r sz mt f; do if [ -r "$f" ]; then h="$(hash_of "$f")" || exit 1; else h=unreadable-statsig; fi; printf '%s %s %s %s\n' "$h" "$sz" "$mt" "$f"; done; }
inv_fixture || die2 "PRE_MUTATION: install/inventory fixture failed"; echo "inventory fixture: held"
unit_inv > $R/units-before.txt || die2 "PRE_MUTATION: unit inventory failed"; echo "inventory: $(wc -l < $R/units-before.txt) entries"
units_state > $R/paths-before.txt || die2 "PRE_MUTATION: trigger unit state read failed"; cat $R/paths-before.txt
prov_inv > $R/provider-before.txt || die2 "PRE_MUTATION: provider inventory failed"
{ sha256sum $LAB/bin/gunbc $LAB/bin/gunbc.prev; git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted HEAD; ls $LAB/attempt-state/attempt-identities | sha256sum; } > $R/pre-snapshot.txt 2>&1; cat $R/pre-snapshot.txt
# ---- ROOT AUTHORITY (silent-bee B3; keen-otter): `sudo -n -l` proves only what the live sudoers PERMITS, not operator
# AUTHORIZATION; the install, mv -T, daemon-reload and the two rm acts need their own explicit scope in J2's grant. every root argv this job (and its finalization) may run, checked with `sudo -n -l
# <exact argv>` BEFORE admit. If the live sudoers is the modelled 5-operation lab grant (stop, the two mv, daemon-reload --
# lab_apply_operator_grant_witness) these install/mv -T/rm -f acts are NOT in it, and J2 refuses here, before anything is
# mutated; they then need an explicit grant (or are performed as calm-crab keeper acts). The grant each act relies on is
# recorded from `sudo -n -l` output in $R/sudo-authority.txt.
SUDO_ARGV=( "/usr/bin/install -m 0644 -o root -g root -- $R/serve.unit $STAGE" "/usr/bin/mv -T -- $STAGE $DEST" "/usr/bin/systemctl daemon-reload" "/usr/bin/rm -f -- $STAGE" )
# `sudo -n -l <argv>` exits 0 even for a rule that REQUIRES a password, so each act must also appear under a NOPASSWD rule in
# the full listing (`NOPASSWD: ALL`, or a NOPASSWD rule naming its program). This parse is textual and is a precheck only.
SL="$(timeout 30 sudo -n -l 2>&1)" || die2 "PRE_MUTATION: sudo -n -l (full listing) failed"; printf '%s\n' "$SL" > $R/sudo-listing.txt
: > $R/sudo-authority.txt
for a in "${SUDO_ARGV[@]}"; do o="$(timeout 30 sudo -n -l $a 2>&1)"; rc=$?; prog="${a%% *}"; printf 'argv=[%s] rc=%s permitted_as=[%s]\n' "$a" $rc "$o" >> $R/sudo-authority.txt
  [ $rc = 0 ] || die2 "PRE_MUTATION: root act not permitted by live sudoers (sudo -n -l rc=$rc): $a"
  printf '%s\n' "$SL" | grep -E 'NOPASSWD:' | grep -qE "NOPASSWD:[[:space:]]*ALL|$prog" || die2 "PRE_MUTATION: $prog is not under a NOPASSWD rule (a password-requiring rule would hang or fail): $a"; done; cat $R/sudo-authority.txt
# The three .path units calm-crab disabled must STILL be inactive+disabled before apply (they are re-enabled only by J3).
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path; do
  a="$(uprop $u ActiveState)" || die2 "PRE_MUTATION: read $u"; e="$(uprop $u UnitFileState)" || die2 "PRE_MUTATION: read $u"
  [ "$a" = inactive ] && [ "$e" = disabled ] || die2 "PRE_MUTATION: $u is [$a/$e]; must be inactive/disabled before apply"; done
# ---- finalization: armed at the first mutation ----
STEP=pre; finalize() { local rc=$? r; [ "$STEP" = done ] && return; [ "$STEP" = pre ] && return
  # Reports EVERY action's rc/readback, labels anything unverified UNRESOLVED_CLEANUP, preserves the primary failure, and
  # NEVER runs daemon-reload (a reload here would make undiffed on-disk changes live). Only the staging name is removed: it is
  # not a unit name, so systemd never loads it.
  echo "FINALIZE begin step=$STEP primary_rc=$rc"
  if printf 'J2_PARTIAL step=%s rc=%s at=%s out=%s\n' "$STEP" "$rc" "$(date -u +%FT%TZ)" "$R" >> $WD/J2_PARTIAL; then echo "FINALIZE J2_PARTIAL updated"; else echo "UNRESOLVED_CLEANUP: J2_PARTIAL update failed (it was written before admit, so a rerun is still blocked)"; fi
  if [ -e $STAGE ]; then timeout 60 sudo -n /usr/bin/rm -f -- $STAGE; r=$?; if [ $r = 0 ] && [ ! -e $STAGE ]; then echo "FINALIZE removed staging"; else echo "UNRESOLVED_CLEANUP: staging removal rc=$r, present=$([ -e $STAGE ] && echo yes || echo no)"; fi; fi
  if [ -e $DEST ]; then echo "FINALIZE left $DEST in place (sha $(hash_of $DEST || echo UNREADABLE)); NOT reloaded; calm-crab decides"; fi
  echo "FINALIZE end step=$STEP primary_rc=$rc (pre-apply bin/gunbc retained at $R/pre-bin-gunbc, NOT restored)"
  return $rc; }
trap 'exit 143' TERM
trap finalize EXIT
cd $WT || die2 "PRE_MUTATION: cd wt"
printf 'J2_PARTIAL step=begin at=%s out=%s (written BEFORE admit; removed only at DONE)\n' "$(date -u +%FT%TZ)" "$R" > $WD/J2_PARTIAL || die2 "PRE_MUTATION: cannot write J2_PARTIAL ahead of admit"
STEP=admit; die2() { echo "LAB_STATE_CHANGED step=$STEP"; echo "NO OBSERVATION (exit 2): $*"; exit 2; }; snap admit
budget BUD3 900 admit; PATH=/usr/bin:/bin timeout --kill-after=30 $BUD3 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/live_deploy/repository_convergence_placement.dag --function repository_converge_owner_wet \
  --arg candidate=$HEAD_EXPECT --arg object_source=$WT --arg receipt=$LAB/receipts/repository-convergence.json --arg instance_id=srv1-lab-codex-feedback-v0 > $R/admit.log 2>&1
A=$?; contain admit; judge_oom admit; echo "ADMIT_RC=$A"; tail -5 $R/admit.log | cut -c1-400; [ $A = 0 ] || die_lab admit "admit rc=$A (900 s cap)"
v="$(git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted)" || die_lab admit "read admitted ref"; [ "$v" = $HEAD_EXPECT ] || die_lab admit "admitted ref reads $v"
shas_ok; STEP=apply; snap apply
budget BUD4 2400 apply; PATH=/usr/bin:/bin timeout --kill-after=30 $BUD4 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/roadmap/roadmap_dashboard_instance_apply.dag --function srv1_lab_dashboard_apply_wet > $R/apply.log 2>&1
A=$?; contain apply; judge_oom apply; echo "APPLY_RC=$A"; grep -vE "◐|✓|^\[pre-entry\]" $R/apply.log | tail -20 | cut -c1-400; [ $A = 0 ] || die_lab apply "apply rc=$A (2400 s cap)"
for f in $KH; do f="${f/#\~/$HOME}"; printf '%s %s\n' "$(hash_of "$f" 2>/dev/null || echo ABSENT)" "$f"; done > $R/known-hosts-after.txt; echo "KNOWN_HOSTS_DIFF (recorded, not refused):"; diff $R/known-hosts-before.txt $R/known-hosts-after.txt; true
prov_inv > $R/provider-after.txt || die_lab apply "provider inventory after"; echo "PROVIDER_STATE_DIFF (recorded, not refused):"; diff $R/provider-before.txt $R/provider-after.txt; true
STEP=apply-trigger-units; units_state > $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state read failed"; diff $R/paths-before.txt $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state changed during apply"
budget BUD5 30 version; STEP=identity; IV="$(timeout $BUD5 $LAB/bin/gunbc --version 2>&1 | head -1)"; IS="$(hash_of $LAB/bin/gunbc)" || die_lab identity "hash served"; JS="$(hash_of $GB)" || die_lab identity "hash witnessed"
SH="$(git -C $LAB/source rev-parse HEAD)" || die_lab identity "read source head"; ST="$(git -C $LAB/source rev-parse 'HEAD^{tree}')" || die_lab identity "read source tree"
echo "IDENTITY served_version=$IV served_sha=$IS witnessed_sha=$JS lab_source_head=$SH lab_source_tree=$ST" | tee $WD/identity.txt
[ "$IS" = "$JS" ] || die_lab identity "served bytes are not the witnessed bytes"
[ "$IV" = "gunbc $HEAD_EXPECT" ] && [ "$SH" = $HEAD_EXPECT ] && [ "$ST" = $TREE_EXPECT ] || die_lab identity "version/source readback"
{ [ -e $DEST ] || [ -L $DEST ] || [ -e $STAGE ]; } && die_lab identity "$DEST or staging appeared during apply"
STEP=install
timeout 60 sudo -n /usr/bin/install -m 0644 -o root -g root -- $R/serve.unit $STAGE || die_lab install "stage install rc"
[ "$(hash_of $STAGE)" = "$SERVE_UNIT_SHA" ] || die_lab install "staged unit text readback"
timeout 60 sudo -n /usr/bin/mv -T -- $STAGE $DEST || die_lab install "atomic rename rc"
v="$(hash_of $DEST)" || die_lab install "hash installed"; [ "$v" = "$SERVE_UNIT_SHA" ] || die_lab install "installed unit text readback"
STEP=pre-reload; unit_inv > $R/units-pre-reload.txt || die_lab pre-reload "unit inventory failed"
diff <(not_dest $R/units-before.txt) <(not_dest $R/units-pre-reload.txt) || die_lab pre-reload "an entry OTHER than the installed serve unit changed (refused BEFORE daemon-reload)"
awk -v d="$DEST" '$1=="F" && $NF==d {f=1} END {exit !f}' $R/units-pre-reload.txt || die_lab pre-reload "installed unit missing from inventory"
STEP=reload; timeout 120 sudo -n /usr/bin/systemctl daemon-reload || die_lab reload "daemon-reload rc"
STEP=post-reload; unit_inv > $R/units-after.txt || die_lab post-reload "unit inventory failed"; diff $R/units-pre-reload.txt $R/units-after.txt || die_lab post-reload "inventory changed across daemon-reload"
units_state > $R/paths-after-reload.txt || die_lab post-reload "trigger unit read"; diff $R/paths-before.txt $R/paths-after-reload.txt || die_lab post-reload "trigger unit state changed by daemon-reload"
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled; do v="$(uprop $UNIT ${p%%:*})" || die_lab post-reload "read ${p%%:*}"; [ "$v" = "${p##*:}" ] || die_lab post-reload "${p%%:*}=[$v], expected ${p##*:}"; done
sha256sum $DEST $LAB/bin/gunbc > $WD/INSTALLED_SHAS || die_lab post-reload "installed shas"
contain final; wt_ok; shas_ok
mark J2_INSTALLED J1B4_GREEN $WD/INSTALLED_SHAS $WD/identity.txt $R/serve.unit $R/admit.log $R/apply.log $R/units-before.txt $R/units-after.txt $R/paths-before.txt $R/paths-after-reload.txt $R/oom-raw.log
rm -f -- $WD/J2_PARTIAL || die_lab done "cannot remove J2_PARTIAL after success"
STEP=done; echo DONE; exit 0
