#!/usr/bin/env bash
# =====================================================================
# EXECUTION PUBLISHED — acceptance grant set rows R4 and R5, srv1, SEED-ONLY (claims only) — v2
#
# WHAT THE GRANT ASKS, AND WHAT THIS PACKET RUNS
#
# calm-eagle-670's acceptance grant set names rows R4, R5 and R6. This packet is FROZEN for R4 and
# R5. R6 is SPLIT OUT — the audit at the foot is why; it is NOT executed here.
#
#   R4  pin = current main, read as origin/main at the freeze: db56530fcb186370e3dbf1aa3c69ded324daa25d
#       (2026-11-06T03:00Z). main had moved eight commits past dff57af7 and moved twice during the
#       freeze; the packet pins ONE commit and the entry blobs it gates are identical at dff57af7,
#       at 551f1d007e, at e98d38e791 and here — so the pin choice costs nothing and the pin (not
#       origin) is what bounds the run.
#         src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag  (five claims, one leg)
#         src/v2/test/claim/rust_crate_partition_witness.dag            (one claim, its own --entry)
#   R5  pin = dff57af790f923066d61721e89ff751a91021278 (the #15936 squash):
#         dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag   (five claims, one leg)
#   Expected: every claim PASS. A bare FAIL is a FALSIFIED grant -> exit 1 at that row, no further
#   row runs (one attempt, no re-run). Anything that is not a bare PASS/FAIL line is NOT AN
#   OBSERVATION -> exit 2, never a bare FAIL.
#
# NO MUTATION LEG, PER ROW. R4: none, none proposed. R5: none, none proposed. R6 (in its own held
# packet): none, none proposed. No `mutation` boundary is defined anywhere in this packet, the row
# sets never name one, and `raw_verify` REFUSES (exit 2) if an ev-mutation.raw — or any R6-shaped
# boundary — appears in the run dir. The absence is stated so no reader takes a PASS as evidence
# about a mutation that was never attempted.
#
# THE ARGV SHAPE, CHECKED AGAINST THE PARSER RATHER THAN ASSUMED
# (src/v1/stage0/src/bin/claim_batch.rs :: fn parse_args, read at the pin):
#  * the binary IS claim_batch — there is no subcommand token. `claim_batch` as argv[1] falls to the
#    `other =>` arm, prints "claim_batch: unknown argument: claim_batch" and exits 2. So every leg
#    here is `<path>/claim_batch --source-root … --entry … --functions …`, no subcommand token.
#  * --functions is CSV: `csv.split(',')` then trim, so the list is ONE comma-separated argument with
#    no spaces inside a name. Space-separated names would ask for one function whose name contains
#    spaces, and nothing in the entry file would match.
#  * the accepted flags are exactly: --source-root, --entry, --function, --functions, --claim-run,
#    --wet, --hermetic, --record, --eval-budget-ms, --fixture-store, --pre-push,
#    --print-entry-closure, --outcomes-out. This packet passes only the first four; the self-test
#    tokenizes each leg command exactly as `bash -c` will and rejects any token outside that set,
#    any function token containing a space or an empty CSV field, and any name outside ^[a-z0-9_]+$.
#
# RUN LOCATION: the launcher's single argument, per the published convention —
#   systemd-run --user --scope --unit=<JOB> … bash acceptance_r4_r5_claim_run_v2.sh "$HOME/runs/<JOB>/out"
# The packet takes EXACTLY ONE positional: the output dir, which the launcher's <JOB> dir parents and
# which the launcher does NOT create. `<JOB>` is named in the packet (see JOB_HINT) and is derived
# back out of the argument as basename(dirname(<out>)). No instance root is assumed: nothing here
# claims /opt/gunbc exists; the run dir IS the launcher's argument, and the scope gate ties this
# process's cgroup to that same <JOB> token, end-anchored, so a run launched outside its booked scope
# is refused rather than measured against somebody else's cgroup.
#
# Memory: `mem.events` carries every memory.* event for this run's cgroup, from before the first git
# command to after the last row, OOM-killer lines included. The cgroup is the run's OWN leaf (read
# from /proc/self/cgroup and verified against the booked <JOB>.scope); the dump names the path it
# took. `require_bound_gate` refuses any line showing the OOM killer taking a process of THIS run,
# an unkillable cgroup at a limit breach, or a limit reached at the current limit — so the file can
# show the whole range and still refuse the run if memory decided anything about it. Each claim leg
# gets its own memory.events sample before and after.
#
# The scope gate (the grant's entry files tracked in the checkout), the run-dir gate and the
# host-identity gate are enforced after the clone AND after every claim leg; a violation exits 3.
# The run dir is created exclusive (mkdir, no -p); a second <JOB>/out under the booked <JOB> dir
# appearing mid-run exits 3 with the paths listed.
#
# v1's two fatal argv defects (a literal `claim_batch` subcommand token, and space-separated
# --functions) are what v2 fixes, together with the run-location inconsistency; all three are now
# covered by assertions in acceptance_r4_r5_selfcheck_v2.sh rather than by reading.
set -euo pipefail
usage() { echo "usage: $0 --predict-only | $0 <launcher-provided-output-dir>" >&2; }
PREDICT_ONLY=0
if [ "$#" -eq 1 ] && [ "$1" = "--predict-only" ]; then
  PREDICT_ONLY=1
elif [ "$#" -eq 1 ] && [ -n "$1" ] && [ "${1#-}" = "$1" ]; then
  # a leading dash is a flag, never a path: an unknown option must not be swallowed as a run dir
  RUN_DIR="$1"
else
  usage; exit 2
fi

# R4: main at freeze. R5: the #15936 squash. Two pins, one packet; each clone gates its own.
PIN_R4=db56530fcb186370e3dbf1aa3c69ded324daa25d
PIN_R5=dff57af790f923066d61721e89ff751a91021278
# The entry blobs at BOTH pins (`git rev-parse <pin>:<path>`, 2026-11-06T02:52Z, verified identical
# at dff57af7, 551f1d007e, e98d38e791 and db56530fcb).
BLOB_R4_STAGE0=332cc981fd194bb9f02f16ff6e3c14d81b874e2b
BLOB_R4_CRATE=abc6313eb8c4d49ad74a7bcafad36b895ea32f76
BLOB_R5_INBOX=adfeb29426eac9d4da67630a12d89f8eabe8754b
ENTRY_R4_STAGE0=src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag
ENTRY_R4_CRATE=src/v2/test/claim/rust_crate_partition_witness.dag
ENTRY_R5_INBOX=dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag
# The claim set per leg, as ONE comma-separated --functions value (the parser splits on ','). No
# name contains a space; no row set names a mutation leg.
ROWS_R4A="witness_duplicate_owner_refuses_holds,witness_unowned_mirror_refuses_holds,witness_shell_partition_duplicate_owner_refuses_holds,witness_leaf_edit_compiles_owner_and_downstream_holds,witness_leaf_edit_excludes_unrelated_packages_holds"
ROWS_R4B="witness_r4_refuses_unit_graph_cycle_holds"
ROWS_R5="witness_inbox_load_refused_is_refused_not_empty,witness_inbox_unreplayable_history_is_refused_not_empty,witness_inbox_live_receipt_removes_node_from_rows,witness_inbox_revoked_receipt_reappears_with_prior,witness_inbox_stale_live_receipt_refuses"
EXPECT_R4A=5; EXPECT_R4B=1; EXPECT_R5=5
# The bound host (srv1) and the bound host identity: checked after every leg, never assumed.
BINDING_HOST=srv1
BINDING_IDENTITY=srv1
ORIGIN=https://github.com/gunb-ai/gunbc.git
JOB_HINT="acceptance-r4r5-srv1"   # the booked scope unit; the launcher's <JOB> must match this shape
# The launcher values (calmcrab-udrv convention), frozen rather than defaulted per host. TWO cold
# release builds (one per pin) and three legs: wall >= 2*T_BUILD + 3*T_ROW + two clones.
RUN_TIMEOUT=21600
T_BUILD=2400
T_ROW=900
JOBS=32                            # frozen, not nproc: the command manifest must hash identically
                                   # wherever it is read, and 32 is the booked job count

# EVERY COMMAND THIS PACKET RUNS, as a string: printed by --predict-only, executed from THE SAME
# VARIABLE by --execute. There is no second copy to drift, so the audited bytes are the run bytes
# (the redirections are attached at the call site and all point inside the run dir).
CMD_CLONE_R4='git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r4"'
CMD_FETCH_R4='git -C "$RUN_DIR/wt-r4" fetch --quiet --depth 1 origin "$PIN_R4"'
CMD_CHECKOUT_R4='git -C "$RUN_DIR/wt-r4" checkout --quiet --detach "$PIN_R4"'
CMD_BUILD_R4='cargo build --locked --release -p v1-compiler --bin claim_batch --jobs '"$JOBS"
CMD_LEG_R4A='"$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$ENTRY_R4_STAGE0"'" --functions "'"$ROWS_R4A"'"'
CMD_LEG_R4B='"$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$ENTRY_R4_CRATE"'" --functions "'"$ROWS_R4B"'"'
CMD_CLONE_R5='git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r5"'
CMD_FETCH_R5='git -C "$RUN_DIR/wt-r5" fetch --quiet --depth 1 origin "$PIN_R5"'
CMD_CHECKOUT_R5='git -C "$RUN_DIR/wt-r5" checkout --quiet --detach "$PIN_R5"'
CMD_BUILD_R5='cargo build --locked --release -p v1-compiler --bin claim_batch --jobs '"$JOBS"
CMD_LEG_R5='"$BIN5" --source-root dag --source-root src/v1 --source-root src/v2 --entry "'"$ENTRY_R5_INBOX"'" --functions "'"$ROWS_R5"'"'
sha_of() { printf '%s' "$1" | sha256sum | awk '{print $1}'; }
bytes_of() { printf '%s' "$1" | wc -c | tr -d ' '; }

if [ "$PREDICT_ONLY" = 1 ]; then
  # PREDICT-ONLY writes NOTHING: no run dir, no file, no clone, no claim.
  echo "PREDICT-ONLY: the R4/R5 seed-only packet (no mutation leg). Nothing runs."
  echo "pin_r4=$PIN_R4"
  echo "pin_r5=$PIN_R5"
  printf 'blob %s %s\n' "$ENTRY_R4_STAGE0" "$BLOB_R4_STAGE0"
  printf 'blob %s %s\n' "$ENTRY_R4_CRATE" "$BLOB_R4_CRATE"
  printf 'blob %s %s\n' "$ENTRY_R5_INBOX" "$BLOB_R5_INBOX"
  echo "launch: systemd-run --user --scope --unit=<JOB> bash $0 \"\$HOME/runs/<JOB>/out\""
  echo "job_hint=$JOB_HINT  (the packet derives <JOB> as basename(dirname(<out>)) and requires this process's cgroup to end in /<JOB>.scope)"
  echo "expansion points: RUN_DIR=<the launcher's single argument>  BIN=<RUN_DIR>/wt-r4/target/release/claim_batch  BIN5=<RUN_DIR>/wt-r5/target/release/claim_batch"
  echo "the bytes/sha256 below are over the LITERAL frozen string, before those are expanded"
  echo "rows: R4/a=$EXPECT_R4A claims, R4/b=$EXPECT_R4B claim, R5=$EXPECT_R5 claims; expected verdict: PASS for all 11"
  echo "no mutation leg in any row; R6 is split into its own HELD packet (live-store read, gated)"
  for c in CMD_CLONE_R4 CMD_FETCH_R4 CMD_CHECKOUT_R4 CMD_BUILD_R4 CMD_LEG_R4A CMD_LEG_R4B CMD_CLONE_R5 CMD_FETCH_R5 CMD_CHECKOUT_R5 CMD_BUILD_R5 CMD_LEG_R5; do
    v="${!c}"
    printf 'cmd %s bytes=%s sha256=%s\n  %s\n' "$c" "$(bytes_of "$v")" "$(sha_of "$v")" "$v"
  done
  exit 0
fi

# ---8< FUNCTIONS BEGIN ---
# (acceptance_r4_r5_selfcheck_v2.sh extracts exactly this region and tests it.)
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

# The raw memory evidence is read with a bare `cat`, so an absent file is silent: this is the only
# reader of the boundary set. Every boundary this packet defines must be present, and every
# boundary it does NOT define (including any mutation leg and anything R6-shaped) must be absent —
# a packet that demanded a leg it never defines could only ever exit 2, and a packet that silently
# accepted one would be running something the grant did not authorize.
raw_verify() {
  local miss="" b
  local need=(start r4_clone_fetch_after r4_build_before r4_build_after leg_R4a_before leg_R4a_after leg_R4b_before leg_R4b_after r5_clone_fetch_after r5_build_before r5_build_after leg_R5_before leg_R5_after)
  for b in r6_clone_fetch_after r6_build_before leg_R6_before leg_R6_after mutation r4a_mutation r4b_mutation r5_mutation; do
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

# A leg whose log is not ONE BARE VERDICT PER CLAIM is NOT AN OBSERVATION. A bare FAIL is a
# falsified grant; anything else exits 2 with the offending line rather than reporting a verdict
# the leg never stated.
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

# THE SCOPE GATE (the launcher convention, end-anchored): /proc/self/cgroup must be exactly one
# "0::" line, must not be the root cgroup, and its path must END in "/<JOB>.scope" with
# <JOB> = basename(dirname(<output-dir>)) — the unit `systemd-run --user --scope --unit=<JOB>`
# creates. A substring test is not acceptable: it would accept a foreign scope that merely
# mentions the token, and then every memory reading in the packet would be somebody else's.
scope_ok() { # <cgroup-file-content> <job-token> -> prints OK or a reason
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

# THE RUN-DIR GATE: the run dir is the launcher's argument and nothing else; exactly the pinned
# clones live inside it; each clone's HEAD is the pin it was fetched at; and no second out dir
# appears under the booked <JOB> dir mid-run (a re-opened or duplicated evidence dir means two
# runs are writing one evidence tree, and nothing downstream of here can be attributed).
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
require_scope_gate() { # entry paths must be TRACKED in the checkout the leg reads
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
    # The capture writes `### memory.current (<path>)` followed by the indented value, so that is
    # what is parsed. v12's arm matched `^memory.current ` and `^memory.max (`, shapes its own
    # capture never emits, so its limit test could never fire; corrected here and tested.
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
# The dir gate bounds WHERE a leg runs; it does not bound WHAT it reads. A pinned checkout bounds
# HEAD, not the bytes a leg opens. So the bytes are gated at the source: each entry file's blob id
# at the pinned HEAD against the constant this packet freezes. The uncheckable half is named rather
# than papered over: claim_batch derives a closure's transitive module set at run time from the
# source roots, so an entry-file blob gate is not a gate on the whole closure.
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

command -v timeout >/dev/null || { echo "timeout(1) missing; every leg is unbounded" >&2; exit 2; }
ulimit -c 0

# THE RUN DIR: the launcher's argument, created exclusively. It is NOT created under an assumed
# instance root and never reused. -p would silently accept a directory someone else made.
JOB=$(basename "$(dirname "$RUN_DIR")" 2>/dev/null || echo "")
[ -n "$JOB" ] || { echo "cannot derive the booked job token from $RUN_DIR" >&2; exit 2; }
[ "$(basename "$RUN_DIR")" = out ] || { echo "run dir $RUN_DIR does not end in /out (the launcher passes \$HOME/runs/<JOB>/out)" >&2; exit 2; }
mkdir -- "$RUN_DIR" 2>/dev/null || { echo "run dir $RUN_DIR already exists (or its job dir is missing); a run dir is never reused" >&2; exit 3; }
chmod 700 "$RUN_DIR"
mkdir -- "$RUN_DIR/raw" 2>/dev/null || { echo "cannot create $RUN_DIR/raw" >&2; exit 2; }
mkdir -- "$RUN_DIR/logs" "$RUN_DIR/tmp" 2>/dev/null || { echo "cannot create $RUN_DIR/logs or $RUN_DIR/tmp" >&2; exit 2; }
chmod 700 "$RUN_DIR/raw" "$RUN_DIR/logs" "$RUN_DIR/tmp"
# TMPDIR is pinned INSIDE the run dir before any clone: claim_batch runs `git show`, path-form
# extdeps probes and `cargo --version` through the shell, and git writes index.lock / tmp_pack
# files. With TMPDIR pinned, every child of every leg keeps its scratch inside the evidence tree.
export TMPDIR="$RUN_DIR/tmp"
export GUNBC_RUN_DIR="$RUN_DIR"
echo "run_dir=$RUN_DIR job_token=$JOB"

# THE SCOPE GATE, before any fetch: this process must be inside the booked <JOB>.scope.
CGRAW=$(cat /proc/self/cgroup 2>/dev/null) || { echo "cannot read /proc/self/cgroup" >&2; exit 3; }
S=$(scope_ok "$CGRAW" "$JOB")
[ "$S" = OK ] || { echo "SCOPE GATE: $S (raw=$(printf '%s' "$CGRAW" | tr '\n' '|'))" >&2; exit 3; }
echo "scope ok cgroup=${CGRAW#0::} job_token=$JOB hint=$JOB_HINT"

ev_capture start
eval "$CMD_CLONE_R4"
eval "$CMD_FETCH_R4"
eval "$CMD_CHECKOUT_R4"
ev_capture r4_clone_fetch_after
require_dir_gate "$RUN_DIR/wt-r4" "$PIN_R4" 2
require_scope_gate "$RUN_DIR/wt-r4" "$ENTRY_R4_STAGE0" "$ENTRY_R4_CRATE"
require_bound_gate
require_blob_gate "$RUN_DIR/wt-r4" "$ENTRY_R4_STAGE0:$BLOB_R4_STAGE0" "$ENTRY_R4_CRATE:$BLOB_R4_CRATE"
# A cold release build of the claim runner is minutes with 32 jobs, and this packet builds it TWICE
# (once per pin, because the runner is Rust and the pins differ). Each build gets its own timeout,
# its own log inside the run dir, and its own memory boundaries — a row is never blamed for a build
# that died, and the build's memory is in the record rather than in the row's line.
cd "$RUN_DIR/wt-r4"
ev_capture r4_build_before
timeout --kill-after=30 "$T_BUILD" bash -c "$CMD_BUILD_R4" > "$RUN_DIR/logs/build-r4.log" 2>&1 \
  || { tail -10 "$RUN_DIR/logs/build-r4.log" >&2; exit 1; }
ev_capture r4_build_after
BIN="$RUN_DIR/wt-r4/target/release/claim_batch"
[ -x "$BIN" ] || { echo "built runner not found at $BIN" >&2; exit 1; }
mem_events "$RUN_DIR/raw/mem.events"
require_bound_gate

# R4, leg A — five claims over the stage0 partition-rebuild scope witness. One attempt: a leg that
# fails to run at all ends the run; nothing here retries.
ev_capture leg_R4a_before
if ! timeout --kill-after=30 "$T_ROW" bash -c "$CMD_LEG_R4A" > "$RUN_DIR/raw/leg_R4a.out" 2>"$RUN_DIR/logs/r4a.log"; then
  tail -12 "$RUN_DIR/logs/r4a.log" >&2
  echo "R4 leg a (seed-only claims) FAILED to run at all; nothing observed; one attempt, no re-run" >&2
  exit 1
fi
cp "$RUN_DIR/raw/leg_R4a.out" "$RUN_DIR/raw/ev-leg_R4a_after.raw"
ev_capture leg_R4a_after
classify "$RUN_DIR/raw/ev-leg_R4a_after.raw" "$EXPECT_R4A" | sed "s|^|R4/a: |"
require_dir_gate "$RUN_DIR/wt-r4" "$PIN_R4" 2
require_scope_gate "$RUN_DIR/wt-r4" "$ENTRY_R4_STAGE0" "$ENTRY_R4_CRATE"
require_bound_gate

# R4, leg B — the one claim the grant places in a SEPARATE entry file: its own --entry, because a
# leg takes one entry.
ev_capture leg_R4b_before
if ! timeout --kill-after=30 "$T_ROW" bash -c "$CMD_LEG_R4B" > "$RUN_DIR/raw/leg_R4b.out" 2>"$RUN_DIR/logs/r4b.log"; then
  tail -12 "$RUN_DIR/logs/r4b.log" >&2
  echo "R4 leg b (seed-only claims) FAILED to run at all; nothing observed; one attempt, no re-run" >&2
  exit 1
fi
cp "$RUN_DIR/raw/leg_R4b.out" "$RUN_DIR/raw/ev-leg_R4b_after.raw"
ev_capture leg_R4b_after
classify "$RUN_DIR/raw/ev-leg_R4b_after.raw" "$EXPECT_R4B" | sed "s|^|R4/b: |"
require_dir_gate "$RUN_DIR/wt-r4" "$PIN_R4" 2
require_scope_gate "$RUN_DIR/wt-r4" "$ENTRY_R4_STAGE0" "$ENTRY_R4_CRATE"
require_bound_gate

# R5 — a SECOND pinned clone at the #15936 squash, and its own build of the runner.
eval "$CMD_CLONE_R5"
eval "$CMD_FETCH_R5"
eval "$CMD_CHECKOUT_R5"
ev_capture r5_clone_fetch_after
require_dir_gate "$RUN_DIR/wt-r5" "$PIN_R5" 2
require_scope_gate "$RUN_DIR/wt-r5" "$ENTRY_R5_INBOX"
require_bound_gate
require_blob_gate "$RUN_DIR/wt-r5" "$ENTRY_R5_INBOX:$BLOB_R5_INBOX"
cd "$RUN_DIR/wt-r5"
ev_capture r5_build_before
timeout --kill-after=30 "$T_BUILD" bash -c "$CMD_BUILD_R5" > "$RUN_DIR/logs/build-r5.log" 2>&1 \
  || { tail -10 "$RUN_DIR/logs/build-r5.log" >&2; exit 1; }
ev_capture r5_build_after
BIN5="$RUN_DIR/wt-r5/target/release/claim_batch"
[ -x "$BIN5" ] || { echo "built runner not found at $BIN5" >&2; exit 1; }
mem_events "$RUN_DIR/raw/mem.events"
require_bound_gate

ev_capture leg_R5_before
if ! timeout --kill-after=30 "$T_ROW" bash -c "$CMD_LEG_R5" > "$RUN_DIR/raw/leg_R5.out" 2>"$RUN_DIR/logs/r5.log"; then
  tail -12 "$RUN_DIR/logs/r5.log" >&2
  echo "R5 (seed-only claims) FAILED to run at all; nothing observed; one attempt, no re-run" >&2
  exit 1
fi
cp "$RUN_DIR/raw/leg_R5.out" "$RUN_DIR/raw/ev-leg_R5_after.raw"
ev_capture leg_R5_after
classify "$RUN_DIR/raw/ev-leg_R5_after.raw" "$EXPECT_R5" | sed "s|^|R5: |"
raw_verify
require_dir_gate "$RUN_DIR/wt-r5" "$PIN_R5" 2
require_scope_gate "$RUN_DIR/wt-r5" "$ENTRY_R5_INBOX"
require_bound_gate

# THE FINAL GATE is the last thing this packet does before reporting, so "no mutation leg" is not a
# promise about a window the packet never left open.
echo "R4=$([ "$(grep -c '^PASS ' "$RUN_DIR/raw/ev-leg_R4a_after.raw")" = "$EXPECT_R4A" ] && [ "$(grep -c '^PASS ' "$RUN_DIR/raw/ev-leg_R4b_after.raw")" = "$EXPECT_R4B" ] && echo PASS || echo FAIL) claims=$((EXPECT_R4A + EXPECT_R4B)) legs=2"
echo "R5=$([ "$(grep -c '^PASS ' "$RUN_DIR/raw/ev-leg_R5_after.raw")" = "$EXPECT_R5" ] && echo PASS || echo FAIL) claims=$EXPECT_R5 legs=1"
echo "R6=NOT_RUN_SPLIT_OUT (live-store read; gated; see the block below)"

# =====================================================================
# R6, SPLIT OUT — THE AUDIT, STATED RATHER THAN ASSUMED
#
# Read path, in order:
#   roadmap_acceptance_inbox_on_authority        dag/gunbc/roadmap/roadmap_acceptance_inbox.dag:140
#     -> roadmap_acceptance_event_history_load   dag/gunbc/roadmap/roadmap_authority.dag:5070
#        -> fabric_state_storage_binding         dag/gunbc/fabric/fabric_storage_binding.dag:36
#           -> roadmap_operator_close_history    dag/gunbc/roadmap/roadmap_operator_acceptance.dag:644
#              -> roadmap_operator_acceptance_history_load  same file:197
#                 -> roadmap_operator_acceptance_read       same file:132
#                    -> event_log_read_partition_with  dag/gunbc/fabric/fabric_event_log.dag:164
#                       -> fabric_storage_closure         dag/gunbc/fabric/fabric_storage_client.dag:193
#
# (1) WRITES: none in this closure. The append primitive (event_log_append_with) sits only in
#     roadmap_operator_acceptance_close / _close_append, which R6 never reaches. R6's closure is
#     read_partition, the object closure walk, and an in-memory replay.
# (2) WHICH STORE BRANCH A BARE srv1 CHECKOUT BINDS: not the seed.
#     `data prod_role_selection: ProdRoleSelection = ProdRoleHeldBy { deployment:
#     srv1_daily_workspace_deployment }` is a committed data constant (dag/gunbc/deployment_risk.dag:71)
#     and srv1_live_dashboard_instance carries exactly that deployment
#     (dag/gunbc/roadmap/roadmap_dashboard_instance.dag:1262-1264). So the prod role RESOLVES to a
#     built-in instance, fabric_storage_placement returns FabricStoragePlaced, and
#     fabric_state_storage_binding maps EVERY placed realization (including the host's own placed
#     branch, whose door_socket it discards) to FabricStorageServed { <instance endpoint>/state }.
#     FabricStorageServed takes served_post — a live network round trip
#     (dag/gunbc/fabric/fabric_storage_client.dag:203-207). That is the live partition: not the
#     seeded carrier, not a local door socket. The SEED is read only under FabricStorageBindingUnplaced
#     (the runtime identity store missing) — which is not the state of a rostered srv1 instance; and a
#     REFUSED placement yields HistoryLoadRefused -> AcceptanceInboxRefused, so the claim's InboxLoaded
#     arm fails: a FAIL, not a seed read. R6 cannot be run seed-only as written without editing the
#     claim or the binding, which no grant here authorizes.
# THEREFORE R6 ships as its own frozen packet (acceptance_r6_live_read_claim_run_v2.sh), HELD: a
# claim runner reading the live acceptance partition is a NEW reader of the live store, and
# keen-otter's msg_3407a188 admits an EXISTING rostered reader only. Its --execute alone exits 3
# with this reason; it runs only when an execution grant that has cleared that gate is named on the
# command line, and that reference is recorded in the run dir.
# =====================================================================
