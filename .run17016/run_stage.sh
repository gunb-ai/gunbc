#!/usr/bin/env bash
# One stage per heavy-slot acquisition. Holder scan is /proc/<pid>/cgroup heavy-slot +
# readlink cwd/exe ONLY (environ is never read). Yields to lane-mates (still-crab-552,
# vivid-crane-403, valiant-swift-602). Never acquires over a lane-mate.
# Usage: run_stage.sh <HEAD_SHA> <STAGE_NAME> <script-on-srv1-relpath> [args...]
HEAD="$1"; STAGE="$2"; REL="$3"; shift 3
SSH=(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net)
echo "LAUNCHER_HOST=$(hostname) LAUNCHER_BOOT=$(cut -c1-8 /proc/sys/kernel/random/boot_id)"
echo "LAUNCHER_CMD: run_stage.sh $HEAD $STAGE $REL $*"
YIELDED=0
while true; do
  [ "${DRY_GUARD:-0}" = "1" ] && break
  H=$("${SSH[@]}" 'for f in /proc/[0-9]*/cgroup; do if grep -q heavy-slot "$f" 2>/dev/null; then pid=$(echo "$f" | cut -d/ -f3); printf "PID=%s CG=%s CWD=%s EXE=%s\n" "$pid" "$(cat "$f" | tr "\n" ",")" "$(readlink /proc/$pid/cwd 2>/dev/null)" "$(readlink /proc/$pid/exe 2>/dev/null)"; fi; done; echo "QUEUE:"; cat $HOME/.heavy-slots/queue 2>/dev/null | head -8') || { echo "SSH_FAIL holder"; exit 3; }
  LANE=$(printf '%s\n' "$H" | grep -E 'CWD=/home/briansrls/(scratch/(still-crab-552|vivid-crane-403|valiant-swift-602)|worktrees/gunbc/(still-crab-552|vivid-crane-403|valiant-swift-602))' || true)
  if [ -n "$LANE" ]; then echo "YIELD lane-mate holding a seat:"; printf '%s\n' "$LANE" | sed 's/^/    /'; sleep 120; continue; fi
  QLANE=$(printf '%s\n' "$H" | sed -n '/^QUEUE:/,$p' | grep -E 'still-crab-552|vivid-crane-403|valiant-swift-602' || true)
  if [ -n "$QLANE" ]; then echo "YIELD lane-mate WAITING in queue (manager rule: never queue over a waiting lane-mate):"; printf '%s\n' "$QLANE" | sed 's/^/    /'; sleep 120; continue; fi
  LOAD=$("${SSH[@]}" 'awk "{print \$1}" /proc/loadavg')
  LOAD_MAX="${LOAD_MAX:-40}"
  awk -v l="$LOAD" -v m="$LOAD_MAX" 'BEGIN{exit (l+0<m)?0:1}' || { echo "HOT srv1 load1min=$LOAD (>=$LOAD_MAX), waiting"; sleep 120; continue; }
  break
done
echo "=== STAGE $STAGE at HEAD $HEAD; holders at acquire-time (hygiene per eager-koi-371 19:33Z: full detail ONLY for lane matches; foreign entries reduced to counts):"
LANE_ALL=$(printf '%s\n' "$H" | grep -E 'CWD=/home/briansrls/(scratch/(still-crab-552|vivid-crane-403|valiant-swift-602|bold-crane-719|eager-koi-371)|worktrees/gunbc/(still-crab-552|vivid-crane-403|valiant-swift-602|bold-crane-719|eager-koi-371))' || true)
N_HOLDS=$(printf '%s\n' "$H" | grep -c '^PID=' || true)
N_LANE=$(printf '%s\n' "$LANE_ALL" | grep -c '^PID=' || true)
echo "  HOLDERS_TOTAL=$N_HOLDS HOLDERS_LANE=$N_LANE (foreign PID/cgroup/cwd/exe withheld)"
[ -n "$LANE_ALL" ] && printf '%s\n' "$LANE_ALL" | sed 's/^/    LANE: /'
N_Q=$(printf '%s\n' "$H" | sed -n '/^QUEUE:/,$p' | tail -n +2 | grep -c . || true)
echo "  QUEUE_TOTAL=$N_Q (foreign ticket text withheld)"
printf '%s\n' "$H" | sed -n '/^QUEUE:/,$p' | tail -n +2 | grep -E 'bold-crane-719|eager-koi-371|still-crab-552|vivid-crane-403|valiant-swift-602' | sed 's/^/    QUEUE_LANE: /'
# LANE-CAP GUARD (Manager B via eager-koi-371): immediately before the acquisition, read the slot
# who-files and REFUSE with fixed exit 81 if the lane (HEAVY_OWNER tree) already holds >= LANE_CAP
# seats. A .who scan cannot prove the lane is EMPTY (never acquire on scan-clear), but it proves the
# lane is OCCUPIED — which is exactly what a fail-closed guard needs. The guard scan lines stay in
# the receipt as printed facts; the DECISION line carries the decision.
[ -n "${H:-}" ] || H=$("${SSH[@]}" 'for f in /proc/[0-9]*/cgroup; do if grep -q heavy-slot "$f" 2>/dev/null; then pid=$(echo "$f" | cut -d/ -f3); printf "PID=%s CG=%s CWD=%s EXE=%s\n" "$pid" "$(cat "$f" | tr "\n" ",")" "$(readlink /proc/$pid/cwd 2>/dev/null)" "$(readlink /proc/$pid/exe 2>/dev/null)"; fi; done; echo "QUEUE:"; cat $HOME/.heavy-slots/queue 2>/dev/null | head -8')
LANE_WHO_DIR="${LANE_WHO_DIR:-$HOME/.heavy-slots}"
LANE_CAP="${LANE_CAP:-1}"
LANE_TOKEN="${HEAVY_OWNER_TOKEN:-eager-koi-371}"
WHO=$("${SSH[@]}" "for f in $LANE_WHO_DIR/slot*.who; do [ -s \"\$f\" ] && head -1 \"\$f\"; done 2>/dev/null")
LANE_HOLDERS=$(printf '%s\n' "$WHO" | awk -v t="$LANE_TOKEN" '$1==t {c++} END {print c+0}')
if [ "${DRY_GUARD:-0}" = "1" ]; then
  if [ "$LANE_HOLDERS" -ge "$LANE_CAP" ]; then echo "LANE_CAP=$LANE_CAP LANE_HOLDERS=$LANE_HOLDERS DECISION=REFUSE_81 (who-dir=$LANE_WHO_DIR) [DRY_GUARD: no heavy-slot invocation]"; printf '%s\n' "$WHO" | grep "^$LANE_TOKEN" | sed 's/^/    WHO: /'; exit 81; fi
  echo "LANE_CAP=$LANE_CAP LANE_HOLDERS=$LANE_HOLDERS DECISION=WOULD_ACQUIRE (who-dir=$LANE_WHO_DIR) [DRY_GUARD: no heavy-slot invocation]"; exit 0
fi
if [ "$LANE_HOLDERS" -ge "$LANE_CAP" ]; then
  echo "LANE_CAP=$LANE_CAP LANE_HOLDERS=$LANE_HOLDERS DECISION=REFUSE_81 (who-dir=$LANE_WHO_DIR)"
  printf '%s\n' "$WHO" | grep "^$LANE_TOKEN" | sed 's/^/    WHO: /'
  exit 81
fi
echo "LANE_CAP=$LANE_CAP LANE_HOLDERS=$LANE_HOLDERS DECISION=ACQUIRE (who-dir=$LANE_WHO_DIR)"
"${SSH[@]}" "cd \$HOME/scratch/bold-crane-719/run && HEAVY_OWNER=eager-koi-371 /home/briansrls/bin/heavy-slot bash $REL $HEAD $*" > "logs/${STAGE}_${HEAD:0:11}.out" 2>&1 &
SPID=$!
echo "SSH_CHILD_PID=$SPID LAUNCH_UTC=$(date -u +%FT%TZ) OUT=logs/${STAGE}_${HEAD:0:11}.out"
wait $SPID; RC=$?
echo "STAGE ${STAGE} RC=${RC} (real heavy-slot rc; never a pipe's) END_UTC=$(date -u +%FT%TZ)"
# Fail-closed on missing provenance: the leg script appends its own *_LEG_RC line; if the log lacks it,
# the receipt is VOID regardless of how the rc looks (absent-line-means-VOID, same rule as verdicts).
if ! grep -qaE "_LEG_RC=[0-9]+ APPENDED_BY_SCRIPT_AT" "logs/${STAGE}_${HEAD:0:11}.out"; then echo "VOID_RC_LINE_ABSENT in logs/${STAGE}_${HEAD:0:11}.out — leg did not certify its own exit; treat as VOID, not green"; RC=6; fi
if [ "$RC" -eq 143 ] || [ "$RC" -eq 137 ]; then echo "SIGTERM_DETECTED_NOT_MINE rc=$RC at $(date -u +%FT%TZ) -- per B rule: STOP, do not re-launch, do not investigate on srv1"; exit "$RC"; fi
exit $RC
