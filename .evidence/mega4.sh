set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
[ "$(git rev-parse HEAD)" = "c0c4b50add210f38e6253c03bf73dacafcfe7ab1" ] || { echo "HEAD-MISMATCH-ABORT"; exit 0; }
git checkout -- . 2>/dev/null || true
echo "CLEAN-STATUS:[$(git status --porcelain | grep -v '^?? \.evidence' | grep -v diagnostic_span | tr '\n' '|')]"
md5sum .evidence/seed.patch .evidence/host.patch .evidence/claim.dag
echo "TOOLCHAIN: $(rustc -V) / $(cargo -V)"
# 1. CLEAN TREE: build the REGENERATOR from the committed tree (the heal route's own command).
set +e
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/rb.log 2>&1
echo "REGENERATOR-BUILD-EXIT=$?"
set -e
sha256sum target/release/claim_executor
# 2. SEED EDITS ONLY (additive: the exported 1-arg surface is unchanged, so the crate still builds).
git apply .evidence/seed.patch && echo "SEED-APPLIED"
# 3. THE REGEN ROUND from the cleanly-built regenerator.
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "REGEN-EXIT=$?"
set -e
grep -aE "required-regen:|rounds|first_generation|^error" /tmp/regen.out | tail -20
echo "ZZ-MIRROR-STATUS"; git status --porcelain src/v1/stage0 | head -20
echo "ZZ-MIRROR-DIFFSTAT"; git diff --stat src/v1/stage0 | tail -8
echo "ZZ-AUTHORITY-COUNT=$(grep -c render_span_location src/v1/stage0/src/v1_std_core.rs || true)"
echo "ZZ-EOF-FIX=$(grep -c last_code_token_end src/v1/stage0/src/v1_compiler_tokenize.rs || true)"
# 4. HOST EDITS on top of the regenerated mirrors.
git apply .evidence/host.patch && echo "HOST-APPLIED"
git status --porcelain src/v1/stage0 | head -10
# 5. BUILD the bins, print their sha256 BEFORE the run.
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse --bin bootstrap_witness > /tmp/b4.log 2>&1
echo "BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/b4.log | head -25 || true
echo "ZZ-BIN-SHA-BEFORE"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse target/release/bootstrap_witness
# 6. THE CLAIM (the GREEN attempt). Raw lines, unannotated.
FUNCS=$(grep -oE '^test fn ([a-z0-9_]+)' dag/test/claim/diagnostic_span_location_rendering_witness.dag | awk '{print $3}' | paste -sd, -)
echo "FUNCS=$FUNCS"
set +e
./target/release/claim_batch --wet --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/test/claim/diagnostic_span_location_rendering_witness.dag --functions "$FUNCS" > /tmp/claim.out 2>&1
echo "CLAIM-EXIT=$?"
set -e
echo "ZZ-CLAIM-ROWS"; grep -E "^PASS |^FAIL " /tmp/claim.out
echo "ZZ-CLAIM-SUMMARY"; tail -4 /tmp/claim.out
echo "ZZ-BIN-SHA-AFTER"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse target/release/bootstrap_witness
# 7. THE BOOTSTRAT: seed cargo check + the v2-bootstrap emitted closure compile (the repo's own instrument).
set +e
./target/release/bootstrap_witness --suite expensive > /tmp/boot.log 2>&1
echo "BOOTSTRAP-EXIT=$?"
set -e
grep -aE "PASS|FAIL|error\[|stage1 cargo check|errors \(ratchet" /tmp/boot.log | tail -25
# 8. SHIP THE REGENERATED MIRRORS BACK, chunked.
tar -C . -czf /tmp/mirrors.tgz $(git status --porcelain src/v1/stage0 | awk '{print $2}') 2>/dev/null || true
base64 -w0 /tmp/mirrors.tgz > /tmp/mirrors.b64
echo "ZZ-TGZ-SIZE=$(wc -c < /tmp/mirrors.tgz) SHA=$(sha256sum /tmp/mirrors.tgz | cut -c1-16)"
split -b 3000 /tmp/mirrors.b64 /tmp/mchunk.
for f in /tmp/mchunk.*; do cat $f; echo; done
echo "ZZ-DONE"
