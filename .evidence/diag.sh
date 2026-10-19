set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
echo "STATE:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' '|')]"
# ensure the full edit set is present (idempotent: skip if already applied)
if ! grep -q "render_span_location" src/v1/00_core.dag; then git checkout -- . 2>/dev/null||true; git apply .evidence/seed.patch; fi
if ! grep -q "crate::v1_std_core::render_span_location" src/v1/stage0/src/module_path_index/index.rs; then git apply .evidence/host.patch || { echo HOST-APPLY-FAILED; git apply --check -v .evidence/host.patch 2>&1 | head -20; exit 0; }; fi
echo "APPLIED-STATE:[$(git status --porcelain | grep -v '^?? \.evidence' | tr '\n' '|')]"
echo "===== EMITTED AUTHORITY SIGNATURE (from the regenerated mirror)"
grep -n "pub fn render_span_location" -A 4 src/v1/stage0/src/v1_std_core.rs | head -12
grep -n "pub fn located_span_of" -A 4 src/v1/stage0/src/v1_std_core.rs | head -10
grep -n "pub struct SourceSpan" -A 6 src/v1/stage0/src/v1_std_core.rs src/v1/stage0/src/std_types.rs 2>/dev/null | head -14
grep -n "pub enum SpanLocation" -A 8 src/v1/stage0/src/v1_std_core.rs | head -14
echo "===== FULL BUILD ERRORS"
set +e
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse > /tmp/b3.log 2>&1
echo "BUILD-EXIT=$?"
set -e
grep -aE "^error|^\s+-->|^  expected|^  found|note: |^error\[" /tmp/b3.log | head -60
echo "ZZ-DONE"
