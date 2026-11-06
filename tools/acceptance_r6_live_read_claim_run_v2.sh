#!/usr/bin/env bash
# =====================================================================
# HELD — NOT AUTHORIZED FOR EXECUTION — acceptance grant set row R6, srv1 — v2
#
# WHY THIS ROW IS ITS OWN PACKET AND NOT IN acceptance_r4_r5_claim_run_v2.sh
#
# R6 runs the one claim
#   dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag
#     witness_inbox_on_authority_projects_on_live_corpus
# which declares `data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree` and, on a bare
# srv1 checkout at this pin, binds the SERVED store branch: a network read of the LIVE fabric
# partition. The audit, in call order:
#
#   roadmap_acceptance_inbox_on_authority        dag/gunbc/roadmap/roadmap_acceptance_inbox.dag:140
#     -> roadmap_acceptance_event_history_load   dag/gunbc/roadmap/roadmap_authority.dag:5070
#        -> fabric_state_storage_binding         dag/gunbc/fabric/fabric_storage_binding.dag:36
#           -> roadmap_operator_close_history    dag/gunbc/roadmap/roadmap_operator_acceptance.dag:644
#              -> roadmap_operator_acceptance_history_load  same file:197
#                 -> roadmap_operator_acceptance_read       same file:132
#                    -> event_log_read_partition_with  dag/gunbc/fabric/fabric_event_log.dag:164
#                       -> fabric_storage_closure         dag/gunbc/fabric/fabric_storage_client.dag:193
#
# NO WRITES in that closure: the append primitive (event_log_append_with) lives only in
# roadmap_operator_acceptance_close / _close_append, which this claim never reaches. The closure
# walk and chain replay mutate nothing on the store.
#
# WHICH BRANCH A BARE srv1 CHECKOUT BINDS — the reason this is gated:
# `data prod_role_selection: ProdRoleSelection = ProdRoleHeldBy { srv1_daily_workspace_deployment }`
# is a committed data constant (dag/gunbc/deployment_risk.dag:71) and srv1_live_dashboard_instance
# carries exactly that deployment (dag/gunbc/roadmap/roadmap_dashboard_instance.dag:1262-1264). So
# the prod role resolves, fabric_storage_placement returns FabricStoragePlaced, and
# fabric_state_storage_binding maps EVERY placed realization — including the host's own placed
# branch, whose door_socket it discards — to FabricStorageServed { <instance endpoint>/state },
# which takes served_post: a live round trip (fabric_storage_client.dag:203-207). The SEED is read
# only under FabricStorageBindingUnplaced, i.e. where the runtime identity store is missing; a
# REFUSED placement gives HistoryLoadRefused -> AcceptanceInboxRefused, so the claim's InboxLoaded
# arm fails — a FAIL, not a seed read. There is no seed-only form of this claim without editing the
# claim or the binding, which no grant in this set authorizes.
#
# THE GATE: keen-otter's msg_3407a188 admits an EXISTING rostered reader of the live store. A claim
# runner starting a new read of the live acceptance partition is not that. So this packet is HELD:
# `--execute` without a cleared grant REFUSES (exit 3) and prints the reason. It runs only when an
# execution grant that has cleared that gate is named on the command line, and the reference is
# recorded in the run dir (grant.txt) with the host that ran it. That is an operator decision: the
# value must be the grant reference keen-otter published, and running with a fabricated one is a
# violation of the grant set, not a workaround.
#
# ARGV SHAPE, CHECKED AGAINST THE PARSER (src/v1/stage0/src/bin/claim_batch.rs :: parse_args, at the
# pin): the binary IS claim_batch — no subcommand token (a literal `claim_batch` argv[1] hits the
# `other =>` arm and exits 2); --functions is CSV (`split(',')`, trimmed), and the accepted flag set
# is --source-root/--entry/--function/--functions/--claim-run/--wet/--hermetic/--record/
# --eval-budget-ms/--fixture-store/--pre-push/--print-entry-closure/--outcomes-out. The self-test
# tokenizes the leg command as `bash -c` will and checks it against that set.
#
# RUN LOCATION: the launcher's single argument, "$HOME/runs/<JOB>/out", created exclusively; this
# process's cgroup must end in /<JOB>.scope. One claim, one leg, one attempt. NO MUTATION LEG, and
# none is proposed. A PASS is a projection match; a refusal (unreadable tail, history that will not
# replay, refused placement) is a FINDING, and the claim fails CLOSED on each — so a bare FAIL here
# is the corpus saying something, not the runner saying nothing. Anything that is not a bare
# PASS/FAIL line is NOT AN OBSERVATION -> exit 2. Gates as in the R4/R5 packet: run dir exclusive,
# scope gate, entry-path scope gate, bound gate (srv1 / srv1), blob gate on the entry file, raw
# memory evidence across the whole run, OOM refusal.
set -euo pipefail
usage() { echo "usage: $0 --predict-only | $0 --execute <execution-grant-ref> <launcher-provided-output-dir>" >&2; }
MODE=""; LIVE_GRANT=""; RUN_DIR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --predict-only) MODE=predict; shift ;;
    --execute) MODE=execute; shift; LIVE_GRANT="${1:-}"; [ -n "$LIVE_GRANT" ] || { usage; exit 2; }; shift ;;
    -*) usage; exit 2 ;;
    *) [ -z "$RUN_DIR" ] || { usage; exit 2; }; RUN_DIR="$1"; shift ;;
  esac
done
[ -n "$MODE" ] || { usage; exit 2; }
[ "$MODE" != predict ] || [ -z "$RUN_DIR" ] || { usage; exit 2; }
if [ "$MODE" = execute ] && [ -z "$RUN_DIR" ]; then usage; exit 2; fi

PIN=dff57af790f923066d61721e89ff751a91021278
ENTRY=dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag
BLOB_LIVE_WITNESS=969b980b189dafa81df4045f62f7d2e70dcc50ad
CLAIM_R6=witness_inbox_on_authority_projects_on_live_corpus
BINDING_HOST=srv1
BINDING_IDENTITY=srv1
ORIGIN=https://github.com/gunb-ai/gunbc.git
JOB_HINT="acceptance-r6-srv1"
T_BUILD=2400
T_ROW=900
JOBS=32

CMD_CLONE='git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r6"'
CMD_FETCH='git -C "$RUN_DIR/wt-r6" fetch --quiet --depth 1 origin "$PIN"'
CMD_CHECKOUT='git -C "$RUN_DIR/wt-r6" checkout --quiet --detach "$PIN"'
CMD_BUILD='cargo build --locked --release -p v1-compiler --bin claim_batch --jobs '"$JOBS"
CMD_LEG_R6='"$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$ENTRY"'" --functions "'"$CLAIM_R6"'"'
sha_of() { printf '%s' "$1" | sha256sum | awk '{print $1}'; }
bytes_of() { printf '%s' "$1" | wc -c | tr -d ' '; }

if [ "$MODE" = predict ]; then
  # PREDICT-ONLY writes NOTHING: no run dir, no file, no clone, no claim, no store contact.
  echo "PREDICT-ONLY: the held R6 packet. Nothing is executed and the live store is not touched."
  echo "pin=$PIN"
  printf 'blob %s %s\n' "$ENTRY" "$BLOB_LIVE_WITNESS"
  echo "claim set: $CLAIM_R6 (1 claim, 1 leg; expected PASS; a refusal is a FINDING)"
  echo "launch: systemd-run --user --scope --unit=<JOB> bash $0 --execute <execution-grant-ref> \"\$HOME/runs/<JOB>/out\""
  echo "job_hint=$JOB_HINT"
  echo "expansion points: RUN_DIR=<the launcher's single argument>  BIN=<RUN_DIR>/wt-r6/target/release/claim_batch"
  echo "the bytes/sha256 below are over the LITERAL frozen string, before those are expanded"
  for c in CMD_CLONE CMD_FETCH CMD_CHECKOUT CMD_BUILD CMD_LEG_R6; do
    v="${!c}"
    printf 'cmd %s bytes=%s sha256=%s\n  %s\n' "$c" "$(bytes_of "$v")" "$(sha_of "$v")" "$v"
  done
  exit 0
fi

# THE GATE. Reaching here means --execute, and --execute without a grant reference naming a cleared
# live-read gate is refused, not run. The refusal prints what would have been read.
if [ -z "$LIVE_GRANT" ]; then
  echo "R6 REFUSED: this claim reads the live fabric acceptance partition over the SERVED store" >&2
  echo "branch (FabricStorageServed -> served_post closure), from a NEW reader. keen-otter's" >&2
  echo "msg_3407a188 admits an existing rostered reader only. Run as:" >&2
  echo "  $0 --execute <execution-grant-ref> \"\$HOME/runs/<JOB>/out\"" >&2
  exit 3
fi

command -v timeout >/dev/null || { echo "timeout(1) missing; every leg is unbounded" >&2; exit 2; }
ulimit -c 0

JOB=$(basename "$(dirname "$RUN_DIR")" 2>/dev/null || echo "")
[ -n "$JOB" ] || { echo "cannot derive the booked job token from $RUN_DIR" >&2; exit 2; }
[ "$(basename "$RUN_DIR")" = out ] || { echo "run dir $RUN_DIR does not end in /out (the launcher passes \$HOME/runs/<JOB>/out)" >&2; exit 2; }
mkdir -- "$RUN_DIR" 2>/dev/null || { echo "run dir $RUN_DIR already exists (or its job dir is missing); a run dir is never reused" >&2; exit 3; }
chmod 700 "$RUN_DIR"
mkdir -- "$RUN_DIR/raw" 2>/dev/null || { echo "cannot create $RUN_DIR/raw" >&2; exit 2; }
mkdir -- "$RUN_DIR/logs" "$RUN_DIR/tmp" 2>/dev/null || { echo "cannot create $RUN_DIR/logs or $RUN_DIR/tmp" >&2; exit 2; }
chmod 700 "$RUN_DIR/raw" "$RUN_DIR/logs" "$RUN_DIR/tmp"
export TMPDIR="$RUN_DIR/tmp"
export GUNBC_RUN_DIR="$RUN_DIR"
echo "run_dir=$RUN_DIR job_token=$JOB grant_ref=$LIVE_GRANT"
echo "grant_ref=$LIVE_GRANT job_token=$JOB" > "$RUN_DIR/grant.txt"

# ---8< FUNCTIONS BEGIN --- (identical vocabulary to the R4/R5 packet, so one self-test covers both)
ev_capture() {
  local b="$1" d="$RUN_DIR/raw"
  {
    date -u "+capture_at=%Y-%m-%dT%H:%M:%SZ boundary=$b"
    cat /sys/fs/cgroup/cgroup.controllers 2>/dev/null || echo "controllers=UNREADABLE"
    for f in memory.current memory.peak memory.min memory.high memory.max memory.events memory.stat; do
      p=/sys/fs/cgroup"$f"
      if [ -n "$(cat /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null)" ]; then
        leaf=$(awk -F: '$1==0{print $3; exit}' /proc/self/cgroup 2>/dev/null)
        if [ -n "$leaf" ] && [ "$leaf" != "/" ] && [ -r "/sys/fs/cgroup$leaf/$f" ]; then
          p="/sys/fs/cgroup$leaf/$f"
        fi
      fi
      echo "### $f ($p)"; sed 's/^/  /' "$p" 2>/dev/null || echo "  UNREADABLE"
    done
    dmesg -T 2>/dev/null | grep -iE "out of memory|oom-kill|killed process" | tail -10 || echo "dmesg_oom=none_or_denied"
  } > "$d/ev-$b.raw" 2>/dev/null
  journalctl -k -q --since "2 hours ago" 2>/dev/null | grep -iE "oom|out of memory|killed process" | tail -20 >> "$d/ev-$b.oomjournal" || true
}
raw_verify() {
  local miss="" b
  local need=(start clone_fetch_after build_before build_after leg_R6_before leg_R6_after)
  for b in r4_clone_fetch_after r5_clone_fetch_after r4_build_before r5_build_before mutation r6_mutation; do
    if [ -e "$RUN_DIR/raw/ev-$b.raw" ]; then echo "STOWAWAY raw boundary ev-$b.raw; this packet defines no $b" >&2; exit 2; fi
  done
  for b in "${need[@]}"; do
    [ -s "$RUN_DIR/raw/ev-$b.raw" ] || miss="$miss $b"
  done
  [ -z "$miss" ] || { echo "RAW MEMORY EVIDENCE MISSING:$miss" >&2; exit 2; }
}
mem_events() {
  local out="$1"
  {
    echo "### memory.events, whole cgroup tree; the cgroup line is the run's own"
    awk -F: '$1==0{print "self_cgroup="$3; exit}' /proc/self/cgroup 2>/dev/null
    echo "### memory.* event counters AND the oom/oom_kill lines"; cat /sys/fs/cgroup/memory.events 2>/dev/null
    root=/sys/fs/cgroup
    leaf=$(awk -F: '$1==0{print $3; exit}' /proc/self/cgroup 2>/dev/null)
    [ -n "$leaf" ] && [ "$leaf" != "/" ] && root="$root$leaf"
    echo "### memory.events of $root"; cat "$root/memory.events" 2>/dev/null
    cat /sys/fs/cgroup/memory.events.local 2>/dev/null
    echo "### memory.peak of $root"; cat "$root/memory.peak" 2>/dev/null
    cat /sys/fs/cgroup/memory.pressure 2>/dev/null
    echo "### oom lines from dmesg"; dmesg -T 2>/dev/null | grep -iE "oom|out of memory|killed process"
    echo "### oom lines from the journal"; journalctl -k -q --since "2 hours ago" 2>/dev/null | grep -iE "oom|out of memory|killed process"
  } > "$out" 2>/dev/null
}
ev_count() { grep -cE "^($1) " "$RUN_DIR/raw/$2" 2>/dev/null || true; }
ev_sum()   { local n=0 b; for b in "$@"; do n=$(( n + $(ev_count PASS "ev-$b.raw") )); done; echo "$n"; }
ev_parse() {
  if [ ! -s "$RUN_DIR/raw/$1" ]; then echo "EMPTY CLAIM LOG: $1" >&2; exit 2; fi
  local n=0
  for b in $2; do
    if [ ! -s "$RUN_DIR/raw/ev-$b.raw" ]; then echo "MISSING RAW BOUNDARY: ev-$b.raw" >&2; exit 2; fi
    n=$(( n + $(grep -c "^PASS " "$RUN_DIR/raw/ev-$b.raw" 2>/dev/null || true) ))
  done
  echo "$n"
}
classify() {
  local leg="$1" expected="$2" n
  n=$(grep -cE '^(PASS|FAIL) ' "$leg" 2>/dev/null || true)
  if [ "$n" -lt "$expected" ]; then echo "$leg: INCONCLUSIVE — $(wc -l < "$leg") lines, expected $expected bare verdicts; exit 2"; exit 2; fi
  local out other
  out=$(grep -E '^(PASS|FAIL) ' "$leg" | tr '\n' ' ')
  other=$(grep -vE '^(PASS|FAIL) ' "$leg" | grep -v '^$' | head -3 | tr '\n' '|')
  if [ -n "$other" ]; then echo "$leg: NOT AN OBSERVATION — non-verdict line(s): ${other}...; exit 2"; exit 2; fi
  echo "$out"
}
scope_ok() { # <cgroup-file-content> <job-token> -> OK or a reason
  local C="$1" J="$2" N P
  N=$(printf '%s\n' "$C" | grep -c . || true)
  [ "$N" = 1 ] || { echo "cgroup has $N lines, expected exactly 1"; return; }
  case "$C" in 0::/*) ;; *) echo "cgroup line does not start with 0::"; return ;; esac
  P=${C#0::}
  { [ -n "$P" ] && [ "$P" != "/" ]; } || { echo "cgroup path is the root cgroup"; return; }
  case "$P" in
    */"$J".scope) echo OK ;;
    *) echo "cgroup path does not end in /$J.scope" ;;
  esac
}
require_dir_gate() {
  local wt="$1" pin="$2" maxclones="$3" job
  job=$(basename "$(dirname "$RUN_DIR")" 2>/dev/null || echo "")
  [ -n "$job" ] || { echo "RUN-DIR GATE: cannot derive the booked job token from $RUN_DIR" >&2; exit 3; }
  local outs extra
  outs=$(find "$(dirname "$RUN_DIR")" -maxdepth 2 -type d -name out 2>/dev/null | sort)
  if [ "$(printf '%s\n' "$outs" | grep -c . || true)" -ne 1 ]; then
    extra=$(printf '%s\n' "$outs" | grep -v "^$RUN_DIR\$" | head -5)
    echo "RUN-DIR GATE: more than one out dir under the booked job dir $(dirname "$RUN_DIR"); others: ${extra:-none}" >&2
    exit 3
  fi
  [ "$outs" = "$RUN_DIR" ] || { echo "RUN-DIR GATE: resolved run dir $outs != $RUN_DIR" >&2; exit 3; }
  local clones
  clones=$(find "$RUN_DIR" -maxdepth 1 -mindepth 1 -type d -name "wt-*" 2>/dev/null | sort)
  if [ "$(printf '%s\n' "$clones" | grep -c . || true)" -gt "$maxclones" ]; then
    echo "RUN-DIR GATE: more than $maxclones pinned clones under $RUN_DIR" >&2
    exit 3
  fi
  [ -d "$wt" ] || { echo "RUN-DIR GATE: clone $wt missing" >&2; exit 3; }
  case "$wt" in "$RUN_DIR"/wt-*) ;; *) echo "RUN-DIR GATE: clone $wt is not inside the run dir" >&2; exit 3 ;; esac
  local head
  head=$(git -C "$wt" rev-parse HEAD 2>/dev/null || echo "unreadable")
  [ "$head" = "$pin" ] || { echo "RUN-DIR GATE: clone $wt HEAD $head != pinned commit $pin" >&2; exit 3; }
}
require_scope_gate() {
  local wt="$1"; shift
  local n
  n=$(git -C "$wt" ls-files -- "$@" | grep -c . || true)
  [ "$n" -eq $# ] || { echo "SCOPE GATE: $n of $# grant-set entry paths tracked in $wt" >&2; exit 3; }
}
require_bound_gate() {
  local h i
  h=$(hostname 2>/dev/null || echo "unreadable")
  [ "$h" = "$BINDING_HOST" ] || { echo "BOUND GATE: host is $h, this grant is bound to $BINDING_HOST" >&2; exit 3; }
  i=$(cat /etc/machine-id 2>/dev/null || echo "unreadable")
  [ -n "$i" ] || { echo "BOUND GATE: host identity unreadable" >&2; exit 3; }
  echo "host=$h identity_seen=$i bound_identity=$BINDING_IDENTITY" > "$RUN_DIR/binding.txt"
  local m
  for m in $(grep -hoiE 'killed process [0-9]+ \(([^)]*)\)|Memory cgroup out of memory: Killed process [0-9]+ \(([^)]*)\)' "$RUN_DIR"/raw/*.oomjournal "$RUN_DIR"/raw/mem.events 2>/dev/null); do
    case "$m" in
      *claim_batch*|*cargo*|*rustc*|*git*|*"$(basename "$0")"*)
        echo "MEMORY DECIDED THE RUN: OOM killer took $m" >&2
        exit 3;;
    esac
  done
  if grep -qE '^oom_kill [1-9]' "$RUN_DIR"/raw/mem.events 2>/dev/null; then
    echo "MEMORY DECIDED THE RUN: memory.events reports oom_kill" >&2
    exit 3
  fi
  if grep -qE '^(oom|oom_group_kill) [1-9]' "$RUN_DIR"/raw/mem.events 2>/dev/null \
     && grep -qE '^max [1-9]' "$RUN_DIR"/raw/mem.events 2>/dev/null; then
    echo "MEMORY DECIDED THE RUN: unkillable cgroup (oom/oom_group_kill) at a limit breach" >&2
    exit 3
  fi
  if grep -qE '^max [1-9]' "$RUN_DIR"/raw/mem.events 2>/dev/null; then
    local cur lim
    cur=$(awk '/^### memory.current /{getline; gsub(/[[:space:]]/,""); print; exit}' "$RUN_DIR"/raw/ev-start.raw 2>/dev/null)
    lim=$(awk '/^### memory.max /{getline; gsub(/[[:space:]]/,""); print; exit}' "$RUN_DIR"/raw/ev-start.raw 2>/dev/null)
    case "$cur" in ''|*[!0-9]*) cur="" ;; esac
    case "$lim" in ''|*[!0-9]*) lim="" ;; esac
    if [ -n "$cur" ] && [ -n "$lim" ] && [ "$cur" -ge "$lim" ]; then
      echo "MEMORY DECIDED THE RUN: cgroup was at its limit ($cur >= $lim) at the boundary that also reports max" >&2
      exit 3
    fi
  fi
}
require_blob_gate() {
  local wt="$1"; shift
  local pair path want have
  for pair in "$@"; do
    path="${pair%%:*}"; want="${pair##*:}"
    have=$(git -C "$wt" rev-parse "HEAD:$path" 2>/dev/null || echo "missing")
    [ "$have" = "$want" ] || { echo "BLOB GATE: $path is blob $have at $wt HEAD, this packet freezes $want" >&2; exit 2; }
    printf 'blob %s %s\n' "$path" "$have" >> "$RUN_DIR/blob-gate.txt"
  done
}
# ---8< FUNCTIONS END ---

CGRAW=$(cat /proc/self/cgroup 2>/dev/null) || { echo "cannot read /proc/self/cgroup" >&2; exit 3; }
S=$(scope_ok "$CGRAW" "$JOB")
[ "$S" = OK ] || { echo "SCOPE GATE: $S (raw=$(printf '%s' "$CGRAW" | tr '\n' '|'))" >&2; exit 3; }
echo "scope ok cgroup=${CGRAW#0::} job_token=$JOB hint=$JOB_HINT"

ev_capture start
eval "$CMD_CLONE"
eval "$CMD_FETCH"
eval "$CMD_CHECKOUT"
ev_capture clone_fetch_after
require_dir_gate "$RUN_DIR/wt-r6" "$PIN" 1
require_scope_gate "$RUN_DIR/wt-r6" "$ENTRY"
require_bound_gate
require_blob_gate "$RUN_DIR/wt-r6" "$ENTRY:$BLOB_LIVE_WITNESS"
cd "$RUN_DIR/wt-r6"
ev_capture build_before
timeout --kill-after=30 "$T_BUILD" bash -c "$CMD_BUILD" > "$RUN_DIR/logs/build.log" 2>&1 \
  || { tail -10 "$RUN_DIR/logs/build.log" >&2; exit 1; }
ev_capture build_after
BIN="$RUN_DIR/wt-r6/target/release/claim_batch"
[ -x "$BIN" ] || { echo "built runner not found at $BIN" >&2; exit 1; }
mem_events "$RUN_DIR/raw/mem.events"
require_bound_gate

ev_capture leg_R6_before
if ! timeout --kill-after=30 "$T_ROW" bash -c "$CMD_LEG_R6" > "$RUN_DIR/raw/leg_R6.out" 2>"$RUN_DIR/logs/r6.log"; then
  tail -12 "$RUN_DIR/logs/r6.log" >&2
  echo "R6 FAILED to run at all; nothing observed; one attempt, no re-run" >&2
  exit 1
fi
cp "$RUN_DIR/raw/leg_R6.out" "$RUN_DIR/raw/ev-leg_R6_after.raw"
ev_capture leg_R6_after
classify "$RUN_DIR/raw/ev-leg_R6_after.raw" 1 | sed "s|^|R6: |"
raw_verify
require_dir_gate "$RUN_DIR/wt-r6" "$PIN" 1
require_scope_gate "$RUN_DIR/wt-r6" "$ENTRY"
require_bound_gate
echo "R6=$([ "$(grep -c '^PASS ' "$RUN_DIR/raw/ev-leg_R6_after.raw")" = 1 ] && echo PASS || echo "FAIL (a refusal is a FINDING)") claims=1 legs=1"
