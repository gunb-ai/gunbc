# SCAFFOLD — dissolve-on: TWO capabilities, because they retire DIFFERENT halves of this file.
#   (1) orch-emitted, for the RESIDUAL SHELL (the stubs and the case assertions): the bash target grammar
#       (v2.extdeps.languages.bash rows, built by v2.extdeps.languages.bash_build, serialized by
#       v2.workflow.bash_emit) has no production for what this file still does by hand — a function definition
#       carrying a case statement over globs, a here-doc that writes a stub script, an array assignment read back
#       as "${arr[@]}", and command substitution inside a test predicate. That is a missing target-language
#       capability (class (b) of the hand-shell gate), not an unauthored migration: when those node kinds land,
#       the case list below is emitted from them instead of typed.
#   (2) modelled, for the WITNESS PROCESS ITSELF: the cases are rows of
#       gunbc.witness.roadmap_cutover (cutover_witness_case_rows), which is the authority for what each case
#       feeds and what disposition it must earn; the helpers it exercises are loaded from the runbook, never
#       copied (see "the authority is the runbook" below). When those rows run as a `gunbc test` row in
#       gunbc.instrument_targets through the modeled filesystem and host-effect routes, this file is DELETED
#       rather than emitted.
#
# Witness for the cutover runbook's own shell: docs/runbooks/roadmap-event-log-cutover.md.
#
#   PART A  argument delivery through the ACTUAL helper invocation shape. `bash -s` claims $0 for itself and makes
#           EVERY word after `--` a positional parameter, so a helper written `bash -s -- runbook "$@"` would bind
#           $1 to the literal string "runbook" — step 1 would export into a relative directory of that name and
#           step 7 would read the wrong export. `bash -n` cannot see that class; this can.
#   PART B  the drop-in export/restore round trip, with a PRE-EXISTING drop-in and a DEPLOYMENT-ADDED one.
#   PART C  the join between the runbook's step headings and the model's rows, in BOTH directions: every modelled
#           step is cited by exactly one heading, and every citation names a modelled step. The step ids are read
#           out of gunbc.witness.roadmap_cutover, so neither file can silently grow a step the other does not know.
#   PART D  the rollback's restart leg, EXECUTED against a stateful systemctl stub: the units the export found
#           active really receive `systemctl start`, the ones it found inactive really do not, and an export that
#           is empty, off-roster, or missing a roster unit refuses with the readers still down. D5 proves the
#           fixture can see the defect it was written for: the pasted-second-list shape is REFUSED as an invalid
#           inventory, not merely reported as "RESTORED 0"; D6 and D7 refuse a unit recorded twice and a word
#           that is not a state `systemctl is-active` can print.
#   PART E  the expected manifest arrives byte-for-byte through RUNE's environment and never through argv; E2
#           runs the shape the runbook no longer uses and requires it to arrive CORRUPT, so E1 is not vacuous.
#   PART F  the stop of the readers: all five units AND the timer must report inactive, no gunbc process may
#           remain, and every stop command must have returned successfully — a red case for each way the world can
#           disagree, plus a join of the roster step 1 MEASURES against the roster step 2 ADMITS, so the two lists
#           cannot drift apart silently.
#   PART G  the migration and the retirements: the input is counted against the export BEFORE any append, the first
#           refused append aborts the run, and step 4 refuses to retire a history step 3 never admitted. These are
#           failure injection — a stub CLI that says no, and the requirement that NOTHING ran after it.
#
# Nothing in this file is a second authority: RUNF, UNITS, export_drop_ins, restore_drop_ins and need are loaded
# out of the runbook at run time, so the thing being witnessed is literally the text the operator runs, and a
# drift between them is a failure rather than a silent re-test of a paraphrase.
#
# Nothing here touches a real system: `sudo` is a function that strips the privilege, and `systemd-run` and
# `systemctl` resolve to stubs on PATH. Run before the migration is authorized, not after it fails.
#
# Run:  bash tools/roadmap_cutover_runbook_witness.sh docs/runbooks/roadmap-event-log-cutover.md
#       (optional second argument: the path to dag/gunbc/witness/roadmap_cutover.dag)
# CASES IMPLEMENTED HERE — one line per gunbc.witness.roadmap_cutover::cutover_witness_case_rows row
# this carrier executes. The two-way join against these rows runs in this file (PART H here, PART B in
# the dispatch witness): a row with no line here is a promise no script keeps, and a line here with no
# row is a case the model has never heard of. The names are data, so prose cannot satisfy the join.
#   guard_admits_an_absolute_export_directory
#   guard_refuses_a_shifted_delivery
#   guard_admits_two_intact_arguments
#   guard_refuses_a_shifted_two_argument_delivery
#   guard_refuses_a_revision_that_is_not_forty_hex
#   drop_in_round_trip_with_pre_existing_and_deploy_added
#   citations_join_the_runbook_headings_to_the_model_rows_both_ways
#   a_citation_naming_an_unmodelled_step_is_reported
#   a_modelled_step_without_its_citation_is_reported
#   rollback_starts_exactly_the_units_the_export_found_active
#   rollback_refuses_an_export_naming_a_unit_off_the_roster
#   rollback_refuses_a_roster_unit_the_export_never_measured
#   rollback_refuses_an_export_that_measured_nothing
#   the_restart_fixture_can_see_the_defect_it_was_written_for
#   the_manifest_reaches_the_reader_intact_through_the_environment
#   the_interpolated_manifest_shape_is_refused_by_the_shell_that_parses_it
#   the_witness_itself_fails_when_a_guard_stops_refusing
#   stop_admits_the_measured_quiet_readers
#   stop_refuses_a_reader_still_answering
#   stop_refuses_a_timer_still_firing
#   stop_refuses_a_unit_that_would_not_stop
#   stop_refuses_a_timer_that_would_not_unload
#   stop_refuses_a_surviving_gunbc_process
#   stop_and_export_share_one_roster
#   rollback_refuses_a_unit_recorded_twice
#   rollback_refuses_a_word_that_is_not_a_state
#   migration_admits_what_the_export_measured
#   migration_stops_at_the_first_refused_append
#   migration_refuses_a_seed_that_changed_after_the_export
#   migration_refuses_a_fabric_object_without_a_line
#   migration_refuses_a_second_revocation
#   retirement_refuses_without_step_three_admission
#   retirement_refuses_with_no_revocation_admitted
#
set -u
RB=${1:?usage: roadmap_cutover_runbook_witness.sh <path to the runbook>}
[ -f "$RB" ] || { echo "WITNESS REFUSED: no runbook at [$RB] — there is nothing to witness"; exit 2; }
FAILURES=0
HELPERS_MISSING=
MODEL_ARG=${2:-}
MODEL=${MODEL_ARG:-$(cd "$(dirname "$RB")/../.." && pwd)/dag/gunbc/witness/roadmap_cutover.dag}
fail() { echo "FAIL $*"; FAILURES=$((FAILURES+1)); }
TMP=$(mktemp -d); SBIN=$TMP/sbin; SUDO=$TMP/bin; EXPORT_DIR=/opt/gunbc/exports/t6-20261110T230000Z
TB=$TMP/guard1.sh; TB7=$TMP/guard7.sh
mkdir -p "$SBIN" "$SUDO"

# ---- load the helpers FROM THE RUNBOOK. No copy lives here: an empty extraction is a refusal, not a fallback. ----
slice_from() { # $1 = exact first line, $2 = prefix of the last line
  awk -v b="$1" -v e="$2" '$0==b{f=1} f{print} f && index($0,e)==1{exit}' "$RB"
}
load_from_runbook() { # $1 = label, $2 = exact first line, $3 = prefix of last line
  local body; body=$(slice_from "$2" "$3")
  if [ -z "$body" ]; then echo "WITNESS REFUSED: the runbook has no [$1] starting at [$2] — refusing to invent one"; FAILURES=$((FAILURES+1)); HELPERS_MISSING=1; return 1; fi
  eval "$body"
  echo "loaded from the runbook: $1"
}
load_from_runbook "RUNF, the transient-unit helper" \
  'RUNF() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- "$@"; }' \
  'RUNF() { sudo systemd-run'
load_from_runbook "UNITS, the five roadmap readers" \
  'UNITS=(gunbc-roadmap.service gunbc-roadmap-tailnet-door.service gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service)' \
  'UNITS=('
load_from_runbook "export_drop_ins" \
  'export_drop_ins() { # $1 = the systemd directory, $2 = the export directory' '}'
load_from_runbook "restore_drop_ins" \
  'restore_drop_ins() { # $1 = the systemd directory, $2 = the export directory' '}'
load_from_runbook "need, the rollback's admission predicate" \
  'need() { local what="$1"; shift; if ! "$@"; then echo "ROLLBACK REFUSED: $what — nothing is started"; exit 1; fi; }' \
  'need() {'
[ "$FAILURES" -eq 0 ] || { echo "WITNESS REFUSED: the runbook did not supply every helper it claims to define"; exit 1; }
echo

# ---- the stubbed privileged surface (the only hand-written shell in this file, and it is a stand-in) ----
cat > "$SUDO/sudo" <<'EOF'
#!/usr/bin/env bash
exec "$@"
EOF
cat > "$SBIN/systemd-run" <<'EOF'
#!/usr/bin/env bash
# Stands in for systemd-run: drops its own options, then execs the command it was given — which is what the real
# one does, and the whole point of the exercise is what the inner bash sees as $1.
args=("$@")
while [ ${#args[@]} -gt 0 ]; do
  case "${args[0]}" in
    --wait|--collect|--pipe) args=("${args[@]:1}") ;;
    --uid)      args=("${args[@]:2}") ;;
    --uid=*)    args=("${args[@]:1}") ;;
    -p)         args=("${args[@]:2}") ;;
    bash)       args=("${args[@]:1}"); break ;;
    *) echo "systemd-run-fake: unexpected leading argument [${args[0]}]" >&2; exit 2 ;;
  esac
done
exec bash "${args[@]}"
EOF
cat > "$SBIN/systemctl" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  is-active) case "${2:-}" in gunbc-roadmap.service) echo active ;; *) echo inactive ;; esac ;;
  *) echo "systemctl-fake $*" ;;
esac
EOF
chmod +x "$SUDO/sudo" "$SBIN/systemd-run" "$SBIN/systemctl"
export PATH="$SUDO:$SBIN:$PATH"

echo "================ PART A: argument delivery through the ACTUAL helper shape ================"
# The scripts under test are the runbook's own argument-guard blocks plus one marker line, delivered exactly the
# way the runbook delivers them.
GUARD1=$(slice_from 'D=$1' 'esac')
GUARD7=$(slice_from 'D=$1; SHA=$2' '[ "${#SHA}" -eq 40 ]')
[ -n "$GUARD1" ] || { echo "FAIL — step 1's guard block not found in the runbook"; FAILURES=$((FAILURES+1)); }
[ -n "$GUARD7" ] || { echo "FAIL — step 7's guard block not found in the runbook"; FAILURES=$((FAILURES+1)); }
printf '%s\n' "$GUARD1" > "$TB";  echo 'echo "GUARD HELD: D=$D"' >> "$TB"
printf '%s\n' "$GUARD7" > "$TB7"; echo 'echo "GUARD HELD: D=$D SHA=$2"' >> "$TB7"
sha40=$(printf 'a%.0s' {1..40})

echo "--- A1: correct helper, one data argument (step 1 passes \$D) ---"
# These five are ASSERTIONS, not commentary. An earlier pass printed "A1 rc=1 — 0 and the absolute path are
# required" next to a rc of 1 and still exited 0, because nothing turned the printed expectation into a
# condition: the witness reported results it did not enforce, and a guard that stopped working would have
# passed. Every case now states the exit status it requires AND the diagnostic it requires, and a case that
# disagrees with its own expectation increments FAILURES.
OUT=$(RUNF "$EXPORT_DIR" < "$TB" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -eq 0 ] || fail "A1: the correct helper refused a well-formed call (rc=$RC)"
printf '%s' "$OUT" | grep -qF "GUARD HELD: D=$EXPORT_DIR" \
  || fail "A1: \$D did not arrive intact — no 'GUARD HELD: D=$EXPORT_DIR' in the output"

echo "--- A2: the SHIFTED helper (a filler word after --), same script ---"
RUNF_shifted() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- runbook "$@"; }
OUT=$(RUNF_shifted "$EXPORT_DIR" < "$TB" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "A2: the SHIFTED helper was NOT refused — a shifted delivery would export into a relative directory named 'runbook'"
printf '%s' "$OUT" | grep -qF 'argument delivery shifted' \
  || fail "A2: the refusal did not name the defect (expected the runbook's own 'argument delivery shifted')"

echo "--- A3: correct helper, two data arguments (step 7 passes \$D then \$SHA) ---"
OUT=$(RUNF "$EXPORT_DIR" "$sha40" < "$TB7" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -eq 0 ] || fail "A3: the correct helper refused a well-formed two-argument call (rc=$RC)"
printf '%s' "$OUT" | grep -qF "GUARD HELD: D=$EXPORT_DIR SHA=$sha40" \
  || fail "A3: \$D and \$SHA did not both arrive intact (expected 'GUARD HELD: D=$EXPORT_DIR SHA=$sha40')"

echo "--- A4: the same script through the SHIFTED helper — the guard must refuse ---"
OUT=$(RUNF_shifted "$EXPORT_DIR" "$sha40" < "$TB7" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "A4: the SHIFTED helper was NOT refused on step 7's script"
printf '%s' "$OUT" | grep -qF 'argument delivery shifted' \
  || fail "A4: the refusal did not name the defect (expected 'argument delivery shifted')"

echo "--- A5: a revision that is not 40 hex is refused by step 7's own guard ---"
OUT=$(RUNF "$EXPORT_DIR" "not-a-sha" < "$TB7" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "A5: a non-hex revision was accepted"
printf '%s' "$OUT" | grep -qF 'which is not 40 hex' \
  || fail "A5: the refusal did not say why (expected 'which is not 40 hex')"

# A6: THE WITNESS ITSELF MUST BE ABLE TO FAIL. The five cases above are only worth what they enforce, so the
# enforcement is tested: the runbook is copied with step 1's argument guard neutralised (the refusal replaced by
# a no-op) and the WHOLE witness is re-run against that copy. If the mutation does not make the witness exit
# nonzero, the A-series is narration again and this case says so.
if [ -n "${RUNBOOK_WITNESS_NOMETA:-}" ]; then
  echo "--- A6: skipped (this IS the meta run) ---"
else
  MUT=$TMP/mutated-runbook.md
  sed 's|^  \*) echo "RUNF REFUSED: the export directory arrived as \[\$D\] — argument delivery shifted"; exit 1 ;;$|  *) : ;;|' "$RB" > "$MUT"
  if [ "$(grep -cF 'RUNF REFUSED: the export directory arrived as' "$MUT")" = "$(grep -cF 'RUNF REFUSED: the export directory arrived as' "$RB")" ]; then
    fail "A6: the mutation did not land — step 1's guard text was not found to neutralise, so this case proves nothing"
  else
    SELF=${BASH_SOURCE[0]:-$0}
    CHILD_LOG=$TMP/meta-child.log
    RUNBOOK_WITNESS_NOMETA=1 bash "$SELF" "$MUT" "$MODEL" > "$CHILD_LOG" 2>&1; RC=$?
    OUT=$(cat "$CHILD_LOG")
    echo "    (meta run: script [$SELF], mutated book [$MUT], rc=$RC, $(wc -c < "$CHILD_LOG") bytes)"
    grep -E '^(FAIL|WITNESS REFUSED)' "$CHILD_LOG" | sed 's/^/    /'
    [ "$RC" -ne 0 ] || fail "A6: a runbook whose argument guard never refuses still passed the witness — the A-series enforces nothing"
    grep -q 'A2' "$CHILD_LOG" || fail "A6: the witness failed (rc=$RC) but its output names no A2 case"
    echo "    A6 OK: a neutralised guard makes the witness itself exit $RC"
  fi
fi
echo

echo "================ PART B: the drop-in round trip, pre-existing AND deployment-added ================"
SYS=$TMP/system; EXP=$TMP/export; DEPLOY=$TMP/deploy
install -d -m 0755 "$SYS/gunbc-roadmap.service.d" "$SYS/gunbc-roadmap-tailnet-door.service.d" \
  "$EXP/units/install" "$DEPLOY"
printf '[Service]\nExecStartPre=/bin/true\n'               > "$SYS/gunbc-roadmap.service.d/pre-existing.conf"
printf '[Unit]\nDescription=added by the modeled deploy\n' > "$SYS/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf"
printf 'ExecStart=/opt/gunbc/bin/gunbc roadmap-page\n'     > "$SYS/gunbc-roadmap.service"

echo "--- B1: export (step 1's helper) ---"
export_drop_ins "$SYS" "$EXP"
( cd "$EXP" && find units/drop-ins -type f | sort )
echo "--- B2: the modeled deploy rewrites a unit file and adds a drop-in under the first unit ---"
printf 'ExecStart=/opt/gunbc/bin/staged/gunbc roadmap-page\n' > "$SYS/gunbc-roadmap.service"
rm -f "$SYS/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf"
install -d -m 0755 "$SYS/gunbc-roadmap.service.d"
printf '# written by the staged deploy\n[Service]\nEnvironment=GUARB=1\n' > "$SYS/gunbc-roadmap.service.d/10-deploy.conf"
echo "--- B3: restore (step 7's helper) ---"
restore_drop_ins "$SYS" "$EXP"
echo "--- B4: assertions ---"
fail() { echo "FAIL $*"; FAILURES=$((FAILURES+1)); }
[ -f "$SYS/gunbc-roadmap.service.d/pre-existing.conf" ] || fail "the pre-existing drop-in did not come back at its own path"
[ ! -e "$SYS/gunbc-roadmap.service.d/10-deploy.conf" ]  || fail "the deployment-added drop-in survived the rollback"
diff -q <(printf '[Service]\nExecStartPre=/bin/true\n') "$SYS/gunbc-roadmap.service.d/pre-existing.conf" \
  || fail "the restored drop-in is not byte-identical"
diff -q <(printf '[Unit]\nDescription=added by the modeled deploy\n') "$SYS/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf" \
  || fail "a drop-in that existed BEFORE the cutover on another unit was not restored — that is the defect this case exists for"
[ "$(ls -A "$SYS/gunbc-roadmap.service.d")" = "pre-existing.conf" ] || fail "the restored unit has extra content: $(ls -A "$SYS/gunbc-roadmap.service.d")"
[ -f "$EXP/units.drop-ins.inventory" ] || fail "step 1 did not record which drop-ins existed"
grep -q 'units/drop-ins/gunbc-roadmap.service.d/pre-existing.conf' "$EXP/units.drop-ins.inventory" \
  || fail "the inventory does not name the pre-existing drop-in"
( need "the export recorded every unit's drop-ins" test -f "$EXP/units.drop-ins.inventory" ) || fail "a satisfied need refused"
( need "an unsatisfied predicate" false; echo "SHOULD NOT REACH" ) && fail "need() did not stop the script"
echo "ROUND TRIP OK: every unit's drop-in directory returned to its own path, byte-identical; the"
echo "               deployment-added one is gone; the pre-cutover ones (on both units) are back."
echo

echo "================ PART C: the runbook's headings and the model's rows join both ways ================"
# The model is the authority for what the cutover IS (cutover_step_rows); the runbook is the text the operator
# runs, and each of its step headings cites the row that authorises it. The step ids live in the MODEL and
# nowhere else: they are read out of the .dag below, never retyped into this file, so a step can be added to one
# file and be refused for not being in the other. A paragraph that is not a row, and a row nobody runs, both fail here.
if [ ! -f "$MODEL" ]; then
  echo "WITNESS REFUSED: no model at [$MODEL] — the step ids are read from it, not invented here"
  FAILURES=$((FAILURES+1))
else
  model_step_ids() { awk '/^data cutover_step_rows/,/^]/' "$MODEL" | sed -n 's/^[[:space:]]*id: "\([^"]*\)".*/\1/p'; }
  cited_ids()      { sed -n 's/^## .*— row `\([^`]*\)`.*/\1/p' "$1"; }
  # one problem line per defect; silence means the join is exact in both directions
  check_citations() {
    local id n
    for id in $(model_step_ids); do
      n=$(cited_ids "$1" | grep -c "^${id}$")
      [ "$n" -eq 1 ] || echo "step [$id] is cited $n time(s); exactly one heading must cite it"
    done
    for id in $(cited_ids "$1"); do
      model_step_ids | grep -q "^${id}$" || echo "the runbook cites row [$id], which is not a modelled step"
    done
  }
  NMODEL=$(model_step_ids | wc -l); NCITED=$(cited_ids "$RB" | wc -l)
  echo "model rows: $NMODEL   runbook headings citing a row: $NCITED   (model: $MODEL)"
  [ "$NMODEL" -gt 0 ] || fail "not one step id could be read from the model — the join would pass by emptiness"

  PROBLEMS=$(check_citations "$RB")
  if [ -z "$PROBLEMS" ]; then echo "C1 OK: every modelled step is cited by exactly one heading, and every citation names a modelled step"
  else echo "$PROBLEMS"; fail "the shipped runbook and the model do not join" ; fi

  # C2: a citation that names nothing — a step renamed in prose, or a paragraph someone added by hand.
  GHOST=$TMP/red_ghost.md; sed 's/— row `observe`/— row `observed`/' "$RB" > "$GHOST"
  OUT=$(check_citations "$GHOST")
  { echo "$OUT" | grep -q 'not a modelled step' && echo "$OUT" | grep -q '\[observed\]'; } \
    || fail "C2: a citation naming an unmodelled step was not reported (got: ${OUT:-<nothing>})"
  [ -n "$OUT" ] || fail "C2 red case passed silently"

  # C3: a modelled step nobody runs any more — its heading lost its citation, so the operator would never see it.
  LOST=$TMP/red_lost.md; sed 's/ — row `stage`//' "$RB" > "$LOST"
  OUT=$(check_citations "$LOST")
  { echo "$OUT" | grep -q 'step \[stage\] is cited 0 time(s)' && ! echo "$OUT" | grep -q 'not a modelled step'; } \
    || fail "C3: a modelled step without its citation was not reported, or something else was (got: ${OUT:-<nothing>})"

  if [ -n "$PROBLEMS" ]; then echo "(the C2/C3 red cases above are the witness refusing a broken book, which is what they exist for)"; fi
  [ "$FAILURES" -eq 0 ] && echo "CITATIONS OK: the book the operator runs and the rows that authorise it are the same nine steps, in both directions."
  echo
fi

echo "================ PART D: the rollback starts exactly the units the export found active ================"
# Step 7 restarts what step 2 measured, and the first version of it restarted NOTHING: the export lines already
# read `<unit> <state>`, and pasting a second unit list in front of them made `read -r U STATE` bind
# `<unit> active` to STATE, so no line ever equalled `active`. A rollback that reports a failure while leaving
# the readers down is worse than one that crashes, so the leg is extracted from the runbook and EXECUTED against
# a stateful systemctl stub: what it starts, and what it refuses to start, are both observed.
slice_until() { # $1 = exact first line, $2 = exact last line (both matched at column 0)
  awk -v b="$1" -v e="$2" '$0==b{f=1} f{print} f && $0==e{exit}' "$RB"
}
RESTORE_LEG=$(slice_until 'sudo systemctl reset-failed "${UNITS[@]}" || true' 'fi')
NEED_FN=$(slice_from 'need() { local what="$1"; shift; if ! "$@"; then echo "ROLLBACK REFUSED: $what — nothing is started"; exit 1; fi; }' 'need() {')
[ -n "$RESTORE_LEG" ] || fail "PART D: the rollback's restore leg could not be located in the runbook — refusing to invent one"
[ -n "$NEED_FN" ]     || fail "PART D: need() could not be loaded from the runbook"

CTL=$TMP/ctl; install -d -m 0755 "$CTL"
cat > "$CTL/systemctl" <<'EOF'
#!/usr/bin/env bash
# A stateful stand-in: `start` records that the unit was started, `is-active` answers from what has been started.
# Without the state, an is-active that always says "active" would let a rollback that started nothing pass.
case "${1:-}" in
  start) shift; for u in "$@"; do echo "$u" >> "$CTL_STARTED"; done ;;
  is-active)
    if [ -f "$CTL_STARTED" ] && grep -qx "${2:-}" "$CTL_STARTED"; then echo active; else echo inactive; fi ;;
  reset-failed) : ;;
  *) echo "systemctl-fake $*" ;;
esac
EOF
chmod +x "$CTL/systemctl"

# The leg is run through the REAL helper (RUNF, loaded from the runbook), with the export directory as data, so
# argument delivery and the restart behaviour are witnessed in the same shape the operator uses.
mk_leg() {
  { printf 'set -euo pipefail\nD=$1\nDISPOSITION=$2\n'
    printf 'UNITS=(%s)\n' "$(printf '"%s" ' "${UNITS[@]}")"
    printf '%s\n' "$NEED_FN"
    printf '%s\n' "$RESTORE_LEG"; } > "$TMP/leg.sh"
}
RBEXP=$TMP/rollback-export; install -d -m 0755 "$RBEXP"
export CTL_STARTED=$TMP/started.log

echo "--- D1: two of five recorded active, DISPOSITION 7A — those two and only those two must come up ---"
printf '%s\n' "gunbc-roadmap.service active" "gunbc-roadmap-tailnet-door.service active" \
  "gunbc-roadmap-belt-discovery.service inactive" "gunbc-roadmap-belt-exit.service inactive" \
  "gunbc-roadmap-belt-credential-recovery.service inactive" > "$RBEXP/units.active.before"
: > "$CTL_STARTED"; mk_leg; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -eq 0 ] || fail "D1: the rollback refused a well-formed export (rc=$RC)"
grep -qx 'gunbc-roadmap.service' "$CTL_STARTED"        || fail "D1: a unit the export found active was NEVER started — this is the reported defect"
grep -qx 'gunbc-roadmap-tailnet-door.service' "$CTL_STARTED" || fail "D1: the tailnet door was never started"
for U in gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service; do
  ! grep -qx "$U" "$CTL_STARTED" || fail "D1: $U was recorded INACTIVE and the rollback started it anyway"
done
printf '%s' "$OUT" | grep -qF 'RESTORED 2 unit(s)' || fail "D1: the rollback did not report restoring 2 units (got: $(printf '%s' "$OUT" | tr '\n' '|'))"

echo "--- D2: the export names a unit this cutover does not own — refuse, and start nothing ---"
printf '%s\n' "gunbc-roadmap.service active" "not-our-unit.service active" > "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "D2: an off-roster unit in the export was accepted"
printf '%s' "$OUT" | grep -qF 'not in the admitted five-unit roster' || fail "D2: the refusal did not name the roster"
[ ! -s "$CTL_STARTED" ] || fail "D2: it refused and started something anyway: $(tr '\n' ' ' < "$CTL_STARTED")"

echo "--- D3: a roster unit has no line in the export — refusing to guess that it was down ---"
printf '%s\n' "gunbc-roadmap.service active" "gunbc-roadmap-tailnet-door.service active" \
  "gunbc-roadmap-belt-discovery.service inactive" "gunbc-roadmap-belt-exit.service inactive" > "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "D3: a missing export line was silently read as 'was down'"
printf '%s' "$OUT" | grep -qF 'has no line for' || fail "D3: the refusal did not say which unit is unmeasured"

echo "--- D4: an empty export — no measurement, no rollback start ---"
: > "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "D4: an empty export was accepted"
printf '%s' "$OUT" | grep -qF 'missing or empty' || fail "D4: the refusal did not say the export was missing or empty"

echo "--- D5: the fixture must be able to see the defect it exists for (the pasted-list shape) ---"
# Same export, same leg — but the export is built the way the defective rollback READ it, with a second unit
# list pasted in front of it. If a rollback regressed to that shape, this witness must see zero starts; if it
# does not see them, the fixture has gone blind and this case fails.
printf '%s\n' "gunbc-roadmap.service active" "gunbc-roadmap-tailnet-door.service active" \
  "gunbc-roadmap-belt-discovery.service inactive" "gunbc-roadmap-belt-exit.service inactive" \
  "gunbc-roadmap-belt-credential-recovery.service inactive" > "$RBEXP/units.active.before"
PASTED=$TMP/pasted-shape.before
paste -d' ' <(printf '%s\n' "${UNITS[@]}") "$RBEXP/units.active.before" > "$PASTED"
cp "$PASTED" "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
if grep -q . "$CTL_STARTED"; then
  fail "D5: the pasted-list shape started $(tr '\n' ' ' < "$CTL_STARTED") — the leg no longer reads the export directly, so the shape that hid the defect is back"
else
  # Zero starts is necessary and NOT sufficient: an inventory the leg cannot trust must refuse, because "RESTORED
  # 0 unit(s)" after a rollback that restored nothing reads as a completed rollback. F7's fix is the refusal.
  [ "$RC" -ne 0 ] || fail "D5: the pasted-list shape was ACCEPTED (rc=0) — a malformed inventory must refuse, not report zero restores"
  echo "$OUT" | grep -qF 'expected exactly two fields' \
    || fail "D5: the refusal did not identify the malformed record: [$OUT]"
  echo "    D5 OK: the pasted shape is REFUSED as an invalid inventory, before anything is started"
fi

# The two shapes the old check could not see either: the same unit twice (a second record silently overwrites the
# first's state) and a word that is not a state systemd reports.
echo "--- D6: a unit recorded twice is refused ---"
# rebuilt from the five measured records: D5 left the PASTED shape in place, and building a new shape on top of a
# broken one would test the wrong input.
printf '%s\n' "gunbc-roadmap.service active" "gunbc-roadmap-tailnet-door.service active" \
  "gunbc-roadmap-belt-discovery.service inactive" "gunbc-roadmap-belt-exit.service inactive" \
  "gunbc-roadmap-belt-credential-recovery.service inactive" > "$RBEXP/units.active.before"
{ cat "$RBEXP/units.active.before"; echo "gunbc-roadmap.service inactive"; } > "$TMP/dup.before"
cp "$TMP/dup.before" "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "D6: a unit recorded twice was accepted — the second record silently overwrites the first's state"
echo "$OUT" | grep -qF 'recorded more than once' || fail "D6: the refusal did not name the duplicate: [$OUT]"
[ ! -s "$CTL_STARTED" ] || fail "D6: it refused and started something anyway: $(tr '\n' ' ' < "$CTL_STARTED")"

echo "--- D7: a word that is not a state systemd reports is refused ---"
grep -vx 'gunbc-roadmap.service inactive' "$TMP/dup.before" \
  | sed 's/^gunbc-roadmap.service active$/gunbc-roadmap.service running/' > "$TMP/state.before"
cp "$TMP/state.before" "$RBEXP/units.active.before"
: > "$CTL_STARTED"; OUT=$(PATH="$CTL:$SUDO:$SBIN:$PATH" RUNF "$RBEXP" 7A < "$TMP/leg.sh" 2>&1); RC=$?
echo "$OUT" | sed 's/^/    /'
[ "$RC" -ne 0 ] || fail "D7: [running] was accepted as a unit state — it is what `systemctl is-failed`-style output looks like, not an is-active answer"
echo "$OUT" | grep -qF 'not a state systemd reports' || fail "D7: the refusal did not name the bad state: [$OUT]"
[ ! -s "$CTL_STARTED" ] || fail "D7: it refused and started something anyway: $(tr '\n' ' ' < "$CTL_STARTED")"
echo

echo "================ PART E: the manifest arrives as DATA, not as shell ================"
# Step 5 passes the expected manifest through the environment (systemd-run --setenv), and step 6 now does the
# same. An earlier step-6 put it INSIDE the command string that RUN hands to `bash -lc`, and the JSON's own
# quotes were then parsed as shell by the shell systemd-run spawns: the reviewer reproduced an object arriving
# as several arguments with its quotes removed. The manifest is the cutover's evidence, and one historical line
# of it carries an apostrophe in a revocation reason, so both are used here verbatim.
slice_from_t() { # like slice_from but both anchors are matched AFTER leading blanks are dropped
  awk -v b="$2" -v e="$3" '{t=$0; sub(/^[ \t]+/,"",t)} t==b{f=1} f{print} f && index(t,e)==1{exit}' "$RB"
}
load_from_runbook_t() { # $1 = label, $2 = first line (blanks-insensitive), $3 = prefix of last line (blanks-insensitive)
  local body; body=$(slice_from_t x "$2" "$3")
  if [ -z "$body" ]; then echo "WITNESS REFUSED: the runbook has no [$1] starting at [$2] — refusing to invent one"; FAILURES=$((FAILURES+1)); HELPERS_MISSING=1; return 1; fi
  eval "$body"
  echo "loaded from the runbook: $1"
}
# RUNE closes on an INDENTED line (`         sudo systemd-run … "$CMD"; }`), so the anchors are matched with
# leading blanks dropped: an end anchor at column 0 would never match, and a slice that runs to EOF would eval
# the runbook's prose.
load_from_runbook_t "RUNE, the environment-passing helper" \
  'RUNE() { local envs=() k; for k in "$@"; do envs+=(--setenv="$k"); done' 'sudo systemd-run'
load_from_runbook_t "RUN, the plain transient-unit helper" \
  'RUN()  { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -lc "$1"; }' \
  'RUN()  { sudo systemd-run'
[ -z "$HELPERS_MISSING" ] || { echo "WITNESS REFUSED: the runbook did not supply RUN/RUNE"; exit 1; }

REC=$TMP/record; install -d -m 0755 "$REC"
cat > "$REC/systemd-run" <<'EOF'
#!/usr/bin/env bash
# Records argv and every --setenv value EXACTLY as received, then runs what the real one would run.
: > "$ARGV_FILE"; : > "$ENV_FILE"
while [ $# -gt 0 ]; do
  case "$1" in
    --setenv=*) printf '%s\n' "${1#--setenv=}" >> "$ENV_FILE"; export "${1#--setenv=}"; shift ;;
    -p) shift 2 ;;
    --uid|--uid=*|--wait|--collect|--pipe) shift ;;
    bash) shift; printf '%s\n' "$*" >> "$ARGV_FILE"; exec bash "$@" ;;
    *) echo "systemd-run-fake: unexpected argument [$1]" >&2; exit 2 ;;
  esac
done
EOF
cat > "$REC/sudo" <<'EOF'
#!/usr/bin/env bash
exec "$@"
EOF
cat > "$REC/gunbc" <<'EOF'
#!/usr/bin/env bash
# Stands in for the reader binary, recording ONE LINE PER ARGUMENT in a form that survives embedded newlines and
# quotes. The question is never what the operator typed: it is what the reader is handed once the shell systemd-run
# spawns has finished with it.
: > "$ARGS_FILE"
for a in "$@"; do printf '%q\n' "$a" >> "$ARGS_FILE"; done
EOF
chmod +x "$REC/systemd-run" "$REC/sudo" "$REC/gunbc"
export ARGV_FILE=$TMP/argv.txt ENV_FILE=$TMP/env.txt ARGS_FILE=$TMP/args.txt

MANIFEST=$TMP/seed.jsonl
printf '%s\n' \
  '{"kind":"acceptance","node":"roadmap-item-1","receipt":{"principal":"principal:google-oidc/accounts/116084671989231734979","id":"receipt-1"}}' \
  '{"kind":"revocation","node":"roadmap-item-2","exact_prior_receipt":"receipt-2","reason":"operator revoked: it was the customer'\''s own row, not ours"}' \
  '{"kind":"acceptance","node":"roadmap-item-3","receipt":{"principal":"principal:google-oidc/accounts/116084671989231734979","id":"receipt-3"}}' \
  > "$MANIFEST"
ROSTER='entry-graph-union-construction,shell-dag-floor-peak-calibration-rows'

echo "--- E1: RUNE delivers the manifest byte-for-byte, and none of it is in argv ---"
# The command is the runbook's shape: an ABSOLUTE $BIN (a `bash -l` rebuilds PATH, which is exactly why the runbook
# never relies on a bare word) that reads both expectations out of the environment.
BIN_E=$REC/gunbc
CMD="$BIN_E run --function roadmap_event_log_read_back_cli --arg expected_manifest=\"\$EXPECTED_MANIFEST\" --arg retired_roster=\"\$EXPECTED_ROSTER\""
: > "$ARGS_FILE"; : > "$ENV_FILE"
PATH="$REC:$SUDO:$PATH" RUNE "EXPECTED_MANIFEST=$(cat "$MANIFEST")" "EXPECTED_ROSTER=$ROSTER" >/dev/null 2>&1
RC=$?
[ "$RC" -eq 0 ] || fail "E1: the environment shape exited $RC"
# --setenv entries arrive in order and the manifest's own newlines are DATA, not separators, so the first entry is
# the manifest with its prefix removed, and the roster is its own entry.
awk 'NR==1{sub(/^EXPECTED_MANIFEST=/,""); print; next} /^EXPECTED_ROSTER=/{exit} {print}' "$ENV_FILE" > "$TMP/env.manifest.got"
diff -q "$MANIFEST" "$TMP/env.manifest.got" >/dev/null \
  || fail "E1: the manifest did not arrive in the environment byte-for-byte ($(grep -c . "$MANIFEST") lines expected, $(grep -c . "$TMP/env.manifest.got") received)"
grep -qx "EXPECTED_ROSTER=$ROSTER" "$ENV_FILE" || fail "E1: the retired roster did not arrive in the environment as its own entry"
# THE OBSERVATION THAT MATTERS: not what was typed, but what the reader is handed. One argument, the whole file,
# apostrophe-bearing revocation line included, byte-for-byte.
printf '%q\n' "expected_manifest=$(cat "$MANIFEST")" > "$TMP/arg.expected"
grep -F 'expected_manifest=' "$ARGS_FILE" | head -1 > "$TMP/arg.got" || true
diff -q "$TMP/arg.expected" "$TMP/arg.got" >/dev/null \
  || fail "E1: the manifest did not reach the reader as one intact argument (got: $(cut -c1-140 "$TMP/arg.got"))"
grep -qF 'operator revoked' "$ARGS_FILE" || fail "E1: the apostrophe-bearing revocation line did not survive the trip"
grep -qF 'EXPECTED_MANIFEST' "$ARGV_FILE" || fail "E1: the command line no longer reads the manifest from the environment"

echo "--- E2: the unsafe interpolated shape must be CAUGHT corrupting it (the fixture's own eyes) ---"
SAFE_CMD="$CMD"
MANIFEST_TEXT=$(cat "$MANIFEST")
# The shape step 6 used to have, reproduced FAITHFULLY: the manifest's own text is spliced into the command string
# by the SHELL THAT BUILDS IT, so the JSON's quotes arrive in the source the inner bash parses. (`$(cat …)` inside
# the string would NOT reproduce it — a substitution the inner shell performs stays data; it is the outer splice
# that turns data back into syntax. That distinction is why RUNE's environment is the fix and not a rewording.)
CMD="$BIN_E run --function roadmap_event_log_read_back_cli --arg expected_manifest=\"$MANIFEST_TEXT\" --arg retired_roster=\"$ROSTER\""
: > "$ARGS_FILE"; : > "$ENV_FILE"
PATH="$REC:$SUDO:$PATH" RUN "$CMD" >/dev/null 2>&1
RC2=$?
NARG=$(grep -c . "$ARGS_FILE" || true)
if [ "$RC2" -ne 0 ]; then
  echo "    E2 OK: SHAPE REFUSED — the interpolated shape never even ran; the spliced JSON left the inner shell with a syntax error (rc=$RC2). That is the strongest form of the finding: the old step 6 could have failed to certify anything at all"
elif diff -q "$TMP/arg.expected" <(grep -F 'expected_manifest=' "$ARGS_FILE" | head -1) >/dev/null 2>&1; then
  fail "E2: the interpolated shape arrived INTACT — the fixture has gone blind, so E1 would prove nothing about the shape step 6 no longer uses"
else
  echo "    E2 OK: SHAPE REFUSED — the interpolated shape arrives ALTERED across $NARG arguments (the JSON's own quotes parsed as shell by the shell systemd-run spawns), which is why step 6 uses RUNE"
fi
CMD="$SAFE_CMD"
echo

echo "================ PART F: stopping the readers is a measured admission, not a printed hope ================"
# Step 2 used to end in three lines of `# expect: inactive` comments. A comment is not an admission: an
# unobserved stop leaves a reader answering from the old carrier for the whole migration window, and these are
# SEPARATE operator-shell blocks, so the `set -euo pipefail` inside step 1's RUNF script never reaches them.
fence_containing() { # the body of the ```bash fence whose text contains $1 — the runbook stays the authority
  awk -v m="$1" '
    /^```bash$/ { inf = 1; buf = ""; next }
    inf && /^```$/ { if (index(buf, m) > 0) { printf "%s", buf; exit } inf = 0 }
    inf { buf = buf $0 "\n" }' "$RB"
}
STOP_LEG=$(fence_containing 'STOP ADMITTED')
[ -n "$STOP_LEG" ] || { echo "WITNESS REFUSED: the runbook has no step-2 block ending in STOP ADMITTED — refusing to invent one"; FAILURES=$((FAILURES+1)); HELPERS_MISSING=1; }
if [ -z "$HELPERS_MISSING" ]; then
  STCTL=$TMP/stctl; install -d -m 0755 "$STCTL"
  cat > "$STCTL/systemctl" <<'EOF'
#!/usr/bin/env bash
# A state table, so "is-active" answers what the case CLAIMED rather than what the last case left behind.
case "$1" in
  is-active) shift
             for u in "$@"; do s=$(awk -v u="$u" '$1==u{print $2}' "$CTL_STATE" | head -1); printf '%s\n' "${s:-inactive}"; done ;;
  stop)      shift
             for u in "$@"; do
               if [ "$u" = "${STUB_STOP_FAIL:-}" ]; then echo "stub: $u refuses to stop" >&2; exit 1; fi
               printf '%s\n' "$u" >> "$CTL_STOPPED"
               # a unit that RESISTS the stop keeps its recorded state — that is what a stuck reader looks like,
               # and it is the only way F2/F3 can ask the question without lying about the world
               [ "$u" = "${STUB_STOP_KEEPS:-}" ] && continue
               grep -vx "$u .*" "$CTL_STATE" > "$CTL_STATE.new" && mv "$CTL_STATE.new" "$CTL_STATE"
               printf '%s inactive\n' "$u" >> "$CTL_STATE"
             done ;;
  reset-failed) : ;;
  *) exit 0 ;;
esac
EOF
  cat > "$STCTL/ps" <<'EOF'
#!/usr/bin/env bash
[ -n "${PS_OUT:-}" ] && printf '%s\n' "${PS_OUT:-}"
exit 0
EOF
  chmod +x "$STCTL/systemctl" "$STCTL/ps"
  export CTL_STOPPED=$TMP/stopped.log
  all_inactive() { : > "$TMP/ctl.state"
    for u in "${UNITS[@]}" gunbc-roadmap-belt-credential-recovery.timer; do printf '%s %s\n' "$u" "${1:-inactive}" >> "$TMP/ctl.state"; done; }
  set_state() { awk -v u="$1" '$1!=u' "$TMP/ctl.state" > "$TMP/ctl.state.new" && mv "$TMP/ctl.state.new" "$TMP/ctl.state"
                printf '%s %s\n' "$1" "$2" >> "$TMP/ctl.state"; }
  run_stop() { # $1 = label; the rest are VAR=value overrides
    local label="$1"; shift
    : > "$CTL_STOPPED"; export CTL_STATE=$TMP/ctl.state
    local out rc; out=$(env PATH="$STCTL:$SUDO:$SBIN:$PATH" "$@" bash "$TMP/stopleg.sh" 2>&1); rc=$?
    echo "  [$label] rc=$rc"; echo "$out" | sed 's/^/    /'
    LAST_OUT="$out"; LAST_RC=$rc
  }
  printf '%s\n' "$STOP_LEG" > "$TMP/stopleg.sh"

  all_inactive; run_stop "every unit and the timer inactive, no process"
  [ "$LAST_RC" -eq 0 ] || fail "F1: the stopped readers were ADMITTED as refused — the step-2 block cannot pass its own happy path"
  echo "$LAST_OUT" | grep -qF 'STOP ADMITTED: 5 units and the timer report inactive' \
    || fail "F1: no admission line was printed: [$LAST_OUT]"
  [ "$(wc -l < "$CTL_STOPPED" | tr -d ' ')" = 6 ] \
    || fail "F1: it stopped $(wc -l < "$CTL_STOPPED") unit(s), expected the five readers AND the timer"

  all_inactive; set_state gunbc-roadmap-tailnet-door.service active
  run_stop "one reader still active after the stop" STUB_STOP_KEEPS=gunbc-roadmap-tailnet-door.service
  [ "$LAST_RC" -ne 0 ] || fail "F2: a reader still answering was admitted — this is the reported defect"
  echo "$LAST_OUT" | grep -qF 'gunbc-roadmap-tailnet-door.service reports [active]' \
    || fail "F2: the refusal did not name the reader still up: [$LAST_OUT]"

  all_inactive; set_state gunbc-roadmap-belt-credential-recovery.timer active
  run_stop "the timer still firing after the stop" STUB_STOP_KEEPS=gunbc-roadmap-belt-credential-recovery.timer
  [ "$LAST_RC" -ne 0 ] || fail "F3: a timer still firing was admitted — the oneshots would write into the migration window"
  echo "$LAST_OUT" | grep -qF 'gunbc-roadmap-belt-credential-recovery.timer reports [active]' \
    || fail "F3: the refusal did not name the timer: [$LAST_OUT]"

  all_inactive; run_stop "one stop command fails" STUB_STOP_FAIL=gunbc-roadmap-belt-exit.service
  [ "$LAST_RC" -ne 0 ] || fail "F4: a failed stop was admitted"
  echo "$LAST_OUT" | grep -qF 'gunbc-roadmap-belt-exit.service did not stop' \
    || fail "F4: the refusal did not name the unit that refused to stop: [$LAST_OUT]"

  all_inactive; run_stop "the timer refuses to be unloaded" STUB_STOP_FAIL=gunbc-roadmap-belt-credential-recovery.timer
  [ "$LAST_RC" -ne 0 ] || fail "F5: a timer that could not be unloaded was admitted"
  echo "$LAST_OUT" | grep -qF 'the belt-credential-recovery timer is still loaded' \
    || fail "F5: the refusal did not name the timer: [$LAST_OUT]"

  all_inactive; run_stop "a gunbc process is still running" PS_OUT="briansrls 4242 1 0 09:00 ? 00:00:01 /opt/gunbc/bin/gunbc serve"
  [ "$LAST_RC" -ne 0 ] || fail "F6: a surviving gunbc process was admitted as stopped"
  echo "$LAST_OUT" | grep -qF 'gunbc process(es) still running' \
    || fail "F6: the refusal did not name the surviving process: [$LAST_OUT]"

  # THE TWO ROSTERS MUST BE THE SAME FIVE. Step 1 measures; step 2 admits. Add a unit to one list and not the
  # other and the migration runs with a reader nobody measured — a whole that no check can see.
  EXP_FENCE=$(fence_containing 'units.active.before')
  ROSTER_MEASURED=$(printf '%s\n' "$EXP_FENCE" | sed -n '/^UNITS=(/,/)/p' | tr -d '()"' | tr -s ' \' ' ' | tr ' ' '\n' | grep -E '^gunbc' | sort)
  ROSTER_ADMITTED=$(printf '%s\n' "$STOP_LEG" | sed -n '/^READERS=(/,/)/p' | tr -d '()"' | tr -s ' \' ' ' | tr ' ' '\n' | grep -E '^gunbc' | sort)
  if [ "$ROSTER_MEASURED" != "$ROSTER_ADMITTED" ]; then
    fail "F7: step 1 measures [$(echo "$ROSTER_MEASURED" | tr '\n' ' ')] but step 2 admits [$(echo "$ROSTER_ADMITTED" | tr '\n' ' ')] — the export and the admission must be the same five"
  else
    echo "  [F7 the two rosters are the same five units] OK: $(echo "$ROSTER_ADMITTED" | tr '\n' ' ')"
  fi
fi
echo

echo "================ PART G: a refused append stops the migration; the input is counted before it ================"
# The migration loops read a file and call the modeled CLI per line. Under a process substitution `set -e` cannot
# see a dead producer, and a loop body that ignores status appends the rest anyway — so the block now prepares its
# inputs with checked commands, counts them against the export, and aborts at the FIRST refusal. These cases are
# failure injection: the stub gunbc refuses one append and the witness requires that nothing after it happened.
MIG_LEG=$(fence_containing 'INPUT ADMITTED')
[ -n "$MIG_LEG" ] || { echo "WITNESS REFUSED: the runbook has no step-3 block containing INPUT ADMITTED — refusing to invent one"; FAILURES=$((FAILURES+1)); }
RET_LEG=$(fence_containing 'did not take its retirement')
[ -n "$RET_LEG" ] || { echo "WITNESS REFUSED: the runbook has no step-4 block that refuses a retirement — refusing to invent one"; FAILURES=$((FAILURES+1)); }
if [ -n "$MIG_LEG" ] && [ -n "$RET_LEG" ]; then
  load_from_runbook_t RUNE 'RUNE() { local envs=() k; for k in "$@"; do envs+=(--setenv="$k"); done' 'sudo systemd-run'
  # the legs run in a CHILD bash, which has no business inventing the runbook's helpers: RUNE is handed
  # down the way bash passes functions, so what executes is the runbook's own RUNE or nothing.
  export -f RUNE
  GEXP=$TMP/migrate-export; install -d -m 0755 "$GEXP/fabric/storage"
  GTREE=$TMP/release-tree; install -d -m 0755 "$GTREE"
  GB=$TMP/gb; install -d -m 0755 "$GB"
  cat > "$GB/gunbc" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$APPEND_LOG"
n=$(wc -l < "$APPEND_LOG" | tr -d ' ')
if [ -n "${STUB_FAIL_ON:-}" ] && [ "$n" -eq "$STUB_FAIL_ON" ]; then
  echo "stub gunbc: this receipt is refused by the modeled CLI (append $n)" >&2; exit 1
fi
exit 0
EOF
  chmod +x "$GB/gunbc"
  seed_export() { # 17 acceptances + 1 revocation, the fabric tail empty, unless $1 says otherwise
    rm -f "$GEXP"/fabric/storage/*; : > "$GEXP/migrate.acceptance.lines"
    : > "$GEXP/seed.jsonl"
    for i in $(seq 1 "${1:-17}"); do
      printf '%s\n' "{\"kind\":\"acceptance_recorded\",\"node\":\"roadmap-node-$i\",\"pr_url\":\"https://github.com/gunb-ai/gunbc/pull/16$i\",\"merged_at\":\"2026-11-0${i: -1}T09:00:00Z\",\"head_sha\":\"$(printf 'a%.0s' $(seq 1 40))\",\"author\":\"principal:google-oidc/accounts/116084671989231734979\"}" >> "$GEXP/seed.jsonl"
    done
    printf '%s\n' '{"kind":"acceptance_revoked","node":"roadmap-node-9","reason":"operator revoked: it was the customer'"'"'s own row, not ours","author":"principal:google-oidc/accounts/116084671989231734979"}' >> "$GEXP/seed.jsonl"
    printf '0\n' > "$GEXP/fabric/acceptance.count"
    printf '18\n' > "$GEXP/seed.count"   # what step 1 measured; it does not move when a later edit changes the file
    [ "${2:-}" = withfabric ] || return 0
    printf '%s\n' 'fabric_storage.object;1731000000:{"payload":{"line":"roadmap-operator-acceptance;roadmap-node-18;https://github.com/gunb-ai/gunbc/pull/16200;2026-11-07T09:00:00Z;ffffffffffffffffffffffffffffffffffffffff"}}' \
      > "$GEXP/fabric/storage/obj-18"
    printf '1\n' > "$GEXP/fabric/acceptance.count"
  }
  run_migrate() { local label="$1"; shift
    : > "$TMP/appends.log"; export APPEND_LOG=$TMP/appends.log ARGV_FILE=$TMP/gargv ENV_FILE=$TMP/genv
    local out rc; out=$(env PATH="$GB:$REC:$SUDO:$SBIN:$PATH" D="$GEXP" BIN="$GB/gunbc" TREE="$GTREE" \
      OPERATOR='principal:google-oidc/accounts/116084671989231734979' "$@" bash "$TMP/migleg.sh" 2>&1); rc=$?
    echo "  [$label] rc=$rc appends=$(wc -l < "$APPEND_LOG" | tr -d ' ')"; echo "$out" | sed 's/^/    /'
    LAST_OUT="$out"; LAST_RC=$rc; LAST_APPENDS=$(wc -l < "$APPEND_LOG" | tr -d ' ')
  }
  printf '%s\n' "$MIG_LEG" > "$TMP/migleg.sh"
  printf '%s\n' "$RET_LEG" > "$TMP/retleg.sh"

  seed_export; run_migrate "the export as measured: 17 seed acceptances, empty fabric tail"
  [ "$LAST_RC" -eq 0 ] || fail "G1: the migration refused a correct export: [$LAST_OUT]"
  [ "$LAST_APPENDS" = 18 ] || fail "G1: $LAST_APPENDS appends reached the CLI, expected 17 acceptances + 1 revocation"
  echo "$LAST_OUT" | grep -qF 'INPUT ADMITTED: 17 acceptance lines (17 from the seed, 0 from the fabric tail) and 1 revocation' \
    || fail "G1: the input was not admitted by name and number: [$LAST_OUT]"
  [ "$(grep -c roadmap_acceptance_record_cli "$APPEND_LOG")" = 17 ] || fail "G1: not every acceptance went through roadmap_acceptance_record_cli"
  [ "$(grep -c roadmap_acceptance_revoke_cli "$APPEND_LOG")" = 1 ] || fail "G1: the revocation did not go through roadmap_acceptance_revoke_cli"

  seed_export; run_migrate "the third append is refused by the modeled CLI" STUB_FAIL_ON=3
  [ "$LAST_RC" -ne 0 ] || fail "G2: an append that was refused did not stop the migration — this is the reported defect"
  echo "$LAST_OUT" | grep -qF 'MIGRATION REFUSED: acceptance 3 of 17' \
    || fail "G2: the refusal did not say WHICH append failed: [$LAST_OUT]"
  [ "$LAST_APPENDS" = 3 ] || fail "G2: it kept appending after the refusal — $LAST_APPENDS appends, expected the run to stop at 3"

  seed_export 16; run_migrate "the seed carries 16 acceptances where the export says 17"
  [ "$LAST_RC" -ne 0 ] || fail "G3: an input cut short was accepted; the migration would have appended 16 and reported success"
  echo "$LAST_OUT" | grep -qF 'the file changed after step 1' \
    || fail "G3: the refusal did not name the short input: [$LAST_OUT]"
  [ "$LAST_APPENDS" = 0 ] || fail "G3: it appended $LAST_APPENDS receipt(s) although the input failed admission"

  seed_export 17 withfabric
  printf '%s\n' 'fabric_storage.object;1731000001:{"payload":{"note":"roadmap-operator-acceptance recorded here lost its line field"}}' > "$GEXP/fabric/storage/obj-19"
  printf '2\n' > "$GEXP/fabric/acceptance.count"
  run_migrate "a fabric object carries no payload.line"
  [ "$LAST_RC" -ne 0 ] || fail "G4: a fabric object with no payload.line was accepted as a receipt"
  echo "$LAST_OUT" | grep -qF 'does not carry a payload.line' || fail "G4: the refusal did not name the malformed object: [$LAST_OUT]"
  [ "$LAST_APPENDS" = 0 ] || fail "G4: it appended $LAST_APPENDS receipt(s) although the fabric tail failed admission"

  # the count is taken from the export file, so the extra revocation is added the way an operator would: in the seed
  seed_export 16
  printf '%s\n' '{"kind":"acceptance_revoked","node":"roadmap-node-11","reason":"second","author":"principal:x"}' >> "$GEXP/seed.jsonl"
  run_migrate "the seed carries two revocations"
  [ "$LAST_RC" -ne 0 ] || fail "G5: a second revocation was accepted without admission"
  echo "$LAST_OUT" | grep -qF 'exactly one revocation' || fail "G5: the refusal did not name the revocation count: [$LAST_OUT]"

  # STEP 4 CANNOT RUN ON A HALF-MIGRATED HISTORY — its precondition is step 3's admission files, not a memory.
  seed_export; : > "$GEXP/migrate.acceptance.lines"; export APPEND_LOG=$TMP/appends.log ARGV_FILE=$TMP/gargv ENV_FILE=$TMP/genv
  : > "$TMP/appends.log"
  OUT=$(env PATH="$GB:$REC:$SUDO:$SBIN:$PATH" D="$GEXP" BIN="$GB/gunbc" TREE="$GTREE" \
    OPERATOR='principal:google-oidc/accounts/116084671989231734979' bash "$TMP/retleg.sh" 2>&1); RC=$?
  echo "  [step 4 with step 3's admission file missing] rc=$RC"; echo "$OUT" | sed 's/^/    /'
  [ "$RC" -ne 0 ] || fail "G6: the retirements ran although step 3 never admitted its input"
  echo "$OUT" | grep -qF 'did not admit its input' || fail "G6: the refusal did not name step 3's admission: [$OUT]"
  [ ! -s "$APPEND_LOG" ] || fail "G6: it retired something: $(wc -l < "$APPEND_LOG") append(s)"

  seed_export; printf 'x\n' > "$GEXP/migrate.acceptance.lines"; : > "$GEXP/migrate.revocation.lines"
  : > "$TMP/appends.log"
  OUT=$(env PATH="$GB:$REC:$SUDO:$SBIN:$PATH" D="$GEXP" BIN="$GB/gunbc" TREE="$GTREE" \
    OPERATOR='principal:google-oidc/accounts/116084671989231734979' bash "$TMP/retleg.sh" 2>&1); RC=$?
  [ "$RC" -ne 0 ] || fail "G7: the retirements ran with an empty revocation file"
  [ ! -s "$APPEND_LOG" ] || fail "G7: it retired something: $(wc -l < "$APPEND_LOG") append(s)"
fi
echo

echo "================ PART I: the hand-shell LOC rows agree with the carriers they describe ================"
# The LOC rows are the scaffold's own account of how much temporary shell it introduces — F5's admission is about
# exactly these numbers, so a row that has quietly gone stale is worse than no row: an operator approves a figure
# that is no longer the file. The counting rule is the model's (`wc -l` of each carrier, every line of which this
# PR adds, all three new); this checks the promised figure against the file it describes. A PASS prints one fixed
# line, with no numbers in it, so the transcript embedded in the runbook cannot make the runbook grow and invalidate
# the very figure it records.
SELF_I=${BASH_SOURCE[0]:-$0}   # this file witnesses its own length too, so it needs its own path here
loc_of() { wc -l < "$1" | tr -d ' '; }
model_loc() { # the Int behind `data <field>: Int = N`
  awk -v f="data $1: Int = " 'index($0, f) == 1 { print substr($0, length(f) + 1) }' "$MODEL"
}
LOC_BAD=0
for pair in "runbook_loc_delta_net:$RB" \
            "runbook_witness_loc_delta_net:$SELF_I" \
            "dispatch_witness_loc_delta_net:$(cd "$(dirname "$RB")/../.." && pwd)/tools/roadmap_cutover_dispatch_witness.sh"; do
  field=${pair%%:*}; file=${pair#*:}
  [ -f "$file" ] || { echo "    LOC row [$field] describes no file at [$file]"; LOC_BAD=1; continue; }
  promised=$(model_loc "$field"); measured=$(loc_of "$file")
  if [ "$promised" != "$measured" ]; then
    echo "    LOC row [$field] says $promised for $file, the file measures $measured lines"
    LOC_BAD=1
  fi
done
[ "$LOC_BAD" -eq 0 ] || fail "PART I: a hand-shell LOC row no longer describes its carrier — re-measure it and say so"
[ "$LOC_BAD" -eq 0 ] && echo "  [the three LOC rows agree with their carriers] OK"
true
echo

echo "================ PART H: the cases this file declares are the model's rows, joined both ways ================"
# The model lists what has to be witnessed; this file runs it. Until now nothing connected the two lists, so a
# model row could promise a case no script runs, and a case could run that the model never authorised. The row
# names are the join key and they are READ from the model, never retyped here; and a refusing row is satisfied
# only if its promised diagnostic actually appears in this file or in the runbook — a promise has to be kept in
# text that executes, not only written down above.
SELF_I=${BASH_SOURCE[0]:-$0}
SELF_H=${BASH_SOURCE[0]:-$0}
model_case_rows() { # name|must|refusal_text for every gunbc.witness.roadmap_cutover case row of carrier runbook_witness_carrier
  awk -v car="runbook_witness_carrier" '
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
[ -n "$ROWS" ] || fail "the join is empty: not one case row could be read from [$MODEL] — an empty read must not pass"
[ -n "$DECL" ] || fail "the join is empty: this file declares no cases at all"
while IFS='|' read -r n m r; do
  [ -n "$n" ] || continue
  printf '%s\n' "$DECL" | grep -qx -- "$n" \
    || fail "the model demands case [$n] of this carrier and this file does not declare it"
  if [ "$m" = refuse ]; then
    [ -n "$r" ] || fail "refusing case [$n] promises no diagnostic"
    grep -qF -- "$r" "$SELF_H" || grep -qF -- "$r" "$RB" \
      || fail "case [$n] promises the diagnostic [$r] and neither this witness nor the runbook prints it"
  elif [ "$m" = admit ]; then
    [ -z "$r" ] || fail "admitting case [$n] carries a refusal string — which is it?"
  else
    fail "case [$n] has must: [$m], which is neither admit nor refuse"
  fi
done <<<"$ROWS"
while read -r n; do
  [ -n "$n" ] || continue
  printf '%s\n' "$ROWS" | cut -d'|' -f1 | grep -qx -- "$n" \
    || fail "this file declares case [$n], which the model has never heard of"
done <<<"$DECL"
echo "model rows for this carrier: $(printf '%s\n' "$ROWS" | grep -c .)   declared here: $(printf '%s\n' "$DECL" | grep -c .)"
echo
if [ "$FAILURES" -eq 0 ]; then echo "ALL ASSERTIONS HOLD"; else echo "$FAILURES ASSERTION(S) FAILED"; fi
exit "$FAILURES"
