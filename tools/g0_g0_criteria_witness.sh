#!/usr/bin/env bash
# HAND-SHELL ENFORCEMENT PROBE for the G0 contained-development-execution criteria proposal (PR #16989).
#
# SCAFFOLD (see the on-carrier marker + justification, named here for the reader of this file):
#   The DECISION RULE of this probe is modeled in .dag and is the real consumer:
#     gunbc.roadmap.g0_contained_dev_execution_proof::g0_two_sided_isolation_holds
#     executed by test.claim.roadmap.g0_criteria_witness::g0_isolation_predicate_accepts_enforced_and_rejects_unmasked
#   The bash-emit capability + dissolution trigger are carried by
#     gunbc.roadmap.g0_contained_dev_execution_proof::g0_bwrap_isolation_bash_emit_dissolution_trigger (std.disposition.Scaffold).
#   WHY THIS LEG CANNOT BE A .dag CONSUMER: producing the three observations the rule consumes (own op
#   permitted; protected/sibling reachable or not) requires EXECUTING a real kernel mount-namespace
#   primitive (bwrap --tmpfs masking). A .dag typecheck or a source grep cannot perform that primitive
#   (gunbc.design_argument::fail_closed). This file only RUNS bwrap and records the per-probe rc; the
#   truth predicate lives in the modeled consumer above. The count and capability-key checks are NOT here
#   any more: they are .dag consumers in test.claim.roadmap.g0_criteria_witness (the reviewer's FINDING 2).
#
# Run on srv1 under the shared limiter, capture the wrapper acquisition line as the FIRST line of the log:
#   HEAVY_OWNER=<session> ~/bin/heavy-slot bash tools/g0_g0_criteria_witness.sh
set -u
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

# ── two-sided hermetic isolation probe (bwrap mount-namespace masking) ─────────────────────────────
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

# NAMED negative case [unmasked_boundary]: run the SAME protected/sibling probes WITHOUT the mask; they
# must reach the fixture (rc 0), proving the probe really reads/execs it and only the boundary denies it.
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

# PASS requires: the enforced bwrap itself succeeded (rc 0 -> masking really happened) and the unmasked
# negative run succeeded (the probes reached the fixture). Per-probe denials (NEG_*_rc != 0 under the
# mask, == 0 without it) are printed above for the reviewer; the rule that combines them is the modeled
# .dag consumer named in the header.
if [ "$bwrap_enf_rc" -eq 0 ] && [ "$bwrap_unm_rc" -eq 0 ]; then
  echo "ISOLATION_PROBE=PASS (two-sided: own op permitted, protected+sibling read/exec denied by mount-namespace masking; NAMED negative [unmasked_boundary] reaches sibling+sibling-exec -> probe discriminates enforced from unenforced)"
else
  echo "ISOLATION_PROBE=INCONCLUSIVE (bwrap enforcement rc=$bwrap_enf_rc unmasked rc=$bwrap_unm_rc): primitive unavailable -> isolation NOT proven; see USERNS_PROBE"
  overall_rc=1
fi

rm -rf "$W"
echo "OVERALL_RC=$overall_rc"
exit $overall_rc
