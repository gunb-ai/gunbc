set -u
echo HEAD=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run() { echo "=== $1"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $1 --functions $2 --hermetic --claim-run > /tmp/cb.log 2>&1; echo "exit=$?"; grep -E "^(PASS|FAIL|UNOBSERVED)|rror:" /tmp/cb.log | head -40; }
run dag/test/claim/approval_apns_publish_witness.dag $(grep -o "^test fn [a-z0-9_]*" dag/test/claim/approval_apns_publish_witness.dag | sed 's/test fn //' | paste -sd, )
run dag/test/claim/approval_request_submission_witness.dag a_filed_request_pushes_every_registered_approver_device_through_apns,an_unavailable_sender_or_no_registered_device_is_named_not_attempted
