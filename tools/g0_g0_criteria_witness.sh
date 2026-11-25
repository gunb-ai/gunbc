#!/usr/bin/env bash
# HAND-SHELL ENFORCEMENT PROBE for the G0 contained-development-execution criteria proposal (PR #16989).
#
# SINGLE VERDICT AUTHORITY (review 98327 FINDING 1 / gunbc.design_argument::single_authority): this file
# does NOT decide PASS. It runs bwrap, records each probe's OWN return code as a text token, and feeds them
# to the model verdict, which returns ProcessExit. The shell then exits with THAT verdict's status and
# NOTHING else (no shell-side && of mask_ok/posctl/etc). The mask-applied and positive-control observations
# are INPUTS to the verdict, consumed there.
#   model decision rule:   gunbc.roadmap.g0_contained_dev_execution_proof::g0_two_sided_isolation_holds
#   model sole verdict:    ...::g0_two_sided_isolation_verdict (-> ProcessExit; reason names the clause)
#   bash-emit capability + dissolution trigger:
#                          ...::g0_bwrap_isolation_bash_emit_dissolution_trigger (std.disposition.Scaffold)
#   WHY it cannot be a .dag consumer: producing the observations requires EXECUTING a real kernel
#   mount-namespace primitive (bwrap --tmpfs masking) that no typecheck or grep can (fail_closed).
#
# Run on srv1 from the repo root (Cargo.toml beside dag/ and src/), under the shared limiter:
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

# invoke the model verdict with the given key=value tokens; echoes its own stdout (the located reason) and
# returns its PROCESS status. The shell's authority derives ONLY from a real, present binary+model.
run_verdict() {
  local tag="$1"; shift
  local argv=(); local kv
  for kv in "$@"; do argv+=(--arg "$kv"); done
  ( cd "$G0_REPO" && "$GUNBC_BIN" run --entry "$G0_MODEL" $G0_SR \
      --function g0_two_sided_isolation_verdict "${argv[@]}" ) > "$SCR/verdict_${tag}.log" 2>&1
  return $?
}

# a captured return code is relayed verbatim; an absent probe line becomes the literal token MISSING so the
# MODEL (not the shell) turns it into a located fail-closed failure (review 98327 FINDING 2).
cap_tok() { local v; v=$(printf '%s\n' "$1" | sed -n "s/^$2=\([0-9][0-9]*\)\$/\1/p" | head -1); [ -n "$v" ] && printf '%s' "$v" || printf 'MISSING'; }

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

# ENFORCED (good) run: mask sibling AND protected. Each probe records its OWN rc.
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
# UNMASKED positive control: the SAME probes must REACH (rc 0) -- proves they really read/exec.
capu=$(bwrap --dev-bind / / -- /bin/sh -c "
        cat \"$W/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"POSCTL_sibling_read_rc=\$?\";
        \"$W/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"POSCTL_protected_exec_rc=\$?\"
      ")
bwrap_unm_rc=$?
# NAMED negative [broken_mask]: separate tree, protected left UNMASKED (sibling still masked).
mk_tree "$W2"
negcap=$(bwrap --dev-bind / / \
      --tmpfs "$W2/instroot/attempts/attempt-B" \
      -- /bin/sh -c "
        cat \"$W2/instroot/attempts/attempt-A/own.txt\" >/dev/null 2>&1; echo \"POS_own_rc=\$?\";
        cat \"$W2/instroot/attempts/attempt-B/sibling.txt\" >/dev/null 2>&1; echo \"NEG_sibling_read_rc=\$?\";
        cat \"$W2/protected/custody/fabric-mac-key\" >/dev/null 2>&1; echo \"NEG_protected_read_rc=\$?\";
        \"$W2/protected/custody/payload.sh\" >/dev/null 2>&1; echo \"NEG_protected_exec_rc=\$?\"
      ")
bwrap_neg_rc=$?

# relay the 14 tokens (verbatim rc, or MISSING)
T_own=$(cap_tok "$cap"    POS_own_rc)
T_mask=$(cap_tok "$cap"   MASKED_ENTRIES)
T_sib=$(cap_tok "$cap"    NEG_sibling_read_rc)
T_pread=$(cap_tok "$cap"  NEG_protected_read_rc)
T_pexec=$(cap_tok "$cap"  NEG_protected_exec_rc)
T_enflaunch="$bwrap_enf_rc"
T_psib=$(cap_tok "$capu"  POSCTL_sibling_read_rc)
T_pexec2=$(cap_tok "$capu" POSCTL_protected_exec_rc)
T_unmlaunch="$bwrap_unm_rc"
T_nown=$(cap_tok "$negcap" POS_own_rc)
T_nsib=$(cap_tok "$negcap" NEG_sibling_read_rc)
T_npread=$(cap_tok "$negcap" NEG_protected_read_rc)
T_npexec=$(cap_tok "$negcap" NEG_protected_exec_rc)
T_neglaunch="$bwrap_neg_rc"

echo "RECORDED_TOKENS: own=$T_own mask=$T_mask sib=$T_sib protected_read=$T_pread protected_exec=$T_pexec enf_launch=$T_enflaunch posctl_sib=$T_psib posctl_exec=$T_pexec2 unm_launch=$T_unmlaunch neg_own=$T_nown neg_sib=$T_nsib neg_protected_read=$T_npread neg_protected_exec=$T_npexec neg_launch=$T_neglaunch"

ARGS_GOOD() {
  printf '%s\n' \
    "own_rc=$T_own" "masked_entries=$T_mask" "sibling_rc=$T_sib" \
    "protected_read_rc=$T_pread" "protected_exec_rc=$T_pexec" "enf_launch_rc=$T_enflaunch" \
    "posctl_sibling_rc=$T_psib" "posctl_exec_rc=$T_pexec2" "unm_launch_rc=$T_unmlaunch" \
    "neg_own_rc=$T_nown" "neg_sibling_rc=$T_nsib" "neg_protected_read_rc=$T_npread" \
    "neg_protected_exec_rc=$T_npexec" "neg_launch_rc=$T_neglaunch"
}

# ---- THE SOLE VERDICT (the model decides; the shell only relays tokens and adopts its status) ----
echo "--- MODEL VERDICT (real observations) ---"
mapfile -t GOOD < <(ARGS_GOOD)
run_verdict good "${GOOD[@]}"; MODEL_RC=$?
sed 's/^/MODEL_OUT: /' "$SCR/verdict_good.log"
echo "MODEL_VERDICT_RC=$MODEL_RC"

# ---- NAMED NEGATIVE [broken_mask] (evidence the check is not vacuous): invert ONE good-path fixture --
#      make the protected EXEC reachable (mask broken) -- the model must go RED naming protected reachable. ----
echo "--- NAMED NEGATIVE [broken_mask]: invert protected_exec fixture (mask broken) -> expect RED ---"
mapfile -t NEGA < <(ARGS_GOOD | sed "s#^protected_exec_rc=.*#protected_exec_rc=0#")
run_verdict neg_broken_mask "${NEGA[@]}"; NEG_A_RC=$?
sed 's/^/NEG_A_MODEL_OUT: /' "$SCR/verdict_neg_broken_mask.log"
echo "NEG_A_MODEL_RC=$NEG_A_RC (expected nonzero naming ISOLATION_BREACHED:protected_exec_reachable_under_mask)"

# ---- NAMED NEGATIVE [missing_observation] (review 98327 FINDING 2): a probe that produced no output must
#      be a typed, located failure -- never a false mistaken for a denial. Feed sibling_rc=MISSING. ----
echo "--- NAMED NEGATIVE [missing_observation]: feed sibling_rc=MISSING -> expect typed failure ---"
mapfile -t NEGB < <(ARGS_GOOD | sed "s#^sibling_rc=.*#sibling_rc=MISSING#")
run_verdict neg_missing "${NEGB[@]}"; NEG_B_RC=$?
sed 's/^/NEG_B_MODEL_OUT: /' "$SCR/verdict_neg_missing.log"
echo "NEG_B_MODEL_RC=$NEG_B_RC (expected nonzero naming MISSING_OR_UNPARSEABLE_OBSERVATION:sibling_rc)"

unshare -Ur true >/dev/null 2>&1; userns_rc=$?
if [ "$userns_rc" -ne 0 ]; then
  echo "USERNS_PROBE=UNAVAILABLE (unshare -Ur rc=$userns_rc): uid/capability (DAC) and live-unit enforcement NOT exercised here (apparmor_restrict_unprivileged_userns)"
fi

# The named-negative demonstrations are evidence; their status is printed, NOT joined into the decision.
# The model verdict itself already refuses to return ExitSuccess unless the negative run went RED for the
# right reason (codes 8/9/10). The SHELL EXITS WITH THE MODEL VERDICT STATUS AND NOTHING ELSE.
rm -rf "$W" "$W2"
echo "SHELL_EXITS_WITH_MODEL_RC=$MODEL_RC"
exit "$MODEL_RC"
