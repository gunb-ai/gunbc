set -u
export CARGO_TARGET_DIR=$PWD/target
git add -A >/dev/null 2>&1
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -1
for i in 1 2; do
  echo "== round $i"
  ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 2>&1 | tail -5
done
git add -A >/dev/null 2>&1
git diff --cached --stat -- src/v1/stage0 | tail -5
git diff --cached --binary -- src/v1/stage0 > /tmp/regen.patch
echo "== PATCH-B64-BEGIN"; gzip -c /tmp/regen.patch | base64 -w0; echo; echo "== PATCH-B64-END"
bash .nc.sh
