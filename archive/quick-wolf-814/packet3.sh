#!/usr/bin/env bash
# PACKET 16011-A v2 — #16011 review-reconciliation witness, HEAD leg only, hermetic claim_batch run. NOT AUTHORIZED TO RUN; review material.
# CREDITS (copied, not re-derived): (a) gen3 v3 (#16006 issuecomment-6698959868, sha afd5fa2e): the frame block oom_parse..oom_verdict
#   is VERBATIM (sha256 cb6d3b29... of exactly that block; bytes = lines 76-124 of the decoded gen3.sh), plus its single-OUT argv, exclusive-mkdir
#   OUT and DEADLINE/phasecap budget pattern. (b) fierce-bee's S3 v4 (#15975 issuecomment-6700033778, sha fef5016e, 40,953 B): oomread/snap/judge_oom,
#   classify/verdict/completed/hash_of/rc_in_contract, roster-free run_entry with JOIN LAW v3 (exactly one bare PASS/FAIL per roster name, no foreign
#   names, no other ALLCAPS keyword, CR refused, rc cross-checked) and its fixtures. ONE EDIT to run_entry: source roots are `dag` and `src/v2` only
#   (no src/v1), per the offline recipe; everything else in run_entry/join is byte-identical to S3 v4.
# CLAIMS SCOPE: the witness feeds WorkerProcessSignaled{signal:9} / WorkerProcessExited{exit_code:1} as DATA into review_reconcile / standing_blocks.
#   No process is killed, there is no memory pressure, and NO claim covers child-OOM liveness (a unit that survives a child's OOM kill writes no exit
#   record; that is gunbc.host.memory_stall_refusal supervised_subject_verdict, declared a gap in the PR). The 446-module text-closure walk quoted in the PR
#   is source evidence only, NOT a typecheck; this run's resolve is the first typecheck of these rows.
# RED-AT-BASE LEG IS NOT CARRIED: head+base would need two builds (2*3400+600 > the 6900 s deadline) and no evidence transfers between legs. The
#   witness's discrimination vs base is therefore UNESTABLISHED by this packet (a base leg is a separate packet).
# SOURCE QUESTIONS RESOLVED BY READING AT HEAD: claim_batch accepts repeated/cumulative flags: src/v1/stage0/src/bin/claim_batch.rs parse arms
#   "--entry" (opens an EntryGroup) and "--function"/"--functions" (push onto entry_groups.last_mut(); error 2 if before --entry). Package that owns the bin:
#   src/v1/stage0/Cargo.toml [package] name = "v1-compiler", [[bin]] name = "claim_batch" path = "src/bin/claim_batch.rs".
# PINS (head leg): HEAD=f65523fb1240fbba34b77eac10fff101610e2117 tree=d9afab357af028ff34a58d88baa19a7a94562e8b (base 50652e0777329a585efa6919ef5ab9a0c81222ae, informational)
#   blob witness dag/test/claim/roadmap/roadmap_review_role_witness.dag = 40b14c9879bc3b6cd4afa0ab62eb7a5a79296336
#   blob source  dag/gunbc/roadmap/roadmap_review_role.dag             = 39cc1ca3176ce87ed4852ed47027f8733c0868d5
#   binary: built IN-LEG (cargo build -j 32 --locked --release -p v1-compiler --bin claim_batch), sha256 printed and re-checked after the run.
# ROSTER (exactly these two, declared order in the witness file, gated against the file's `test fn` list after checkout):
#   the_reconciliation_orders_changes_over_blocks_over_running_over_pending_over_approved  a_killed_or_exited_stage_without_a_verdict_is_failed_not_refused
#   pre-registered: both PASS.
# ARGV EXACTLY (launcher): calmcrab-udrv.sh 16011-a-claim-batch 7200 32 28 4096 3200% <STAGED packet.sh> <PACKET_SHA256> OUT
#   (same booked bounds as gen3/S3: memory.max 32GiB, memory.high 28GiB, swap 0, pids 4096, cpu 3200%, verified against the real cgroup files before any fetch).
#   claim_batch argv inside: --claim-run --source-root dag --source-root src/v2 --entry dag/test/claim/roadmap/roadmap_review_role_witness.dag --functions <name>,<name>  (Hermetic, NO --wet)
# MODES: `bash packet.sh --classifier-fixtures` = local mktemp only, no network/cgroup/fetch; exits 0 if every fixture holds, 2 otherwise.
# EXIT: 0 = both rows PASS as pre-registered, all gates held. 1 = a row contradicted its expectation while every row observed. 2 = NO OBSERVATION: any gate,
#   fetch, build, timeout, OOM rise (oom/oom_kill), counter incoherence, out-of-contract rc (124/signal), missing completion line, or a name without a SINGLE bare verdict.
# COMPLETION LINE: stdout ends "DONE" only on rc 0/1; exactly one '[process-partition] total_ms=<digits>' line is required in the claim_batch log.
# EFFECT SCOPE (keen-otter ruling): the build WRITES the shared CARGO_HOME registry/git cache (~/.cargo) outside OUT. Possible rustup/network effects: rust-toolchain.toml pins 1.93.0, but earlier retained toolchain evidence is NOT fresh proof that nothing gets installed (an install would write ~/.rustup and use network). Ambient git config and GUNBC_* assumptions are UNKNOWN; #15997 is NOT cited to clear them. The red leg temporarily swaps one tracked file in the DISPOSABLE checkout under OUT (never the PR checkout) and restores it, re-verifying blob, tree_clean, HEAD/tree and binary sha. Build -j 32. Every claim run has timeout -k K.
# BUDGET (a PROPOSAL needing a named keen-otter grant; RT 7200 is launcher capability, not authorization): DEADLINE = start+6900 (RT 7200 - 300 reserve). Phases: fetch 600+K30, build 2400+K30, head run 1000+K30, red run 1000+K30, final 90 => 5210 <= 6900. Fixtures run first (bounded, seconds) before any phase.
# CLAIMS SCOPE: modeled data only. Signal 9 and exit 1 are DATA; no real kill or memory pressure is exercised; no claim about real OOM behaviour or child-OOM liveness (gunbc.host.memory_stall_refusal). Red leg: head claim_batch+head witness with the BASE role module; a resolve/closure refusal there is NO_OBSERVATION. Reusing the head binary for the red leg assumes claim_batch loads sources dynamically from --source-root (unproven here; the binary sha is re-checked and a resolve refusal exits 2).
#   A phase starts only if its FULL cap + K + FINAL_ALLOW fits the time left, else rc 2 (never a shortened cap). Network: anonymous HTTPS fetch of one public commit
#   from github.com and the crates.io index/static.crates.io for `--locked` registry crates; sizes unmeasured. Disk: nothing deleted; needs >= 25 GiB free (refuses).
set -u
JOB=16011-a-claim-batch
# RED LEG (claim-level, source substitution): head claim_batch + head witness, role module swapped to the BASE blob.
BASEPIN=50652e0777329a585efa6919ef5ab9a0c81222ae
BASEBLOB=6448fa7ebddf43152b7e5660b938ce37772dac6c   # base roadmap_review_role.dag; head blob is SBLOB 39cc1ca3
# PRE-REGISTERED base-leg verdicts: a_killed_... MUST FAIL (base standing_blocks refuses Exited/Signaled; no stage-death branch).
# the_reconciliation_... expected PASS (its text is unchanged between base and head; a deviation is REPORTED, not hidden).
# A resolve/closure refusal of this leg is NO_OBSERVATION (exit 2), NEVER red.; UNIT="$JOB.scope"
PIN=f65523fb1240fbba34b77eac10fff101610e2117; TREE=d9afab357af028ff34a58d88baa19a7a94562e8b
WBLOB=40b14c9879bc3b6cd4afa0ab62eb7a5a79296336; SBLOB=39cc1ca3176ce87ed4852ed47027f8733c0868d5
WFILE=dag/test/claim/roadmap/roadmap_review_role_witness.dag; SFILE=dag/gunbc/roadmap/roadmap_review_role.dag
FETCH_CAP_S=600; BUILD_CAP_S=2400; ENTRY_CAP_S=1000; K=30; BUDGET=6900; FINAL_ALLOW=90
ROSTER="the_reconciliation_orders_changes_over_blocks_over_running_over_pending_over_approved a_killed_or_exited_stage_without_a_verdict_is_failed_not_refused"
die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
# ---- VERBATIM frame block (gen3 v3 lines 76-124, sha256 cb6d3b29...) ----
oom_parse() { # $1=snapshot text $2=key
  local all ok; all=$(printf '%s\n' "$1" | awk -v k="$2" '$1==k' | wc -l | tr -d ' '); ok=$(printf '%s\n' "$1" | grep -cE "^$2 [0-9]+\$")
  if [ "$all" = 1 ] && [ "$ok" = 1 ]; then printf '%s\n' "$1" | grep -E "^$2 [0-9]+\$" | cut -d' ' -f2; else echo UNKNOWN; fi; }
oom_key() { [ -r "$1" ] || { echo UNKNOWN; return; }; local t; t=$(cat "$1" 2>/dev/null) || { echo UNKNOWN; return; }; oom_parse "$t" "$2"; }
oom_read() { echo "$(oom_key "$1" oom_kill)"; }
oom_pair() { [ -r "$1" ] || { echo UNKNOWN/UNKNOWN; return; }; local t; t=$(cat "$1" 2>/dev/null) || { echo UNKNOWN/UNKNOWN; return; }
  echo "$(oom_parse "$t" oom_kill)/$(oom_parse "$t" oom)"; }
# GIT CLEAN CHECK: git status is captured and its rc REQUIRED to be 0 before its output is tested for emptiness; a failed
#   git status (empty stdout) is NOT clean. Used before the build and at restore.
tree_clean() { local out; out=$(git status --porcelain 2>/dev/null) || return 1; [ -z "$out" ]; }
# tree_changes: prints the number of porcelain entries and RETURNS 0 on a successful status (0 for a clean tree: 'grep -c'
# alone prints 0 but returns 1 on no match, which made every clean tree read as a failure); a failed git status prints
# status-failed and returns 1.
tree_changes() { local out; out=$(git status --porcelain 2>/dev/null) || { echo status-failed; return 1; }
  if [ -z "$out" ]; then echo 0; else printf '%s\n' "$out" | grep -c .; fi; return 0; }
# BINARY HASH (keen-otter (2)): each hash command must SUCCEED and yield exactly one 64-hex sha, else it fails; an
#   equality test is made only between two successful reads, so two failed reads can never compare equal.
bin_sha() { local o h; o=$(sha256sum -- "$1" 2>/dev/null) || return 1; h=${o%% *}
  [[ "$h" =~ ^[0-9a-f]{64}$ ]] && [ "$(printf '%s\n' "$o" | grep -c .)" = 1 ] || return 1; printf '%s' "$h"; }
# RAW OOM EVIDENCE (keen-otter (4)): every boundary decision reads memory.events ONCE through oom_snap, which writes those
#   exact raw bytes, timestamped and sequence-numbered, under OUT/memevents/ and returns the pair parsed from that same text.
oom_snap() { # $1=event file $2=tag; prints KILL/OOM parsed from the bytes it saved
  local t f n; n=$(( $(find "$MEMDIR" -maxdepth 1 -type f 2>/dev/null | wc -l) + 1 )); f=$MEMDIR/$(printf '%03d' $n)-$(date -u +%Y%m%dT%H%M%S.%NZ)-$2.txt
  if [ -r "$1" ] && t=$(cat "$1" 2>/dev/null); then
    # exclusive create (noclobber): the raw bytes must be PUBLISHED before any numeric pair is returned; a failed
    # write (missing or read-only MEMDIR, an existing target) returns UNKNOWN/UNKNOWN, so every caller is NO_OBSERVATION.
    if ( set -C; printf '%s\n' "$t" > "$f" ) 2>/dev/null; then echo "$(oom_parse "$t" oom_kill)/$(oom_parse "$t" oom)"
    else echo UNKNOWN/UNKNOWN; fi
  else ( set -C; : > "$f.unreadable" ) 2>/dev/null; echo UNKNOWN/UNKNOWN; fi; }
# OUTPUT PATH: refuses ANY existing entry: a file, a directory (empty or not), a symlink, a dangling symlink.
out_absent() { ! { [ -e "$1" ] || [ -L "$1" ]; }; }
oom_pair_verdict() { local a=$(oom_verdict "${1%/*}" "${2%/*}") b=$(oom_verdict "${1#*/}" "${2#*/}")
  if [ "$a" = none ] && [ "$b" = none ]; then echo none; elif [ "$a" = UNKNOWN ] || [ "$b" = UNKNOWN ]; then echo UNKNOWN
  elif [ "$a" = COUNTER_INCOHERENT ] || [ "$b" = COUNTER_INCOHERENT ]; then echo COUNTER_INCOHERENT; else echo OOM_KILLED; fi; }
BOOKED="memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000"
bounds_check() { # $1=cgroup dir; prints the first mismatch and fails, or prints nothing
  local w k v got
  for w in $BOOKED; do k=${w%%=*}; v=${w#*=}; v=${v//_/ }
    [ -r "$1/$k" ] || { echo "$k unreadable"; return 1; }
    got=$(cat "$1/$k" 2>/dev/null) || { echo "$k unreadable"; return 1; }
    [ "$got" = "$v" ] || { echo "$k is '$got', booked '$v'"; return 1; }
  done; }
scope_gate() { # $1=/proc/self/cgroup content $2=JOB; prints the cgroup path, or fails
  local n; n=$(printf '%s\n' "$1" | grep -c .)
  [ "$n" = 1 ] && [ -n "$2" ] || return 1
  case "$1" in 0::/*) ;; *) return 1;; esac
  case "${1#0::}" in */"$2".scope) printf '%s' "${1#0::}";; *) return 1;; esac; }
oom_verdict() { case "$1" in ''|*[!0-9]*) echo UNKNOWN; return;; esac; case "$2" in ''|*[!0-9]*) echo UNKNOWN; return;; esac
  if [ "$2" -lt "$1" ]; then echo COUNTER_INCOHERENT; elif [ "$2" -gt "$1" ]; then echo OOM_KILLED; else echo none; fi; }
# ---- end verbatim frame block ----
# ---- S3 v4 helpers (verbatim lines 88-125) ----
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
# The REAL completion line, copied verbatim from calm-crab-645's G1 cpr log (sha a0e1f1f6), with sha256 checked below.
REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
[ "$(printf %s "$REAL_COMPLETION" | sha256sum | cut -d' ' -f1)" = "7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2" ] || die2 "REFUSED_SETUP: the real-line fixture is not the verbatim bytes"
scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
# ---- S3 v4 roster_gate + run_entry (verbatim 240-312; one edit: src/v1 root dropped) ----
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

run_entry() { # LEG BIN TAG PATH FUNCS(csv) : one claim_batch invocation; every row contract-checked with v17's verdict classifier, then rc vs THIS entry's verdicts
  local LEG="$1" BIN="$2" T="$3" F="$4" FUNCS="$5"
  local L="$R/claims-$LEG-$T.log" RC VE VD N
  snap "run $LEG $T"
  timeout -k "$K" "$ENTRY_CAP_S" "$BIN" --claim-run --source-root dag --source-root src/v2 --entry "$F" --functions "$FUNCS" > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
  rc_in_contract "$RC" || die2 "leg $LEG entry $T: exit status $RC is out of contract (0/1/2 only; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg $LEG entry $T: no single '[process-partition] total_ms=<digits>' completion line; batch incomplete whatever verdict lines printed"
  echo "verdict lines ($LEG $T):"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"
  # CLOSURE LAW: closure-diagnostics marker = NO_OBSERVATION, never row FAILs (keen-otter msg_f0f45c91).
  if grep -qaE 'reference_closure: .*fail-closed|closure diagnostics in' "$L"; then die2 "leg $LEG entry $T: CLOSURE DIAGNOSTIC — closure failure reads NO_OBSERVATION, never row verdicts: $(grep -aE 'reference_closure: .*fail-closed|closure diagnostics in' "$L" | head -1)"; fi
  # JOIN LAW v2 (keen-otter msg_2356fbe3 + msg_b5063276): outcome lines are detected CASE-INSENSITIVELY
  # for the known keyword family (incl. a leading space and a trailing colon), and EVERY such line must be
  # exactly 'PASS <name>' or 'FAIL <name>' for a SELECTED roster name. Unknown ALLCAPS-token lines stay a
  # CASE-SENSITIVE scan (no global grep -i: that was #15988 v2's bug). CR bytes refuse. The batch rc is
  # cross-checked against THIS validated set (below), never against loose FAIL lines. Malformed
  # outcome-like lines ('fail a', ' FAIL a', 'FAIL: a') are refused, never ignored. Validated against the
  # real retained claim_batch logs (2026-11-06 session-container runs): normal output is not refused.
  LC_ALL=C grep -qa $'\r' "$L" && die2 "leg $LEG entry $T: JOIN LAW — CR byte in outcome log"
  local _JV
  # NOTE (sleek-owl): awk 'exit' STILL RUNS END — an error path prints 'MALFORMED:...' (or
  # FOREIGN/ALLCAPS) AND THEN 'COUNT:n'; _JV therefore holds both lines, and the 'case COUNT:*'
  # test below refuses precisely because the FIRST line is not COUNT.
  _JV=$(LC_ALL=C awk -v roster=" $(echo "$FUNCS" | tr ',' ' ') " '
    BEGIN { nk=split("pass fail failed skip skipped error unobserved panicked abort aborted timeout timedout timed_out ignored xfail notbool runtimeerror", K, " ") }
    {
      line=$0
      t=line; sub(/^[ 	]+/, "", t); lw=tolower(t)
      outcome_like=0
      for (i=1;i<=nk;i++) { k=K[i]
        if (lw==k || substr(lw,1,length(k)+1)==k":" || substr(lw,1,length(k)+1)==k" " || substr(lw,1,length(k)+1)==k"	") { outcome_like=1; break } }
      if (outcome_like) {
        if (line !~ /^(PASS|FAIL) [A-Za-z0-9_]+$/) { print "MALFORMED:" line; exit }
        n=$2
        if (index(roster, " " n " ") == 0) { print "FOREIGN:" n; exit }
        cnt++; next
      }
      if (line ~ /^[A-Z][A-Z0-9_]*([ 	:]|$)/) { print "ALLCAPS:" line; exit }
    }
    END { print "COUNT:" cnt+0 }' "$L")
  case "$_JV" in COUNT:*) : ;; *) die2 "leg $LEG entry $T: JOIN LAW v2 — $_JV";; esac
  [ "${_JV#COUNT:}" = "$(echo "$FUNCS" | tr ',' ' ' | wc -w)" ] || die2 "leg $LEG entry $T: JOIN LAW v2 — ${_JV#COUNT:} outcome lines for $(echo "$FUNCS" | tr ',' ' ' | wc -w) selected names (duplicate or shortfall; per-name verdicts settle the rest)"
  VE=0
  for N in $(echo "$FUNCS" | tr ',' ' '); do
    VD="$(verdict "$L" "$N")"; echo "$LEG $T $N=$VD"
    if [ "$VD" = NONE ]; then MISSING=1; die2 "leg $LEG entry $T: '$N' has no SINGLE BARE PASS/FAIL verdict (absent, duplicate, conflicted, reasoned, UNOBSERVED or PANICKED)"; fi
    [ "$VD" = FAIL ] && VE=1
    OBS["$LEG.$T.$N"]="$VD"
    if [ "${EXPECT["$LEG.$T.$N"]:-NONE}" = "ANY" ]; then echo "OBSERVED-ANY leg $LEG entry $T fn $N: $VD (pre-registered ANY)"
    elif [ "$VD" != "${EXPECT["$LEG.$T.$N"]:-NONE}" ]; then DEVIATIONS=$((DEVIATIONS+1)); echo "DEVIATION leg $LEG entry $T fn $N: observed $VD, pre-registered ${EXPECT["$LEG.$T.$N"]:-NONE}"; fi
  done
  [ "$RC" = "$VE" ] || die2 "leg $LEG entry $T: claim_batch returned $RC, its own verdict lines say $VE"
}
# ---- FIXTURES (S3 v4 fixture block, verbatim except the literal scope name -> this packet's JOB) ----
classifier_fixtures() {
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
[ "$(r scope_ok '0::/a/16011-a-claim-batch.scope') $(r scope_ok '0::/a/16011-a-claim-batchXscope') $(r scope_ok '0::/a/x16011-a-claim-batch.scope') $(r scope_ok '0::/a/16011-a-claim-batch.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
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
  # frame-block fixtures (this packet): oom_verdict / oom_pair_verdict / oom_parse on the verbatim gen3 block
  [ "$(oom_verdict 0 0)" = none ] && [ "$(oom_verdict 0 1)" = OOM_KILLED ] && [ "$(oom_verdict 2 1)" = COUNTER_INCOHERENT ] && [ "$(oom_verdict x 1)" = UNKNOWN ] || die2 "REFUSED_SETUP: oom_verdict fixture misfire"
  [ "$(oom_parse $'oom 0\noom_kill 3' oom_kill)" = 3 ] && [ "$(oom_parse $'oom_kill 1\noom_kill 1' oom_kill)" = UNKNOWN ] || die2 "REFUSED_SETUP: oom_parse fixture misfire"
  # JOIN-LAW fixtures drive the REAL run_entry against stub binaries and a fake cgroup directory (local mktemp only)
  FXR="$(mktemp -d)" || die2 "mktemp"; R="$FXR"; CG="$FXR/cg"; mkdir "$CG" && printf 'oom 0\noom_kill 0\n' > "$CG/memory.events" || die2 "fake cgroup"
  declare -gA EXPECT OBS; MISSING=0; DEVIATIONS=0
  FXR="$FXR/join-fixture"; mkdir -p "$FXR" || die2 "REFUSED_SETUP: join fixture dir"
FXR="$R/join-fixture"; mkdir -p "$FXR" || die2 "REFUSED_SETUP: join fixture dir"
EXPECT["fx.fxt.a"]=PASS; EXPECT["fx.fxt.b"]=FAIL
fx_run_t() { local LG="$1" RCW="$2"; local B="$FXR/bin-t-$RCW-$(basename "$LG" .log)"
  printf '#!/usr/bin/env bash
cat "$STUB_LOG"
exit "$STUB_RC"
' > "$B"; chmod +x "$B"
  { printf '%s
' "$REAL_COMPLETION"; printf '%s
' "$3"; } > "$LG"
  unset 'OBS[fx.fxt.a]' 'OBS[fx.fxt.b]' 'EXPECT[fx.fxt.a]' 'EXPECT[fx.fxt.b]'
  OBS["fx.fxt.a"]=PASS; EXPECT["fx.fxt.a"]=PASS
  OBS["fx.fxt.witness_teardown_dependency_arm_refuses_golden_independent"]=PASS
  EXPECT["fx.fxt.witness_teardown_dependency_arm_refuses_golden_independent"]=PASS
  ( export STUB_LOG="$LG" STUB_RC="$RCW"; run_entry fx "$B" fxt "join-fixture.dag" "a,witness_teardown_dependency_arm_refuses_golden_independent" ) >/dev/null 2>&1
  local RC=$?
  unset 'OBS[fx.fxt.a]' 'OBS[fx.fxt.b]' 'EXPECT[fx.fxt.a]' 'EXPECT[fx.fxt.b]'
  OBS["fx.fxt.a"]=PASS; EXPECT["fx.fxt.a"]=PASS; OBS["fx.fxt.b"]=FAIL; EXPECT["fx.fxt.b"]=FAIL
  echo $RC ; }
fx_run() { local LG="$1" RCW="$2"; local B="$FXR/bin-$RCW-$(basename "$LG" .log)"
  printf '#!/usr/bin/env bash
cat "$STUB_LOG"
exit "$STUB_RC"
' > "$B"; chmod +x "$B"
  { printf '%s
' "$REAL_COMPLETION"; printf '%s
' "$3"; } > "$LG"
  ( export STUB_LOG="$LG" STUB_RC="$RCW"; run_entry fx "$B" fxt "join-fixture.dag" "a,b" ) >/dev/null 2>&1
  echo $? ; }
[ "$(fx_run "$FXR/f1.log"  0 "$(printf 'PASS a
PASS b
PASS foreign_fn')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture foreign PASS accepted"
[ "$(fx_run "$FXR/f2.log"  1 "$(printf 'PASS a
PASS b
FAIL foreign_fn')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture foreign FAIL accepted"
[ "$(fx_run "$FXR/f3.log"  0 "$(printf 'PASS a
PASS b
SKIPPED foreign_fn')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture foreign SKIPPED accepted"
[ "$(fx_run "$FXR/f4.log"  0 "$(printf 'PASS a
PASS b
PASS a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture duplicate PASS accepted"
[ "$(fx_run "$FXR/f5.log"  1 "$(printf 'PASS a
PASS b
FAIL b (boom)')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture reasoned FAIL accepted"
[ "$(fx_run "$FXR/f6.log"  2 "$(printf 'PASS a
UNOBSERVED b (resolve did not reach a verdict)')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture UNOBSERVED accepted"
[ "$(fx_run "$FXR/f7.log"  0 "$(printf 'PASS a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture missing name accepted"
[ "$(fx_run "$FXR/f8.log"  1 "$(printf 'PASS a
FAIL b')")" = 0 ] || die2 "REFUSED_SETUP: JOIN fixture valid mixed PASS/FAIL refused"
[ "$(fx_run "$FXR/f9.log"  0 "$(printf 'PASS a
PASS b
fail a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture lowercase fail a accepted (msg_b5063276 hole)"
[ "$(fx_run "$FXR/f10.log" 0 "$(printf 'PASS a
PASS b
 FAIL a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture leading-space FAIL accepted (msg_b5063276 hole)"
[ "$(fx_run "$FXR/f11.log" 0 "$(printf 'PASS a
PASS b
FAIL: a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture trailing-colon FAIL accepted (msg_b5063276 hole)"
[ "$(fx_run "$FXR/f12.log" 0 "$(printf 'PASS a
PASS b
Pass a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture mixed-case Pass accepted"
[ "$(fx_run "$FXR/f13.log" 0 "$(printf 'PASS a
PASS b
skipped b')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture lowercase skipped accepted"
[ "$(fx_run "$FXR/f14.log" 0 "$(printf 'PASS a
PASS b
ERROR b: panic')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture ERROR-colon accepted"
[ "$(fx_run "$FXR/f15.log" 0 "$(printf 'PASS a
PASS b
OOM Kills 0')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture unknown ALLCAPS line accepted"
[ "$(fx_run "$FXR/f16.log" 0 "$(printf 'PASS a\nPASS b\r')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture CR byte accepted"
[ "$(fx_run "$FXR/f17.log" 1 "$(printf 'resolve completed
claim_batch: 1 entry group(s), 56 witness(es) total
memory-cgroup-bind: max_memory_bytes=27487790694 usage_bytes=8GiB oom_kills=0
[resolve] dag/x.dag: 710182ms (1697 modules)
[witness] settled
PASS a
FAIL b')")" = 0 ] || die2 "REFUSED_SETUP: JOIN fixture diagnostic-noise mix refused (must pass)"
[ "$(fx_run "$FXR/f18.log" 1 "$(printf 'closure diagnostics in dag/x.dag: name Cons not found')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture closure-diagnostics marker not NO_OBSERVATION"
[ "$(fx_run "$FXR/f19.log" 0 "$(printf 'PASS a
PASS b
TIMEOUT: a')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture TIMEOUT-colon accepted (sleek-owl hole 1)"
[ "$(fx_run "$FXR/f20.log" 0 "$(printf 'PASS a
PASS b
ABORTED')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture bare ABORTED accepted (sleek-owl hole 2)"
[ "$(fx_run "$FXR/f21.log" 0 "$(printf 'PASS a
PASS b
IGNORED: b')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture IGNORED-colon accepted (sleek-owl hole 3)"
# portability fixtures (mgr msg_41a78785, silent-bear-373 finding): trailing-'t' roster name must be
# ACCEPTED (bash parameter-expansion bug class), and TAB-containing outcome-like lines must refuse.
[ "$(fx_run_t "$FXR/f22.log" 0 "$(printf 'PASS a
PASS witness_teardown_dependency_arm_refuses_golden_independent')")" = 0 ] || die2 "REFUSED_SETUP: JOIN fixture trailing-t roster name refused (portability trap)"
[ "$(fx_run "$FXR/f23.log" 1 "$(printf 'PASS a
FAIL b
IGNORED	x')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture IGNORED-TAB accepted (portability trap)"
[ "$(fx_run "$FXR/f24.log" 1 "$(printf 'PASS a
FAIL b
ZAPED	x')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture unknown-ALLCAPS-TAB accepted (portability trap)"
[ "$(fx_run "$FXR/f25.log" 0 "$(printf 'PASS a
PASS<TAB>b')")" = 2 ] || die2 "REFUSED_SETUP: JOIN fixture PASS-TAB-name accepted (portability trap)"
unset 'OBS[fx.fxt.a]' 'OBS[fx.fxt.b]' 'EXPECT[fx.fxt.a]' 'EXPECT[fx.fxt.b]'
echo "JOIN-LAW v3 self-test: 25/25 (21 + trailing-t roster accept + IGNORED-TAB/ZAPED-TAB/PASS-TAB refuses)"
  # REAL-FILE ROSTER FIXTURE (calm-crab-645 review): the declared `test fn` order of the pinned witness blob 40b14c98 at freeze time, as a synthetic file; the frozen ROSTER must pass roster_gate and its swap must refuse
  { for n in an_approve_verdict_decodes request_changes_carries_located_reasons malformed_verdicts_are_undecodable the_reconciliation_orders_changes_over_blocks_over_running_over_pending_over_approved a_killed_or_exited_stage_without_a_verdict_is_failed_not_refused every_review_criterion_is_grounded_in_the_design_argument the_brief_carries_the_criterion_the_diff_range_and_the_check_and_the_verdict_instruction_is_separate a_stated_divergence_is_admitted_and_an_unstated_one_requests_changes every_conformance_domain_carries_a_label_owns_and_boundary a_one_element_conformance_home_list_renders_as_before a_synthetic_conformance_domain_produces_exactly_one_design_section production_conformance_domains_appear_in_generated_design label_and_owns_change_projection_but_not_criterion_identity conformance_reader_text_does_not_affect_plan_revision every_conformance_domain_is_a_criterion_whose_brief_names_its_homes; do printf 'test fn %s() -> Bool {\n  true\n}\n' "$n"; done; } > "$R/pinned-decl.dag"
  roster_gate "$R/pinned-decl.dag" $ROSTER || die2 "REFUSED_SETUP: frozen ROSTER refused by roster_gate over the pinned declaration order"
  roster_gate "$R/pinned-decl.dag" $(printf '%s\n' $ROSTER | tac) && die2 "REFUSED_SETUP: swapped ROSTER accepted by roster_gate (order law dead)"
  [ "$(fx_run "$FXR/f30.log" 1 "$(printf 'PASS a
PASS b')")" = 2 ] || die2 "REFUSED_SETUP: rc-law fixture PASS/PASS with rc 1 accepted"
[ "$(fx_run "$FXR/f31.log" 0 "$(printf 'PASS a
PASS b')")" = 0 ] || die2 "REFUSED_SETUP: rc-law control PASS/PASS rc 0 refused"
[ "$(fx_run "$FXR/f32.log" 0 "$(printf 'reference_closure: x fail-closed
PASS a
PASS b')")" = 2 ] || die2 "REFUSED_SETUP: closure-marker with two PASS and rc 0 accepted"
echo "classifier fixtures: all held"; rm -rf "$R"
}
if [ "${1:-}" = --classifier-fixtures ]; then classifier_fixtures; exit 0; fi
# ---- REAL RUN (HEAD leg) ----
left(){ echo $(( START + BUDGET - $(date +%s) )); }
opcap(){ if [ -z "${START:-}" ]; then echo "$1"; return 0; fi; local l; l=$(left); [ "$l" -gt 0 ] || return 1; [ "$l" -lt "$1" ] && echo "$l" || echo "$1"; }
# phasecap <cap> -> cap, ONLY if the full cap + K + FINAL_ALLOW fits in the time left; else failure (refused before start)
phasecap(){ local l; l=$(( $(left) - FINAL_ALLOW - K )); [ "$l" -ge "$1" ] || return 1; echo "$1"; }
disk_ok(){ local a; a=$(df -Pk -- "$1" 2>/dev/null | awk 'NR==2 {print $4}'); case "$a" in ''|*[!0-9]*) return 1;; esac; [ "$a" -ge "$2" ]; }
[ "$#" = 1 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir> | --classifier-fixtures"
CGP="$(scope_gate "$(cat /proc/self/cgroup 2>/dev/null)" "$JOB")" || die2 "REFUSED_SETUP: /proc/self/cgroup is not exactly one 0:: line ending in /$JOB.scope"
CG="/sys/fs/cgroup$CGP"
BM="$(bounds_check "$CG")" || die2 "REFUSED_SETUP: bound mismatch: $BM"
out_absent "$1" || die2 "REFUSED_SETUP: output path $1 already exists in some form"
mkdir -- "$1" || die2 "mkdir $1 (exclusive)"
cd "$1" || die2 "cd $1"; R="$PWD"
START=$(date +%s)
declare -A EXPECT OBS; MISSING=0; DEVIATIONS=0
for N in $ROSTER; do EXPECT["A.w.$N"]=PASS; done
disk_ok "$R" $((25*1024*1024)) || die2 "REFUSED_SETUP: less than 25 GiB free under $R"
snap "start"
# FIXTURES FIRST (keen-otter ruling): any fixture failure exits 2 here, before fetch, build or any claim run.
classifier_fixtures > "$R/fixtures.log" 2>&1 || { cat "$R/fixtures.log"; die2 "REFUSED_SETUP: bounded fixtures failed before any costly phase"; }
git init -q wt && cd wt || die2 "git init"
C=$(phasecap "$FETCH_CAP_S") || die2 "deadline: fetch phase cannot fit"
timeout -k "$K" "$C" git fetch -q --depth 1 https://github.com/gunb-ai/gunbc "$PIN" "$BASEPIN" || die2 "fetch $PIN $BASEPIN failed or timed out"
git checkout -q "$PIN" || die2 "checkout failed"
[ "$(git rev-parse HEAD)" = "$PIN" ] || die2 "HEAD is not the pin"
[ "$(git rev-parse 'HEAD^{tree}')" = "$TREE" ] || die2 "tree is not the pinned tree"
tree_clean . || die2 "checkout is not clean"
[ "$(git hash-object "$WFILE")" = "$WBLOB" ] || die2 "witness blob is not the pinned blob"
[ "$(git hash-object "$SFILE")" = "$SBLOB" ] || die2 "source blob is not the pinned blob"
roster_gate "$WFILE" $ROSTER || die2 "roster is not an in-order single-occurrence subset of the declared test fn list in $WFILE"
snap "build"
C=$(phasecap "$BUILD_CAP_S") || die2 "deadline: build phase cannot fit"
(cd src/v1/stage0 && CARGO_TARGET_DIR="$R/target" timeout -k "$K" "$C" cargo build -j 32 --locked --release -p v1-compiler --bin claim_batch) > "$R/build.log" 2>&1
RC=$?; mem "after_build"; judge_oom "build"; [ "$RC" = 0 ] || die2 "build failed rc=$RC (see build.log)"
BIN="$R/target/release/claim_batch"; BS="$(hash_of "$BIN")" || die2 "cannot hash $BIN"
echo "claim_batch sha256=$BS head=$PIN tree=$TREE"
C=$(phasecap "$ENTRY_CAP_S") || die2 "deadline: run phase cannot fit"
ENTRY_CAP_S="$C"
run_entry A "$BIN" w "$WFILE" "$(echo $ROSTER | tr ' ' ',')"
tree_clean . || die2 "post-run tree is not clean"
[ "$(hash_of "$BIN")" = "$BS" ] || die2 "binary sha changed during the run"
[ "${#OBS[@]}" = 2 ] || die2 "not every roster name has an observed verdict (${#OBS[@]} of 2)"
echo "OBSERVED:"; for k in "${!OBS[@]}"; do echo "  $k = ${OBS[$k]} (expected ${EXPECT[$k]})"; done | sort
# ---- RED LEG ----
git cat-file -e "$BASEPIN^{commit}" 2>/dev/null || die2 "base commit not fetched"
git show "$BASEPIN:$SFILE" > "$R/base_role.dag" || die2 "base blob unreadable"
[ "$(git hash-object "$R/base_role.dag")" = "$BASEBLOB" ] || die2 "base blob is not the pinned base blob"
C=$(phasecap "$ENTRY_CAP_S") || die2 "deadline: red-leg run phase cannot fit"
ENTRY_CAP_S="$C"
EXPECT["B.w.a_killed_or_exited_stage_without_a_verdict_is_failed_not_refused"]=FAIL
EXPECT["B.w.the_reconciliation_orders_changes_over_blocks_over_running_over_pending_over_approved"]=PASS
cp "$R/base_role.dag" "$SFILE" || die2 "swap failed"
run_entry B "$BIN" w "$WFILE" "$(echo $ROSTER | tr ' ' ',')"   # any die2 inside = NO_OBSERVATION (exit 2); the worktree is the disposable copy under OUT
git checkout -- "$SFILE" || die2 "restore failed"
tree_clean . || die2 "tree not clean after restore"
[ "$(git rev-parse HEAD)" = "$PIN" ] && [ "$(git rev-parse 'HEAD^{tree}')" = "$TREE" ] || die2 "HEAD/tree changed across red leg"
[ "$(git hash-object "$SFILE")" = "$SBLOB" ] || die2 "source blob not restored"
[ "$(hash_of "$BIN")" = "$BS" ] || die2 "binary sha changed across red leg"
echo "JOIN_MISSING=$MISSING FAILS=$DEVIATIONS"
echo DONE
[ "$DEVIATIONS" -gt 0 ] && exit 1
exit 0
