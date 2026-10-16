#!/usr/bin/env bash
# Evidence runner for PR #14168 (merry-fox-82's gate): execute every .dag claim my change touches, on a
# fresh remote build, and falsify each key law against a one-line broken variant.
#
# WHY THIS IS COMMITTED AND NOT PIPED IN: ctrl-build --remote feeds the command over its own stdin, so a
# heredoc redirected into it deadlocks the dispatch (three dispatches hung in the fetch stage before this
# file existed). Committed, the run is reproducible by anyone:
#   ctrl-build --remote --no-fallback --timeout 5h -- bash scripts/recovery_evidence_run.sh
#
# WHY EVERY ENTRY THAT IMPORTS A MODULE I EDITED, AND NOT JUST MY TWO FILES: heal-generated-artifacts is
# not a required check, so asrock_rack.dag, ipmi.dag and redfish.dag sit OUTSIDE the required gate's
# closure -- this run is their only execution evidence. Every test entry that imports an edited module runs
# its claims; every production importer at least gets a compile check.
set -uo pipefail
export GITHUB_ACTIONS=1

# Bound the cgroup before anything spawns: the runner is shared, and an unbounded cargo build is somebody
# else's OOM.
mkdir -p /sys/fs/cgroup/x 2>/dev/null || true
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
echo 30064771072 > /sys/fs/cgroup/x/memory.max 2>/dev/null || true
echo $$ > /sys/fs/cgroup/x/cgroup.procs 2>/dev/null || true

echo "=== BUILD (release; one dispatch, because the runs below execute what this builds) ==="
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -5

ENTRIES="$(cat scripts/recovery-entries.txt)"

run_claims () {
  ./target/release/gunbc run --source-root dag --source-root src/v1 --source-root src/v2 \
    --entry "$1" --claim-run 2>&1 \
    | grep -iE "GREEN|RED|FAIL|parse-failed|error|unbound|not found|refused" | head -40
}

echo
echo "===== COMPILE EVERY ENTRY ====="
for E in $ENTRIES; do
  out=$(./target/release/gunbc compile --source-root dag --entry "$E" --output-dir "/tmp/out-$$" --target dag 2>&1 \
        | grep -iE "error|refused|parse-failed|unbound|not found|cannot|expected" | head -8)
  if [ -z "$out" ]; then
    echo "COMPILE-OK   $E"
  else
    echo "COMPILE-FAIL $E"
    echo "$out"
  fi
done

echo
echo "===== CLAIM RUN, EVERY TEST ENTRY ====="
for E in $ENTRIES; do
  case "$E" in dag/test/*) ;; *) continue ;; esac
  echo "--- $E"
  run_claims "$E"
done

MOD=dag/gunbc/machine_intake/bmc_recovery_route.dag
WIT=dag/test/claim/machine_intake/machine_intake_bmc_recovery_route_witness.dag
IPMI=dag/extdeps/bmc/ipmi.dag
CEN=dag/gunbc/auth/privileged_effect_census.dag
RED=dag/extdeps/bmc/redfish.dag
ARR=dag/gunbc/machine_intake/arrival_converge.dag

# falsify LABEL FILE OLD NEW ENTRY -- break one line, require the claim to go RED, restore.
falsify () {
  echo
  echo "##### FALSIFY: $1"
  MUT_FILE="$2" MUT_OLD="$3" MUT_NEW="$4" python3 <<'PY' || { echo "   MUTATION-TARGET-NOT-FOUND (the law is not where the claim says it is)"; git checkout -- "$2"; return; }
import os, sys
p, o, n = os.environ["MUT_FILE"], os.environ["MUT_OLD"], os.environ["MUT_NEW"]
s = open(p).read()
if s.count(o) != 1:
    print("   (found %d occurrences -- refusing to mutate an ambiguous target)" % s.count(o))
    sys.exit(4)
open(p, "w").write(s.replace(o, n, 1))
PY
  if ./target/release/gunbc run --source-root dag --source-root src/v1 --source-root src/v2 \
       --entry "$5" --claim-run 2>&1 | grep -qE "RED|FAIL|failed"; then
    echo "   RESULT: RED as required"
  else
    echo "   RESULT: *** STILL GREEN -- the claim does not test what it claims ***"
  fi
  git checkout -- "$2"
}

echo
echo "===== FALSIFICATION: every key law must go RED on a one-line break ====="

falsify "0xC7-for-0xCC: the completion code the destructive reset is justified by" "$IPMI" \
  "data ipmi_completion_code_request_data_length_invalid: IpmiCompletionCode = IpmiCompletionCode { code: 199 }" \
  "data ipmi_completion_code_request_data_length_invalid: IpmiCompletionCode = IpmiCompletionCode { code: 204 }" \
  "$WIT"

falsify "the identity join stops checking Manufacturer ID (a foreign board could open the measured row)" "$IPMI" \
  "  a.manufacturer_id == b.manufacturer_id
    && a.product_id == b.product_id" \
  "  a.product_id == b.product_id" \
  "$MOD"

falsify "the identity join stops checking Firmware Revision minor (an unmeasured release gets promoted)" "$IPMI" \
  "    && a.firmware_revision_minor == b.firmware_revision_minor" \
  "    && true" \
  "$MOD"

falsify "the readback stops requiring the PasswordChangeRequired id (an unparseable body, e.g. a bare 401, would prove a reset)" "$RED" \
  "    JsonDocumentUnreadable { gap: _ } => RedfishPasswordChangeNotIndicated" \
  "    JsonDocumentUnreadable { gap: _ } => RedfishPasswordChangeRequired" \
  "$MOD"

falsify "the argv credential predicate stops being read (a password on the command line reads as passwordless)" "$MOD" \
  "InBandChannelPasswordless { transport: _, argv_credential_words: words } => list_length(items: words) == 0" \
  "InBandChannelPasswordless { transport: _, argv_credential_words: _ } => true" \
  "$MOD"

falsify "the transport parse fails open (any name becomes an in-band transport)" "$IPMI" \
  '  else if wire == "lanplus" {' \
  '  else if wire == "open" {' \
  "$MOD"

falsify "the netboot provider is treated as executed (a path that never served a host stands as proof)" "$MOD" \
  "        NetbootRescueMediumDeclared => Present { value: RescueMediumNotExecuted { medium: medium } }" \
  "        NetbootRescueMediumDeclared => none" \
  "$MOD"

falsify "the reset gate accepts a ResetToDefaultsType that keeps the user table" "$MOD" \
  "    Present { value: a } => a != ResetAll
    Absent => true" \
  "    Present { value: _ } => false
    Absent => false" \
  "$MOD"

falsify "the reset gate stops checking the durable unit hold (wipe a chassis someone else drives)" "$MOD" \
  "  } else if recovery_hold_admits_this_unit(unit_key: route.subject.subject.unit_key, hold: unit_hold) == false {" \
  "  } else if false {" \
  "$MOD"

falsify "the seam stops checking that the recovery standing is for THIS unit" "$ARR" \
  "      if (s.subject.subject.unit_key as NonEmptyStr) != (key.subject as NonEmptyStr) {" \
  "      if false {" \
  "$WIT"

falsify "the census site names a declaration that is not in the tree (review 81115 #1's defect)" "$CEN" \
  "    site: site(module_path: \"gunbc.machine_intake.bmc_recovery_route\", decl_name: \"admit_recovery_in_band_password_write\"),
    effect: recovery_in_band_set_user_password_effect()," \
  "    site: site(module_path: \"gunbc.machine_intake.bmc_recovery_route\", decl_name: \"admit_recovery_in_band_password_write_typo\"),
    effect: recovery_in_band_set_user_password_effect()," \
  "$WIT"

falsify "the password effect is softened to reversible (its authorization pattern would move silently)" "$CEN" \
  "    reversibility: IrreversibleEffect {
      what_is_lost: \"an enabled ADMINISTRATOR account left holding a password this run never stored; the exposure window is not undone by setting the password again, and its later removal is a separate actuation rather than a reapply\" as NonEmptyStr,
    }," \
  "    reversibility: ReversibleByReapply," \
  "$WIT"

falsify "a convergence-step roster names a declaration that does not exist" "$MOD" \
  '  decl_name: "recovery_standing_handed_off",' \
  '  decl_name: "recovery_standing_handed_off_typo",' \
  "$WIT"

echo
echo "===== AFTER ALL MUTATIONS: the tree must be clean and the baseline must reproduce ====="
git status --porcelain
run_claims "$MOD"
run_claims "$WIT"
echo "===== DONE ====="
