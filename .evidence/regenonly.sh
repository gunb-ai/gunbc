set -e
cd /root/workspace/repo-root
BASE=65b25985167c5abbaf5a6df2b26218fe618a13f3
echo "HEAD-START=$(git rev-parse HEAD)"
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo "HEAD-MISMATCH -- refusing"; exit 0; }
for pf in .evidence/seed.patch .evidence/host.patch; do
  [ -s "$pf" ] || { echo "ABORT-EMPTY-PATCH $pf"; exit 0; }
done
# pre-edit state, then the cleanly-built regenerator (heal.yml:84-94 route).
git checkout -- . 2>/dev/null || true
git apply -R .evidence/host.patch
git apply -R .evidence/seed.patch
echo "PRE-EDIT seed-authority=$(grep -c 'fn render_span_location' src/v1/00_core.dag || true) eof-fix=$(grep -c last_code_token_end src/v1/01_tokenize.dag || true)"
set +e
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc > /tmp/ro.log 2>&1
echo "REGENERATOR-BUILD-EXIT=$?"
set -e
grep -aE "^error" /tmp/ro.log | head -8 || true
sha256sum target/release/claim_executor
git apply .evidence/seed.patch && echo "SEED-APPLIED"
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "REGEN-EXIT=$?"
set -e
grep -aE "required-regen: elapsed|declared_divergent|refusal" /tmp/regen.out | tail -6
MIRRORS="src/v1/stage0/src/v1_std_core.rs src/v1/stage0/src/v1_compiler_tokenize.rs src/v1/stage0/src/v1_compiler_parse.rs src/v1/stage0/src/v1_compiler_compile.rs src/v1/stage0/src/v1_compiler_emit_rust.rs"
echo "ZZ-MIRROR-DIFFSTAT"; git diff --stat $MIRRORS | tail -8
echo "ZZ-MIRROR-SHA256"; sha256sum $MIRRORS
git diff $BASE -- $MIRRORS > /tmp/mirrors.diff
echo "ZZ-DIFF-BYTES=$(wc -c < /tmp/mirrors.diff) DIFF-SHA=$(sha256sum /tmp/mirrors.diff | cut -c1-64)"
gzip -9 -c /tmp/mirrors.diff > /tmp/mirrors.diff.gz
base64 -w0 /tmp/mirrors.diff.gz > /tmp/mirrors.b64
echo "ZZ-B64-BYTES=$(wc -c < /tmp/mirrors.b64)"
rm -f /tmp/mchunk.*; split -b 2000 /tmp/mirrors.b64 /tmp/mchunk.
N=0
for f in /tmp/mchunk.*; do echo "ZZCHUNK-START $N"; cat $f; echo; echo "ZZCHUNK-END $N"; N=$((N+1)); done
echo "ZZ-CHUNKS=$N"
echo "ZZ-DONE"
