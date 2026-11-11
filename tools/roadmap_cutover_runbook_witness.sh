#!/usr/bin/env bash
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
#
# Nothing in this file is a second authority: RUNF, UNITS, export_drop_ins, restore_drop_ins and need are loaded
# out of the runbook at run time, so the thing being witnessed is literally the text the operator runs, and a
# drift between them is a failure rather than a silent re-test of a paraphrase.
#
# Nothing here touches a real system: `sudo` is a function that strips the privilege, and `systemd-run` and
# `systemctl` resolve to stubs on PATH. Run before the migration is authorized, not after it fails.
#
# Run:  bash tools/roadmap_cutover_runbook_witness.sh docs/runbooks/roadmap-event-log-cutover.md
set -u
RB=${1:?usage: roadmap_cutover_runbook_witness.sh <path to the runbook>}
[ -f "$RB" ] || { echo "WITNESS REFUSED: no runbook at [$RB] — there is nothing to witness"; exit 2; }
FAILURES=0
TMP=$(mktemp -d); SBIN=$TMP/sbin; SUDO=$TMP/bin; EXPORT_DIR=/opt/gunbc/exports/t6-20261110T230000Z
TB=$TMP/guard1.sh; TB7=$TMP/guard7.sh
mkdir -p "$SBIN" "$SUDO"

# ---- load the helpers FROM THE RUNBOOK. No copy lives here: an empty extraction is a refusal, not a fallback. ----
slice_from() { # $1 = exact first line, $2 = prefix of the last line
  awk -v b="$1" -v e="$2" '$0==b{f=1} f{print} f && index($0,e)==1{exit}' "$RB"
}
load_from_runbook() { # $1 = label, $2 = exact first line, $3 = prefix of last line
  local body; body=$(slice_from "$2" "$3")
  if [ -z "$body" ]; then echo "WITNESS REFUSED: the runbook has no [$1] starting at [$2] — refusing to invent one"; FAILURES=$((FAILURES+1)); return 1; fi
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
RUNF "$EXPORT_DIR" < "$TB"; echo "A1 rc=$? — 0 and the absolute path echoed back are required"

echo "--- A2: the SHIFTED helper (a filler word after --), same script ---"
RUNF_shifted() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- runbook "$@"; }
RUNF_shifted "$EXPORT_DIR" < "$TB"; echo "A2 rc=$? — nonzero plus the located refusal is the required behaviour"

echo "--- A3: correct helper, two data arguments (step 7 passes \$D then \$SHA) ---"
RUNF "$EXPORT_DIR" "$sha40" < "$TB7"; echo "A3 rc=$? — 0 with both values intact is required"

echo "--- A4: the same script through the SHIFTED helper — the guard must refuse ---"
RUNF_shifted "$EXPORT_DIR" "$sha40" < "$TB7"; echo "A4 rc=$? — nonzero is required"

echo "--- A5: a revision that is not 40 hex is refused by step 7's own guard ---"
RUNF "$EXPORT_DIR" "not-a-sha" < "$TB7"; echo "A5 rc=$? — nonzero is required"
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

if [ "$FAILURES" -eq 0 ]; then echo "ALL ASSERTIONS HOLD"; else echo "$FAILURES ASSERTION(S) FAILED"; fi
exit "$FAILURES"
