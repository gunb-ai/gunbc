# script.sh (frozen copy for fierce-wren-19-r6-dff57af7, v6): acceptance grant set row R6, gunbc.
# HELD — NOT AUTHORIZED FOR EXECUTION. PREPARE ONLY: this file has not been RUN anywhere, and the live
# fabric store has not been contacted. Same hardened srv1 frame as the R4/R5 packet (template:
# wise-moth-707-rdpo's rdpo.sh, sha256 aaf32964184a30d66f6320de707bfb03975f0df683cccd1133dcf2d024ed9366,
# gunbc#15938 comment 6672252712); the gates, helpers, classifier, OOM contract and fixtures here are the
# SAME text as that packet's, so one review covers both.
#
# THE ONE CLAIM (row R6):
#   dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag
#     witness_inbox_on_authority_projects_on_live_corpus
# WHAT v5 ADDS: the completion gate read the WRONG STREAM (manager msg_1c5a519f, found by sleek-owl).
#   Everything below under "WHAT v4 ADDS" is kept from v4 unchanged (sleek-owl's three blockers
#   on v3 176234a92458c5a6, verified by sunny-bee-153); every v4 gate, fixture and control still runs.
#   (1) the launcher argv is calmcrab-udrv.sh <JOB> <RUNTIME_S> <MEMMAX_G> <MEMHIGH_G> <TASKS> <CPU%>
#       <STAGED_SCRIPT> <SCRIPT_SHA> OUT — seconds, GiB, %, the PACKET's own sha, literal OUT — and the R
#       ledger row is booked in GiB (keen-otter msg_c9d1df56); the bounds gate DERIVES its byte values from
#       those same GiB constants so the argv, the manifest and the gate cannot disagree.
#   (2) a completion-record gate on the leg (see COMPLETION RECORD in the classifier section).
#   (3) verdict exactness per name (see VERDICT RULES). R6 stays HELD: without a cleared execution-grant
#       reference the packet refuses with exit 2 before creating anything.#
# Expected: a bare PASS. A refusal is a FINDING for the grant set, not a runner failure: this claim fails
# closed on an unreadable tail, on history that will not replay, and on a refused placement.
#
# WHY IT IS ITS OWN PACKET, AND WHY IT IS HELD (the read-path audit, at this pin):
#   roadmap_acceptance_inbox_on_authority        dag/gunbc/roadmap/roadmap_acceptance_inbox.dag:140
#     -> roadmap_acceptance_event_history_load   dag/gunbc/roadmap/roadmap_authority.dag:5070
#        -> fabric_state_storage_binding         dag/gunbc/fabric/fabric_storage_binding.dag:36
#           -> roadmap_operator_close_history    dag/gunbc/roadmap/roadmap_operator_acceptance.dag:644
#              -> roadmap_operator_acceptance_history_load  same file:197
#                 -> roadmap_operator_acceptance_read       same file:132
#                    -> event_log_read_partition_with      dag/gunbc/fabric/fabric_event_log.dag:164
#                       -> fabric_storage_closure           dag/gunbc/fabric/fabric_storage_client.dag:193
# NO WRITES in that closure: the append primitive (event_log_append_with) is reached only through
# roadmap_operator_acceptance_close / _close_append, which this claim never touches; the closure walk and
# chain replay mutate nothing on the store.
# WHICH BRANCH A BARE srv1 CHECKOUT BINDS: `data prod_role_selection: ProdRoleSelection = ProdRoleHeldBy
# { deployment: srv1_daily_workspace_deployment }` is a committed constant (dag/gunbc/deployment_risk.dag:71)
# and srv1_live_dashboard_instance carries exactly that deployment
# (dag/gunbc/roadmap/roadmap_dashboard_instance.dag:1262-1264). So the prod role resolves,
# fabric_storage_placement returns FabricStoragePlaced, and fabric_state_storage_binding maps EVERY placed
# realization — including the host's own placed branch, whose door_socket it discards — to
# FabricStorageServed { <instance endpoint>/state }, which takes served_post: a live network round trip
# (fabric_storage_client.dag:203-207). The SEED is read only under FabricStorageBindingUnplaced, i.e. where
# the runtime identity store is missing; a REFUSED placement yields HistoryLoadRefused ->
# AcceptanceInboxRefused, so the claim's InboxLoaded arm fails — a FAIL, not a seed read. There is no
# seed-only form of this claim without editing the claim or the store binding, and no grant in this set
# authorizes that. A claim runner starting a NEW read of the live acceptance partition is not the "existing
# rostered reader" keen-otter's msg_3407a188 admits, so the row is HELD.
# THE HOLD IS ENFORCED, NOT DOCUMENTED: the second positional argument must be an execution-grant reference
# that has cleared msg_3407a188. Without it the packet refuses with exit 2 (v3 exit contract: every refusal
# is NO_OBSERVATION; there is no gate-only code) and creates NOTHING — no OUT, no clone. With one, it records
# the reference in OUT/grant-ref.txt. Clearing that gate is keen-otter's decision, not this packet's, and
# naming a reference that was never published is a violation of the grant set, not a workaround.
#
# STEPS (every one wrapped in timeout; the CAPPED SUM is this row's worst case):
#   setup   git init 300 s, shallow fetch of the pin 300 s, checkout 120 s                              =   720 s
#   build   cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch, 2400 s (a cold srv1
#           release build of this crate took ~2100 s, rdpo; nonzero / timeout / missing binary is 2)    =  2400 s
#   legs    one leg, 900 s, one claim_batch invocation carrying the single claim                        =   900 s
#   CAPPED SUM = 720 + 2400 + 900 = 4020 s, an admitted ceiling, not a prediction of the wall.
# WALL ACCOUNTING: scope RuntimeMaxSec 4800 s (the capped sum 4020 s plus a 780 s margin); the driver
#   service calmcrab-drv-<JOB>.service RuntimeMaxSec 5400 s, sized FROM the scope (4800 + 600 s for staging,
#   the ledger write and teardown). If the launcher books different numbers, the bounds gate refuses rather
#   than adapting to them.
#
# BOOKED BOUNDS (identical to the R4/R5 row: the same -j 32 build of the same crate on the same host):
# Bounds are booked in the LAUNCHER'S units (GiB, seconds, CPU %, tasks) and the byte values the cgroup
# files carry are DERIVED from them here, so the launcher argv, the manifest and the gate cannot drift.
# cpu.max's period is the default 100000 us, so the quota is CPU_PCT/100 * 100000 = CPU_PCT*1000 us.
RUNTIME_S=4800; DRIVER_S=5100; LAUNCHER_CAP=7200; N_LEGS=1; MEMMAX_G=32; MEMHIGH_G=28; TASKS_MAX=4096; CPU_PCT=3200
BOOKED="memory.max=$((MEMMAX_G*1073741824)) memory.high=$((MEMHIGH_G*1073741824)) memory.swap.max=0 pids.max=$TASKS_MAX cpu.max=$((CPU_PCT*1000))_100000"
#
# (The BOOKED string is the same as the R4/R5 row's: memory.max 32G, memory.high 28G UNMEASURED,
#  memory.swap.max 0, pids.max 4096 (threads), cpu.max "3200000 100000" paired with -j 32.)
#
# LAUNCHER (calmcrab-udrv.sh, v4 form) — no bare systemd-run. Full argv, in the launcher's order:
#   calmcrab-udrv.sh <JOB> <RUNTIME_S> <MEMMAX_G> <MEMHIGH_G> <TASKS> <CPU%> <STAGED_SCRIPT> <SCRIPT_SHA> OUT
# For THIS row the call is literally:
#   calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> <v6 packet sha256> OUT
# Units are the launcher's: RUNTIME in SECONDS (the scope's wall, sized from the capped per-step timeouts
# below), memory in GiB, CPU with the %% sign, SCRIPT_SHA is the PACKET'S OWN sha256 (NOT the source commit),
# and OUT is passed LITERALLY — the script derives JOB as basename(dirname(OUT)), so OUT must be
# $HOME/runs/fierce-wren-19-r6-dff57af7/out. MEMMAX_G/MEMHIGH_G/TASKS/CPU%% are the same numbers the bounds gate below re-derives.
# LEDGER ROWS (R/S/D), three of them, named by the launcher's convention:
#   fierce-wren-19-r6-dff57af7-res   cap 32 (GiB — the R row is booked in GiB, not bytes, per keen-otter msg_c9d1df56)
#   fierce-wren-19-r6-dff57af7-drv   cap 1
#   fierce-wren-19-r6-dff57af7       cap 0
# UNITS: fierce-wren-19-r6-dff57af7.scope (the row's own scope; the scope gate requires this process's cgroup path to END in
#   "/fierce-wren-19-r6-dff57af7.scope") and calmcrab-drv-fierce-wren-19-r6-dff57af7.service (the driver service). R is written before the scope
#   starts; S starts the scope and the driver service under those bookings; D writes the row's exit code
#   back to the ledger. This script books nothing and writes no ledger: it REFUSES (2) when the environment
#   it was started in does not match the booking.
#
# EXIT CONTRACT (v3 item 4, identical to the R4/R5 packet plus the hold). THREE codes, no others:
#   0  the claim printed a bare PASS and the runner's own exit agreed (expectation held)
#   1  the claim printed a bare FAIL (expectation FALSIFIED — a finding for the grant set)
#   2  NO_OBSERVATION: the HELD refusal when no cleared execution-grant reference is named, and every other
#      usage, setup, scope, bounds, clone, checkout, tree, blob, name, build, timeout, OOM, counter,
#      classifier or return-code failure. The hold refusal happens BEFORE anything is created.

set -u

J=fierce-wren-19-r6-dff57af7
U=https://github.com/gunb-ai/gunbc
H=dff57af790f923066d61721e89ff751a91021278; T=356f2894a60edda51cdbe79879b2fb3b7aea75c4
E6=dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag; B6=969b980b189dafa81df4045f62f7d2e70dcc50ad

# the one claim name (a space-separated list of one: classification, the names gate, leg_state)
N6="witness_inbox_on_authority_projects_on_live_corpus"
# the SAME name as one comma-separated --functions argument (the parser does csv.split(',') then trims)
csv() { local s; s=$(printf '%s,' $1); printf '%s' "${s%,}"; }
C6=$(csv "$N6")

T_SETUP_CLONE=300; T_SETUP_FETCH=300; T_SETUP_CHECKOUT=120; T_BUILD=2400; T_LEG=900
# the row's CAPPED SUM: one clone + one shallow fetch + one checkout + one build + N_LEGS legs
WALL_SUM=$(( T_SETUP_CLONE + T_SETUP_FETCH + T_SETUP_CHECKOUT + T_BUILD + N_LEGS*T_LEG ))

# FROZEN COMMAND STRINGS: what --predict-only prints (bytes + sha256 over the literal string) and what the
# run executes are the SAME variables. Expansion points named in the manifest: $OUT (the launcher's evidence
# dir, the one positional argument) and $BIN6 (the runner built inside the pinned clone); the origin, the
# pin, the entry path and the CSV are already expanded here, at definition time.
CMD_CLONE='git init -q "$OUT/wt-r6"'
CMD_FETCH='git -C "$OUT/wt-r6" fetch -q --depth 1 https://github.com/gunb-ai/gunbc dff57af790f923066d61721e89ff751a91021278'
CMD_CHECKOUT='git -C "$OUT/wt-r6" checkout -q --detach FETCH_HEAD'
CMD_BUILD='cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch'
CMD_LEG_R6='"$BIN6" --source-root dag --source-root src/v1 --source-root src/v2 --entry "dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag" --functions "witness_inbox_on_authority_projects_on_live_corpus"'

# ---------------------------------------------------------------- gates and helpers (tested by --classifier-fixtures)
scope_gate() { # $1=/proc/self/cgroup content $2=JOB; prints the cgroup path, or fails
  local n; n=$(printf '%s\n' "$1" | grep -c .)
  [ "$n" = 1 ] && [ -n "$2" ] || return 1
  case "$1" in 0::/*) ;; *) return 1;; esac
  case "${1#0::}" in */"$2".scope) printf '%s' "${1#0::}";; *) return 1;; esac; }
bounds_check() { # $1=cgroup dir; prints the first mismatch and fails, or prints nothing
  local w k v got
  for w in $BOOKED; do k=${w%%=*}; v=${w#*=}; v=${v//_/ }
    [ -r "$1/$k" ] || { echo "$k unreadable"; return 1; }
    got=$(cat "$1/$k" 2>/dev/null) || { echo "$k unreadable"; return 1; }
    [ "$got" = "$v" ] || { echo "$k is '$got', booked '$v'"; return 1; }
  done; }
out_absent() { ! { [ -e "$1" ] || [ -L "$1" ]; }; }
bin_sha() { local o h; o=$(sha256sum -- "$1" 2>/dev/null) || return 1; h=${o%% *}
  [[ "$h" =~ ^[0-9a-f]{64}$ ]] && [ "$(printf '%s\n' "$o" | grep -c .)" = 1 ] || return 1; printf '%s' "$h"; }
# OOM PARSE: one read per sample; each key exactly once as a whole line ^key [0-9]+$; anything else UNKNOWN.
oom_parse() { # $1=snapshot text $2=key
  local all ok; all=$(printf '%s\n' "$1" | awk -v k="$2" '$1==k' | wc -l | tr -d ' '); ok=$(printf '%s\n' "$1" | grep -cE "^$2 [0-9]+\$")
  if [ "$all" = 1 ] && [ "$ok" = 1 ]; then printf '%s\n' "$1" | grep -E "^$2 [0-9]+\$" | cut -d' ' -f2; else echo UNKNOWN; fi; }
oom_key() { [ -r "$1" ] || { echo UNKNOWN; return; }; local t; t=$(cat "$1" 2>/dev/null) || { echo UNKNOWN; return; }; oom_parse "$t" "$2"; }
oom_pair() { [ -r "$1" ] || { echo UNKNOWN/UNKNOWN; return; }; local t; t=$(cat "$1" 2>/dev/null) || { echo UNKNOWN/UNKNOWN; return; }
  echo "$(oom_parse "$t" oom_kill)/$(oom_parse "$t" oom)"; }
oom_verdict() { case "$1" in ''|*[!0-9]*) echo UNKNOWN; return;; esac; case "$2" in ''|*[!0-9]*) echo UNKNOWN; return;; esac
  if [ "$2" -lt "$1" ]; then echo COUNTER_INCOHERENT; elif [ "$2" -gt "$1" ]; then echo OOM_KILLED; else echo none; fi; }
oom_pair_verdict() { local a=$(oom_verdict "${1%/*}" "${2%/*}") b=$(oom_verdict "${1#*/}" "${2#*/}")
  if [ "$a" = none ] && [ "$b" = none ]; then echo none; elif [ "$a" = UNKNOWN ] || [ "$b" = UNKNOWN ]; then echo UNKNOWN
  elif [ "$a" = COUNTER_INCOHERENT ] || [ "$b" = COUNTER_INCOHERENT ]; then echo COUNTER_INCOHERENT; else echo OOM_KILLED; fi; }
oom_snap() { # $1=event file $2=tag; publishes the raw bytes by exclusive create, THEN prints KILL/OOM
  local t f n; n=$(( $(find "$MEMDIR" -maxdepth 1 -type f 2>/dev/null | wc -l) + 1 )); f=$MEMDIR/$(printf '%03d' $n)-$(date -u +%Y%m%dT%H%M%S.%NZ)-$2.txt
  if [ -r "$1" ] && t=$(cat "$1" 2>/dev/null); then
    if ( set -C; printf '%s\n' "$t" > "$f" ) 2>/dev/null; then echo "$(oom_parse "$t" oom_kill)/$(oom_parse "$t" oom)"
    else echo UNKNOWN/UNKNOWN; fi
  else ( set -C; : > "$f.unreadable" ) 2>/dev/null; echo UNKNOWN/UNKNOWN; fi; }
# CLAIM CLASSIFIER (v4; the contract is identical to the R4/R5 packet):
#   VERDICT RULES (v4, sleek-owl blocker 3). Only a BARE `^PASS <name>$` counts as held and only a BARE
#   `^FAIL <name>$` counts as falsified. Each expected name must also have EXACTLY ONE outcome line of ANY
#   keyword -- PASS/FAIL/UNOBSERVED/PANICKED, bare or qualified: zero outcome lines for a name, or two, is
#   NO_OBSERVATION, and a QUALIFIED line (`FAIL <fn> (reason)`) is NO_OBSERVATION, never a falsification.
#   Anything else is also NO_OBSERVATION: a contradiction, an indented line, a name that is not one of this
#   row's, an empty or missing log, or a timeout (rc 124).
# COMPLETION RECORD (v4, sleek-owl blocker 2, keen-otter msg_a9e94d36). The runner's exit 1 ALSO covers
#   setup/resolve refusals, so bare verdict lines alone cannot prove a leg completed. claim_batch's
#   ProcessPartition::render emits EXACTLY ONE `[process-partition] total_ms=...` record per process, on the
#   Ok path only. Per leg the gate therefore does TWO stages: (a) count lines with the PREFIX
#   `^[process-partition] total_ms` and require exactly one -- a single-stage filter-and-count cannot see a
#   malformed competitor sitting next to a good one; (b) test the SHAPE `^\[process-partition\] total_ms=[0-9]+( |$)`
#   on that one line. Zero, two, or one valid plus one malformed is NO_OBSERVATION. Whole-line anchoring
#   (`...total_ms=[0-9]+$`) is WRONG: render pushes the span fields unconditionally, and that exact mistake
#   is what spent #15695's G1 attempt (15/15 bare PASS and still NO_OBSERVATION). The neighbour line
#   `[process-partition-exit] run_returned_ms=... main_end_ms=...` does NOT start with `[process-partition]`
#   and must not be counted -- hence the `]` in the prefix.
#   WHICH STREAM, checked at the pin: that record goes to STDERR, not stdout. The pinned source has exactly
#   one emission of the record -- the single eprintln!("{}", partition.render(&timings)) call site in the
#   run() path of src/v1/stage0/src/bin/claim_batch.rs (blob 12a71786bd92f683dbd0aec2ae5d6f426e651a44
#   -- the same bytes at dff57af7 and at db56530f), while every PASS/FAIL verdict goes to stdout. So the
#   completion gate reads the leg stderr capture (logs/<leg>.err) and the classifier reads the leg stdout
#   (<leg>.out). Pointing that gate at stdout reads 0 records on every leg and NO_OBSERVATIONs a full-green
#   row: the #15695 G1 shape by another route. The pinned source is therefore blob-gated like every other
#   input and re-read for its render site (producer_gate), and the fixtures drive THIS packet's own leg() with
#   a stub standing in for the binary: record on stderr plus verdict on stdout is an observation, record only
#   on stdout is not, and two records on stderr is not.
wall_gate() { # $1=capped per-step sum $2=scope RuntimeMaxSec $3=installed launcher cap $4=driver RuntimeMaxSec
  local s=$1 r=$2 cap=$3 d=$4
  [ "$r" -le "$cap" ] || { echo "RUNTIME_S $r exceeds the installed calmcrab-udrv v4 preflight cap of $cap; the launcher would refuse before any scope exists (PRE_SCOPE_REFUSAL, the #15695 diag-v2 shape at 11700)"; return; }
  [ "$s" -le "$r" ] || { echo "the capped per-step sum $s s exceeds the scope RuntimeMaxSec $r; a cut-off leg is NO_OBSERVATION"; return; }
  [ "$d" = "$((r+300))" ] || { echo "the driver service must be RuntimeMaxSec $((r+300)), the scope's $r plus 300 s; the packet says $d"; return; }
  echo OK; }

claim_classify() { # $1=output file $2=rc $3=name -> HELD | FAILED | NO_OBSERVATION
  local f=$1 rc=$2 n=$3 p q
  [ -f "$f" ] || { echo NO_OBSERVATION; return; }
  [ "$rc" = 124 ] && { echo NO_OBSERVATION; return; }
  case "$rc" in ''|missing|*[!0-9]*) echo NO_OBSERVATION; return;; esac
  # exactly one outcome line for this name, of ANY keyword (bare or qualified); zero or two -> no observation
  any=$(grep -cE "^(PASS|FAIL|UNOBSERVED|PANICKED) $n( |$)" "$f" 2>/dev/null)
  [ "$any" = 1 ] || { echo NO_OBSERVATION; return; }
  p=$(grep -c "^PASS $n\$" "$f" 2>/dev/null); q=$(grep -c "^FAIL $n\$" "$f" 2>/dev/null)
  if [ "$p" = 1 ] && [ "$q" = 0 ] && [ "$rc" = 0 ]; then echo HELD; return; fi
  if [ "$p" = 0 ] && [ "$q" = 1 ] && [ "$rc" = 1 ]; then echo FAILED; return; fi
  # a qualified FAIL, an UNOBSERVED or a PANICKED line, or a bare line whose process exit disagrees, is not
  # an observation at all: the claim was attempted but its outcome was not reported in a countable form.
  echo NO_OBSERVATION; }
# COMPLETION RECORD GATE: two stages on purpose (see the header). Prefix count first, then shape on that one.
PP_PREFIX='^[[]process-partition[]] total_ms'
PP_SHAPE='^[[]process-partition[]] total_ms=[0-9]+( |$)'
PROD=src/v1/stage0/src/bin/claim_batch.rs
PRODBLOB=12a71786bd92f683dbd0aec2ae5d6f426e651a44
pp_check() { # $1=output file -> "OK <the record>" or the reason it is not an observation
  local f=$1 n line
  [ -n "$PP_PREFIX" ] && [ -n "$PP_SHAPE" ] || { echo "the completion predicates are empty; a vacuous pattern would count every line"; return; }
  [ -r "$f" ] || { echo "unreadable leg output"; return; }
  n=$(grep -c "$PP_PREFIX" "$f" 2>/dev/null)
  [ "$n" = 1 ] || { echo "completion records: $n matched the prefix, exactly 1 is required"; return; }
  line=$(grep -m1 "$PP_PREFIX" "$f" 2>/dev/null)
  printf '%s\n' "$line" | grep -qE "$PP_SHAPE" || { echo "the single completion record is malformed: $line"; return; }
  echo "OK $line"; }
leg_state() { # $1=classes file ("name CLASS" lines) $2=name list; AS_EXPECTED | FALSIFIED | NOOBS
  local n c
  for n in $2; do
    c=$(grep -c "^$n " "$1" 2>/dev/null); [ "$c" = 1 ] || { echo NOOBS; return; }
    c=$(grep "^$n " "$1" | cut -d' ' -f2)
    [ "$c" = NO_OBSERVATION ] && { echo NOOBS; return; }
  done
  [ "$(grep -c ' NO_OBSERVATION$' "$1" 2>/dev/null)" = 0 ] || { echo NOOBS; return; }
  if grep -q ' FAILED$' "$1" 2>/dev/null; then echo FALSIFIED; else echo AS_EXPECTED; fi; }
# lines that LOOK like verdicts must all be exact bare lines for names of this leg; a qualified line, an
# indented one, or a name this leg did not ask for is NO_OBSERVATION for the whole leg.
stray_verdict_lines() { # $1=output file $2=name list -> count of verdict-shaped lines that are not exact bare lines for these names
  local f=$1 names=$2 tot exact n
  tot=$(grep -cE '^(PASS|FAIL|UNOBSERVED|PANICKED) ' "$f" 2>/dev/null || echo 0)
  exact=$(for n in $names; do grep -cE "^(PASS|FAIL) $n\$" "$f" 2>/dev/null; done | awk '{s+=$1} END{print s+0}')
  [ "$tot" = "$exact" ] && echo 0 || echo "stray=$tot exact=$exact"; }
names_gate() { # $1=checkout dir $2=entry file $3=name list: every name must be a `test fn` in the entry file
  local w=$1 e=$2 n miss=""
  [ -r "$w/$e" ] || { echo "entry file $e unreadable"; return 1; }
  for n in $3; do grep -q "^test fn $n(" "$w/$e" || miss="$miss $n"; done
  [ -z "$miss" ] || { echo "not a test fn in $e:$miss"; return 1; }; }
tm() { grep -hE 'Maximum resident|Elapsed' "$1" 2>/dev/null | tr -s ' ' | tr '\n' ' '; }

grant_gate() { # $1 = the second positional argument: executing a HELD row needs a grant reference
  local g=${1-}
  case "$g" in
    "") echo "row R6 is HELD behind keen-otter's msg_3407a188: a claim runner starting a NEW read of the live acceptance partition is not the existing rostered reader that message admits. Run as: script.sh <OUT> <execution-grant-ref>, where the reference is the one keen-otter published when that gate was cleared; this packet cannot decide that for itself."; return 1;;
    --*) echo "row R6: the second argument must be an execution-grant reference, not the flag '$g'"; return 1;;
    *[![:graph:]]*) echo "row R6: an execution-grant reference must be one printable token, not blank or padded"; return 1;;
  esac
  printf '%s' "$g"; }

# ---- three run-path helpers lifted ABOVE the fixture controls, on purpose: bash only knows a function
# ---- after the line that defines it, and fixture part 2 drives the real leg() with a stub standing in for the
# ---- claim_batch binary, so the fixtures cannot drift from the wiring they are meant to check. Nothing here
# ---- runs at parse time; the run path calls these further down, exactly as in v4. The clone and blob-gate
# ---- helpers were left in place so the gate-order control keeps its v4 meaning.
nox() { echo "$1" >> "$O"; echo "VERDICT=NO_OBSERVATION reason=$2" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"; echo "NO_OBSERVATION: $1 (reason=$2)" >&2; exit 2; }

producer_gate() { # $1 = a checked-out claim_batch.rs: the completion record MUST be rendered to stderr
  local f=$1 n line
  [ -r "$f" ] || { echo "the pinned claim_batch.rs is unreadable: $f"; return; }
  n=$(grep -cF 'partition.render(&timings)' "$f" 2>/dev/null)
  [ "$n" = 1 ] || { echo "expected exactly one completion-record render site, found $n in $f"; return; }
  line=$(grep -m1 -F 'partition.render(&timings)' "$f")
  case "$line" in
    *'eprintln!("{}", partition.render(&timings));'*) echo OK ;;
    *) echo "the completion record is NOT emitted to stderr (eprintln) at the render site: $line"; return ;;
  esac; }

leg() { # $1=leg name $2=CMD variable name $3=name list
  local L=$1 rc k0 k1 LO st cl sv
  cmd=${2}
  k0=$(oom_snap "$EV" "$L-leg-before")
  timeout "$T_LEG" bash -c "$2" > "$OUT/$L.out" 2> "$OUT/logs/$L.err"; rc=$?
  k1=$(oom_snap "$EV" "$L-leg-after"); LO=$(oom_pair_verdict "$k0" "$k1"); LASTOOM=$k1
  echo "== $L rc=$rc oom=$LO ($k0 -> $k1) $(tm "$OUT/logs/$L.err")" >> "$O"
  if [ "$LO" != none ]; then
    echo "$L oom=$LO" >> "$O"; nox "leg $L: memory.events moved $k0 -> $k1" "$([ "$LO" = OOM_KILLED ] && echo leg-oom || echo counter-incoherent)"
  fi
  sv=$(stray_verdict_lines "$OUT/$L.out" "$3")
  if [ "$sv" != 0 ]; then
    grep -E '^(PASS|FAIL|UNOBSERVED|PANICKED) ' "$OUT/$L.out" 2>/dev/null | head -6 | cut -c1-300 | sed 's/^/  | /' >> "$O"
    nox "leg $L is not exactly bare verdict lines: $sv" classifier
  fi
  : > "$OUT/$L.classes"
  pc=$(pp_check "$OUT/logs/$L.err")
  echo "== $L completion_record: $pc" >> "$O"
  # a leg without exactly one well-formed completion record never finished the producer's reporting path,
  # so its verdict lines are not an observation; leg_state reads this line and the leg becomes NOOBS (exit 2).
  [ "${pc%% *}" = OK ] || echo "completion NO_OBSERVATION" >> "$OUT/$L.classes"
  for cl in $3; do echo "$cl $(claim_classify "$OUT/$L.out" "$rc" "$cl")" >> "$OUT/$L.classes"; done
  cat "$OUT/$L.classes" | sed 's/^/  /' >> "$O"
  st=$(leg_state "$OUT/$L.classes" "$3")
  echo "== $L state=$st" >> "$O"; echo "$st" > "$OUT/$L.state"; }

# ---------------------------------------------------------------- fixture mode: run BEFORE any probe is spent
if [ "${1:-}" = "--classifier-fixtures" ]; then
  T=$(mktemp -d); okn=0; badn=0
  ck() { if [ "$3" = "$2" ]; then echo "FIXTURE PASS $1 -> $3"; okn=$((okn+1)); else echo "FIXTURE FAIL $1 expected $2 got $3"; badn=$((badn+1)); fi; }
  L() { printf '%s\n' "$1"; }
  # the classifier, in both directions, on the runner's real line shapes
  L "PASS witness_inbox_stale_live_receipt_refuses" > $T/p;  ck bare-pass HELD "$(claim_classify $T/p 0 witness_inbox_stale_live_receipt_refuses)"
  L "FAIL witness_inbox_stale_live_receipt_refuses" > $T/f;  ck bare-fail FAILED "$(claim_classify $T/f 1 witness_inbox_stale_live_receipt_refuses)"
  ck pass-wrong-rc NO_OBSERVATION "$(claim_classify $T/p 1 witness_inbox_stale_live_receipt_refuses)"
  ck fail-wrong-rc NO_OBSERVATION "$(claim_classify $T/f 0 witness_inbox_stale_live_receipt_refuses)"
  ck all-pass-but-partition-refused NO_OBSERVATION "$(claim_classify $T/p 1 witness_inbox_stale_live_receipt_refuses)"
  ck timeout-124 NO_OBSERVATION "$(claim_classify $T/p 124 witness_inbox_stale_live_receipt_refuses)"
  ck signal-137 NO_OBSERVATION "$(claim_classify $T/p 137 witness_inbox_stale_live_receipt_refuses)"
  ck missing-rc NO_OBSERVATION "$(claim_classify $T/p missing witness_inbox_stale_live_receipt_refuses)"
  ck empty-rc NO_OBSERVATION "$(claim_classify $T/p '' witness_inbox_stale_live_receipt_refuses)"
  ck no-such-file NO_OBSERVATION "$(claim_classify $T/nothing-here 0 witness_inbox_stale_live_receipt_refuses)"
  ck name-not-in-log NO_OBSERVATION "$(claim_classify $T/p 0 witness_inbox_other_thing)"
  { L "PASS witness_inbox_stale_live_receipt_refuses"; L "PASS witness_inbox_stale_live_receipt_refuses"; } > $T/d
  ck duplicate-line NO_OBSERVATION "$(claim_classify $T/d 0 witness_inbox_stale_live_receipt_refuses)"
  { L "PASS witness_inbox_stale_live_receipt_refuses"; L "FAIL witness_inbox_stale_live_receipt_refuses"; } > $T/c
  ck contradictory NO_OBSERVATION "$(claim_classify $T/c 1 witness_inbox_stale_live_receipt_refuses)"
  L "  PASS witness_inbox_stale_live_receipt_refuses" > $T/i; ck indented NO_OBSERVATION "$(claim_classify $T/i 0 witness_inbox_stale_live_receipt_refuses)"
  for q in 'FAIL witness_x (runtime error [E]: boom)' 'FAIL witness_x (returned ProcessExit::ExitFailure, code 3: no)' 'UNOBSERVED witness_x (route gap)' 'PANICKED witness_x (the host unwound)'; do
    L "$q" > $T/q; ck "qualified-$(printf '%s' "$q" | cut -c1-4)" NO_OBSERVATION "$(claim_classify $T/q 2 witness_x)"
    ck "qualified-stray-$(printf '%s' "$q" | cut -c1-4)" "stray=1 exact=0" "$(stray_verdict_lines $T/q 'witness_x')"
  done
  # stray-line detector: a qualified line among clean ones must be caught per leg, not per name
  { L "PASS a_holds"; L "FAIL b_holds (runtime error [E]: boom)"; L "PASS c_holds"; } > $T/s
  ck stray-detected "stray=3 exact=2" "$(stray_verdict_lines $T/s 'a_holds b_holds c_holds')"
  { L "PASS a_holds"; L "PASS b_holds"; L "PASS c_holds"; } > $T/s2
  ck stray-clean 0 "$(stray_verdict_lines $T/s2 'a_holds b_holds c_holds')"
  L "PASS a_holds" > $T/s3; ck stray-extra-name "stray=1 exact=0" "$(stray_verdict_lines $T/s3 'b_holds')"
  # leg state
  mk() { : > $T/$1; for x in a b c; do echo "$x HELD" >> $T/$1; done; }
  mk l1; ck leg-as-expected AS_EXPECTED "$(leg_state $T/l1 'a b c')"
  mk l2; sed -i 's/^b HELD/b FAILED/' $T/l2; ck leg-falsified FALSIFIED "$(leg_state $T/l2 'a b c')"
  mk l3; sed -i 's/^b HELD/b NO_OBSERVATION/' $T/l3; ck leg-noobs NOOBS "$(leg_state $T/l3 'a b c')"
  mk l4; sed -i '/^c /d' $T/l4; ck leg-missing-name NOOBS "$(leg_state $T/l4 'a b c')"
  mk l5; echo "d HELD" >> $T/l5; ck leg-extra-name-recorded AS_EXPECTED "$(leg_state $T/l5 'a b c')"
  mk l6; sed -i 's/^a HELD/a HELD/; s/^b HELD/b FAILED/; s/^c HELD/c NO_OBSERVATION/' $T/l6; ck leg-noobs-beats-falsified NOOBS "$(leg_state $T/l6 'a b c')"
  # names gate, against a real entry-file shape
  mkdir -p $T/wt/dag; printf 'module x\ntest fn a_holds() -> Bool {\n  true\n}\ntest fn c_holds() -> Bool {\n  true\n}\n' > $T/wt/dag/e.dag
  names_gate $T/wt dag/e.dag 'a_holds c_holds' && ck names-ok ok ok || ck names-ok ok refused
  names_gate $T/wt dag/e.dag 'a_holds b_holds' >/dev/null && ck names-typo refused accepted || ck names-typo refused refused
  names_gate $T/wt dag/nope.dag 'a_holds' >/dev/null && names_gate $T/wt dag/nope.dag 'a_holds'; ck names-missing-entry refused refused
  # the CSV the legs pass, and its arity (the parser splits on ',' and trims)
  ck csv-r6 1 "$(printf '%s' "$C6" | awk -F',' '{print NF}')"
  ck csv-is-the-one-name "$N6" "$C6"
  ck csv-no-spaces 0 "$(printf '%s' "$C6" | grep -c ' ')"
  ck names-match-csv 0 "$( ( IFS=','; set -f; for x in $C6; do case " $N6 " in *" $x "*) ;; *) echo bad;; esac; done ) | grep -c bad)"
  ck name-shape 0 "$(printf '%s\n' $N6 | grep -cvE '^[a-z0-9_]+$')"
  grant_gate "" >/dev/null && ck grant-empty refused accepted || ck grant-empty refused refused
  grant_gate "   " >/dev/null && ck grant-blank refused accepted || ck grant-blank refused refused
  grant_gate "--yes" >/dev/null && ck grant-flaglike refused accepted || ck grant-flaglike refused refused
  grant_gate "keen-otter-msg_3407a188-cleared-2026-11-06" >/dev/null && ck grant-named accepted accepted || ck grant-named accepted refused
  # OOM: one read, exact keys, publish-before-parse, rise/fall/unknown
  mkdir -p $T/mem; MEMDIR=$T/mem
  printf 'low 0\nhigh 3\nmax 1\noom 0\noom_kill 0\nfail 0\ndiscard_oom 0\n' > $T/ev
  ck snap-parsed 0/0 "$(oom_snap $T/ev t)"
  ck snap-raw-published 1 "$(ls $T/mem | grep -c -- '-t.txt$')"
  ck snap-unreadable UNKNOWN/UNKNOWN "$(oom_snap $T/nope u)"
  ck snap-missing-memdir UNKNOWN/UNKNOWN "$(MEMDIR=$T/no-such-dir; oom_snap $T/ev w)"
  mkdir $T/ro; chmod 555 $T/ro; ck snap-readonly-memdir UNKNOWN/UNKNOWN "$(MEMDIR=$T/ro; oom_snap $T/ev w)"; chmod 755 $T/ro
  ck oom-single 0 "$(oom_parse 'oom 0' oom)"; ck oom-kill-single 3 "$(oom_parse 'oom_kill 3' oom_kill)"
  ck oom-extra-field UNKNOWN "$(oom_parse 'oom 0 garbage' oom)"
  ck oom-duplicate UNKNOWN "$(oom_parse "$(printf 'oom 0\noom bad')" oom)"
  ck oom-duplicate-valid UNKNOWN "$(oom_parse 'oom 0
oom 0' oom)"
  ck oom-bare-dup UNKNOWN "$(oom_parse "$(printf 'oom 0\noom')" oom)"
  ck oom-tab-dup UNKNOWN "$(oom_parse "$(printf 'oom 0\noom\tbad')" oom)"
  ck oom-tab-only UNKNOWN "$(oom_parse "$(printf 'oom\t0')" oom)"
  ck oom-kill-not-oom 0 "$(oom_parse 'oom_kill 3
oom 0' oom)"
  ck oom-key-absent UNKNOWN "$(oom_parse 'low 0' oom)"
  ck oom-pair-one-read 0/0 "$(oom_pair $T/ev)"
  ck oom-pair-unreadable UNKNOWN/UNKNOWN "$(oom_pair $T/nope)"
  ck oom-rise OOM_KILLED "$(oom_verdict 0 1)"; ck oom-fall COUNTER_INCOHERENT "$(oom_verdict 1 0)"
  ck oom-same none "$(oom_verdict 2 2)"; ck oom-unknown UNKNOWN "$(oom_verdict 0 x)"
  ck oompair-none none "$(oom_pair_verdict 0/0 0/0)"; ck oompair-oom OOM_KILLED "$(oom_pair_verdict 0/0 0/1)"
  ck oompair-fall COUNTER_INCOHERENT "$(oom_pair_verdict 1/0 0/0)"; ck oompair-unknown UNKNOWN "$(oom_pair_verdict 0/UNKNOWN 0/0)"
  # bounds gate: every booked value must match exactly
  mkdir -p $T/cg; printf '34359738368\n' > $T/cg/memory.max; printf '30064771072\n' > $T/cg/memory.high
  printf '0\n' > $T/cg/memory.swap.max; printf '4096\n' > $T/cg/pids.max; printf '3200000 100000\n' > $T/cg/cpu.max
  bounds_check $T/cg >/dev/null && ck bounds-ok ok ok || ck bounds-ok ok refused
  for k in memory.max memory.high memory.swap.max pids.max cpu.max; do
    printf 'NOTTHEBOOKEDVALUE\n' > $T/cg/$k; bounds_check $T/cg >/dev/null && ck "bounds-mismatch-$k" refused accepted || ck "bounds-mismatch-$k" refused refused
    printf '34359738368\n' > $T/cg/$k
  done
  printf 'max\n' > $T/cg/memory.max; bounds_check $T/cg >/dev/null && ck bounds-unlimited refused accepted || ck bounds-unlimited refused refused
  printf '34359738368\n' > $T/cg/memory.max; mv $T/cg/cpu.max $T/cg/cpu.gone
  bounds_check $T/cg >/dev/null && ck bounds-unreadable refused accepted || ck bounds-unreadable refused refused
  mv $T/cg/cpu.gone $T/cg/cpu.max
  # scope gate
  ck scope-ok /user.slice/user-1000.slice/x/fierce-wren-19-r6-dff57af7.scope "$(scope_gate "0::/user.slice/user-1000.slice/x/fierce-wren-19-r6-dff57af7.scope" fierce-wren-19-r6-dff57af7)"
  sg() { scope_gate "$1" "$2" >/dev/null && ck "$3" refused accepted || ck "$3" refused refused; }
  sg "0::/" fierce-wren-19-r6-dff57af7 scope-root
  sg "0::/x/other.scope" fierce-wren-19-r6-dff57af7 scope-foreign
  sg "0::/x/fierce-wren-19-r6-dff57af7.scope.d" fierce-wren-19-r6-dff57af7 scope-trailing
  sg "0::/x/prefierce-wren-19-r6-dff57af7.scope" fierce-wren-19-r6-dff57af7 scope-prefix-hit
  sg "0::/x/fierce-wren-19-r6-dff57af7.scope/sub" fierce-wren-19-r6-dff57af7 scope-nested
  sg "1::/x/fierce-wren-19-r6-dff57af7.scope" fierce-wren-19-r6-dff57af7 scope-not-v2
  sg "$(printf '0::/x/y.scope\n1::/z')" y scope-two-lines
  sg "0::/x/y.scope" "" scope-empty-job
  # OUT freshness: any existing entry refuses; a fresh path is accepted
  mkdir $T/od; : > $T/of; ln -s $T/of $T/ol; ln -s $T/nowhere $T/odl
  for x in od of ol odl; do out_absent $T/$x && ck "out-$x" refused accepted || ck "out-$x" refused refused; done
  out_absent $T/fresh && ck out-absent accepted accepted || ck out-absent accepted refused
  mkdir -- "$T/ex" 2>/dev/null; mkdir -- "$T/ex" 2>/dev/null && ck mkdir-exclusive refused accepted || ck mkdir-exclusive refused refused
  # bin_sha: a failed read can never compare equal to another failed read
  bin_sha $T/no-such-file >/dev/null && ck hash-missing refused accepted || ck hash-missing refused refused
  printf 'x' > $T/hf; ck hash-ok-64hex ok "$( [[ "$(bin_sha $T/hf)" =~ ^[0-9a-f]{64}$ ]] && echo ok || echo bad)"
  ( bin_sha $T/no-such-file; bin_sha $T/no-such-other ) >/dev/null 2>&1; a=$(bin_sha $T/no-such-file); ra=$?; b=$(bin_sha $T/no-such-other); rb=$?
  [ $ra != 0 ] && [ $rb != 0 ] && ck hash-two-failures-not-equal refused refused || ck hash-two-failures-not-equal refused accepted
  # wall accounting must be arithmetic a reader can check, not prose
  ck capped-sum 4020 "$WALL_SUM"
  ck wall-gate-accepts-this-row OK "$(wall_gate "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S")"
  ck runtime-s-within-installed-launcher-cap ok "$([ "$RUNTIME_S" -le "$LAUNCHER_CAP" ] && echo ok || echo over-cap)"
  ck driver-wall-is-scope-plus-300 ok "$([ "$DRIVER_S" = "$((RUNTIME_S+300))" ] && echo ok || echo short)"
  ck wall-gate-refuses-over-launcher-cap "RUNTIME_S 9900 exceeds the installed calmcrab-udrv v4 preflight cap of 7200" "$(wall_gate "$WALL_SUM" 9900 "$LAUNCHER_CAP" 10200 | cut -d';' -f1)"
  ck wall-gate-refuses-sum-above-scope 1 "$(wall_gate 99999 "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S" | grep -c 'exceeds the scope RuntimeMaxSec')"
  ck wall-gate-refuses-wrong-driver-offset 1 "$(wall_gate "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" "$((DRIVER_S+300))" | grep -c 'plus 300 s')"
  # ---------------- the frozen argv, tokenized exactly as bash -c will tokenize it ----------------
  # A leg is executed as: timeout <t> bash -c "<the frozen string>". So what claim_batch's arg loop sees is
  # that string AFTER bash has expanded it. eval "set -- <string>" reproduces that expansion here, in this
  # container, without running a claim: it turns the packet's own argv into data and checks it.
  # PARSER_FLAGS is the flag set accepted by src/v1/stage0/src/bin/claim_batch.rs at the pin (blob
  # 12a71786bd92f683dbd0aec2ae5d6f426e651a44); anything else makes that file print
  # "claim_batch: unknown argument: ..." and exit 2, which this packet scores NO_OBSERVATION, not FAIL.
  PARSER_FLAGS='--source-root --entry --function --functions --claim-run --wet --hermetic --record --eval-budget-ms --fixture-store --pre-push --print-entry-closure --outcomes-out'
  BIN6=${BIN6:-/fixture/wt-r6/target/release/claim_batch}
  argv_check() { # argv_check <label> <frozen string> <entry> <names...>: prints one line per defect, silent if sound
    local label=$1 cmd=$2 ent=$3; shift 3; local names="$*"; local i j x fv='' nf nn
    eval "set -- $cmd" 2>/dev/null || { echo "$label: the frozen string does not tokenize as a command"; return; }
    [ "$#" -gt 1 ] || { echo "$label: nothing follows the binary path"; return; }
    [ "$1" = "$BIN6" ] || { echo "$label: argv[0] is '$1', not the pinned runner path"; return; }
    case " $* " in *" claim_batch "*) echo "$label: argv carries a literal 'claim_batch' token after the binary"; return;; esac
    for i in "$@"; do case "$i" in --*) case " $PARSER_FLAGS " in *" $i "*) ;; *) echo "$label: '$i' is not a flag the pinned parser accepts";; esac;; esac; done
    local ne=0 nfn=0
    for ((i=1; i<=$#; i++)); do
      case "${!i}" in
        --entry)     ne=$((ne+1));   j=$((i+1)); [ "${!j:-}" = "$ent" ] || echo "$label: --entry value is '${!j:-}', not $ent" ;;
        --functions) nfn=$((nfn+1)); j=$((i+1)); fv="${!j:-}" ;;
      esac
    done
    [ "$ne" = 1 ]  || echo "$label: --entry appears $ne times (exactly 1 is required)"
    [ "$nfn" = 1 ] || echo "$label: --functions appears $nfn times (exactly 1 is required)"
    case "$fv" in *' '*) echo "$label: --functions '$fv' contains a space; the parser splits on ',' and trims, so this asks for ONE name with spaces";; esac
    [ -n "$fv" ] || echo "$label: --functions value is empty"
    nf=$(printf '%s' "$fv" | awk -F',' '{print NF}'); nn=$(printf '%s\n' $names | grep -c .)
    [ "$nf" = "$nn" ] || echo "$label: --functions carries $nf comma-separated names but the leg expects $nn"
    [ "$(printf '%s\n' $names | grep -cvE '^[a-z0-9_]+$')" = 0 ] || echo "$label: a name in the leg's list is not ^[a-z0-9_]+$"
    ( IFS=','; set -f; for x in $fv; do case " $names " in *" $x "*) ;; *) echo "$label: --functions field '$x' is not one of the leg's names";; esac; done )
    for x in $names; do case ",$fv," in *",$x,"*) ;; *) echo "$label: '$x' is missing from --functions";; esac; done
    return 0; }
  ck argv-R6-sound "" "$(argv_check R6 "$CMD_LEG_R6" "$E6" $N6)"
  # the checker itself, in the other direction: each defect the manager named must be reported
  ck argv-ctl-subcommand 1 "$(argv_check ctl "\"$BIN6\" claim_batch --source-""root dag --entry x --functions a" x a | grep -c "literal 'claim_batch'")"
  ck argv-ctl-spaces     1 "$(argv_check ctl "\"$BIN6\" --source-root dag --entry x --functions \"a b\"" x a b | grep -c 'contains a space')"
  ck argv-ctl-badflag    1 "$(argv_check ctl "\"$BIN6\" --entry x --functions a --frobnicate" x a | grep -c 'not a flag')"
  ck argv-ctl-typo       1 "$(argv_check ctl "\"$BIN6\" --entry x --functions \"a,zzz\"" x a b | grep -c 'not one of the leg' )"
  ck argv-ctl-missing    1 "$(argv_check ctl "\"$BIN6\" --entry x --functions \"a\"" x a b | grep -c 'missing from --functions')"
  ck argv-ctl-double     1 "$(argv_check ctl "\"$BIN6\" --entry x --entry y --functions a" x a | grep -c 'appears 2 times')"
  ck argv-ctl-noentry    1 "$(argv_check ctl "\"$BIN6\" --source-root dag --functions a" x a | grep -c -- '--entry appears 0 times')"
  ck argv-ctl-wrongentry 1 "$(argv_check ctl "\"$BIN6\" --entry other.dag --functions a" "${E6:-x}" a | grep -c 'entry value is')"
  # ---------------- self-structure: the packet refuses to run if its own frozen text lost an invariant --
  # The needle is written split across two quotes so this call site can never match the pattern it seeks.
  inv() { local a=$1 b=$2 lbl=$3 want=$4 got
    got=$(grep -vE '^[[:space:]]*#' "$0" | grep -cF -- "$a$b"); ck "self-structure: $lbl" "$want" "$got"; }
  invmin() { local a=$1 b=$2 lbl=$3 want=$4 got
    got=$(grep -vE '^[[:space:]]*#' "$0" | grep -cF -- "$a$b"); ck "self-structure: $lbl" yes "$([ "${got:-0}" -ge "$want" ] && echo yes || echo no)"; }
  inv 'set -C; pri' 'ntf' 'the OOM snapshot is published by an exclusive create, before any parsing' 1
  inv 'set -C; :' ' > "$f.unrea' 'a failed counter read still leaves a raw marker' 1
  inv 'grep -c ' '^oom_kill' 'the forbidden nonzero-kill grep shape is absent' 0
  inv 'BM=$(bounds_che' 'ck' 'the booked bounds are gated before any work' 1
  inv 'LO=$(oom_pair_ver' 'dict "$k0"' 'each leg boundary pair is compared, not merely recorded' 1
  inv 'LASTOOM=$k' '1' 'the last leg-after snapshot is threaded to the final comparison' 1
  inv 'FV=$(oom_pair_ver' 'dict "$LAST' 'the final snapshot is compared against the last leg' 1
  inv 'claim_batch --source-roo' 't' 'no leg command carries a subcommand token' 0
  inv 'exit ' '3' 'no fourth exit code exists anywhere in the script' 0
  inv 'exit ' '1' 'exactly one falsification exit' 1
  invmin 'exit ' '2; }' 'refusal helpers exit with the no-observation code' 1
  inv 'dmes' 'g ' 'memory evidence never consults the kernel ring buffer' 0
  inv 'journalct' 'l ' 'memory evidence never consults the journal' 0
  invmin 'memory.eve' 'nts' 'the evidence source is the cgroup counter file' 1
  inv 'timeout "$T_LE' 'G"' 'a leg runs under its own capped timeout' 1
  inv 'timeout "$T_BUIL' 'D"' 'the build runs under its own capped timeout' 1
  inv 'WM=$(wall_ga' 'te' 'the packet refuses a booked wall above the installed launcher cap, before any fetch' 1
  inv 'LAUNCHER_CA' 'P=7200' 'the installed calmcrab-udrv v4 cap is frozen as a constant, not prose' 1
  inv 'timeout "$T_SETUP_FETC' 'H"' 'the fetch runs under its own capped timeout' 1
  inv 'pc=$(pp_che' 'ck "$OUT/logs/$L.err")' 'the completion gate reads the leg stderr capture, where the record is emitted' 1
  inv 'pp_check "$OUT/$L.o' 'ut' 'no completion gate reads the leg stdout, which carries only the verdicts' 0
  inv 'claim_classify "$OUT/$L.o' 'ut' 'the verdict classifier reads the leg stdout' 1
  inv 'm=$(producer_ga' 'te' 'the pin proves the pinned producer emits the record to stderr' 1
  inv 'blob_gate "$W" "$PROD" "$PRODBL' 'OB")' 'the pinned claim_batch.rs is blob-gated like the parser flag set' 1
  inv 'ck e2e-record-on-std' 'err-is-an-observation AS_EXPECTED' 'the fixtures drive a real leg and require the stderr record to be an observation' 1
  inv 'ck e2e-record-only-on-std' 'out-is-not-an-observation NOOBS' 'the fixtures require a stdout-only record to be no observation' 1
  inv 'ck e2e-two-records-is-not-an-observat' 'ion NOOBS' 'the fixtures reject a duplicated record' 1
  inv 'pp_check() ' '{' 'the completion-record gate is defined' 1
  inv 'PP_PRE' "FIX='^[[]process-partition" 'the completion prefix keeps the closing bracket (the -exit neighbour must not count)' 1
  inv 'any=$(grep -cE "' '^(PASS|FAIL' 'each expected name needs exactly one outcome line of any keyword' 1
  inv 'MEMMAX_G*' '1073741824' 'the booked byte bounds are derived from the launcher GiB values' 1
  inv 'grant_gate "$GRA' 'NT"' 'the held-row grant gate is on the run path' 1
  ln_b=$(grep -nF -- 'BM=$(bounds_che'ck "$0" | tail -1 | cut -d: -f1)
  nf1='T_SETUP_FETC'; nf2='H"'; ln_f=$(grep -nF -- "$nf1$nf2" "$0" | head -1 | cut -d: -f1)
  ck bounds-precedes-fetch ok "$([ -n "$ln_b" ] && [ -n "$ln_f" ] && [ "$ln_b" -lt "$ln_f" ] && echo ok || echo out-of-order)"

  # -------- completion-record gate (v4 blocker 2). The REAL record, byte for byte, from the pinned
  # producer: calm-crab-645's srv1 run, ~/runs/cc1-15695-495d-v16/out/claims-cpr.log (log sha256
  # a0e1f1f63669ca4ea0b06d903c144c9fdc3f9f0e95dfab2485ad03752af6ed38), line 66 of 78, the only line
  # beginning '[process-partition] total_ms'. 259 bytes, no CR, no leading/trailing space. The pinned
  # digest is over those 259 bytes WITHOUT a trailing LF; the fixture re-verifies it on every call, so a
  # fixture whose bytes drift refuses the run.
  PP_REAL='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
  PP_REAL_SHA=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2
  PP_SECOND='[process-partition] total_ms=67069 pre_run_ms=0 args_ms=2 index_build_ms=41481 edge_index_warm_ms=17633 output_policy_ms=3805 roster_ms=0 published_mock_precompute_ms=1393 entries_ms=2752 reporting_ms=0 entries_resolve_ms=1837 entries_witness_ms=908'

  # fixture inputs for the completion-record and verdict rules (written under the scratch dir only)
  printf '%s' "$PP_REAL" > "$T/pp_real.txt"; printf '\n' >> "$T/pp_real.txt"
  printf '%s\n' "$PP_SECOND" > "$T/pp_second.txt"
  printf '[process-partition] total_ms=123\n' > "$T/pp_bare.txt"
  printf '%s\n%s\n' "$PP_REAL" "$PP_SECOND" > "$T/pp_dup.txt"
  printf '%s\n[process-partition] total_ms=zz entries_witness_ms=1\n' "$PP_REAL" > "$T/pp_valid_bad.txt"
  printf '[process-partition] total_ms=abc entries_resolve_ms=1\n' > "$T/pp_bad.txt"
  printf 'PASS w_x\n' > "$T/pp_none.txt"
  printf '%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=340188\n' "$PP_REAL" > "$T/pp_neighbour.txt"
  printf '[process-partition-exit] run_returned_ms=1 main_end_ms=1\n' > "$T/pp_neighbour_only.txt"
  printf 'PASS w_x\n' > "$T/v_pass.txt"; printf 'FAIL w_x\n' > "$T/v_fail.txt"
  printf 'FAIL w_x (returned ProcessExit::ExitFailure, code 1: boom)\n' > "$T/v_qfail.txt"
  printf 'UNOBSERVED w_x (no receipt)\n' > "$T/v_unobs.txt"
  printf 'PANICKED w_x (runtime panic)\n' > "$T/v_panic.txt"
  printf 'PASS w_x\nPASS w_x\n' > "$T/v_two.txt"
  printf 'PASS w_x\nFAIL w_x\n' > "$T/v_both.txt"
  printf 'PASS w_x\nFAIL w_x (runtime error: boom)\n' > "$T/v_bare_plus_q.txt"
  printf 'total_ms=1\n' > "$T/v_none.txt"
  printf 'PASS w_xy\n' > "$T/v_other.txt"
  ck completion-real-bytes 259 "$(printf '%s' "$PP_REAL" | wc -c | tr -d ' ')"
  ck completion-real-sha "$PP_REAL_SHA" "$(printf '%s' "$PP_REAL" | sha256sum | cut -d' ' -f1)"
  ck completion-gate-reverifies-sha OK "$(pp_check "$T/pp_real.txt" | cut -d" " -f1)"
  ck completion-second-sample OK "$(pp_check "$T/pp_second.txt" | cut -d" " -f1)"
  ck completion-predicate-shape-only OK "$(pp_check "$T/pp_bare.txt" | cut -d" " -f1)"
  ck completion-duplicate "completion records: 2 matched the prefix, exactly 1 is required" "$(pp_check "$T/pp_dup.txt")"
  ck completion-valid-plus-malformed "completion records: 2 matched the prefix, exactly 1 is required" "$(pp_check "$T/pp_valid_bad.txt")"
  ck completion-malformed-only "the single completion record is malformed: [process-partition] total_ms=abc entries_resolve_ms=1" "$(pp_check "$T/pp_bad.txt")"
  ck completion-none "completion records: 0 matched the prefix, exactly 1 is required" "$(pp_check "$T/pp_none.txt")"
  ck completion-neighbour-not-counted OK "$(pp_check "$T/pp_neighbour.txt" | cut -d" " -f1)"
  ck completion-neighbour-only "completion records: 0 matched the prefix, exactly 1 is required" "$(pp_check "$T/pp_neighbour_only.txt")"
  ck completion-unreadable "unreadable leg output" "$(pp_check "$T/pp_missing.txt")"
  ck completion-wholeline-anchor-is-wrong 0 "$(printf '%s\n' "$PP_REAL" | grep -cE '^[[]process-partition[]] total_ms=[0-9]+$')"
  ck completion-shape-accepts-real 1 "$(printf '%s\n' "$PP_REAL" | grep -cE "$PP_SHAPE")"
  # fixture part 1 of 2: the leg-level controls run once leg() is defined (see part 2)
  # -------- verdict rules (v4 blocker 3): bare lines count, one outcome line per name, qualified refuses
  ck verdict-bare-pass-rc0 HELD "$(claim_classify "$T/v_pass.txt" 0 w_x)"
  ck verdict-bare-fail-rc1 FAILED "$(claim_classify "$T/v_fail.txt" 1 w_x)"
  ck verdict-qualified-fail-is-not-falsified NO_OBSERVATION "$(claim_classify "$T/v_qfail.txt" 1 w_x)"
  ck verdict-qualified-fail-rc0 NO_OBSERVATION "$(claim_classify "$T/v_qfail.txt" 0 w_x)"
  ck verdict-unobserved NO_OBSERVATION "$(claim_classify "$T/v_unobs.txt" 2 w_x)"
  ck verdict-panicked NO_OBSERVATION "$(claim_classify "$T/v_panic.txt" 1 w_x)"
  ck verdict-two-outcome-lines NO_OBSERVATION "$(claim_classify "$T/v_two.txt" 0 w_x)"
  ck verdict-pass-and-fail NO_OBSERVATION "$(claim_classify "$T/v_both.txt" 1 w_x)"
  ck verdict-bare-pass-plus-qualified-fail NO_OBSERVATION "$(claim_classify "$T/v_bare_plus_q.txt" 0 w_x)"
  ck verdict-no-outcome-line NO_OBSERVATION "$(claim_classify "$T/v_none.txt" 0 w_x)"
  ck verdict-bare-pass-rc1-cost-partition NO_OBSERVATION "$(claim_classify "$T/v_pass.txt" 1 w_x)"
  ck verdict-bare-pass-rc124 NO_OBSERVATION "$(claim_classify "$T/v_pass.txt" 124 w_x)"
  ck verdict-prefix-name-not-matched NO_OBSERVATION "$(claim_classify "$T/v_other.txt" 0 w_x)"

  echo "fixture part 1 ok=$okn fail=$badn so far"
fi

# ---------------------------------------------------------------- fixture part 2: the leg-level controls
# need leg(), producer_gate and nox DEFINED, so they run here rather than in the block above -- in the same
# shell as part 1 (the scratch dir, the counters and the pinned record fixture carry over), and still BEFORE
# any probe is spent and BEFORE the held-row grant gate: no clone, no fetch, no build, no claim, and nothing
# written outside the scratch dir.
if [ "${1:-}" = "--classifier-fixtures" ]; then
  # ---- THE COMPLETION RECORD REACHES THE GATE, end to end through this packet's own leg(). The producer
  # ---- emits the record with eprintln! (stderr) and the verdicts with println! (stdout); leg() sends stdout
  # ---- to <leg>.out and stderr to logs/<leg>.err. A stub stands in for the claim_batch binary, so a gate
  # ---- pointed at the wrong stream, or a duplicated record, fails HERE and not on srv1.
  EOUT=$T/e2e; mkdir -p "$EOUT/mem" "$EOUT/logs"
  OUT=$EOUT; O=$EOUT/receipt.txt; MEMDIR=$EOUT/mem; EV=$EOUT/events
  printf 'low 0\nhigh 3\nmax 1\noom 0\noom_kill 0\nfail 0\ndiscard_oom 0\n' > "$EV"
  cat > "$T/stub_stderr.sh" <<STUBE2E
#!/bin/sh
printf 'PASS w_x\n'
printf '%s\n' '$PP_REAL' >&2
exit 0
STUBE2E
  cat > "$T/stub_stdout.sh" <<STUBE2E
#!/bin/sh
printf 'PASS w_x\n'
printf '%s\n' '$PP_REAL'
exit 0
STUBE2E
  cat > "$T/stub_dup.sh" <<STUBE2E
#!/bin/sh
printf 'PASS w_x\n'
printf '%s\n' '$PP_REAL' >&2
printf '%s\n' '$PP_REAL' >&2
exit 0
STUBE2E
  chmod +x "$T/stub_stderr.sh" "$T/stub_stdout.sh" "$T/stub_dup.sh"
  BIN4=$T/stub_stderr.sh; BIN5=$T/stub_stderr.sh; BIN6=$T/stub_stderr.sh; export BIN4 BIN5 BIN6
  leg E2E_STDERR "\"\$BIN4\" --source-root dag --entry x.dag --functions w_x" w_x
  ck e2e-record-on-stderr-is-an-observation AS_EXPECTED "$(cat "$OUT/E2E_STDERR.state")"
  ck e2e-record-on-stderr-counted-in-the-receipt 1 "$(grep -c 'E2E_STDERR completion_record: OK' "$O")"
  ck e2e-record-on-stderr-holds-the-name 1 "$(grep -c 'w_x HELD' "$OUT/E2E_STDERR.classes")"
  BIN4=$T/stub_stdout.sh
  leg E2E_STDOUT "\"\$BIN4\" --source-root dag --entry x.dag --functions w_x" w_x
  ck e2e-record-only-on-stdout-is-not-an-observation NOOBS "$(cat "$OUT/E2E_STDOUT.state")"
  BIN4=$T/stub_dup.sh
  leg E2E_DUP "\"\$BIN4\" --source-root dag --entry x.dag --functions w_x" w_x
  ck e2e-two-records-is-not-an-observation NOOBS "$(cat "$OUT/E2E_DUP.state")"
  # ---- the producer render site, on synthetic sources
  cat > "$T/pb_ok.rs" <<'STUBPB'
    eprintln!("{}", partition.render(&timings));
STUBPB
  cat > "$T/pb_stdout.rs" <<'STUBPB'
    println!("{}", partition.render(&timings));
STUBPB
  cat > "$T/pb_two.rs" <<'STUBPB'
    eprintln!("{}", partition.render(&timings));
    let _ = partition.render(&timings);
STUBPB
  ck producer-stderr-render-site OK "$(producer_gate "$T/pb_ok.rs")"
  ck producer-stdout-render-site-refuses "the completion record is NOT emitted to stderr (eprintln) at the render site" "$(producer_gate "$T/pb_stdout.rs" | cut -d: -f1)"
  ck producer-two-render-sites-refuses "expected exactly one completion-record render site, found 2" "$(producer_gate "$T/pb_two.rs" | cut -d' ' -f1-8)"
  ck producer-unreadable-refuses "the pinned claim_batch.rs is unreadable: $T/nope.rs" "$(producer_gate "$T/nope.rs")"
  ck completion-predicates-empty-refuses "the completion predicates are empty; a vacuous pattern would count every line" "$(PP_PREFIX= ; pp_check "$T/pp_real.txt")"
  echo "FIXTURES pass=$okn fail=$badn"; [ $badn -eq 0 ]; exit $?
fi

# ---------------------------------------------------------------- predict-only: prints the manifest, does nothing else
sha_of() { printf '%s' "$1" | sha256sum | awk '{print $1}'; }
bytes_of() { printf '%s' "$1" | wc -c | tr -d ' '; }
if [ "${1:-}" = "--predict-only" ]; then
  # Writes NOTHING, contacts NOTHING — no OUT, no clone, no build, no store. The live partition is not
  # touched by printing this manifest. Bytes and sha256 are over the LITERAL frozen string, before $OUT and
  # $BIN6 are expanded at run time.
  echo "PREDICT-ONLY row=R6 job=$J state=HELD gate=keen-otter-msg_3407a188 host_booked=srv1"
  echo "pin=$H tree=$T"
  printf 'blob %s %s\n' "$E6" "$B6"
  echo "bounds=$BOOKED"
  echo "launcher_units runtime_s=$RUNTIME_S memmax_gib=$MEMMAX_G memhigh_gib=$MEMHIGH_G tasks=$TASKS_MAX cpu=$CPU_PCT%"
  echo "launcher_call=calmcrab-udrv.sh $J $RUNTIME_S $MEMMAX_G $MEMHIGH_G $TASKS_MAX $CPU_PCT% <STAGED_SCRIPT> <this packet sha256> OUT"
  echo "ledger_rows=$J-res(cap ${MEMMAX_G}GiB) $J-drv(cap 1) $J(cap 0)"
  echo "completion_record_gate=exactly-one-prefix-count-then-shape sha_pinned=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2"
  echo "completion_record_stream=stderr(leg capture logs/<leg>.err) verdict_stream=stdout(<leg>.out) render=the single eprintln partition.render(&timings) emission in run() blob=$PRODBLOB"
  echo "verdict_rules=bare_only;one_outcome_line_per_name_any_keyword;qualified_is_NO_OBSERVATION"
  echo "timeouts_s setup_clone=$T_SETUP_CLONE setup_fetch=$T_SETUP_FETCH setup_checkout=$T_SETUP_CHECKOUT build=$T_BUILD leg=$T_LEG"
  echo "wall_capped_sum_s=$WALL_SUM scope_RuntimeMaxSec=$RUNTIME_S driver_RuntimeMaxSec=$DRIVER_S launcher_cap=$LAUNCHER_CAP legs=$N_LEGS"
  echo "driver_service=calmcrab-drv-$J.service RuntimeMaxSec=$DRIVER_S (RUNTIME_S plus 300 s)"
  echo "launcher_preflight_cap=7200 (calmcrab-udrv v4, 855920ac): a RUNTIME_S above it is a PRE_SCOPE_REFUSAL, so this packet refuses it itself"
  echo "claims_r6=$(printf '%s' "$C6" | awk -F',' '{print NF}')"
  echo "execute form: script.sh <OUT> <execution-grant-ref>   (no reference = refused, exit 2, nothing created)"
  echo "exit_contract: 0=expectations_held 1=falsified_bare_FAIL 2=NO_OBSERVATION(every refusal, incl. the hold)"
  for c in CMD_CLONE CMD_FETCH CMD_CHECKOUT CMD_BUILD CMD_LEG_R6; do
    v="${!c}"; printf 'cmd %s bytes=%s sha256=%s\n  %s\n' "$c" "$(bytes_of "$v")" "$(sha_of "$v")" "$v"
  done
  exit 0
fi

# ---------------------------------------------------------------- usage: <OUT> [<execution-grant-ref>]
[ $# -ge 1 ] && [ $# -le 2 ] && [ -n "$1" ] && [ "${1#-}" = "$1" ] || { echo "NO_OBSERVATION: usage: script.sh <OUT> [<execution-grant-ref>] — the launcher's fresh evidence dir (<RUNTIME>/out) plus, for execution, the grant reference that cleared msg_3407a188; a leading dash is never a path" >&2; exit 2; }
OUT=$1
GRANT=${2:-}
case "$OUT" in */out) ;; *) echo "NO_OBSERVATION: OUT '$OUT' does not end in /out (calmcrab-udrv v4 maps the row's evidence dir to <RUNTIME>/out)" >&2; exit 2;; esac
JOB=$(basename "$(dirname "$OUT")" 2>/dev/null || echo "")
[ -n "$JOB" ] || { echo "NO_OBSERVATION: cannot derive the booked JOB from OUT '$OUT' (expected <RUNTIME>/<JOB>/out)" >&2; exit 2; }
# THE HOLD, checked BEFORE anything is created: an ungranted execution leaves no directory behind.
g=$(grant_gate "$GRANT") || { echo "NO_OBSERVATION: $g" >&2; exit 2; }
command -v timeout >/dev/null || { echo "NO_OBSERVATION: timeout(1) is missing and every step would be unbounded" >&2; exit 2; }
[ -x /usr/bin/time ] || { echo "NO_OBSERVATION: /usr/bin/time is missing; elapsed and peak-RSS evidence could not be recorded" >&2; exit 2; }
out_absent "$OUT" || { echo "NO_OBSERVATION: OUT $OUT already exists (any entry — file, directory, symlink or dangling symlink — refuses; a row's evidence dir is never reused)" >&2; exit 2; }
mkdir -- "$OUT" || { echo "NO_OBSERVATION: exclusive mkdir of $OUT failed (an entry appeared, or no permission)" >&2; exit 2; }
chmod 700 "$OUT"
export OUT
MEMDIR=$OUT/memevents; mkdir -- "$MEMDIR" || { echo "NO_OBSERVATION: cannot make $MEMDIR" >&2; exit 2; }
mkdir -- "$OUT/logs" || { echo "NO_OBSERVATION: cannot make $OUT/logs" >&2; exit 2; }
export TMPDIR=$OUT/tmp; mkdir -- "$TMPDIR" || { echo "NO_OBSERVATION: cannot make $TMPDIR" >&2; exit 2; }
O=$OUT/receipt.txt
echo "job=$JOB booked_job=$J state=executing-under-grant host=$(hostname 2>/dev/null || echo unreadable) machine_id=$(cat /etc/machine-id 2>/dev/null || echo unreadable) start=$(date -u +%FT%TZ)" > "$O"
S=$(bin_sha "$0") || nox "cannot hash the staged script at $0" script-hash
echo "script_sha256=$S (compare with the published frozen value; a file cannot gate its own hash)" >> "$O"
echo "$GRANT" > "$OUT/grant-ref.txt"
echo "grant_ref=$GRANT (the reference this row was executed under; its clearance is keen-otter's, not this packet's)" >> "$O"

# FIXTURES BEFORE ANY WORK IS SPENT: a misfired classifier ends the row with no claim attempted.
bash "$0" --classifier-fixtures > "$OUT/logs/selfcheck.log" 2>&1 || nox "the packet's own classifier fixtures misfired; no claim attempted (see logs/selfcheck.log)" fixtures
echo "fixtures: $(grep -c '^FIXTURE PASS' "$OUT/logs/selfcheck.log") pass, $(grep -c '^FIXTURE FAIL' "$OUT/logs/selfcheck.log") fail" >> "$O"

CG=$(scope_gate "$(cat /proc/self/cgroup 2>/dev/null)" "$JOB") || nox "scope gate: /proc/self/cgroup is not exactly one 0:: line whose path ends in /$JOB.scope: $(tr '\n' ' ' < /proc/self/cgroup 2>/dev/null)" scope
EV=/sys/fs/cgroup$CG/memory.events
GA=$(oom_snap "$EV" gate-a); GB=$(oom_snap "$EV" gate-b)
[ "$(oom_pair_verdict "$GA" "$GB")" = none ] || nox "memory.events at $EV unreadable or already OOM-marked at the gate: $GA -> $GB" "$([ "$(oom_pair_verdict "$GA" "$GB")" = UNKNOWN ] && echo counter-incoherent || echo oom-before-work)"
BM=$(bounds_check "/sys/fs/cgroup$CG") || nox "booked bounds: $BM" bounds
WM=$(wall_gate "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S") || nox "wall gate: $WM" wall
echo "wall sum=${WALL_SUM}s scope_RuntimeMaxSec=${RUNTIME_S} driver_RuntimeMaxSec=${DRIVER_S} launcher_cap=${LAUNCHER_CAP} legs=${N_LEGS}" >> "$O"
echo "scope=$CG bounds_ok=$BOOKED oom_gate=$GA->$GB" >> "$O"
echo "host recorded, not gated: the booking is enforced by the launcher's ledger rows and the bounds gate above, not by a hostname string" >> "$O"
blob_gate() { # $1=clone $2=path $3=blob
  local have; have=$(git -C "$1" rev-parse "HEAD:$2" 2>/dev/null) || { echo "missing"; return 1; }
  [ "$have" = "$3" ] || { echo "$2 is blob $have at HEAD, this packet freezes $3"; return 1; }
  printf 'blob %s %s\n' "$2" "$have" >> "$OUT/blob-gate.txt"; }

W=$OUT/wt-r6
timeout "$T_SETUP_CLONE" bash -c "$CMD_CLONE" > "$OUT/logs/wt-r6-init.log" 2>&1 || nox "git init of $W failed or exceeded ${T_SETUP_CLONE}s" clone
timeout "$T_SETUP_FETCH" bash -c "$CMD_FETCH" > "$OUT/logs/wt-r6-fetch.log" 2>&1 || nox "shallow fetch of $H into $W failed or exceeded ${T_SETUP_FETCH}s" clone
timeout "$T_SETUP_CHECKOUT" bash -c "$CMD_CHECKOUT" > "$OUT/logs/wt-r6-checkout.log" 2>&1 || nox "checkout of FETCH_HEAD in $W failed or exceeded ${T_SETUP_CHECKOUT}s" clone
gotH=$(git -C "$W" rev-parse HEAD 2>/dev/null || echo unreadable)
gotT=$(git -C "$W" rev-parse 'HEAD^{tree}' 2>/dev/null || echo unreadable)
[ "$gotH" = "$H" ] || nox "$W HEAD is $gotH, this packet freezes $H" pin
[ "$gotT" = "$T" ] || nox "$W tree is $gotT, this packet freezes $T for $H" tree
m=$(blob_gate "$W" "$E6" "$B6") || nox "blob gate: $m" blob
m=$(names_gate "$W" "$E6" "$N6") || nox "names gate: $m" names
m=$(blob_gate "$W" "$PROD" "$PRODBLOB") || nox "blob gate: $m" blob
m=$(producer_gate "$W/$PROD") || nox "producer stream gate: $m" producer
echo "cloned wt-r6 head=$gotH tree=$gotT blobs+names+producer gated" >> "$O"

cd "$W" || nox "cannot cd into $W" clone
k0=$(oom_snap "$EV" wt-r6-build-before)
timeout "$T_BUILD" /usr/bin/time -v bash -c "$CMD_BUILD" > "$OUT/logs/wt-r6-build.log" 2>&1; BX=$?
k1=$(oom_snap "$EV" wt-r6-build-after); BO=$(oom_pair_verdict "$k0" "$k1")
echo "== build wt-r6 exit=$BX oom=$BO $(tm "$OUT/logs/wt-r6-build.log")" >> "$O"
[ "$BX" = 124 ] && nox "wt-r6 build timed out at ${T_BUILD}s" build-timeout
[ "$BX" = 0 ] || { tail -10 "$OUT/logs/wt-r6-build.log" >> "$O"; nox "wt-r6 build exit=$BX" build; }
[ "$BO" = none ] || nox "wt-r6 build oom=$BO ($k0 -> $k1)" "$([ "$BO" = OOM_KILLED ] && echo build-oom || echo counter-incoherent)"
BIN6=$W/target/release/claim_batch
[ -x "$BIN6" ] || nox "wt-r6 build produced no executable at $BIN6" build
echo "binary $BIN6 sha256=$(bin_sha "$BIN6" || echo unreadable)" >> "$O"
export BIN6

LASTOOM=""
leg R6 "$CMD_LEG_R6" "$N6"

FA=$(oom_snap "$EV" final)
FV=$(oom_pair_verdict "$LASTOOM" "$FA")
echo "== final oom vs last leg after: $LASTOOM -> $FA verdict=$FV" >> "$O"
[ "$FV" = none ] || nox "memory events moved between the last leg and the final snapshot: $LASTOOM -> $FA ($FV)" "$([ "$FV" = OOM_KILLED ] && echo oom-in-teardown || echo counter-incoherent)"
echo "memory.peak (printed, never judged): $(cat "/sys/fs/cgroup$CG/memory.peak" 2>/dev/null || echo unreadable)" >> "$O"

S6=$(cat "$OUT/R6.state")
echo "row R6 state: R6=$S6 (one live-corpus projection; a refusal of the SUBJECT is a finding)" >> "$O"
if [ "$S6" = NOOBS ]; then
  echo "VERDICT=NO_OBSERVATION reason=the-leg-had-no-observation" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
  echo "NO_OBSERVATION: the R6 leg produced no clean bare-verdict observation" >&2; exit 2
fi
if [ "$S6" = FALSIFIED ]; then
  echo "VERDICT=FALSIFIED R6=$S6" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
  echo "FALSIFIED: the R6 claim printed a bare FAIL (a finding for the grant set)" >&2; exit 1
fi
echo "VERDICT=AS_EXPECTED claims=1 legs=1 mutation_legs=0" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
exit 0
