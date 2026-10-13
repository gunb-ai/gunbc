#!/usr/bin/env bash
# Untracked. Do not git add. Remote BuildBuddy must receive this file in the dispatch tree.
set -euo pipefail
cd "$(dirname "$0")"
MEM_MAX_BYTES=30064771072
CG="/sys/fs/cgroup/c2-verify-$$"
if [[ ! -d /sys/fs/cgroup ]]; then
  echo "c2_verify: no cgroup filesystem" >&2
  exit 2
fi
mkdir -p "${CG}"
echo "${MEM_MAX_BYTES}" > "${CG}/memory.max"
echo 0 > "${CG}/memory.swap.max" 2>/dev/null || true
echo "$$" > "${CG}/cgroup.procs"
MEM="$(cat /sys/fs/cgroup/memory.max 2>/dev/null || true)"
# After moving, this process's cgroup memory.max must be the pin, not max.
OWN_CG="$(cat /proc/self/cgroup | sed -n 's/^0:://p')"
OWN_MAX="$(cat "/sys/fs/cgroup${OWN_CG}/memory.max" 2>/dev/null || true)"
if [[ -z "${OWN_MAX}" || "${OWN_MAX}" == "max" ]]; then
  echo "c2_verify: refusing an unbounded memory.max (own=${OWN_MAX})" >&2
  exit 2
fi
echo "cgroup=${OWN_CG} memory.max=${OWN_MAX}"
export CARGO_TARGET_DIR="${PWD}/target"
cargo build --release -p v1-compiler --bin gunbc
GUNBC="./target/release/gunbc"
run_claims() {
  local entry="$1"
  echo "=== claim-run ${entry} ==="
  "${GUNBC}" run --source-root dag --source-root src/v2 --entry "${entry}" --claim-run
}
run_claims dag/test/claim/gcp_iam_converge_witness.dag
echo "=== MUTATION: payload permission on secretPolicy role ==="
sed -i '0,/  "secretmanager.secrets.get",/s//  "secretmanager.secrets.get",\n  "secretmanager.versions.access",/' dag/gunbc/auth/gcp_iam_converge.dag
grep -c 'secretmanager.versions.access' dag/gunbc/auth/gcp_iam_converge.dag
set +e
run_claims dag/test/claim/gcp_iam_converge_witness.dag
