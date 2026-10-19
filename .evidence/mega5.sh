set -e
cd /root/workspace/repo-root
HEAD0=$(git rev-parse HEAD)
echo "HEAD-START=$HEAD0"
echo "TOOLCHAIN: $(rustc -V) / $(cargo -V)"
md5sum .evidence/seed.patch .evidence/host.patch dag/test/claim/diagnostic_span_location_rendering_witness.dag
for pf in .evidence/seed.patch .evidence/host.patch; do
  [ -s "$pf" ] || { echo "ABORT-EMPTY-PATCH $pf ($(wc -c < $pf) bytes)"; exit 0; }
done
git checkout -- . 2>/dev/null || true
echo "CLEAN-STATUS:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' '|')]"
# 1. CLEAN TREE: build the REGENERATOR from the committed tree (heal.yml:84-94 route).
CK=$( { git rev-parse HEAD; sha256sum Cargo.lock; rustc -V; cargo -V; } | sha256sum | cut -c1-20 )
mkdir -p /tmp/rcache/$CK
if [ -x /tmp/rcache/$CK/claim_executor ]; then
  cp /tmp/rcache/$CK/claim_executor /tmp/rcache/$CK/gunbc target/release/; echo "REGENERATOR-CACHE-HIT $CK"
else
  set +e; cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/rb.log 2>&1; echo "REGENERATOR-BUILD-EXIT=$?"; set -e
  tail -3 /tmp/rb.log
  cp target/release/claim_executor target/release/gunbc /tmp/rcache/$CK/
fi
sha256sum target/release/claim_executor target/release/gunbc
# 2. SEED EDITS ONLY (additive; exported 1-arg surface unchanged so the crate still builds).
git apply --check .evidence/seed.patch && git apply .evidence/seed.patch && echo "SEED-APPLIED" || { echo "SEED-PATCH-REJECTED"; exit 0; }
# 3. THE REGEN ROUND from the cleanly-built regenerator.
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "REGEN-EXIT=$?"
set -e
grep -aE "required-regen:|rounds|first_generation|^error" /tmp/regen.out | tail -20
echo "ZZ-MIRROR-DIFFSTAT"; git diff --stat src/v1/stage0 | tail -8
echo "ZZ-AUTHORITY-COUNT=$(grep -c render_span_location src/v1/stage0/src/v1_std_core.rs || true)"
echo "ZZ-EOF-FIX=$(grep -c last_code_token_end src/v1/stage0/src/v1_compiler_tokenize.rs || true)"
# 4. HOST EDITS on top of the regenerated mirrors.
git apply --check .evidence/host.patch && git apply .evidence/host.patch && echo "HOST-APPLIED" || { echo "HOST-PATCH-REJECTED"; exit 0; }
# 5. BUILD, print binary sha256 BEFORE the run.
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse > /tmp/b5.log 2>&1
echo "BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/b5.log | head -25 || true
echo "ZZ-BIN-SHA-BEFORE"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
# 6. THE CLAIM. Raw lines, unannotated.
FUNCS=$(grep -oE '^test fn ([a-z0-9_]+)' dag/test/claim/diagnostic_span_location_rendering_witness.dag | awk '{print $3}' | paste -sd, -)
set +e
./target/release/claim_batch --wet --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/test/claim/diagnostic_span_location_rendering_witness.dag --functions "$FUNCS" > /tmp/claim.out 2>&1
echo "CLAIM-EXIT=$?"
set -e
echo "ZZ-CLAIM-ROWS"; grep -E "^PASS |^FAIL " /tmp/claim.out
echo "ZZ-CLAIM-SUMMARY"; tail -4 /tmp/claim.out
echo "ZZ-BIN-SHA-AFTER"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
# 7. THE MIRRORS BACK AS A PATCH, with hashes so the copy can be verified.
echo "ZZ-MIRROR-SHA256"; sha256sum $(git status --porcelain src/v1/stage0 | awk '{print $2}')
git diff src/v1/stage0 > /tmp/mirrors.patch
echo "ZZ-MIRROR-PATCH-SIZE=$(wc -c < /tmp/mirrors.patch) SHA=$(sha256sum /tmp/mirrors.patch | cut -c1-16)"
echo "ZZ-MIRROR-PATCH-B64-START"; base64 -w0 /tmp/mirrors.patch; echo; echo "ZZ-MIRROR-PATCH-B64-END"
echo "HEAD-END=$(git rev-parse HEAD)"
[ "$HEAD0" = "$(git rev-parse HEAD)" ] && echo "HEAD-STABLE=yes" || echo "HEAD-STABLE=NO-RUN-VOID"
echo "ZZ-DONE"
