set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/g14-root
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 40 origin main || { echo CLONE_FAIL; exit 1; }
for SHA in 7876d9e2e8880000000000000000000000000000; do :; done
for SPEC in "7876d9e2e888^" "7876d9e2e888" "77f138fe2722^" "77f138fe2722"; do
  SHA=$(git rev-parse "$SPEC" 2>/dev/null) || { echo "PARSE_FAIL $SPEC"; continue; }
  git checkout -q $SHA || { echo "CHECKOUT_FAIL $SHA"; continue; }
  echo "TREE_OK $SHA ($SPEC) t=$(date +%H:%M:%S)"
  timeout 900 cargo check -q -p v1-compiler --release --bins 2>&1 | grep -aE '^error|error\[' | head -8
  echo "CHECK_RC=$? ($SPEC)"
done
echo "ALL_DONE_G14 $(date +%H:%M:%S)"
