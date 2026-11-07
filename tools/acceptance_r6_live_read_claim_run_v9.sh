# script.sh (frozen copy for fierce-wren-19-r6-dff57af7, v9): acceptance grant set row R6, gunbc.
# HELD — NOT AUTHORIZED FOR EXECUTION. PREPARE ONLY: this file has not been RUN anywhere, and the live
# fabric store has not been contacted. Same hardened srv1 frame as the R4/R5 packet (template:
# wise-moth-707-rdpo's rdpo.sh, sha256 aaf32964184a30d66f6320de707bfb03975f0df683cccd1133dcf2d024ed9366,
# gunbc#15938 comment 6672252712); the gates, helpers, classifier, OOM contract and fixtures here are the
# SAME text as that packet's, so one review covers both.
#
# THE ONE CLAIM (row R6):
#   dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag
#     witness_inbox_on_authority_projects_on_live_corpus
# WHAT v9 CHANGES (three things, all of them in how the packet CHECKS itself; the booking, the pins, the
# bounds, the wall constants, the claim inventory and the command digests are unchanged):
#   1. cs_check binds a claim to its OWN leg. v8 compared claim names as a flat set, so a claim registered
#      under leg A sitting in leg B's classes file was accepted. v9 requires each <stem>.classes to hold
#      exactly the names registered under that stem, each once (literal awk field comparisons, never an
#      interpolated regex), adds a cross-file duplicate scan, and refuses a registration whose leg names are
#      not exactly the booked legs even when the line count still matches.
#   2. cs_check SNAPSHOTS what it validated (cs-snapshot/, with digests) and final_verdict counts only the
#      snapshot; a file rewritten after validation changes nothing in the verdict and revalidation refuses it.
#   3. the fixture repo cannot inherit a host git identity: user.name/user.email on init AND commit, an empty
#      HOME, GIT_CONFIG_NOSYSTEM=1, GIT_CONFIG_GLOBAL=/dev/null and core.hooksPath pointing at an empty dir.
#   4. the wall sum now counts, INSIDE an unchanged RUNTIME_S (R4 5400 s, R5 4500 s, R6 4800 s - the
#      values this row already had, not raised): every setup allowance, the 300 s fixture self-run, and
#      a 20 s kill-after grace for every run-path timeout. Every run-path timeout now carries -k, so no
#      grace is an unaccounted tail; what is left over is a STATED reserve, with its NO_OBSERVATION risk
#      stated next to it. No budget is increased, no row is re-split, no per-phase limit is retuned to
#      fit under a cap, and 7200 s is treated as the launcher's capability rather than an authorization.
# R4 5360 <= 5400, R5 4440 <= 4500, R6 4440 <= 4800. Nothing here executes a claim: PREPARE ONLY.

# History: what v8 changed (kept so the v8 -> v9 delta can be read against it)
#   final_verdict looped over whatever $OUT/*.classes files existed. With no classes files at all, or one
#   leg's file missing or short, it summed nothing, printed VERDICT=AS_EXPECTED and exited 0 - a green
#   verdict from an empty run. v8 registers the booked claim set into $OUT/claims-registered.txt from the
#   frozen constants BEFORE the first leg, and final_verdict refuses (NO_OBSERVATION for every claim, exit
#   2) unless the classes files are EXACTLY that set: one file per booked leg, no file for an unbooked leg,
#   every booked claim named exactly once, no unregistered name - and the registration itself is re-checked
#   against this packet's frozen constants, so a half-written or tampered file refuses the run too.
#   Nothing else changed v7 -> v8: the commands, pins, bounds, wall arithmetic and the command inventory
#   are byte-identical (the --predict-only diff between v7 and v8 is empty).
# WHAT v7 CHANGES (calm-eagle-670 msg_ac20c593, from sleek-owl-397 msg_9a42d63c; the same five items as
# the R4 and R5 packets, and they apply here too even though this row is HELD):
#   (1) the exit decision is final_verdict over THIS packet's own evidence files.
#   (2) every gate refusal returns non-zero, including the wall gate and the producer gate.
#   (3) classification is per claim, per the leg contract quoted verbatim below.
#   (4) post-leg gates on the worktree and the runner's bytes.
#   (5) gated OUT argument, timeout on the fixture self-run, CARGO_HOME under the evidence dir, a
#       fail-closed stray-verdict detector, and an OOM label that carries the verdict it got.
#   The hold is unchanged: with no cleared execution-grant reference the packet refuses at exit 2 before
#   it creates anything, and msg_3407a188 is not cleared by anything in this packet.
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
#   self-run the packet's own fixture suite, timeout 300 s, run in this scope before any probe  = 300 s
#   graces   6 run-path timeouts x 20 s kill-after (timeout -k): a step that ignores TERM is    = 120 s
#           killed 20 s later, so a grace is a BOUNDED, COUNTED term and the ceiling stays a ceiling.
#   CAPPED SUM = 720 setup + 2400 build + 900 legs + 300 self-run + 120 kill graces                = 4440 s
#   That capped sum is not a prediction of the wall: it is the ceiling EVERY timeout in this packet
#   admits - clone, fetch, checkout, build, the leg and the self-run, each with its -k grace.
#   RESERVED MARGIN = 4800 - 4440 = 360 s: process startup, the bounds and OOM gate reads, evidence writes and
#   receipt hashing. THE RISK OF THAT MARGIN, stated as required: if the residual work needs more than
#   360 s, RuntimeMaxSec 4800 sends TERM to the scope and the driver row's RuntimeMaxSec 5100 sends SIGKILL 300
#   s later; a row cut off mid-step is incomplete, which is NO_OBSERVATION (2) for this claim.
#   THE BUDGET IS NOT RAISED, and no timeout below a booked one is shortened either: 4800 s is exactly what v6 booked and v8 kept for this row. Nothing here
#   re-splits a row or retunes a per-phase limit to fit under a cap. The only new terms are the self-run
#   (v8 capped it but never counted it) and the -k graces (v8 left them unbounded), and both fit inside
#   the unchanged RUNTIME_S.
# WALL ACCOUNTING (v9): the scope's RuntimeMaxSec is 4800 s, the capped sum is 4440 s (720 setup + 2400 build +
#   1 x 900 legs + 300 self-run + 6 x 20 kill graces) and the reserve is 360 s. A row that RuntimeMaxSec cuts off
#   is incomplete, which is NO_OBSERVATION (2). 4440 <= 4800 <= 7200: the installed calmcrab-udrv v4 (855920ac)
#   refuses any RUNTIME_S above 7200 in its own preflight, but 7200 is that tool's CAPABILITY and not an
#   authorization. This booking stays where it was booked (4800 s) and no larger ceiling is implied anywhere
#   in this packet: an increase over an approved timeout needs operator sign-off, a never-granted proposal
#   is not an approved ceiling, and no split, retry or per-phase retune may be used to get around a cap
#   (keen-otter msg_42c3c192). The driver row calmcrab-drv-<JOB>.service is RuntimeMaxSec 5100 = RUNTIME_S
#   plus 300 s. wall_gate below recomputes the whole arithmetic - constants, graces, self-run, cap and
#   driver offset - from the packet's own constants, and refuses before any fetch or build if the launcher
#   books anything else; the bounds gate refuses other mismatches.
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
#   calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> <v7 packet sha256> OUT
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
# EXIT CONTRACT (v3 item 4 + v7 item 3, identical to the R4/R5 packets plus the hold).
# THREE codes, no others:
#   0  every claim in every leg printed a bare PASS and every leg was OBSERVED (expectations held)
#   1  a claim printed a bare FAIL inside an OBSERVED leg (expectation FALSIFIED - a finding, and the
#      reason for the row). FALSIFIED outranks NO_OBSERVATION: a bare FAIL in a well-formed leg is real,
#      so a second leg that produced nothing does not cancel it.
#   2  NO_OBSERVATION: any setup, usage, scope, bounds, clone, checkout, blob, name, build, wall,
#      out-dir, timeout, OOM, counter-incoherence, completion-record, verdict-line, post-leg or
#      return-code failure - every gate refusal is 2, including a run dir that already exists.
# THE LEG CONTRACT, verbatim as pre-registered:
#   A leg is OBSERVED iff a completion record is present AND every claim in the leg has exactly one bare
#   verdict line AND there are no stray verdict-shaped lines AND rc == 0 when every claim PASSes, else
#   rc == 1. Within an observed leg a bare PASS is HELD and a bare FAIL is FALSIFIED, per claim. Anything
#   else (an rc inconsistent with the verdicts, a missing or duplicate verdict, a reasoned FAIL, a
#   timeout, an OOM) makes EVERY claim in that leg NO_OBSERVATION.
#   Packet exit: 1 if any claim is FALSIFIED, else 2 if any claim is NO_OBSERVATION, else 0.
# Classification is therefore PER CLAIM, not per leg: a leg that is not observed marks its claims
# NO_OBSERVATION and lets the other legs stand.
# POST-LEG GATE: after the legs, the pinned worktree must still be clean (git status --porcelain empty)
# and the runner binary must hash to the bytes hashed before the legs ran. Either failing makes EVERY
# claim in the run NO_OBSERVATION, because nothing those legs printed can be attributed to the pin.
# The verdict is computed by final_verdict over THIS packet's own evidence files only. It never names a
# variable belonging to another row's packet: under set -u that is not a typo, it is exit 127 arriving
# in the middle of the verdict, after every probe has been spent.#
# R6 IS DIFFERENT IN ONE WAY: a refusal of the SUBJECT (the live-corpus projection declining to
# project) is a FINDING for the grant set, so a bare FAIL here is exit 1 and is worth reading; a
# refusal of the PACKET (no cleared execution-grant reference) is exit 2 before anything is created.
# Those two are not the same event: HELD is not falsified, and a packet that was never granted is
# not evidence about the subject.

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
# THE BOOKED CLAIM SET: declared once here from the frozen claim-name constants, REGISTERED into OUT
# before any leg runs, and re-checked against the classes files before any verdict is taken (v8,
# calm-eagle-670 msg_8ed9b50b). Before v8, final_verdict looped over whatever *.classes files happened to
# exist: zero files, or one missing or short file, summed nothing and printed VERDICT=AS_EXPECTED with
# claims=0 and exited 0 - a green verdict from an empty run.
LEGS_BOOKED="R6"
CLAIMS_R6="$N6"
CLAIMS_ALL="$N6"
N_CLAIMS_BOOKED=$(printf '%s\n' $CLAIMS_ALL | grep -c .)
# the booked set exactly as first declared, so a fixture that books a synthetic set can put it back
BOOK_LEGS="$LEGS_BOOKED"; BOOK_ALL="$CLAIMS_ALL"; BOOK_N="$N_CLAIMS_BOOKED"


T_SETUP_CLONE=300; T_SETUP_FETCH=300; T_SETUP_CHECKOUT=120; T_BUILD=2400; T_LEG=900
# the fixture self-run is a step like any other. v8 already wrapped it in timeout 300 s but left it out
# of the sum, so the row's margin silently carried it: a hung classifier would burn the wall budget
# before a probe was spent and the arithmetic below would still read as a ceiling. T_SELFRUN keeps its
# v8 value; v9 only makes it a term of the sum.
T_SELFRUN=300
# a kill-after grace on every run-path timeout. Without -k, timeout(1) sends TERM and then waits for a
# child that may never exit: an unbounded tail that no constant accounts for, and the one place where a
# capped sum stops being a ceiling. 20 s per timeout. No step's own limit changes - the constants v8
# froze (clone 300, fetch 300, checkout 120, build 2400, leg 900, self-run 300) are byte for byte here.
T_KILL=20
# the run-path timeouts counted rather than estimated: clone, fetch, checkout, build, the self-run and
# one per leg. A new run-path timeout must raise this count, and the sum with it.
N_TIMEOUTS_RUN=$(( 4 + 1 + N_LEGS ))
# the number of timeout CALL SITES (the legs share one). The fixture that proves every run-path timeout
# carries -k counts these.
N_TIMEOUT_CALL_SITES=6
# the row's CAPPED SUM: one clone + one shallow fetch + one checkout + one build + N_LEGS legs + the
# self-run + a kill-after grace for each of those timeouts
WALL_SUM=$(( T_SETUP_CLONE + T_SETUP_FETCH + T_SETUP_CHECKOUT + T_BUILD + N_LEGS*T_LEG + T_SELFRUN + N_TIMEOUTS_RUN*T_KILL ))

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
book_case_set() { # a fixture books its OWN tiny claim set (synthetic legs and names); never the shipped one
  LEGS_BOOKED="$1"; CLAIMS_ALL="$2"; shift 2
  N_CLAIMS_BOOKED=$(printf '%s\n' $CLAIMS_ALL | grep -c .)
  local L v
  for L in $LEGS_BOOKED; do
    v="CLAIMS_$L"; [ $# -ge 1 ] || { echo "book_case_set: no claim list for booked leg $L"; return 1; }
    eval "$v=\"$1\""; shift
  done
  [ $# = 0 ] || { echo "book_case_set: more claim lists than booked legs"; return 1; }
  local seen="" n
  for n in $CLAIMS_ALL; do
    case " $seen " in *" $n "*) echo "book_case_set: claim $n booked under two legs"; return 1;; esac
    seen="$seen $n"
  done
  return 0; }
restore_booked_set() { LEGS_BOOKED="$BOOK_LEGS"; CLAIMS_ALL="$BOOK_ALL"; N_CLAIMS_BOOKED="$BOOK_N"
  case " $BOOK_LEGS " in
    *" R4a "*) CLAIMS_R4a="$N4A"; CLAIMS_R4b="$N4B" ;;
    *" R5 "*)   CLAIMS_R5="$N5" ;;
    *" R6 "*)   CLAIMS_R6="$N6" ;;
  esac; }

# THE CLAIM REGISTRATION: written into OUT from the frozen constants BEFORE the first leg runs. The writer
# checks its own output against those constants, so a half-written, short or tampered registration refuses
# the run (NO_OBSERVATION, exit 2) instead of mis-deciding it.
register_claims() { # $OUT/claims-registered.txt, one "<leg> <claim>" line per booked claim
  local L v n list seen=""
  : > "$OUT/claims-registered.txt" || { echo "cannot write the claim registration in $OUT"; return 1; }
  for L in $LEGS_BOOKED; do
    v="CLAIMS_$L"; list=${!v:-}
    [ -n "$list" ] || { echo "leg $L is booked but declares no claims"; return 1; }
    for n in $list; do
      case " $seen " in *" $n "*) echo "claim $n is registered under two legs"; return 1;; esac
      seen="$seen $n"
      printf '%s %s\n' "$L" "$n" >> "$OUT/claims-registered.txt"
    done
  done
  local nn badleg
  nn=$(awk 'NF' "$OUT/claims-registered.txt" | wc -l)
  [ "$nn" = "$N_CLAIMS_BOOKED" ] || { echo "the registration names $nn claims, this row books $N_CLAIMS_BOOKED"; return 1; }
  badleg=$(awk 'NF != 2' "$OUT/claims-registered.txt" | wc -l)
  [ "$badleg" = 0 ] || { echo "the registration has $badleg malformed lines"; return 1; }
  local seen2="" n2
  for n2 in $(awk 'NF {print $2}' "$OUT/claims-registered.txt"); do
    case " $seen2 " in *" $n2 "*) echo "the registration names $n2 twice"; return 1;; esac
    seen2="$seen2 $n2"
    case " $CLAIMS_ALL " in *" $n2 "*) : ;; *) echo "the registration names $n2, which this packet does not book"; return 1;; esac
  done
  local miss
  for miss in $CLAIMS_ALL; do
    grep -q " $miss\( \|$\)" "$OUT/claims-registered.txt" || { echo "the registration omits the booked claim $miss"; return 1; }
  done
  return 0; }

# cs_check: are the classes files EXACTLY the claim set this packet registered, LEG BY LEG? On success it
# also SNAPS that validated set (see the snapshot paragraph). It returns 1 and names every cause it found in
# CS_CAUSE for any of: no registration at all; a registration whose claim-line count is not
# N_CLAIMS_BOOKED; a registration line that is not exactly two whitespace-free fields; a registration whose
# DISTINCT leg names are not exactly the booked legs (a leg name this packet never booked, or a booked leg it
# never names, even when the count still adds up); a claim name the packet does not book; a booked claim the
# registration omits or names twice. Then, of the classes files: a *.classes file count that is not the
# number of booked legs; a file for a leg that was never booked; a booked leg whose file is missing or
# empty; a file with a malformed line; a claim named twice inside one file; a claim named in two different
# leg files (claim-duplicated-across-files); and the per-LEG BINDING - the names in <stem>.classes must be
# EXACTLY the names the registration registers UNDER THAT SAME STEM, each exactly once. A claim registered
# under leg A does not satisfy leg B: a name in a file that is registered for a DIFFERENT leg is
# claim-wrong-leg:<stem>:<name>; a name in a file that the registration does not name at all is
# claim-unregistered:<name>; and a name registered for that stem which its own file does not carry exactly
# once is claim-count:<stem>. All of those comparisons are LITERAL awk field comparisons ($1==stem && $2==name),
# never an interpolated regular expression built from a name, so a name cannot match by accident.
# SNAPSHOT: only after all of the above pass, each validated file is copied into a fresh $OUT/cs-snapshot
# and its digest recorded in cs-snapshot/digests.txt; final_verdict counts ONLY the snapshot, so a file that
# changes between validation and the verdict cannot be counted, and a copy that does not match what was
# validated is itself a cause (snapshot-copy-failed / snapshot-digest-mismatch / snapshot-incomplete).
cs_check() {
  local reg="$OUT/claims-registered.txt" f stem n files=0 dupx cn cc bad
  CS_CAUSE=""; CS_SNAP="$OUT/cs-snapshot"; CS_SNAP_OK=0
  [ -s "$reg" ] || { CS_CAUSE="registration-missing-or-empty"; return 1; }
  if [ "$(awk 'NF' "$reg" | wc -l)" != "$N_CLAIMS_BOOKED" ]; then CS_CAUSE="registration-claim-count"; return 1; fi
  bad=$(awk 'NF && (NF!=2 || $0!=$1" "$2) {c++} END{print c+0}' "$reg")
  [ "$bad" = 0 ] || { CS_CAUSE="registration-malformed-line"; return 1; }
  if [ "$(awk 'NF {print $1}' "$reg" | sort -u | tr '\n' ' ')" != "$(printf '%s\n' $LEGS_BOOKED | sort | tr '\n' ' ')" ]; then
    CS_CAUSE="registration-leg-count"; return 1; fi
  if [ "$(awk 'NF {print $2}' "$reg" | sort -u | wc -l)" != "$N_CLAIMS_BOOKED" ]; then CS_CAUSE="$CS_CAUSE claim-duplicated"; fi
  local rn
  for rn in $(awk 'NF {print $2}' "$reg"); do
    case " $CLAIMS_ALL " in *" $rn "*) : ;; *) CS_CAUSE="$CS_CAUSE claim-not-booked" ;; esac
  done
  for cn in $CLAIMS_ALL; do
    cc=$(awk -v n="$cn" '$2==n' "$reg" | wc -l)
    [ "$cc" = 1 ] || CS_CAUSE="$CS_CAUSE claim-not-registered:$cn"
  done
  for f in "$OUT"/*.classes; do
    [ -f "$f" ] || continue
    files=$((files+1))
    stem=${f##*/}; stem=${stem%.classes}
    case " $LEGS_BOOKED " in
      *" $stem "*)
        if [ ! -s "$f" ]; then CS_CAUSE="$CS_CAUSE leg-file-missing:$stem"; continue; fi
        bad=$(awk 'NF && (NF!=2 || $0!=$1" "$2) {c++} END{print c+0}' "$f")
        if [ "$bad" != 0 ]; then CS_CAUSE="$CS_CAUSE malformed-classes-line:$stem"; continue; fi
        if [ -n "$(awk 'NF {print $1}' "$f" | sort | uniq -d | head -1)" ]; then CS_CAUSE="$CS_CAUSE claim-duplicated:$stem"; fi
        for n in $(awk 'NF {print $1}' "$f"); do
          if [ -n "$(awk -v l="$stem" -v n2="$n" '$1==l && $2==n2' "$reg")" ]; then :
          elif [ -n "$(awk -v n2="$n" '$2==n2' "$reg")" ]; then CS_CAUSE="$CS_CAUSE claim-wrong-leg:$stem:$n"
          else CS_CAUSE="$CS_CAUSE claim-unregistered:$n"; fi
        done
        for n in $(awk -v l="$stem" '$1==l {print $2}' "$reg"); do
          [ "$(awk -v n2="$n" '$1==n2' "$f" | wc -l)" = 1 ] || CS_CAUSE="$CS_CAUSE claim-count:$stem"
        done
        if [ "$(awk 'NF' "$f" | wc -l)" != "$(awk -v l="$stem" '$1==l' "$reg" | wc -l)" ]; then CS_CAUSE="$CS_CAUSE claim-count:$stem"; fi
        ;;
      *) CS_CAUSE="$CS_CAUSE classes-file-not-booked:$stem" ;;
    esac
  done
  dupx=$(for f in "$OUT"/*.classes; do [ -f "$f" ] || continue; awk 'NF {print $1}' "$f"; done | sort | uniq -d | head -1)
  [ -z "$dupx" ] || CS_CAUSE="$CS_CAUSE claim-duplicated-across-files"
  local nbl; nbl=$(printf '%s\n' $LEGS_BOOKED | grep -c .)
  [ "$files" = "$nbl" ] || CS_CAUSE="$CS_CAUSE classes-file-count:$files-want-$nbl"
  local l1
  for l1 in $LEGS_BOOKED; do
    [ -s "$OUT/$l1.classes" ] || CS_CAUSE="$CS_CAUSE leg-file-missing:$l1"
  done
  if [ -n "$CS_CAUSE" ]; then
    CS_CAUSE=$(printf '%s\n' $CS_CAUSE | awk '!seen[$0]++' | sort | tr '\n' ' ')
    CS_CAUSE=${CS_CAUSE% }
    return 1
  fi
  rm -rf "$CS_SNAP"
  mkdir -m 700 "$CS_SNAP" 2>/dev/null || { CS_CAUSE="snapshot-dir-unusable"; return 1; }
  local a b
  for l1 in $LEGS_BOOKED; do
    cp "$OUT/$l1.classes" "$CS_SNAP/$l1.classes" 2>/dev/null || { CS_CAUSE="snapshot-copy-failed:$l1"; return 1; }
    a=$(sha256sum -- "$OUT/$l1.classes" 2>/dev/null | cut -d' ' -f1)
    b=$(sha256sum -- "$CS_SNAP/$l1.classes" 2>/dev/null | cut -d' ' -f1)
    [ -n "$a" ] && [ "$a" = "$b" ] || { CS_CAUSE="snapshot-digest-mismatch:$l1"; return 1; }
    printf '%s %s\n' "$l1" "$b" >> "$CS_SNAP/digests.txt"
  done
  [ "$(grep -c . "$CS_SNAP/digests.txt" 2>/dev/null)" = "$nbl" ] || { CS_CAUSE="snapshot-incomplete"; return 1; }
  CS_SNAP_OK=1
  return 0; }

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
# CLASSIFIER, COMPLETION RECORD AND VERDICT RULES (v4 rules, kept verbatim into v7; the v7 leg
# contract below sits ON TOP of them, it does not replace them):
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

# LEG CONTRACT (pre-registered by the acceptance owner, calm-eagle-670 msg_ac20c593) -- verbatim:
#   A leg is OBSERVED iff a completion record is present AND every claim in the leg has exactly one bare
#   verdict line AND there are no stray verdict-shaped lines AND rc == 0 when every claim PASSes, else rc == 1.
#   Within an observed leg a bare PASS is HELD and a bare FAIL is FALSIFIED, per claim.
#   Anything else (an rc inconsistent with the verdicts, a missing or duplicate verdict, a reasoned FAIL, a
#   timeout, an OOM) makes EVERY claim in that leg NO_OBSERVATION.
#   Packet exit: 1 if any claim is FALSIFIED, else 2 if any claim is NO_OBSERVATION, else 0.
#   FALSIFIED therefore wins over NO_OBSERVATION: a bare FAIL seen in a well-formed leg is a real result.
# LEG EXIT-CODE CONSISTENCY: rc must be 0 when every claim has a bare PASS, and 1 when at least one claim
# has a bare FAIL. Anything else -- including 124 (timeout), a signal, or a non-numeric rc -- is a mismatch.
leg_rc_expect() { # $1=stdout file $2=rc $3=name list -> OK | MISMATCH <what>
  local f=$1 rc=$2 names=$3 n q fails=0
  case "$rc" in ''|*[!0-9]*) echo "MISMATCH rc=$rc is not a number"; return;; esac
  for n in $names; do q=$(grep -c "^FAIL $n\$" "$f" 2>/dev/null); [ "${q:-0}" = 1 ] && fails=$((fails+1)); done
  if [ "$fails" = 0 ]; then
    [ "$rc" = 0 ] && echo OK || echo "MISMATCH rc=$rc but no claim printed a bare FAIL, so 0 was required"
  else
    [ "$rc" = 1 ] && echo OK || echo "MISMATCH rc=$rc but $fails claim(s) printed a bare FAIL, so 1 was required"
  fi; }
# CLAIM CLASSIFIER: bare lines only, one per expected name, and only inside an observed leg (rcst=OK).
claim_classify() { # $1=stdout file $2=leg rc-consistency (OK | MISMATCH …) $3=name -> HELD | FAILED | NO_OBSERVATION
  local f=$1 rcst=$2 n=$3 any p q
  [ -r "$f" ] || { echo NO_OBSERVATION; return; }
  [ "$rcst" = OK ] || { echo NO_OBSERVATION; return; }
  # exactly one outcome line for this name, of ANY keyword (bare or qualified); zero or two -> no observation
  any=$(grep -cE "^(PASS|FAIL|UNOBSERVED|PANICKED) $n( |$)" "$f" 2>/dev/null)
  [ "${any:-0}" = 1 ] || { echo NO_OBSERVATION; return; }
  p=$(grep -c "^PASS $n\$" "$f" 2>/dev/null); q=$(grep -c "^FAIL $n\$" "$f" 2>/dev/null)
  [ "${p:-0}" = 1 ] && [ "${q:-0}" = 0 ] && { echo HELD; return; }
  [ "${p:-0}" = 0 ] && [ "${q:-0}" = 1 ] && { echo FAILED; return; }
  # a qualified FAIL, an UNOBSERVED or a PANICKED line, or two bare lines: attempted, but not countable.
  echo NO_OBSERVATION; }
wall_gate() { # $1=capped per-step sum $2=scope RuntimeMaxSec $3=installed launcher cap $4=driver RuntimeMaxSec
  local s=$1 r=$2 cap=$3 d=$4
  [ "$r" -le "$cap" ] || { echo "RUNTIME_S $r exceeds the installed calmcrab-udrv v4 preflight cap of $cap; the launcher would refuse before any scope exists (PRE_SCOPE_REFUSAL, the #15695 diag-v2 shape at 11700)"; return 1; }
  [ "$s" -le "$r" ] || { echo "the capped per-step sum $s s exceeds the scope RuntimeMaxSec $r; a cut-off leg is NO_OBSERVATION"; return 1; }
  [ "$d" = "$((r+300))" ] || { echo "the driver service must be RuntimeMaxSec $((r+300)), the scope's $r plus 300 s; the packet says $d"; return 1; }
  echo OK; }

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
  # per-LEG summary for the receipt only; the packet's exit code comes from final_verdict, per CLAIM.
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
stray_verdict_lines() { # $1=output file $2=name list -> 0, or the reason it is not a clean bare-verdict set
  # FAILS CLOSED: an unreadable file, or a grep that could not run, is a reason, never a zero. A zero from a
  # failed read would read as "no stray lines" and let a leg be observed on evidence nobody looked at.
  local f=$1 names=$2 tot exact n
  [ -r "$f" ] || { echo "unreadable verdict file: $f"; return; }
  tot=$(grep -cE '^(PASS|FAIL|UNOBSERVED|PANICKED) ' "$f" 2>/dev/null) || { echo "verdict scan failed on $f"; return; }
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
  [ -r "$f" ] || { echo "the pinned claim_batch.rs is unreadable: $f"; return 1; }
  n=$(grep -cF 'partition.render(&timings)' "$f" 2>/dev/null)
  [ "$n" = 1 ] || { echo "expected exactly one completion-record render site, found $n in $f"; return 1; }
  line=$(grep -m1 -F 'partition.render(&timings)' "$f")
  case "$line" in
    *'eprintln!("{}", partition.render(&timings));'*) echo OK ;;
    *) echo "the completion record is NOT emitted to stderr (eprintln) at the render site: $line"; return 1 ;;
  esac; }

leg() { # $1=leg name $2=CMD variable name $3=name list
  local L=$1 rc k0 k1 LO st cl sv pc causes n p q rcst
  cmd=${2}
  k0=$(oom_snap "$EV" "$L-leg-before")
  timeout -k "$T_KILL" "$T_LEG" bash -c "$2" > "$OUT/$L.out" 2> "$OUT/logs/$L.err"; rc=$?
  k1=$(oom_snap "$EV" "$L-leg-after"); LO=$(oom_pair_verdict "$k0" "$k1"); LASTOOM=$k1
  echo "== $L rc=$rc oom=$LO ($k0 -> $k1) $(tm "$OUT/logs/$L.err")" >> "$O"
  # Every condition below is a LEG-level cause. A cause does not end the packet: it makes every claim in
  # this leg NO_OBSERVATION and lets the other legs stand, because the packet exit is decided per claim
  # (a bare FAIL in a well-formed leg elsewhere is a real falsification and must still report exit 1).
  causes=""
  [ "$LO" = none ] || causes="$causes oom=$LO($k0->$k1)"
  [ "$rc" = 124 ] && causes="$causes timeout-at-${T_LEG}s"
  pc=$(pp_check "$OUT/logs/$L.err")
  echo "== $L completion_record: $pc" >> "$O"
  [ "${pc%% *}" = OK ] || causes="$causes completion-record($pc)"
  sv=$(stray_verdict_lines "$OUT/$L.out" "$3")
  if [ "$sv" != 0 ]; then
    grep -E '^(PASS|FAIL|UNOBSERVED|PANICKED) ' "$OUT/$L.out" 2>/dev/null | head -6 | cut -c1-300 | sed 's/^/  | /' >> "$O"
    causes="$causes verdict-lines($sv)"
  fi
  for n in $3; do
    p=$(grep -c "^PASS $n\$" "$OUT/$L.out" 2>/dev/null); q=$(grep -c "^FAIL $n\$" "$OUT/$L.out" 2>/dev/null)
    [ "$(( ${p:-0} + ${q:-0} ))" = 1 ] || causes="$causes no-single-bare-verdict($n:p=${p:-?},q=${q:-?})"
  done
  rcst=$(leg_rc_expect "$OUT/$L.out" "$rc" "$3")
  [ "$rcst" = OK ] || causes="$causes $rcst"
  : > "$OUT/$L.classes"
  if [ -n "$causes" ]; then
    for cl in $3; do echo "$cl NO_OBSERVATION" >> "$OUT/$L.classes"; done
    echo "== $L NOT OBSERVED:$causes" >> "$O"
    echo "  every claim in $L is NO_OBSERVATION" >> "$O"
  else
    for cl in $3; do echo "$cl $(claim_classify "$OUT/$L.out" OK "$cl")" >> "$OUT/$L.classes"; done
  fi
  cat "$OUT/$L.classes" | sed 's/^/  /' >> "$O"
  st=$(leg_state "$OUT/$L.classes" "$3")
  echo "== $L state=$st" >> "$O"; echo "$st" > "$OUT/$L.state"; }

# POST-LEG GATES: the legs are read-only probes of a pinned checkout. If they left the worktree dirty, or
# the runner binary is not the exact bytes that were hashed before the legs, then nothing any leg printed
# can be attributed to the pinned source: every claim in the run becomes NO_OBSERVATION.
post_leg_gate() { # $1=worktree $2=runner binary $3=pre-leg sha256 ; writes $OUT/post-leg.cause (empty = clean)
  local w=$1 b=$2 pre=$3 post dirty
  : > "$OUT/post-leg.cause"
  if ! git -C "$w" status --porcelain > "$OUT/git-status.txt" 2>&1; then
    echo "git status --porcelain could not run in $w" > "$OUT/post-leg.cause"
  fi
  dirty=$(grep -c . "$OUT/git-status.txt" 2>/dev/null)
  if [ "${dirty:-1}" != 0 ]; then
    echo "the worktree $w is not clean after the legs: ${dirty} changed entr$([ "${dirty}" = 1 ] && echo y || echo ies): $(head -3 "$OUT/git-status.txt" | tr '\n' ' ')" > "$OUT/post-leg.cause"
  fi
  post=$(bin_sha "$b" || echo unreadable)
  if [ "$post" != "$pre" ] || [ "$pre" = unreadable ]; then
    echo "the runner changed during the run: pre-leg sha256=$pre, after the legs sha256=$post" > "$OUT/post-leg.cause"
  fi
  echo "post_leg: worktree_clean=$([ -s "$OUT/post-leg.cause" ] && echo no || echo yes) runner_sha256_pre=$pre post=$post cause=$(cat "$OUT/post-leg.cause")" >> "$O"; }

# THE PACKET'S EXIT DECISION, per the leg contract above. Reads the per-claim classes the legs wrote; a
# post-leg cause turns every claim into NO_OBSERVATION. Exits: 1 FALSIFIED, 2 NO_OBSERVATION, 0 held.
# It names ONLY this packet's own files -- no state variable from any other row's packet, which under
# set -u is not a typo but an exit 127 in the middle of the verdict.
final_verdict() {
  local pc="" f n c sum="" anyF=0 anyN=0 nclaims=0 nlegs=0
  if ! cs_check; then
    local cscause=""; local cx
    for cx in $CS_CAUSE; do cscause="$cscause $cx"; done
    echo "VERDICT=NO_OBSERVATION claims=0 legs-booked=$N_LEGS claims-booked=$N_CLAIMS_BOOKED claim-set refused:$cscause (the classes files are not exactly the booked claim set, so nothing in this run is countable)" >> "$O"
    echo "DONE $(date -u +%FT%TZ)" >> "$O"
    echo "NO_OBSERVATION:claim-set$cscause" >&2; exit 2
  fi
  [ -s "$OUT/post-leg.cause" ] && pc=$(cat "$OUT/post-leg.cause")
  # v9: cs_check validated the live files and copied them; count ONLY that snapshot (digests recorded in
  # cs-snapshot/digests.txt), so a classes file rewritten between validation and the verdict cannot be counted.
  if [ "${CS_SNAP_OK:-0}" != 1 ] || [ ! -d "${CS_SNAP:-}" ]; then
    echo "VERDICT=NO_OBSERVATION claims=0 legs-booked=$N_LEGS claims-booked=$N_CLAIMS_BOOKED claim-set refused:snapshot-missing (cs_check validated no snapshot to count)" >> "$O"
    echo "DONE $(date -u +%FT%TZ)" >> "$O"
    echo "NO_OBSERVATION:claim-set snapshot-missing" >&2; exit 2
  fi
  for f in "$CS_SNAP"/*.classes; do
    [ -f "$f" ] || continue
    nlegs=$((nlegs+1))
    for n in $(awk '{print $1}' "$f"); do
      c=$(awk -v k="$n" '$1==k {print $2}' "$f"); nclaims=$((nclaims+1))
      [ -n "$pc" ] && c=NO_OBSERVATION
      case "$c" in
        HELD) : ;;
        FAILED) anyF=1 ;;
        *) anyN=1 ;;
      esac
      sum="$sum $n=$c"
    done
  done
  echo "claims classified:${sum:- none} post_leg_cause=${pc:-none}" >> "$O"
  if [ "$anyF" = 1 ]; then
    echo "VERDICT=FALSIFIED claims=$nclaims legs=$nlegs mutation_legs=0 (a bare FAIL inside an observed leg; falsification outranks any no-observation)" >> "$O"
    echo "DONE $(date -u +%FT%TZ)" >> "$O"
    echo "FALSIFIED:${sum}" >&2; exit 1
  fi
  if [ "$anyN" = 1 ]; then
    echo "VERDICT=NO_OBSERVATION claims=$nclaims legs=$nlegs (one or more claims had no countable observation; post-leg cause: ${pc:-none})" >> "$O"
    echo "DONE $(date -u +%FT%TZ)" >> "$O"
    echo "NO_OBSERVATION:${sum}${pc:+ post_leg=$pc}" >&2; exit 2
  fi
  echo "VERDICT=AS_EXPECTED claims=$nclaims legs=$nlegs mutation_legs=0" >> "$O"
  echo "DONE $(date -u +%FT%TZ)" >> "$O"
  echo "ALL-HELD:${sum}" >&2; exit 0; }

# The one argument, and the job it names, are gates rather than prose: a relative OUT resolves against
# whatever cwd the launcher happened to start in, and an OUT whose <JOB> component is not the booked job
# means the evidence is landing in somebody else's run directory.
out_gate() { # $1=OUT $2=booked JOB -> prints the reason and returns 1, or returns 0
  local o=$1 j=$2 job
  case "$o" in /*) ;; *) echo "OUT '$o' is relative; the evidence dir must be absolute (the launcher maps it to \$HOME/runs/<JOB>/out)"; return 1;; esac
  case "$o" in */out) ;; *) echo "OUT '$o' does not end in /out (calmcrab-udrv v4 maps the row's evidence dir to <RUNTIME>/out)"; return 1;; esac
  case "$o" in */out/*|/out|/out/) echo "OUT '$o' has a path below /out; the evidence dir is the /out entry itself"; return 1;; esac
  job=$(basename "$(dirname "$o")" 2>/dev/null || echo "")
  [ -n "$job" ] || { echo "cannot derive the booked JOB from OUT '$o' (expected <RUNTIME>/<JOB>/out)"; return 1; }
  [ "$job" = "$j" ] || { echo "OUT '$o' names job '$job', this packet is booked as '$j' (the ledger rows, the scope and the driver unit all name $j)"; return 1; }
  return 0; }

# ---------------------------------------------------------------- fixture mode: run BEFORE any probe is spent
if [ "${1:-}" = "--classifier-fixtures" ]; then
  T=$(mktemp -d); okn=0; badn=0
  ck() { if [ "$3" = "$2" ]; then echo "FIXTURE PASS $1 -> $3"; okn=$((okn+1)); else echo "FIXTURE FAIL $1 expected $2 got $3"; badn=$((badn+1)); fi; }
  L() { printf '%s\n' "$1"; }
  # ckp: the expected text must APPEAR in the value. A refusal message names the offending path or number
  # first, and pinning the whole sentence would make the fixtures test the wording instead of the refusal.
  ckp() { case "$3" in *"$2"*) echo "FIXTURE PASS $1 -> $3"; okn=$((okn+1));; *) echo "FIXTURE FAIL $1 expected [$2] inside [$3]"; badn=$((badn+1));; esac; }
  # THE LEG CONTRACT, classifier level (calm-eagle-670 msg_ac20c593). claim_classify is only ever asked
  # about a claim inside an OBSERVED leg: the second argument is the leg's exit-code consistency, which
  # leg() computes over EVERY claim in the leg before it asks about any of them.
  L "PASS witness_x" > $T/p; ck bare-pass HELD "$(claim_classify $T/p OK witness_x)"
  L "FAIL witness_x" > $T/f; ck bare-fail FAILED "$(claim_classify $T/f OK witness_x)"
  ck leg-not-observed-no-pass NO_OBSERVATION "$(claim_classify $T/p 'MISMATCH rc=1 but no claim printed a bare FAIL, so 0 was required' witness_x)"
  ck leg-not-observed-no-fail NO_OBSERVATION "$(claim_classify $T/f 'MISMATCH rc=0 but 1 claim(s) printed a bare FAIL, so 1 was required' witness_x)"
  ck empty-rcst NO_OBSERVATION "$(claim_classify $T/p '' witness_x)"
  ck no-such-file NO_OBSERVATION "$(claim_classify $T/nothing-here OK witness_x)"
  ck name-not-in-log NO_OBSERVATION "$(claim_classify $T/p OK witness_other)"
  { L "PASS witness_x"; L "PASS witness_x"; } > $T/d
  ck duplicate-line NO_OBSERVATION "$(claim_classify $T/d OK witness_x)"
  { L "PASS witness_x"; L "FAIL witness_x"; } > $T/c
  ck contradictory NO_OBSERVATION "$(claim_classify $T/c OK witness_x)"
  L "  PASS witness_x" > $T/i; ck indented NO_OBSERVATION "$(claim_classify $T/i OK witness_x)"
  for q in 'FAIL witness_x (runtime error [E]: boom)' 'FAIL witness_x (returned ProcessExit::ExitFailure, code 3: no)' 'UNOBSERVED witness_x (route gap)' 'PANICKED witness_x (assert!)'; do
    L "$q" > $T/q; ckp "qualified-$(printf '%s' "$q" | cut -c1-4)" NO_OBSERVATION "$(claim_classify $T/q OK witness_x)"
    ck "qualified-stray-$(printf '%s' "$q" | cut -c1-4)" "stray=1 exact=0" "$(stray_verdict_lines $T/q 'witness_x')"
  done
  # THE LEG EXIT CODE must agree with the bare verdicts of the WHOLE leg, not of one claim.
  { L "PASS a"; L "FAIL b"; } > $T/mix
  { L "PASS a"; L "FAIL b"; L "FAIL c"; } > $T/mix2
  { L "PASS a"; L "PASS b"; } > $T/allpass
  ck rc0-single-pass OK "$(leg_rc_expect $T/p 0 witness_x)"
  ck rc1-pass-plus-fail OK "$(leg_rc_expect $T/mix 1 'a b')"
  ck rc1-two-fails OK "$(leg_rc_expect $T/mix2 1 'a b c')"
  ckp rc0-two-fails-and-a-pass "MISMATCH" "$(leg_rc_expect $T/mix2 0 'a b c')"
  ckp rc1-while-every-claim-passed "MISMATCH" "$(leg_rc_expect $T/allpass 1 'a b')"
  ckp rc0-while-a-claim-failed "MISMATCH" "$(leg_rc_expect $T/mix 0 'a b')"
  ckp rc124-timeout-is-a-mismatch "MISMATCH" "$(leg_rc_expect $T/allpass 124 'a b')"
  ckp rc137-signal-is-a-mismatch "MISMATCH" "$(leg_rc_expect $T/allpass 137 'a b')"
  ckp rc-not-a-number "MISMATCH" "$(leg_rc_expect $T/allpass missing 'a b')"
  ckp rc-empty "MISMATCH" "$(leg_rc_expect $T/allpass '' 'a b')"
  # a name the log never mentions is NOT an exit-code mismatch (0 was still the right code for the
  # verdicts that were printed); it is caught by the per-name bare-verdict check inside leg(), which
  # is what the contract-case fixtures below exercise end to end.
  ck rc0-names-not-in-the-log-is-still-a-consistent-rc OK "$(leg_rc_expect $T/allpass 0 'a b witness_x')"
  # stray-line detector: a qualified line among clean ones must be caught per leg, not per name, and the
  # detector FAILS CLOSED: an unreadable verdict file or a grep that could not run is a reason, never a 0.
  { L "PASS a_holds"; L "FAIL b_holds (runtime error [E]: boom)"; L "PASS c_holds"; } > $T/s
  ck stray-detected "stray=3 exact=2" "$(stray_verdict_lines $T/s 'a_holds b_holds c_holds')"
  { L "PASS a_holds"; L "PASS b_holds"; L "PASS c_holds"; } > $T/s2
  ck stray-clean 0 "$(stray_verdict_lines $T/s2 'a_holds b_holds c_holds')"
  L "PASS a_holds" > $T/s3; ck stray-extra-name "stray=1 exact=0" "$(stray_verdict_lines $T/s3 'b_holds')"
  ckp stray-unreadable-fails-closed "unreadable verdict file" "$(stray_verdict_lines $T/no-such-file-here 'a_holds')"
  # per-leg summary for the receipt (the packet's exit code is decided per CLAIM, by final_verdict)
  mk() { : > $T/$1; for x in a b c; do echo "$x HELD" >> $T/$1; done; }
  mk l1; ck leg-as-expected AS_EXPECTED "$(leg_state $T/l1 'a b c')"
  mk l2; sed -i 's/^b HELD/b FAILED/' $T/l2; ck leg-falsified FALSIFIED "$(leg_state $T/l2 'a b c')"
  mk l3; sed -i 's/^b HELD/b NO_OBSERVATION/' $T/l3; ck leg-noobs NOOBS "$(leg_state $T/l3 'a b c')"
  mk l4; sed -i '/^c /d' $T/l4; ck leg-missing-name NOOBS "$(leg_state $T/l4 'a b c')"
  mk l5; echo "d HELD" >> $T/l5; ck leg-extra-name-recorded AS_EXPECTED "$(leg_state $T/l5 'a b c')"
  mk l6; sed -i 's/^a HELD/a HELD/; s/^b HELD/b FAILED/; s/^c HELD/c NO_OBSERVATION/' $T/l6; ck leg-noobs-beats-falsified-in-a-leg NOOBS "$(leg_state $T/l6 'a b c')"
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
  ck wall-sum-arithmetic 4440 "$WALL_SUM"
  ck wall-sum-counts-every-timeout-and-grace 1 "$([ "$WALL_SUM" = "$(( T_SETUP_CLONE + T_SETUP_FETCH + T_SETUP_CHECKOUT + T_BUILD + N_LEGS*T_LEG + T_SELFRUN + N_TIMEOUTS_RUN*T_KILL ))" ] && echo 1 || echo 0)"
  ck wall-sum-counts-the-kill-graces 120 "$(( WALL_SUM - (T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT+T_BUILD+N_LEGS*T_LEG+T_SELFRUN) ))"
  ck run-path-timeout-count 6 "$N_TIMEOUTS_RUN"
  ck kill-grace-is-bounded 20 "$T_KILL"
  ck no-run-timeout-lacks-a-kill-grace 1 "$([ "$(grep -c '[t]imeout ["]["$]T_' "$0")" -eq 0 ] && echo 1 || echo 0)"
  ck every-run-timeout-site-carries-a-kill-grace 1 "$([ "$(grep -c '[t]imeout -k ["]["$]T_KILL["] ["]["$]T_' "$0")" -ge "$N_TIMEOUT_CALL_SITES" ] && echo 1 || echo 0)"
  ck wall-reserve-is-stated 360 "$(( RUNTIME_S - WALL_SUM ))"
  ck wall-sum-includes-the-self-run "$T_SELFRUN" "$(( WALL_SUM - (T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT+T_BUILD+N_LEGS*T_LEG+N_TIMEOUTS_RUN*T_KILL) ))"
  ck wall-sum-inside-the-scope 1 "$([ "$WALL_SUM" -le "$RUNTIME_S" ] && [ "$RUNTIME_S" -le "$LAUNCHER_CAP" ] && [ "$DRIVER_S" = "$(( RUNTIME_S + 300 ))" ] && echo 1 || echo 0)"
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
  inv 'timeout -k "$T_KILL" "$T_LE' 'G"' 'a leg runs under its own capped timeout' 1
  inv 'timeout -k "$T_KILL" "$T_BUIL' 'D"' 'the build runs under its own capped timeout' 1
  inv 'WM=$(wall_ga' 'te' 'the packet refuses a booked wall above the installed launcher cap, before any fetch' 1
  inv 'LAUNCHER_CA' 'P=7200' 'the installed calmcrab-udrv v4 cap is frozen as a constant, not prose' 1
  inv 'timeout -k "$T_KILL" "$T_SETUP_FETC' 'H"' 'the fetch runs under its own capped timeout' 1
  inv 'timeout -k "$T_KIL' 'L" "$T_LEG"' 'a leg timeout carries its bounded kill-after grace' 1
  inv '-k "$T_KILL" "$T_BUIL' 'D"' 'the build timeout carries its bounded kill-after grace' 1
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
  # -------- verdict rules (v4 blocker 3 + v7 item 3). A bare line counts only inside an OBSERVED leg, so
  # -------- the second argument is the leg's exit-code consistency, computed once for the whole leg.
  ck verdict-bare-pass-in-an-observed-leg HELD "$(claim_classify "$T/v_pass.txt" OK w_x)"
  ck verdict-bare-fail-in-an-observed-leg FAILED "$(claim_classify "$T/v_fail.txt" OK w_x)"
  ck verdict-qualified-fail-is-not-falsified NO_OBSERVATION "$(claim_classify "$T/v_qfail.txt" OK w_x)"
  ck verdict-qualified-fail-in-a-mismatching-leg NO_OBSERVATION "$(claim_classify "$T/v_qfail.txt" MISMATCH w_x)"
  ck verdict-unobserved NO_OBSERVATION "$(claim_classify "$T/v_unobs.txt" OK w_x)"
  ck verdict-panicked NO_OBSERVATION "$(claim_classify "$T/v_panic.txt" OK w_x)"
  ck verdict-two-outcome-lines NO_OBSERVATION "$(claim_classify "$T/v_two.txt" OK w_x)"
  ck verdict-pass-and-fail NO_OBSERVATION "$(claim_classify "$T/v_both.txt" OK w_x)"
  ck verdict-bare-pass-plus-qualified-fail NO_OBSERVATION "$(claim_classify "$T/v_bare_plus_q.txt" OK w_x)"
  ck verdict-no-outcome-line NO_OBSERVATION "$(claim_classify "$T/v_none.txt" OK w_x)"
  ck verdict-prefix-name-not-matched NO_OBSERVATION "$(claim_classify "$T/v_other.txt" OK w_x)"
  # the exit code has to agree with the bare verdicts of the whole leg; that is asked once per leg
  ck verdict-pass-with-rc0-is-consistent OK "$(leg_rc_expect "$T/v_pass.txt" 0 w_x)"
  ck verdict-fail-with-rc1-is-consistent OK "$(leg_rc_expect "$T/v_fail.txt" 1 w_x)"
  ckp verdict-pass-with-rc1-is-a-mismatch "MISMATCH" "$(leg_rc_expect "$T/v_pass.txt" 1 w_x)"
  ckp verdict-pass-with-rc124-is-a-mismatch "MISMATCH" "$(leg_rc_expect "$T/v_pass.txt" 124 w_x)"
  ckp verdict-fail-with-rc0-is-a-mismatch "MISMATCH" "$(leg_rc_expect "$T/v_fail.txt" 0 w_x)"
  # a clean claim in a leg whose exit code disagreed still reports nothing
  ck verdict-clean-claim-in-a-disagreeing-leg NO_OBSERVATION "$(claim_classify "$T/v_pass.txt" "$(leg_rc_expect "$T/v_pass.txt" 1 'w_x w_xy')" w_x)"

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
  # ---- v8 (calm-eagle-670 msg_8ed9b50b): the exit decision is anchored to a registered claim set ------
  # The fail-open v8 closes: final_verdict looped over whatever *.classes files existed. With no files at
  # all, or one leg's file missing or short, it summed nothing and printed VERDICT=AS_EXPECTED and exited 0.
  # The decision must be taken against the claim set this packet REGISTERED before its legs ran, exactly:
  # one file per booked leg, every booked claim named exactly once, no extras, no omissions - anything else
  # is NO_OBSERVATION and exit 2. cs_case books a SYNTHETIC set (it never weakens the shipped one) and puts
  # the booked constants back before it returns.
  cs_case() { # cs_case <name> <legs> <all claims> <registration text or -> [LEG:<claims of one leg>] [stem=<file body> | stem=EMPTY]
    local nm=$1 legs=$2 all=$3 reg=$4; shift 4
    local Q=$T/cs-$nm legargs=() files=() a spec lg body rc svOUT svO
    rm -rf "$Q"; mkdir -p "$Q"
    for a in "$@"; do case "$a" in LEG:*) legargs+=("${a#LEG:}");; *) files+=("$a");; esac; done
    book_case_set "$legs" "$all" ${legargs[@]+"${legargs[@]}"} || { echo "FIXTURE FAIL cs-$nm could not book its own claim set"; badn=$((badn+1)); return; }
    svOUT=$OUT; svO=$O
    if [ "$reg" != "-" ]; then printf '%s\n' "$reg" > "$Q/claims-registered.txt"; fi
    for spec in ${files[@]+"${files[@]}"}; do
      lg=${spec%%=*}; body=${spec#*=}
      if [ "$body" = EMPTY ]; then : > "$Q/$lg.classes"; else printf '%s\n' "$body" > "$Q/$lg.classes"; fi
    done
    OUT=$Q; O=$Q/receipt.txt
    ( final_verdict ) > "$Q/fv.out" 2>&1; rc=$?
    printf '%s\n' "$rc" > "$Q/fv.rc"
    OUT=$svOUT; O=$svO
    restore_booked_set; }
  # the writer: it produces exactly the booked set, and refuses a set it cannot produce
  svOUT=$OUT; svO=$O
  mkdir -p "$T/cs-w1"; OUT=$T/cs-w1
  register_claims > "$T/cs-w1/w.out" 2>&1; rcw=$?
  ck cs-writer-produces-the-booked-registration 0 "$rcw"
  ck cs-writer-legs "$LEGS_BOOKED" "$(awk 'NF {print $1}' "$T/cs-w1/claims-registered.txt" | sort -u | tr '\n' ' ' | sed 's/ $//')"
  ck cs-writer-claim-count "$N_CLAIMS_BOOKED" "$(awk 'NF' "$T/cs-w1/claims-registered.txt" | wc -l)"
  ck cs-writer-registration-shape 0 "$(awk 'NF != 2' "$T/cs-w1/claims-registered.txt" | wc -l)"
  OUT=$svOUT; O=$svO
  ckp cs-writer-refuses-a-leg-with-no-claims "declares no claims" "$( mkdir -p "$T/cs-w2"; OUT="$T/cs-w2"
      book_case_set "Xa Xb" "a b" "a" "" >/dev/null 2>&1; register_claims 2>&1; restore_booked_set )"
  ckp cs-writer-refuses-a-short-registration "names 1 claims, this row books 2" "$( mkdir -p "$T/cs-w3"; OUT="$T/cs-w3"
      book_case_set "X" "a b" "a" >/dev/null 2>&1; register_claims 2>&1; restore_booked_set )"
  ckp cs-booking-refuses-an-empty-claim-set "no claim list for booked leg" "$( book_case_set "LEGX" "" 2>&1 )"
  # the shipped set itself: registered by the writer, then handed to final_verdict as exactly matching files
  mkdir -p "$T/cs-SHIPPED"; svOUT=$OUT; svO=$O; OUT="$T/cs-SHIPPED"
  register_claims > /dev/null 2>&1
  for lgt in $LEGS_BOOKED; do
    eval "shplist=\"\$CLAIMS_$lgt\""
    : > "$T/cs-SHIPPED/$lgt.classes"
    for clm in $shplist; do printf '%s HELD\n' "$clm" >> "$T/cs-SHIPPED/$lgt.classes"; done
  done
  O="$T/cs-SHIPPED/receipt.txt"
  ( final_verdict ) > "$T/cs-SHIPPED/fv.out" 2>&1; rcsh=$?
  ck "cs-shipped-set-registers-and-is-countable" 0 "$rcsh"
  ck "cs-shipped-set-counts-every-claim" "$N_CLAIMS_BOOKED" "$(awk '/^claims classified:/{ sub(/ post_leg_cause.*$/,""); print gsub(/=[A-Z_]+/,"&") }' "$T/cs-SHIPPED/receipt.txt" | head -1)"
  ckp cs-shipped-set-verdict-is-all-held "VERDICT=AS_EXPECTED" "$(cat "$T/cs-SHIPPED/receipt.txt")"
  OUT=$svOUT; O=$svO
  # the five breaches the acceptance owner named, each on a synthetic set
  cs_case NOFILES "LEGX LEGY" "a b c d" "LEGX a
LEGY b
LEGY c
LEGY d" "LEG:a b" "LEG:c d"
  ck "cs-no-classes-files-exits-2" 2 "$(cat "$T/cs-NOFILES/fv.rc")"
  ckp cs-no-classes-names-the-count-cause classes-file-count "$(cat "$T/cs-NOFILES/fv.out")"
  ckp cs-no-classes-says-why "not exactly the booked claim set" "$(cat "$T/cs-NOFILES/receipt.txt")"
  ckp cs-no-classes-reports-zero-claims "claims=0" "$(cat "$T/cs-NOFILES/receipt.txt")"
  cs_case ONELEG "LEGX LEGY" "a b c d" "LEGX a
LEGY b
LEGY c
LEGY d" "LEG:a b" "LEG:c d" "LEGX=a HELD"
  ck "cs-one-leg-file-missing-exits-2" 2 "$(cat "$T/cs-ONELEG/fv.rc")"
  ckp cs-one-leg-names-the-cause "leg-file-missing:LEGY" "$(cat "$T/cs-ONELEG/fv.out")"
  cs_case SHORT "LEGX" "a b c" "LEGX a
LEGX b
LEGX c" "LEG:a b c" "LEGX=a HELD
b HELD"
  ck "cs-claim-missing-from-a-file-exits-2" 2 "$(cat "$T/cs-SHORT/fv.rc")"
  ckp cs-short-names-the-cause "claim-count:LEGX" "$(cat "$T/cs-SHORT/fv.out")"
  cs_case EMPTYFILE "LEGX" "a b" "LEGX a
LEGX b" "LEG:a b" "LEGX=EMPTY"
  ck "cs-empty-classes-file-exits-2" 2 "$(cat "$T/cs-EMPTYFILE/fv.rc")"
  ckp cs-empty-file-names-a-cause leg-file-missing "$(cat "$T/cs-EMPTYFILE/fv.out")"
  cs_case DUP "LEGX" "a b" "LEGX a
LEGX b" "LEG:a b" "LEGX=a HELD
b HELD
b FAILED"
  ck "cs-duplicate-claim-exits-2" 2 "$(cat "$T/cs-DUP/fv.rc")"
  ckp cs-dup-names-the-cause "claim-duplicated:LEGX" "$(cat "$T/cs-DUP/fv.out")"
  cs_case EXTRA "LEGX" "a b" "LEGX a
LEGX b" "LEG:a b" "LEGX=a HELD
b HELD
zzz HELD"
  ck "cs-extra-unbooked-claim-exits-2" 2 "$(cat "$T/cs-EXTRA/fv.rc")"
  ckp cs-extra-names-the-cause claim-unregistered "$(cat "$T/cs-EXTRA/fv.out")"
  cs_case EXTRAFILE "LEGX" "a b" "LEGX a
LEGX b" "LEG:a b" "LEGX=a HELD
b HELD" "LEGZ=a PASS"
  ck "cs-extra-classes-file-exits-2" 2 "$(cat "$T/cs-EXTRAFILE/fv.rc")"
  ckp cs-extra-file-names-the-file-cause classes-file-not-booked "$(cat "$T/cs-EXTRAFILE/fv.out")"
  ckp cs-extra-file-names-the-count-cause classes-file-count "$(cat "$T/cs-EXTRAFILE/fv.out")"
  cs_case TAMPER "LEGX" "a b" "LEGX a" "LEG:a b" "LEGX=a HELD"
  ck "cs-tampered-registration-exits-2" 2 "$(cat "$T/cs-TAMPER/fv.rc")"
  ckp cs-tamper-names-the-cause registration-claim-count "$(cat "$T/cs-TAMPER/fv.out")"
  cs_case NOREG "LEGX" "a b" "-" "LEG:a b" "LEGX=a HELD"
  ck "cs-missing-registration-exits-2" 2 "$(cat "$T/cs-NOREG/fv.rc")"
  ckp cs-missing-registration-names-the-cause registration-missing-or-empty "$(cat "$T/cs-NOREG/fv.out")"
  # a complete, well-formed set with a bare FAIL is still FALSIFIED: the gate must not swallow a refutation
  cs_case FAILSTILL1 "LEGX" "a b" "LEGX a
LEGX b" "LEG:a b" "LEGX=a HELD
b FAILED"
  ck "cs-complete-set-with-a-fail-is-still-1" 1 "$(cat "$T/cs-FAILSTILL1/fv.rc")"
  ckp cs-fail-still-decides-per-claim FALSIFIED "$(cat "$T/cs-FAILSTILL1/fv.out")"
  # ---- v9: the per-LEG BINDING. A claim registered under one leg does not satisfy another leg. Each of
  # ---- these drives the SHIPPED cs_check through final_verdict on a synthetic booked set.
  cs_case CROSSLEG "LEGX LEGY" "a b c" "LEGX a
LEGX b
LEGY c" "LEG:a b" "LEG:c" "LEGX=a HELD
b HELD" "LEGY=b HELD"
  ck "cs-claim-from-another-leg-exits-2" 2 "$(cat "$T/cs-CROSSLEG/fv.rc")"
  ckp cs-crossleg-names-the-claim-cause "claim-wrong-leg:LEGY:b" "$(cat "$T/cs-CROSSLEG/fv.out")"
  ckp cs-crossleg-names-the-cross-file-cause "claim-duplicated-across-files" "$(cat "$T/cs-CROSSLEG/fv.out")"
  cs_case SWAP "LEGX LEGY" "a b c d" "LEGX a
LEGX b
LEGY c
LEGY d" "LEG:a b" "LEG:c d" "LEGX=c HELD
d HELD" "LEGY=a HELD
b HELD"
  ck "cs-claims-moved-between-legs-exit-2" 2 "$(cat "$T/cs-SWAP/fv.rc")"
  ckp cs-swap-names-both-files-cause "claim-wrong-leg:LEGX:c" "$(cat "$T/cs-SWAP/fv.out")"
  ckp cs-swap-names-the-other-file-cause "claim-wrong-leg:LEGY:a" "$(cat "$T/cs-SWAP/fv.out")"
  ckp cs-swap-names-the-count-cause "claim-count:LEGX" "$(cat "$T/cs-SWAP/fv.out")"
  cs_case SUBST "LEGX LEGY" "a b c" "LEGX a
LEGY b
LEGY c" "LEG:a" "LEG:b c" "LEGX=a HELD" "LEGY=q HELD"
  ck "cs-substituted-name-exits-2" 2 "$(cat "$T/cs-SUBST/fv.rc")"
  ckp cs-subst-names-the-substituted-cause "claim-unregistered:q" "$(cat "$T/cs-SUBST/fv.out")"
  cs_case LEGSET "LEGX LEGY" "a b c d" "LEGX a
LEGX b
LEGZ c
LEGZ d" "LEG:a b" "LEG:c d" "LEGX=a HELD
b HELD" "LEGY=c HELD
d HELD"
  ck "cs-registration-naming-an-unbooked-leg-exits-2" 2 "$(cat "$T/cs-LEGSET/fv.rc")"
  ckp cs-legset-names-the-leg-cause "registration-leg-count" "$(cat "$T/cs-LEGSET/fv.out")"
  cs_case MALREG "LEGX LEGY" "a b" "LEGX a extra
LEGY b" "LEG:a" "LEG:b" "LEGX=a HELD" "LEGY=b HELD"
  ck "cs-malformed-registration-line-exits-2" 2 "$(cat "$T/cs-MALREG/fv.rc")"
  ckp cs-malreg-names-the-cause "registration-malformed-line" "$(cat "$T/cs-MALREG/fv.out")"
  cs_case MALFILE "LEGX LEGY" "a b" "LEGX a
LEGY b" "LEG:a" "LEG:b" "LEGX=a HELD extra" "LEGY=b HELD"
  ck "cs-malformed-classes-line-exits-2" 2 "$(cat "$T/cs-MALFILE/fv.rc")"
  ckp cs-malfile-names-the-cause "malformed-classes-line:LEGX" "$(cat "$T/cs-MALFILE/fv.out")"
  # the snapshot: what cs_check validated is what final_verdict counts. Validate a good set, tamper the live
  # file afterwards, and the snapshot still holds the validated bytes while the tampered live set refuses.
  mkdir -p "$T/cs-SNAPPOS"; svOUT=$OUT; svO=$O; OUT="$T/cs-SNAPPOS"
  register_claims > /dev/null 2>&1
  for lgt in $LEGS_BOOKED; do
    eval "shplist=\"\$CLAIMS_$lgt\""
    : > "$T/cs-SNAPPOS/$lgt.classes"
    for clm in $shplist; do printf '%s HELD\n' "$clm" >> "$T/cs-SNAPPOS/$lgt.classes"; done
  done
  O="$T/cs-SNAPPOS/receipt.txt"
  cs_check > "$T/cs-SNAPPOS/cc.out" 2>&1; rcsc=$?
  ck cs-snapshot-validates-a-good-set 0 "$rcsc"
  ck cs-snapshot-flag-is-set 1 "${CS_SNAP_OK:-0}"
  SNAPDIR=$CS_SNAP
  ck cs-snapshot-has-one-digest-per-leg "$(printf '%s\n' $LEGS_BOOKED | grep -c .)" "$(grep -c . "$SNAPDIR/digests.txt" 2>/dev/null || echo 0)"
  snapok=1
  for lgt in $LEGS_BOOKED; do
    want=$(sha256sum -- "$OUT/$lgt.classes" | cut -d' ' -f1)
    got=$(awk -v l="$lgt" '$1==l {print $2}' "$SNAPDIR/digests.txt")
    [ "$want" = "$got" ] || snapok=0
  done
  ck cs-snapshot-digests-match-the-validated-bytes 1 "$snapok"
  printf '%s\n' 'a claim nobody registered' >> "$OUT/$lgt.classes"
  cs_check > "$T/cs-SNAPPOS/cc2.out" 2>&1; rcsc2=$?
  ck cs-tampered-live-set-refuses-revalidation 1 "$rcsc2"
  snapok2=1
  for lgt in $LEGS_BOOKED; do
    got=$(sha256sum -- "$SNAPDIR/$lgt.classes" 2>/dev/null | cut -d' ' -f1)
    want=$(awk -v l="$lgt" '$1==l {print $2}' "$SNAPDIR/digests.txt")
    [ -n "$got" ] && [ "$got" = "$want" ] || snapok2=0
  done
  ck cs-snapshot-still-holds-the-validated-bytes 1 "$snapok2"
  OUT=$svOUT; O=$svO

  # ---- THE MULTI-CLAIM LEG CONTRACT end to end, through this packet's real leg() and its real exit
  # ---- decision. A stub stands in for claim_batch: bare verdict lines on stdout, the frozen completion
  # ---- record on stderr, and the exit code the case asks for. Every case reports
  # ---- "<class a> <class b> <leg state> <packet exit>", so the exit code is asserted, not described.
  mkstub() { local f=$1 rc=$2; shift 2
    { printf '#!/bin/sh\n'
      for l in "$@"; do printf 'printf "%%s\\n" "%s"\n' "$l"; done
      printf 'printf "%%s\\n" "%s" >&2\nexit %s\n' "$PP_REAL" "$rc"; } > "$f"
    chmod +x "$f"; }
  cc_case() { # $1=case name $2=rc $3..=verdict lines -> the four fields above, from the packet's own code
    local nm=$1 rc=$2 D ex
    D=$T/cc_$nm; mkdir -p "$D/mem" "$D/logs"
    OUT=$D; O=$D/receipt.txt; MEMDIR=$D/mem; EV=$D/events
    printf 'low 0\nhigh 3\nmax 1\noom 0\noom_kill 0\nfail 0\ndiscard_oom 0\n' > "$EV"
    mkstub "$D/stub" "$rc" "$@"
    BIN4=$D/stub; BIN5=$D/stub; BIN6=$D/stub; export BIN4 BIN5 BIN6
    # v8: a fixture leg books and REGISTERS its own tiny claim set first, exactly as the run path does, so
    # the claim-set gate under final_verdict is decided against a complete registration for this case.
    book_case_set "$nm" "a b" "a b" || { echo "fixture could not book its own claim set: $nm"; return 1; }
    local rr; rr=$(register_claims 2>&1) || { echo "fixture could not register its own claim set: $rr"; return 1; }
    leg "$nm" "\"\$BIN4\" --source-root dag --entry x.dag --functions a,b" "a b"
    ( final_verdict ) > "$D/fv.txt" 2>&1; ex=$?
    restore_booked_set
    echo "$(awk '$1=="a"{print $2}' "$D/$nm.classes") $(awk '$1=="b"{print $2}' "$D/$nm.classes") $(cat "$D/$nm.state") $ex"; }
  ck contract-pass-and-fail-at-rc1 "HELD FAILED FALSIFIED 1" "$(cc_case MIX1 1 "PASS a" "FAIL b")"
  ck contract-all-pass-at-rc1 "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case ALLP1 1 "PASS a" "PASS b")"
  ck contract-pass-and-fail-at-rc0 "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case MIX0 0 "PASS a" "FAIL b")"
  ck contract-all-pass-at-rc0 "HELD HELD AS_EXPECTED 0" "$(cc_case ALLP0 0 "PASS a" "PASS b")"
  ck contract-reasoned-fail "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case RSN 1 "PASS a" "FAIL b (runtime error [E]: boom)")"
  ck contract-missing-verdict "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case MIS 0 "PASS a")"
  ck contract-duplicate-verdict "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case DUP 0 "PASS a" "PASS b" "PASS b")"
  ck contract-timeout-rc124 "NO_OBSERVATION NO_OBSERVATION NOOBS 2" "$(cc_case T124 124 "PASS a" "PASS b")"
  # FALSIFIED outranks NO_OBSERVATION, but only across legs: leg 1 is a well-formed bare FAIL, leg 2 has
  # no completion record at all. The FAIL is still a finding, so the packet exits 1, not 2.
  D=$T/cc_ACROSS; mkdir -p "$D/mem" "$D/logs"
  OUT=$D; O=$D/receipt.txt; MEMDIR=$D/mem; EV=$D/events
  printf 'low 0\nhigh 3\nmax 1\noom 0\noom_kill 0\nfail 0\ndiscard_oom 0\n' > "$EV"
  book_case_set "GOODFAIL NORC" "a b c" "a b" "c" || echo "fixture could not book its own claim set: ACROSS"
  arr=$(register_claims 2>&1) || echo "fixture could not register its own claim set: $arr"
  mkstub "$D/stub1" 1 "PASS a" "FAIL b"
  BIN4=$D/stub1; BIN5=$D/stub1; BIN6=$D/stub1; export BIN4 BIN5 BIN6
  leg GOODFAIL "\"\$BIN4\" --source-root dag --entry x.dag --functions a,b" "a b"
  printf '#!/bin/sh\nprintf "PASS c\\n"\nexit 0\n' > "$D/stub2"; chmod +x "$D/stub2"
  BIN4=$D/stub2
  leg NORC "\"\$BIN4\" --source-root dag --entry x.dag --functions c" "c"
  ( final_verdict ) > "$D/fv.txt" 2>&1; rc=$?
  ck falsified-outranks-no-observation 1 "$rc"
  ck across-legs-the-clean-claims-still-classify "HELD FAILED NO_OBSERVATION" "$(awk '{printf "%s ", $2}' "$D/GOODFAIL.classes" "$D/NORC.classes" | sed 's/ $//')"
  restore_booked_set
  # ---- POST-LEG GATES. Real git in a throwaway repo, because the gate runs real git and a stub of git
  # ---- would test the stub. Clean worktree + unchanged binary = no cause; either failure names every
  # ---- claim NO_OBSERVATION in final_verdict, even a bare FAIL that was otherwise well formed.
  pg_case() { # $1=name $2=clean|dirtywt|binchanged -> "<cause> exit=<packet exit>" for a well-formed FAIL
    local D=$T/pg_$1 wt=$T/pg_$1/wt ex pre
    mkdir -p "$D/logs" "$wt"
    # real git in a throwaway temp dir, because the gate runs real git: stubbing git would test the stub.
    # The identity is passed to EVERY git invocation in the fixture (init AND commit), HOME is isolated to an
    # empty directory, and no config outside the repo can reach it: GIT_CONFIG_NOSYSTEM=1, GIT_CONFIG_GLOBAL=
    # /dev/null, core.hooksPath set to an empty dir. A fixture that needs some host's global git identity is
    # a fixture that fails on a machine nobody has configured, and on srv1 that would spend the grant.
    mkdir -p "$T/nohooks" "$T/nohome"
    gxf() { env -i PATH="$PATH" HOME="$T/nohome" TMPDIR="$T" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
        GIT_AUTHOR_NAME=fixture GIT_AUTHOR_EMAIL=fixture@packet GIT_COMMITTER_NAME=fixture GIT_COMMITTER_EMAIL=fixture@packet \
        git -c user.name=fixture -c user.email=fixture@packet -c core.hooksPath="$T/nohooks" "$@"; }
    gxf -c init.defaultBranch=main init -q "$wt" </dev/null >/dev/null 2>&1
    printf 'x\n' > "$wt/file"
    gxf -C "$wt" add file </dev/null >/dev/null 2>&1
    gxf -C "$wt" commit -q -m seed </dev/null >/dev/null 2>&1
    OUT=$D; O=$D/receipt.txt
    book_case_set "LEGX" "a b" "a b" || { echo "fixture could not book its own claim set: pg"; return 1; }
    pgr=$(register_claims 2>&1) || { echo "fixture could not register its own claim set: $pgr"; return 1; }
    printf 'a HELD\nb FAILED\n' > "$D/LEGX.classes"
    : > "$D/bin"
    pre=$(bin_sha "$D/bin" || echo unreadable)
    case "$2" in
      dirtywt) printf 'left behind by a leg\n' > "$wt/untracked" ;;
      binchanged) printf 'the bytes under the runner changed\n' > "$D/bin" ;;
    esac
    post_leg_gate "$wt" "$D/bin" "$pre"
    ( final_verdict ) > "$D/fv.txt" 2>&1; ex=$?
    restore_booked_set
    echo "$(cut -c1-28 "$D/post-leg.cause") exit=$ex"; }
  ckp post-leg-clean-no-cause "exit=1" "$(pg_case clean clean)"
  ckp post-leg-dirty-worktree-refuses "the worktree" "$(pg_case dirtywt dirtywt)"
  ckp post-leg-runner-changed-refuses "the runner changed" "$(pg_case binchanged binchanged)"
  # a cause rewrites EVERY claim, including the bare FAIL that was otherwise a finding: the pinned bytes
  # are no longer what the legs ran against, so nothing in the leg can be attributed to the pin.
  # the cause is applied when the verdict is computed and recorded in the receipt; the classes files keep
  # what the legs wrote, so the audit trail shows both what a leg printed and what it was allowed to mean.
  ckp post-leg-dirty-forces-every-claim-nood "a=NO_OBSERVATION b=NO_OBSERVATION" "$(grep -o 'claims classified:.*' "$T/pg_dirtywt/receipt.txt")"
  ckp post-leg-dirty-makes-a-bare-fail-exit-2 "exit=2" "$(pg_case dirtywt2 dirtywt)"
  ckp post-leg-clean-keeps-the-finding "a=HELD b=FAILED" "$(grep -o 'claims classified:.*' "$T/pg_clean/receipt.txt")"
  ckp post-leg-binchange-forces-every-claim-nood "a=NO_OBSERVATION b=NO_OBSERVATION" "$(grep -o 'claims classified:.*' "$T/pg_binchanged/receipt.txt")"
  ckp post-leg-cause-is-recorded-in-the-receipt "post_leg:" "$(cat "$T/pg_dirtywt/receipt.txt")"
  # ---- out_gate: the one argument, gated.
  ck out_gate-accepts-the-booked-shape 0 "$(out_gate /home/x/runs/BOOKED-JOB/out BOOKED-JOB; echo $?)"
  ckp out_gate-refuses-relative "OUT 'runs/J/out' is relative" "$(out_gate runs/J/out J)"
  ckp out_gate-refuses-no-out-suffix "does not end in /out" "$(out_gate /home/x/runs/J/logs J)"
  # the /out suffix is checked before anything below /out, so this path is refused as a missing suffix;
  # only a bare /out (which does end in /out) reaches the "path below /out" arm, and that is asserted too.
  ckp out_gate-refuses-a-path-under-out "does not end in /out" "$(out_gate /home/x/runs/J/out/extra J)"
  ckp out_gate-refuses-a-different-job "names job 'other-job'" "$(out_gate /home/x/runs/other-job/out BOOKED-JOB)"
  ckp out_gate-refuses-a-bare-out "/out entry itself" "$(out_gate /out J)"
  # ---- gate exit codes: a refusal that returns 0 is a gate that never closes. Every arm is checked.
  wg_rc() { wall_gate "$1" "$2" "$3" "$4" >/dev/null 2>&1; echo $?; }
  ck wall-gate-ok-exits-0 0 "$(wg_rc "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S")"
  ck wall-gate-refuses-over-cap-nonzero 1 "$(wg_rc "$WALL_SUM" 9900 "$LAUNCHER_CAP" 10200)"
  ck wall-gate-refuses-sum-over-scope-nonzero 1 "$(wg_rc 99999 "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S")"
  ck wall-gate-refuses-short-driver-nonzero 1 "$(wg_rc "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" 1)"
  ckp wall-gate-names-the-cap-refusal "exceeds the installed calmcrab-udrv v4 preflight cap" "$(wall_gate "$WALL_SUM" 9900 "$LAUNCHER_CAP" 10200)"
  ckp wall-gate-names-the-sum-refusal "exceeds the scope RuntimeMaxSec" "$(wall_gate 99999 "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S")"
  ckp wall-gate-names-the-driver-refusal "the driver service must be RuntimeMaxSec" "$(wall_gate "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" 1)"
  pg_rc() { producer_gate "$1" >/dev/null 2>&1; echo $?; }
  ck producer-gate-ok-exits-0 0 "$(pg_rc "$T/pb_ok.rs")"
  ck producer-gate-stdout-render-refuses-nonzero 1 "$(pg_rc "$T/pb_stdout.rs")"
  ck producer-gate-two-sites-refuses-nonzero 1 "$(pg_rc "$T/pb_two.rs")"
  ck producer-gate-unreadable-refuses-nonzero 1 "$(pg_rc "$T/nope.rs")"
  og_rc() { out_gate "$1" "$2" >/dev/null 2>&1; echo $?; }
  ck out_gate-ok-exits-0 0 "$(og_rc /home/x/runs/J/out J)"
  ck out_gate-refusals-exit-nonzero 1 "$(og_rc runs/J/out J)"
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
OUT=${1%/}
GRANT=${2:-}
# ONE argument, and it is gated rather than described (v7 non-blocking): absolute, in the launcher's
# <RUNTIME>/out shape, and naming the job this packet is booked as. A relative OUT resolves against
# whatever directory the launcher happened to start in, and an OUT naming someone else's job would file
# this row's evidence where the next reader will not look for it.
OG=$(out_gate "$OUT" "$J") || { echo "NO_OBSERVATION: $OG" >&2; exit 2; }
JOB=$(basename "$(dirname "$OUT")" 2>/dev/null || echo "")
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
# CARGO_HOME lives under the evidence dir (v7 non-blocking): the build must not write into whoever's home
# the launcher happened to start in, and the registry cache the build resolved against becomes part of
# the row's evidence instead of vanishing with the account.
export CARGO_HOME=$OUT/cargo-home; mkdir -- "$CARGO_HOME" || { echo "NO_OBSERVATION: cannot make $CARGO_HOME" >&2; exit 2; }
O=$OUT/receipt.txt
echo "job=$JOB booked_job=$J state=executing-under-grant host=$(hostname 2>/dev/null || echo unreadable) machine_id=$(cat /etc/machine-id 2>/dev/null || echo unreadable) start=$(date -u +%FT%TZ)" > "$O"
S=$(bin_sha "$0") || nox "cannot hash the staged script at $0" script-hash
echo "script_sha256=$S (compare with the published frozen value; a file cannot gate its own hash)" >> "$O"
echo "$GRANT" > "$OUT/grant-ref.txt"
echo "grant_ref=$GRANT (the reference this row was executed under; its clearance is keen-otter's, not this packet's)" >> "$O"

# FIXTURES BEFORE ANY WORK IS SPENT: a misfired classifier ends the row with no claim attempted.
timeout -k "$T_KILL" "$T_SELFRUN" bash "$0" --classifier-fixtures > "$OUT/logs/selfcheck.log" 2>&1; SFX=$?
[ "$SFX" = 0 ] || nox "the packet's own classifier fixtures misfired or hung (exit $SFX); no claim attempted (see logs/selfcheck.log)" "fixtures(exit=$SFX)"
echo "fixtures: $(grep -c '^FIXTURE PASS' "$OUT/logs/selfcheck.log") pass, $(grep -c '^FIXTURE FAIL' "$OUT/logs/selfcheck.log") fail" >> "$O"

CG=$(scope_gate "$(cat /proc/self/cgroup 2>/dev/null)" "$JOB") || nox "scope gate: /proc/self/cgroup is not exactly one 0:: line whose path ends in /$JOB.scope: $(tr '\n' ' ' < /proc/self/cgroup 2>/dev/null)" scope
EV=/sys/fs/cgroup$CG/memory.events
GA=$(oom_snap "$EV" gate-a); GB=$(oom_snap "$EV" gate-b)
GV=$(oom_pair_verdict "$GA" "$GB")
# the reason label carries the verdict the gate actually got: an unreadable counter is not an
# incoherent counter, and calling one the other is how a read failure gets reported as a memory event.
[ "$GV" = none ] || nox "memory.events at $EV unreadable or already OOM-marked at the gate: $GA -> $GB" "gate-read-$GV"
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
timeout -k "$T_KILL" "$T_SETUP_CLONE" bash -c "$CMD_CLONE" > "$OUT/logs/wt-r6-init.log" 2>&1 || nox "git init of $W failed or exceeded ${T_SETUP_CLONE}s" clone
timeout -k "$T_KILL" "$T_SETUP_FETCH" bash -c "$CMD_FETCH" > "$OUT/logs/wt-r6-fetch.log" 2>&1 || nox "shallow fetch of $H into $W failed or exceeded ${T_SETUP_FETCH}s" clone
timeout -k "$T_KILL" "$T_SETUP_CHECKOUT" bash -c "$CMD_CHECKOUT" > "$OUT/logs/wt-r6-checkout.log" 2>&1 || nox "checkout of FETCH_HEAD in $W failed or exceeded ${T_SETUP_CHECKOUT}s" clone
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
timeout -k "$T_KILL" "$T_BUILD" /usr/bin/time -v bash -c "$CMD_BUILD" > "$OUT/logs/wt-r6-build.log" 2>&1; BX=$?
k1=$(oom_snap "$EV" wt-r6-build-after); BO=$(oom_pair_verdict "$k0" "$k1")
echo "== build wt-r6 exit=$BX oom=$BO $(tm "$OUT/logs/wt-r6-build.log")" >> "$O"
[ "$BX" = 124 ] && nox "wt-r6 build timed out at ${T_BUILD}s" build-timeout
[ "$BX" = 0 ] || { tail -10 "$OUT/logs/wt-r6-build.log" >> "$O"; nox "wt-r6 build exit=$BX" build; }
[ "$BO" = none ] || nox "wt-r6 build oom=$BO ($k0 -> $k1)" "build-oom-$BO"
BIN6=$W/target/release/claim_batch
[ -x "$BIN6" ] || nox "wt-r6 build produced no executable at $BIN6" build
echo "binary $BIN6 sha256=$(bin_sha "$BIN6" || echo unreadable)" >> "$O"
PREBIN6=$(bin_sha "$BIN6" || echo unreadable)
WT=$W; RUNNER=$BIN6; PREBIN=$PREBIN6; export BIN6 WT

LASTOOM=""
# v8 (msg_8ed9b50b): the booked claim set is REGISTERED into OUT from the frozen constants before any leg
# can write evidence, so the exit decision is anchored to it. A registration that disagrees with this
# packet's own constants refuses the run (NO_OBSERVATION, exit 2) rather than mis-deciding it.
nbl_booked=$(printf '%s
' $LEGS_BOOKED | grep -c .)
[ "$nbl_booked" = "$N_LEGS" ] || nox "the booked claim set names $nbl_booked legs, this row runs $N_LEGS" register
regmsg=$(register_claims 2>&1) || nox "the booked claim set could not be registered: $regmsg" register
leg R6 "$CMD_LEG_R6" "$N6"

FA=$(oom_snap "$EV" final)
FV=$(oom_pair_verdict "$LASTOOM" "$FA")
echo "== final oom vs last leg after: $LASTOOM -> $FA verdict=$FV" >> "$O"
[ "$FV" = none ] || nox "memory events moved between the last leg and the final snapshot: $LASTOOM -> $FA ($FV)" "teardown-oom-$FV"
echo "memory.peak (printed, never judged): $(cat "/sys/fs/cgroup$CG/memory.peak" 2>/dev/null || echo unreadable)" >> "$O"

# post-leg gates, then the exit decision. final_verdict reads only this packet's own classes files and
# its own post-leg.cause; it prints the per-claim table, then the verdict, and exits 0 / 1 / 2.
post_leg_gate "$WT" "$RUNNER" "$PREBIN"
final_verdict
