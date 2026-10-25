set -u
echo HEAD=$(git rev-parse HEAD) dirty=$(git status --porcelain | wc -l)
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -3
run() { echo "=== $1"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $1 --functions $2 --hermetic --claim-run > /tmp/cb.log 2>&1; echo "exit=$?"; grep -E "^(PASS|FAIL|UNOBSERVED)|rror|typecheck" /tmp/cb.log | head -60; }
run dag/test/claim/host_credential_custody_converge_witness.dag every_rostered_row_places,every_dispatch_option_selects_its_own_row
run dag/test/claim/gcp_secret_access_witness.dag the_roster_is_the_single_authority_for_the_spark_and_mtcollins1_grants
run dag/test/claim/gcp_iam_converge_witness.dag the_fleet_roster_set_is_every_non_root_roster_row,no_grant_set_names_a_trust_root
run dag/test/claim/approval_device_routes_witness.dag malformed_bodies_refuse_400_on_every_route,the_non_writer_process_is_refused_on_the_mutating_routes,the_push_update_refuses_503_before_any_crypto_while_the_native_handler_is_unbound,the_roadmap_mounts_no_device_route
