# 17016 run staging (srv1, ~/scratch/bold-crane-719)

- HEAD tree: head-$FULL_SHA (pushed as refs/heads/head-<sha> from the worktree).
- FLOOR module + floor binary: the cutover main 4036f188602 is NOT on srv1 yet; the
  floor-side legs (stage2b floor-module, stageF floor batch) must run under
  ~/scratch/<owner>/cutover trees per eager-koi's correction. Until a floor binary is
  named in-thread, stages 2b/F run with the head-built binary against the head's OWN
  copy of the floor module — labelled 'head-built on floor-module copy, floor-binary
  leg owed' — the floor-binary re-run is a separate acquisition when the binary lands.
- Probes edit the FLOOR module copy at $T/src/v2/test/claim/filesystem_effect_reach_wall.dag
  (correct post-cutover path; the dag/test/claim/ path is the pre-cutover head's own copy).
- Every stage prints HOST/BOOT/sha256/built-commit/load; run_stage.sh prints holders
  before acquisition and yields on a lane-mate (one of still-crab-552/vivid-crane-403/bold-crane-719).
- The old stage_17016_final.sh (pid 1098035) is superseded at head 640492783bfd; kill
  only after re-reading /proc/1098035/environ (HEAVY_OWNER) + cwd under my scratch.
