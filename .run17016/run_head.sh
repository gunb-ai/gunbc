#!/usr/bin/env bash
# Full sequence at one head. Order: fetch floor main (plain git) -> floor binary (slot)
# -> head tree push/fetch (plain git) -> head build (slot) -> G gate -> 2 module ->
# 2b floor-module -> P1 -> P2 -> F floor batch. One acquisition per slot stage; the
# runner YIELDS between stages when a lane-mate (still-crab-552/vivid-crane-403,
# bold-crane-719 excluded only for own leftovers) holds or waits.
HEAD="$1"; SHORT="${HEAD:0:11}"
ROOT="/home/briansrls/.worktrees/gunbc/bold-crane-719"
R="$ROOT/.run17016"; LOGDIR="$R/logs"; mkdir -p "$LOGDIR"
SSH=(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net)
say(){ printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*"; }
for REL in stage_common.sh stage_fetch.sh stage0_main_tree.sh stage0b_floor_bin.sh stage1_build.sh stageG_gate.sh stage2_module.sh stage2b_floorsrc.sh stageP1_probe.sh stageP2_probe.sh stageF_floor.sh run_stage.sh; do
  "${SSH[@]}" "mkdir -p \$HOME/scratch/bold-crane-719" >/dev/null && "${SSH[@]}" "cat > \$HOME/scratch/bold-crane-719/$REL" < "$R/$REL" || { say "PUSH_FAIL $REL"; exit 4; }
done
git -C "$ROOT" rev-parse --verify "$HEAD^{commit}" >/dev/null 2>&1 || { say "HEAD_NOT_LOCAL"; exit 5; }
git -C "$ROOT" push -f -q origin "$HEAD:refs/heads/head-$HEAD" || { say "HEAD_PUSH_FAIL"; exit 5; }
git ls-remote origin "refs/heads/head-$HEAD" | grep -q "$HEAD" && say "LSREMOTE_OK head-$SHORT" || { say "LSREMOTE_MISSING"; exit 5; }
say "FETCH floor main (plain git, no slot)"
"${SSH[@]}" "cd \$HOME/scratch/bold-crane-719 && bash stage0_main_tree.sh" 2>&1 | tee "$LOGDIR/floormain_${SHORT}.log"
[ "${PIPESTATUS[0]}" -eq 0 ] || { say "FLOORMAIN_FAIL"; exit 6; }
say "FETCH head tree (plain git, no slot)"
"${SSH[@]}" "cd \$HOME/scratch/bold-crane-719 && bash stage_fetch.sh $HEAD" 2>&1 | tee "$LOGDIR/fetch_${SHORT}.log"
[ "${PIPESTATUS[0]}" -eq 0 ] || { say "FETCH_FAIL"; exit 6; }
for PAIR in "stage0b_floor_bin.sh 0b" "stage1_build.sh 1" "stageG_gate.sh G" "stage2_module.sh 2" "stage2b_floorsrc.sh 2b" "stageP1_probe.sh P1" "stageP2_probe.sh P2" "stageF_floor.sh F"; do
  set -- $PAIR; REL=$1; ST=$2
  say "=== stage $ST ($REL) head=$SHORT"
  bash "$R/run_stage.sh" "$HEAD" "$ST" "$REL" 2>&1 | tee -a "$LOGDIR/stage${ST}_${SHORT}.log"
  RC=${PIPESTATUS[0]}
  say "stage $ST rc=$RC"
  [ "$RC" -eq 0 ] || { say "STOP nonzero rc at stage $ST"; exit "$RC"; }
  sleep 5
done
say "ALL STAGES COMPLETE head=$SHORT"
