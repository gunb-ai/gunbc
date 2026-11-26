#!/usr/bin/env bash
# One stage per heavy-slot acquisition, with yield-before-acquire and per-stage holder print.
# Usage: run_stage.sh <HEAD_SHA> <STAGE_NAME> <script-on-srv1-relpath>
HEAD="$1"; STAGE="$2"; REL="$3"
SSH=(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net)
SCRATCH="~/scratch/bold-crane-719"
YIELDED=0
while true; do
  H=$("${SSH[@]}" "for d in \$HOME/.heavy-slots/slot-*; do [ -d \"\$d\" ] || continue; for p in \$d/*.pid; do [ -f \"\$p\" ] || continue; pid=\$(cat \"\$p\" 2>/dev/null); [ -n \"\$pid\" ] || continue; owner=\$(tr '\0' '\n' < /proc/\$pid/environ 2>/dev/null | grep '^HEAVY_OWNER=' | cut -d= -f2); cwd=\$(readlink /proc/\$pid/cwd 2>/dev/null); exe=\$(readlink /proc/\$pid/exe 2>/dev/null); echo \"slot=\$(basename \$d) pid=\$pid owner=\$owner cwd=\$cwd exe=\$exe\"; done; done; cat \$HOME/.heavy-slots/queue 2>/dev/null | head -8") || { echo "SSH_FAIL holder"; exit 3; }
  LANE=$(printf '%s\n' "$H" | grep -E 'owner=eager-koi-371' | grep -E 'cwd=/home/briansrls/scratch/(bold-crane-719|still-crab-552|vivid-crane-403)\b' | grep -v "bold-crane-719" || true)
  if [ -n "$LANE" ] && [ "$YIELDED" -eq 0 ]; then echo "YIELD lane-mate waiting (own tag, not one of three)"; sleep 120; YIELDED=1; continue; fi
  if [ -n "$LANE" ]; then echo "YIELD lane-mate holding"; sleep 120; continue; fi
  LOAD=$(awk '{print $1}' /proc/loadavg)
  "${SSH[@]}" "awk '{print \$1}' /proc/loadavg" | awk '{exit ($1<40)?0:1}' || { echo "HOT srv1 load ($LOAD)"; sleep 120; continue; }
  break
done
echo "=== STAGE $STAGE at HEAD $HEAD; holders:"
printf '%s\n' "$H" | sed 's/^/    /'
"${SSH[@]}" "cd \$HOME/scratch/bold-crane-719 && HEAVY_OWNER=eager-koi-371 /home/briansrls/bin/heavy-slot bash $REL"
RC=$?
echo "STAGE ${STAGE} RC=${RC} (real heavy-slot rc; never a pipe)"
exit $RC
