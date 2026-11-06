#!/bin/bash
# ==== K1 (rev16 DRAFT, review material only): install the reviewed serve unit, then daemon-reload. ====
# keen-otter rulings msg_bc8bc508 / msg_e41d8062: the root acts left J2/J4; this script is the exact root act for the install.
# CAPABILITY / PROVENANCE (keen-otter msg_560eeac5):
#   escalation msg_e8888e14, created 2026-11-06T20:12:59Z. Question: "Who performs the srv1 lab restoration's root acts (install
#     serve unit, daemon-reload, start/stop)? keen-otter requires a named root holder." Options: "you run reviewed K1/K2 scripts
#     with sudo yourself" (recommended) | "you attest existing exact NOPASSWD argv rules on srv1".
#   operator's direct chat reply, ~20:15Z: "You have srv1 access to briansrls, I give you permission to run those"
#   keen-otter's resolution (msg_560eeac5): the permission covers loyal-deer-449, as briansrls, performing the described
#     reviewed K1/K2 acts. It does NOT include J3. It must not be re-asked. Sudo capability stays UNVERIFIED (no probe is
#     made; a `sudo -n` refusal at run time is a stop); the execution frame stays UNDECIDED; all review, chain and
#     exact-packet holds remain.
# EXECUTION FRAME (review material, not decided; per calm-crab msg_e90e4723 and keen-otter msg_e41d8062):
#   - run ONCE (one attempt), from a fresh copy whose sha256 equals the reviewed one, as `sudo -n bash K1.sh <OUT> <req> <sha>`;
#   - serialized with calm-crab's launcher: no keeper launch may be running or start while K1 runs; loyal-deer-449 posts
#     START (with this script's sha and its args) to calm-crab before, and END (with the receipt and its sha) after;
#   - receipt and reconcile discipline: OUT/k1-receipt.txt (one line, K1_OK or K1_FAILED/K1_UNRESOLVED with the state) and
#     OUT/k1.log are retained and handed to calm-crab, who reconciles them against its own ledger; no second attempt follows
#     a failure without a new decision.
#   - K1 needs J2's request (J2_READY must exist and bind it, J2_PARTIAL must be absent): J1 -> J2 must have run through
#     calm-crab's launcher first. J3 (the .path units) is NOT part of K1 and is not authorized by the K1/K2 permission.
# Usage:  K1.sh <OUT (must not exist)> <J2 k1-request.txt> <its sha256>
# Exit: 0 = installed, reloaded, verified (K1_OK); 2 = refused or failed, with the exact state in the receipt.
# Boundaries and dispositions (every step: one timeout under ONE whole-act deadline; status checked BEFORE content):
#   B0 before any mutation: refuse, nothing changed.
#   B1 install to STAGE fails      -> STAGE removed (verified absent or reported); DEST untouched.
#   B2 STAGE readback mismatch     -> STAGE removed; DEST untouched.
#   B3 mv -T STAGE DEST fails      -> STAGE removed; DEST absent (verified or reported).
#   B4 DEST readback mismatch      -> DEST removed (verified or reported); NOT reloaded.
#   B5 something besides DEST changed (inventory before reload) -> DEST removed, inventory re-taken and required EQUAL to the
#                                     pre-inventory (VERIFIED restore), else UNRESOLVED; NOT reloaded.
#   B6 daemon-reload fails         -> UNRESOLVED: DEST present, systemd's view unknown; NO further action.
#   B7 post-reload state/inventory not as expected -> UNRESOLVED; NO further action (no removal after a reload).
#   A deadline reached at any boundary is reported with the state at that boundary (never a silent exit).
set -uo pipefail
die() { echo "K1 NO OBSERVATION (exit 2): $*"; exit 2; }
[ "$EUID" = 0 ] || die "B0: not root (K1 is the root act; run by the named holder only)"
export PATH=/usr/sbin:/usr/bin:/sbin:/bin LC_ALL=C
for _en in $(compgen -e); do [[ $_en == GUNBC_* ]] && die "B0: GUNBC_* variables are exported (names only checked)"; done
[[ ${EPOCHSECONDS:-} =~ ^[0-9]+$ ]] || die "B0: bash >= 5 required"
K1_END=$(( EPOCHSECONDS + 900 ))   # ONE whole-act deadline
UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; STAGE=/etc/systemd/system/.$UNIT.k1-stage
WD=/home/briansrls/runs/ld449-r2work; MUT=0; RELOADED=no
OUT="${1:-}"; REQ="${2:-}"; REQS="${3:-}"
[[ $OUT = /* ]] && [[ $REQ = /* ]] && [[ $REQS =~ ^[0-9a-f]{64}$ ]] || die "B0: usage: K1.sh <OUT> <k1-request.txt> <sha256>"
{ [ -e "$OUT" ] || [ -L "$OUT" ]; } && die "B0: OUT $OUT already exists"; mkdir -m 0755 -- "$OUT" || die "B0: mkdir OUT"
exec > >(tee -a "$OUT/k1.log") 2>&1 || die "B0: cannot log to OUT"
receipt() { printf '%s\n' "$1" > "$OUT/k1-receipt.txt" || echo "K1 WARNING: receipt write failed: $1"; echo "$1"; }
state() { printf 'dest=%s stage=%s reloaded=%s' "$([ -e $DEST ] && echo present || echo absent)" "$([ -e $STAGE ] && echo present || echo absent)" "$RELOADED"; }
fail() { local st; st="$(state)"; receipt "K1_${3:-FAILED} step=$1 $st reason=$2"; die "$1: $2 ($st)"; }
# capv N STEP: CV = min(N, K1_END - now), set IN THIS SHELL; a reached deadline reports the boundary's state (never inside $(...)).
capv() { local r=$(( K1_END - EPOCHSECONDS )); if [ $r -lt 5 ]; then [ $MUT = 1 ] && fail "$2" "K1 deadline reached" UNRESOLVED; die "$2: K1 deadline reached before this step"; fi; CV=$(( r < $1 ? r : $1 )); }
# ---- B0: inputs, binding, pre-state ----
[ -f "$REQ" ] && [ ! -L "$REQ" ] || die "B0: the request is not a regular file"
capv 30 B0; h="$(timeout $CV sha256sum -- "$REQ")" || die "B0: cannot hash the request"; [ "${h%% *}" = "$REQS" ] || die "B0: the request is not the cited bytes"
# the request: EXACTLY 3 lines, as J2 writes them as its last act
mapfile -t RL < "$REQ" || die "B0: cannot read the request"; [ ${#RL[@]} = 3 ] || die "B0: the request is not exactly 3 lines"
[ "${RL[0]}" = "K1_REQUEST unit=$UNIT dest=$DEST" ] || die "B0: request line 1 is not the fixed unit and destination"
[[ ${RL[1]} =~ ^K1_UNIT_SHA\ ([0-9a-f]{64})$ ]] || die "B0: request line 2"; USHA="${BASH_REMATCH[1]}"
[[ ${RL[2]} =~ ^K1_UNIT_TEXT\ (/[^[:space:]]+)$ ]] || die "B0: request line 3"; USRC="${BASH_REMATCH[1]}"
# J2_READY must bind THIS request (exact RECEIPT line) and J2 must not be partial
{ [ -e $WD/J2_PARTIAL ] || [ -L $WD/J2_PARTIAL ]; } && die "B0: $WD/J2_PARTIAL exists (J2 did not finish cleanly)"
M=$WD/markers/J2_READY; [ -f $M ] && [ ! -L $M ] || die "B0: J2_READY marker missing"
mapfile -t ML < $M || die "B0: cannot read J2_READY"; _b=0; for _l in "${ML[@]}"; do [ "$_l" = "RECEIPT $REQS $REQ" ] && _b=$((_b+1)); done
[ $_b = 1 ] || die "B0: J2_READY does not bind exactly this request ($REQS $REQ)"
# BIND THE BYTES: a regular, non-symlink source copied into a private root-owned dir; THAT copy is hashed and installed.
[ -f "$USRC" ] && [ ! -L "$USRC" ] || die "B0: the unit text is not a regular file"
PRIV="$(mktemp -d /run/r2-k1.XXXXXX)" || die "B0: mktemp"; chmod 0700 "$PRIV" || die "B0: chmod private dir"
capv 30 B0; timeout $CV cp -- "$USRC" "$PRIV/unit" || die "B0: cannot copy the unit text"
capv 30 B0; h="$(timeout $CV sha256sum -- "$PRIV/unit")" || die "B0: cannot hash the private copy"; [ "${h%% *}" = "$USHA" ] || die "B0: the unit text is not the reviewed sha (the mutable source moved)"
{ [ -e $DEST ] || [ -L $DEST ] || [ -e $STAGE ] || [ -L $STAGE ]; } && die "B0: $DEST or its stage already exists"
capv 30 B0; v="$(timeout $CV systemctl show $UNIT -p LoadState --value)" || die "B0: read LoadState"; [ "$v" = not-found ] || die "B0: $UNIT is [$v], not not-found"
# inv FILE STEP: /etc/systemd/system recursively + this instance's names (any depth) under /run/systemd/system. Directories by
# type, mode, owner only (adding DEST changes the parent's mtime by design); other entries by type, mode, owner, size, mtime,
# inode, path, link target. FILE.nd = FILE without DEST and STAGE (path field: 4 for d lines, 7 otherwise). Returns 1 on error.
inv() { capv 120 "$2"; timeout $CV find /etc/systemd/system -xdev \( \( -type d -printf 'd %m %u:%g %p\n' \) -o -printf '%y %m %u:%g %s %T@ %i %p -> %l\n' \) > "$1.raw" 2> "$1.err" || return 1
  capv 60 "$2"; timeout $CV find /run/systemd/system -xdev -name 'gunbc-srv1-lab-codex-feedback-v0*' \( \( -type d -printf 'd %m %u:%g %p\n' \) -o -printf '%y %m %u:%g %s %T@ %i %p -> %l\n' \) >> "$1.raw" 2>> "$1.err" || return 1
  [ -s "$1.err" ] && return 1; sort "$1.raw" > "$1" || return 1; awk -v d="$DEST" -v s="$STAGE" '{p = ($1=="d") ? $4 : $7} (p==d || p==s) {next} {print}' "$1" > "$1.nd"; }
inv "$PRIV/pre" B0 || die "B0: pre-inventory failed"
echo "K1 B0 held: request $REQS bound by J2_READY, unit $USHA, dest absent, $UNIT not-found, pre-inventory taken"
# ---- mutations: every refusal from here reports the state at its boundary ----
MUT=1
# B1
capv 60 B1; if ! timeout $CV install -m 0644 -o root -g root -- "$PRIV/unit" $STAGE; then timeout 30 rm -f -- $STAGE; fail B1 "install to stage failed"; fi
# B2
capv 30 B2; h="$(timeout $CV sha256sum -- $STAGE)" && [ "${h%% *}" = "$USHA" ] || { timeout 30 rm -f -- $STAGE; fail B2 "stage readback is not the reviewed sha"; }
# B3
capv 30 B3; if ! timeout $CV mv -T -- $STAGE $DEST; then timeout 30 rm -f -- $STAGE; fail B3 "rename failed"; fi
# B4: the bytes ACTUALLY installed
capv 30 B4; h="$(timeout $CV sha256sum -- $DEST)" && [ "${h%% *}" = "$USHA" ] || { timeout 30 rm -f -- $DEST; fail B4 "installed bytes are not the reviewed sha; NOT reloaded"; }
# B5: nothing besides DEST changed, before the reload makes anything live
inv "$PRIV/mid" B5 || { timeout 30 rm -f -- $DEST; fail B5 "inventory failed; dest removal attempted; NOT reloaded" UNRESOLVED; }
cmp -s "$PRIV/pre.nd" "$PRIV/mid.nd"; case $? in 0) ;; *) diff "$PRIV/pre.nd" "$PRIV/mid.nd" | head -40
  timeout 30 rm -f -- $DEST
  if inv "$PRIV/back" B5 && cmp -s "$PRIV/pre" "$PRIV/back"; then fail B5 "an entry other than $UNIT changed; dest removed and the inventory VERIFIED equal to pre; NOT reloaded"
  else fail B5 "an entry other than $UNIT changed; dest removal or re-inventory did NOT verify; NOT reloaded" UNRESOLVED; fi;; esac
# B6
capv 120 B6; timeout $CV systemctl daemon-reload || fail B6 "daemon-reload failed; no further action" UNRESOLVED
RELOADED=yes
# B7
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled; do capv 30 B7; v="$(timeout $CV systemctl show $UNIT -p ${p%%:*} --value)" || fail B7 "read ${p%%:*} failed" UNRESOLVED
  [ "$v" = "${p##*:}" ] || fail B7 "${p%%:*}=[$v], expected ${p##*:}" UNRESOLVED; done
inv "$PRIV/post" B7 || fail B7 "post-reload inventory failed" UNRESOLVED; cmp -s "$PRIV/mid" "$PRIV/post" || fail B7 "the inventory changed across daemon-reload" UNRESOLVED
P0="$(sha256sum -- "$PRIV/pre")" || fail B7 "hash pre-inventory" UNRESOLVED; P1="$(sha256sum -- "$PRIV/post")" || fail B7 "hash post-inventory" UNRESOLVED
receipt "K1_OK unit=$UNIT dest=$DEST installed_sha=$USHA request=$REQS pre_inventory=${P0%% *} post_inventory=${P1%% *} LoadState=loaded ActiveState=inactive UnitFileState=disabled"
exit 0
