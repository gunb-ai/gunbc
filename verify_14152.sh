#!/usr/bin/env bash
# Verification at head 5d750054b34 for #14152: does the test crate still build and pass
# after the reject rows, the discharge control, the RFM row rewrite, and the deletion of
# both probe files? Full v1-compiler-tests lib run, so a deletion that breaks an
# unrelated control (claim registration, ledger parsing) surfaces here rather than in CI.
set -u
echo "HEAD=$(git rev-parse HEAD)"
echo "SEED sig=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-12) (expect 36 / 5e9c37585fe0)"
echo "probe files gone: $(ls dag/test/claim/constructor_tag_route_probe_red.dag src/v2/test/claim/manual/constructor_tag_route_probe_red.dag 2>&1 | head -2)"

echo "########## FULL v1-compiler-tests lib run ##########"
cargo test -p v1-compiler-tests --lib > /tmp/full.log 2>&1
echo "FULL_EXIT=$?"
echo "--- failures:"
grep -E "^test .* FAILED|^failures:|^error" /tmp/full.log | head -30
echo "--- failure list:"
sed -n '/^failures:/,/test result/p' /tmp/full.log | grep -E "^ +v1_tests|^ +cross|::" | sort -u | head -25
echo "--- result lines:"
grep -E "test result:" /tmp/full.log | tail -3
echo "--- my own rows:"
grep -E "^test cross_coproduct" /tmp/full.log | sed 's/ \.\.\. ◐ started compile.frontend//' | sed 's/^test //; s/ \.\.\. / => /' | sort | tail -20
echo "########## DONE ##########"
