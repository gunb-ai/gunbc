#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
S=$HOME/scratch/swift-hawk-617
HEAD_SHA=59fedef65b86427d6261b193906f2f33d0c879a3
cd "$S/gunbc_green" || { echo "NO_TREE"; echo "CLAIM_BATCH_RC=9"; exit 9; }
git -C "$S/gunbc_green" fetch origin session/swift-hawk-617 2>&1 | tail -2
git -C "$S/gunbc_green" checkout -f "$HEAD_SHA" 2>&1 | tail -2
FS=conj_binary_emits_holds,disj_binary_emits_holds,prefix_not_emits_holds,literal_compare_emits_holds,int_literal_value_emits_holds,corpus_probes_emits_holds,corpus_no_longer_refused_at_emission_holds,corpus_reason_is_not_transform_shape_holds,negate_refuses_not_emits_holds,negate_reason_is_named_not_transform_shape_holds,negate_reason_is_not_body_scope_holds,negate_reason_names_operator_operand_or_arity_holds,inner_refusal_surfaces_inner_reason_holds,prec_or_grouped_and_flat_bodies_differ_holds,prec_or_grouped_has_grouping_holds,prec_or_flat_has_no_grouping_holds,prec_not_grouped_and_flat_bodies_differ_holds,prec_not_grouped_wraps_operand_holds,flat_binding_operand_emits_without_redundant_parens_holds
echo "=== green_consolidated wrapper pid=$$ start=$(date -Iseconds) ==="
echo "HEAD=$HEAD_SHA"
echo "TREE_HEAD=$(git rev-parse HEAD)"
echo "WORKING_TREE_DIRTY=$( [ -n "$(git status --porcelain --untracked-files=no)" ] && echo yes || echo no )"
echo "CLAIM_BATCH_SHA256=$(sha256sum "$S/bin/claim_batch" | awk '{print $1}')"
echo "FUNCTIONS=$FS"
echo "ENTRY=src/v2/test/claim/self_host/stage0_conjunction_emission_claims.dag"
echo "FULL_COMMAND=claim_batch --source-root $S/gunbc_green/dag --source-root $S/gunbc_green/src/v2 --entry src/v2/test/claim/self_host/stage0_conjunction_emission_claims.dag --functions $FS"
HEAVY_OWNER=swift-hawk-617 "$HOME/bin/heavy-slot" bash -lc "cd $S/gunbc_green && timeout 3600 $S/bin/claim_batch --source-root $S/gunbc_green/dag --source-root $S/gunbc_green/src/v2 --entry src/v2/test/claim/self_host/stage0_conjunction_emission_claims.dag --functions $FS"
echo "CLAIM_BATCH_RC=$?"
echo "=== wrapper end $(date -Iseconds) ==="
