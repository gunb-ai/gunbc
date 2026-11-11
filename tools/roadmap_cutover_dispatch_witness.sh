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
#
set -u
RB=${1:?usage: roadmap_cutover_dispatch_witness.sh <path to the runbook>}
TMP=$(mktemp -d); BIN=$TMP/bin; export PATH=$BIN:$PATH
MODEL=${2:-$(cd "$(dirname "$RB")/../.." && pwd)/dag/gunbc/witness/roadmap_cutover.dag}
[ -f "$MODEL" ] || { echo "WITNESS REFUSED: no model at [$MODEL] — the case rows are read from it, not invented here"; exit 1; }
mkdir -p "$BIN"

cat > "$BIN/gh" <<'STUB'
#!/usr/bin/env bash
# Stands in for gh, answering only what dispatch_stage asks. Every call is logged, so a case can assert not just
# the answer but the SEQUENCE — a function that polled after a dispatch that failed is not "refused", it is wrong.
echo "$*" >> "${GH_CALLS:-/dev/null}"
case "$1 $2" in
  "workflow run"*) [ -n "${STUB_FAIL_DISPATCH:-}" ] && exit 1; exit 0 ;;
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
        printf '%s\n' ${STUB_OLD_IDS:-} | grep . || true ;;
      # The poll: `--json databaseId,headSha,event` with `- [ids] | .[]`. The exclusion set is taken FROM THE
      # QUERY the function built (plain parameter expansion; a sed address here is its own little minefield), so
      # the fixture cannot accidentally hand it the answer the function forgot to ask for.
      *'databaseId,headSha,event'*)
        [ -n "${STUB_FAIL_POLL:-}" ] && exit 1
        ex=${q#*'] - ['}; ex=${ex%%']'*}; ex=${ex//,/ }   # jq's list is comma-separated; the cases match on words
        for id in ${STUB_MATCH_IDS:-}; do
          case " $ex " in *" $id "*) continue ;; esac
          printf '%s\n' "$id"
        done ;;
      *) echo "" ;;
    esac ;;
  "run watch"*) [ "${STUB_CONCLUSION:-success}" = success ] && exit 0 || exit 1 ;;
  "api "*)
    case "$*" in
      *'.inputs.mode'*) echo "${STUB_MODE:-}" ;;
      *'.head_sha'*)    echo "${STUB_SHA:-}" ;;
      *) echo "" ;;
    esac ;;
  *) exit 0 ;;
esac
STUB
chmod +x "$BIN/gh"

# The inner script is the runbook's own function plus the runbook's own call, run with sleep stubbed out so the
# retry loop and its 15-minute cap are exercised without waiting ten seconds a turn.
# Lifted from the runbook itself; empty means the runbook stopped declaring the fence, and the witness says so.
LOCK_LINE=$(grep -E '^STAGE_LOCK=' "$RB" || true)
[ -n "$LOCK_LINE" ] || { echo "WITNESS REFUSED: the runbook declares no STAGE_LOCK, so dispatch_stage has no exclusive boundary"; exit 1; }
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
TAG=t6-cutover-\$SHA
LOGS=$TMP/logs
install -d "\$LOGS"
dispatch_stage plan plan_artifact_hash=abc
INNER

SHA=$(printf 'a%.0s' {1..40})
FAILED=0
run_case() { # $1 = label, $2 = admit|refuse, $3 = the refusal the runbook must name (empty for admit), rest = STUB_* overrides
  local label="$1" want="$2" say="$3"; shift 3
  local STUB_LOCK_HELD="" STUB_EXPECT_LIST_CALLS="" STUB_LOCK_PATH="" kv
  # The case's own overrides are visible HERE too, not only in the child: the fence holder and the call-count
  # assertion are decided by the witness, and a case that sets them for the child alone would silently be a case
  # that does nothing.
  for kv in "$@"; do
    case "$kv" in
      STUB_LOCK_HELD=*|STUB_EXPECT_LIST_CALLS=*|STUB_LOCK_PATH=*) eval "local ${kv%%=*}=\"${kv#*=}\"" ;;
    esac
  done
  local out rc calls want_calls holder=""
  want_calls=${STUB_EXPECT_LIST_CALLS:-}
  : > "$TMP/gh_calls"
  # THE FENCE IS HELD BY A REAL PROCESS on a real file, so what this case witnesses is the kernel's answer to a
  # contended flock — not a stub that was told to refuse.
  if [ -n "${STUB_LOCK_HELD:-}" ]; then
    ( exec 9>"${STUB_LOCK_PATH:?}"; flock -x 9; sleep 15 ) & holder=$!
    sleep 0.3
  fi
  out=$(env GH_CALLS="$TMP/gh_calls" "$@" RB="$RB" SHA="$SHA" bash "$TMP/inner.sh" 2>&1); rc=$?
  [ -n "$holder" ] && kill "$holder" 2>/dev/null
  wait "$holder" 2>/dev/null
  polls=$(grep -c '] - \[' "$TMP/gh_calls" 2>/dev/null); polls=${polls:-0}; polls=$(printf '%s\n' "$polls" | head -1)
  if [ -n "$want_calls" ] && [ "$polls" != "$want_calls" ]; then
    echo "FAIL $label — the GitHub call sequence is wrong: $polls poll(s) issued, expected $want_calls"; FAILED=1; return
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

echo
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
