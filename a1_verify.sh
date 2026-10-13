#!/usr/bin/env bash
# Untracked. Do not git add.
set -euo pipefail
cd "$(dirname "$0")"
MEM_MAX_BYTES=17179869184
CG="/sys/fs/cgroup/a1-verify-$$"
mkdir -p "${CG}"
echo "${MEM_MAX_BYTES}" > "${CG}/memory.max"
echo 0 > "${CG}/memory.swap.max" 2>/dev/null || true
echo "$$" > "${CG}/cgroup.procs"
OWN_CG="$(sed -n 's/^0:://p' /proc/self/cgroup)"
OWN_MAX="$(cat "/sys/fs/cgroup${OWN_CG}/memory.max" 2>/dev/null || true)"
if [[ -z "${OWN_MAX}" || "${OWN_MAX}" == "max" ]]; then
  echo "refusing unbounded memory.max" >&2
  exit 2
fi
echo "cgroup=${OWN_CG} memory.max=${OWN_MAX}"
export CARGO_TARGET_DIR="${PWD}/target"
(
  while true; do echo "HEARTBEAT $(date -u +%H:%M:%S)"; sleep 60; done
) &
HB=$!
trap 'kill ${HB} 2>/dev/null || true' EXIT
unset RUSTC_WRAPPER || true
/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc --bin claim_batch
GUNBC="./target/release/gunbc"
BATCH="./target/release/claim_batch"
echo "=== claim-run dag/test/claim/host_credential_custody_converge_witness.dag ==="
"${GUNBC}" run --source-root dag --source-root src/v2 --entry dag/test/claim/host_credential_custody_converge_witness.dag --claim-run
WET_FNS="a_planted_staging_symlink_does_not_publish_or_touch_its_referent_by_real_execution,an_ordinary_publication_lands_a_regular_file_by_real_execution,a_second_publication_over_an_existing_regular_receipt_stays_a_regular_file_by_real_execution"
run_wet() {
  local tag="$1"
  echo "=== claim_batch --wet ${tag} ==="
  "${BATCH}" --wet --source-root dag --source-root src/v2 --entry dag/test/claim/host_custody_export_receipt_wet_witness.dag --functions "${WET_FNS}"
}
run_wet green
echo "=== MUTATION drop unlink-staging ==="
python3 - <<'PY'
from pathlib import Path
p = Path("dag/gunbc/typed_remote_file_write.dag")
t = p.read_text()
old = '''    TypedRemoteFileWriteStep {
      step: "unlink-staging",
      argv: typed_remote_file_unlink_staging_argv(write: write),
      stdin_payload: "",
    },
'''
if old not in t:
    raise SystemExit("unlink step not found")
p.write_text(t.replace(old, "", 1))
print("dropped unlink-staging step")
PY
set +e
mut_log="$(mktemp)"
run_wet mutant 2>&1 | tee "${mut_log}"
echo "MUTATION_CLAIM_RUN_EXIT=${PIPESTATUS[0]}"
set -e
planted_fail=$(grep -c 'FAIL a_planted_staging_symlink_does_not_publish_or_touch_its_referent_by_real_execution' "${mut_log}" || true)
ord_pass=$(grep -c 'PASS an_ordinary_publication_lands_a_regular_file_by_real_execution' "${mut_log}" || true)
sec_pass=$(grep -c 'PASS a_second_publication_over_an_existing_regular_receipt_stays_a_regular_file_by_real_execution' "${mut_log}" || true)
echo "MUTATION planted_fail=${planted_fail} ordinary_pass=${ord_pass} second_pass=${sec_pass}"
if [[ "${planted_fail}" != "1" || "${ord_pass}" != "1" || "${sec_pass}" != "1" ]]; then
  echo "MUTATION_VERDICT unexpected" >&2
  exit 1
fi
echo "MUTATION_DONE planted-symlink FAIL; two ordinary PASS"
