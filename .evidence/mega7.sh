set -e
cd /root/workspace/repo-root
HEAD0=$(git rev-parse HEAD)
echo "HEAD-START=$HEAD0"
echo "TOOLCHAIN: $(rustc -V) / $(cargo -V)"
BR=$(git rev-parse --abbrev-ref HEAD)
echo "BRANCH=$BR"
for pf in .evidence/seed.patch .evidence/host.patch; do
  [ -s "$pf" ] || { echo "ABORT-EMPTY-PATCH $pf"; exit 0; }
done
md5sum .evidence/seed.patch .evidence/host.patch dag/test/claim/diagnostic_span_location_rendering_witness.dag
# 1. BACK TO THE PRE-EDIT STATE so the regenerator itself builds from committed bytes.
#    ORDER MATTERS: `git checkout -- .` FIRST (it restores everything to HEAD, which CARRIES the
#    edits), and only THEN the reverse-applies. Running checkout after them silently undoes the
#    revert -- measured: 8 x E0425 with the seed authority present and the mirrors still stale.
git checkout -- . 2>/dev/null || true
git apply -R .evidence/host.patch
git apply -R .evidence/seed.patch
echo "PRE-EDIT-VERIFY seed-authority=$(grep -c 'fn render_span_location' src/v1/00_core.dag || true) host-authority-call=$(grep -c 'crate::v1_std_core::render_span_location' src/v1/stage0/src/module_path_index/index.rs || true) host-located-name=$(grep -c interpreter_blocking_diagnostic_messages_with_source_indices src/v1/stage0/src/cli_run/emit_host.rs || true) eof-fix=$(grep -c last_code_token_end src/v1/01_tokenize.dag || true)"
echo "CLEAN-STATUS:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' '|')]"
# 2. CLEAN TREE: build the REGENERATOR from the committed (pre-edit) bytes -- heal.yml:84-94 route.
set +e
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/rb.log 2>&1
echo "REGENERATOR-BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/rb.log | head -10 || true
sha256sum target/release/claim_executor target/release/gunbc
# 3. SEED EDITS ONLY (additive; exported 1-arg surface unchanged, so the crate still builds).
git apply --check .evidence/seed.patch && git apply .evidence/seed.patch && echo "SEED-APPLIED" || { echo "SEED-PATCH-REJECTED"; exit 0; }
# 4. THE REGEN ROUND from the cleanly-built regenerator.
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "REGEN-EXIT=$?"
set -e
grep -aE "required-regen:|rounds|first_generation|^error" /tmp/regen.out | tail -20
echo "ZZ-MIRROR-DIFFSTAT"; git diff --stat src/v1/stage0 | tail -8
echo "ZZ-AUTHORITY-COUNT=$(grep -c render_span_location src/v1/stage0/src/v1_std_core.rs || true)"
echo "ZZ-EOF-FIX=$(grep -c last_code_token_end src/v1/stage0/src/v1_compiler_tokenize.rs || true)"
echo "ZZ-LOCATED-NAME=$(grep -c interpreter_blocking_diagnostic_messages_with_source_indices src/v1/stage0/src/v1_compiler_compile.rs || true)"
# 5. HOST EDITS on top of the regenerated mirrors.
git apply --check .evidence/host.patch && git apply .evidence/host.patch && echo "HOST-APPLIED" || { echo "HOST-PATCH-REJECTED"; exit 0; }
# 6. BUILD; print binary sha256 BEFORE the run.
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse > /tmp/b7.log 2>&1
echo "BUILD-EXIT=$?"
set -e
grep -aE "^error|^\s+-->" /tmp/b7.log | head -30 || true
echo "ZZ-BIN-SHA-BEFORE"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
# 7. THE CLAIM. Raw lines, unannotated.
FUNCS=$(grep -oE '^test fn ([a-z0-9_]+)' dag/test/claim/diagnostic_span_location_rendering_witness.dag | awk '{print $3}' | paste -sd, -)
set +e
./target/release/claim_batch --wet --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/test/claim/diagnostic_span_location_rendering_witness.dag --functions "$FUNCS" > /tmp/claim.out 2>&1
echo "CLAIM-EXIT=$?"
set -e
echo "ZZ-CLAIM-ROWS"; grep -E "^PASS |^FAIL " /tmp/claim.out
echo "ZZ-CLAIM-SUMMARY"; tail -4 /tmp/claim.out
echo "ZZ-BIN-SHA-AFTER"; sha256sum target/release/gunbc target/release/claim_batch target/release/v1_src_dag_parse
# 8. SHIP EVERYTHING BACK AS A COMMIT ON A SCRATCH REF (log-safe; no base64 truncation).
echo "ZZ-MIRROR-SHA256"; sha256sum src/v1/stage0/src/v1_std_core.rs src/v1/stage0/src/v1_compiler_tokenize.rs src/v1/stage0/src/v1_compiler_parse.rs src/v1/stage0/src/v1_compiler_compile.rs src/v1/stage0/src/v1_compiler_emit_rust.rs 2>/dev/null
git add -A src/v1
git -c user.name=gunbc-agent -c user.email=agent@gunbc.local commit -q -m "diagnostics: one seed authority renders a span's location

Seed authority + EOF-span fix + host renderers calling the emitted authority,
with the stage0 mirrors written by the regenerator
(claim_executor --regen-round-cost --source-root dag --source-root src/v2)." && echo "COMMITTED=$(git rev-parse HEAD)"
SCRATCH="session/bright-swift-151-mirrors"
git push -f -q origin "HEAD:refs/heads/$SCRATCH" && echo "PUSHED=$SCRATCH" || echo "PUSH-FAILED"
echo "HEAD-END=$(git rev-parse HEAD)"
[ "$HEAD0" = "$(git rev-parse HEAD)" ] && echo "HEAD-STABLE=yes" || echo "HEAD-STABLE=pre-edit-restore-applied"
echo "ZZ-DONE"
