#!/bin/bash
# r2 restoration packet REVISION 6, job J2 (launcher job ld449-r2-j2). Author loyal-deer-449. NOT a grant request; byte review first.
# FRAME: #15967 packet_G.sh v4 (sha c781341a...), itself #15695 v4a (hardened frame #15931 v12 lineage): die2, scope gate, cgroup
#   bound gates, OOM parser/judge, verdict classifier, completion parser (REAL line verbatim, sha-checked), rc contract,
#   tree_clean, out_reusable, hash_of and their fixtures are copied BYTE FOR BYTE; only the job name in the scope gate and
#   its fixture, and the booked cgroup values, differ. Exit contract: 0 held, 1 deviation from a pre-registered verdict,
#   2 no observation (every refusal, timeout 124, signal >128, OOM, incoherent counter, survivors).
# LAUNCH: calmcrab-udrv.sh ld449-r2-j2 6000 16 14 4096 3200% <STAGED> <SHA> OUT <SERVE_UNIT_SHA>
# PINS: r2 head 26fe4b8f350a (tree b0ee022a6bf8) = merged integration abcbf27e4cc3 (#15990) + #15919 own commits + r2 1/2..5/5;
#   RED = r2 1/2 154afcd00667. One external process at a time; survivors in this job's cgroup are killed and verified gone.
set -uo pipefail
JOB=ld449-r2-j2
UNIT="$JOB.scope"
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
HEAD_EXPECT=26fe4b8f350a99bd57a9bc342d4625b0cb96804b; TREE_EXPECT=b0ee022a6bf8f9f9f35d496174afd681300c9847
RED_REV=154afcd00667d3c84609f551c45e0ab4f5b83bad; NO15175=f38a00cb75d85e3a7879c27f7288c2a8bba993ae
WD=/home/briansrls/runs/ld449-r2work; WT=$WD/wt; WR=$WD/wt-red; TG=$WD/target; MK=$WD/markers
DECLARED_SERVE_BINARY=/home/briansrls/runs/ld449-r2work/target/release/gunbc
OFF=the_lab_tick_binds_the_slots_cutover_as_not_run_and_writes_no_record
SPW=dag/test/claim/roadmap/session_placement_wet_witness.dag
export PATH=$HOME/.cargo/bin:$PATH
shas_ok() { [ -f $WD/SHAS ] && sha256sum -c --quiet $WD/SHAS >/dev/null 2>&1 || die2 "binary shas changed or missing"; }
shas_id() { hash_of $WD/SHAS; }
# Markers (B1). A marker is EVIDENCE for the next precheck, never authority (each job is its own grant; calm-crab verifies the
# launcher receipts -- frozen.sha256, drv.log, printed SHAS -- independently). It is one field per line:
#   HEAD, TREE, SCRIPT (the writing job's own sha, taken before cd), SHAS (binaries), OUT (the writing job's output dir),
#   PRIOR <name> <sha256 of that marker file>, and one RECEIPT <sha256> <absolute path> per retained receipt.
# need() re-reads reality before trusting a marker: the worktree's HEAD and tree (wt_ok), the binaries (shas_ok), and then the
# WHOLE chain back to J1: every marker's fields, every RECEIPT re-hashed, every PRIOR link re-hashed. Stated limit: a process
# running as the same user can still fabricate consistent files; the markers raise the cost of that, they do not prevent it.
mark() { local name=$1 prior=$2 ps=none f h t; shift 2; [ "$prior" != none ] && { ps="$(hash_of $MK/$prior)" || die2 "cannot hash prior marker $prior"; }
  mkdir -p $MK || die2 "mkdir markers"; t=$MK/.$name.tmp
  { printf 'HEAD %s\nTREE %s\nSCRIPT %s\nSHAS %s\nOUT %s\nPRIOR %s %s\n' $HEAD_EXPECT $TREE_EXPECT $SELF_SHA "$(shas_id)" "$R" $prior $ps
    for f in "$@"; do h="$(hash_of "$f")" || exit 1; printf 'RECEIPT %s %s\n' "$h" "$f"; done; } > $t || die2 "marker write $name"
  mv -f $t $MK/$name || die2 "marker rename $name"; cat $MK/$name; }
mfield() { awk -v k="$2" '$1==k{print substr($0,length(k)+2)}' "$1"; }
need_one() { local m=$MK/$1 want_script=$2 want_prior=$3 v h f pn psha
  [ -f $m ] || die2 "marker $1 missing"
  [ "$(mfield $m HEAD)" = $HEAD_EXPECT ] && [ "$(mfield $m TREE)" = $TREE_EXPECT ] || die2 "marker $1: head/tree"
  [ "$(mfield $m SHAS)" = "$(shas_id)" ] || die2 "marker $1: binaries differ"
  v="$(mfield $m SCRIPT)"; [[ $v =~ ^[0-9a-f]{64}$ ]] || die2 "marker $1: script field"; [ -z "$want_script" ] || [ "$v" = "$want_script" ] || die2 "marker $1 not written by reviewed script $want_script"
  [ "$(grep -c '^PRIOR ' $m)" = 1 ] || die2 "marker $1: PRIOR"; pn="$(mfield $m PRIOR | cut -d' ' -f1)"; psha="$(mfield $m PRIOR | cut -d' ' -f2)"
  [ "$pn" = "$want_prior" ] || die2 "marker $1 chains to $pn, expected $want_prior"
  [ "$pn" = none ] || { h="$(hash_of $MK/$pn)" || die2 "hash $pn"; [ "$h" = "$psha" ] || die2 "marker $1's prior $pn changed since it was written"; }
  [ "$(grep -c '^RECEIPT ' $m)" -ge 1 ] || die2 "marker $1 binds no receipts"
  while read -r _ h f; do [ "$(hash_of "$f")" = "$h" ] || die2 "marker $1: receipt $f changed or missing"; done < <(grep '^RECEIPT ' $m)
  NEXT_PRIOR="$pn"; }
# need <marker> <EXACT reviewed script sha> <its prior marker>: verifies the named marker, then walks and verifies the chain to J1.
CHAIN_PRIORS="J1R_GREEN:J1_GREEN J1B1_GREEN:J1R_GREEN J1B2_GREEN:J1B1_GREEN J1B3_GREEN:J1B2_GREEN J1B4_GREEN:J1B3_GREEN J2_INSTALLED:J1B4_GREEN J1_GREEN:none"
need() { local cur exp x; wt_ok; shas_ok; need_one $1 $2 $3; cur="$NEXT_PRIOR"
  while [ "$cur" != none ]; do exp=""; for x in $CHAIN_PRIORS; do [ "${x%%:*}" = "$cur" ] && exp="${x##*:}"; done
    [ -n "$exp" ] || die2 "chain: unknown marker $cur"; need_one $cur "" $exp; cur="$NEXT_PRIOR"; done; echo "chain verified to J1"; }
# wt_ok: the worktree is really at the pin and pristine, including ignored files (tree_clean cannot see a different commit).
wt_ok() { local o; [ "$(git -C $WT rev-parse HEAD)" = $HEAD_EXPECT ] && [ "$(git -C $WT rev-parse 'HEAD^{tree}')" = $TREE_EXPECT ] || die2 "worktree is not at the pin"
  o="$(git -C $WT status --porcelain --ignored)" || die2 "git status failed"; [ -z "$o" ] || die2 "worktree has changes or ignored files"; }
# Writes OUTSIDE OUT by wet witnesses (B3): they mktemp under fixed /tmp templates. Before each wet run the job lists the
# entries matching those templates; after it, every NEW entry owned by this user is recorded in $R/tmp-created.txt and
# removed (rm -rf -- of that literal, template-checked path); a removal failure is exit 2.
TMP_GLOBS='/tmp/gunbc_g5_* /tmp/gunbc_attempt_state_move_wet_* /tmp/wre3-*'
tmp_list() { local g p; for g in $TMP_GLOBS; do for p in $g; do [ -e "$p" ] && [ -O "$p" ] && echo "$p"; done; done | sort; }
tmp_clean() { local p; comm -13 <(printf '%s\n' "$1") <(tmp_list) | while read -r p; do
    case "$p" in /tmp/gunbc_g5_?*|/tmp/gunbc_attempt_state_move_wet_?*|/tmp/wre3-?*) ;; *) exit 1;; esac
    echo "$p" >> $R/tmp-created.txt; rm -rf -- "$p" || exit 1; done || die2 "cleanup of wet-witness /tmp entries failed"; }
# run_entry LEG TAG DIR ENTRY FUNCS(csv) WET(0|1) CAP : one claim_batch run, serial, under timeout CAP (timeout puts the run in
# its own process group and signals the group); every fn must have exactly one bare verdict; rc must equal this run's verdicts.
run_entry() { local LEG="$1" T="$2" DIR="$3" F="$4" FUNCS="$5" WET="$6" CAP="$7" L RC VE VD N A=""
  L="$R/claims-$LEG-$T.log"; [ "$WET" = 1 ] && A=--wet
  roster_gate "$DIR/$F" $(echo "$FUNCS" | tr ',' ' ') || die2 "leg $LEG $T: roster not declared exactly once, in order, in $F"
  local TB=""; [ "$WET" = 1 ] && TB="$(tmp_list)"
  snap "run $LEG $T"
  ( cd "$DIR" && timeout --kill-after=30 "$CAP" "$TG/release/claim_batch" $A --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$F" --functions "$FUNCS" ) > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; contain "run $LEG $T"; [ "$WET" = 1 ] && tmp_clean "$TB"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
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
mapfile -t B0 < "$CG/cgroup.procs" || die2 "REFUSED_SETUP: cannot read $CG/cgroup.procs"
survivors() { local -a now; local p b keep out=""; mapfile -t now < "$CG/cgroup.procs" || { echo READ_FAILED; return; }
  for p in "${now[@]}"; do keep=0; [ "$p" = "$$" ] && keep=1; [ "$p" = "$BASHPID" ] && keep=1
    for b in "${B0[@]}"; do [ "$p" = "$b" ] && keep=1; done; [ $keep = 0 ] && out="$out $p"; done; printf '%s' "$out"; }
contain() { local s i; s="$(survivors)"; [ "$s" = READ_FAILED ] && die2 "cannot read own cgroup after $1"; [ -z "$s" ] && return 0
  echo "SURVIVORS after $1:$s"; kill -TERM $s 2>/dev/null
  for i in $(seq 1 30); do s="$(survivors)"; [ -z "$s" ] && return 0; sleep 1; done; kill -KILL $s 2>/dev/null
  for i in $(seq 1 10); do s="$(survivors)"; [ -z "$s" ] && return 0; sleep 1; done; die2 "survivors remain in this job's cgroup after $1:$s"; }
# prior_scope_gone <job>: the prior job's launcher scope must be inactive (no survivors kept it alive) in BOTH managers.
prior_scope_gone() { local v m; for m in --user --system; do v="$(timeout 30 systemctl $m show "$1.scope" -p ActiveState --value)" || die2 "cannot read $1.scope ($m)"
  case "$v" in inactive|failed) ;; *) die2 "prior job scope $1.scope is [$v] in $m manager (survivors or still running)";; esac; done; }
# uprop <unit> <property>: a CHECKED systemctl read (rc 0 required); empty output stays empty and callers decide.
uprop() { timeout 30 systemctl show "$1" -p "$2" --value; }
# ==== end r2 common ====
J1B4_SHA=a5dae8d49738ee1015b9ac58df2dc110fe87b322e99a0b0658e23aea1e0ecaa0
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
# ADMIT (B8; APPLY-EFFECTS section A): repository_converge_owner_wet runs unprivileged as briansrls (whoami, git; no sudo, ssh,
#   systemd-run or network): git fetch of 26fe4b8f from $WT into /opt/gunbc-lab/codex-provider-feedback-v0/source (objects,
#   FETCH_HEAD); `git update-ref refs/gunbc/srv1-lab-admitted <new> <old>` (CAS on the value read at its start); `git reset
#   --hard` there (moves the checked-out branch, index, tracked files, ORIG_HEAD); deletes source/dag/gunbc/{recurring_failure_mode,
#   rung_drop}/roster.dag if present; writes the receipt non-atomically to $LAB/receipts/repository-convergence.json. It has NO
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
die_lab() { echo "LAB_STATE_CHANGED step=$1"; STEP="$1"; die2 "step=$1: $2"; }
SERVE_UNIT_SHA="$EXTRA_ARG"; [[ $SERVE_UNIT_SHA =~ ^[0-9a-f]{64}$ ]] || die2 "PRE_MUTATION: arg 2 must be the reviewed serve unit sha256"
prior_scope_gone ld449-r2-j1b4; need J1B4_GREEN "$J1B4_SHA" J1B3_GREEN
[ "$TG/release/gunbc" = "$DECLARED_SERVE_BINARY" ] || die2 "PRE_MUTATION: declared serve binary path"
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
  timeout 300 find "$root" -xdev -print0 > $R/.inv.find 2> $R/.inv.err || return 1; [ -s $R/.inv.err ] && return 1
  sort -z $R/.inv.find | while IFS= read -r -d '' f; do
    [ -n "$filt" ] && case "${f#$root/}" in $filt|*/$filt) ;; *) [ "$f" = "$root" ] || continue;; esac
    m="$(stat -c '%a %U:%G %s %Y %i' -- "$f")" || exit 1
    if [ -L "$f" ]; then t="$(readlink -- "$f")" || exit 1; printf 'L %s %s -> %s\n' "$m" "$f" "$t"
    elif [ -d "$f" ]; then printf 'D %s %s\n' "$m" "$f"
    elif [ -f "$f" ]; then if [ -r "$f" ]; then h="$(hash_of "$f")" || exit 1; else h="unreadable-statsig"; fi; printf 'F %s %s %s\n' "$m" "$h" "$f"
    else printf 'X %s %s\n' "$m" "$f"; fi; done; }
unit_inv() { inv_dir /etc/systemd/system && inv_dir /run/systemd/system 'gunbc-srv1-lab-codex-feedback-v0*' && inv_dir /home/briansrls/.config/systemd/user; }
PATHS="gunbc-srv1-lab-codex-feedback-v0-belt-exit.path gunbc-srv1-lab-codex-feedback-v0-belt-discovery.path gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain.path gunbc-srv1-lab-codex-feedback-v0-publication.path gunbc-srv1-lab-codex-feedback-v0-belt-credential-recovery.timer"
units_state() { local u p v t; for u in $PATHS; do printf '%s' "$u"; for p in Id LoadState UnitFileState ActiveState SubState Triggers; do v="$(uprop $u $p)" || return 1; printf ' %s=%s' $p "$v"; done
    t="$(uprop $u Triggers)" || return 1; for x in $t; do for p in LoadState UnitFileState ActiveState SubState; do v="$(uprop $x $p)" || return 1; printf ' %s.%s=%s' $x $p "$v"; done; done; echo; done; }
prov_inv() { [ -d "$PROVIDER_STATE_ROOT" ] || { echo "ABSENT $PROVIDER_STATE_ROOT"; return 0; }; stat -c "DIR %a %U:%G %n" "$PROVIDER_STATE_ROOT" || return 1
  timeout 300 find "$PROVIDER_STATE_ROOT" -xdev -type f -printf '%s %T@ %p\n' > $R/.prov.find 2>/dev/null || return 1
  sort -k3 $R/.prov.find | while read -r sz mt f; do if [ -r "$f" ]; then h="$(hash_of "$f")" || exit 1; else h=unreadable-statsig; fi; printf '%s %s %s %s\n' "$h" "$sz" "$mt" "$f"; done; }
unit_inv > $R/units-before.txt || die2 "PRE_MUTATION: unit inventory failed"; echo "inventory: $(wc -l < $R/units-before.txt) entries"
units_state > $R/paths-before.txt || die2 "PRE_MUTATION: trigger unit state read failed"; cat $R/paths-before.txt
prov_inv > $R/provider-before.txt || die2 "PRE_MUTATION: provider inventory failed"
{ sha256sum $LAB/bin/gunbc $LAB/bin/gunbc.prev; git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted HEAD; ls $LAB/attempt-state/attempt-identities | sha256sum; } > $R/pre-snapshot.txt 2>&1; cat $R/pre-snapshot.txt
# ---- finalization: armed at the first mutation ----
STEP=pre; finalize() { local rc=$?; [ "$STEP" = done ] && return; [ "$STEP" = pre ] && return
  { echo "J2_PARTIAL step=$STEP rc=$rc at=$(date -u +%FT%TZ) out=$R"; sha256sum $LAB/bin/gunbc 2>&1; ls -l $DEST $STAGE 2>&1; } > $WD/J2_PARTIAL
  local cleaned=0; if [ -e $STAGE ]; then timeout 60 sudo -n /usr/bin/rm -f -- $STAGE && cleaned=1; fi
  if [ -e $DEST ] && [ "$(hash_of $DEST)" != "$SERVE_UNIT_SHA" ]; then timeout 60 sudo -n /usr/bin/rm -f -- $DEST && cleaned=1; fi
  [ $cleaned = 1 ] && timeout 120 sudo -n /usr/bin/systemctl daemon-reload
  echo "FINALIZED step=$STEP (see $WD/J2_PARTIAL; pre-apply bin/gunbc retained at $R/pre-bin-gunbc, NOT restored)" | tee -a $WD/J2_PARTIAL; }
trap finalize EXIT
cd $WT || die2 "PRE_MUTATION: cd wt"
STEP=admit; snap admit
PATH=/usr/bin:/bin timeout --kill-after=30 900 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/live_deploy/repository_convergence_placement.dag --function repository_converge_owner_wet \
  --arg candidate=$HEAD_EXPECT --arg object_source=$WT --arg receipt=$LAB/receipts/repository-convergence.json --arg instance_id=srv1-lab-codex-feedback-v0 > $R/admit.log 2>&1
A=$?; contain admit; judge_oom admit; echo "ADMIT_RC=$A"; tail -5 $R/admit.log | cut -c1-400; [ $A = 0 ] || die_lab admit "admit rc=$A (900 s cap)"
v="$(git -C $LAB/source rev-parse refs/gunbc/srv1-lab-admitted)" || die_lab admit "read admitted ref"; [ "$v" = $HEAD_EXPECT ] || die_lab admit "admitted ref reads $v"
shas_ok; STEP=apply; snap apply
PATH=/usr/bin:/bin timeout --kill-after=30 2400 $GB run --source-root dag --source-root src/v2 --entry dag/gunbc/roadmap/roadmap_dashboard_instance_apply.dag --function srv1_lab_dashboard_apply_wet > $R/apply.log 2>&1
A=$?; contain apply; judge_oom apply; echo "APPLY_RC=$A"; grep -vE "◐|✓|^\[pre-entry\]" $R/apply.log | tail -20 | cut -c1-400; [ $A = 0 ] || die_lab apply "apply rc=$A (2400 s cap)"
prov_inv > $R/provider-after.txt || die_lab apply "provider inventory after"; echo "PROVIDER_STATE_DIFF (recorded, not refused):"; diff $R/provider-before.txt $R/provider-after.txt; true
STEP=apply-trigger-units; units_state > $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state read failed"; diff $R/paths-before.txt $R/paths-after.txt || die_lab apply-trigger-units "trigger unit state changed during apply"
STEP=identity; IV="$($LAB/bin/gunbc --version 2>&1 | head -1)"; IS="$(hash_of $LAB/bin/gunbc)" || die_lab identity "hash served"; JS="$(hash_of $GB)" || die_lab identity "hash witnessed"
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
diff <(grep -v " $DEST\$" $R/units-before.txt) <(grep -v " $DEST\$" $R/units-pre-reload.txt) || die_lab pre-reload "an entry OTHER than the installed serve unit changed (refused BEFORE daemon-reload)"
grep -q " $DEST\$" $R/units-pre-reload.txt || die_lab pre-reload "installed unit missing from inventory"
STEP=reload; timeout 120 sudo -n /usr/bin/systemctl daemon-reload || die_lab reload "daemon-reload rc"
STEP=post-reload; unit_inv > $R/units-after.txt || die_lab post-reload "unit inventory failed"; diff $R/units-pre-reload.txt $R/units-after.txt || die_lab post-reload "inventory changed across daemon-reload"
units_state > $R/paths-after-reload.txt || die_lab post-reload "trigger unit read"; diff $R/paths-before.txt $R/paths-after-reload.txt || die_lab post-reload "trigger unit state changed by daemon-reload"
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled; do v="$(uprop $UNIT ${p%%:*})" || die_lab post-reload "read ${p%%:*}"; [ "$v" = "${p##*:}" ] || die_lab post-reload "${p%%:*}=[$v], expected ${p##*:}"; done
sha256sum $DEST $LAB/bin/gunbc > $WD/INSTALLED_SHAS || die_lab post-reload "installed shas"
contain final; wt_ok; shas_ok
mark J2_INSTALLED J1B4_GREEN $WD/INSTALLED_SHAS $WD/identity.txt $R/serve.unit $R/admit.log $R/apply.log $R/units-before.txt $R/units-after.txt $R/paths-before.txt $R/paths-after-reload.txt $R/oom-raw.log
STEP=done; echo DONE; exit 0
