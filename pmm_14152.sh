#!/bin/bash
# POST-MERGE reject/accept split, run IN the mirrored tree (ctrl-build mirrors my
# tracked files as patches; the mirror fidelity is PROVEN by the mirror hashes below
# matching the exact head b5bb104e3ba exactly). Fresh-dir git-archive fails because
# stage0/build.rs requires `git rev-parse HEAD`.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "PMM: HEAD(runner)=$(git rev-parse HEAD)"
declare -A WANT=(
 [src/v1/stage0/src/v1_compiler_infer.rs]=5e9c37585fe088ac
 [src/v1/stage0/src/v1_compiler_infer_types.rs]=60a4e2b1d2ca20dd
 [src/v1/stage0/src/v1_compiler_emit_rust.rs]=bd80c3bd8d3d1058
 [src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs]=07dc012551d40f9f
)
ok=1
for f in "${!WANT[@]}"; do
  got=$(sha256sum "$f" | cut -c1-16)
  [ "$got" = "${WANT[$f]}" ] && echo "PMM: MIRROR-OK $f" || { echo "PMM: MIRROR-MISMATCH $f got=$got want=${WANT[$f]}"; ok=0; }
done
[ "$ok" = 1 ] || { echo "PMM: ABORT mirror not faithful"; exit 3; }
echo "PMM: dag fold U=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) (want 33)"
echo "PMM: reject tests present=$(grep -c "refuses_with_the_fold_message\|differing_argument_list_refuses" src/v1/tests/src/cross_coproduct_generic_locate_test.rs) (want 3+)"
echo "===== TARGETED RUN: cross_coproduct controls ====="
cargo test -p v1-compiler-tests --lib cross_coproduct 2>&1 | grep -E "^test |test result|error" | tail -25
echo "===== FULL v1-compiler-tests --lib (regression sweep) ====="
cargo test -p v1-compiler-tests --lib 2>&1 | grep -E "test result|^error|FAILED|panicked" | tail -15
echo "PMM_DONE"
