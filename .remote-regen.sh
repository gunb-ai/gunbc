set -uo pipefail
export RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -3
ls -la target/release/gunbc target/release/claim_executor
for w in dag/test/claim/cloudflare_boot_origin_witness_test.dag dag/test/claim/cloudflare_r2_origin_mint_run_witness_test.dag dag/test/claim/cloudflare_r2_origin_object_witness_test.dag dag/test/claim/network_boot_delivery_join_witness_test.dag dag/test/claim/cloudflare_r2_token_witness_test.dag dag/test/claim/machine_intake/boot_artifact_delivery_witness_test.dag; do
  echo "=== WITNESS $w"
  target/release/gunbc run --source-root dag --source-root src/v2 --entry $w --claim-run 2>&1 | tail -25
  echo "=== exit=${PIPESTATUS[0]}"
done
echo "=== REGEN"
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -v "^$" | tail -30
echo "=== regen-exit=${PIPESTATUS[0]}"
echo "=== DIFF-BEGIN"
cd target/stage0-regen-candidate/src 2>/dev/null && for f in *.rs; do if ! cmp -s "$f" "$OLDPWD/src/v1/stage0/src/$f"; then echo "DRIFT $f"; fi; done; cd "$OLDPWD"
diff -u src/v1/stage0/src/extdeps_uri.rs target/stage0-regen-candidate/src/extdeps_uri.rs > /tmp/uri.patch; echo "patch-lines=$(wc -l < /tmp/uri.patch)"
base64 -w0 /tmp/uri.patch
echo
echo "=== DIFF-END"
