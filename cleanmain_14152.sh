#!/bin/bash
# TRUE CLEAN-MAIN BASELINE. My earlier 'clean main also refuses' claim was WRONG and is being
# corrected to the parent: that dispatch's arms A and C paired MAIN's 04_infer.dag with MY census
# claim file, which calls unify_generics with text:/module_name: -- parameters main's 5-parameter
# version does not declare. So the 6 diagnostics came from an arity mismatch I created, not from main.
# This script installs main's version of EVERY file my branch touches (staged locally into keep/clean
# from origin/main, because the runner has no origin/main ref) and runs the same instrument. If this
# refuses, the required-regen route is broken independent of this PR; if it is clean, the 6 were mine.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
echo "=== files being reverted to main: $(find keep/clean -type f | wc -l)"
find keep/clean -type f | while read -r f; do
  rel=${f#keep/clean/}
  [ -f "$rel" ] && cp "$f" "$rel"
done
git diff --stat 2>/dev/null | tail -2
echo "clean-main check: dagrelation=$(grep -c generic_types_unify src/v1/04_infer.dag) (expect 0) claimtext=$(grep -c 'text:' src/v1/tests/claim/generic_identity_census.dag) (expect 0) seedfold=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) (expect 0)"
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/cleanmain.log 2>&1
echo "CLEAN-MAIN rc=$?  $(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/cleanmain.log | head -1)"
echo "===== full output after the refusal header, unfiltered ====="
awk '/refused: v2 self-compile/,0' /tmp/cleanmain.log | head -40
echo "===== (end) ====="
grep -a "^required-regen:" /tmp/cleanmain.log | head -4 | cut -c1-220
cp /tmp/cleanmain.log cleanmain.log
# restore my tree from the staged copies (git metadata on the runner is not mine, so no checkout)
echo "=== restoring my files from keep/mine* and seedgreen/ ==="
cp keep/my_04_infer.dag src/v1/04_infer.dag
for f in v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs; do cp "seedgreen/$f" "src/v1/stage0/src/$f"; done
echo "restored dag=$(sha256sum src/v1/04_infer.dag|cut -c1-12) seed=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-12)"
echo CLEANMAIN_DONE
