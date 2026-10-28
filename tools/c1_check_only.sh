set -uo pipefail
echo "SUBJECT_SHA=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR="$PWD/target"
/opt/cargo/bin/cargo check --release -p v1-compiler --bin gunbc --bin claim_batch --message-format short 2>&1 | grep -E "error|warning: unused|Finished" | head -30
echo "check_rc=${PIPESTATUS[0]}"
echo MARKER_CHECK_DONE
