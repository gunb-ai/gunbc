set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
echo "DIRTY-COUNT=$(git status --porcelain | grep -vc '^?? \.evidence' || true)"
echo "STATE seed-authority=$(grep -c 'fn render_span_location' src/v1/00_core.dag || true) mirror-authority=$(grep -c 'pub fn render_span_location' src/v1/stage0/src/v1_std_core.rs || true) host-call=$(grep -c 'crate::v1_std_core::render_span_location' src/v1/stage0/src/module_path_index/index.rs || true)"
set +e
cargo build --release -p v1-compiler --lib > /tmp/e.log 2>&1
echo "LIB-BUILD-EXIT=$?"
set -e
grep -a -A 14 "E0308" /tmp/e.log | head -80
echo "=== the authority's emitted signature, verbatim:"
sed -n '5185,5196p' src/v1/stage0/src/v1_std_core.rs
echo "ZZ-DONE"
