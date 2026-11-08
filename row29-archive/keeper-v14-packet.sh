#!/usr/bin/env bash
# row29_claim_packet.sh  (v14 — PRE-STAGED, UNFROZEN pending keen-otter go-ahead — MINOR disposition of sleek-owl v13 review on top of v13 (9e81076f): (A) correct PIPE wording (EXIT trap may print on SIGPIPE but rc 141 stays NO_OBSERVATION); (B) self_test() REFUSES inside the booked launcher scope, closing the latent live---run hazard introduced by the v13 H5 pre-gate control, and correct the stale L500 "never spawns a live --run" claim; (C) document the argv-reachability of the test-only arms. No production change; pins the FIXED import-fix commit 483fa8db.)
# Objective: at a frozen pin, build claim_batch and evaluate the FIVE literal witness claims of
#   dag/test/claim/live_deploy/user_work_slice_admission_witness.dag
# over the shipped user_work_slice_admit_against / std.measure measure_fits rule; decide whether each
# returns PASS. Establishes RULE BEHAVIOR ON THE CLAIMS' INPUTS ONLY (see LIMITATIONS). NOT live srv1
# placement, NOT before-launch wiring, NOT the live MemoryMax/MemoryCurrent parse route (the shipped
# actuator belt_user_work_slice_fit reads the LIVE 44 GiB MemoryMax and its parsers are NOT exercised
# here), NOT compute_reserve_against's Below path (declared unmet obligation), NOT complete rewire.
#
# Frame: keeper calmcrab-udrv v4, POSITIONAL argv, srv1 one-shot transient user scope under
# user@1000.service/app.slice (a DIFFERENT cgroup subtree from gunbc-user-work.slice). Hermetic claim
# context. NO live-system inspection/mutation of the running lab, NO host_effect dispatch, NO unit/slice
# admission edit, NO kill, NO counter reset. Data-only.
#
# Effects (declared): network (GitHub fetch + crates.io during build), disk (OUT + shared CARGO_HOME
# cache OUTSIDE OUT), CPU/mem under the booked scope. None of these are effect-free on srv1.
#
# Booking: NEVER-GRANTED proposal under the operator's budget rule. RT/MX/MH/TK/CPU below are PROPOSED
# bounds, not inherited authorization; a named operator grant is required to run. TASKS is proposed at
# 4096 (see rationale at TASKS); this is part of the ungranted proposal, not an automatic raise.
#
# Usage (launcher runs the frozen staged script by sha):  row29_claim_packet.sh --run <OUT>
#   RUN mode is reached ONLY via the explicit --run token (a structural guard makes any non-run abort
#   unable to enter run code). <OUT> is argv 2, MUST be absolute, MUST end with the booked /<JOB>/out,
#   and is created EXCLUSIVELY (plain mkdir, no -p) ONLY AFTER the scope gate and booked-bounds readback
#   pass. Unknown/missing modes and any non-run abort refuse rc 2 with NO run-side effect (no directory).
#   Reviewer-local audit, no build/host/network:
#     bash row29_claim_packet.sh --self-test       # gates + abort/mode/real-caller controls
#     bash row29_claim_packet.sh --limitations
#     bash row29_claim_packet.sh --verify-captured <captured-file> <producer-rc>   # re-score a capture
#
# EXIT CONTRACT: the receipt (one PACKET_VERDICT block) is written EXACTLY ONCE, by emit_receipt after a
#   complete classify, or by the EXIT trap on any abort -- NEVER by classify itself. rc ALONE never
#   determines a verdict; the keeper MUST match rc to the single PACKET_VERDICT line (and, for a
#   mismatch, its 'bare-FAIL rows=' line) per the COLLECTION RULE in the entry section. Anything
#   unmatched (rc 2, 128+n, 141, duplicate/absent receipt, SIGKILL, session loss) is NO_OBSERVATION.
#   Signals: TERM/HUP/INT are trapped -> exit 128+n -> the EXIT trap emits exactly ONE NO_OBSERVATION and
#   forces rc 2. PIPE and QUIT are UNTRAPPED BY DESIGN: PIPE exits rc 141 (128+13); bash's EXIT trap may
#   still PRINT a block if stdout is still writable, but that block is NOT a valid receipt and the
#   COLLECTION RULE maps rc 141 to NO_OBSERVATION regardless (fails closed); bash IGNORES QUIT. SIGKILL and session
#   loss are untrappable and are covered ONLY by the COLLECTION RULE, never by a receipt.
#   0 = OBSERVED_MATCH     (five bare PASS rows + a single well-formed completion line + producer rc 0)
#   1 = OBSERVED_MISMATCH  (>=1 BARE 'FAIL <fn>' row, no unobserved/reasoned rows, producer rc 1)
#   2 = NO_OBSERVATION     (join cannot form: missing/duplicate/extra/reasoned-FAIL/UNOBSERVED/PANICKED
#                           row, absent/malformed/duplicate completion line, a producer-rc/verdict
#                           contradiction, an out-of-contract rc, a gate refusal, closure/resolve
#                           refusal, a scope/bounds/OOM refusal, or a captured file exceeding CAP).
#                           NO truncated success is ever accepted.
# Layer 2 (never scored as a red, never attributed to the PR absent evidence): raw producer rc and any
#   closure-diagnostics/NotBool/panic/runtime text are echoed as DIAGNOSTIC lines only.

set -u
set -o pipefail
export LC_ALL=C

# ============================ frozen identities (hard-set, readonly; NO env override) ============================
readonly REPO_URL='https://github.com/gunb-ai/gunbc'
readonly JOB='row29-16013-483fa8db'
readonly PIN_HEAD='483fa8db3d5e4bc7d560072046dcfda53b469f89'
readonly PIN_TREE='4cc7c265197285218c3ef61e7344e56fd11acb8d'
readonly BLOB_WITNESS='9e3ebe6c34a1c1eab19a2361aeb658b6a42edc04'
readonly BLOB_MEASURE='b3489866e8d1b72ca87c3cfb8217daeb7a7f2e35'
readonly BLOB_HOSTCAP='c411aa42ea19c0eae3df04ab075cd5c5fee4f290'
readonly BLOB_SLICE='20502bf36b9ca7cce1808881730903a36940538f'
readonly BLOB_BELT='210bf8278002f514feb4c7508c4d14ea262533e3'
readonly BLOB_CONCERN='81a9fe09c9c28e68a35aa46ea02aa15ab1989804'
readonly ENTRY_PATH='dag/test/claim/live_deploy/user_work_slice_admission_witness.dag'
readonly PKG='v1-compiler'          # src/v1/stage0/Cargo.toml [package] name (NOT v1_stage0)
readonly BIN_REL='target/release/claim_batch'
readonly BUILD_JOBS=32
readonly CAP_BYTES=1048576          # 1 MiB hard cap on the captured combined file (refuse-on-exceed)
readonly FUNCS='srv1_shape_refuses_the_goal_audit_reviewer_before_launch a_review_that_fits_is_admitted_not_refused resident_above_the_ceiling_is_a_refusal_with_nothing_available a_zero_ceiling_is_a_finite_refusal_not_unbounded the_decision_turns_on_the_unit_bound_not_a_busy_stall'

# ---- proposed (NEVER-GRANTED) envelope bounds; the launcher passes these positionally. The booked
# ---- values the script verifies against the scope's OWN cgroup MUST equal the argv EXACTLY.
readonly RT_S=3600                  # RUNTIME_S capability; script deadline enforced at RT_S-300 = 3300s
readonly MEMMAX_G=24                # scope MemoryMax (integer GiB)  -> memory.max bytes
readonly MEMHIGH_G=20               # scope MemoryHigh (integer GiB) -> memory.high bytes
readonly TASKS=4096                 # pids.max. RATIONALE: -j 32 cargo builds run up to 32 rustc
                                    # processes plus their LLVM codegen threads plus build scripts, and
                                    # all count against pids.max; 256 risks fork/thread EAGAIN and a
                                    # spent attempt. 4096 is the value every prior srv1 run of this
                                    # frame used without hitting pids. PROPOSED, part of the never-
                                    # granted booking; NOT an automatic raise absent operator sign-off.
readonly CPUPCT=3200                # 32 cores -> cpu.max "3200000 100000"
readonly MEMMAX_BYTES=25769803776   # 24*1024^3 (booked, compared literally)
readonly MEMHIGH_BYTES=21474836480  # 20*1024^3 (booked)
readonly SWAPMAX='0'                # memory.swap.max booked
readonly CPUMAX='3200000 100000'    # cpu.max booked (CPUPCT 3200% * 100000 / 100, period 100000)
# Independence guard: CPUMAX must equal the launcher's cpu.max formula, so a transcription error in
# the frozen literal cannot silently reach a run (it would only be caught at bounds_check, spending
# the attempt). Launcher v4 computes cpu.max as "$((Q*1000)) 100000" with Q=CPUPCT; assert against it.
[ "$CPUMAX" = "$((CPUPCT*1000)) 100000" ] || { echo "PACKET_VERDICT=NO_OBSERVATION"; echo "REASON=booked CPUMAX literal inconsistent with CPUPCT"; exit 2; }
# Independence guards for the memory byte literals: each booked byte value must equal GiB*1024^3 of its
# booked GiB count, so a transcription error in MEMMAX_BYTES/MEMHIGH_BYTES cannot silently reach a run.
[ "$MEMMAX_BYTES"  = "$((MEMMAX_G*1073741824))"  ] || { echo "PACKET_VERDICT=NO_OBSERVATION"; echo "REASON=booked MEMMAX_BYTES literal inconsistent with MEMMAX_G"; exit 2; }
[ "$MEMHIGH_BYTES" = "$((MEMHIGH_G*1073741824))" ] || { echo "PACKET_VERDICT=NO_OBSERVATION"; echo "REASON=booked MEMHIGH_BYTES literal inconsistent with MEMHIGH_G"; exit 2; }
# swap must be booked at EXACTLY 0 (a nonzero SWAPMAX would let an OOM spill to swap and defeat the memory
# gate; it is also a transcription hazard). Independent literal, checked on every invocation including --run.
[ "$SWAPMAX" = 0 ] || { echo "PACKET_VERDICT=NO_OBSERVATION"; echo "REASON=booked SWAPMAX literal not 0"; exit 2; }

# ---- phase admission (full fixed caps + kill grace + terminal allowance; refuse BEFORE starting
# ---- rather than clip). Coherent total under DEADLINE_S; RT is NOT silently raised.
readonly KILL_GRACE=30
readonly FETCH_CAP=300
readonly BUILD_CAP=1800
readonly RUN_CAP=600
readonly FINAL_ALLOW=200            # reserve for classify/reporting (no producer, bounded)
# worst-case total = (300+30)+(1800+30)+(600+30)+200 = 2990 <= DEADLINE_S 3300. Slack 310s.
readonly DEADLINE_S=$((RT_S - 300))

# ---- REAL completion lines, verbatim from calm-crab-645's #15997 A2 retained log (sha-gated so the
# ---- fixture is genuine bytes, not invented). cost-partition is JSON; the join never expects a literal
# ---- 'Reconciled' in stdout.
readonly REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
readonly REAL_COMPLETION_SHA='7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2'

V_MATCH=OBSERVED_MATCH; V_MISMATCH=OBSERVED_MISMATCH; V_NO=NO_OBSERVATION

# ============================ helpers ============================
# die2 is DEFINED below with the single-receipt machinery (entry section); every refusal routes through emit_receipt.
diag() { echo "DIAG=$*"; }
hash_of() { local o h; o="$(sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }

# completed FILE -> 0 iff EXACTLY one '[process-partition] total_ms=<digits>' line (the real shape).
completed() { local n; [ -r "$1" ] || return 1
  n="$(grep -acE '^\[process-partition\] total_ms=[0-9]+([[:space:]]|$)' -- "$1" 2>/dev/null)" || return 1
  [ "$n" = "1" ]; }
# cost_ok FILE -> 0 iff >=1 '[cost-partition] {..JSON..}' line (JSON shape; never a bare Reconciled).
cost_ok() { local n; [ -r "$1" ] || return 1
  n="$(grep -acE '^\[cost-partition\] \{.*"basis".*\}' -- "$1" 2>/dev/null)" || return 1
  [ "$n" -ge 1 ]; }

# check_cap FILE -> 0 iff readable, wc succeeds, and size <= CAP_BYTES. A read failure REFUSES (returns 1);
# it never echoes 0. This is the counter-read-failure-refuses law.
check_cap() { local b; [ -r "$1" ] || return 1
  b="$(wc -c < "$1" 2>/dev/null)" || return 1
  [[ $b =~ ^[0-9]+$ ]] || return 1
  [ "$b" -le "$CAP_BYTES" ]; }

tree_clean() { local o; o="$(git -C "$1" status --porcelain 2>/dev/null)" || return 1; [ -z "$o" ]; }

# roster_gate FILE -> 0 iff the 'test fn' declaration ORDER equals FUNCS exactly (literal compare).
roster_gate() { local f="$1" i=0 want got exp
  local -a exp_arr; read -r -a exp_arr <<<"$FUNCS"
  [ "${#exp_arr[@]}" -gt 0 ] || return 1
  while IFS= read -r line; do
    want="${exp_arr[$i]:-__NONE__}"
    [ "$want" != "__NONE__" ] || return 1
    got="$(printf '%s\n' "$line" | sed -nE 's/^[[:space:]]*test[[:space:]]+fn[[:space:]]+([A-Za-z0-9_]+).*/\1/p')"
    [ "$got" = "$want" ] || return 1
    i=$((i+1))
  done < <(grep -E '^[[:space:]]*test[[:space:]]+fn[[:space:]]+[A-Za-z0-9_]+' -- "$f" 2>/dev/null)
  [ "$i" = "${#exp_arr[@]}" ]; }

# ---- scope gate: read the CGROUP FILE (default /proc/self/cgroup), require EXACTLY one '0::<path>'
# ---- line whose path ends with the LITERAL "/<JOB>.scope" (literal suffix removal, not regex). Any
# ---- unreadable file, non-single line, non-'0::', or non-scope suffix REFUSES. Echoes the canonical
# ---- path so the SAME path is reused for all booked readbacks and OOM snapshots.
scope_gate() { local cfile="${1:-}" lines path rest; [ -n "$cfile" ] || { diag "scope-gate: no path supplied (production passes /proc/self/cgroup literally; self-test passes a fixture path) -> refuse"; return 1; }
  [ -r "$cfile" ] || { diag "scope-gate: $cfile unreadable -> refuse"; return 1; }
  lines="$(grep -c '' -- "$cfile" 2>/dev/null)" || return 1
  [ "$lines" = "1" ] || { diag "scope-gate: expected exactly 1 cgroup line, got $lines -> refuse"; return 1; }
  local line; IFS= read -r line < "$cfile" || return 1
  case "$line" in 0::*) ;; *) diag "scope-gate: line is not 0::<path> -> refuse"; return 1;; esac
  path="${line#0::}"
  [ -n "$path" ] || { diag "scope-gate: empty path -> refuse"; return 1; }
  rest="${path%"${JOB}.scope"}"                     # literal suffix removal (quoted pattern = literal)
  [ "$rest" != "$path" ] && [ -n "$rest" ] || { diag "scope-gate: path does not end in /$JOB.scope -> refuse"; return 1; }
  case "$path" in *"/$JOB.scope") ;; *) diag "scope-gate: suffix not /$JOB.scope -> refuse"; return 1;; esac
  printf '%s' "$path"; }

# ---- bounds_check CGDIR: read the 5 booked files from the scope's OWN cgroup and compare to the booked
# ---- literals EXACTLY. Any missing file, failed read, empty, or mismatch REFUSES (exit 2 in run mode).
bounds_check() { local d="$1" got
  local -A want=(
    [memory.max]="$MEMMAX_BYTES" [memory.high]="$MEMHIGH_BYTES"
    [memory.swap.max]="$SWAPMAX" [pids.max]="$TASKS" [cpu.max]="$CPUMAX" )
  local f exp
  for f in memory.max memory.high memory.swap.max pids.max cpu.max; do
    exp="${want[$f]}"
    [ -r "$d/$f" ] || { diag "bounds: $d/$f unreadable -> refuse"; return 1; }
    got="$(cat "$d/$f" 2>/dev/null)" || { diag "bounds: $d/$f read failed -> refuse"; return 1; }
    [ -n "$got" ] || { diag "bounds: $d/$f empty -> refuse"; return 1; }
    [ "$got" = "$exp" ] || { diag "bounds: $d/$f='$got' != booked '$exp' -> refuse"; return 1; }
  done; }

# ---- OOM counter snapshot/judge: read memory.events, extract 'oom' and 'oom_kill' as a pair. Any read
# ---- failure or non-numeric counter is incoherent -> refuse. oom_assert compares a BEFORE pair to the
# ---- current pair; any rise in oom or oom_kill REFUSES, regardless of producer rc.
oom_read() { local d="$1" line o ok
  [ -r "$d/memory.events" ] || return 1
  line="$(grep -E '^(oom|oom_kill) ' "$d/memory.events" 2>/dev/null)" || return 1
  o="$(printf '%s\n' "$line" | awk '$1=="oom"{print $2}')"
  ok="$(printf '%s\n' "$line" | awk '$1=="oom_kill"{print $2}')"
  [[ $o =~ ^[0-9]+$ ]] || return 1
  [[ $ok =~ ^[0-9]+$ ]] || return 1
  printf '%s %s' "$o" "$ok"; }
oom_assert() { # $1=before_pair $2=cgdir $3=phase-name
  local -a b; read -r -a b <<<"$1"
  local cur; cur="$(oom_read "$2")" || { diag "OOM: $3 read failed/incoherent -> refuse"; return 1; }
  local -a c; read -r -a c <<<"$cur"
  [ "${#b[@]}" = 2 ] && [ "${#c[@]}" = 2 ] || { diag "OOM: $3 incoherent counters -> refuse"; return 1; }
  if [ "${c[0]}" -gt "${b[0]}" ] || [ "${c[1]}" -gt "${b[1]}" ]; then
    diag "OOM: $3 oom ${b[0]}->${c[0]} oom_kill ${b[1]}->${c[1]} -> refuse"; return 1; fi; }
mem_snapshot() { local d="$1" tag="$2" cur pk
  cur="$(cat "$d/memory.current" 2>/dev/null || echo NA)"
  pk="$(cat "$d/memory.peak" 2>/dev/null || echo NA)"
  diag "mem[$tag] current=$cur peak=$pk"; }

# ---- classify L RC -> echo verdict, return its exit code (0/1/2). VERDICT JOIN reads ONLY anchored
# ---- producer outcome rows ('^<KW> <fn>$' with exact 2-token boundaries, literal name equality via
# ---- awk '$1==KW && $2==fn && NF==2'); per-claim '[witness]' telemetry, JSON cost-partition and the
# ---- process-partition line are NOT verdicts and CANNOT invalidate an otherwise complete join.
classify() {
  # classify L RC -> builds the COMPLETE receipt into global RPT and returns its exit code (0/1/2).
  # It NEVER prints: the single write happens only in emit_receipt after classify returns cleanly, so a
  # mid-classification abort can leak no partial receipt and a mismatch is never claimed before the full
  # join holds. VERDICT JOIN reads ONLY anchored producer outcome rows ('^<KW> <fn>$' with exact 2-token
  # boundaries, literal name equality via awk '$1==KW && $2==fn && NF==2'); per-claim '[witness]'
  # telemetry, JSON cost-partition and the process-partition line are NOT verdicts and CANNOT invalidate
  # an otherwise complete join.
  local L="$1" RC="$2"
  RPT=""
  if ! check_cap "$L"; then rpt "$V_NO" "REASON=captured file missing/unreadable or exceeds CAP (refuse-on-read-failure)"; return 2; fi
  if ! completed "$L"; then rpt "$V_NO" "REASON=completion line absent/duplicate/malformed"; return 2; fi
  if ! cost_ok "$L"; then rpt "$V_NO" "REASON=JSON cost-partition line absent"; return 2; fi

  local -a F; read -r -a F <<<"$FUNCS"
  local total=0
  local -a verdicts=()
  local fn npass nfailing nreason nnone
  for fn in "${F[@]}"; do
    npass="$(awk -v n="$fn" 'index($0,"PASS "n)==1 && $1=="PASS" && $2==n && NF==2' "$L" 2>/dev/null | grep -c '' || true)"
    nfailing="$(awk -v n="$fn" 'index($0,"FAIL "n)==1 && $1=="FAIL" && $2==n && NF==2' "$L" 2>/dev/null | grep -c '' || true)"
    nreason="$(awk -v n="$fn" 'index($0,"FAIL "n)==1 && $1=="FAIL" && $2==n && NF>2' "$L" 2>/dev/null | grep -c '' || true)"
    nnone="$(awk -v n="$fn" '(($1=="UNOBSERVED"||$1=="PANICKED") && $2==n) && index($0,$1" "n)==1' "$L" 2>/dev/null | grep -c '' || true)"
    local cnt=$((npass+nfailing))
    if [ "$nnone" -gt 0 ] || [ "$nreason" -gt 0 ]; then rpt "$V_NO" "REASON=fn $fn has UNOBSERVED/PANICKED or reasoned-FAIL row"; return 2; fi
    if [ "$cnt" -ne 1 ]; then rpt "$V_NO" "REASON=fn $fn has $cnt bare outcome rows (want exactly 1)"; return 2; fi
    if [ "$npass" = "1" ]; then verdicts+=("PASS"); else verdicts+=("FAIL"); fi
    total=$((total+1))
  done
  [ "$total" = "${#F[@]}" ] || { rpt "$V_NO" "REASON=row count != FUNCS count"; return 2; }

  local fcnt extra
  extra="$(awk '($1=="PASS"||$1=="FAIL"||$1=="UNOBSERVED"||$1=="PANICKED") && NF>=2 && index($0,$1" ")==1 {print $2}' "$L" 2>/dev/null \
    | sort -u | while read -r nm; do
        ok=0; for fn in "${F[@]}"; do [ "$nm" = "$fn" ] && ok=1; done; [ "$ok" = 0 ] && echo "$nm"; done | grep -c '' || true)"
  [ "$extra" = "0" ] || { rpt "$V_NO" "REASON=extra non-claim outcome name(s) in capture"; return 2; }

  local nP=0 nX=0 v; for v in "${verdicts[@]}"; do [ "$v" = PASS ] && nP=$((nP+1)) || nX=$((nX+1)); done
  if [ "$nX" = 0 ] && [ "$nP" = "${#F[@]}" ]; then
    if [ "$RC" = "0" ]; then rpt "$V_MATCH"; return 0;
      else rpt "$V_NO" "REASON=all-PASS rows but producer rc=$RC (contradiction)"; return 2; fi
  fi
  if [ "$nX" -ge 1 ]; then
    if [ "$RC" = "1" ]; then rpt "$V_MISMATCH" "bare-FAIL rows=$nX of ${#F[@]} (observed mismatch)"; return 1;
      else rpt "$V_NO" "REASON=bare-FAIL row(s) but producer rc=$RC (contradiction)"; return 2; fi
  fi
  rpt "$V_NO" "REASON=no verdicts formed"; return 2; }

LIMITATIONS() { cat <<'L'
LIMITATIONS (what a PASS does and does not establish):
- Establishes: rule behavior of user_work_slice_admit_against / std.measure measure_fits on the FIVE
  literal claim inputs at the frozen pin. Nothing else.
- Does NOT establish: live srv1 placement, the before-launch wiring of belt_review_spawn/belt_audit_spawn,
  the live MemoryMax/MemoryCurrent parse route (the shipped belt reads the live 44 GiB MemoryMax via
  parse_systemd_memory_limit_show/parse_cgroup_nonnegative_bytes — those parsers, incl. their Unbounded and
  Absent arms, are NOT exercised), or compute_reserve_against's Below path (declared unmet obligation).
- Witness ceiling is the SOURCE ruling 2 x 25 GiB = 50 GiB (user_work_slice_memory_max_of), NOT the host-
  enforced 44 GiB. A PASS is pure-fold-on-these-inputs only; no wet compute claim is made or authorized.
- Environment: the launcher's inherited environment (including any GUNBC_* variables and the build/claim
  environment) is UNKNOWN under this launcher and is NOT isolated in this revision; deliberately isolating
  it (cleared env + explicit PATH) would be a SEPARATELY REVIEWED change. No environment values are printed.
- PIN: frozen at the FIXED import-fix commit 483fa8db (tree 4cc7c265, slice blob 20502bf3). RUN mode fetches
  that commit, so it is fetchable/runnable ONLY AFTER 483fa8db is on origin (a separate named push decision).
  Rationale: the witness calls user_work_slice_admit_against in slice_bounds.dag, whose body calls
  measure_fits; at the PRE-fix f5755e6 that callee is unimported and the module fails effect-resolution
  (heal's identical error), so RUN mode there is necessarily NO_OBSERVATION. Pinning to the fixed tree is
  required for the fold to resolve and the five witness claims to be meaningful.
- RUN MODE ENTRY: the launcher's extra argv is now "--run <OUT>" (was "<OUT>"). RUN is reachable ONLY via
  the --run token; the launcher must pass exactly "--run <absolute-OUT-under-runs/<JOB>/out>". A mistyped
  mode, an unknown/missing mode, or an abort in --verify-captured/--self-test refuses rc 2 before any
  filesystem or network effect (asserted by --self-test abort/mode controls). The keeper's collection rule
  (see the entry COLLECTION RULE) is authoritative and independent of the script.
- PIN ACQUISITION: git fetch of 483fa8db from origin is currently "not our ref" (the commit is not yet
  pushed), so a real --run cannot acquire the tree until the named push decision lands. This blocks ONLY
  the proposed run's acquisition route; the exact commit/tree/blob identity is independently verifiable
  from the posted raw-commit and format-patch bytes (comments 6702291816 / 6702077436) WITHOUT fetching
  origin, which sleek-owl has done (git am replay -> 483fa8db / 4cc7c265 / 20502bf3, all EXACT).
L
}

# ============================ self-test (real-shaped fixtures; pure-bash; exits 2 on misfire) ============================
self_test() {
  local dir rc bad=0
  # v14 SAFETY guard (MINOR B): the behavioral pre-gate control (H5) spawns a live --run whose ONLY stop
  # before the real run preamble (fetch/build) is scope_gate(/proc/self/cgroup). So the self-test MUST NOT
  # run inside the booked launcher scope. Refuse outright, before any child is spawned, if the gate would
  # let a real run through (i.e. /proc/self/cgroup matches the booked scope). Defense-in-depth, exercised
  # only inside the booked scope; uses the SAME scope_gate the run preamble relies on, so it cannot
  # false-positive out-of-scope (multi-line/unreadable/non-0:: cgroups all make scope_gate refuse).
  if scope_gate /proc/self/cgroup >/dev/null 2>&1; then
    echo "SELF_TEST=FAIL reason=self-test-refused-inside-booked-scope"; return 2
  fi
  dir="$(mktemp -d)" || { echo SELF_TEST=FAIL; return 2; }
  # shellcheck disable=SC2064
  trap "rm -rf '$dir'" RETURN
  local CL; CL="$(printf '%s' "$REAL_COMPLETION" 2>/dev/null; true)"
  local csha; csha="$(printf '%s' "$REAL_COMPLETION" | sha256sum | awk '{print $1}')"
  [ "$csha" = "$REAL_COMPLETION_SHA" ] || { echo "SELF_TEST=FAIL reason=completion-fixture-sha"; return 2; }

  # a real-shaped telemetry line per claim (the thing that broke v4's join), JSON cost-partition, completion
  telem() { printf '[witness] %s: cpu=12 wall=34 eval_steps=56 mem=78\n' "$1"; }
  costline() { printf '[cost-partition] {"basis":"wallclock","verdict":{"state":"Reconciled","total_ms":340188}}\n'; }

  mk_allpass() { # 5 PASS + telemetry + cost + completion; $2=rc echo file, sets file $1
    local f="$1"; : > "$f"; local fn
    for fn in $FUNCS; do printf 'PASS %s\n' "$fn" >> "$f"; telem "$fn" >> "$f"; done
    costline >> "$f"; printf '%s\n' "$REAL_COMPLETION" >> "$f"; }
  mk_withfail() { # fn $2 becomes bare FAIL (rc1 expected)
    local f="$1" target="$2"; : > "$f"; local fn
    for fn in $FUNCS; do if [ "$fn" = "$target" ]; then printf 'FAIL %s\n' "$fn"; else printf 'PASS %s\n' "$fn"; fi >> "$f"; telem "$fn" >> "$f"; done
    costline >> "$f"; printf '%s\n' "$REAL_COMPLETION" >> "$f"; }

  expect() { # $1 label  $2 want-code  $3 file  $4 rc
    classify "$3" "$4" >/dev/null 2>&1; local got=$?
    if [ "$got" != "$2" ]; then echo "SELF_TEST=FAIL reason=$1 got=$got want=$2"; bad=1; fi; }

  # normal telemetry-positive all-PASS rc0 -> 0
  mk_allpass "$dir/ap"; expect allpass_telemetry 0 "$dir/ap" 0
  # one bare FAIL rc1 -> 1
  mk_withfail "$dir/f1" srv1_shape_refuses_the_goal_audit_reviewer_before_launch; expect barefail_rc1 1 "$dir/f1" 1
  # bare FAIL rc0 -> contradiction 2
  mk_withfail "$dir/f2" srv1_shape_refuses_the_goal_audit_reviewer_before_launch; expect barefail_rc0_contradiction 2 "$dir/f2" 0
  # bare FAIL rc2 -> contradiction 2
  mk_withfail "$dir/f3" srv1_shape_refuses_the_goal_audit_reviewer_before_launch; expect barefail_rc2_contradiction 2 "$dir/f3" 2
  # all PASS rc1 -> contradiction 2
  mk_allpass "$dir/ap2"; expect allpass_rc1_contradiction 2 "$dir/ap2" 1
  # reasoned FAIL (closure diagnostics) -> 2 (never a bare Boolean FAIL)
  mk_allpass "$dir/rf"; printf 'FAIL a_review_that_fits_is_admitted_not_refused (closure diagnostics in slice_bounds)\n' >> "$dir/rf"
  expect reasoned_fail 2 "$dir/rf" 1
  # UNOBSERVED row -> 2
  mk_allpass "$dir/un"; printf 'UNOBSERVED a_zero_ceiling_is_a_finite_refusal_not_unbounded\n' >> "$dir/un"
  expect unobserved 2 "$dir/un" 1
  # missing row (drop one PASS + its telemetry) -> 2
  : > "$dir/miss"; local fn first=1
  for fn in $FUNCS; do if [ "$first" = 1 ]; then first=0; continue; fi; printf 'PASS %s\n' "$fn" >> "$dir/miss"; telem "$fn" >> "$dir/miss"; done
  costline >> "$dir/miss"; printf '%s\n' "$REAL_COMPLETION" >> "$dir/miss"
  expect missing_row 2 "$dir/miss" 0
  # duplicate row -> 2
  mk_allpass "$dir/dup"; printf 'PASS the_decision_turns_on_the_unit_bound_not_a_busy_stall\n' >> "$dir/dup"
  expect duplicate_row 2 "$dir/dup" 0
  # extra non-claim name -> 2
  mk_allpass "$dir/extra"; printf 'PASS some_other_claim_not_in_roster\n' >> "$dir/extra"
  expect extra_name 2 "$dir/extra" 0
  # completion line absent -> 2
  mk_allpass "$dir/nc"; grep -vE '^\[process-partition\] total_ms=' "$dir/nc" > "$dir/nc2"; mv "$dir/nc2" "$dir/nc"
  expect no_completion 2 "$dir/nc" 0
  # completion line duplicate -> 2
  mk_allpass "$dir/dc"; printf '%s\n' "$REAL_COMPLETION" >> "$dir/dc"
  expect dup_completion 2 "$dir/dc" 0
  # cost-partition JSON absent -> 2
  mk_allpass "$dir/nco"; grep -vE '^\[cost-partition\] ' "$dir/nco" > "$dir/nco2"; mv "$dir/nco2" "$dir/nco"
  expect no_costline 2 "$dir/nco" 0
  # timeout rc124 with all PASS -> 2 (out of contract)
  mk_allpass "$dir/t124"; expect allpass_rc124 2 "$dir/t124" 124

  # ---- scope gate fixtures ----
  printf '0::/user.slice/user-1000.slice/user@1000.service/app.slice/%s.scope\n' "$JOB" > "$dir/cg_ok"
  scope_gate "$dir/cg_ok" >/dev/null 2>&1 || { echo "SELF_TEST=FAIL reason=scope_ok_should_pass"; bad=1; }
  printf '0::/user.slice/app.slice/%s.scope\n0::/other.slice/x\n' "$JOB" > "$dir/cg_two"; scope_gate "$dir/cg_two" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=scope_two_lines_should_refuse"; bad=1; }
  printf '12:name:/x/%s.scope\n' "$JOB" > "$dir/cg_bad"; scope_gate "$dir/cg_bad" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=scope_non_0_should_refuse"; bad=1; }
  printf '0::/user.slice/app.slice/wrong.scope\n' > "$dir/cg_wrong"; scope_gate "$dir/cg_wrong" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=scope_wrong_job_should_refuse"; bad=1; }
  scope_gate "$dir/does_not_exist" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=scope_unreadable_should_refuse"; bad=1; }

  # ---- bounds gate fixtures (real-caller: same code path, injected cgroup dir) ----
  mkdir -p "$dir/cgdir"; printf '%s' "$MEMMAX_BYTES" > "$dir/cgdir/memory.max"
  printf '%s' "$MEMHIGH_BYTES" > "$dir/cgdir/memory.high"; printf '%s' "$SWAPMAX" > "$dir/cgdir/memory.swap.max"
  printf '%s' "$TASKS" > "$dir/cgdir/pids.max"; printf '%s' "$CPUMAX" > "$dir/cgdir/cpu.max"
  bounds_check "$dir/cgdir" >/dev/null 2>&1 || { echo "SELF_TEST=FAIL reason=bounds_ok_should_pass"; bad=1; }
  printf '1' > "$dir/cgdir/pids.max"; bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_pids_mismatch_should_refuse"; bad=1; }
  printf '%s' "$TASKS" > "$dir/cgdir/pids.max"; rm -f "$dir/cgdir/cpu.max"; bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_missing_should_refuse"; bad=1; }
  # cpu.max must be VALIDATED by bounds_check (guards against bounds_check ignoring cpu.max / a wrong booked literal)
  printf '%s' "$CPUMAX" > "$dir/cgdir/cpu.max"
  printf '%s' "$((CPUPCT*1000+100000)) 100000" > "$dir/cgdir/cpu.max"; bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_cpu_mismatch_should_refuse"; bad=1; }
  printf '%s' "$CPUMAX" > "$dir/cgdir/cpu.max"
  # memory.max / memory.high / swap must be ENFORCED by bounds_check (not ignored): stub a sentinel != booked and require refuse, then restore.
  printf '%s' "$((MEMMAX_BYTES+1))" > "$dir/cgdir/memory.max";  bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_memmax_mismatch_should_refuse"; bad=1; }
  printf '%s' "$MEMMAX_BYTES" > "$dir/cgdir/memory.max"
  printf '%s' "$((MEMHIGH_BYTES+1))" > "$dir/cgdir/memory.high"; bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_memhigh_mismatch_should_refuse"; bad=1; }
  printf '%s' "$MEMHIGH_BYTES" > "$dir/cgdir/memory.high"
  printf 'max' > "$dir/cgdir/memory.swap.max"; bounds_check "$dir/cgdir" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=bounds_swap_mismatch_should_refuse"; bad=1; }
  printf '%s' "$SWAPMAX" > "$dir/cgdir/memory.swap.max"
  # independent hard-literal positive ties (break self-reference: a wrongly-transcribed booked byte value,
  # even one consistent with its GiB count, makes the matching hard-literal stub mismatch and bounds_check
  # refuse, FAILing the self-test). Ground-truth for THIS frame: 24G / 20G / 0 / 4096 / "3200000 100000".
  # v13: EACH tie gets its OWN stub dir, seeded with all booked values, so one bad tie cannot cascade into
  # the later ties' diagnostics (a wrong literal FAILs exactly the one tie it belongs to).
  _hl() { # $1=name $2=file $3=hard-literal value  -> isolated dir seeded booked, override one file, must pass
    local d="$dir/hl_$1"; mkdir -p "$d"
    printf '%s' "$MEMMAX_BYTES"  > "$d/memory.max"
    printf '%s' "$MEMHIGH_BYTES" > "$d/memory.high"
    printf '%s' "$TASKS"         > "$d/pids.max"
    printf '%s' "$CPUMAX"        > "$d/cpu.max"
    printf '%s' "$SWAPMAX"       > "$d/memory.swap.max"
    printf '%s' "$3" > "$d/$2"
    bounds_check "$d" >/dev/null 2>&1 || { echo "SELF_TEST=FAIL reason=bounds_$1_hardliteral_should_pass"; bad=1; }; }
  _hl memmax  memory.max      '25769803776'
  _hl memhigh memory.high     '21474836480'
  _hl pids    pids.max        '4096'
  _hl cpu     cpu.max         '3200000 100000'
  # swap hard-literal tie (INDEPENDENT of $SWAPMAX): the booked swap bound must BE the literal 0, so a
  # tampered SWAPMAX (e.g. 1) makes bounds_check reject the hard-literal stub '0' and FAIL the self-test.
  _hl swap    memory.swap.max '0'
  printf '%s' "$MEMMAX_BYTES" > "$dir/cgdir/memory.max"; printf '%s' "$MEMHIGH_BYTES" > "$dir/cgdir/memory.high"; printf '%s' "$TASKS" > "$dir/cgdir/pids.max"; printf '%s' "$CPUMAX" > "$dir/cgdir/cpu.max"; printf '%s' "$SWAPMAX" > "$dir/cgdir/memory.swap.max"

  # ---- OOM judge fixtures ----
  printf 'low 0\nhigh 0\nmax 0\noom 3\noom_kill 2\noom_group_kill 0\n' > "$dir/cgdir/memory.events"
  oom_assert "3 2" "$dir/cgdir" steady >/dev/null 2>&1 || { echo "SELF_TEST=FAIL reason=oom_steady_should_pass"; bad=1; }
  printf 'oom 3\noom_kill 3\n' > "$dir/cgdir/memory.events"; oom_assert "3 2" "$dir/cgdir" rise >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=oom_kill_rise_should_refuse"; bad=1; }
  printf 'oom x\noom_kill y\n' > "$dir/cgdir/memory.events"; oom_assert "3 2" "$dir/cgdir" incoherent >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=oom_incoherent_should_refuse"; bad=1; }

  # ---- roster gate: file order equals FUNCS; inversion refuses ----
  { for fn in $FUNCS; do printf 'test fn %s() -> Bool { true }\n' "$fn"; done; } > "$dir/roster_ok"
  roster_gate "$dir/roster_ok" >/dev/null 2>&1 || { echo "SELF_TEST=FAIL reason=roster_ok_should_pass"; bad=1; }
  { printf 'test fn %s() -> Bool { true }\n' a_review_that_fits_is_admitted_not_refused
    for fn in $FUNCS; do [ "$fn" = a_review_that_fits_is_admitted_not_refused ] && continue; printf 'test fn %s() -> Bool { true }\n' "$fn"; done; } > "$dir/roster_bad"
  roster_gate "$dir/roster_bad" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=roster_inversion_should_refuse"; bad=1; }

  # ---- check_cap: unreadable file refuses ----
  check_cap "$dir/nope" >/dev/null 2>&1 && { echo "SELF_TEST=FAIL reason=check_cap_unreadable_should_refuse"; bad=1; }

  # ---- negative fixture: an exported CG_FILE must NOT be able to redirect run-mode scope/bounds/OOM ----
  # The launcher-bounds gate proves the script runs inside the launcher's cgroup; if its path came from the
  # environment, a forged CG_FILE would steer bounds_check + every OOM read. Production must read the
  # kernel's /proc/self/cgroup literally and take the path ONLY as a hard-coded literal, never from env.
  local self prod
  self="${BASH_SOURCE[0]:-$0}"
  case "$self" in /*) : ;; *) self="$(cd -- "$(dirname -- "$self")" 2>/dev/null && pwd)/$(basename -- "$self")";; esac
  # production run-mode region = from the entry marker to EOF (self_test() and all fixture code, including
  # this fixture's own CG_FILE use, sit ABOVE this marker and are therefore excluded from the assertion).
  prod="$(sed -n '/^# ============================ entry ====/,$p' "$self" 2>/dev/null)"
  [ -n "$prod" ] || { echo "SELF_TEST=FAIL reason=could-not-isolate-production-region"; bad=1; }
  if printf '%s' "$prod" | grep -q 'CG_FILE'; then
    echo "SELF_TEST=FAIL reason=CG_FILE-seam-present-in-production-region"; bad=1; fi
  # the production scope entry must be the hard-coded literal call
  printf '%s' "$prod" | grep -q 'scope_gate /proc/self/cgroup' \
    || { echo "SELF_TEST=FAIL reason=production-scope-not-hardcoded-literal"; bad=1; }
  # with a forged CG_FILE exported and a crafted VALID cgroup file, the parser still reads ONLY its positional
  # argument (which production hard-codes to /proc/self/cgroup) and never the env:
  printf '0::/user.slice/user@1000.service/app.slice/%s.scope\n' "$JOB" > "$dir/evil_cgroup"
  CG_FILE="$dir/evil_cgroup"; export CG_FILE
  # forged path passed positionally IS read (proves authority = positional, not env):
  scope_gate "$dir/evil_cgroup" >/dev/null 2>&1 \
    || { echo "SELF_TEST=FAIL reason=scope-positional-authority-broken"; bad=1; }
  # the same forged file must NOT be consulted when the parser is pointed at an unreadable positional,
  # even though CG_FILE names it (env is never consulted):
  scope_gate "$dir/definitely_missing_cgroup_file" >/dev/null 2>&1 \
    && { echo "SELF_TEST=FAIL reason=CG_FILE-env-was-consulted-should-refuse"; bad=1; }
  unset CG_FILE

  # ================= v9 abort / mode-isolation / real-caller controls (calm-crab B1-B3, sleek-owl 1-3) =
  # Real caller = bash running this script; NO production env seam; NO run mode entered, so NO network.
  # canonical script path computed ONCE above (as 'self', absolute); reuse it for real-caller child spawns.
  local SELF; SELF="$self"; [ -r "$SELF" ] || SELF="$0"
  countpv(){ grep -c '^PACKET_VERDICT=' <<<"$1" || true; }
  child(){ # $1=cwd  rest=argv  -> prints child stdout; sets C_RC
    ( cd "$1" && bash "$SELF" "${@:2}" ) 2>/dev/null; }

  # (A) classify must print NOTHING (single write is decoupled to emit_receipt): a mid-classification
  #     abort can therefore never leak a partial match/mismatch receipt.
  mk_allpass "$dir/csilent"
  local csilent; csilent="$( classify "$dir/csilent" 0 2>/dev/null )"
  [ -z "$csilent" ] || { echo "SELF_TEST=FAIL reason=classify-must-not-print leaked=[$csilent]"; bad=1; }

  local cwd c1 crc cnv
  # (B) typo mode -> rc 2, exactly one NO_OBSERVATION, NO directory created (B1)
  cwd="$dir/c_typo"; mkdir -p "$cwd"
  c1="$( child "$cwd" --self-tset )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
      && printf '%s' "$c1" | grep -q 'unknown or missing mode'; } \
    || { echo "SELF_TEST=FAIL reason=typo-mode rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=typo-mode-created-dir"; bad=1; }
  # (C) missing mode -> rc 2, one NO_OBSERVATION, no dir
  cwd="$dir/c_missing"; mkdir -p "$cwd"
  c1="$( child "$cwd" )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q 'unknown or missing mode'; } || { echo "SELF_TEST=FAIL reason=missing-mode rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=missing-mode-created-dir"; bad=1; }
  # (D) unknown flag -> rc 2, one NO_OBSERVATION, no dir
  cwd="$dir/c_unk"; mkdir -p "$cwd"
  c1="$( child "$cwd" --definitely-not-a-mode )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q 'unknown or missing mode'; } || { echo "SELF_TEST=FAIL reason=unknown-mode rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=unknown-mode-created-dir"; bad=1; }
  # (E) --run RELATIVE OUT -> refuse at absolute check, BEFORE scope/mkdir/fetch; no dir
  cwd="$dir/c_rel"; mkdir -p "$cwd"
  c1="$( child "$cwd" --run relative/out )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "absolute path"; } \
    || { echo "SELF_TEST=FAIL reason=rel-out rc=$crc npv=$cnv"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=rel-out-created-dir"; bad=1; }
  # (F) --run WRONG-SUFFIX OUT -> refuse at booked-path check, BEFORE scope/mkdir/fetch; no dir
  cwd="$dir/c_sfx"; mkdir -p "$cwd"
  c1="$( child "$cwd" --run "$cwd/wrongname/out" )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ]; } || { echo "SELF_TEST=FAIL reason=bad-suffix rc=$crc npv=$cnv"; bad=1; }
  [ -e "$cwd/wrongname/out" ] && { echo "SELF_TEST=FAIL reason=bad-suffix-created-out"; bad=1; }
  # (G) STRUCTURAL scope-before-mkdir: in the production region, scope_gate AND the first bounds_check
  #     must precede 'mkdir -- "$OUT"' (so no directory is created unless scope+bounds already passed).
  #     Checked by inspection HERE AND behaviorally by control (H5), which DOES spawn a live --run (correct
  #     booked suffix, parent present), stopped only by the run preamble's scope gate BEFORE mkdir. (H5) is
  #     safe ONLY because the self-test runs OUTSIDE the booked scope; self_test() therefore REFUSES outright
  #     if scope_gate would let a real run through (see the in-scope SAFETY guard at the top of self_test()).
  local i_sc i_bd i_mkfirst i_mkp
  i_sc="$(printf '%s\n' "$prod" | grep -n 'scope_gate /proc/self/cgroup' | head -1 | cut -d: -f1)"
  i_bd="$(printf '%s\n' "$prod" | grep -n 'bounds_check "$CGDIR"' | head -1 | cut -d: -f1)"
  # FIRST mkdir touching "$OUT" of ANY form (any flags, INCLUDING -p) must come AFTER bounds_check, so no
  # directory can exist unless scope+bounds already passed. This catches an INSERTED 'mkdir -p -- "$OUT"'
  # before the gate, which the old exact 'mkdir -- "$OUT"' grep alone would MISS (self-test stayed GREEN).
  i_mkfirst="$(printf '%s\n' "$prod" | grep -nE 'mkdir.*\$OUT' | head -1 | cut -d: -f1)"
  # and NO mkdir of "$OUT" may use -p (a recursive mkdir would create parents, defeating exclusivity).
  i_mkp="$(printf '%s\n' "$prod" | grep -nE 'mkdir[[:space:]].*-p.*\$OUT' | head -1 | cut -d: -f1)"
  { [ -n "$i_sc" ] && [ -n "$i_bd" ] && [ -n "$i_mkfirst" ] \
      && [ "$i_sc" -lt "$i_bd" ] && [ "$i_bd" -lt "$i_mkfirst" ] && [ -z "$i_mkp" ]; } \
    || { echo "SELF_TEST=FAIL reason=mkdir-not-after-scope-and-bounds-or-has-p sc=$i_sc bd=$i_bd mkfirst=$i_mkfirst mkp=$i_mkp"; bad=1; }
  # the structural run-isolation guard and the explicit --run token must be present in the production text
  printf '%s' "$prod" | grep -qF 'if [ "$MODE" != run ]' \
    || { echo "SELF_TEST=FAIL reason=structural-run-guard-missing"; bad=1; }
  printf '%s' "$prod" | grep -q -- '--run)' \
    || { echo "SELF_TEST=FAIL reason=explicit-run-token-missing"; bad=1; }
  # (H) trap children: unbound / arithmetic / TERM -> exactly one NO_OBSERVATION, rc 2 (through real exec)
  local kind
  # (H) trap children THROUGH REAL EXEC: exactly one NO_OBSERVATION, rc 2. v13 SPLITS signal vs
  #     non-signal aborts and asserts the DISTINGUISHING reason, so a kill that NEVER fires cannot pass:
  #     it would fall to the 'exited its arm ... must not happen' fallback (also one NO_OBSERVATION rc2),
  #     which the old rc/npv checks alone accepted as GREEN (keen-otter msg_1387812e item 3).
  for kind in unbound arith; do          # non-signal aborts: reach on_abort OR the guard, never the fallback
    c1="$( child "$cwd" __selftest_trap__ "$kind" )"; crc=$?; cnv="$(countpv "$c1")"
    { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
        && ! printf '%s' "$c1" | grep -q 'exited its arm'; } \
      || { echo "SELF_TEST=FAIL reason=abort-$kind rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  done
  for kind in term hup int; do            # signals: the trap MUST fire (on_abort 'aborted: raw rc'); a
    c1="$( child "$cwd" __selftest_trap__ "$kind" )"; crc=$?; cnv="$(countpv "$c1")"  # never-fired kill -> fallback -> RED
    { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
        && printf '%s' "$c1" | grep -q 'aborted: raw rc' \
        && ! printf '%s' "$c1" | grep -q 'exited its arm'; } \
      || { echo "SELF_TEST=FAIL reason=trap-$kind rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  done
  # (H2) POST-receipt signal: a receipt was ALREADY emitted, then a signal. Must be EXACTLY one receipt
  #      (the emitted MATCH, no appended NO_OBSERVATION) AND rc forced to 2 -- NOT the receipt's rc. Catches
  #      dropping the TERM/HUP trap and the v7 MAJOR (EXIT pass-through on _done alone instead of rc-match).
  for kind in postterm posthup postint; do
    c1="$( child "$cwd" __selftest_trap__ "$kind" )"; crc=$?; cnv="$(countpv "$c1")"
    { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_MATCH$" \
        && ! printf '%s' "$c1" | grep -q "$V_NO"; } \
      || { echo "SELF_TEST=FAIL reason=postreceipt-$kind rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  done
  # (H3) tail failure: classify-like RPT built then aborted BEFORE emit -> exactly one NO_OBSERVATION, the
  #      stale MATCH ('should-never-be-seen') NEVER surfaces, and no MATCH receipt is emitted.
  c1="$( child "$cwd" __selftest_trap__ tailfail )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
      && ! printf '%s' "$c1" | grep -q 'should-never-be-seen' \
      && ! printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_MATCH$"; } \
    || { echo "SELF_TEST=FAIL reason=tailfail rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  # (H4) verify-abort (EXECUTED, not a grep): an abort in the verify path must NOT reach run code -> empty
  #      cwd (no booked OUT directory created), exactly one NO_OBSERVATION, no stale MATCH leak.
  cwd="$dir/c_verifyabort"; mkdir -p "$cwd"
  c1="$( child "$cwd" __selftest_verifyabort__ )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
      && printf '%s' "$c1" | grep -q 'non-run mode terminated abnormally' \
      && ! printf '%s' "$c1" | grep -q 'should-never-be-seen'; } \
    || { echo "SELF_TEST=FAIL reason=verifyabort rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=verifyabort-created-dir"; bad=1; }
  # (H4b) self-test abort THROUGH THE ACTUAL DISPATCH must reach the run-isolation GUARD: MODE=self aborts
  #      its case compound and bash continues past 'esac' ONLY to be refused by the guard (whose reason is
  #      asserted here). DELETING the guard routes it into the run preamble instead -> reason changes -> RED.
  cwd="$dir/c_selfabort"; mkdir -p "$cwd"
  c1="$( child "$cwd" __selftest_selfabort__ )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
      && printf '%s' "$c1" | grep -q 'non-run mode terminated abnormally' \
      && ! printf '%s' "$c1" | grep -q 'should-never-be-seen'; } \
    || { echo "SELF_TEST=FAIL reason=selfabort rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ -z "$(ls -A "$cwd")" ] || { echo "SELF_TEST=FAIL reason=selfabort-created-dir"; bad=1; }
  # (H5) BEHAVIORAL pre-gate mkdir control (no grep spelling trusted): drive --run through the ACTUAL
  #      dispatch with an ABSOLUTE booked OUT under a scratch cwd. Here the process is NOT inside the booked
  #      launcher scope, so scope_gate (/proc/self/cgroup) MUST refuse BEFORE any directory exists: rc 2,
  #      EXACTLY one receipt, the scope-gate refusal reason, and an EMPTY scratch (booked OUT absent). An
  #      inserted 'mkdir "${OUT}"', 'install -d -- "$OUT"' or 'mkdir -p -- "$OUT"' BEFORE the gate creates
  #      the directory first -> this control goes RED, regardless of how the mkdir is spelled.
  # v13: create the booked OUT PARENT ($JOB) with the leaf ABSENT (the production 'exclusive mkdir'
#      invariant), so a single-level 'mkdir -- "$OUT"' / 'mkdir "${OUT}"' moved before the gate CAN
#      create the leaf and is caught — a bare mkdir of a nested path would otherwise silently no-op.
  cwd="$dir/c_pregate"; mkdir -p "$cwd/$JOB"
  c1="$( child "$cwd" --run "$cwd/$JOB/out" )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 2 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_NO$" \
      && printf '%s' "$c1" | grep -q 'scope gate refused'; } \
    || { echo "SELF_TEST=FAIL reason=pregate-behavioral rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  [ ! -e "$cwd/$JOB/out" ] || { echo "SELF_TEST=FAIL reason=pregate-behavioral-created-dir"; bad=1; }
  # (I) valid match / mismatch THROUGH the real caller (--verify-captured): correct rc + one receipt + count
  mk_allpass "$dir/rc_match"
  c1="$( child "$cwd" --verify-captured "$dir/rc_match" 0 )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 0 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_MATCH$"; } \
    || { echo "SELF_TEST=FAIL reason=realcaller-match rc=$crc npv=$cnv"; bad=1; }
  mk_withfail "$dir/rc_mis" srv1_shape_refuses_the_goal_audit_reviewer_before_launch
  c1="$( child "$cwd" --verify-captured "$dir/rc_mis" 1 )"; crc=$?; cnv="$(countpv "$c1")"
  { [ "$crc" = 1 ] && [ "$cnv" = 1 ] && printf '%s' "$c1" | grep -q "^PACKET_VERDICT=$V_MISMATCH$" \
      && printf '%s' "$c1" | grep -q 'bare-FAIL rows=1 of 5'; } \
    || { echo "SELF_TEST=FAIL reason=realcaller-mismatch rc=$crc npv=$cnv out=[$c1]"; bad=1; }
  # (K) v14 in-scope SAFETY control (MINOR B; keen-otter msg_a530c1b3): hermetically simulate --self-test
  #     executed INSIDE the booked launcher scope WITHOUT any real scope or host probe: stub scope_gate to
  #     SUCCEED and arm 'bash' as a tripwire. child() is self_test's ONLY external surface and spawns
  #     `bash "$SELF"`; run-mode fetch/build live behind the scope gate INSIDE that child. The top-of-
  #     self_test SAFETY guard must refuse BEFORE fixture creation or any child dispatch, so the tripwire
  #     file must be ABSENT. Asserts rc 2 + the refusal reason + no SELF_TEST=PASS + no tripwire (=> no
  #     child => fetch/build unreachable). INVERSION: removing the top guard lets self_test proceed under
  #     the forced scope => a child spawns => tripwire fires => this control goes RED. Recursion is bounded
  #     one level by _ROW29_GUARDPROBE (the inner probe self-test does not re-run (K)).
  if [ -n "${_ROW29_GUARDPROBE-}" ]; then : ; else
    _gp_ok=0
    (
      export _ROW29_GUARDPROBE=1
      trip="$dir/ktrip"; rm -f "$trip"
      bash() { : > "$trip"; return 9; }
      scope_gate() { return 0; }
      kout="$(self_test 2>/dev/null)"; krc=$?
      if [ "$krc" = 2 ] && printf '%s' "$kout" | grep -q 'self-test-refused-inside-booked-scope' \
         && ! printf '%s' "$kout" | grep -q 'SELF_TEST=PASS' && [ ! -e "$trip" ]; then exit 0; fi
      echo "guardprobe: krc=$krc trip=$([ -e "$trip" ] && echo yes || echo no) out=[$kout]" >&2
      exit 1
    ) && _gp_ok=1 || _gp_ok=0
    [ "$_gp_ok" = 1 ] || { echo "SELF_TEST=FAIL reason=guardprobe"; bad=1; }
  fi
  if [ "$bad" = 0 ]; then echo SELF_TEST=PASS; return 0; else echo SELF_TEST=FAIL; return 2; fi
}

# ============================ entry ============================
# ============================ entry ============================
# Single-receipt machinery. EXACTLY ONE complete PACKET_VERDICT block is ever written to stdout, and
# ONLY from emit_receipt (after a complete classify) or from the EXIT trap (on any abort). classify and
# die2 build the receipt into RPT and never print it, so a mid-classification abort can leak no partial
# receipt. The EXIT/signal traps are armed for ALL modes; --self-test/--limitations mark the receipt
# complete so their own PASS/FAIL or text is the output and no spurious verdict is appended.
#
# COLLECTION RULE (the keeper MUST apply on collection, independent of the script):
#   VALID only if ALL hold: the session completed intact (ssh rc 0, scope ended, keeper reconcile clean);
#   the script rc is EXACTLY 0 or 1; stdout has EXACTLY ONE PACKET_VERDICT line; and the pair is either
#   (rc 0, OBSERVED_MATCH) or (rc 1, OBSERVED_MISMATCH followed by its 'bare-FAIL rows=N of 5' line),
#   with the joined claims-fd.log preserved and its sha recorded. EVERYTHING ELSE is NO_OBSERVATION:
#   missing / duplicate / contradictory verdicts, rc 2, any other rc, a signal (128+n), the trap's
#   'aborted:' reason, SIGKILL, or session loss. A mismatch is NEVER inferred from rc 1 alone; no trap
#   can produce a receipt for SIGKILL or a lost session, and none is expected. A post-receipt interruption
#   (a signal AFTER the one receipt has printed) is NOT an intact observation merely because a verdict line
#   exists: it forces rc 2, so the (rc,verdict) pair fails the rule and collection treats it NO_OBSERVATION.
RPT=""
rpt() { RPT="PACKET_VERDICT=$1"; shift; local l; for l in "$@"; do [ -n "$l" ] && RPT+=$'\n'"$l"; done; }
_receipt_done=0; _receipt_rc=0
emit_receipt() { local c="${1:-2}"; _receipt_done=1; _receipt_rc="$c"
  [ -n "$RPT" ] || { RPT="PACKET_VERDICT=$V_NO"; RPT+=$'\n'"REASON=internal: no receipt assembled"; _receipt_rc=2; }
  printf '%s\n' "$RPT"; exit "$_receipt_rc"; }
on_exit() { local rc=$?
  if [ "$_receipt_done" = 1 ]; then if [ "$rc" = "$_receipt_rc" ]; then exit "$rc"; fi; exit 2; fi
  printf 'PACKET_VERDICT=%s\nREASON=aborted: raw rc %s without a complete receipt\n' "$V_NO" "$rc"; _receipt_done=1; exit 2; }
on_sig() { exit $((128 + ${1:-0})); }
die2() { RPT=""; rpt "$V_NO" "REASON=$1"; emit_receipt 2; }
trap 'on_exit' EXIT
trap 'on_sig 15' TERM
trap 'on_sig 1'  HUP
trap 'on_sig 2'  INT

MODE=""
case "${1:-}" in
  --self-test)   MODE=self; self_test "$@"; _c=$?; _receipt_done=1; _receipt_rc="$_c"; exit "$_c";;
  --limitations) MODE=lim; LIMITATIONS; _receipt_done=1; _receipt_rc=0; exit 0;;
  --verify-captured) MODE=verify; shift
      if [ $# -lt 2 ]
        then rpt "$V_NO" "REASON=usage: --verify-captured <captured-file> <producer-rc>"; emit_receipt 2
      fi
      classify "$1" "$2"; _c=$?; emit_receipt "$_c";;
  --run) MODE=run;;
  __selftest_verifyabort__) MODE=verify
      # EXECUTED proof (not a grep) that an abort DURING the verify path can NEVER fall through into run
      # mode: a classify-like RPT is built, then an arithmetic abort fires. MODE stays 'verify', so the
      # structural guard below refuses BEFORE any mkdir -- the booked OUT directory is never created.
      rpt "$V_MATCH" "REASON=should-never-be-seen"; _eg=$((1/0));;
# (C, keen-otter): the __selftest_trap__ / __selftest_verifyabort__ / __selftest_selfabort__ arms and the
#     postterm/posthup/postint/tailfail kinds are reachable from PRODUCTION argv (like the v12 test arms).
#     Each is bounded: it can only end at the run-isolation guard or a self-signal (rc 2 / one NO_OBSERVATION,
#     no run effect, no filesystem effect), and the (A)/(B)/(K) controls prove they cannot reach run code.
  __selftest_selfabort__) MODE=self
      # EXECUTED (not a grep) proof that an abort in a NON-run mode reaches the run-isolation guard: MODE
      # stays 'self', the arithmetic abort abandons this arm, and bash continues past 'esac' ONLY to be
      # refused by the guard (control H4b asserts its reason). Deleting the guard -> run preamble -> RED.
      rpt "$V_MATCH" "REASON=should-never-be-seen"; _eg=$((1/0));;
  __selftest_trap__) MODE=trap
      case "${2:-}" in
        unbound) unset EG_UNBOUND_PROBE; : "$EG_UNBOUND_PROBE";;   # set -u unbound reference
        arith)   _eg=$((1/0));;                                     # arithmetic (division by zero)
        term)    kill -TERM "${BASHPID}";;                           # trapped termination (self, via BASHPID)
        hup)     kill -HUP  "${BASHPID}";;                           # trapped hangup (must be trapped too)
        int)     kill -INT  "${BASHPID}";;                           # trapped interrupt
        # POST-receipt signal: emit ONE receipt as if classify+emit_receipt completed (set _receipt_done +
        # _receipt_rc, print once), THEN signal. The EXIT trap must NOT append a second receipt and MUST
        # force rc 2 (not the receipt's rc). Catches dropping the TERM/HUP trap AND the v7 MAJOR (rc pass-
        # through on _done alone). No clean exit is used; the signal is what terminates the process.
        postterm) rpt "$V_MATCH" "REASON=postreceipt-control"; printf '%s\n' "$RPT"; _receipt_done=1; _receipt_rc=0; kill -TERM "${BASHPID}";;
        posthup)  rpt "$V_MATCH" "REASON=postreceipt-control"; printf '%s\n' "$RPT"; _receipt_done=1; _receipt_rc=0; kill -HUP  "${BASHPID}";;
        postint)  rpt "$V_MATCH" "REASON=postreceipt-control"; printf '%s\n' "$RPT"; _receipt_done=1; _receipt_rc=0; kill -INT  "${BASHPID}";;
        # tail failure: a classify-like RPT is fully built but NOTHING printed; an abort fires BEFORE
        # emit_receipt. The stale MATCH ('should-never-be-seen') must NEVER surface; only NO_OBSERVATION.
        tailfail) rpt "$V_MATCH" "REASON=should-never-be-seen"; _eg=$((1/0));;
        *) rpt "$V_NO" "REASON=unknown trap child kind"; emit_receipt 2;;
      esac
      rpt "$V_NO" "REASON=trap child exited its arm without triggering the trap (must not happen)"; emit_receipt 2;;
  *) rpt "$V_NO" "REASON=unknown or missing mode (use --run/--self-test/--limitations/--verify-captured)"; emit_receipt 2;;
esac
# STRUCTURAL run-isolation guard: run-mode code below is reachable ONLY when MODE=run (set solely by the
# --run token). If a non-run arm aborted its compound (arithmetic/unbound) and bash continued past
# 'esac', MODE != run here and we refuse BEFORE any run-side effect. Guarantees a --verify-captured /
# --self-test abort can never fall through into run mode or touch the filesystem.
if [ "$MODE" != run ]
  then RPT=""; rpt "$V_NO" "REASON=non-run mode terminated abnormally before completion (no run effect)"; emit_receipt 2
fi

# ---- run mode (reached ONLY with MODE=run via the --run token; the structural guard above makes any
# ---- non-run abort unable to arrive here). Validate the OUT path shape, then the scope and the booked
# ---- bounds, BEFORE creating ANY output directory: a mistyped mode or bad OUT can never touch the
# ---- filesystem, and a scope/bounds refusal spends no directory.
[ $# -ge 2 ] || die2 "usage: row29_claim_packet.sh --run <OUT>"
OUT="$2"
[[ "$OUT" = /* ]] || die2 "OUT must be an absolute path (refuse before any effect)"
case "$OUT" in */"$JOB"/out) : ;; *) die2 "OUT must end with the booked /$JOB/out (refuse before any effect)";; esac
CGPATH="$(scope_gate /proc/self/cgroup)" || die2 "scope gate refused (see DIAG)"
CGDIR="/sys/fs/cgroup${CGPATH}"
bounds_check "$CGDIR" || die2 "booked bounds do not match the scope's own cgroup (refuse before effect)"
# exclusive create, AFTER scope+bounds validation: plain mkdir (NO -p) is atomic-exclusive; it fails if
# OUT already exists or its parent is absent (no silent parent creation). No .marker race guard needed.
mkdir -- "$OUT" 2>/dev/null || die2 "OUT must be absent with its parent present (exclusive mkdir): $OUT"
OUT="$(cd "$OUT" && pwd)" || die2 "cannot canonicalize OUT"
START_TS="$(date +%s)"
left() { echo $(( START_TS + DEADLINE_S - $(date +%s) )); }
admit() { # $1 phase name  $2 cap  $3 tail_reserve  -> refuse before starting if cap+grace+tail+final won't fit
  local need=$(( $2 + KILL_GRACE + $3 + FINAL_ALLOW ))
  [ "$(left)" -ge "$need" ] || die2 "phase '$1' cannot fit: need ${need}s, have $(left)s (refuse before start; RT not raised)"
}

WT="$OUT/wt"; L="$OUT/claims-fd.log"
oom_before="$(oom_read "$CGDIR")" || die2 "OOM baseline read failed/incoherent (refuse)"

# fetch: init a repo, exact-pin SHALLOW fetch (no whole-history clone), checkout FETCH_HEAD.
admit fetch "$FETCH_CAP" $(( BUILD_CAP + KILL_GRACE + RUN_CAP + KILL_GRACE ))
git init -q "$WT" || die2 "git init failed"
git -C "$WT" remote add origin "$REPO_URL" || die2 "git remote add failed"
timeout -k 30 "$FETCH_CAP" git -C "$WT" fetch -q --depth 1 origin "$PIN_HEAD" || die2 "git fetch --depth 1 pin failed"
git -C "$WT" checkout -q --detach FETCH_HEAD || die2 "git checkout FETCH_HEAD failed"
[ "$(git -C "$WT" rev-parse HEAD)" = "$PIN_HEAD" ] || die2 "HEAD != PIN_HEAD"
[ "$(git -C "$WT" rev-parse 'HEAD^{tree}')" = "$PIN_TREE" ] || die2 "tree != PIN_TREE"
verify_blob() { [ "$(git -C "$WT" rev-parse "HEAD:$1")" = "$2" ] || die2 "blob mismatch: $1"; }
verify_blob "$ENTRY_PATH" "$BLOB_WITNESS"
verify_blob "dag/std/measure.dag" "$BLOB_MEASURE"
verify_blob "dag/gunbc/compute/host_capacity.dag" "$BLOB_HOSTCAP"
verify_blob "dag/gunbc/live_deploy/slice_bounds.dag" "$BLOB_SLICE"
verify_blob "dag/gunbc/roadmap/roadmap_belt_actuate.dag" "$BLOB_BELT"
verify_blob "dag/gunbc/integration_concern.dag" "$BLOB_CONCERN"
roster_gate "$WT/$ENTRY_PATH" || die2 "roster order != file declaration order"
tree_clean "$WT" || die2 "post-checkout: git status --porcelain not empty"
oom_assert "$oom_before" "$CGDIR" fetch || die2 "OOM during fetch (refuse)"
bounds_check "$CGDIR" || die2 "bounds drifted after fetch (refuse)"

# build claim_batch in-leg (network + shared CARGO_HOME cache OUTSIDE OUT). --locked; -j 32; full fixed cap.
admit build "$BUILD_CAP" $(( RUN_CAP + KILL_GRACE ))
BUILD_BEFORE="$(oom_read "$CGDIR")" || die2 "OOM pre-build read failed (refuse)"
( cd "$WT" && timeout -k 30 "$BUILD_CAP" cargo build --release -p "$PKG" --bin claim_batch --locked -j "$BUILD_JOBS" ) \
  || die2 "build failed or exceeded BUILD_CAP=$BUILD_CAP"
BIN="$WT/$BIN_REL"; BIN_SHA="$(hash_of "$BIN")" || die2 "cannot hash claim_batch binary"
echo "claim_batch sha256=$BIN_SHA"
oom_assert "$BUILD_BEFORE" "$CGDIR" build || die2 "OOM during build (refuse)"

# one claim run, combined 2>&1 capture (verdict lines on STDOUT; completion/cost on STDERR). full fixed cap.
admit run "$RUN_CAP" 0
RUN_BEFORE="$(oom_read "$CGDIR")" || die2 "OOM pre-run read failed (refuse)"
( cd "$WT" && timeout -k 30 "$RUN_CAP" "$BIN" \
    --source-root dag --source-root src/v2 --claim-run --hermetic \
    --entry "$ENTRY_PATH" --functions "$(echo "$FUNCS" | tr ' ' ',')" ) > "$L" 2>&1
RC=$?
echo "claim_batch_exit=$RC"
mem_snapshot "$CGDIR" after_run
oom_assert "$RUN_BEFORE" "$CGDIR" run || die2 "OOM during claim run (refuse)"
# whole-run OOM boundary (regardless of producer rc)
oom_assert "$oom_before" "$CGDIR" whole-run || die2 "OOM over the whole run (refuse)"

# binary unchanged; worktree clean after run
[ "$(hash_of "$BIN")" = "$BIN_SHA" ] || die2 "post-run: claim_batch binary sha changed"
tree_clean "$WT" || die2 "post-run: git status --porcelain not empty"

# classify + exit per the contract (check_cap is inside classify; refuses on read failure/exceed)
# classify builds the whole receipt; emit_receipt prints it ONCE and exits with its code. Any abort
# before this point yields the trap's single NO_OBSERVATION; a partial match/mismatch is never printed.
classify "$L" "$RC"; _c=$?; emit_receipt "$_c"
