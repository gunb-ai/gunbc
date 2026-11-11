#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: the model is gunbc.witness.roadmap_cutover, and the promise is written there
# (dispatch_witness_dissolve_trigger), not here. Two capabilities retire DIFFERENT halves of this file.
#   (1) orch-emitted, for the RESIDUAL SHELL (the gh stub, the case driver, the run-id set difference): the bash
#       target grammar (v2.extdeps.languages.bash rows, built by v2.extdeps.languages.bash_build, serialized by
#       v2.workflow.bash_emit) has no production for a case statement over glob patterns, an indexed array read
#       back with slice arithmetic ("${args[@]:1}"), a here-doc that materialises a stub executable, or command
#       substitution inside an assignment used as a predicate. That is a missing target-language capability
#       (class (b) of the hand-shell gate), not an unauthored migration.
#   (2) modelled, for the WITNESS PROCESS: the six cases below are rows of
#       gunbc.witness.roadmap_cutover::cutover_witness_case_rows, and dispatch_stage itself is loaded from the
#       runbook rather than copied. When fleet-converge's own admission
#       (gunbc.fleet.fleet_converge_plan_revision::admit_dashboard_deploy_dispatch_inputs) is exercised through
#       modelled test rows in gunbc.instrument_targets, this file is DELETED rather than emitted.
#
# Witness for the cutover runbook's CI-dispatch step (docs/runbooks/roadmap-event-log-cutover.md, step 5b).
#
# It drives the runbook's OWN `dispatch_stage` function — extracted from the runbook file at run time, never copied —
# against a `gh` that answers exactly the queries that function makes. The cases prove the runbook cannot activate
# on a run it did not dispatch: one declaring another mode, one at another revision, one that was already there
# before the dispatch, one that never appeared, and one that did not conclude success all refuse.
#
# Run:  bash tools/roadmap_cutover_dispatch_witness.sh docs/runbooks/roadmap-event-log-cutover.md
set -u
RB=${1:?usage: roadmap_cutover_dispatch_witness.sh <path to the runbook>}
TMP=$(mktemp -d); BIN=$TMP/bin; export PATH=$BIN:$PATH
mkdir -p "$BIN"

cat > "$BIN/gh" <<'STUB'
#!/usr/bin/env bash
# Stands in for gh, answering only what dispatch_stage asks. The "which run did my dispatch create" query is
# emulated honestly — the visible pool minus the ids that existed before the dispatch — so a pre-existing run
# cannot impersonate the dispatched one here any more than it can in production.
case "$1 $2" in
  "workflow run"*) exit 0 ;;
  "run list"*)
    q=""
    for a in "$@"; do q="$q$a"; done
    case "$q" in
      *'.[].databaseId'*) echo "${STUB_OLD_IDS:-}" ;;
      *headSha*)
        best=""
        for id in ${STUB_MATCH_IDS:-}; do
          case " ${STUB_OLD_IDS:-} " in *" $id "*) continue ;; esac
          best="$id"
        done
        echo "$best" ;;
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
run_case() { # $1 = label, $2 = admit|refuse, rest = STUB_* overrides
  local label="$1" want="$2"; shift 2
  local out rc
  out=$(env "$@" RB="$RB" SHA="$SHA" bash "$TMP/inner.sh" 2>&1); rc=$?
  if [ "$want" = refuse ] && [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -qE 'NO DEPLOY|REFUSED'; then
    echo "OK   $label — refused (rc=$rc): $(printf '%s' "$out" | head -1)"
  elif [ "$want" = admit ] && [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'STAGE plan run='; then
    echo "OK   $label — admitted: $(printf '%s' "$out" | head -1)"
  else
    echo "FAIL $label rc=$rc :: $(printf '%s' "$out" | head -2)"; FAILED=1
  fi
}

run_case "the dispatched run is identified and admitted" admit \
  STUB_OLD_IDS=20 STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the newest run declares another mode" refuse \
  STUB_OLD_IDS=20 STUB_MATCH_IDS=21 STUB_MODE=sweep STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the run resolves to another revision" refuse \
  STUB_OLD_IDS=20 STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$(printf 'b%.0s' {1..40})" STUB_CONCLUSION=success
run_case "no new run appeared after the dispatch" refuse \
  STUB_OLD_IDS=20 STUB_MATCH_IDS=20 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "a pre-existing run cannot impersonate the dispatch" refuse \
  STUB_OLD_IDS="20 21" STUB_MATCH_IDS="20 21" STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=success
run_case "the dispatched run did not conclude success" refuse \
  STUB_OLD_IDS=20 STUB_MATCH_IDS=21 STUB_MODE=plan STUB_SHA="$SHA" STUB_CONCLUSION=failure

echo
[ "$FAILED" -eq 0 ] && echo "ALL DISPATCH-IDENTIFICATION CASES HOLD" || echo "A DISPATCH-IDENTIFICATION CASE FAILED"
exit "$FAILED"
