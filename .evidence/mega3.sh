set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
[ "$(git rev-parse HEAD)" = "2817c10af02705a778f49122ec78d537c232b4ac" ] || { echo "HEAD-MISMATCH-ABORT"; exit 0; }
git checkout -- . 2>/dev/null || true
echo "CLEAN-STATUS:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' ' ')]"
md5sum .evidence/seed.patch .evidence/host.patch .evidence/claim.dag
echo "TOOLCHAIN: $(rustc -V) / $(cargo -V)"
# 1. CLEAN TREE: build the REGENERATOR from the committed tree (heal.yml:84-94 route).
set +e
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/rb.log 2>&1
echo "REGENERATOR-BUILD-EXIT=$?"
set -e
tail -3 /tmp/rb.log
sha256sum target/release/claim_executor
# 2. SEED EDITS ONLY (additive; exported 1-arg surface unchanged).
git apply --check .evidence/seed.patch && git apply .evidence/seed.patch && echo "SEED-APPLIED" || { echo "SEED-PATCH-REJECTED"; exit 0; }
git status --porcelain src/v1 | head -8
# 3. THE REGEN ROUND, from the cleanly-built regenerator.
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "REGEN-EXIT=$?"
set -e
grep -aE "required-regen:|refusal|rounds|first_generation|^error" /tmp/regen.out | tail -25
echo "ZZ-MIRROR-STATUS"; git status --porcelain src/v1/stage0 | head -20
echo "ZZ-MIRROR-DIFFSTAT"; git diff --stat src/v1/stage0 | tail -8
echo "ZZ-AUTHORITY-COUNT=$(grep -c render_span_location src/v1/stage0/src/v1_std_core.rs || true)"
echo "ZZ-EOF-FIX=$(grep -c last_code_token_end src/v1/stage0/src/v1_compiler_tokenize.rs || true)"
# 4. HOST EDITS on top of the regenerated mirrors.
git apply --check .evidence/host.patch && git apply .evidence/host.patch && echo "HOST-APPLIED" || { echo "HOST-PATCH-REJECTED"; exit 0; }
git status --porcelain src/v1/stage0 | head -10
# 5. BUILD and RUN THE CLAIM.
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse > /tmp/b2.log 2>&1
echo "BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/b2.log | head -25 || true
sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
FUNCS=$(grep -oE '^test fn ([a-z0-9_]+)' dag/test/claim/diagnostic_span_location_rendering_witness.dag | awk '{print $3}' | paste -sd, -)
echo "FUNCS=$FUNCS"
set +e
./target/release/claim_batch --wet --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/test/claim/diagnostic_span_location_rendering_witness.dag --functions "$FUNCS" > /tmp/claim.out 2>&1
echo "CLAIM-EXIT=$?"
set -e
echo "ZZ-CLAIM-ROWS"; grep -E "^PASS |^FAIL " /tmp/claim.out
echo "ZZ-CLAIM-SUMMARY"; tail -4 /tmp/claim.out
sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
# 6. SHIP THE REGENERATED MIRRORS BACK.
tar -C . -czf /tmp/mirrors.tgz $(git status --porcelain src/v1/stage0 | awk '{print $2}') 2>/dev/null || true
echo "ZZ-TGZ-SIZE=$(wc -c < /tmp/mirrors.tgz) SHA=$(sha256sum /tmp/mirrors.tgz | cut -c1-16)"
split -b 60000 /tmp/mirrors.tgz.b64 /tmp/mchunk. 2>/dev/null || true
base64 -w0 /tmp/mirrors.tgz > /tmp/mirrors.tgz.b64
split -b 60000 /tmp/mirrors.tgz.b64 /tmp/mchunk.
for f in /tmp/mchunk.*; do echo "ZZCHUNK-$f"; cat $f; echo; done
echo "ZZ-DONE"
