#!/usr/bin/env bash
# chain_v2: B-ordered evidence for the ONE re-launch at head a8d7e7a4eab.
# Order: script sha256+bytes (local+srv1) -> stage 0 (no slot, real rc) -> acquisitions.
# One acquisition per stage job (run_stage.sh); SIGTERM(143/137) aborts the chain per B rule 3.
set -u
cd "$(dirname "$0")" || exit 9
# Single-chain lock: two chains ran concurrently at 13:32Z (a pkill self-match left v2e alive
# while v2f started). One chain per head, lock held for the run.
exec 8> chain.lock
flock -n 8 || { echo "CHAIN_LOCK_HELD another chain is running; refusing to start"; exit 5; }
HEAD=a8d7e7a4eab815982f75a3b086bb084ad822f942
SSH=(ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net)
echo "CHAIN_V2_START_UTC=$(date -u +%FT%TZ) CHAIN_PID=$$ HEAD=$HEAD"
echo "=== EVIDENCE BLOCK 1: stage script sha256 + byte counts (local) ==="
sha256sum stage_common.sh run_stage.sh stage0_main_tree.sh stage_fetch.sh stage1_build.sh stage2_module.sh stage2_run.sh stageG_gate.sh stageP1_probe.sh stageP2_probe.sh stage0b_floor_bin.sh stage2b_floorsrc.sh stageF_floor.sh
wc -c stage_common.sh run_stage.sh stage0_main_tree.sh stage_fetch.sh stage1_build.sh stage2_module.sh stage2_run.sh stageG_gate.sh stageP1_probe.sh stageP2_probe.sh stage0b_floor_bin.sh stage2b_floorsrc.sh stageF_floor.sh
echo "=== EVIDENCE BLOCK 1b: same on srv1 (must match) ==="
"${SSH[@]}" 'cd ~/scratch/bold-crane-719/run && sha256sum stage_common.sh run_stage.sh stage0_main_tree.sh stage_fetch.sh stage1_build.sh stage2_module.sh stage2_run.sh stageG_gate.sh stageP1_probe.sh stageP2_probe.sh stage0b_floor_bin.sh stage2b_floorsrc.sh stageF_floor.sh && wc -c stage_common.sh run_stage.sh stage0_main_tree.sh stage_fetch.sh stage1_build.sh stage2_module.sh stage2_run.sh stageG_gate.sh stageP1_probe.sh stageP2_probe.sh stage0b_floor_bin.sh stage2b_floorsrc.sh stageF_floor.sh'
echo "SRV1_HASHES_RC=$?"
echo "=== EVIDENCE BLOCK 2: stage 0 (no slot) with real rc ==="
bash run_noslot.sh stage0_main_tree.sh; RC0=$?
echo "STAGE0_RC=$RC0"
[ "$RC0" -ne 0 ] && { echo "CHAIN_STOP stage0 rc=$RC0"; exit "$RC0"; }
bash run_noslot.sh stage_fetch.sh "$HEAD"; RCF=$?
echo "STAGE_FETCH_RC=$RCF"
[ "$RCF" -ne 0 ] && { echo "CHAIN_STOP stage_fetch rc=$RCF"; exit "$RCF"; }
echo "=== EVIDENCE BLOCK 3: acquisitions, one per job, in order ==="
for STAGE in stage1 stagePrecheck stage2 stageG stageP1 stageP2 stage0b stage2b stageF; do
  case "$STAGE" in
    stage1) REL=stage1_build.sh;; stagePrecheck) REL=stage_precheck.sh;; stage2) REL=stage2_run.sh;; stageG) REL=stageG_gate.sh;;
    stageP1) REL=stageP1_probe.sh;; stageP2) REL=stageP2_probe.sh;; stage0b) REL=stage0b_floor_bin.sh;;
    stage2b) REL=stage2b_floorsrc.sh;; stageF) REL=stageF_floor.sh;;
  esac
  bash run_stage.sh "$HEAD" "$STAGE" "$REL"; RC=$?
  echo "CHAIN_STEP $STAGE rc=$RC at $(date -u +%FT%TZ)"
  if [ "$RC" -ne 0 ]; then echo "CHAIN_STOP after $STAGE rc=$RC"; exit "$RC"; fi
done
echo "CHAIN_V2_COMPLETE_UTC=$(date -u +%FT%TZ)"
