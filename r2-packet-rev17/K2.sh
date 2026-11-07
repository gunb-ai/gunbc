#!/bin/bash
# ==== K2 (rev17 DRAFT, review material only): start the unit K1 installed, observe it (bounded), and on ANY failure stop an
#      invocation of K2's OWN start in the SAME run. Replaces J4 (retired). ====
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
#   whose sha256 equals the reviewed one, as `sudo -n bash K2.sh <k1-receipt> <its sha> <j3-receipt> <its sha> <OUT>`; NO
#   OTHER ACTOR may start, stop or restart this unit while K2 runs (no keeper launch running or starting; START/END posted to
#   calm-crab); PRIV/k2-receipt.txt persisted, read back and EXPORTED before any success text, and printed; stdout is the log;
#   calm-crab reconciles;
#   no second attempt without a new decision.
# LIFECYCLE AND STOP CONTRACT (keen-otter msg_029acc92 #5). K2's stop authority covers THIS unit, during K2's own run, for
#   invocations that ORIGINATE from K2's start: the InvocationID read right after K2's `systemctl start`, and any invocation
#   systemd creates by an AUTOMATIC Restart=on-failure restart of it. Origin is decided with NRestarts: an InvocationID change
#   accompanied by an NRestarts increase is an automatic restart of K2's start (stop authorized); a change WITHOUT an NRestarts
#   increase is ANOTHER actor (NO stop: UNRESOLVED). InvocationID and NRestarts are re-read immediately BEFORE the stop and
#   again AFTER it, and both readings are recorded in the receipt. RESIDUAL WINDOW: between that last pre-stop read and the stop
#   itself (one `systemctl stop` call), another actor could restart the unit; the stop would then hit its invocation. systemd
#   offers no stop conditional on an InvocationID, so K2 cannot close that window and does NOT widen its stop authority to
#   cover it. What K2 does (keen-otter msg_c144f271): it re-reads InvocationID and NRestarts AFTER the stop; if either differs
#   from the pre-stop reading, or cannot be read, or the InvocationID reads empty, the result is UNRESOLVED ("the stop's target
#   cannot be attributed to K2's start"), never STOPPED_VERIFIED_DOWN. This detects a restart inside the window only after the
#   fact and only as far as systemd retains the last InvocationID of an inactive unit (assumed, UNVERIFIED on srv1's systemd:
#   if it is cleared on stop, every stop reads UNRESOLVED, which is fail-closed). The execution frame's exclusion rule (no
#   other actor starts/stops/restarts this unit during K2) is what bounds the window; the frame is UNDECIDED, so this residual
#   stands until it is decided.
# TIME (keen-otter msg_029acc92 #2): K2_END = start + 2340 s. Every external command runs as `timeout -k 5 <cap>`, its cap
#   computed in the main shell from the REMAINING budget. Reserves: STOP_RESERVE 360 s (stop 120 + verification reads + the
#   cgroup read, all from what remains of it), RECEIPT_RESERVE 30 s (persisted receipt + report), the 5 s kill grace on every
#   step (the receipt reserve is 60 s: persist, sync, read <= 10 each, export <= 15, export readback <= 15). BEFORE
#   `systemctl start` K2 requires the FULL allowance: start 120 + 5 + the reviewed observation allowance
#   OBS_ALLOW (900 s) + STOP_RESERVE + RECEIPT_RESERVE; otherwise it refuses before starting. The work phase (start and
#   observation) ends at WORK_END = K2_END - STOP_RESERVE - RECEIPT_RESERVE.
# INPUTS (keen-otter msg_029acc92 #3): the K1 receipt, the J3 receipt and J2_READY are SNAPSHOTTED into the root-owned 0700
#   PRIV=/run/r2-k2.XXXXXX first; only the snapshots are hashed and parsed. The expected K1 receipt sha is argument 3.
#   PRIV is RETAINED on every path as evidence (its path is in every receipt); the operator removes it after calm-crab's
#   reconcile (`rm -rf` as root; /run is tmpfs).
# OUTPUT DESIGN (sleek-owl msg_cfe1a18a M1, refined by keen-otter msg_c144f271; BLOCKING for privileged execution): root
#   opens, creates and writes nothing in a user-writable namespace at any time. Root writes only inside PRIV, created by mktemp
#   under the root-owned /run and VERIFIED by stat (0:0, mode 700, a directory; a symlink refuses): the receipt, the cgroup
#   reads and the serve properties. The export to OUT runs with privilege DROPPED: `runuser -u briansrls -- tee -- OUT/<name>
#   < PRIV/<name>` (root opens only the PRIV file, as stdin; every component of OUT is resolved as briansrls), read back as
#   briansrls. OUT is checked usable BEFORE start (k2-begin.txt exported and read back; failure refuses before start).
#   EVIDENCE: PRIV retained on every path; exported copies OUT/k2-begin.txt and OUT/k2-receipt.txt. An export failure after
#   the start is K2_UNRESOLVED step=X, exit 2, PRIV retained and its path printed; the serve's state is as the PRIV receipt
#   records (an UP serve is NOT stopped for an export failure: that would be a stop outside the lifecycle contract's causes).
# CGROUP ROOT: /sys/fs/cgroup, a fixed constant (no environment seam).
# J3 (the three .path units) is a SEPARATE act with execution authorization NONE (J3.md). K2 requires the J3 receipt, re-read
#   live; the K1/K2 permission does not authorize J3.
# Usage:  K2.sh <K1 k1-receipt.txt> <its expected sha256> <J3 receipt> <its sha256> <OUT dir, written only as briansrls>
# Exit: 0 = up and listening as an invocation of K2's own start, K2_UP persisted and read back; 2 = everything else
#   (refused before start, stopped and verified down, or UNRESOLVED), with the state in the receipt.
set -uo pipefail
GRACE=5; STOP_RESERVE=360; RECEIPT_RESERVE=60; START_CAP=120; OBS_ALLOW=900
EXPORT_USER=briansrls; PRIV_OWNER=0:0
UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; WD=/home/briansrls/runs/ld449-r2work
CGR=/sys/fs/cgroup; PRIV="(none yet)"; PRIVOK=no; INV=""; NRL=""; AUTO=0; STARTED=no
receipt() { local r c rb; [ $PRIVOK = yes ] || return 1
  r=$(( K2_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 10 ? r : 10 ))
  printf '%s\n' "$1" > "$PRIV/k2-receipt.txt" || return 1; timeout -k $GRACE $c sync -f "$PRIV/k2-receipt.txt" || return 1
  rb="$(timeout -k $GRACE $c cat -- "$PRIV/k2-receipt.txt")" || return 1; [ "$rb" = "$1" ]; }
# xport NAME TEXT: export PRIV/NAME to OUT/NAME with privilege DROPPED (OUTPUT DESIGN); 1 on failure or a different readback
xport() { local r c rb; r=$(( K2_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 15 ? r : 15 ))
  timeout -k $GRACE $c runuser -u $EXPORT_USER -- tee -- "$OUT/$1" < "$PRIV/$1" > /dev/null || return 1
  r=$(( K2_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 15 ? r : 15 ))
  rb="$(timeout -k $GRACE $c runuser -u $EXPORT_USER -- cat -- "$OUT/$1")" || return 1; [ "$rb" = "$2" ]; }
end() { local line="K2_$1 started=$STARTED invocation=${INV:-none} auto_restarts=$AUTO priv=$PRIV reason=$2"
  if receipt "$line"; then echo "$line"
    if ! xport k2-receipt.txt "$line"; then if [ $STARTED = yes ]; then echo "K2_UNRESOLVED step=X after the start the receipt's export to OUT ($OUT) failed or read back different; evidence retained in PRIV=$PRIV"
      else echo "K2 export of the refusal receipt to OUT failed; it is retained in PRIV=$PRIV"; fi; fi
  else echo "K2_UNRESOLVED step=receipt could not persist [$line]"; fi; echo "K2 NO OBSERVATION (exit 2): $2"; exit 2; }
die() { [ $STARTED = yes ] && end UNRESOLVED "$1"; end REFUSED "$1"; }
T() { timeout -k $GRACE "$CV" "$@"; }
# capw N: work-phase cap = min(N, WORK_END - now - GRACE); returns 1 (no time) instead of a 0 cap
capw() { local r=$(( WORK_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; CV=$(( r < $1 ? r : $1 )); }
# caps N: stop-phase cap = min(N, what remains of the stop reserve before the receipt reserve)
caps() { local r=$(( K2_END - EPOCHSECONDS - GRACE - RECEIPT_RESERVE )); [ $r -ge 1 ] || return 1; CV=$(( r < $1 ? r : $1 )); }
wprop() { capw 30 || return 124; T systemctl show "$1" -p "$2" --value; }
sprop() { caps 30 || return 124; T systemctl show "$1" -p "$2" --value; }
# ---- prechecks (nothing started) ----
[ "$EUID" = 0 ] || { echo "K2 NO OBSERVATION (exit 2): not root"; exit 2; }
export PATH=/usr/sbin:/usr/bin:/sbin:/bin LC_ALL=C
for _en in $(compgen -e); do [[ $_en == GUNBC_* ]] && { echo "K2 NO OBSERVATION (exit 2): GUNBC_* variables are exported (names only checked)"; exit 2; }; done
[[ ${EPOCHSECONDS:-} =~ ^[0-9]+$ ]] || { echo "K2 NO OBSERVATION (exit 2): bash >= 5 required"; exit 2; }
K2_END=$(( EPOCHSECONDS + 2340 )); WORK_END=$(( K2_END - STOP_RESERVE - RECEIPT_RESERVE ))
K1R="${1:-}"; K1S="${2:-}"; J3R="${3:-}"; J3S="${4:-}"; OUT="${5:-}"
[[ $K1R = /* ]] && [[ $K1S =~ ^[0-9a-f]{64}$ ]] && [[ $J3R = /* ]] && [[ $J3S =~ ^[0-9a-f]{64}$ ]] && [[ $OUT = /* ]] && [ $# = 5 ] || { echo "K2 NO OBSERVATION (exit 2): usage: K2.sh <k1-receipt> <sha256> <j3-receipt> <sha256> <OUT dir>"; exit 2; }
capw 10 || { echo "K2 NO OBSERVATION (exit 2): no time"; exit 2; }; PRIV="$(T mktemp -d /run/r2-k2.XXXXXX)" || die "mktemp of the private dir failed"; capw 10 || die "no time"; T chmod 0700 "$PRIV" || die "chmod private dir"
capw 10 || die "no time"; _ps="$(T stat -c '%u:%g:%a:%F' -- "$PRIV")" || die "stat of the private dir failed"
[ "$_ps" = "$PRIV_OWNER:700:directory" ] && [ ! -L "$PRIV" ] || die "the private dir [$PRIV] is [$_ps], not owner $PRIV_OWNER mode 700 directory (a symlink refuses)"
PRIVOK=yes; echo "K2 private dir (retained, verified $_ps): $PRIV"
snap() { capw 30 || die "no time to snapshot $3"; [ -e "$1" ] || die "$3 [$1] is absent"; T cp -P -- "$1" "$PRIV/$2" || die "cannot snapshot $3"; [ -f "$PRIV/$2" ] && [ ! -L "$PRIV/$2" ] || die "$3 is not a regular file"; }
hashf() { local o; capw 60 || return 1; o="$(T sha256sum -- "$1")" || return 1; o="${o%% *}"; [[ $o =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$o"; }
snap "$K1R" k1-receipt "the K1 receipt"; h="$(hashf "$PRIV/k1-receipt")" || die "cannot hash the K1 receipt snapshot"; [ "$h" = "$K1S" ] || die "the K1 receipt is not the expected bytes (argument 3)"
mapfile -t KL < "$PRIV/k1-receipt" || die "cannot read the K1 receipt snapshot"; [ ${#KL[@]} = 1 ] || die "the K1 receipt is not exactly 1 line"
[[ ${KL[0]} =~ ^K1_OK\ unit="$UNIT"\ dest="$DEST"\ installed_sha=([0-9a-f]{64})\ request=([0-9a-f]{64})\ request_path=(/[^[:space:]]+)\ j2_inventory=[0-9a-f]{64}\ pre_inventory=[0-9a-f]{64}\ post_inventory=[0-9a-f]{64}\ LoadState=loaded\ ActiveState=inactive\ UnitFileState=disabled\ priv=/[^[:space:]]+$ ]] || die "the K1 receipt is not a K1_OK line for $UNIT"   # $UNIT/$DEST quoted: literal
ISHA="${BASH_REMATCH[1]}"; RSHA="${BASH_REMATCH[2]}"; RPATH="${BASH_REMATCH[3]}"
{ [ -e $WD/J2_PARTIAL ] || [ -L $WD/J2_PARTIAL ]; } && die "$WD/J2_PARTIAL exists"
snap $WD/markers/J2_READY J2_READY "the J2_READY marker"; mapfile -t ML < "$PRIV/J2_READY" || die "cannot read the J2_READY snapshot"
_b=0; for _l in "${ML[@]}"; do [ "$_l" = "RECEIPT $RSHA $RPATH" ] && _b=$((_b+1)); done; [ $_b = 1 ] || die "J2_READY does not bind the EXACT request K1 executed ($RSHA $RPATH)"
snap "$J3R" j3-receipt "the J3 receipt"; h="$(hashf "$PRIV/j3-receipt")" || die "cannot hash the J3 receipt snapshot"; [ "$h" = "$J3S" ] || die "J3 receipt is not the cited bytes"
mapfile -t J3L < "$PRIV/j3-receipt" || die "cannot read the J3 receipt snapshot"; [ "${#J3L[@]}" = 3 ] || die "J3 receipt is not exactly 3 lines"
for u in gunbc-srv1-lab-codex-feedback-v0-belt-exit gunbc-srv1-lab-codex-feedback-v0-belt-discovery gunbc-srv1-lab-codex-feedback-v0-attempt-unit-drain; do
  iv="$(wprop $u.path InvocationID)" || die "read $u.path InvocationID"; ts="$(wprop $u.path ActiveEnterTimestampMonotonic)" || die "read $u.path timestamp"
  a="$(wprop $u.path ActiveState)" || die "read $u.path ActiveState"; e="$(wprop $u.path UnitFileState)" || die "read $u.path UnitFileState"
  [ -n "$iv" ] && [ -n "$ts" ] && [ "$a" = active ] && [ "$e" = enabled ] || die "$u.path is not active/enabled with a live invocation (J3 not done)"
  want="J3_OK $u.path ActiveState=active UnitFileState=enabled Triggers=$u.service InvocationID=$iv ActiveEnterTimestampMonotonic=$ts"
  _n=0; for _l in "${J3L[@]}"; do [ "$_l" = "$want" ] && _n=$((_n+1)); done; [ $_n = 1 ] || die "J3 receipt line for $u.path does not equal the live reading"; done
h="$(hashf $DEST)" || die "cannot hash $DEST"; [ "$h" = "$ISHA" ] || die "$DEST is not the bytes K1 installed"
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled FragmentPath:$DEST NeedDaemonReload:no; do v="$(wprop $UNIT ${p%%:*})" || die "read ${p%%:*}"; [ "$v" = "${p#*:}" ] || die "${p%%:*} is [$v], need ${p#*:}"; done
dp="$(wprop $UNIT DropInPaths)" || die "read DropInPaths"; [ -z "$dp" ] || die "unit has drop-ins [$dp] not reviewed"
listen() { caps 30 || return 124; T ss -Hltnp 'sport = :18081'; }
l="$(listen)" || die "ss read failed before start"; [ -z "$l" ] || die ":18081 already has a listener before start"
# cg_procs CG CAP: the whole subtree read as ONE subprocess under ONE cap (rc 2 = the directory is absent)
cg_procs() { local d=$CGR$1 to=$2; [ -d "$d" ] || return 2; [ "$to" -ge 1 ] || return 1
  timeout -k $GRACE "$to" bash -c 'set -o pipefail; find "$1" -name cgroup.procs -print0 | xargs -0r cat --' _ "$d" > $PRIV/.cgp 2> $PRIV/.cgp.err || return 1
  [ -s $PRIV/.cgp.err ] && return 1; local -a pp; local x; mapfile -t pp < $PRIV/.cgp || return 1
  for x in "${pp[@]}"; do [[ $x =~ ^[0-9]+$ ]] || return 1; done; if [ ${#pp[@]} -gt 0 ]; then printf '%s\n' "${pp[@]}"; fi; return 0; }
# origin CUR NR: 0 = an invocation of K2's own start (unchanged, or an automatic restart: NRestarts increased; INV/NRL/AUTO
# advance), 1 = another actor (changed without an NRestarts increase)
origin() { [ "$1" = "$INV" ] && return 0; [[ $2 =~ ^[0-9]+$ ]] && [[ $NRL =~ ^[0-9]+$ ]] && [ "$2" -gt "$NRL" ] && { echo "automatic restart: invocation $INV -> $1, NRestarts $NRL -> $2"; INV="$1"; NRL="$2"; AUTO=$((AUTO+1)); return 0; }; return 1; }
stop_verified() { local a cg l p r
  cg="$(sprop $UNIT ControlGroup)" || return 1; [ -n "$cg" ] && [ -d "$CGR$cg" ] || { echo "the unit cgroup [$cg] is not present before the stop"; return 1; }
  caps 120 || return 1; T systemctl stop $UNIT || { echo "stop rc failed"; return 1; }
  a="$(sprop $UNIT ActiveState)" || return 1; case "$a" in inactive|failed) ;; *) echo "after stop ActiveState=$a"; return 1;; esac
  r=$(( K2_END - EPOCHSECONDS - GRACE - RECEIPT_RESERVE )); p="$(cg_procs "$cg" $r)"; case $? in 0) [ -z "$p" ] || { echo "cgroup subtree still has: $p"; return 1; };; 2) ;; *) echo "the cgroup subtree could not be read within the remaining stop reserve"; return 1;; esac
  l="$(listen)" || return 1; [ -z "$l" ] || { echo "listener remains"; return 1; }; return 0; }
unresolved() { local s="(state not read: no time left in the reserve)" r=$(( K2_END - EPOCHSECONDS - GRACE - RECEIPT_RESERVE ))
  if [ $r -ge 2 ]; then CV=$(( r < 20 ? r : 20 )); s="$(T systemctl show $UNIT -p ActiveState -p SubState -p Result -p InvocationID -p NRestarts -p ExecMainStatus 2>&1)"; s="${s//$'\n'/ }"; fi
  end UNRESOLVED "$1 state=[$s]"; }
fail_ours() { local cur nr cur2 nr2
  cur="$(sprop $UNIT InvocationID)" || unresolved "$1; InvocationID read before the stop failed"; nr="$(sprop $UNIT NRestarts)" || unresolved "$1; NRestarts read before the stop failed"
  origin "$cur" "$nr" || unresolved "$1; invocation [$cur] with NRestarts [$nr] (last own: [$INV] at [$NRL]) is NOT from K2's start; NOT stopped"
  echo "PRE-STOP invocation=$cur NRestarts=$nr"
  if stop_verified; then
    # post-stop attribution (LIFECYCLE AND STOP CONTRACT, residual window): any difference, failed read or empty id is UNRESOLVED
    cur2="$(sprop $UNIT InvocationID)" || unresolved "$1; stopped and verified down, but the post-stop InvocationID read failed: the stop's target cannot be attributed (pre-stop invocation=$cur NRestarts=$nr)"
    nr2="$(sprop $UNIT NRestarts)" || unresolved "$1; stopped and verified down, but the post-stop NRestarts read failed: the stop's target cannot be attributed (pre-stop invocation=$cur NRestarts=$nr)"
    echo "POST-STOP invocation=[$cur2] NRestarts=[$nr2]"
    [ -n "$cur2" ] && [ "$cur2" = "$cur" ] && [ "$nr2" = "$nr" ] || unresolved "$1; stopped and verified down, but the invocation changed between the last pre-stop read and the stop (pre-stop invocation=$cur NRestarts=$nr; post-stop invocation=[$cur2] NRestarts=[$nr2]): the stop may have hit another actor's invocation"
    end STOPPED_VERIFIED_DOWN "$1; pre-stop invocation=$cur NRestarts=$nr; post-stop invocation=$cur2 NRestarts=$nr2"
  else unresolved "$1; stop failed or could not be verified (pre-stop invocation=$cur NRestarts=$nr)"; fi; }
printf '%s\n' "K2_BEGIN k1_receipt=$K1S j3_receipt=$J3S priv=$PRIV" > "$PRIV/k2-begin.txt" || die "cannot write k2-begin.txt in PRIV"
xport k2-begin.txt "K2_BEGIN k1_receipt=$K1S j3_receipt=$J3S priv=$PRIV" || die "OUT [$OUT] is not writable as $EXPORT_USER (k2-begin.txt export or readback failed); refused before start"
# ---- FULL ALLOWANCE BEFORE START ----
NR0="$(wprop $UNIT NRestarts)" || die "read NRestarts"; [[ $NR0 =~ ^[0-9]+$ ]] || die "NRestarts is not a number [$NR0]"
_need=$(( START_CAP + GRACE + OBS_ALLOW + STOP_RESERVE + RECEIPT_RESERVE )); _rem=$(( K2_END - EPOCHSECONDS ))
[ $_rem -ge $_need ] || die "${_rem}s left < ${_need}s needed (start ${START_CAP}+${GRACE} + observation ${OBS_ALLOW} + stop ${STOP_RESERVE} + receipt ${RECEIPT_RESERVE}); refused before start"
# ---- start (the one mutation that starts a process) ----
CV=$START_CAP; STARTED=yes; T systemctl start $UNIT; r=$?
INV="$(sprop $UNIT InvocationID)" || unresolved "start rc=$r; InvocationID read failed"; NRL="$NR0"
echo "START_RC=$r INVOCATION=[$INV] NRESTARTS0=$NR0"
if [ -z "$INV" ]; then b="$(sprop $UNIT ActiveState)" || unresolved "start rc=$r; empty InvocationID; ActiveState read failed"
  case "$b" in inactive|failed) STARTED=no; end NOT_STARTED "start rc=$r produced no invocation; unit [$b]; nothing to stop";; *) unresolved "start rc=$r; empty InvocationID while unit is [$b]";; esac; fi
capw 30 || fail_ours "DEADLINE before the post-start property read"
T systemctl show $UNIT -p LoadState -p ActiveState -p SubState -p Result -p ExecMainPID -p ExecMainStatus -p NRestarts -p ControlGroup -p InvocationID -p User -p CapabilityBoundingSet -p NoNewPrivileges -p RestrictSUIDSGID > $PRIV/serve-props.txt || fail_ours "post-start property read failed or timed out"
mapfile -t _sp < $PRIV/serve-props.txt; printf '%s\n' "${_sp[@]}"
[ $r = 0 ] || fail_ours "start rc=$r"
LP=""
while :; do
  capw 60 || fail_ours "DEADLINE: work budget exhausted before a listener appeared"
  c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out while waiting"; n="$(wprop $UNIT NRestarts)" || fail_ours "NRestarts read failed while waiting"
  if [ "$c" != "$INV" ]; then origin "$c" "$n" || unresolved "invocation changed to [$c] WITHOUT an NRestarts increase ([$NRL] -> [$n]): another actor; NOT stopped"; fail_ours "the start crashed (automatic restart observed)"; fi
  b="$(wprop $UNIT ActiveState)" || fail_ours "ActiveState read failed or timed out while waiting"; case "$b" in active|activating) ;; *) fail_ours "STARTUP_ERROR ActiveState=$b";; esac
  capw 30 || fail_ours "DEADLINE before ss"; o="$(T ss -Hltnp 'sport = :18081')" || fail_ours "ss read failed or timed out while waiting"
  LP=""; _lpo=0; while IFS= read -r _ll; do [ -n "$_ll" ] || continue; _lpo=1; _rest="$_ll"; _found=0
    while [[ $_rest =~ pid=([0-9]+) ]]; do _found=1; case " $LP " in *" ${BASH_REMATCH[1]} "*) ;; *) LP="$LP${BASH_REMATCH[1]} ";; esac; _rest="${_rest#*pid=${BASH_REMATCH[1]}}"; done
    [ $_found = 1 ] || fail_ours "a :18081 listener line carries no pid (not attributable)"; done <<< "$o"
  if [ $_lpo = 1 ]; then break; fi; capw 30 || fail_ours "DEADLINE"; T sleep $CV
done
CG2="$(wprop $UNIT ControlGroup)" || fail_ours "ControlGroup read failed or timed out"
c="$(wprop $UNIT InvocationID)" || fail_ours "InvocationID read failed or timed out"; [ "$c" = "$INV" ] || fail_ours "the invocation changed after the listener appeared"
[ -n "$CG2" ] && [ -d "$CGR$CG2" ] || fail_ours "unit cgroup missing"
capw 30 || fail_ours "DEADLINE before the cgroup read"; procs="$(cg_procs "$CG2" $CV)" || fail_ours "cannot read the unit cgroup subtree (failed or out of budget)"
[ -n "$LP" ] || fail_ours "NOT_LISTENING_OR_NOT_IDENTIFIED pids=[none]"
for p in $LP; do [[ $'\n'$procs$'\n' == *$'\n'$p$'\n'* ]] || fail_ours "listener pid $p is not in $UNIT's cgroup subtree"; done
LINE="K2_UP unit=$UNIT invocation=$INV auto_restarts=$AUTO listener_pids=${LP% } cgroup=$CG2 installed_sha=$ISHA k1_receipt=$K1S j3_receipt=$J3S priv=$PRIV"
receipt "$LINE" || unresolved "the serve is UP (invocation $INV) but the K2_UP receipt could not be persisted or read back"
xport k2-receipt.txt "$LINE" || { echo "K2_UNRESOLVED step=X the serve is UP (invocation $INV) and K2_UP is persisted at $PRIV/k2-receipt.txt, but its export to OUT ($OUT) failed or read back different; NOT stopped; evidence retained in PRIV=$PRIV"; echo "K2 NO OBSERVATION (exit 2): X: receipt not exported"; exit 2; }
echo "$LINE"; exit 0
