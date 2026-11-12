# SCAFFOLD — dissolve-on: the model is gunbc.witness.roadmap_cutover, and the promise is written there
# (dispatch_witness_dissolve_trigger), not here. Two capabilities retire DIFFERENT halves of this file.
#   (1) orch-emitted, for the RESIDUAL SHELL (the gh stub, the case driver, the run-id set difference): the bash
#       target grammar (v2.extdeps.languages.bash rows, built by v2.extdeps.languages.bash_build, serialized by
#       v2.workflow.bash_emit) has no production for a case statement over glob patterns, an indexed array read
#       back with slice arithmetic ("${args[@]:1}"), a here-doc that materialises a stub executable, or command
#       substitution inside an assignment used as a predicate. That is a missing target-language capability
#       (class (b) of the hand-shell gate), not an unauthored migration.
#   (2) modelled, for the WITNESS PROCESS: the ten cases below are rows of
#       gunbc.witness.roadmap_cutover::cutover_witness_case_rows, and dispatch_stage itself is loaded from the
#       runbook rather than copied. When fleet-converge's own admission
#       (gunbc.fleet.fleet_converge_plan_revision::admit_dashboard_deploy_dispatch_inputs) is exercised through
#       modelled test rows in gunbc.instrument_targets, this file is DELETED rather than emitted.
#
# Witness for the cutover runbook's CI-dispatch step (docs/runbooks/roadmap-event-log-cutover.md, step 5b).
#
# It drives the runbook's OWN `dispatch_stage` function — extracted from the runbook file at run time, never copied —
# against a `gh` that answers exactly the queries that function makes. The "which run did my dispatch create" query
# is emulated honestly: the exclusion set is read out of the query the function itself built, so a run that was
# already there cannot impersonate the dispatched one, and a function that lost its snapshot loses its exclusion
# set too — which is what the snapshot cases are for.
#
# The cases: the dispatched run is admitted; a run declaring another mode, one at another revision, one that was
# already there, one that never appeared and one that did not conclude success all refuse; and the four ways the
# correlation itself can silently go wrong all refuse — a snapshot command that fails, a snapshot that comes back
# empty, a dispatch already in flight at the revision, and two new runs appearing at once.
#
# Run:  bash tools/roadmap_cutover_dispatch_witness.sh docs/runbooks/roadmap-event-log-cutover.md
# CASES IMPLEMENTED HERE — one line per gunbc.witness.roadmap_cutover::cutover_witness_case_rows row
# this carrier executes. The two-way join against these rows runs in this file (PART H here, PART B in
# the dispatch witness): a row with no line here is a promise no script keeps, and a line here with no
# row is a case the model has never heard of. The names are data, so prose cannot satisfy the join.
#   dispatch_admits_the_run_it_created
#   dispatch_refuses_a_run_declaring_another_mode
#   dispatch_refuses_a_run_at_another_revision
#   dispatch_refuses_when_no_run_appeared
#   dispatch_refuses_a_pre_existing_run_impersonating_the_dispatch
#   dispatch_refuses_a_run_that_did_not_conclude_success
#   dispatch_refuses_a_snapshot_whose_command_failed
#   dispatch_refuses_a_snapshot_that_came_back_empty
#   dispatch_refuses_a_dispatch_already_in_flight_at_the_revision
#   dispatch_refuses_an_in_flight_check_that_failed
#   dispatch_refuses_two_new_runs_at_one_revision
#   dispatch_refuses_a_poll_that_failed
#   dispatch_refuses_a_run_waiting_for_a_runner
#   dispatch_refuses_a_run_awaiting_a_decision
#   dispatch_refuses_a_run_that_was_requested
#   dispatch_refuses_a_failed_dispatch_and_never_polls
#   dispatch_refuses_while_another_dispatch_holds_the_fence
#   dispatch_refuses_without_a_fence_path
#   dispatch_refuses_a_run_at_the_same_revision_from_another_ref
#   dispatch_refuses_a_run_whose_record_names_another_ref
#   dispatch_refuses_a_dispatch_ref_that_already_exists
#   dispatch_refuses_when_the_dispatch_ref_cannot_be_pushed
#   dispatch_refuses_a_dispatch_ref_resolving_to_another_revision
#   dispatch_refuses_without_a_per_invocation_reference
#   dispatch_refuses_a_reference_that_does_not_name_this_revision
#   dispatch_executes_the_whole_activation_prelude
#   dispatch_refuses_the_prelude_when_the_read_back_was_never_verified
#   activation_refuses_an_admission_whose_verification_could_not_be_redone
#   activation_refuses_an_admission_with_no_verification_attempt_recorded
#   activation_refuses_a_forged_admission_pair
#   activation_refuses_a_verification_name_that_cannot_be_destroyed
#   activation_refuses_a_verification_whose_name_cannot_be_published
#
set -u
RB=${1:?usage: roadmap_cutover_dispatch_witness.sh <path to the runbook>}
TMP=$(mktemp -d); BIN=$TMP/bin; export PATH=$BIN:$PATH
MODEL=${2:-$(cd "$(dirname "$RB")/../.." && pwd)/dag/gunbc/witness/roadmap_cutover.dag}
[ -f "$MODEL" ] || { echo "WITNESS REFUSED: no model at [$MODEL] — the case rows are read from it, not invented here"; exit 1; }
# THE ROSTER, OUT OF THE MODEL. Step 5 reads the roster step 0 emitted, and step 0 emits
# roadmap_migration_retirement_ids. A prelude that typed its own node list here would certify the activation gate
# against a roster the model does not hold — the same second-home mistake the runbook refuses in its own prose —
# so the ids are read out of the event log, and an empty answer refuses this witness.
ELOG=${ELOG_ARG:-$(cd "$(dirname "$MODEL")/../roadmap" && pwd)/roadmap_event_log.dag}
model_retirement_ids() {
  [ -f "$ELOG" ] || { echo "WITNESS REFUSED: no event log at [$ELOG] to read the retirement roster from" >&2; return 1; }
  awk '/^fn roadmap_migration_retirement_ids/,/^}/' "$ELOG" | sed -n 's/^[[:space:]]*"\([^"]*\)".*/\1/p'
}
mkdir -p "$BIN"

cat > "$BIN/gh" <<'STUB'
#!/usr/bin/env bash
# Stands in for gh, answering only what dispatch_stage asks. Every call is logged, so a case can assert not just
# the answer but the SEQUENCE — a function that polled after a dispatch that failed is not "refused", it is wrong.
echo "$*" >> "${GH_CALLS:-/dev/null}"
case "$1 $2" in
  "workflow run"*) [ -n "${STUB_FAIL_DISPATCH:-}" ] && exit 1
                   # POOL mode is for the three-stage transaction: a dispatched run comes into EXISTENCE here and
                   # only then appears in any later query, so the poll cannot see a run that was never dispatched
                   # and the pre-dispatch snapshot cannot be handed a future run for free. Without STUB_POOL the
                   # single-stage cases answer exactly as they always have.
                   if [ -n "${STUB_POOL:-}" ]; then
                     for id in $STUB_POOL; do
                       grep -qx -- "$id" "${STUB_STATE:?}" 2>/dev/null && continue
                       printf '%s\n' "$id" >> "${STUB_STATE:?}"; break
                     done
                   fi
                   exit 0 ;;
  "run list"*)
    q=""
    for a in "$@"; do q="$q$a"; done
    case "$q" in
      # The in-flight check: a projection ending in `| length`. It is answered BY STATUS, from the statuses the
      # QUERY ITSELF names: STUB_LIVE_STATUSES says which nonterminal states a run is really sitting in, and a run
      # is only counted if the query asks for that state. So a function that drops `waiting` from its filter does
      # not get a free pass from a fixture that knows a waiting run exists — it gets 0 and dispatches beside it.
      *'| length'*) [ -n "${STUB_FAIL_LIVE:-}" ] && exit 1
                    n=0
                    for s in ${STUB_LIVE_STATUSES:-}; do printf '%s' "$q" | grep -qF "\"$s\"" && n=$((n + 1)); done
                    echo $(( n + ${STUB_LIVE:-0} )) ;;
      # The pre-dispatch snapshot: `--json databaseId -q '.[].databaseId'`. Its exit status is what the runbook
      # checks, so the stub must be able to fail it, and to answer with nothing.
      *'.[].databaseId'*)
        [ -n "${STUB_FAIL_SNAPSHOT:-}" ] && exit 1
        [ -n "${STUB_EMPTY_SNAPSHOT:-}" ] && exit 0
        if [ -n "${STUB_POOL:-}" ]; then
          { printf '%s\n' ${STUB_OLD_IDS:-}; cat "${STUB_STATE:-/dev/null}" 2>/dev/null; } | grep . || true
        else
          printf '%s\n' ${STUB_OLD_IDS:-} | grep . || true
        fi ;;
      # The poll: `--json databaseId,headSha,event` with `- [ids] | .[]`. The exclusion set is taken FROM THE
      # QUERY the function built (plain parameter expansion; a sed address here is its own little minefield), so
      # the fixture cannot accidentally hand it the answer the function forgot to ask for.
      *'databaseId,headSha,event'*)
        [ -n "${STUB_FAIL_POLL:-}" ] && exit 1
        # THE POLL ANSWERS ONLY FOR THE REF IT WAS ASKED ABOUT. GitHub filters runs by the ref they were
        # dispatched from; a stub that ignored that filter would let another host's dispatch look admissible.
        # want_ref is read out of the QUERY the function built, so a runbook that dropped the filter cannot be
        # passed by a fixture — and a case asserts separately that the filter is in the query at all.
        want_ref=$(printf '%s\n' "$q" | sed -n 's/.*headBranch=="\([^"]*\)".*/\1/p')
        printf '%s' "$want_ref" > "${STUB_LAST_REF:-/dev/null}"
        ex=${q#*'] - ['}; ex=${ex%%']'*}; ex=${ex//,/ }   # jq's list is comma-separated; the cases match on words
        for id in ${STUB_MATCH_IDS:-}; do
          case " $ex " in *" $id "*) continue ;; esac
          # In POOL mode a run that was never dispatched does not exist, so it cannot be adopted.
          if [ -n "${STUB_POOL:-}" ]; then grep -qx -- "$id" "${STUB_STATE:?}" 2>/dev/null || continue; fi
          if [ -z "${STUB_IGNORE_REF_FILTER:-}" ] && [ -n "$want_ref" ]; then
            idref=$(printf '%s\n' "${STUB_RUN_REFS:-}" | tr ' ' '\n' | sed -n "s|^$id=||p")
            [ -n "$idref" ] || idref="$want_ref"        # a run the case never named belongs to this dispatch
            [ "$idref" = "$want_ref" ] || continue
          fi
          printf '%s\n' "$id"
        done ;;
      *) echo "" ;;
    esac ;;
  "run watch"*) [ "${STUB_CONCLUSION:-success}" = success ] && exit 0 || exit 1 ;;
  # the plan's artifact hash, which the prelude carries onto the two later dispatches
  "run view"*) printf 'plan_artifact_hash=%s\n' "${STUB_PLAN_HASH:-$(printf 'c%.0s' {1..64})}"; exit 0 ;;
  "api "*)
    case "$*" in
      *'.inputs.mode'*)
        # Which run is being asked about is read out of the QUERY. In the three-stage transaction each stage
        # dispatches a different mode, so a stub that answered one static mode could never witness all three.
        if [ -n "${STUB_ID_MODES:-}" ]; then
          rid=$(printf '%s' "$*" | sed -n 's|.*/runs/\([0-9][0-9]*\).*|\1|p')
          printf '%s\n' "$STUB_ID_MODES" | tr ' ' '\n' | sed -n "s|^$rid=||p"
        else
          echo "${STUB_MODE:-}"
        fi ;;
      *'.head_sha'*)    echo "${STUB_SHA:-}" ;;
      # the ref the run was dispatched FROM is what makes it THIS invocation's run. Default: whatever the poll
      # last asked for — ours — so the admitted case is admitted by agreement, not by a stub's gift.
      *'.head_branch'*) echo "${STUB_RUN_REF:-$(cat "${STUB_LAST_REF:-/dev/null}" 2>/dev/null)}" ;;
      *) echo "" ;;
    esac ;;
  *) exit 0 ;;
esac
STUB
chmod +x "$BIN/gh"

# THE GIT STUB. dispatch_stage CREATES the per-invocation dispatch ref itself (probe, local tag, push, verify),
# so the witness answers for git and logs every call. Knobs:
#   STUB_TAG_EXISTS=1        an object of that ref name is already on the remote — a collision, so a refusal
#   STUB_FAIL_TAG_PUSH=1     the push of the ref fails, which means nothing was dispatched
#   STUB_TAG_RESOLVES=<sha>  what the post-push probe answers (default: the revision we asked to pin)
#   STUB_NOT_A_COMMIT=1      the local revision does not resolve to a commit
cat > "$BIN/git" <<'GITSTUB'
#!/usr/bin/env bash
printf '%s\n' "git $*" >> "${GIT_CALLS:-/dev/null}"
clean=(); skip=0
for a in "$@"; do
  if [ "$skip" = 1 ]; then skip=0; continue; fi
  if [ "$a" = "-C" ]; then skip=1; continue; fi
  clean+=("$a")
done
sub="${clean[0]:-}"; rest="${clean[*]:-}"
case "$sub" in
  rev-parse) [ -n "${STUB_NOT_A_COMMIT:-}" ] && exit 1; exit 0 ;;
  tag)       exit 0 ;;
  push)      [ -n "${STUB_FAIL_TAG_PUSH:-}" ] && exit 1
             printf '%s\n' "$rest" >> "${GIT_PUSHES:-/dev/null}"
             # WHICH ref, recorded separately: the three-stage transaction pushes three different refs, and the
             # pre-push collision probe must answer per ref — otherwise stage 2 sees stage 1's push and refuses a
             # transaction that is proceeding exactly as written.
             for a in "${clean[@]:1}"; do case "$a" in refs/tags/*) printf '%s\n' "$a" >> "${GIT_PUSHED_REFS:-/dev/null}" ;; esac; done
             exit 0 ;;
  ls-remote)
    # The probe BEFORE the push answers empty unless the case says a ref of that name is already there; the probe
    # AFTER it answers what the ref resolves to. Which one this is, the stub reads from whether a push has run —
    # so a runbook that verified before pushing gets the empty answer and must refuse.
    q=""
    for a in "${clean[@]:1}"; do case "$a" in refs/*) q="$a" ;; esac; done
    case "$q" in
      refs/*)
        # A ref nobody pushed does not exist: the probe before the push answers empty, the probe after it answers
        # what the ref resolves to. STUB_TAG_EXISTS forces the collision for the case that needs one.
        if [ -n "${STUB_TAG_EXISTS:-}" ] || grep -qx -- "$q" "${GIT_PUSHED_REFS:-/dev/null}" 2>/dev/null; then
          printf '%s\t%s\n' "${STUB_TAG_RESOLVES:-${STUB_WANTED_SHA:-$(printf 'a%.0s' {1..40})}}" "$q"
        fi
        exit 0 ;;
      # a BRANCH query is the activation gate asking where the event branch stands now
      *) printf '%s\trefs/heads/%s\n' "${STUB_TIP:-${STUB_WANTED_SHA:-$(printf 'a%.0s' {1..40})}}" "${q#origin/}"; exit 0 ;;
    esac ;;
esac
exit 0
GITSTUB
chmod +x "$BIN/git"

# THE SUDO / SYSTEMD-RUN STAND-IN. RUNE is `sudo systemd-run --wait --collect --pipe --uid=… -p MemoryMax=24G
# --setenv=K=V … bash -lc "$CMD"`. Dropping the unit flags and running the command here is the whole point: the
# --setenv= values become the child's ENVIRONMENT, exactly as systemd would, so the case sees whether the runbook's
# data reaches the verifier as data. A sudo that quietly discarded --setenv= would hide the very defect F2 was about.
cat > "$BIN/sudo" <<'SUDOSTUB'
#!/usr/bin/env bash
keep=(); skip=0
for x in "$@"; do
  if [ "$skip" = 1 ]; then skip=0; continue; fi
  case "$x" in
    systemd-run|--wait|--collect|--pipe) ;;
    -p) skip=1 ;;
    --uid=*) ;;
    --setenv=*) keep+=("${x#--setenv=}");;
    *) keep+=("$x") ;;
  esac
done
if [ ${#keep[@]} -eq 0 ]; then echo "sudo stub: nothing left to run" >&2; exit 1; fi
env "${keep[@]}"
SUDOSTUB
chmod +x "$BIN/sudo"

# The inner script is the runbook's own function plus the runbook's own call, run with sleep stubbed out so the
# retry loop and its 15-minute cap are exercised without waiting ten seconds a turn.
# Lifted from the runbook itself; empty means the runbook stopped declaring the fence, and the witness says so.
LOCK_LINE=$(grep -E '^STAGE_LOCK=' "$RB" || true)
[ -n "$LOCK_LINE" ] || { echo "WITNESS REFUSED: the runbook declares no STAGE_LOCK, so dispatch_stage has no exclusive boundary"; exit 1; }
T6_LINES=$(grep -E '^T6_NONCE=|^: "\$\{T6_NONCE:|^T6_REF_BASE=' "$RB" || true)
[ -n "$T6_LINES" ] || { echo "WITNESS REFUSED: the runbook mints no per-invocation dispatch reference, so a run adopted at a per-revision tag is somebody's run, not ours"; exit 1; }
cat > "$TMP/inner.sh" <<INNER
sleep() { :; }
eval "\$(sed -n '/^dispatch_stage() {/,/^}/p' "$RB")"
# The fence path comes from THE RUNBOOK, not from this witness — the line below is lifted verbatim from the
# dispatch block, so the witness cannot accidentally author the boundary it is testing. STUB_NO_LOCK removes it
# (the guard must then refuse), and STUB_LOCK_PATH relocates it (so another process can hold it).
if [ -z "\${STUB_NO_LOCK:-}" ]; then
$LOCK_LINE
fi
if [ -n "\${STUB_LOCK_PATH:-}" ]; then STAGE_LOCK=\${STUB_LOCK_PATH}; fi
REPO=gunb-ai/gunbc
SHA="\$SHA"
SHA="\$SHA"
export STUB_WANTED_SHA="\$SHA"
# The per-invocation dispatch reference is lifted FROM THE RUNBOOK (the same three lines the operator shell
# runs), so this witness cannot author the identity it is testing. STUB_NO_NONCE withholds it (the guard must
# refuse), STUB_REF_BASE replaces it (so a reference naming another revision is caught).
if [ -z "\${STUB_NO_NONCE:-}" ]; then
$T6_LINES
fi
if [ -n "\${STUB_REF_BASE:-}" ]; then T6_REF_BASE=\${STUB_REF_BASE}; fi
LOGS=$TMP/logs
install -d "\$LOGS"
dispatch_stage plan plan_artifact_hash=abc
INNER

SHA=$(printf 'a%.0s' {1..40})
FAILED=0
run_case() { # $1 = label, $2 = admit|refuse, $3 = the refusal the runbook must name (empty for admit), rest = STUB_* overrides
  local label="$1" want="$2" say="$3"; shift 3
  local STUB_LOCK_HELD="" STUB_EXPECT_LIST_CALLS="" STUB_LOCK_PATH="" STUB_EXPECT_REF_FILTER="" kv
  # The case's own overrides are visible HERE too, not only in the child: the fence holder and the call-count
  # assertion are decided by the witness, and a case that sets them for the child alone would silently be a case
  # that does nothing.
  for kv in "$@"; do
    case "$kv" in
      STUB_LOCK_HELD=*|STUB_EXPECT_LIST_CALLS=*|STUB_LOCK_PATH=*|STUB_EXPECT_REF_FILTER=*) eval "local ${kv%%=*}=\"${kv#*=}\"" ;;
    esac
  done
  local out rc calls want_calls holder=""
  want_calls=${STUB_EXPECT_LIST_CALLS:-}
  : > "$TMP/gh_calls"
  : > "$TMP/git_calls"
  : > "$TMP/git_pushes"
  # THE FENCE IS HELD BY A REAL PROCESS on a real file, so what this case witnesses is the kernel's answer to a
  # contended flock — not a stub that was told to refuse.
  if [ -n "${STUB_LOCK_HELD:-}" ]; then
    ( exec 9>"${STUB_LOCK_PATH:?}"; flock -x 9; sleep 15 ) & holder=$!
    sleep 0.3
  fi
  out=$(env GH_CALLS="$TMP/gh_calls" GIT_CALLS="$TMP/git_calls" GIT_PUSHES="$TMP/git_pushes" GIT_PUSHED_REFS="$TMP/git_pushed_refs" STUB_LAST_REF="$TMP/last_ref" "$@" RB="$RB" SHA="$SHA" bash "$TMP/inner.sh" 2>&1); rc=$?
  [ -n "$holder" ] && kill "$holder" 2>/dev/null
  wait "$holder" 2>/dev/null
  polls=$(grep -c '] - \[' "$TMP/gh_calls" 2>/dev/null); polls=${polls:-0}; polls=$(printf '%s\n' "$polls" | head -1)
  if [ -n "$want_calls" ] && [ "$polls" != "$want_calls" ]; then
    echo "FAIL $label — the GitHub call sequence is wrong: $polls poll(s) issued, expected $want_calls"; FAILED=1; return
  if [ "${STUB_EXPECT_REF_FILTER:-}" = 1 ] && ! grep -q 'headBranch=="' "$TMP/gh_calls"; then
    echo "FAIL $label — the poll never named the dispatch ref; a run dispatched from another ref would have been adopted as ours"; FAILED=1; return
  fi
  fi
  if [ "$want" = refuse ] && [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -qF -- "$say"; then
    echo "OK   $label — refused (rc=$rc): $(printf '%s' "$out" | grep -F -- "$say" | head -1)"
  elif [ "$want" = admit ] && [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'STAGE plan run='; then
    echo "OK   $label — admitted: $(printf '%s' "$out" | head -1)"
  else
    echo "FAIL $label rc=$rc wanted=$want said=[$say] :: $(printf '%s' "$out" | head -2)"; FAILED=1
  fi
}

run_case "the dispatched run is identified and admitted" admit "" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the newest run declares another mode" refuse "declares mode=sweep, expected plan" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=sweep STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the run resolves to another revision" refuse "is at $(printf 'b%.0s' {1..40}), expected" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$(printf 'b%.0s' {1..40})" STUB_CONCLUSION=success
run_case "no new run appeared after the dispatch" refuse "no run appeared for mode=plan" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS="20" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a pre-existing run cannot impersonate the dispatch" refuse "no run appeared for mode=plan" \
  STUB_OLD_IDS="20 21" STUB_MATCH_IDS="20 21" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the dispatched run did not conclude success" refuse "did not complete successfully" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=failure

# THE CORRELATION ITSELF (review 93394 F4). Each of these, without the guard, silently adopts a run that this
# invocation did not create, and the mode/head_sha assertions below it would still have passed — because the run
# really is at that revision and in that mode. That is why they are refused here.
run_case "a snapshot command that fails is refused" refuse "the pre-dispatch run snapshot" \
  STUB_FAIL_SNAPSHOT=1 STUB_MATCH_IDS="20" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a snapshot that comes back empty is refused" refuse "is empty" \
  STUB_EMPTY_SNAPSHOT=1 STUB_MATCH_IDS="20" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a dispatch already in flight at the revision is refused" refuse "queued" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_LIVE=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the in-flight check failing is refused" refuse "the in-flight check" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_FAIL_LIVE=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "two new runs at one revision are refused (no guessing which is ours)" refuse "two dispatches are racing" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS="21 22" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
# A run sitting in a state the FILTER DOES NOT NAME must still be found. This is the whole point of naming the
# population inside the query: `--status queued --status in_progress` twice proved nothing, and a waiting run
# walked straight into the poll and was adopted.
run_case "a run waiting for a runner is found by the in-flight check" refuse "waiting" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_LIVE_STATUSES="waiting" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a run awaiting a decision is found by the in-flight check" refuse "action_required" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_LIVE_STATUSES="action_required" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a run requested but not started is found by the in-flight check" refuse "requested" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_LIVE_STATUSES="requested" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success

# A dispatch that failed dispatched NOTHING. Polling after it can only find somebody else's run, so the refusal
# must come BEFORE any poll — asserted by counting the run-list calls: exactly one, the snapshot.
run_case "a failed dispatch is refused and never polled" refuse "nothing was dispatched" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success STUB_FAIL_DISPATCH=1 STUB_EXPECT_LIST_CALLS=0

# THE EXCLUSIVE BOUNDARY. With the fence held by another process on this host, this invocation must refuse rather
# than queue into a second run; and with no fence path at all it must refuse rather than run unguarded. The held
# case uses a REAL flock on a real file — not a stubbed refusal — so what is witnessed is the kernel's answer.
run_case "a fence held by another dispatch is refused" refuse "holds $TMP/held.lock on this host" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success STUB_LOCK_PATH="$TMP/held.lock" STUB_LOCK_HELD=1
run_case "no fence path at all is refused" refuse "the exclusive-dispatch fence would have no path" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success STUB_NO_LOCK=1
run_case "a poll that fails is refused" refuse "the poll for mode=plan failed" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_FAIL_POLL=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success

# WHOSE RUN IS IT? (review 93394, round nine, F4.) The fence answers two shells on ONE host. A dispatch from
# another host — or another operator shell with another fence path — cannot be blocked by it, and its run agrees
# on everything else the poll checks: same revision, same mode, genuinely a workflow_dispatch. Identity therefore
# comes from the ref THIS invocation created and pushed itself, and these cases are what prove that it is used.
run_case "a run at the same revision and mode from another ref is never adopted" refuse "no run appeared for mode=plan" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success \
  STUB_RUN_REFS="21=t6-cutover-$SHA-ffffffffffffffff-plan" STUB_EXPECT_REF_FILTER=1
run_case "a run whose own record names another ref is refused even when handed to us" refuse "not the ref this invocation created" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success \
  STUB_IGNORE_REF_FILTER=1 STUB_RUN_REF=t6-cutover-$SHA-ffffffffffffffff-plan
run_case "a dispatch ref that already exists is refused before anything is pushed" refuse "already exists" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success STUB_TAG_EXISTS=1
run_case "a dispatch ref that cannot be pushed is refused and nothing is dispatched" refuse "could not be pushed" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success \
  STUB_FAIL_TAG_PUSH=1 STUB_EXPECT_LIST_CALLS=0
run_case "a dispatch ref resolving to another revision is refused" refuse "not $SHA" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success \
  STUB_TAG_RESOLVES="$(printf 'b%.0s' {1..40})"
run_case "no per-invocation reference is refused" refuse "the per-invocation dispatch reference was never minted" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success STUB_NO_NONCE=1
run_case "a reference that does not name this revision is refused" refuse "not derived from this revision" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success \
  STUB_REF_BASE="t6-cutover-$(printf 'b%.0s' {1..40})-ffffffffffffffff"

echo

echo
echo "================ THE WHOLE ACTIVATION PRELUDE, EXECUTED ================="
# Every case above runs dispatch_stage on its own. The step-5b fence is MORE than that function: five gate
# predicates, the nonce minting, the exclusive fence, three dispatches, and the plan hash carried from the first to
# the other two — and those surrounding lines, not the function, are what the operator's shell actually runs at
# activation time. An earlier draft of this runbook left a tag prelude behind that read a $TAG nothing ever
# assigned: under `set -u` the activation dies with an unbound variable at 04:00 on srv1, and without it the same
# line becomes `git tag -f ""` against an empty name. Neither is visible to a case that lifts the function out.
# So this section lifts the ENTIRE fence out of the runbook and executes it, under `set -u` on purpose.
prelude_fail() { echo "FAIL $1"; FAILED=1; }   # same reporter the cases above use, defined here so order cannot break it

# Fences are lifted WHOLE, by position — never by a pattern that can run past a closing line. An earlier draft took
# the helper block with `sed -n '/^RUNE() {/,/^}/p'`; RUNE's body ends `…; }` on the same line, so no `^}` ever
# matched and the "helper block" ran on through the runbook's prose until some unrelated brace. That is the same
# class of defect this whole section exists to catch, found in the catcher.
fence_at() { # $1 = 1-based index of the ```bash fence to lift, body verbatim
  awk -v want="$1" '/^[[:space:]]*```bash$/ { n++; on = 1; next }
                    on && /^[[:space:]]*```$/ { on = 0; next }
                    on && n == want { print }' "$RB"
}
HELPERS=$(fence_at 1)                       # RUNF, RUNE, RUN and the pinned BIN/TREE conventions
FENCE=$(awk '/^[[:space:]]*```bash$/ { on = 1; b = ""; next }
             on && /^[[:space:]]*```$/ { on = 0; if (b ~ /dispatch_stage\(\) \{/) printf "%s", b; next }
             on { b = b $0 "\n" }' "$RB")
[ -n "$HELPERS" ] || { echo "WITNESS REFUSED: the runbook's first fence (RUNF/RUNE/RUN) could not be lifted, so the activation gate cannot be executed with its own helpers"; exit 1; }
[ -n "$FENCE" ] || { echo "WITNESS REFUSED: the runbook carries no step-5b fence containing dispatch_stage — there is no activation prelude left to execute"; exit 1; }
printf '%s\n' "$HELPERS" | grep -q 'RUNE()' \
  || { echo "WITNESS REFUSED: the lifted helper fence defines no RUNE, so the gate's live re-verification could not really run"; exit 1; }
# STEP 5'S OWN WORDS, not a copy of them. The admission this witness hands step 5b is produced by EXECUTING the
# lines the runbook's step 5 uses to name a verification and write its admission, lifted from that fence by line
# position and refused if absent. A fixture that hand-wrote the admission in its own format could drift from the
# producer and still pass; this one cannot, because the format is the runbook's text on both sides of the edge.
S5_AT=$(grep -n 'the only state from which step 5b may be entered' "$RB" | head -1 | cut -d: -f1)
[ -n "$S5_AT" ] || { echo "WITNESS REFUSED: the runbook no longer carries step 5's admission line, so neither its mint nor its admission can be lifted and the consumer would be tested against a fixture the producer does not write"; exit 1; }
# The mint comes FIRST in step 5 — that ordering is the whole point — so the anchors are located globally and then
# checked to sit inside step 5's own fence, rather than searched forward from the admission line.
MINT_FROM=$(grep -n '^VERIFY_ATTEMPT=$(printf' "$RB" | head -1 | cut -d: -f1)
DESTROY_FROM=$(grep -n '^if \[ -e "\$D/verify.attempt" \]' "$RB" | head -1 | cut -d: -f1)
ADMIT_FROM=$(grep -n "^  { echo 'MIGRATION VERIFIED'" "$RB" | head -1 | cut -d: -f1)
ADMIT_TO=$(grep -n '^  } > "\$D/migration\.verified"$' "$RB" | head -1 | cut -d: -f1)
[ -n "$MINT_FROM" ] && [ -n "$DESTROY_FROM" ] && [ -n "$ADMIT_FROM" ] && [ -n "$ADMIT_TO" ] \
  || { echo "WITNESS REFUSED: step 5's mint, one-act destruction or admission anchors are not where the runbook is supposed to define them ($RB:$S5_AT) — the lift is refused rather than hand-written"; exit 1; }
DESTROY_TO=$(awk -v s="$DESTROY_FROM" 'NR>s && /^fi$/ {print NR; exit}' "$RB")
PUBLISH_FROM=$(grep -n '^printf .*"\$VERIFY_ATTEMPT" > "\$D/verify.attempt"' "$RB" | head -1 | cut -d: -f1)
# The publish is TWO refusals — the write, and the read-back that must agree with it. Lift through the last one
# before the admission, or the witness would certify a publish block that had quietly lost its read-back.
PUBLISH_TO=$(awk -v s="${PUBLISH_FROM:-0}" -v e="${ADMIT_FROM:-0}" 'NR>=s && NR<e && /exit 1; \}/ {n=NR} END{print n}' "$RB")
[ -n "$DESTROY_TO" ] && [ -n "$PUBLISH_FROM" ] && [ -n "$PUBLISH_TO" ] \
  || { echo "WITNESS REFUSED: step 5's destruction act or its publish-and-read-back could not be lifted from $RB (destroy=$DESTROY_FROM..${DESTROY_TO:-?} publish=${PUBLISH_FROM:-?}..${PUBLISH_TO:-?}) — the order this witness certifies would be the fixture's, not the producer's"; exit 1; }
FENCE_START=$(awk -v a="$ADMIT_FROM" 'NR<a && /^[[:space:]]*```bash$/ {n=NR} END{print n+0}' "$RB")
# THE ORDER, CHECKED AND NOT ASSUMED: mint, then ONE act that destroys the previous authorization, then publish
# this attempt's name, then the admission. Publish-before-destroy is the defect this ordering exists to prevent, so
# a runbook that reordered these lines has to make THIS witness fail rather than let the consumer cases pass on a
# producer that no longer carries the guarantee.
[ "$MINT_FROM" -gt "$FENCE_START" ] && [ "$MINT_FROM" -lt "$DESTROY_FROM" ] \
  && [ "$DESTROY_TO" -lt "$PUBLISH_FROM" ] && [ "$PUBLISH_TO" -lt "$ADMIT_FROM" ] \
  && [ "$ADMIT_FROM" -lt "$ADMIT_TO" ] && [ "$ADMIT_TO" -lt "$S5_AT" ] \
  || { echo "WITNESS REFUSED: step 5's lines are not in the order it must keep (fence=$FENCE_START mint=$MINT_FROM destroy=$DESTROY_FROM..$DESTROY_TO publish=$PUBLISH_FROM..$PUBLISH_TO admission=$ADMIT_FROM..$ADMIT_TO admit-line=$S5_AT) — the previous attempt's documents have to be destroyed BEFORE this attempt publishes its name, or a failed publication leaves the old name beside the old admission"; exit 1; }
MINT_LINES=$(sed -n "${MINT_FROM}p" "$RB")
DESTROY_LINES=$(sed -n "${DESTROY_FROM},${DESTROY_TO}p" "$RB")
PUBLISH_LINES=$(sed -n "${PUBLISH_FROM},${PUBLISH_TO}p" "$RB")
ADMIT_LINES=$(sed -n "${ADMIT_FROM},${ADMIT_TO}p" "$RB")
[ -n "$MINT_LINES" ] && [ -n "$DESTROY_LINES" ] && [ -n "$PUBLISH_LINES" ] && [ -n "$ADMIT_LINES" ] \
  || { echo "WITNESS REFUSED: one of step 5's four blocks (mint, one-act destruction, publish, admission) lifted empty from $RB:$S5_AT — a copy would let the consumer pass against a producer that does not exist"; exit 1; }
# The destruction has to be ONE act naming all three documents. A loop that deletes them one at a time stops at its
# first failure and leaves the pair intact, which is the defect the one-act form closes.
[ "$(printf '%s\n' "$DESTROY_LINES" | grep -c 'rm -f --')" = 1 ] \
  || { echo "WITNESS REFUSED: step 5's destruction is not one rm act ($(printf '%s\n' "$DESTROY_LINES" | grep -c 'rm -f --') of them) — a previous attempt's name and the document naming it have to go together or not at all"; exit 1; }
for doc in '$D/verify.attempt' '$D/migration.verified' '$D/read-back.out'; do
  printf '%s\n' "$DESTROY_LINES" | grep -qF "$doc" \
    || { echo "WITNESS REFUSED: step 5's one destruction act does not name $doc — the consumer would be trusting a document the producer never agreed to destroy"; exit 1; }
done
[ "$(printf '%s\n' "$DESTROY_LINES" | grep -c 'for stale in ')" = 1 ] \
  && [ "$(printf '%s\n' "$DESTROY_LINES" | grep -cE '^[[:space:]]*done$')" = 1 ] \
  || { echo "WITNESS REFUSED: step 5's destruction act does not re-check every operand it claims to have removed ($(printf '%s\n' "$DESTROY_LINES" | grep -c 'for stale in ') absence loops) — an rm that reports success and leaves a file behind must be caught here, not by step 5b"; exit 1; }
printf '%s\n' "$PUBLISH_LINES" | grep -q '> "\$D/verify.attempt"' \
  && printf '%s\n' "$PUBLISH_LINES" | grep -q 'cat "\$D/verify.attempt"' \
  || { echo "WITNESS REFUSED: step 5 publishes its verification name without reading it back — a write that fails while an old file survives would then be indistinguishable from a fresh attempt"; exit 1; }
printf '%s\n' "$ADMIT_LINES" | grep -q 'ATTEMPT=$VERIFY_ATTEMPT' \

PRE_CMD=$(grep -m1 -- '--function roadmap_event_log_read_back_cli' "$RB")
PRE_MANIFEST=$(grep -m1 '^MANIFEST=' "$RB")
PRE_ROSTER=$(grep -m1 '^ROSTER=' "$RB")
[ -n "$PRE_CMD" ] && [ -n "$PRE_MANIFEST" ] && [ -n "$PRE_ROSTER" ] \
  || { echo "WITNESS REFUSED: the runbook no longer defines the step-5 verifier command, manifest and roster the prelude inherits — the gate cannot be executed, only described"; exit 1; }

TIPV=$(printf 'd%.0s' {1..40})   # where the event branch stands, as the stub git will answer and the receipt records

run_prelude() { # $1 label, $2 admit|refuse, $3 expected diagnostic, rest: STUB_* overrides and SKIP_VERIFIED=1
  local label="$1" want="$2" say="$3"; shift 3
  local PD=$TMP/prelude PDIR=$TMP/prelude/D RCT=$TMP/prelude_rune_calls skipv="" stale="" noatt="" unatt="" nopub="" forged="" kv
  for kv in "$@"; do case "$kv" in
    SKIP_VERIFIED=1) skipv=1 ;;
    STALE_ADMISSION=1) stale=1 ;;
    NO_VERIFY_ATTEMPT=1) noatt=1 ;;
    UNDELETABLE_ATTEMPT=1) unatt=1 ;;
    UNWRITABLE_ATTEMPT=1) nopub=1 ;;
    FORGED_ADMISSION=1) forged=1 ;;
  esac; done
  rm -rf "$PD"; install -d "$PD" "$PDIR/branch"
  : > "$RCT"; : > "$TMP/prelude_state"; : > "$TMP/prelude_gh"; : > "$TMP/prelude_git"
  : > "$TMP/prelude_pushes"; : > "$TMP/prelude_pushed"; : > "$TMP/prelude_last_ref"
  # The stub the pinned binary stands in for. It records the argv it was handed and MUST NOT touch read-back.out:
  # the gate hashes that file before it re-verifies, so a stub that rewrote it would witness nothing.
  cat > "$PD/gunbc-stub" <<'STUBBED'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${PRELUDE_RUNE_CALLS:-/dev/null}"
printf 'READ-BACK OK the carrier agrees with the seed\n'
exit 0
STUBBED
  chmod +x "$PD/gunbc-stub"
  # what step 1 exported and step 5 admitted, in the shape the prelude expects to find them
  printf '%s\n' '{"kind":"acceptance_recorded","node":"roadmap-node-1","pr_url":"https://github.com/gunb-ai/gunbc/pull/1601","merged_at":"2026-11-05T09:00:00Z","head_sha":"'"$(printf 'a%.0s' {1..40})"'","author":"principal:google-oidc/accounts/116084671989231734979"}' > "$PDIR/seed.jsonl"
  printf 'READ-BACK OK 1 record(s) agree with the seed\n' > "$PDIR/read-back.out"
  # The roster step 4 iterated and step 5 checks the read-back against, in the shape step 0's precheck emits it:
  # one node id per line, taken from the model. Step 5's own guard refuses an unreadable roster, so a prelude that
  # left it out would answer every consumer case with that refusal instead of with the gate.
  model_retirement_ids > "$PDIR/cutover.retire.roster"
  [ -s "$PDIR/cutover.retire.roster" ] \
    || { echo "WITNESS REFUSED: the model answered no retirement node for the prelude's roster ($ELOG) — the gate would run against nothing"; exit 1; }
  # The verification is named and its admission written by the runbook's own lines, run under the same stub git the
  # prelude sees, so TIP_NOW and READ_BACK_SHA256 are computed the way the producer computes them — not pasted.
  # The producer, run by the producer's own lines. `entry` = mint, destroy the previous authorization in one act,
  # publish this attempt's name; `admit` adds the admission that names it. ONE shell for the whole entry: the
  # publish and the admission both expand $VERIFY_ATTEMPT, so running the lifted blocks in separate shells would
  # write an empty name and every consumer case below would pass for the wrong reason.
  produce_admission() { # $1 = entry | admit
    local script="$MINT_LINES
$DESTROY_LINES
$PUBLISH_LINES"
    if [ "$1" = admit ]; then
      script="$script
$ADMIT_LINES"
    fi
    env PATH="$BIN:$PATH" D="$PDIR" SHA="$SHA" BIN="$PD/gunbc-stub" TREE="$PD/tree" \
        STUB_TIP="$TIPV" GIT_CALLS="$TMP/prelude_git_producer" bash -c "set -u
$script" \
      || { echo "WITNESS BUG: step 5's own entry lines refused in the fixture, so no consumer case below means anything"; exit 1; }
    [ "$1" = admit ] || return 0
    [ -s "$PDIR/verify.attempt" ] || { echo "WITNESS BUG: the producer wrote no verification attempt"; exit 1; }
    [ "$(sed -n 's/^ATTEMPT=//p' "$PDIR/migration.verified")" = "$(head -n1 "$PDIR/verify.attempt")" ] \
      || { echo "WITNESS BUG: the producer's own admission does not name its own attempt — the runbook's step 5 text is broken, not this witness"; }
  }
  # A pass-through `rm` that refuses exactly one named target. Everything else about it matches the real rm — every
  # operand attempted, non-zero status if any failed — because the runbook's one-act destruction depends on that
  # behaviour and a stub that stopped at the first refusal would certify a guarantee the real rm does not give.
  rmstub() {
    install -d "$TMP/rmstub"
    cat > "$TMP/rmstub/rm" <<'RMSTUB2'
#!/usr/bin/env bash
flags=(); targets=(); rc=0
for a in "$@"; do case "$a" in --) ;; -*) flags+=("$a");; *) targets+=("$a");; esac; done
for p in "${targets[@]}"; do
  if [ -n "${STUB_RM_FAIL_ON:-}" ] && [[ "$p" == *"$STUB_RM_FAIL_ON"* ]]; then
    echo "rm: cannot remove '$p': Operation not permitted" >&2; rc=1; continue
  fi
  /usr/bin/rm "${flags[@]}" -- "$p" || rc=1
done
exit "$rc"
RMSTUB2
    chmod 0755 "$TMP/rmstub/rm"
  }
  # The re-entry, performed by the runbook's own mint-and-destroy lines (the publish is NOT in them, which is the
  # point: an entry that cannot destroy the previous authorization must not publish a name either).
  re_enter() { # $1 = the rm target that will refuse
    rmstub
    local out rc=0
    out=$(env PATH="$TMP/rmstub:$BIN:$PATH" D="$PDIR" STUB_RM_FAIL_ON="$1" \
        bash -c "set -u
$MINT_LINES
$DESTROY_LINES" 2>&1) || rc=$?
    [ "$rc" -ne 0 ] \
      || { echo "WITNESS BUG: the refused step 5 re-entry exited 0 — it did not refuse the deletion it was told to refuse, so its consumer state is not the state under test: [$out]"; exit 1; }
    echo "$out" | grep -q 'MIGRATION REFUSED' \
      || { echo "WITNESS BUG: the refused step 5 re-entry exited $rc without its refusal diagnostic: [$out]"; exit 1; }
  }
  if [ "$stale" = 1 ]; then
    # THE PRODUCER'S OWN REFUSAL STATE, produced rather than described. Step 5 runs to completion (name A, admission
    # naming A). It is then entered again and its `rm` refuses `migration.verified`: the act fails, the admission
    # stays, and everything the act COULD remove — this attempt's published name among it — is gone. That is the
    # reviewer's publication scenario: an admission that cannot be destroyed, and no name left standing beside it.
    produce_admission admit
    re_enter 'migration.verified'
    [ ! -e "$PDIR/verify.attempt" ] \
      || { echo "WITNESS BUG: the refused re-entry left a published name standing, so the consumer would be facing a live pair rather than a broken one"; exit 1; }
    [ -s "$PDIR/migration.verified" ] \
      || { echo "WITNESS BUG: the injected failure never happened — the admission this case says cannot be destroyed was destroyed"; exit 1; }
  elif [ "$unatt" = 1 ]; then
    # The mirror: the NAME cannot be destroyed. The same act takes the admission and the read-back with it, so the
    # consumer faces an orphaned name with no document left to match it.
    produce_admission admit
    re_enter 'verify.attempt'
    [ -s "$PDIR/verify.attempt" ] \
      || { echo "WITNESS BUG: the injected failure never happened — the attempt name this case says cannot be destroyed was destroyed"; exit 1; }
    [ ! -e "$PDIR/migration.verified" ] \
      || { echo "WITNESS BUG: the admission survived the act that was supposed to take it, so step 5b would find a pair again"; exit 1; }
  elif [ "$nopub" = 1 ]; then
    # A publication that cannot be performed: with the previous authorization already destroyed by the successful
    # entry above, the name's path is occupied by something `printf >` cannot write. The publish block must refuse,
    # and nothing may be left that step 5b could read as a verification.
    produce_admission admit
    # Only the NAME's path is made unwritable; the admission from the completed entry stays, so the consumer is
    # facing the state the reviewer described — a verification document with nothing behind it — rather than the
    # missing-admission gate it would hit first if this fixture cleared both.
    rm -f -- "$PDIR/verify.attempt" \
      || { echo "WITNESS BUG: the fixture could not clear the published name before the publication injection"; exit 1; }
    mkdir -p "$PDIR/verify.attempt" \
      || { echo "WITNESS BUG: the fixture could not put an unwritable object at the attempt name's path"; exit 1; }
    local pub_rc=0
    env PATH="$BIN:$PATH" D="$PDIR" bash -c "set -u
VERIFY_ATTEMPT=f00df00df00df00d
$PUBLISH_LINES" >/dev/null 2>&1 || pub_rc=$?
    [ "$pub_rc" -ne 0 ] \
      || { echo "WITNESS BUG: step 5's publish block accepted a name it could not write — the read-back is not being performed"; exit 1; }
  elif [ "$forged" = 1 ]; then
    # A pair the producer cannot produce: the admission from a completed step 5 and a name nobody minted. The
    # consumer's equality predicate is what stands between that pair and a deployment, so it needs a case — and the
    # honest way to give it one is to say the state is forged rather than to pretend a refusal can leave it behind.
    produce_admission admit
    rm -f -- "$PDIR/verify.attempt" \
      || { echo "WITNESS BUG: the fixture could not clear the published name before forging it"; exit 1; }
    printf '%s\n' 'deadbeefdeadbeef' > "$PDIR/verify.attempt" \
      || { echo "WITNESS BUG: the fixture could not forge a verification name"; exit 1; }
  elif [ "$noatt" = 1 ]; then
    produce_admission admit
    rm -f -- "$PDIR/verify.attempt" || { echo "WITNESS BUG: the fixture could not remove the attempt file"; exit 1; }
  elif [ "$skipv" != 1 ]; then
    produce_admission admit
  else
    # A step 5 that named its verification and never reached an admission: the entry lines run, the admission does
    # not. Minting here is what keeps this case from passing on the wrong gate.
    produce_admission entry
  fi
  # set -u is the point; sleep is neutralised; the helpers and the inherited step-5 lines come from the runbook itself
  { printf 'set -u\n'
    printf 'sleep() { :; }\n'
    printf '%s\n' "$HELPERS"
    # The preamble's four operator slots, filled as the operator fills them before the first fence: the staged
    # revision, and the export directory steps 1 and 5 wrote. BIN and TREE point into this sandbox because the
    # pinned binary is a stub here — that substitution is the whole of what is fake about this run.
    printf 'SHA=%s\n' "$SHA"
    printf 'D=%s\n' "$PDIR"
    printf 'BIN=%s\n' "$PD/gunbc-stub"
    printf 'TREE=%s\n' "$PD/tree"
    printf '%s\n' "$PRE_MANIFEST"
    printf '%s\n' "$PRE_ROSTER"
    printf '%s\n' "$PRE_CMD"
    printf '%s\n' "$FENCE"; } > "$PD/whole.sh"
  local out rc=0
  out=$(env PATH="$BIN:$PATH" GH_CALLS="$TMP/prelude_gh" GIT_CALLS="$TMP/prelude_git" \
      GIT_PUSHES="$TMP/prelude_pushes" GIT_PUSHED_REFS="$TMP/prelude_pushed" \
      STUB_LAST_REF="$TMP/prelude_last_ref" STUB_STATE="$TMP/prelude_state" STUB_TIP="$TIPV" \
      STUB_POOL="21 22 23" STUB_MATCH_IDS="21 22 23" STUB_OLD_IDS="20" STUB_SHA="$SHA" \
      STUB_ID_MODES="21=plan 22=apply 23=dashboard_deploy" \
      PRELUDE_RUNE_CALLS="$RCT" RB="$RB" SHA="$SHA" "$@" bash "$PD/whole.sh" 2>&1) || rc=$?
  LAST_OUT="$out"; LAST_RC=$rc; LAST_PD="$PD"; LAST_RCT="$RCT"
  if [ "$want" = admit ]; then
    [ "$rc" -eq 0 ] || prelude_fail "$label: the activation prelude refused (rc=$rc): [$(printf '%s' "$out" | tail -c 400)]"
    [ "$say" = ok ] || prelude_fail "$label: an admitting case named a diagnostic [$say]"
    [ "$FAILED" = 0 ] || return 0     # the caller's own assertions below report the detail; no double-count
    echo "OK $label rc=$rc — the whole fence ran, gates to DEPLOYED"
  else
    [ "$rc" -ne 0 ] || prelude_fail "$label: the activation prelude ADMITTED when it should have refused: [$(printf '%s' "$out" | tail -c 400)]"
    echo "$out" | grep -qF -- "$say" || prelude_fail "$label: refused, but not with [$say]: [$(printf '%s' "$out" | tail -c 400)]"
    [ "$FAILED" = 0 ] || return 0
    echo "OK $label rc=$rc — refused with [$say]"
  fi
}

echo "--- the whole fence, gates and all: three dispatches at one revision, three per-stage refs ---"
run_prelude "the whole activation prelude" admit ok
OUT="$LAST_OUT"
echo "    completed (rc=$LAST_RC), tail: [$(printf '%s' "$OUT" | tail -c 260)]"
for stage in plan apply dashboard_deploy; do
  echo "$OUT" | grep -qF "STAGE $stage run=" \
    || prelude_fail "the prelude never reached the $stage stage — the lines AROUND dispatch_stage are what broke: [$(printf '%s' "$OUT" | tail -c 300)]"
done
echo "$OUT" | grep -qE 'DEPLOYED ref_base=t6-cutover-[0-9a-f]{40}-[0-9a-f]{16} plan=21 apply=22 deploy=23 plan_artifact_hash=[0-9a-f]{64}' \
  || prelude_fail "the transaction did not complete as three distinct identified runs carrying the plan hash: [$(printf '%s' "$OUT" | tail -c 400)]"
[ "$(grep -c 'PINNED DISPATCH REF stage=' <<<"$OUT")" = 3 ] \
  || prelude_fail "the prelude pinned $(grep -c 'PINNED DISPATCH REF stage=' <<<"$OUT") dispatch refs, not one per stage"
[ "$(grep -o 'ref=t6-cutover-[0-9a-f]*-[0-9a-f]*-' <<<"$OUT" | sort -u | wc -l)" = 1 ] \
  || prelude_fail "the three stages did not share ONE per-invocation reference — a nonce per stage is not a nonce per invocation"
# No force-tag route may survive anywhere in the prelude, and every dispatch must name the ref it was pinned to.
grep -qE 'tag -f|push -f' "$TMP/prelude_git" \
  && prelude_fail "the prelude still forces a tag or a push: [$(grep -E 'tag -f|push -f' "$TMP/prelude_git" | head -1)]"
[ "$(grep -c ' tag ' "$TMP/prelude_git")" = 3 ] \
  || prelude_fail "the prelude created $(grep -c ' tag ' "$TMP/prelude_git") tags, expected exactly one per stage"
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 3 ] \
  || prelude_fail "the prelude dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") runs, expected the three stages"
grep -q -- '--ref t6-cutover-' "$TMP/prelude_gh" \
  || prelude_fail "a dispatch went out without --ref naming this invocation's reference"
# and the live re-verification must have run, with the manifest reaching the verifier as DATA through the
# environment — the same route F2 was about, now seen from inside the gate that depends on it.
[ -s "$LAST_RCT" ] || prelude_fail "the prelude's on-the-spot re-verification never reached the binary: RUNE was skipped, so the gate was reading a receipt it did not re-check"
grep -qF -- 'expected_manifest={"kind":"acceptance_recorded"' "$LAST_RCT" \
  || prelude_fail "the manifest did not reach the verifier intact and as data: [$(head -1 "$LAST_RCT")]"

echo "--- the same whole fence with step 5's admission absent: the gate refuses before GitHub is contacted ---"
run_prelude "the activation prelude without step 5's admission" refuse 'step 5 never admitted the read-back' SKIP_VERIFIED=1
echo "    refused (rc=$LAST_RC): [$(printf '%s' "$LAST_OUT" | tail -c 200)]"
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) although step 5 had never admitted the read-back — the gate is prose, not a gate"
[ "$(grep -c ' tag ' "$TMP/prelude_git")" = 0 ] \
  || prelude_fail "it minted a dispatch reference before the gate refused"

echo "--- the same whole fence after a step 5 that REFUSED on a failed destruction: zero dispatches ---"
# The reviewer's publication sequence, produced by the runbook's own lines: a successful verification, then a
# re-entry whose `rm` refuses the admission. The act fails, so step 5 refuses — and the name it had published is
# gone with everything else the act could remove, while the admission it could not remove stands alone. Every other
# predicate the prelude has is satisfied by that surviving document (same revision, same read-back digest, same
# live tip), so only the absence of a name behind it can stop a deployment here.
run_prelude "the activation prelude on an admission whose name its own destruction took" refuse \
  'no verification attempt is recorded' STALE_ADMISSION=1
echo "    refused (rc=$LAST_RC): [$(printf '%s' "$LAST_OUT" | tail -c 220)]"
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) on an admission from a step 5 that refused — activation would have happened behind a refused verification"
[ "$(grep -c ' tag ' "$TMP/prelude_git")" = 0 ] \
  || prelude_fail "it minted a dispatch reference from documents a refused step 5 left behind"
[ ! -s "$TMP/prelude_pushed" ] \
  || prelude_fail "it pushed a ref: [$(cat "$TMP/prelude_pushed")]"

echo "--- the same whole fence when the admission is paired with a name no producer minted: zero dispatches ---"
# The binding predicate's own case. The producer can no longer reach this state — the assertions in the stale mode
# above certify that a refused entry leaves no published name beside the admission it could not remove — so the
# pair is forged here on purpose: it is what the consumer's equality check is for, and a predicate nobody can
# violate is not a predicate.
run_prelude "the activation prelude on a forged admission-and-name pair" refuse \
  'does not name THIS verification' FORGED_ADMISSION=1
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) on an admission whose name was never minted by step 5"
[ ! -s "$TMP/prelude_pushed" ] \
  || prelude_fail "it pushed a ref from a forged pair: [$(cat "$TMP/prelude_pushed")]"

echo "--- the same whole fence when the verification NAME cannot be destroyed: zero dispatches ---"
# The mirror of the case above: the name survives the refused act and the admission does not, so the consumer is
# left with an orphaned name and nothing to match it against.
run_prelude "the activation prelude when only the verification name survived the refused act" refuse \
  'step 5 never admitted the read-back' UNDELETABLE_ATTEMPT=1
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) on an orphaned verification name"

echo "--- the same whole fence when the verification name cannot be published: zero dispatches ---"
# The reviewer's repro class made concrete: the name's path cannot be written, so nothing is published, and the
# admission that survives from the earlier successful entry has no name standing behind it.
run_prelude "the activation prelude when this attempt could not publish its verification name" refuse \
  'no verification attempt is recorded' UNWRITABLE_ATTEMPT=1
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) with a verification name that was never published"
echo "--- the same whole fence with the verification name missing entirely: zero dispatches ---"
# The other way an admission can arrive without a live producer: an export whose admission was copied in and whose
# mint file is not there. Refused with its own words, because the repair differs from the case above — there the
# producer ran and refused, here it never ran at all.
run_prelude "the activation prelude with no verification attempt recorded" refuse \
  'no verification attempt is recorded' NO_VERIFY_ATTEMPT=1
[ "$(grep -c 'workflow run' "$TMP/prelude_gh")" = 0 ] \
  || prelude_fail "it dispatched $(grep -c 'workflow run' "$TMP/prelude_gh") run(s) with no verification attempt recorded"

echo "================ PART B: the cases this file declares are the model's rows, joined both ways ================"
# Same join as PART H of the runbook witness, over the dispatch carrier. The names come from the model: a case
# that runs here with no row, or a row with no case here, is a defect in one of the two and is reported as one.
fail_case() { echo "FAIL $1"; FAILED=1; }   # the file's own reporter: a refused join must reach the exit status
SELF_H=${BASH_SOURCE[0]:-$0}
model_case_rows() { # name|must|refusal_text for every gunbc.witness.roadmap_cutover case row of carrier dispatch_witness_carrier
  awk -v car="dispatch_witness_carrier" '
    /^[[:space:]]*name:/         { n = $0; sub(/^[^"]*"/, "", n); sub(/".*/, "", n) }
    /^[[:space:]]*carrier:/      { c = $0; sub(/^[^:]*:[[:space:]]*/, "", c); sub(/[[:space:]]*$/, "", c) }
    /^[[:space:]]*must:/         { m = $0; sub(/^[^"]*"/, "", m); sub(/".*/, "", m) }
    /^[[:space:]]*refusal_text:/ { r = $0; sub(/^[^"]*"/, "", r); sub(/".*/, "", r)
                                   if (c == car) printf "%s|%s|%s\n", n, m, r }' "$MODEL"
}
declared_cases() { awk '/^# CASES IMPLEMENTED HERE/ {on=1; next}
                       on { if ($0 ~ /^#   [a-z0-9_]+$/) { sub(/^#   /, ""); print; next }
                            if ($0 ~ /^#/) next   # prose between the header and the names is skipped, not fatal
                            exit }' "$SELF_H"
}
ROWS=$(model_case_rows)
DECL=$(declared_cases)
[ -n "$ROWS" ] || fail_case "the join is empty: not one case row could be read from [$MODEL] — an empty read must not pass"
[ -n "$DECL" ] || fail_case "the join is empty: this file declares no cases at all"
while IFS='|' read -r n m r; do
  [ -n "$n" ] || continue
  printf '%s\n' "$DECL" | grep -qx -- "$n" \
    || fail_case "the model demands case [$n] of this carrier and this file does not declare it"
  if [ "$m" = refuse ]; then
    [ -n "$r" ] || fail_case "refusing case [$n] promises no diagnostic"
    grep -qF -- "$r" "$SELF_H" || grep -qF -- "$r" "$RB" \
      || fail_case "case [$n] promises the diagnostic [$r] and neither this witness nor the runbook prints it"
  elif [ "$m" = admit ]; then
    [ -z "$r" ] || fail_case "admitting case [$n] carries a refusal string — which is it?"
  else
    fail_case "case [$n] has must: [$m], which is neither admit nor refuse"
  fi
done <<<"$ROWS"
while read -r n; do
  [ -n "$n" ] || continue
  printf '%s\n' "$ROWS" | cut -d'|' -f1 | grep -qx -- "$n" \
    || fail_case "this file declares case [$n], which the model has never heard of"
done <<<"$DECL"
echo "model rows for this carrier: $(printf '%s\n' "$ROWS" | grep -c .)   declared here: $(printf '%s\n' "$DECL" | grep -c .)"

[ "$FAILED" -eq 0 ] && echo "ALL DISPATCH-IDENTIFICATION CASES HOLD" || echo "A DISPATCH-IDENTIFICATION CASE FAILED"
exit "$FAILED"
