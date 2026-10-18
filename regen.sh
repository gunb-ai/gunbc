set -e
cd "$(git rev-parse --show-toplevel)"
cargo build --release --bin claim_executor
set +e
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
echo "regen rc=$?"
tail -5 /tmp/regen.out
C=target/stage0-regen-candidate/src
test -d $C || { echo NOCANDIDATE; exit 3; }
echo ==CHANGED
(cd $C && for f in $(find . -type f); do cmp -s $f ../../../src/v1/stage0/src/$f 2>/dev/null || echo $f; done) | head -50
echo ==TAR
(cd $C && for f in $(find . -type f); do cmp -s $f ../../../src/v1/stage0/src/$f 2>/dev/null || echo $f; done) | tar -C $C -czf - -T - | base64 -w0
echo
echo ==ENDTAR
