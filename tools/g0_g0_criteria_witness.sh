#!/usr/bin/env bash
# HAND-SHELL ENFORCEMENT PROBE for the G0 contained-development-execution criteria proposal (PR #16989).
#
# SCAFFOLD (reviewer FINDING 1): the DECISION RULE and its VERDICT are modeled in .dag and the MODEL is the
# authority -- this file does NOT decide PASS by its own exit status (a trailing `echo` always returns 0;
# gating PASS on the bwrap launch rc would be fail-open). This file only runs bwrap and records the REAL
# per-probe rc, translates them into three observations, and feeds them to the model verdict, whose process
# exit is the verdict.
#   model decision rule:   gunbc.roadmap.g0_contained_dev_execution_proof::g0_two_sided_isolation_holds
#   model verdict surface: gunbc.roadmap.g0_contained_dev_execution_proof::g0_two_sided_isolation_verdict (-> ProcessExit)
#   bash-emit capability + dissolution trigger:
#                          ...g0_contained_dev_execution_proof::g0_bwrap_isolation_bash_emit_dissolution_trigger (std.disposition.Scaffold)
#   WHY it cannot be a .dag consumer: producing the three observations requires EXECUTING a real kernel
#   mount-namespace primitive (bwrap --tmpfs masking), which no typecheck or grep can (fail_closed).
#
# Run on srv1 from the repo root (Cargo.toml beside dag/ and src/), under the shared limiter; capture the
# wrapper acquisition line as the FIRST line of the log:
#   HEAVY_OWNER=<session> ~/bin/heavy-slot bash tools/g0_g0_criteria_witness.sh
# The nested `gunbc run` calls run inside THIS heavy-slot scope (one stage); they are not separately queued.
set -u
SCR="${SCR:-$HOME/scratch/${HEAVY_OWNER:-unknown}}"
GUNBC_BIN="${GUNBC_BIN:-/opt/gunbc/bin/gunbc}"
G0_REPO="${G0_REPO:-$PWD}"
G0_MODEL="${G0_MODEL:-dag/gunbc/roadmap/g0_contained_dev_execution_proof.dag}"
G0_SR="${G0_SR:---source-root dag --source-root src/v1 --source-root src/v2}"

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
echo "GUNBC_BIN=${GUNBC_BIN}"
echo "G0_REPO=${G0_REPO}"

overall_rc=0

model_available=true
if [ ! -x "$GUNBC_BIN" ] || [ ! -f "$G0_REPO/Cargo.toml" ] || [ ! -f "$G0_REPO/$G0_MODEL" ]; then
  model_available=false
  echo "MODEL_UNAVAILABLE=gunbc or repo/model root missing (GUNBC_BIN=$GUNBC_BIN G0_REPO=$G0_REPO G0_MODEL=$G0_MODEL)"
fi

# run the model verdict with the observed booleans; echoes its process exit on its own line.
run_model_verdict() {
  local own="$1" prot="$2" sib="$3" tag="$4"
  ( cd "$G0_REPO" && "$GUNBC_BIN" run --entry "$G0_MODEL" $G0_SR --function g0_two_sided_isolation_verdict \
      --arg own_op_permitted="$own" --arg protected_reachable="$prot" --arg sibling_reachable="$sib" ) \
      > "$SCR/model_${tag}.log" 2>&1
  return $?
}

# ── two-sided hermetic isolation probe (bwrap mount-namespace masking) ─────────────────────────────
W="$SCR/g0_witness.$$"; W2="$SCR/g0_neg.$$"
rm -rf "$W" "$W2"
mk_tree() {
  local base="$1"
  mkdir -p "$base/instroot/attempts/attempt-A" "$base/instroot/attempts/attempt-B" "$base/protected/custody"
  printf 'OWN_OP_OK\n'       > "$base/instroot/attempts/attempt-A/own.txt"
  printf 'SIBLING_SECRET\n'  > "$base/instroot/attempts/attempt-B/sibling.txt"
  printf 'HMAC_KEY_SECRET\n' > "$base/protected/custody/fabric-mac-key"
  printf '#!/bin/sh\necho EXEC_REACHED\n' > "$base/protected/custody/payload.sh"
  chmod +x "$base/protected/custody/payload.sh"
}
mk_tree "$W"

# re-run capturing stdout to parse the per-probe rc deterministically (the rc each probe echoed, not the echo's rc)
cap=$(bwrap --dev-bind / / \
      --tmpfs "$W/instroot/attempts/attempt-B" --tmpfs "$W/protected" \
      -- /bin/sh -c "
        cat \"$W/instroot/attempts/attempt-A/own.txt\" >/dev/null 2>&1; echo \"POS_own_rc=\$?\";
        ls -A \"$W/protected\" | wc -l | tr -d ' ' | sed 's/^/MASKED_ENTRIES=/';
        cat \"$W/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"NEG_sibling_read_rc=\$?\";
        cat \"$W/protected/custody/fabric-mac-key\" >/dev/null 2>&1; echo \"NEG_protected_read_rc=\$?\";
        \"$W/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"NEG_protected_exec_rc=\$?\"
      ")
bwrap_enf_rc=$?
capu=$(bwrap --dev-bind / / -- /bin/sh -c "
        cat \"$W/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"POSCTL_sibling_read_rc=\$?\";
        \"$W/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"POSCTL_protected_exec_rc=\$?\"
      ")
bwrap_unm_rc=$?
echo "ENFORCED_OBS:"; printf '%s\n' "$cap" | sed 's/^/  /'
echo "UNMASKED_OBS:"; printf '%s\n' "$capu" | sed 's/^/  /'
echo "BWRAP_ENFORCED_RC=$bwrap_enf_rc"
echo "BWRAP_UNMASKED_RC=$bwrap_unm_rc"

POS_own_rc=$(           printf '%s\n' "$cap"  | sed -n 's/^POS_own_rc=\([0-9]*\)$/\1/p')
MASKED_ENTRIES=$(       printf '%s\n' "$cap"  | sed -n 's/^MASKED_ENTRIES=\([0-9]*\)$/\1/p')
NEG_sibling_read_rc=$(  printf '%s\n' "$cap"  | sed -n 's/^NEG_sibling_read_rc=\([0-9]*\)$/\1/p')
NEG_protected_read_rc=$(printf '%s\n' "$cap"  | sed -n 's/^NEG_protected_read_rc=\([0-9]*\)$/\1/p')
NEG_protected_exec_rc=$(printf '%s\n' "$cap"  | sed -n 's/^NEG_protected_exec_rc=\([0-9]*\)$/\1/p')
POSCTL_sibling_read_rc=$( printf '%s\n' "$capu" | sed -n 's/^POSCTL_sibling_read_rc=\([0-9]*\)$/\1/p')
POSCTL_protected_exec_rc=$(printf '%s\n' "$capu" | sed -n 's/^POSCTL_protected_exec_rc=\([0-9]*\)$/\1/p')

# ---- derive the three observations + the positive control ----
own_perm=false; [ "$POS_own_rc" = "0" ] && own_perm=true
sib_reach=true;  [ "$NEG_sibling_read_rc" != "0" ] && sib_reach=false
prot_reach=true; if [ "$NEG_protected_read_rc" != "0" ] && [ "$NEG_protected_exec_rc" != "0" ]; then prot_reach=false; fi
mask_ok=false;   [ "$MASKED_ENTRIES" = "0" ] && mask_ok=true
posctl=false;    if [ "$POSCTL_sibling_read_rc" = "0" ] && [ "$POSCTL_protected_exec_rc" = "0" ]; then posctl=true; fi
echo "OBS_own_op_permitted=$own_perm"
echo "OBS_protected_reachable=$prot_reach"
echo "OBS_sibling_reachable=$sib_reach"
echo "OBS_mask_applied=$mask_ok"
echo "OBS_positive_control_reached=$posctl"

# ---- the MODEL consumes the real observations; its process exit is the verdict ----
MODEL_RC=127
if [ "$model_available" = true ]; then
  run_model_verdict "$own_perm" "$prot_reach" "$sib_reach" good; MODEL_RC=$?
  echo "MODEL_VERDICT_RC=$MODEL_RC"
else
  echo "MODEL_VERDICT_RC=127 (model unavailable)"
fi

# ---- NAMED negative case [broken_mask]: a SEPARATE tree whose protected subtree is deliberately NOT masked
# => protected becomes reachable => the model must go RED (nonzero). Good tree ($W) is untouched. ----
NEG_MODEL_RC=0
if [ "$model_available" = true ]; then
  mk_tree "$W2"
  negcap=$(bwrap --dev-bind / / \
        --tmpfs "$W2/instroot/attempts/attempt-B" \
        -- /bin/sh -c "
          cat \"$W2/instroot/attempts/attempt-A/own.txt\" >/dev/null 2>&1; echo \"POS_own_rc=\$?\";
          cat \"$W2/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"NEG_sibling_read_rc=\$?\";
          cat \"$W2/protected/custody/fabric-mac-key\" >/dev/null 2>&1; echo \"NEG_protected_read_rc=\$?\";
          \"$W2/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"NEG_protected_exec_rc=\$?\"
        ")
  neg_own=$(printf '%s\n' "$negcap" | sed -n 's/^POS_own_rc=\([0-9]*\)$/\1/p')
  neg_sib=$(printf '%s\n' "$negcap" | sed -n 's/^NEG_sibling_read_rc=\([0-9]*\)$/\1/p')
  neg_pr=$(printf '%s\n' "$negcap" | sed -n 's/^NEG_protected_read_rc=\([0-9]*\)$/\1/p')
  neg_pe=$(printf '%s\n' "$negcap" | sed -n 's/^NEG_protected_exec_rc=\([0-9]*\)$/\1/p')
  echo "BROKEN_MASK_OBS:"; printf '%s\n' "$negcap" | sed 's/^/  /'
  nown=false; [ "$neg_own" = "0" ] && nown=true
  nsib=true;  [ "$neg_sib" != "0" ] && nsib=false
  nprot=true; if [ "$neg_pr" != "0" ] && [ "$neg_pe" != "0" ]; then nprot=false; fi
  run_model_verdict "$nown" "$nprot" "$nsib" neg; NEG_MODEL_RC=$?
  echo "MODEL_VERDICT_NEG_RC=$NEG_MODEL_RC (expected nonzero: protected reachable when mask is broken)"
fi

unshare -Ur true >/dev/null 2>&1; userns_rc=$?
if [ "$userns_rc" -ne 0 ]; then
  echo "USERNS_PROBE=UNAVAILABLE (unshare -Ur rc=$userns_rc): uid/capability (DAC) and live-unit enforcement NOT exercised here (apparmor_restrict_unprivileged_userns)"
fi

# PASS requires: the MODEL held on the good run, the per-probe observations really denied protected+sibling
# under the mask (mask_ok, !sib_reach, !prot_reach, own permitted), the positive control really reached them
# unmasked, AND the NAMED negative (broken mask) made the model go RED.
if [ "$MODEL_RC" -eq 0 ] && [ "$own_perm" = true ] && [ "$sib_reach" = false ] && [ "$prot_reach" = false ] \
   && [ "$mask_ok" = true ] && [ "$posctl" = true ] \
   && [ "$model_available" = true ] && [ "$NEG_MODEL_RC" -ne 0 ]; then
  echo "ISOLATION_PROBE=PASS (model verdict on real observations; NAMED negative [broken_mask] -> MODEL_VERDICT_NEG_RC=$NEG_MODEL_RC)"
else
  echo "ISOLATION_PROBE=FAIL (MODEL_RC=$MODEL_RC own_perm=$own_perm sib_reach=$sib_reach prot_reach=$prot_reach mask_ok=$mask_ok posctl=$posctl model_avail=$model_available NEG_MODEL_RC=$NEG_MODEL_RC)"
  overall_rc=1
fi

rm -rf "$W" "$W2"
echo "OVERALL_RC=$overall_rc"
exit $overall_rc
