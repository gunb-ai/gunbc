#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
S=$HOME/scratch/swift-hawk-617
HEAD_SHA=e948da35ce057267455a9ab666fc8ffaf0719a6b
ENTRY=src/v2/test/claim/self_host/stage0_prefix_inner_reason_probe.dag
FS=prefix_emits_holds,prefix_stage_emission_holds,prefix_stage_inference_holds,prefix_reason_is_transform_shape_holds,prefix_reason_is_arity_invalid_holds,prefix_reason_is_catalog_miss_holds,prefix_reason_is_operator_shape_unsupported_holds,prefix_reason_is_realization_arity_mismatch_holds,corpus_emits_holds,corpus_stage_emission_holds,corpus_stage_inference_holds,corpus_reason_is_transform_shape_holds,corpus_reason_is_arity_invalid_holds,corpus_reason_is_catalog_miss_holds,corpus_reason_is_operator_shape_unsupported_holds,corpus_reason_is_realization_arity_mismatch_holds,negate_emits_holds,negate_stage_emission_holds,negate_reason_is_transform_shape_holds,negate_reason_is_arity_invalid_holds,negate_reason_is_catalog_miss_holds,negate_reason_is_operator_shape_unsupported_holds,negate_reason_is_realization_arity_mismatch_holds,nwrap_emits_holds,nwrap_stage_emission_holds,nwrap_reason_is_transform_shape_holds,nwrap_reason_is_arity_invalid_holds,nwrap_reason_is_catalog_miss_holds,nwrap_reason_is_operator_shape_unsupported_holds,nwrap_reason_is_realization_arity_mismatch_holds,nwrap_reason_equals_standalone_negate_holds
cd "$S/gunbc_green" || { echo "NO_TREE"; echo "CLAIM_BATCH_RC=9"; exit 9; }
git -C "$S/gunbc_green" fetch origin session/swift-hawk-617 2>&1 | tail -2
git -C "$S/gunbc_green" checkout -f "$HEAD_SHA" 2>&1 | tail -2
echo "=== probe_reason wrapper pid=$$ start=$(date -Iseconds) ==="
echo "HOST=$(uname -n)"
echo "BOOT=$(cut -c1-8 /proc/sys/kernel/random/boot_id)"
echo "HEAD=$HEAD_SHA"
echo "TREE_HEAD=$(git rev-parse HEAD)"
echo "WORKING_TREE_DIRTY=$( [ -n "$(git status --porcelain --untracked-files=no)" ] && echo yes || echo no )"
echo "CLAIM_BATCH_SHA256=$(sha256sum "$S/bin/claim_batch" | awk '{print $1}')"
echo "ENTRY=$ENTRY"
echo "FUNCTIONS=$FS"
echo "FULL_COMMAND=claim_batch --source-root $S/gunbc_green/dag --source-root $S/gunbc_green/src/v2 --entry $ENTRY --functions $FS"
HEAVY_OWNER=swift-hawk-617 "$HOME/bin/heavy-slot" bash -lc "cd $S/gunbc_green && timeout 3600 $S/bin/claim_batch --source-root $S/gunbc_green/dag --source-root $S/gunbc_green/src/v2 --entry $ENTRY --functions $FS"
echo "CLAIM_BATCH_RC=$?"
echo "=== wrapper end $(date -Iseconds) ==="
