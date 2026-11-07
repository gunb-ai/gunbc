#!/bin/bash
# ==== K1 (rev17 DRAFT, review material only): install the reviewed serve unit, then daemon-reload. ====
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
#   - run ONCE (one attempt), from a fresh copy whose sha256 equals the reviewed one, as
#     `sudo -n bash K1.sh <k1-request.txt> <its sha256> <reviewed unit sha256> <OUT dir>`;
#   - serialized with calm-crab's launcher: no keeper launch may be running or start while K1 runs; loyal-deer-449 posts
#     START (this script's sha and its args) to calm-crab before, and END (the receipt and its sha) after;
#   - receipt and reconcile: PRIV/k1-receipt.txt (ONE line) is persisted and read back, then EXPORTED (below), before any success
#     text, and is printed to stdout; the operator captures stdout as the log (K1 writes no asynchronous log); calm-crab
#     reconciles; no second attempt without a new decision.
# OUTPUT DESIGN (sleek-owl msg_cfe1a18a M1, refined by keen-otter msg_c144f271; BLOCKING for privileged execution):
#   - Root opens, creates and writes NOTHING in a user-writable namespace at any time (no log, receipt, scratch or final copy
#     there). Root writes only (a) inside PRIV, a root-private dir created by mktemp under the root-owned /run and then VERIFIED
#     by stat: owner:group 0:0, mode 700, file type directory (a symlink reads `symbolic link` and refuses), and (b) the unit
#     tree it is installing (DEST/STAGE under /etc/systemd/system).
#   - The export to OUT runs with privilege DROPPED: `runuser -u briansrls -- tee -- OUT/<name> < PRIV/<name>`. Root's shell
#     opens ONLY the PRIV file (as stdin; the fd is inherited, never re-opened by path); the open of OUT/<name>, and every path
#     component of OUT, are resolved by a process running as briansrls, so a planted symlink or a renamed OUT reaches only what
#     briansrls could reach anyway. The readback (`runuser -u briansrls -- cat -- OUT/<name>`) must equal the receipt line.
#   - OUT is checked usable BEFORE any mutation: B0 exports k1-begin.txt (the request sha) the same way and reads it back; a
#     failure refuses at B0 with nothing changed.
#   - EVIDENCE: PRIV (receipt, snapshots, inventories, diffs) is retained on every path; the exported copies are OUT/k1-begin.txt
#     and OUT/k1-receipt.txt. An export failure AFTER a mutation is K1_UNRESOLVED step=X, exit 2, with PRIV retained and its
#     path printed to stdout (the persisted PRIV receipt still records the real outcome).
#   - K1 needs J2's request (J2_READY must bind it, J2_PARTIAL absent): J1 -> J2 must have run through calm-crab's launcher
#     first. J3 (the .path units) is NOT part of K1 (J3.md).
# Usage:  K1.sh <J2 k1-request.txt> <its sha256> <the reviewed serve-unit sha256> <OUT dir, written only as briansrls>
# UnitFileState (keen-otter msg_c144f271): B7 requires `disabled`, which systemd reports for a loaded, unlinked unit whose
#   [Install] section names WantedBy=/RequiredBy=/UpheldBy=/Alias=; without one it reports `static`. K1 reads that contract
#   from the ACTUAL reviewed rendered bytes (its snapshot, sha-bound to the reviewed sha; J1R's render check is the upstream
#   copy of the same test) and refuses at B0, before any install, when it is missing.
# INVENTORY ROOTS: a root directory that does not exist is recorded as an explicit `ABSENT <root>` line (an empty scope, the
#   same on J2's and K1's side); a find error (non-zero rc or any stderr) refuses. The two are never conflated.
# Exit: 0 = installed, reloaded, verified AND the K1_OK receipt persisted and read back; 2 = everything else.
# TIME (keen-otter msg_029acc92 #2): ONE whole-act deadline K1_END = start + 2400 s. Every external command runs as
#   `timeout -k 5 <cap>`. A normal step starts only if its FULL cap fits in K1_END - now - 5 (kill grace) - 40 (cleanup
#   reserve) - 30 (receipt reserve); otherwise it is refused BEFORE it starts (before any mutation: K1_REFUSED; after:
#   K1_UNRESOLVED with the boundary's state). Cleanup steps (rm of STAGE/DEST) use the cleanup reserve; the receipt uses the
#   receipt reserve (60 s: persist <= 10, sync <= 10, read <= 10, export <= 15, export readback <= 15). The full success
#   path's caps sum to ~1800 s including graces.
# INPUTS (keen-otter msg_029acc92 #3): every authority input (the request, J2_READY, the unit text, J2's unit inventory) is
#   first SNAPSHOTTED into the root-owned 0700 private dir PRIV with a timed `cp -P`; only the snapshot is hashed, parsed and
#   used. The expected unit sha is an INDEPENDENT argument, and the request's K1_UNIT_SHA must equal it.
# PRIVATE DIR: PRIV=/run/r2-k1.XXXXXX is RETAINED on every path (success, refusal, failure) as evidence; its path is in every
#   receipt and refusal. K1 never deletes it. Removal note: after calm-crab's reconcile the operator removes it (`rm -rf` as
#   root); /run is tmpfs, so a reboot also clears it.
# INVENTORY HANDOFF (keen-otter msg_029acc92 #4): J2 records its post-apply unit inventory (lines `D mode owner:group path`,
#   `F mode owner:group size mtime(integer s) inode path`, `L ... path -> target`, `X mode owner:group path`, `ABSENT root`,
#   sorted, then `--- sha256` and the readable files' hashes). K1 lists the SAME scope with the SAME find -printf format and
#   compares it with the PROJECTION of J2's file (everything before `--- sha256`; the content hashes are excluded because J2
#   cannot read root-only files). Any difference refuses at B0, before any mutation.
# Boundaries and dispositions:
#   B0 before any mutation: refuse (K1_REFUSED), nothing changed.
#   B1 install to STAGE fails          -> STAGE removed (verified or reported); DEST untouched.
#   B2 STAGE readback mismatch         -> STAGE removed; DEST untouched.
#   B3 mv -T STAGE DEST fails          -> STAGE removed; DEST absent (verified or reported).
#   B4 DEST readback mismatch          -> DEST removed (verified or reported); NOT reloaded.
#   B5 something besides DEST changed  -> DEST removed, inventory re-taken and required EQUAL to the pre-inventory (a VERIFIED
#                                         restore), else UNRESOLVED; NOT reloaded.
#   B6 daemon-reload fails             -> UNRESOLVED: DEST present, systemd's view unknown; NO further action.
#   B7 post-reload state/inventory not as expected -> UNRESOLVED; NO further action (nothing is removed after a reload).
#   R  the success receipt cannot be persisted or read back -> UNRESOLVED (the install and reload DID happen), exit 2.
#   X  the export to OUT fails or reads back different after a mutation -> UNRESOLVED, exit 2, PRIV retained and printed.
#   Any refusal for lack of time reports the state at its boundary.
set -uo pipefail
GRACE=5; CLEANUP_RESERVE=40; RECEIPT_RESERVE=60; EXPORT_USER=briansrls; PRIV_OWNER=0:0
UNIT=gunbc-srv1-lab-codex-feedback-v0.service; DEST=/etc/systemd/system/$UNIT; STAGE=/etc/systemd/system/.$UNIT.k1-stage
WD=/home/briansrls/runs/ld449-r2work; MUT=0; RELOADED=no; PRIV="(none yet)"; PRIVOK=no; RECEIPT_OK=no
INV_ROOTS=(/etc/systemd/system /etc/systemd/user /home/briansrls/.config/systemd/user "/run/systemd/system:gunbc-srv1-lab-codex-feedback-v0*")
# ---- receipt: persisted, synced and read back BEFORE any success text (keen-otter msg_029acc92 #1). Returns 1 on any failure.
receipt() { local r c rb; [ $PRIVOK = yes ] || return 1
  r=$(( K1_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 10 ? r : 10 ))
  printf '%s\n' "$1" > "$PRIV/k1-receipt.txt" || return 1
  timeout -k $GRACE $c sync -f "$PRIV/k1-receipt.txt" || return 1
  rb="$(timeout -k $GRACE $c cat -- "$PRIV/k1-receipt.txt")" || return 1; [ "$rb" = "$1" ] || return 1; RECEIPT_OK=yes; return 0; }
# xport NAME TEXT: export PRIV/NAME to OUT/NAME with privilege DROPPED (OUTPUT DESIGN): root opens only the PRIV file, as
# stdin; the writer and the readback run as $EXPORT_USER. Returns 1 on any failure or a readback different from TEXT.
xport() { local r c rb; r=$(( K1_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 15 ? r : 15 ))
  timeout -k $GRACE $c runuser -u $EXPORT_USER -- tee -- "$OUT/$1" < "$PRIV/$1" > /dev/null || return 1
  r=$(( K1_END - EPOCHSECONDS - GRACE )); [ $r -ge 1 ] || return 1; c=$(( r < 15 ? r : 15 ))
  rb="$(timeout -k $GRACE $c runuser -u $EXPORT_USER -- cat -- "$OUT/$1")" || return 1; [ "$rb" = "$2" ]; }
state() { printf 'dest=%s stage=%s reloaded=%s priv=%s' "$([ -e $DEST ] && echo present || echo absent)" "$([ -e $STAGE ] && echo present || echo absent)" "$RELOADED" "$PRIV"; }
# end CODE STEP REASON: persist the receipt line, then report; exit 2 (every non-success end goes through here)
end() { local line st; st="$(state)"; line="K1_$1 step=$2 $st reason=$3"
  if receipt "$line"; then echo "$line"
    if ! xport k1-receipt.txt "$line"; then if [ $MUT = 1 ]; then echo "K1_UNRESOLVED step=X after a mutation the receipt's export to OUT ($OUT) failed or read back different; evidence retained in PRIV=$PRIV"
      else echo "K1 export of the refusal receipt to OUT failed; it is retained in PRIV=$PRIV"; fi; fi
  else echo "K1_UNRESOLVED step=receipt could not persist [$line]"; fi
  echo "K1 NO OBSERVATION (exit 2): $2: $3"; exit 2; }
die() { if [ $MUT = 1 ]; then end UNRESOLVED "$1" "$2"; else end REFUSED "$1" "$2"; fi; }
fail() { end "${3:-FAILED}" "$1" "$2"; }
# capv N STEP: CV = N if the FULL cap fits before the reserves, else refuse BEFORE the step (set in THIS shell, never in $(...)).
capv() { local r=$(( K1_END - EPOCHSECONDS - GRACE - CLEANUP_RESERVE - RECEIPT_RESERVE )); [ $r -ge $1 ] || die "$2" "${r}s left before the reserves < this step's ${1}s cap; refused before the step"; CV=$1; }
# capc N: cleanup slice inside the cleanup reserve (CV=0 = no time: the caller reports the cleanup as not attempted)
capc() { local r=$(( K1_END - EPOCHSECONDS - GRACE - RECEIPT_RESERVE )); CV=$(( r < $1 ? r : $1 )); [ $CV -ge 1 ] || CV=0; }
rmc() { capc 30; [ $CV -ge 1 ] && timeout -k $GRACE $CV rm -f -- "$1"; [ ! -e "$1" ]; }   # cleanup: true only if verified absent
T() { timeout -k $GRACE "$CV" "$@"; }
# showdiff A B STEP: a timed diff into PRIV, shown with builtins (first 40 lines); display only, its status is not a gate
showdiff() { local -a dl; capv 30 "$3"; T diff "$1" "$2" > "$PRIV/diff-$3.txt"; mapfile -t -n 40 dl < "$PRIV/diff-$3.txt"; [ ${#dl[@]} -gt 0 ] && printf '%s\n' "${dl[@]}"; return 0; }
# ---- B0 ----
[ "$EUID" = 0 ] || { echo "K1 NO OBSERVATION (exit 2): B0: not root"; exit 2; }
export PATH=/usr/sbin:/usr/bin:/sbin:/bin LC_ALL=C
for _en in $(compgen -e); do [[ $_en == GUNBC_* ]] && { echo "K1 NO OBSERVATION (exit 2): B0: GUNBC_* variables are exported (names only checked)"; exit 2; }; done
[[ ${EPOCHSECONDS:-} =~ ^[0-9]+$ ]] || { echo "K1 NO OBSERVATION (exit 2): B0: bash >= 5 required"; exit 2; }
K1_END=$(( EPOCHSECONDS + 2400 ))
REQ="${1:-}"; REQS="${2:-}"; USHA="${3:-}"; OUT="${4:-}"
[[ $REQ = /* ]] && [[ $REQS =~ ^[0-9a-f]{64}$ ]] && [[ $USHA =~ ^[0-9a-f]{64}$ ]] && [[ $OUT = /* ]] && [ $# = 4 ] || { echo "K1 NO OBSERVATION (exit 2): B0: usage: K1.sh <k1-request.txt> <sha256> <reviewed unit sha256> <OUT dir>"; exit 2; }
capv 10 B0; PRIV="$(T mktemp -d /run/r2-k1.XXXXXX)" || die B0 "mktemp of the private dir failed"; capv 10 B0; T chmod 0700 "$PRIV" || die B0 "chmod private dir"
capv 10 B0; _ps="$(T stat -c '%u:%g:%a:%F' -- "$PRIV")" || die B0 "stat of the private dir failed"
[ "$_ps" = "$PRIV_OWNER:700:directory" ] && [ ! -L "$PRIV" ] || die B0 "the private dir [$PRIV] is [$_ps], not owner $PRIV_OWNER mode 700 directory (a symlink refuses)"
PRIVOK=yes; echo "K1 private dir (retained, verified $_ps): $PRIV"
snap() { capv 30 B0; [ -e "$1" ] || die B0 "$3 [$1] is absent"; T cp -P -- "$1" "$PRIV/$2" || die B0 "cannot snapshot $3"; [ -f "$PRIV/$2" ] && [ ! -L "$PRIV/$2" ] || die B0 "$3 is not a regular file"; }
hsnap() { local h; capv 30 B0; h="$(T sha256sum -- "$PRIV/$1")" || die B0 "cannot hash the snapshot of $2"; [ "${h%% *}" = "$3" ] || die B0 "the snapshot of $2 is not the cited bytes"; }
snap "$REQ" request "the request"; hsnap request "the request" "$REQS"
mapfile -t RL < "$PRIV/request" || die B0 "cannot read the request snapshot"; [ ${#RL[@]} = 4 ] || die B0 "the request is not exactly 4 lines"
[ "${RL[0]}" = "K1_REQUEST unit=$UNIT dest=$DEST" ] || die B0 "request line 1 is not the fixed unit and destination"
[[ ${RL[1]} =~ ^K1_UNIT_SHA\ ([0-9a-f]{64})$ ]] || die B0 "request line 2"; [ "${BASH_REMATCH[1]}" = "$USHA" ] || die B0 "the request's K1_UNIT_SHA is not the reviewed unit sha given as argument 4"
[[ ${RL[2]} =~ ^K1_UNIT_TEXT\ (/[^[:space:]]+)$ ]] || die B0 "request line 3"; USRC="${BASH_REMATCH[1]}"
[[ ${RL[3]} =~ ^K1_J2_INVENTORY\ ([0-9a-f]{64})\ (/[^[:space:]]+)$ ]] || die B0 "request line 4"; ISHA="${BASH_REMATCH[1]}"; ISRC="${BASH_REMATCH[2]}"
{ [ -e $WD/J2_PARTIAL ] || [ -L $WD/J2_PARTIAL ]; } && die B0 "$WD/J2_PARTIAL exists (J2 did not finish cleanly)"
snap $WD/markers/J2_READY J2_READY "the J2_READY marker"
mapfile -t ML < "$PRIV/J2_READY" || die B0 "cannot read the J2_READY snapshot"
bound() { local l n=0; for l in "${ML[@]}"; do [ "$l" = "RECEIPT $1 $2" ] && n=$((n+1)); done; [ $n = 1 ]; }
bound "$REQS" "$REQ" || die B0 "J2_READY does not bind exactly this request ($REQS $REQ)"
bound "$USHA" "$USRC" || die B0 "J2_READY does not bind the unit text at the reviewed sha"
bound "$ISHA" "$ISRC" || die B0 "J2_READY does not bind the J2 unit inventory"
snap "$USRC" unit "the unit text"; hsnap unit "the unit text" "$USHA"
# [Install] contract from the reviewed rendered bytes (see UnitFileState): refuse here, before any install, when it would be static
capv 30 B0; _ic="$(T awk '/^\[/ {s = ($0 == "[Install]")} s && /^(WantedBy|RequiredBy|UpheldBy|Alias)=[^[:space:]]/ {n++} END {print n+0}' "$PRIV/unit")" || die B0 "cannot read the [Install] contract of the unit snapshot"
[[ $_ic =~ ^[0-9]+$ ]] && [ "$_ic" -ge 1 ] || die B0 "the reviewed unit has no [Install] section naming WantedBy/RequiredBy/UpheldBy/Alias: it would load as static, not disabled (refused before install)"
snap "$ISRC" j2inv "J2's unit inventory"; hsnap j2inv "J2's unit inventory" "$ISHA"
{ [ -e $DEST ] || [ -L $DEST ] || [ -e $STAGE ] || [ -L $STAGE ]; } && die B0 "$DEST or its stage already exists"
capv 30 B0; v="$(T systemctl show $UNIT -p LoadState --value)" || die B0 "read LoadState"; [ "$v" = not-found ] || die B0 "$UNIT is [$v], not not-found"
# inv FILE STEP: J2's scope and J2's find -printf format exactly (see INVENTORY HANDOFF); FILE = the sorted listing;
# FILE.nd = FILE without DEST and STAGE (path field: 4 for D/X lines, 7 for F/L lines). Returns 1 on any error or stderr.
inv() { local r root filt; : > "$1.l" && : > "$1.err" || return 1
  for r in "${INV_ROOTS[@]}"; do root="${r%%:*}"; filt=""; [ "$root" != "$r" ] && filt="${r#*:}"
    if [ ! -d "$root" ]; then echo "ABSENT $root" >> "$1.l" || return 1; continue; fi
    capv 120 "$2"; T bash -c 'f=(-true); [ -n "$2" ] && f=(-name "$2"); find "$1" -xdev "${f[@]}" \( \( -type d -printf "D %m %u:%g %p\n" \) -o \( -type l -printf "L %m %u:%g %s %Ts %i %p -> %l\n" \) -o \( -type f -printf "F %m %u:%g %s %Ts %i %p\n" \) -o -printf "X %m %u:%g %p\n" \)' _ "$root" "$filt" >> "$1.l" 2>> "$1.err" || return 1; done
  [ -s "$1.err" ] && return 1; capv 30 "$2"; T sort "$1.l" > "$1" || return 1
  capv 30 "$2"; T awk -v d="$DEST" -v s="$STAGE" '{p = ($1=="D" || $1=="X") ? $4 : $7} (p==d || p==s) {next} {print}' "$1" > "$1.nd"; }
inv "$PRIV/pre" B0 || die B0 "pre-inventory failed"
# J2's projection: everything before the `--- sha256` separator (exactly one separator required)
capv 30 B0; _ns="$(T grep -cx -- '--- sha256' "$PRIV/j2inv")"; [ "$_ns" = 1 ] || die B0 "J2's inventory has not exactly one '--- sha256' separator"
capv 30 B0; T awk '$0=="--- sha256" {exit} {print}' "$PRIV/j2inv" > "$PRIV/j2proj" || die B0 "projection of J2's inventory failed"
capv 30 B0; T cmp -s "$PRIV/j2proj" "$PRIV/pre"; case $? in 0) ;; 1) showdiff "$PRIV/j2proj" "$PRIV/pre" B0; die B0 "the unit-config tree changed since J2's post-apply inventory (refused before any mutation)";; *) die B0 "compare with J2's inventory failed";; esac
printf '%s\n' "K1_BEGIN request=$REQS unit=$USHA priv=$PRIV" > "$PRIV/k1-begin.txt" || die B0 "cannot write k1-begin.txt in PRIV"
xport k1-begin.txt "K1_BEGIN request=$REQS unit=$USHA priv=$PRIV" || die B0 "OUT [$OUT] is not writable as $EXPORT_USER (k1-begin.txt export or readback failed); refused before any mutation"
echo "K1 B0 held: request $REQS (snapshot), unit $USHA, J2 inventory $ISHA equal to K1's pre-inventory, dest absent, $UNIT not-found, priv $PRIV"
# ---- mutations ----
MUT=1
capv 60 B1; if ! T install -m 0644 -o root -g root -- "$PRIV/unit" $STAGE; then rmc $STAGE; fail B1 "install to stage failed"; fi
capv 30 B2; h="$(T sha256sum -- $STAGE)" && [ "${h%% *}" = "$USHA" ] || { rmc $STAGE; fail B2 "stage readback is not the reviewed sha"; }
capv 30 B3; if ! T mv -T -- $STAGE $DEST; then rmc $STAGE; fail B3 "rename failed"; fi
capv 30 B4; h="$(T sha256sum -- $DEST)" && [ "${h%% *}" = "$USHA" ] || { rmc $DEST; fail B4 "installed bytes are not the reviewed sha; NOT reloaded"; }
inv "$PRIV/mid" B5 || { rmc $DEST; fail B5 "inventory failed; dest removal attempted; NOT reloaded" UNRESOLVED; }
capv 30 B5; T cmp -s "$PRIV/pre.nd" "$PRIV/mid.nd"; case $? in 0) ;; *) showdiff "$PRIV/pre.nd" "$PRIV/mid.nd" B5
  rmc $DEST
  if inv "$PRIV/back" B5 && capv 30 B5 && T cmp -s "$PRIV/pre" "$PRIV/back"; then fail B5 "an entry other than $UNIT changed; dest removed and the inventory VERIFIED equal to pre; NOT reloaded"
  else fail B5 "an entry other than $UNIT changed; dest removal or re-inventory did NOT verify; NOT reloaded" UNRESOLVED; fi;; esac
capv 120 B6; T systemctl daemon-reload || fail B6 "daemon-reload failed; no further action" UNRESOLVED
RELOADED=yes
for p in LoadState:loaded ActiveState:inactive UnitFileState:disabled; do capv 30 B7; v="$(T systemctl show $UNIT -p ${p%%:*} --value)" || fail B7 "read ${p%%:*} failed" UNRESOLVED
  [ "$v" = "${p##*:}" ] || fail B7 "${p%%:*}=[$v], expected ${p##*:}" UNRESOLVED; done
inv "$PRIV/post" B7 || fail B7 "post-reload inventory failed" UNRESOLVED
capv 30 B7; T cmp -s "$PRIV/mid" "$PRIV/post" || fail B7 "the inventory changed across daemon-reload" UNRESOLVED
capv 30 B7; P0="$(T sha256sum -- "$PRIV/pre")" || fail B7 "hash pre-inventory" UNRESOLVED; capv 30 B7; P1="$(T sha256sum -- "$PRIV/post")" || fail B7 "hash post-inventory" UNRESOLVED
LINE="K1_OK unit=$UNIT dest=$DEST installed_sha=$USHA request=$REQS request_path=$REQ j2_inventory=$ISHA pre_inventory=${P0%% *} post_inventory=${P1%% *} LoadState=loaded ActiveState=inactive UnitFileState=disabled priv=$PRIV"
receipt "$LINE" || { echo "K1_UNRESOLVED step=R the install and reload HAPPENED but the success receipt could not be persisted or read back ($(state))"; echo "K1 NO OBSERVATION (exit 2): R: receipt not persisted"; exit 2; }
xport k1-receipt.txt "$LINE" || { echo "K1_UNRESOLVED step=X the install and reload HAPPENED and K1_OK is persisted at $PRIV/k1-receipt.txt, but its export to OUT ($OUT) failed or read back different; evidence retained in PRIV=$PRIV"; echo "K1 NO OBSERVATION (exit 2): X: receipt not exported"; exit 2; }
echo "$LINE"; exit 0
