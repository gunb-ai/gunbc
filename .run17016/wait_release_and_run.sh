#!/usr/bin/env bash
# Wait for valiant-swift-602 to release (no seat held by any lane-mate, no lane-mate ticket),
# then acquire stage2 once. Guard scan printed before acquisition (run_stage prints its own too).
SSH=(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net)
START=$(date +%s)
while true; do
  if [ $(( $(date +%s) - START )) -gt 10800 ]; then echo "WATCH_TIMEOUT_3H no acquisition"; exit 7; fi
  H=$("${SSH[@]}" 'for f in /proc/[0-9]*/cgroup; do if grep -q heavy-slot "$f" 2>/dev/null; then pid=$(echo "$f" | cut -d/ -f3); printf "PID=%s CG=%s CWD=%s EXE=%s\n" "$pid" "$(cat "$f" | tr "\n" ",")" "$(readlink /proc/$pid/cwd 2>/dev/null)" "$(readlink /proc/$pid/exe 2>/dev/null)"; fi; done; echo "QUEUE:"; cat $HOME/.heavy-slots/queue 2>/dev/null | head -8')
  LANE=$(printf '%s\n' "$H" | grep -E 'still-crab-552|vivid-crane-403|valiant-swift-602|bold-crane-719' || true)
  if [ -n "$LANE" ]; then echo "[$(date -u +%FT%TZ)] lane-mate present, waiting:"; printf '%s\n' "$LANE" | sed 's/^/    /'; sleep 180; continue; fi
  break
done
echo "[$(date -u +%FT%TZ)] GUARD SCAN at release (verbatim):"
printf '%s\n' "$H" | sed 's/^/    /'
echo "[$(date -u +%FT%TZ)] acquiring stage2"
bash run_stage.sh a8d7e7a4eab815982f75a3b086bb084ad822f942 stage2 stage2_run.sh 34
RC=$?
echo "WATCHER_STAGE2_RC=$RC at $(date -u +%FT%TZ)"
exit "$RC"
