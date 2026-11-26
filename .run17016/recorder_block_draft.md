# Recorder block draft — egress-effects-1-filesystem-provider (node://egress-effects-1-filesystem-provider)
# Kind: RedProbe (dag/gunbc/roadmap/roadmap_status.dag :126-127). NOT a historical pair:
# gunbc lands squash and the witness is born in this PR, so no tree at red_at's parent can run it.
# B waived the verify_reverts phrase (tool does not exist at origin/main; both greps print 0 hits).
# Certification is the MEASURED block below — printed, not asserted.
red_kind=probe
green_at=M            # M = squash-merge sha of gunbc#17016 on main (fill after merge)
red_base=M            # == red_at's first parent, certified by prints 1-2
red_at=<plant-sha>    # single-commit child of M
probe_ref=refs/heads/red-control/egress-effects-1-filesystem-provider  # ONE non-force push, only after prints 1-6 appear

# Plant: single-commit child of M reverting the fix at carrier grain, witness UNTOUCHED:
#   effect_demand.dag, name_resolve.dag, demand_engine_restatement_follow_up.dag restored to pre-PR states;
#   dag/test/claim/filesystem_effect_reach_wall.dag kept byte-identical to its landed blob at M.

# MEASURED CERTIFICATION PRINTS (all must appear before the push):
#  1. git rev-list --parents -n1 <plant-sha>          -> exactly '... M' (first parent == M)
#  2. git merge-base --is-ancestor M <plant-sha>; rc  # 0
#  3. git diff --numstat M..<plant-sha>               -> names exactly the three reverted files, nothing else
#  4. git rev-parse M:dag/test/claim/filesystem_effect_reach_wall.dag        (landed blob)
#  5. git rev-parse <plant-sha>:dag/test/claim/filesystem_effect_reach_wall.dag  (must equal print 4)
#  6. diff of src/v1/stage0 between M and plant -> empty; print STAGE0_RUST_DIFFERS=0

# Both legs, ONE binary (same sha256 printed beside both), built at M, inside heavy-slot, run route:
#   gunbc run --source-root dag --source-root src/v1 --source-root src/v2 \
#     --entry dag/test/claim/filesystem_effect_reach_wall.dag --function <claim>, per claim.
# GREEN leg at M tree: all 34 claims rc 0. RED leg at plant tree: refusal rc != 0, non-panic, names its cause.
# Each leg coloured by its own rc lines; zero-selected = VOID; rc 101 VOIDs (panic is not refusal).
