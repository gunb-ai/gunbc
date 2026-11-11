#!/usr/bin/env bash
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
set -u
RB=${1:?usage: roadmap_cutover_dispatch_witness.sh <path to the runbook>}
TMP=$(mktemp -d); BIN=$TMP/bin; export PATH=$BIN:$PATH
mkdir -p "$BIN"

cat > "$BIN/gh" <<'STUB'
#!/usr/bin/env bash
# Stands in for gh, answering only what dispatch_stage asks.
case "$1 $2" in
  "workflow run"*) exit 0 ;;
  "run list"*)
    q=""
    for a in "$@"; do q="$q$a"; done
    case "$q" in
      # The in-flight check: `--json headSha,event` with a `| length` projection.
      *'| length'*) [ -n "${STUB_FAIL_LIVE:-}" ] && exit 1
                    echo "${STUB_LIVE:-0}" ;;
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
cat > "$TMP/inner.sh" <<INNER
sleep() { :; }
eval "\$(sed -n '/^dispatch_stage() {/,/^}/p' "$RB")"
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
  local out rc
  out=$(env "$@" RB="$RB" SHA="$SHA" bash "$TMP/inner.sh" 2>&1); rc=$?
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
run_case "a dispatch already in flight at the revision is refused" refuse "queued or in progress" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_LIVE=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the in-flight check failing is refused" refuse "the in-flight check" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_FAIL_LIVE=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "two new runs at one revision are refused (no guessing which is ours)" refuse "two dispatches are racing" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS="21 22" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a poll that fails is refused" refuse "the poll for mode=plan failed" \
  STUB_OLD_IDS="20" STUB_MATCH_IDS=21 STUB_FAIL_POLL=1 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success

echo
[ "$FAILED" -eq 0 ] && echo "ALL DISPATCH-IDENTIFICATION CASES HOLD" || echo "A DISPATCH-IDENTIFICATION CASE FAILED"
exit "$FAILED"
