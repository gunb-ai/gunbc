#!/usr/bin/env bash
# Mutation-witness gate for W2 foldable-now-A (all 10, non-negotiable per roster mgr).
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"
bin="${V2_COMPILER:-target/release/gunbc}"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc not found at $bin" >&2
  exit 2
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

claim_run() {
  local entry="$1" function="$2"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run 2>&1
}

expect_green() {
  local label="$1" entry="$2" function="$3"
  local out result
  out="$(claim_run "$entry" "$function")"
  result="$(echo "$out" | tail -1)"
  if [[ "$result" != "true" ]]; then
    echo "FAIL baseline GREEN: $label (got '$result')" >&2
    echo "$out" >&2
    exit 1
  fi
  echo "GREEN $label"
}

expect_red() {
  local label="$1" entry="$2" function="$3"
  local out result exit_code=0
  out="$(claim_run "$entry" "$function")" || exit_code=$?
  result="$(echo "$out" | tail -1)"
  if [[ "$result" == "true" && "$exit_code" -eq 0 ]]; then
    echo "FAIL mutation RED: $label stayed GREEN" >&2
    echo "$out" >&2
    exit 1
  fi
  echo "RED   $label (result='$result' exit=$exit_code)"
}

# 1 introspect advisory
entry="$tmpdir/1.dag"
sed 's/apply_lens_introspect_rejection_is_advisory_claim_holds()/apply_lens_introspect_rejection_is_advisory_claim_holds() == false/' \
  src/v4/test/claim/lens_application/sg_claims.dag > "$entry"
expect_green "1.introspect" src/v4/test/claim/lens_application/sg_claims.dag lens_application_introspect_advisory_holds
expect_red "1.introspect" "$entry" lens_application_introspect_advisory_holds

# 2 synthesis gap
entry="$tmpdir/2.dag"
sed 's/synthesis_gap_poly_lens_non_empty()/synthesis_gap_poly_lens_non_empty() == false/' \
  src/v4/test/claim/lens_application/sg_claims.dag > "$entry"
expect_green "2.synthesis_gap" src/v4/test/claim/lens_application/sg_claims.dag lens_application_synthesis_gap_polynomial_holds
expect_red "2.synthesis_gap" "$entry" lens_application_synthesis_gap_polynomial_holds

# 3 idempotency
entry="$tmpdir/3.dag"
sed 's/idempotency_write_effect_claim_holds()/idempotency_write_effect_claim_holds() == false/' \
  src/v4/test/claim/lens_idempotency/sg_claims.dag > "$entry"
expect_green "3.idempotency" src/v4/test/claim/lens_idempotency/sg_claims.dag lens_idempotency_write_effect_holds
expect_red "3.idempotency" "$entry" lens_idempotency_write_effect_holds

# 4 identical variant
entry="$tmpdir/4.dag"
sed 's/Unrealized => true/NotDuplicatePayload => true/' \
  src/v4/test/claim/lens_identical_variant_payload/sg_claims.dag > "$entry"
expect_green "4.identical_variant" src/v4/test/claim/lens_identical_variant_payload/sg_claims.dag lens_identical_variant_payload_unrealized_scaffold_holds
expect_red "4.identical_variant" "$entry" lens_identical_variant_payload_unrealized_scaffold_holds

# 5 registry required ids
entry="$tmpdir/5.dag"
sed 's/lens_registry_ids_resolve(lens_ids: lens_registry_v0_required_ids)/false/' \
  src/v4/test/claim/lens_registry/sg_claims.dag > "$entry"
expect_green "5.registry_ids" src/v4/test/claim/lens_registry/sg_claims.dag lens_registry_required_ids_resolve_holds
expect_red "5.registry_ids" "$entry" lens_registry_required_ids_resolve_holds

# 6 registry singleton
entry="$tmpdir/6.dag"
sed 's/lens_registry_row_count(lens_id: Complexity) == 1/lens_registry_row_count(lens_id: Complexity) == 2/' \
  src/v4/test/claim/lens_registry/sg_claims.dag > "$entry"
expect_green "6.registry_singleton" src/v4/test/claim/lens_registry/sg_claims.dag lens_registry_singleton_row_counts_holds
expect_red "6.registry_singleton" "$entry" lens_registry_singleton_row_counts_holds

# 7 boundary prune — wrong expected frontier
entry="$tmpdir/7.dag"
sed 's/irt1_boundary_prune_receipt_claim_holds()/irt1_boundary_prune_receipt_claim_holds() == false/' \
  src/v4/test/claim/lens_affected_set/sg_claims.dag > "$entry"
expect_green "7.boundary_prune" src/v4/test/claim/lens_affected_set/sg_claims.dag lens_affected_set_irt1_boundary_prune_holds
expect_red "7.boundary_prune" "$entry" lens_affected_set_irt1_boundary_prune_holds

# 8 fail closed pending
entry="$tmpdir/8.dag"
sed 's/fail_closed_pending_escalation_claim_holds()/fail_closed_pending_escalation_claim_holds() == false/' \
  src/v4/test/claim/lens_affected_set/sg_claims.dag > "$entry"
expect_green "8.fail_closed_pending" src/v4/test/claim/lens_affected_set/sg_claims.dag lens_affected_set_irt1_fail_closed_pending_escalation_holds
expect_red "8.fail_closed_pending" "$entry" lens_affected_set_irt1_fail_closed_pending_escalation_holds

# 9 fail closed absorption
entry="$tmpdir/9.dag"
sed 's/irt1_fail_closed_absorption_receipt_claim_holds()/irt1_fail_closed_absorption_receipt_claim_holds() == false/' \
  src/v4/test/claim/lens_affected_set/sg_claims.dag > "$entry"
expect_green "9.fail_closed_absorption" src/v4/test/claim/lens_affected_set/sg_claims.dag lens_affected_set_irt1_fail_closed_absorption_holds
expect_red "9.fail_closed_absorption" "$entry" lens_affected_set_irt1_fail_closed_absorption_holds

# 10 empty diff
entry="$tmpdir/10.dag"
sed 's/irt1_empty_diff_frontier_receipt_claim_holds()/irt1_empty_diff_frontier_receipt_claim_holds() == false/' \
  src/v4/test/claim/lens_affected_set/sg_claims.dag > "$entry"
expect_green "10.empty_diff" src/v4/test/claim/lens_affected_set/sg_claims.dag lens_affected_set_irt1_empty_diff_frontier_holds
expect_red "10.empty_diff" "$entry" lens_affected_set_irt1_empty_diff_frontier_holds

echo "::notice title=W2 mutation witness::all 10 foldable-now-A witnesses GREEN baseline + RED under mutation"
