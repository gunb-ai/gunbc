#!/usr/bin/env bash
# Slim remaining host evidence. Do not commit.
set -u
(
  while true; do echo "HEARTBEAT $(date -u +%FT%TZ)" >&2; sleep 30; done
) &
HB=$!
trap 'kill $HB 2>/dev/null || true' EXIT
# Do not mint a nested cgroup: the last run's cgroup had memory.max but no memory.peak,
# so gunbc test refused MemoryReceiptUnreadable.
echo "cgroup=$(cat /proc/self/cgroup | tr '\n' ' ')"
echo "memory.max=$(cat /sys/fs/cgroup/memory.max 2>/dev/null || echo none)"
ls /sys/fs/cgroup/memory.peak /sys/fs/cgroup/memory.max 2>&1 | head

ROOT="$(pwd)"
report() { echo "VERDICT $*"; }

run_one() {
  local label=$1
  shift
  echo "=== RUN $label ==="
  echo "cmd: $*"
  set +e
  "$@"
  local ex=$?
  set -e
  report "$label exit=$ex"
  return 0
}

run_one BRANCH_BUILD cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin claim_executor
G="$ROOT/target/release/gunbc"
CB="$ROOT/target/release/claim_batch"
CE="$ROOT/target/release/claim_executor"

export HEAL_REPOSITORY="${HEAL_REPOSITORY:-gunb-ai/gunbc}"
export HEAL_PULL_NUMBER="${HEAL_PULL_NUMBER:-13925}"
export HEAL_HEAD_REF="${HEAL_HEAD_REF:-session/bright-deer-140}"
export HEAL_BASE_REF="${HEAL_BASE_REF:-main}"
export HEAL_RUN_ID="${HEAL_RUN_ID:-evidence-local}"

run_one BRANCH_HEAL_OVERLAP "$G" run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet
run_one BRANCH_GUNBC_TEST "$G" test //gunbc/instruments:bare-reference-channel-outcome
run_one BRANCH_CLAIM_BATCH "$CB" --source-root dag --source-root src/v2 \
  --entry dag/test/claim/heal_candidate_witness.dag --function witness_an_empty_listing_is_an_absence \
  --entry dag/test/claim/emitter_bare_variant_expected_adoption_witness.dag --function w_optional_field_bare_variant_keeps_its_wrapper
run_one BRANCH_HEAL_SEED_REGEN "$G" run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet
run_one BRANCH_HEAL_CANDIDATE "$G" run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/heal_candidate_stage0.dag --function heal_candidate_stage0_produce_wet
run_one BRANCH_REGEN "$CE" --regen-round-cost --source-root dag --source-root src/v2

echo "=== FETCH MAIN ==="
git fetch origin main
rm -rf /tmp/gunbc-main
git worktree add --detach /tmp/gunbc-main origin/main
run_one MAIN_BUILD bash -lc 'cd /tmp/gunbc-main && cargo build --release -p v1-compiler --bin gunbc --bin claim_executor'
MG=/tmp/gunbc-main/target/release/gunbc
MCE=/tmp/gunbc-main/target/release/claim_executor
(
  cd /tmp/gunbc-main
  run_one MAIN_HEAL_OVERLAP "$MG" run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet
  run_one MAIN_HEAL_SEED_REGEN "$MG" run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet
  run_one MAIN_REGEN "$MCE" --regen-round-cost --source-root dag --source-root src/v2
)
echo "=== ALL VERDICTS DONE ==="
