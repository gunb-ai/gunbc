#!/bin/bash
# ==== K2 (rev16 DRAFT, review material only): start the unit K1 installed, observe it (bounded), and on ANY failure stop THIS
#      start's invocation in the SAME run. Replaces J4 (retired). ====
# CAPABILITY / PROVENANCE (keen-otter msg_560eeac5):
#   escalation msg_e8888e14, created 2026-11-06T20:12:59Z. Question: "Who performs the srv1 lab restoration's root acts (install
#     serve unit, daemon-reload, start/stop)? keen-otter requires a named root holder." Options: "you run reviewed K1/K2 scripts
#     with sudo yourself" (recommended) | "you attest existing exact NOPASSWD argv rules on srv1".
#   operator's direct chat reply, ~20:15Z: "You have srv1 access to briansrls, I give you permission to run those"
#   keen-otter's resolution (msg_560eeac5): the permission covers loyal-deer-449, as briansrls, performing the described
#     reviewed K1/K2 acts. It does NOT include J3. It must not be re-asked. Sudo capability stays UNVERIFIED (no probe is
#     made; a `sudo -n` refusal at run time is a stop); the execution frame stays UNDECIDED; all review, chain and
#     exact-packet holds remain.
# EXECUTION FRAME (review material, not decided; calm-crab msg_e90e4723, keen-otter msg_e41d8062): run ONCE from a fresh copy
#   whose sha256 equals the reviewed one, as `sudo -n bash K2.sh <OUT> <k1-receipt> <sha> <j3-receipt> <sha>`; no keeper launch
#   running or starting meanwhile; START/END (with the receipt sha) posted to calm-crab; OUT/k2-receipt.txt and OUT/k2.log
#   retained and reconciled by calm-crab; no second attempt without a new decision.
# CLEANUP DOES NOT DEPEND ON ANY OTHER PARTY: the only stop K2 may perform is of THIS start's invocation (the InvocationID read
#   right after its own start), inside this run, before its deadline: STOP_RESERVE (360 s) is held back for it. If the
#   InvocationID changed (a Restart=on-failure restart or another actor), K2 does NOT stop: it reports UNRESOLVED with the
#   readings, and the decision is the operator's / calm-crab's (stated risk: a crash loop continues until then).
# J3 (enabling the three .path units) is a SEPARATE, separately decided act (J3-PLAN.txt, unchanged). K2 requires the J3
#   receipt, re-read live, exactly as J4 did; the K1/K2 permission does not authorize J3.
# Usage:  K2.sh <OUT (must not exist)> <K1 k1-receipt.txt> <its sha256> <J3 receipt> <its sha256>
# Exit: 0 = the serve is up and listening as this start's invocation (K2_UP); 2 = refused, failed-and-stopped-verified, or
#   UNRESOLVED, with the exact state in the receipt.
set -uo pipefail
die() { echo "K2 NO OBSERVATION (exit 2): $*"; exit 2; }
[ "$EUID" = 0 ] || die "not root (K2 is the root act; run by the named holder only)"
export PATH=/usr/sbin:/usr/bin:/sbin:/bin LC_ALL=C
for _en in $(compgen -e); do [[ $_en == GUNBC_* ]] && die "GUNBC_* variables are exported (names only checked)"; done
[[ ${EPOCHSECONDS:-} =~ ^[0-9]+$ ]] || die "bash >= 5 required"
K2_END=$(( EPOCHSECONDS + 2340 )); STOP_RESERVE=360; WORK_END=$(( K2_END - STOP_RESERVE ))   # ONE whole-act deadline
UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; WD=/home/briansrls/runs/ld449-r2work
OUT="${1:-}"; K1R="${2:-}"; K1S="${3:-}"; J3R="${4:-}"; J3S="${5:-}"
[[ $OUT = /* ]] && [[ $K1R = /* ]] && [[ $K1S =~ ^[0-9a-f]{64}$ ]] && [[ $J3R = /* ]] && [[ $J3S =~ ^[0-9a-f]{64}$ ]] || die "usage: K2.sh <OUT> <k1-receipt> <sha256> <j3-receipt> <sha256>"
{ [ -e "$OUT" ] || [ -L "$OUT" ]; } && die "OUT $OUT already exists"; mkdir -m 0755 -- "$OUT" || die "mkdir OUT"; R="$OUT"
exec > >(tee -a "$OUT/k2.log") 2>&1 || die "cannot log to OUT"
receipt() { printf '%s\n' "$1" > "$OUT/k2-receipt.txt" || echo "K2 WARNING: receipt write failed: $1"; echo "$1"; }
# reads: work path bounded by WORK_END, stop path by K2_END; each returns 124 when no time is left (never refused in a subshell)
wprop() { local r=$(( WORK_END - EPOCHSECONDS )); [ $r -ge 1 ] || return 124; timeout $(( r < 30 ? r : 30 )) systemctl show "$1" -p "$2" --value; }
sprop() { local r=$(( K2_END - EPOCHSECONDS )); [ $r -ge 1 ] || return 124; timeout $(( r < 30 ? r : 30 )) systemctl show "$1" -p "$2" --value; }
wcap() { local r=$(( WORK_END - EPOCHSECONDS )); [ $r -ge 1 ] || { echo 0; return; }; echo $(( r < $1 ? r : $1 )); }
hashf() { local o; o="$(timeout 60 sha256sum -- "$1")" || return 1; o="${o%% *}"; [[ $o =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$o"; }
# ---- prechecks (nothing started) ----
[ -f "$K1R" ] && [ ! -L "$K1R" ] || die "the K1 receipt is not a regular file"; h="$(hashf "$K1R")" || die "cannot hash the K1 receipt"; [ "$h" = "$K1S" ] || die "the K1 receipt is not the cited bytes"
mapfile -t KL < "$K1R" || die "cannot read the K1 receipt"; [ ${#KL[@]} = 1 ] || die "the K1 receipt is not exactly 1 line"
[[ ${KL[0]} =~ ^K1_OK\ unit="$UNIT"\ dest="$DEST"\ installed_sha=([0-9a-f]{64})\ request=([0-9a-f]{64})\ pre_inventory=[0-9a-f]{64}\ post_inventory=[0-9a-f]{64}\ LoadState=loaded\ ActiveState=inactive\ UnitFileState=disabled$ ]] || die "the K1 receipt is not a K1_OK line for $UNIT"   # $UNIT/$DEST are quoted in the regex: literal, not patterns
ISHA="${BASH_REMATCH[1]}"; RSHA="${BASH_REMATCH[2]}"
{ [ -e $WD/J2_PARTIAL ] || [ -L $WD/J2_PARTIAL ]; } && die "$WD/J2_PARTIAL exists"
M=$WD/markers/J2_READY; [ -f $M ] && [ ! -L $M ] || die "J2_READY marker missing"; mapfile -t ML < $M || die "cannot read J2_READY"
_b=0; for _l in "${ML[@]}"; do [[ $_l =~ ^RECEIPT\ $RSHA\ /[^[:space:]]+/k1-request\.txt$ ]] && _b=$((_b+1)); done; [ $_b = 1 ] || die "J2_READY does not bind the request K1 executed ($RSHA)"
# J3 receipt (unchanged law from J4): exactly 3 lines, each equal to the live reading of its .path unit
[ -f "$J3R" ] && [ ! -L "$J3R" ] || die "the J3 receipt is not a regular file"; h="$(hashf "$J3R")" || die "cannot hash the J3 receipt"; [ "$h" = "$J3S" ] || die "J3 receipt is not the cited bytes"
mapfile -t J3L < "$J3R" || die "cannot read the J3 receipt"; [ "${#J3L[@]}" = 3 ] || die "J3 receipt is not exactly 3 lines"
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit gunbc-srv1-lab-codex-feedback-v0-belt-discovery gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain; do
  iv="$(wprop $u.path InvocationID)" || die "read $u.path InvocationID"; ts="$(wprop $u.path ActiveEnterTimestampMonotonic)" || die "read $u.path timestamp"
  a="$(wprop $u.path ActiveState)" || die "read $u.path ActiveState"; e="$(wprop $u.path UnitFileState)" || die "read $u.path UnitFileState"
  [ -n "$iv" ] && [ -n "$ts" ] && [ "$a" = active ] && [ "$e" = enabled ] || die "$u.path is not active/enabled with a live invocation (J3 not done)"
  want="J3_OK $u.path ActiveState=active UnitFileState=enabled Triggers=$u.service InvocationID=$iv ActiveEnterTimestampMonotonic=$ts"
  _n=0; for _l in "${J3L[@]}"; do [ "$_l" = "$want" ] && _n=$((_n+1)); done; [ $_n = 1 ] || die "J3 receipt line for $u.path does not equal the live reading"; done
# the installed unit, live: the bytes K1 installed, loaded, inactive, disabled, no drop-ins, no pending reload
h="$(hashf $DEST)" || die "cannot hash $DEST"; [ "$h" = "$ISHA" ] || die "$DEST is not the bytes K1 installed"
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled FragmentPath:$DEST NeedDaemonReload:no; do v="$(wprop $UNIT ${p%%:*})" || die "read ${p%%:*}"; [ "$v" = "${p#*:}" ] || die "${p%%:*} is [$v], need ${p#*:}"; done
dp="$(wprop $UNIT DropInPaths)" || die "read DropInPaths"; [ -z "$dp" ] || die "unit has drop-ins [$dp] not reviewed"
listen() { local r=$(( K2_END - EPOCHSECONDS )); [ $r -ge 1 ] || return 124; timeout $(( r < 30 ? r : 30 )) ss -Hltnp 'sport = :18081'; }
l="$(listen)" || die "ss read failed before start"; [ -z "$l" ] || die ":18081 already has a listener before start"
cg_procs() { local d=${CGROOT:-/sys/fs/cgroup}$1 to=${2:-30}; [ -d "$d" ] || return 2; [ "$to" -ge 1 ] || return 1
  timeout --kill-after=5 "$to" bash -c 'set -o pipefail; find "$1" -name cgroup.procs -print0 | xargs -0r cat --' _ "$d" > $R/.cgp 2> $R/.cgp.err || return 1
  [ -s $R/.cgp.err ] && return 1; local -a pp; local x; mapfile -t pp < $R/.cgp || return 1
  for x in "${pp[@]}"; do [[ $x =~ ^[0-9]+$ ]] || return 1; done; if [ ${#pp[@]} -gt 0 ]; then printf '%s\n' "${pp[@]}"; fi; return 0; }
# stop_verified: invocation-scoped, in THIS run, inside the stop reserve. The unit's cgroup directory must EXIST before the
# stop (independent review rev15 #8: a wrong path must not read as "gone"); after it, the subtree must be empty or removed.
stop_verified() { local a cg l p
  cg="$(sprop $UNIT ControlGroup)" || return 1; [ -n "$cg" ] && [ -d "${CGROOT:-/sys/fs/cgroup}$cg" ] || { echo "the unit cgroup [$cg] is not present before the stop"; return 1; }
  local r=$(( K2_END - EPOCHSECONDS )); [ $r -ge 1 ] || return 1; timeout $(( r < 120 ? r : 120 )) systemctl stop $UNIT || { echo "stop rc failed"; return 1; }
  a="$(sprop $UNIT ActiveState)" || return 1; case "$a" in inactive|failed) ;; *) echo "after stop ActiveState=$a"; return 1;; esac
  p="$(cg_procs "$cg")"; case $? in 0) [ -z "$p" ] || { echo "cgroup subtree still has: $p"; return 1; };; 2) ;; *) return 1;; esac
  l="$(listen)" || return 1; [ -z "$l" ] || { echo "listener remains"; return 1; }; return 0; }
unresolved() { local s; s="$(timeout 30 systemctl show $UNIT -p ActiveState -p SubState -p Result -p InvocationID -p NRestarts -p ExecMainStatus 2>&1 | tr '\n' ' ')"
  receipt "K2_UNRESOLVED invocation=${INV:-none} reason=$1 state=[$s]"; die "$1 (UNRESOLVED; the operator / calm-crab decides)"; }
fail_ours() { local cur; cur="$(sprop $UNIT InvocationID)" || unresolved "$1; InvocationID read failed"
  [ "$cur" = "$INV" ] || unresolved "$1; InvocationID is now [$cur], not this start's [$INV]; not stopped"
  if stop_verified; then receipt "K2_STOPPED_VERIFIED_DOWN invocation=$INV reason=$1"; die "$1; this start's invocation stopped and verified down"; else unresolved "$1; stop failed or could not be verified"; fi; }
# ---- start (the one mutation that starts a process) ----
NR0="$(wprop $UNIT NRestarts)" || die "read NRestarts"
c=$(wcap 300); [ "$c" -ge 1 ] || die "no time budget left to start"; timeout $c systemctl start $UNIT; r=$?
INV="$(sprop $UNIT InvocationID)" || unresolved "start rc=$r; InvocationID read failed"
echo "START_RC=$r INVOCATION=[$INV] NRESTARTS0=$NR0"
if [ -z "$INV" ]; then b="$(sprop $UNIT ActiveState)" || unresolved "start rc=$r; empty InvocationID; ActiveState read failed"
  case "$b" in inactive|failed) receipt "K2_NOT_STARTED start_rc=$r unit=$b"; die "start rc=$r produced no invocation; unit [$b]; nothing to stop";; *) unresolved "start rc=$r; empty InvocationID while unit is [$b]";; esac; fi
t=$(wcap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE before the post-start property read"
timeout $t systemctl show $UNIT -p LoadState -p ActiveState -p SubState -p Result -p ExecMainPID -p ExecMainStatus -p NRestarts -p ControlGroup -p InvocationID -p User -p CapabilityBoundingSet -p NoNewPrivileges -p RestrictSUIDSGID > $R/serve-props.txt || fail_ours "post-start property read failed or timed out"; cat $R/serve-props.txt
[ $r = 0 ] || fail_ours "start rc=$r"
LP=""
while :; do
  [ "$(wcap 60)" -ge 1 ] || fail_ours "DEADLINE: work budget exhausted before a listener appeared"
  c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out while waiting"; [ "$c" = "$INV" ] || fail_ours "INVOCATION_CHANGED"
  b="$(wprop $UNIT ActiveState)" || fail_ours "ActiveState read failed or timed out while waiting"; case "$b" in active|activating) ;; *) fail_ours "STARTUP_ERROR ActiveState=$b";; esac
  t=$(wcap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE before ss"; o="$(timeout $t ss -Hltnp 'sport = :18081')" || fail_ours "ss read failed or timed out while waiting"
  LP=""; _lpo=0; while IFS= read -r _ll; do [ -n "$_ll" ] || continue; _lpo=1; _rest="$_ll"; _found=0
    while [[ $_rest =~ pid=([0-9]+) ]]; do _found=1; case " $LP " in *" ${BASH_REMATCH[1]} "*) ;; *) LP="$LP${BASH_REMATCH[1]} ";; esac; _rest="${_rest#*pid=${BASH_REMATCH[1]}}"; done
    [ $_found = 1 ] || fail_ours "a :18081 listener line carries no pid (not attributable)"; done <<< "$o"
  if [ $_lpo = 1 ]; then break; fi; t=$(wcap 30); [ "$t" -ge 1 ] || fail_ours "DEADLINE"; sleep $t
done
CG2="$(wprop $UNIT ControlGroup)" || fail_ours "ControlGroup read failed or timed out"
c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out"
[ -n "$CG2" ] && [ -d "${CGROOT:-/sys/fs/cgroup}$CG2" ] || fail_ours "unit cgroup missing"
t=$(wcap 30); procs="$(cg_procs "$CG2" "$t")" || fail_ours "cannot read the unit cgroup subtree (failed or out of budget)"
[ -n "$LP" ] && [ "$c" = "$INV" ] || fail_ours "NOT_LISTENING_OR_NOT_IDENTIFIED pids=[${LP:-none}]"
for p in $LP; do [[ $'\n'$procs$'\n' == *$'\n'$p$'\n'* ]] || fail_ours "listener pid $p is not in $UNIT's cgroup subtree"; done
receipt "K2_UP unit=$UNIT invocation=$INV listener_pids=${LP% } cgroup=$CG2 installed_sha=$ISHA k1_receipt=$K1S j3_receipt=$J3S"
exit 0
