# script.sh (frozen copy for fierce-wren-19-r4r5-db56530f, v3): acceptance grant set rows R4 and R5,
# gunbc. PREPARE ONLY: this file has not been RUN anywhere. It is the staged script a booked calmcrab-udrv
# v4 row would execute on srv1, re-framed on keen-otter's hardened srv1 frame (template:
# wise-moth-707-rdpo's rdpo.sh, sha256 aaf32964184a30d66f6320de707bfb03975f0df683cccd1133dcf2d024ed9366,
# gunbc#15938 comment 6672252712). No claim execution, no ssh, no partition write happened here.
#
# WHAT v4 ADDS (sleek-owl's three blockers on v3 176234a92458c5a6, verified by sunny-bee-153):
#   (1) the launcher argv is calmcrab-udrv.sh <JOB> <RUNTIME_S> <MEMMAX_G> <MEMHIGH_G> <TASKS> <CPU%>
#       <STAGED_SCRIPT> <SCRIPT_SHA> OUT — seconds, GiB, %, the PACKET's own sha, literal OUT — and the R
#       ledger row is booked in GiB (keen-otter msg_c9d1df56); the bounds gate DERIVES its byte values from
#       those same GiB constants so the argv, the manifest and the gate cannot disagree.
#   (2) a completion-record gate per leg (see COMPLETION RECORD above), keyed on ProcessPartition::render.
#   (3) verdict exactness per name (see VERDICT RULES above).
#
# THIS ROW DISCRIMINATES NOTHING (v3 requirement). R4 and R5 are PRESENCE observations: each claim is a
# witness that must hold on the pinned tree, and NO mutation leg is run or proposed — there is no edit,
# no rebuild-and-recompare, and no `mutation` boundary anywhere in this file. So a full-green row proves
# only that the pinned tree still answers "yes" to those twelve witnesses; it cannot localize a cause,
# and it must not be recorded as discriminating anything. A bare FAIL is a falsified expectation (exit 1),
# which is a FINDING for the grant set, not a diagnosis.
#
# ROWS (eleven claims, three claim_batch invocations; every claim expected to hold):
#   R4/a  5 claims  entry src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag
#   R4/b  1 claim   entry src/v2/test/claim/rust_crate_partition_witness.dag          (separate --entry)
#   R5    5 claims  entry dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag
# Row R6 (one live-corpus claim) is NOT here: it reads the live fabric store and is held behind
# keen-otter's msg_3407a188; it has its own file, acceptance_r6_live_read_claim_run_v3.sh.
#
# PINS (fetched from the hard-coded origin by sha; HEAD and the tree must both match, and the entry
# files must be the frozen blobs, before anything is built):
#   R4  3ddaefb5-style pair, here: db56530fcb186370e3dbf1aa3c69ded324daa25d  tree 48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc
#   R5  dff57af790f923066d61721e89ff751a91021278  tree 356f2894a60edda51cdbe79879b2fb3b7aea75c4   (the #15936 squash)
#   entry blobs (identical at both pins and at current origin/main):
#     332cc981fd194bb9f02f16ff6e3c14d81b874e2b  src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag
#     abc6313eb8c4d49ad74a7bcafad36b895ea32f76  src/v2/test/claim/rust_crate_partition_witness.dag
#     adfeb29426eac9d4da67630a12d89f8eabe8754b  dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag
#
# STEPS (every one is wrapped in timeout; the CAPPED SUM is the row's worst case):
#   setup   per pin: clone 300 s, shallow fetch of the pin 300 s, checkout 120 s          = 2 x 720  = 1440 s
#   build   cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch, 2400 s each pin. A cold
#           srv1 release build of this crate took ~2100 s (N7, rdpo); the binary is claim_batch, built from
#           the same crate, so the same budget. A nonzero exit, a timeout or a missing binary is 2.  2 x 2400 = 4800 s
#   legs    900 s each: R4/a (5 claims), R4/b (1), R5 (5). One claim_batch invocation per leg; claims in a
#           leg are evaluated in one process, so a leg that times out loses all its claims at once.  3 x 900  = 2700 s
#   CAPPED SUM = 1440 + 4800 + 2700 = 8940 s. That capped sum is NOT a prediction of the wall: it is the
#   ceiling the timeouts admit.
# WALL ACCOUNTING (v3): the scope's RuntimeMaxSec is 9900 s — the capped sum (8940 s) plus a 960 s margin
#   for process startup, the two builds' cargo-network stall allowance and evidence writing; a row that
#   RuntimeMaxSec cuts off is incomplete, which is NO_OBSERVATION (2). The driver row
#   calmcrab-drv-<JOB>.service is sized FROM the scope: RuntimeMaxSec 10500 s = the scope's 9900 plus 600 s
#   for staging, the ledger write and teardown. If the launcher books different numbers, the bounds gate
#   below refuses the run rather than adapting to them.
#
# BOOKED BOUNDS (keen-otter (1)); the bounds gate (v3 item 2) reads its OWN cgroup's memory.max,
#   memory.high, memory.swap.max, pids.max and cpu.max AFTER the scope gate and BEFORE any fetch or build,
#   and refuses with 2 on any mismatch or unreadable file. Dumping these is not a gate:
#   memory.max=34359738368 (32G), memory.high=30064771072 (28G, a CEILING and UNMEASURED: no -j 32 build
#   peak exists for this binary, and an OOM wastes the attempt), memory.swap.max=0, pids.max=4096
#   (pids.max counts THREADS: 32 rustc jobs, their linkers and cargo, then one claim_batch at a time),
#   cpu.max="3200000 100000" (CPUQuota=3200% at the default 100 ms period, PAIRED with -j 32: srv1 has
#   nproc=128 and the ~2100 s cold build was uncapped, so 32 jobs on 32 CPUs stays inside the 2400 s cap).
#   These are separate reads, NOT an atomic snapshot; no atomicity is claimed.
# Bounds are booked in the LAUNCHER'S units (GiB, seconds, CPU %, tasks) and the byte values the cgroup
# files carry are DERIVED from them here, so the launcher argv, the manifest and the gate cannot drift.
# cpu.max's period is the default 100000 us, so the quota is CPU_PCT/100 * 100000 = CPU_PCT*1000 us.
RUNTIME_S=9900; MEMMAX_G=32; MEMHIGH_G=28; TASKS_MAX=4096; CPU_PCT=3200
BOOKED="memory.max=$((MEMMAX_G*1073741824)) memory.high=$((MEMHIGH_G*1073741824)) memory.swap.max=0 pids.max=$TASKS_MAX cpu.max=$((CPU_PCT*1000))_100000"
#
# LAUNCHER (calmcrab-udrv.sh, v4 form) — no bare systemd-run. Full argv, in the launcher's order:
#   calmcrab-udrv.sh <JOB> <RUNTIME_S> <MEMMAX_G> <MEMHIGH_G> <TASKS> <CPU%> <STAGED_SCRIPT> <SCRIPT_SHA> OUT
# For THIS row the call is literally:
#   calmcrab-udrv.sh fierce-wren-19-r4r5-db56530f 9900 32 28 4096 3200% <STAGED_SCRIPT> <v4 packet sha256> OUT
# Units are the launcher's: RUNTIME in SECONDS (the scope's wall, sized from the capped per-step timeouts
# below), memory in GiB, CPU with the %% sign, SCRIPT_SHA is the PACKET'S OWN sha256 (NOT the source commit),
# and OUT is passed LITERALLY — the script derives JOB as basename(dirname(OUT)), so OUT must be
# $HOME/runs/fierce-wren-19-r4r5-db56530f/out. MEMMAX_G/MEMHIGH_G/TASKS/CPU%% are the same numbers the bounds gate below re-derives.
# LEDGER ROWS (R/S/D), three of them, named by the launcher's convention:
#   fierce-wren-19-r4r5-db56530f-res   cap 32 (GiB — the R row is booked in GiB, not bytes, per keen-otter msg_c9d1df56)
#   fierce-wren-19-r4r5-db56530f-drv   cap 1
#   fierce-wren-19-r4r5-db56530f       cap 0
# UNITS: fierce-wren-19-r4r5-db56530f.scope (the row's own scope; the scope gate requires this process's cgroup path to END in
#   "/fierce-wren-19-r4r5-db56530f.scope") and calmcrab-drv-fierce-wren-19-r4r5-db56530f.service (the driver service). R is written before the scope
#   starts; S starts the scope and the driver service under those bookings; D writes the row's exit code
#   back to the ledger. This script books nothing and writes no ledger: it REFUSES (2) when the environment
#   it was started in does not match the booking.
#
# EXIT CONTRACT (v3 item 4). THREE codes, no others:
#   0  every claim in every leg printed a bare PASS and the runner's own exit agreed (expectations held)
#   1  a claim printed a bare FAIL (expectation FALSIFIED — a finding, and the reason for the row)
#   2  NO_OBSERVATION: any setup, usage, scope, bounds, clone, checkout, blob, name, build, timeout,
#      OOM, counter-incoherence, classifier or return-code failure — every gate refusal is 2, including
#      a run dir that already exists and a leg whose output is not exactly bare verdict lines.
# There is deliberately NO "couldn't run" code and no gate-only code: a leg that cannot run, a build that
# fails and a refusal at any gate are all 2, because none of them produced an observation.
#
# OOM (v3 item 3, keen-otter (1)/(4) as rdpo hardened it): ONE snapshot per boundary through oom_snap,
#   which reads memory.events ONCE and publishes those exact raw bytes, sequence-numbered and timestamped,
#   under OUT/memevents/ by EXCLUSIVE CREATE (set -C) BEFORE any number is returned; a failed read or a
#   failed write returns UNKNOWN/UNKNOWN, and every caller turns UNKNOWN into exit 2. Each key must occur
#   EXACTLY ONCE as a whole line `^key [0-9]+$`; an extra field, a duplicate key, a tab-separated record or
#   a non-digit value is UNKNOWN. oom AND oom_kill are both compared, boundary by boundary: a rise is
#   OOM_KILLED (2), a FALL cannot happen inside one cgroup's life so it is COUNTER_INCOHERENT (2), an
#   unreadable counter is UNKNOWN (2). Boundaries: gate-a/gate-b before any work, build-before/build-after
#   for each pin, leg-before/leg-after for each leg, and a FINAL snapshot compared against the last leg's
#   after-snapshot (a kill in evidence writing or teardown is caught there). No `grep '^oom_kill [1-9]'`,
#   no silent `cat`: memory.peak and the time/RSS lines are PRINTED for the reader and never judged.
#
# CLASSIFIER (what counts as an observation). claim_batch's own contract at this pin
#   (src/v1/stage0/src/bin/claim_batch.rs, blob 12a71786bd92f683dbd0aec2ae5d6f426e651a44 at BOTH pins):
#   `ClaimOutcome::Pass => println!("PASS {}", function)`; a plain Fail prints `FAIL <function>`; every
#   other outcome prints a QUALIFIED line — `FAIL <fn> (runtime error …)`, `FAIL <fn> (returned … code N…)`,
#   `UNOBSERVED <fn> (…)`, `PANICKED <fn> (…)` — and its own process exit is 0 if everything passed and the
#   run's cost-accounting partition reconciled, 1 if anything failed, 2 if anything was unobserved.
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
#   and must not be counted -- hence the `]` in the prefix. The process exit
#   code is checked SEPARATELY from the text, as rdpo does: all-HELD requires rc 0, one-or-more FAILED
#   requires rc 1; rc 2 with only bare verdict lines, rc 124, a signal 128+N or a missing status is
#   NO_OBSERVATION — including the case where every claim passed but the run's cost-accounting partition
#   refused, which this runner reports as exit 1 with no FAIL line.
# BEFORE ANY LEG: the names gate. validate_explicit_functions in that same binary refuses a stale or
#   typoed --function name, which would spend the leg's 900 s for nothing; so each expected name is
#   grepped for `^test fn <name>(` in the CHECKED-OUT entry file first, and a missing name exits 2.
#
# NO MUTATION, NO WRITES OUTSIDE OUT, NO HIDDEN STATE: no rm, no push, no commit, no git checkout of a
#   patch, no process matching, no /opt/gunbc or other instance root; the origin is hard-coded as
#   https://github.com/gunb-ai/gunbc and no other tree or home path is read. Every write is under OUT,
#   made after OUT was created by an exclusive quoted `mkdir --` (no -p) following an absence precheck
#   that refuses ANY existing entry — file, directory, symlink or dangling symlink.
set -u

J=fierce-wren-19-r4r5-db56530f
U=https://github.com/gunb-ai/gunbc
H4=db56530fcb186370e3dbf1aa3c69ded324daa25d; T4=48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc
H5=dff57af790f923066d61721e89ff751a91021278; T5=356f2894a60edda51cdbe79879b2fb3b7aea75c4
E4A=src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag; B4A=332cc981fd194bb9f02f16ff6e3c14d81b874e2b
E4B=src/v2/test/claim/rust_crate_partition_witness.dag;            B4B=abc6313eb8c4d49ad74a7bcafad36b895ea32f76
E5=dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag;    B5=adfeb29426eac9d4da67630a12d89f8eabe8754b

# name lists (space-separated: classification, the names gate, and leg_state)
N4A="witness_duplicate_owner_refuses_holds witness_unowned_mirror_refuses_holds witness_shell_partition_duplicate_owner_refuses_holds witness_leaf_edit_compiles_owner_and_downstream_holds witness_leaf_edit_excludes_unrelated_packages_holds"
N4B="witness_r4_refuses_unit_graph_cycle_holds"
N5="witness_inbox_load_refused_is_refused_not_empty witness_inbox_unreplayable_history_is_refused_not_empty witness_inbox_live_receipt_removes_node_from_rows witness_inbox_revoked_receipt_reappears_with_prior witness_inbox_stale_live_receipt_refuses"
# the SAME names as one comma-separated --functions argument (the parser does csv.split(',') then trims)
csv() { local s; s=$(printf '%s,' $1); printf '%s' "${s%,}"; }
C4A=$(csv "$N4A"); C4B=$(csv "$N4B"); C5=$(csv "$N5")

T_SETUP_CLONE=300; T_SETUP_FETCH=300; T_SETUP_CHECKOUT=120; T_BUILD=2400; T_LEG=900

# FROZEN COMMAND STRINGS: what --predict-only prints (bytes + sha256 over the literal string) and what
# the run executes are the SAME variables. Expansion points named in the manifest: $OUT (the one argument),
# $BIN4 / $BIN5 (the runner built inside each pinned clone), $U and the pins (already expanded here).
CMD_CLONE_4='git init -q "$OUT/wt-r4"'
CMD_FETCH_4='git -C "$OUT/wt-r4" fetch -q --depth 1 '"$U"' '"$H4"''
CMD_CHECKOUT_4='git -C "$OUT/wt-r4" checkout -q --detach FETCH_HEAD'
CMD_BUILD_4='cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch'
CMD_LEG_R4A='"$BIN4" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$E4A"'" --functions "'"$C4A"'"'
CMD_LEG_R4B='"$BIN4" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$E4B"'" --functions "'"$C4B"'"'
CMD_CLONE_5='git init -q "$OUT/wt-r5"'
CMD_FETCH_5='git -C "$OUT/wt-r5" fetch -q --depth 1 '"$U"' '"$H5"''
CMD_CHECKOUT_5='git -C "$OUT/wt-r5" checkout -q --detach FETCH_HEAD'
CMD_BUILD_5='cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch'
CMD_LEG_R5='"$BIN5" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$E5"'" --functions "'"$C5"'"'

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
# CLAIM CLASSIFIER: bare lines only, one per expected name, plus the runner's own exit code checked apart.
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
pp_check() { # $1=output file -> "OK <the record>" or the reason it is not an observation
  local f=$1 n line
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
  ck csv-r4a 5 "$(printf '%s' "$C4A" | awk -F',' '{print NF}')"
  ck csv-r4b 1 "$(printf '%s' "$C4B" | awk -F',' '{print NF}')"
  ck csv-r5 5 "$(printf '%s' "$C5" | awk -F',' '{print NF}')"
  ck csv-no-spaces 0 "$(printf '%s%s%s' "$C4A" "$C4B" "$C5" | grep -c ' ')"
  ck names-match-csv 0 "$( ( IFS=','; set -f; for x in $C4A $C4B $C5; do case " $N4A $N4B $N5 " in *" $x "*) ;; *) echo bad;; esac; done ) | grep -c bad)"
  ck name-shape 0 "$(printf '%s\n' $N4A $N4B $N5 | grep -cvE '^[a-z0-9_]+$')"
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
  ck scope-ok /user.slice/user-1000.slice/x/fierce-wren-19-r4r5-db56530f.scope "$(scope_gate "0::/user.slice/user-1000.slice/x/fierce-wren-19-r4r5-db56530f.scope" fierce-wren-19-r4r5-db56530f)"
  sg() { scope_gate "$1" "$2" >/dev/null && ck "$3" refused accepted || ck "$3" refused refused; }
  sg "0::/" fierce-wren-19-r4r5-db56530f scope-root
  sg "0::/x/other.scope" fierce-wren-19-r4r5-db56530f scope-foreign
  sg "0::/x/fierce-wren-19-r4r5-db56530f.scope.d" fierce-wren-19-r4r5-db56530f scope-trailing
  sg "0::/x/prefierce-wren-19-r4r5-db56530f.scope" fierce-wren-19-r4r5-db56530f scope-prefix-hit
  sg "0::/x/fierce-wren-19-r4r5-db56530f.scope/sub" fierce-wren-19-r4r5-db56530f scope-nested
  sg "1::/x/fierce-wren-19-r4r5-db56530f.scope" fierce-wren-19-r4r5-db56530f scope-not-v2
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
  ck capped-sum 8940 "$(( 2*(T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT) + 2*T_BUILD + 3*T_LEG ))"
  ck scope-wall-gte-sum ok "$([ 9900 -ge $(( 2*(T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT) + 2*T_BUILD + 3*T_LEG )) ] && echo ok || echo short)"
  ck driver-wall-gte-scope ok "$([ 10500 -ge 9900 ] && echo ok || echo short)"
  # ---------------- the frozen argv, tokenized exactly as bash -c will tokenize it ----------------
  # A leg is executed as: timeout <t> bash -c "<the frozen string>". So what claim_batch's arg loop sees is
  # that string AFTER bash has expanded it. eval "set -- <string>" reproduces that expansion here, in this
  # container, without running a claim: it turns the packet's own argv into data and checks it. This is the
  # assertion that would have caught v1's two fatal argv defects.
  # PARSER_FLAGS is the flag set accepted by src/v1/stage0/src/bin/claim_batch.rs at both pins (blob
  # 12a71786bd92f683dbd0aec2ae5d6f426e651a44); anything else makes that file print
  # "claim_batch: unknown argument: ..." and exit 2, which this packet scores NO_OBSERVATION, not FAIL.
  PARSER_FLAGS='--source-root --entry --function --functions --claim-run --wet --hermetic --record --eval-budget-ms --fixture-store --pre-push --print-entry-closure --outcomes-out'
  BIN4=${BIN4:-/fixture/wt-r4/target/release/claim_batch}
  BIN5=${BIN5:-/fixture/wt-r5/target/release/claim_batch}
  argv_check() { # argv_check <label> <frozen string> <entry> <names...>: prints one line per defect, silent if sound
    local label=$1 cmd=$2 ent=$3; shift 3; local names="$*"; local i j x fv='' nf nn
    eval "set -- $cmd" 2>/dev/null || { echo "$label: the frozen string does not tokenize as a command"; return; }
    [ "$#" -gt 1 ] || { echo "$label: nothing follows the binary path"; return; }
    [ "$1" = "$BIN4" ] || [ "$1" = "$BIN5" ] || { echo "$label: argv[0] is '$1', not the pinned runner path"; return; }
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
  ck argv-R4a-sound "" "$(argv_check R4a "$CMD_LEG_R4A" "$E4A" $N4A)"
  ck argv-R4b-sound "" "$(argv_check R4b "$CMD_LEG_R4B" "$E4B" $N4B)"
  ck argv-R5-sound  "" "$(argv_check R5  "$CMD_LEG_R5"  "$E5"  $N5)"
  # the checker itself, in the other direction: each defect the manager named must be reported
  ck argv-ctl-subcommand 1 "$(argv_check ctl "\"$BIN4\" claim_batch --source-""root dag --entry x --functions a" x a | grep -c "literal 'claim_batch'")"
  ck argv-ctl-spaces     1 "$(argv_check ctl "\"$BIN4\" --source-root dag --entry x --functions \"a b\"" x a b | grep -c 'contains a space')"
  ck argv-ctl-badflag    1 "$(argv_check ctl "\"$BIN4\" --entry x --functions a --frobnicate" x a | grep -c 'not a flag')"
  ck argv-ctl-typo       1 "$(argv_check ctl "\"$BIN4\" --entry x --functions \"a,zzz\"" x a b | grep -c 'not one of the leg' )"
  ck argv-ctl-missing    1 "$(argv_check ctl "\"$BIN4\" --entry x --functions \"a\"" x a b | grep -c 'missing from --functions')"
  ck argv-ctl-double     1 "$(argv_check ctl "\"$BIN4\" --entry x --entry y --functions a" x a | grep -c 'appears 2 times')"
  ck argv-ctl-noentry    1 "$(argv_check ctl "\"$BIN4\" --source-root dag --functions a" x a | grep -c -- '--entry appears 0 times')"
  ck argv-ctl-wrongentry 1 "$(argv_check ctl "\"$BIN4\" --entry other.dag --functions a" "${E4A:-x}" a | grep -c 'entry value is')"
  # ---------------- self-structure: the packet refuses to run if its own frozen text lost an invariant --
  # These are the assertions that hold the hardened frame in place AFTER the packet is staged: the packet
  # reads its own text (comment lines stripped) and refuses if a required construct is absent or an
  # excluded one has appeared. The needle is written split across two quotes so this call site can never
  # match the pattern it is looking for.
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
  inv 'timeout "$T_SETUP_FETC' 'H"' 'the fetch runs under its own capped timeout' 1
  inv 'pc=$(pp_che' 'ck' 'the completion-record gate runs on every leg' 1
  inv 'pp_check() ' '{' 'the completion-record gate is defined' 1
  inv 'PP_PRE' "FIX='^[[]process-partition" 'the completion prefix keeps the closing bracket (the -exit neighbour must not count)' 1
  inv 'any=$(grep -cE "' '^(PASS|FAIL' 'each expected name needs exactly one outcome line of any keyword' 1
  inv 'MEMMAX_G*' '1073741824' 'the booked byte bounds are derived from the launcher GiB values' 1
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

  echo "FIXTURES pass=$okn fail=$badn"; [ $badn -eq 0 ]; exit $?
fi

# ---------------------------------------------------------------- predict-only: prints the manifest, does nothing else
sha_of() { printf '%s' "$1" | sha256sum | awk '{print $1}'; }
bytes_of() { printf '%s' "$1" | wc -c | tr -d ' '; }
if [ "${1:-}" = "--predict-only" ]; then
  # Writes NOTHING, contacts nothing: no run dir, no clone, no build, no store. The bytes and sha256 below
  # are over the LITERAL frozen string, before $OUT / $BIN4 / $BIN5 are expanded at run time.
  echo "PREDICT-ONLY row=R4+R5 job=$J host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4r5_v4_manifest.md"
  echo "THIS ROW DISCRIMINATES NOTHING: presence observations, no mutation leg, none proposed."
  echo "pin_r4=$H4 tree_r4=$T4"
  echo "pin_r5=$H5 tree_r5=$T5"
  printf 'blob %s %s\n' "$E4A" "$B4A"; printf 'blob %s %s\n' "$E4B" "$B4B"; printf 'blob %s %s\n' "$E5" "$B5"
  echo "bounds=$BOOKED"
  echo "launcher_units runtime_s=$RUNTIME_S memmax_gib=$MEMMAX_G memhigh_gib=$MEMHIGH_G tasks=$TASKS_MAX cpu=$CPU_PCT%"
  echo "launcher_call=calmcrab-udrv.sh $J $RUNTIME_S $MEMMAX_G $MEMHIGH_G $TASKS_MAX $CPU_PCT% <STAGED_SCRIPT> <this packet sha256> OUT"
  echo "ledger_rows=$J-res(cap ${MEMMAX_G}GiB) $J-drv(cap 1) $J(cap 0)"
  echo "completion_record_gate=exactly-one-prefix-count-then-shape sha_pinned=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2"
  echo "verdict_rules=bare_only;one_outcome_line_per_name_any_keyword;qualified_is_NO_OBSERVATION"
  echo "timeouts_s setup_clone=$T_SETUP_CLONE setup_fetch=$T_SETUP_FETCH setup_checkout=$T_SETUP_CHECKOUT build=$T_BUILD leg=$T_LEG"
  echo "wall_capped_sum_s=$(( 2*(T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT) + 2*T_BUILD + 3*T_LEG )) scope_RuntimeMaxSec=9900 driver_RuntimeMaxSec=10500"
  echo "claims_r4a=$(printf '%s' "$C4A" | awk -F',' '{print NF}') claims_r4b=$(printf '%s' "$C4B" | awk -F',' '{print NF}') claims_r5=$(printf '%s' "$C5" | awk -F',' '{print NF}')"
  echo "exit_contract: 0=expectations_held 1=falsified_bare_FAIL 2=NO_OBSERVATION(every refusal)"
  for c in CMD_CLONE_4 CMD_FETCH_4 CMD_CHECKOUT_4 CMD_BUILD_4 CMD_LEG_R4A CMD_LEG_R4B CMD_CLONE_5 CMD_FETCH_5 CMD_CHECKOUT_5 CMD_BUILD_5 CMD_LEG_R5; do
    v="${!c}"; printf 'cmd %s bytes=%s sha256=%s\n  %s\n' "$c" "$(bytes_of "$v")" "$(sha_of "$v")" "$v"
  done
  exit 0
fi

# ---------------------------------------------------------------- usage: ONE argument, the launcher's OUT
[ $# -eq 1 ] && [ -n "$1" ] && [ "${1#-}" = "$1" ] || { echo "NO_OBSERVATION: usage: script.sh <OUT> — one argument, the launcher's fresh evidence dir (<RUNTIME>/out); a leading dash is a flag, never a path" >&2; exit 2; }
OUT=$1
export OUT
case "$OUT" in */out) ;; *) echo "NO_OBSERVATION: OUT '$OUT' does not end in /out (calmcrab-udrv v4 maps the row's evidence dir to <RUNTIME>/out)" >&2; exit 2;; esac
JOB=$(basename "$(dirname "$OUT")" 2>/dev/null || echo "")
[ -n "$JOB" ] || { echo "NO_OBSERVATION: cannot derive the booked JOB from OUT '$OUT' (expected <RUNTIME>/<JOB>/out)" >&2; exit 2; }
command -v timeout >/dev/null || { echo "NO_OBSERVATION: timeout(1) is missing and every step would be unbounded" >&2; exit 2; }
[ -x /usr/bin/time ] || { echo "NO_OBSERVATION: /usr/bin/time is missing; elapsed and peak-RSS evidence could not be recorded" >&2; exit 2; }
out_absent "$OUT" || { echo "NO_OBSERVATION: OUT $OUT already exists (any entry — file, directory, symlink or dangling symlink — refuses; a row's evidence dir is never reused)" >&2; exit 2; }
mkdir -- "$OUT" || { echo "NO_OBSERVATION: exclusive mkdir of $OUT failed (an entry appeared, or no permission)" >&2; exit 2; }
chmod 700 "$OUT"
MEMDIR=$OUT/memevents; mkdir -- "$MEMDIR" || { echo "NO_OBSERVATION: cannot make $MEMDIR" >&2; exit 2; }
mkdir -- "$OUT/logs" || { echo "NO_OBSERVATION: cannot make $OUT/logs" >&2; exit 2; }
export TMPDIR=$OUT/tmp; mkdir -- "$TMPDIR" || { echo "NO_OBSERVATION: cannot make $TMPDIR" >&2; exit 2; }
O=$OUT/receipt.txt
echo "job=$JOB booked_job=$J host=$(hostname 2>/dev/null || echo unreadable) machine_id=$(cat /etc/machine-id 2>/dev/null || echo unreadable) start=$(date -u +%FT%TZ)" > "$O"
nox() { echo "$1" >> "$O"; echo "VERDICT=NO_OBSERVATION reason=$2" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"; echo "NO_OBSERVATION: $1 (reason=$2)" >&2; exit 2; }
S=$(bin_sha "$0") || nox "cannot hash the staged script at $0" script-hash
echo "script_sha256=$S (compare with the published frozen value; a file cannot gate its own hash)" >> "$O"

# FIXTURES BEFORE ANY WORK IS SPENT: a misfired classifier ends the row with no claim attempted.
bash "$0" --classifier-fixtures > "$OUT/logs/selfcheck.log" 2>&1 || nox "the packet's own classifier fixtures misfired; no claim attempted (see logs/selfcheck.log)" fixtures
echo "fixtures: $(grep -c '^FIXTURE PASS' "$OUT/logs/selfcheck.log") pass, $(grep -c '^FIXTURE FAIL' "$OUT/logs/selfcheck.log") fail" >> "$O"

CG=$(scope_gate "$(cat /proc/self/cgroup 2>/dev/null)" "$JOB") || nox "scope gate: /proc/self/cgroup is not exactly one 0:: line whose path ends in /$JOB.scope: $(tr '\n' ' ' < /proc/self/cgroup 2>/dev/null)" scope
EV=/sys/fs/cgroup$CG/memory.events
GA=$(oom_snap "$EV" gate-a); GB=$(oom_snap "$EV" gate-b)
[ "$(oom_pair_verdict "$GA" "$GB")" = none ] || nox "memory.events at $EV unreadable or already OOM-marked at the gate: $GA -> $GB" "$([ "$(oom_pair_verdict "$GA" "$GB")" = UNKNOWN ] && echo counter-incoherent || echo oom-before-work)"
BM=$(bounds_check "/sys/fs/cgroup$CG") || nox "booked bounds: $BM" bounds
echo "scope=$CG bounds_ok=$BOOKED oom_gate=$GA->$GB" >> "$O"
echo "host recorded, not gated: the booking is enforced by the launcher's ledger rows and the bounds gate above, not by a hostname string" >> "$O"

blob_gate() { # $1=clone $2=path $3=blob
  local have; have=$(git -C "$1" rev-parse "HEAD:$2" 2>/dev/null) || { echo "missing"; return 1; }
  [ "$have" = "$3" ] || { echo "$2 is blob $have at HEAD, this packet freezes $3"; return 1; }
  printf 'blob %s %s\n' "$2" "$have" >> "$OUT/blob-gate.txt"; }

pin_clone() { # $1=worktree name $2=pin $3=tree; fetches, checks out, gates HEAD/tree/blobs/names
  local W=$OUT/$1 H T n
  H=$( [ "$1" = wt-r4 ] && printf '%s' "$H4" || printf '%s' "$H5" )
  T=$( [ "$1" = wt-r4 ] && printf '%s' "$T4" || printf '%s' "$T5" )
  timeout "$T_SETUP_CLONE" bash -c "$([ "$1" = wt-r4 ] && printf '%s' "$CMD_CLONE_4" || printf '%s' "$CMD_CLONE_5")" > "$OUT/logs/$1-init.log" 2>&1 \
    || nox "git init of $W failed or exceeded ${T_SETUP_CLONE}s" clone
  timeout "$T_SETUP_FETCH" bash -c "$([ "$1" = wt-r4 ] && printf '%s' "$CMD_FETCH_4" || printf '%s' "$CMD_FETCH_5")" > "$OUT/logs/$1-fetch.log" 2>&1 \
    || nox "shallow fetch of $H into $W failed or exceeded ${T_SETUP_FETCH}s" clone
  timeout "$T_SETUP_CHECKOUT" bash -c "$([ "$1" = wt-r4 ] && printf '%s' "$CMD_CHECKOUT_4" || printf '%s' "$CMD_CHECKOUT_5")" > "$OUT/logs/$1-checkout.log" 2>&1 \
    || nox "checkout of FETCH_HEAD in $W failed or exceeded ${T_SETUP_CHECKOUT}s" clone
  local gotH gotT
  gotH=$(git -C "$W" rev-parse HEAD 2>/dev/null || echo unreadable)
  gotT=$(git -C "$W" rev-parse 'HEAD^{tree}' 2>/dev/null || echo unreadable)
  [ "$gotH" = "$H" ] || nox "$W HEAD is $gotH, this packet freezes $H" pin
  [ "$gotT" = "$T" ] || nox "$W tree is $gotT, this packet freezes $T for $H" tree
  if [ "$1" = wt-r4 ]; then
    m=$(blob_gate "$W" "$E4A" "$B4A") || nox "blob gate: $m" blob
    m=$(blob_gate "$W" "$E4B" "$B4B") || nox "blob gate: $m" blob
    m=$(names_gate "$W" "$E4A" "$N4A") || nox "names gate: $m" names
    m=$(names_gate "$W" "$E4B" "$N4B") || nox "names gate: $m" names
  else
    m=$(blob_gate "$W" "$E5" "$B5") || nox "blob gate: $m" blob
    m=$(names_gate "$W" "$E5" "$N5") || nox "names gate: $m" names
  fi
  echo "cloned $1 head=$gotH tree=$gotT blobs+names gated" >> "$O"; }

build_pin() { # $1=worktree; builds claim_batch under the booked bounds, OOM judged across the build
  local W=$OUT/$1 k0 k1 BO B
  cd "$W" || nox "cannot cd into $W" clone
  k0=$(oom_snap "$EV" "$1-build-before")
  timeout "$T_BUILD" /usr/bin/time -v bash -c "$([ "$1" = wt-r4 ] && printf '%s' "$CMD_BUILD_4" || printf '%s' "$CMD_BUILD_5")" > "$OUT/logs/$1-build.log" 2>&1; BX=$?
  k1=$(oom_snap "$EV" "$1-build-after"); BO=$(oom_pair_verdict "$k0" "$k1")
  echo "== build $1 exit=$BX oom=$BO $(tm "$OUT/logs/$1-build.log")" >> "$O"
  [ "$BX" = 124 ] && nox "$1 build timed out at ${T_BUILD}s" build-timeout
  [ "$BX" = 0 ] || { tail -10 "$OUT/logs/$1-build.log" >> "$O"; nox "$1 build exit=$BX" build; }
  [ "$BO" = none ] || nox "$1 build oom=$BO ($k0 -> $k1)" "$([ "$BO" = OOM_KILLED ] && echo build-oom || echo counter-incoherent)"
  B=$W/target/release/claim_batch
  [ -x "$B" ] || nox "$1 build produced no executable at $B" build
  echo "binary $B sha256=$(bin_sha "$B" || echo unreadable)" >> "$O"; }

LASTOOM=""
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
  pc=$(pp_check "$OUT/$L.out")
  echo "== $L completion_record: $pc" >> "$O"
  # a leg without exactly one well-formed completion record never finished the producer's reporting path,
  # so its verdict lines are not an observation; leg_state reads this line and the leg becomes NOOBS (exit 2).
  [ "${pc%% *}" = OK ] || echo "completion NO_OBSERVATION" >> "$OUT/$L.classes"
  for cl in $3; do echo "$cl $(claim_classify "$OUT/$L.out" "$rc" "$cl")" >> "$OUT/$L.classes"; done
  cat "$OUT/$L.classes" | sed 's/^/  /' >> "$O"
  st=$(leg_state "$OUT/$L.classes" "$3")
  echo "== $L state=$st" >> "$O"; echo "$st" > "$OUT/$L.state"; }

pin_clone wt-r4
build_pin wt-r4
pin_clone wt-r5
build_pin wt-r5
# The leg commands are executed with bash -c on the FROZEN string, so the variables that string names
# must be EXPORTED: a child shell does not inherit a plain shell assignment, and a lost BIN would make
# argv[0] empty — a defect that reads as a runner failure rather than as what it is.
BIN4=$OUT/wt-r4/target/release/claim_batch; BIN5=$OUT/wt-r5/target/release/claim_batch
[ -x "$BIN4" ] || nox "runner missing at $BIN4" build
[ -x "$BIN5" ] || nox "runner missing at $BIN5" build
export OUT BIN4 BIN5

leg R4a "$CMD_LEG_R4A" "$N4A"
leg R4b "$CMD_LEG_R4B" "$N4B"
leg R5  "$CMD_LEG_R5"  "$N5"

FA=$(oom_snap "$EV" final)
FV=$(oom_pair_verdict "$LASTOOM" "$FA")
echo "== final oom vs last leg after: $LASTOOM -> $FA verdict=$FV" >> "$O"
[ "$FV" = none ] || nox "memory events moved between the last leg and the final snapshot: $LASTOOM -> $FA ($FV)" "$([ "$FV" = OOM_KILLED ] && echo oom-in-teardown || echo counter-incoherent)"
echo "memory.peak (printed, never judged): $(cat "/sys/fs/cgroup$CG/memory.peak" 2>/dev/null || echo unreadable)" >> "$O"

S4A=$(cat "$OUT/R4a.state"); S4B=$(cat "$OUT/R4b.state"); S5=$(cat "$OUT/R5.state")
echo "row R4+R5 states: R4a=$S4A R4b=$S4B R5=$S5 (presence only; this row discriminates nothing)" >> "$O"
if [ "$S4A" = NOOBS ] || [ "$S4B" = NOOBS ] || [ "$S5" = NOOBS ]; then
  echo "VERDICT=NO_OBSERVATION reason=one-or-more-legs-had-no-observation" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
  echo "NO_OBSERVATION: a leg produced no clean bare-verdict observation (R4a=$S4A R4b=$S4B R5=$S5)" >&2; exit 2
fi
if [ "$S4A" = FALSIFIED ] || [ "$S4B" = FALSIFIED ] || [ "$S5" = FALSIFIED ]; then
  echo "VERDICT=FALSIFIED R4a=$S4A R4b=$S4B R5=$S5" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
  echo "FALSIFIED: a claim printed a bare FAIL (R4a=$S4A R4b=$S4B R5=$S5)" >&2; exit 1
fi
echo "VERDICT=AS_EXPECTED claims=11 legs=3 mutation_legs=0" >> "$O"; echo "DONE $(date -u +%FT%TZ)" >> "$O"
exit 0
