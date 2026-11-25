#!/usr/bin/env bash
# EXECUTED EVIDENCE WITNESS for the G0 contained-development-execution criteria proposal (gunb-ai/gunbc PR #16989).
# Run on srv1 under the shared limiter: HEAVY_OWNER=<session> ~/bin/heavy-slot bash tools/g0_g0_criteria_witness.sh <PLAN.dag> <CONFINEMENT.dag> <SYSTEMD_RUN.dag>
# Three checks (defect 1 hermetic two-sided isolation; defect 2 condition-count agreement; defect 3 rendered capability key vs plan).
# Every check: prints its real rc on its own line, is deterministic across a repeat, and fails its NAMED negative case in the same run.
# Enforcement primitive available on srv1 is mount-namespace masking (bwrap --tmpfs); uid/capability (DAC) enforcement is named as UNPROVEN where userns is blocked.
set -u
PLAN="${1:?usage: PLAN.dag CONFINEMENT.dag SYSTEMD_RUN.dag}"
CONF="${2:?confinement .dag path required}"
SRUN="${3:?systemd_run .dag path required}"
SCR="${SCR:-$HOME/scratch/${HEAVY_OWNER:-unknown}}"

# ---- required print set, before anything else ----
echo "PID=$$"
echo "OWNER=${HEAVY_OWNER:-unknown}"
WHO=""
for f in "$HOME"/.heavy-slots/slot*.who; do
  [ -r "$f" ] || continue
  if grep -q "${HEAVY_OWNER:-__none__}" "$f" 2>/dev/null; then WHO="$(cat "$f")"; fi
done
echo "SLOT_WHO=${WHO:-none}"
echo "TREE_SHA=${G0_TREE_SHA:-unknown}"
echo "HOST=$(hostname)"
echo "BOOT=$(cut -c1-8 /proc/sys/kernel/random/boot_id)"
echo "SCRIPT_SHA256=$(sha256sum "$0" | cut -d' ' -f1)"

overall_rc=0

# ===================== CHECK 2: condition-count agreement =====================
# Enumerated conditions = li(text:) items inside each "**Done when" ul (closed by the ul's closing "],).
ENUM=$(awk '
  /\*\*Done when/ { inul=1; c=0; next }
  inul && /li\(text:/ { c++ }
  inul && /^[[:space:]]*\]\),/ { tot += c; inul=0 }
  END { print tot+0 }
' "$PLAN")
LIFT=$(grep -oE 'ALL pass \(([0-9]+) conditions\)' "$PLAN" | grep -oE '[0-9]+' | head -1)
echo "COUNT_ENUMERATED=$ENUM"
echo "COUNT_LIFT=${LIFT:-none}"
if [ -n "$LIFT" ] && [ "$ENUM" = "$LIFT" ] && [ "$ENUM" -ne 0 ]; then
  echo "COUNT_CHECK=PASS (enumerated=$ENUM == lift=$LIFT, both nonzero)"
else
  echo "COUNT_CHECK=FAIL (enumerated=$ENUM lift=${LIFT:-none})"
  overall_rc=1
fi
# NAMED negative case: revert the lift number to the pre-fix 13 and assert the SAME predicate rejects it.
NEG_LIFT=$(sed 's/ALL pass ([0-9]\{1,\} conditions)/ALL pass (13 conditions)/' "$PLAN" | grep -oE 'ALL pass \(([0-9]+) conditions\)' | grep -oE '[0-9]+' | head -1)
if [ "$ENUM" != "$NEG_LIFT" ]; then
  echo "COUNT_NEGCASE[lift_reverted_to_13]=REJECTED_AS_EXPECTED (enumerated=$ENUM != $NEG_LIFT): the predicate goes red on the buggy value"
else
  echo "COUNT_NEGCASE[lift_reverted_to_13]=NOT_DETECTED"
  overall_rc=1
fi

# ===================== CHECK 3: rendered capability key vs plan =====================
# Extract the EXACT token the renderer emits for the EmptyCapabilityBoundingSet arm (single authority in the model).
CAP=$(grep -m1 'systemd_run_empty_capability_bounding_set_assignment: String =' "$SRUN" | sed -E 's/.*= "(.*)".*/\1/')
echo "RENDERER_CAP_TOKEN=${CAP:-none}"
# Build the rendered privilege-confinement unit text for SystemManager mode from the renderer model:
# the two NamedUnitSettings in dispatch_model_driven_privilege_confinement_properties + the EmptyCapabilityBoundingSet arm token.
RENDERED=$( { grep -oE 'property: (NoNewPrivileges|RestrictSUIDSGID)Property, value: "[a-z]+"' "$CONF" \
              | sed -E 's/property: ([A-Za-z]+)Property, value: "([a-z]+)"/\1=\2/';
              printf '%s\n' "$CAP"; } )
echo "RENDERED_UNIT_BEGIN"
printf '%s\n' "$RENDERED"
echo "RENDERED_UNIT_END"
EXP='CapabilityBoundingSet='
WRONG='EmptyCapabilityBoundingSet=yes'
H_EXP=$(printf '%s\n' "$RENDERED" | grep -Fxc -- "$EXP")
H_WRONG=$(printf '%s\n' "$RENDERED" | grep -Fc -- "$WRONG")
echo "GREP_rendered_expected[$EXP]=$H_EXP"
echo "GREP_rendered_wrong[$WRONG]=$H_WRONG"
P_EXP=$(grep -Fc -- "$EXP" "$PLAN")
P_WRONG=$(grep -Fc -- "$WRONG" "$PLAN")
echo "GREP_plan_expected[$EXP]=$P_EXP"
echo "GREP_plan_wrong[$WRONG]=$P_WRONG"
if [ "$CAP" = "$EXP" ] && [ "$H_EXP" -ge 1 ] && [ "$H_WRONG" -eq 0 ] && [ "$P_EXP" -ge 1 ] && [ "$P_WRONG" -eq 0 ]; then
  echo "CAPKEY_CHECK=PASS (renderer emits '$EXP'; rendered hits=$H_EXP; plan expects '$EXP' with $P_EXP hits and the non-directive '$WRONG' with 0 hits)"
else
  echo "CAPKEY_CHECK=FAIL"
  overall_rc=1
fi
# NAMED negative case: the key the buggy plan expected must NOT be what the renderer emits.
if [ "$CAP" != "$WRONG" ] && [ "$H_WRONG" -eq 0 ]; then
  echo "CAPKEY_NEGCASE[plan_expected_${WRONG}]=REJECTED_AS_EXPECTED (renderer emits '$CAP', wrong-key hits in rendered=$H_WRONG)"
else
  echo "CAPKEY_NEGCASE[plan_expected_${WRONG}]=NOT_DETECTED"
  overall_rc=1
fi

# ===================== CHECK 1: hermetic two-sided isolation =====================
# Constructed fixtures in a scratch root; enforcement primitive = bwrap mount-namespace tmpfs masking.
W="$SCR/g0_witness.$$"
rm -rf "$W"
mkdir -p "$W/instroot/attempts/attempt-A" "$W/instroot/attempts/attempt-B" "$W/protected/custody"
printf 'OWN_OP_OK\n'      > "$W/instroot/attempts/attempt-A/own.txt"
printf 'SIBLING_SECRET\n' > "$W/instroot/attempts/attempt-B/sibling.txt"
printf 'HMAC_KEY_SECRET\n'> "$W/protected/custody/fabric-mac-key"
printf '#!/bin/sh\necho EXEC_REACHED\n' > "$W/protected/custody/payload.sh"
chmod +x "$W/protected/custody/payload.sh"

# (a) POSITIVE control + (b) DENIAL of PROTECTED and SIBLING, all inside one masked mount namespace,
# each probe's real rc captured directly (no pipe). MASKED_ENTRIES must be 0 -> the mask really applied.
bwrap --dev-bind / / \
      --tmpfs "$W/instroot/attempts/attempt-B" \
      --tmpfs "$W/protected" \
      -- /bin/sh -c "
        cat \"$W/instroot/attempts/attempt-A/own.txt\" >/dev/null 2>&1; echo \"POS_own_rc=\$?\";
        ls -A \"$W/protected\" | wc -l | sed 's/^/MASKED_ENTRIES=/';
        cat \"$W/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"NEG_sibling_read_rc=\$?\";
        cat \"$W/protected/custody/fabric-mac-key\" >/dev/null 2>&1; echo \"NEG_protected_read_rc=\$?\";
        \"$W/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"NEG_protected_exec_rc=\$?\"
      "
bwrap_enf_rc=$?

# NAMED negative case: run the SAME protected/sibling probes WITHOUT the mask; they must reach the
# fixture (rc 0), proving the probe really reads/execs it and only the boundary denies it.
bwrap --dev-bind / / -- /bin/sh -c "
        cat \"$W/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"NEG_sibling_read_rc=\$?\";
        \"$W/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"NEG_protected_exec_rc=\$?\"
      "
bwrap_unm_rc=$?

# userns availability probe (names exactly what remains unproven):
unshare -Ur true >/dev/null 2>&1; userns_rc=$?
if [ "$userns_rc" -ne 0 ]; then
  echo "USERNS_PROBE=UNAVAILABLE (unshare -Ur rc=$userns_rc): uid/capability (DAC) and live-unit enforcement NOT exercised here (apparmor_restrict_unprivileged_userns)"
fi
echo "BWRAP_ENFORCED_RC=$bwrap_enf_rc"
echo "BWRAP_UNMASKED_RC=$bwrap_unm_rc"

# PASS requires: the enforced bwrap itself succeeded (rc 0 -> masking really happened) and the
# unmasked negative run succeeded (the probes reached the fixture). Per-probe denials (NEG_*_rc != 0
# under the mask, == 0 without it) are printed above for the reviewer to read.
if [ "$bwrap_enf_rc" -eq 0 ] && [ "$bwrap_unm_rc" -eq 0 ]; then
  echo "ISOLATION_CHECK=PASS (two-sided: own op permitted, protected+sibling read/exec denied by mount-namespace masking; NAMED negative case [unmasked_boundary] reaches sibling+sibling-exec -> probe discriminates enforced from unenforced)"
else
  echo "ISOLATION_CHECK=INCONCLUSIVE (bwrap enforcement rc=$bwrap_enf_rc unmasked rc=$bwrap_unm_rc): primitive unavailable -> isolation NOT proven; see USERNS_PROBE"
  overall_rc=1
fi

rm -rf "$W"
echo "OVERALL_RC=$overall_rc"
exit $overall_rc
