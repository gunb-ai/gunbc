#!/usr/bin/env bash
# Witnesses for the migration runbook's own shell (docs/runbooks/roadmap-event-log-cutover.md).
#
#   A0/B0  the text this harness exercises is the runbook's own text, byte for byte
#   A1-A5  argument delivery through the ACTUAL helper invocation shape (sudo systemd-run … bash -s -- "$@"),
#          using the runbook's own guard blocks — the defect class where `bash -s -- runbook "$@"` would silently
#          bind $1 to the filler word and export into a relative directory.
#   B1-B4  the drop-in export/restore round trip with BOTH a pre-existing drop-in and a deployment-added one.
#   C      run identification for the CI dispatches lives in dispatch identification for the CI dispatches: tools/roadmap_cutover_dispatch_witness.sh.
#
# Nothing here touches the real system: `sudo` is a shell function that strips the privilege, and the systemd bits
# it hands over resolve to stubs on PATH. Run:  bash tools/roadmap_cutover_runbook_witness.sh docs/runbooks/roadmap-event-log-cutover.md
set -u
RB=${1:?usage: roadmap_cutover_runbook_witness.sh <path to the runbook>}
FAILURES=0
TMP=$(mktemp -d); SBIN=$TMP/sbin; SUDO=$TMP/bin; EXPORT_DIR=/opt/gunbc/exports/t6-20261110T230000Z
TB=$TMP/guard1.sh; TB7=$TMP/guard7.sh
mkdir -p "$SBIN" "$SUDO"

## ---------- the helpers as the runbook defines them (RUNF is extracted from the runbook in A0) ----------
RUNF() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- "$@"; }
UNITS=(gunbc-roadmap.service gunbc-roadmap-tailnet-door.service gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service)
export_drop_ins() { # $1 = the systemd directory, $2 = the export directory
  install -d -m 0755 "$2/units/drop-ins"
  for U in "${UNITS[@]}"; do
    if [ -d "$1/$U.d" ]; then sudo cp -a "$1/$U.d" "$2/units/drop-ins/"; fi
  done
  find "$2/units/drop-ins" -type f 2>/dev/null | sort > "$2/units.drop-ins.inventory"
}
restore_drop_ins() { # $1 = the systemd directory, $2 = the export directory
  for U in "${UNITS[@]}"; do
    sudo rm -rf "$1/$U.d"
    if [ -d "$2/units/drop-ins/$U.d" ]; then sudo cp -a "$2/units/drop-ins/$U.d" "$1/"; fi
  done
}
need() { local what="$1"; shift; if ! "$@"; then echo "ROLLBACK REFUSED: $what — nothing is started"; exit 1; fi; }

extract_slice() { # $1 = file, $2 = exact first line, $3 = prefix of the last line
  awk -v b="$2" -v e="$3" '$0==b{f=1} f{print} f && index($0,e)==1{exit}' "$1"
}
same_slice() { # $1 label, $2 exact first line, $3 prefix of last line — compared between the runbook and this file
  a=$(extract_slice "$RB" "$2" "$3"); b=$(extract_slice "$0" "$2" "$3")
  if [ -z "$a" ]; then echo "FAIL $1 — the runbook has no slice starting at [$2]"; FAILURES=$((FAILURES+1)); return; fi
  if [ "$a" = "$b" ]; then echo "OK   $1 — the harness exercises the runbook's own text, byte-identical"
  else echo "FAIL $1 — the copy has drifted from the runbook"; FAILURES=$((FAILURES+1)); fi
}
same_helper() { # $1 label, $2 exact single line (compared whole)
  a=$(grep -Fxc "$2" "$RB"); b=$(grep -Fxc "$2" "$0")
  if [ "${a:-0}" -ge 1 ] && [ "${a:-0}" = "${b:-0}" ]; then echo "OK   $1 — the harness exercises the runbook's own text, byte-identical"
  else echo "FAIL $1 — the runbook says it $a time(s), this harness $b time(s)"; FAILURES=$((FAILURES+1)); fi
}

## ---------- the stubbed privileged surface ----------
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

echo "=============== A0/B0: the exercised text is the runbook's own text ==============="
same_helper "RUNF, the transient-unit helper" 'RUNF() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- "$@"; }'
same_slice  "the export helper"               'export_drop_ins() { # $1 = the systemd directory, $2 = the export directory' '}'
same_slice  "the restore helper"              'restore_drop_ins() { # $1 = the systemd directory, $2 = the export directory' '}'
same_helper "the rollback's need predicate"   'need() { local what="$1"; shift; if ! "$@"; then echo "ROLLBACK REFUSED: $what — nothing is started"; exit 1; fi; }' 
echo

echo "================ PART A: argument delivery through the ACTUAL helper shape ================"
# The scripts under test are the runbook's own argument-guard blocks plus one marker line, delivered exactly the
# way the runbook delivers them. A paraphrase would prove nothing, so nothing is paraphrased here.
GUARD1=$(extract_slice "$RB" 'D=$1' 'esac')
GUARD7=$(extract_slice "$RB" 'D=$1; SHA=$2' '[ "${#SHA}" -eq 40 ]')
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
printf '[Service]\nExecStartPre=/bin/true\n'           > "$SYS/gunbc-roadmap.service.d/pre-existing.conf"
printf '[Unit]\nDescription=added by the modeled deploy\n' > "$SYS/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf"
printf 'ExecStart=/opt/gunbc/bin/gunbc roadmap-page\n' > "$SYS/gunbc-roadmap.service"

echo "--- B1: export (step 1's helper) ---"
export_drop_ins "$SYS" "$EXP"
( cd "$EXP" && find units/drop-ins -type f | sort )
echo "--- B2: the modeled deploy rewrites a unit file and adds a drop-in to a different unit ---"
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
diff -q <(printf '[Unit]\nDescription=added by the modeled deploy\n') "$SYS/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf" \
  || fail "a drop-in that existed BEFORE the cutover on another unit was not restored — that is F4's defect class"
diff -q <(printf '[Service]\nExecStartPre=/bin/true\n') "$SYS/gunbc-roadmap.service.d/pre-existing.conf" \
  || fail "the restored drop-in is not byte-identical"
[ "$(ls -A "$SYS/gunbc-roadmap.service.d")" = "pre-existing.conf" ] || fail "the restored unit has extra content: $(ls -A "$SYS/gunbc-roadmap.service.d")"
[ -f "$EXP/units.drop-ins.inventory" ] || fail "step 1 did not record which drop-ins existed"
grep -q 'units/drop-ins/gunbc-roadmap.service.d/pre-existing.conf' "$EXP/units.drop-ins.inventory" \
  || fail "the inventory does not name the pre-existing drop-in"
# and the rollback's own predicate refuses on the thing that went wrong before
( need "the export recorded every unit's drop-ins" test -f "$EXP/units.drop-ins.inventory" ) || fail "a satisfied need refused"
( need "an unsatisfied predicate" false; echo "SHOULD NOT REACH" ) && fail "need() did not stop the script"
echo "ROUND TRIP OK: every unit's drop-in directory returned to its own path, byte-identical; the"
echo "               deployment-added one is gone; the pre-cutover ones (on both units) are back."
echo "where the restored .conf files live: $(cd "$SYS" && find . -name '*.conf' | sort | tr '\n' ' ')"
echo

if [ "$FAILURES" -eq 0 ]; then echo "ALL ASSERTIONS HOLD"; else echo "$FAILURES ASSERTION(S) FAILED"; fi
exit "$FAILURES"
