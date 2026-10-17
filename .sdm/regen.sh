set -e
# The stage0 mirror is produced by the seed's own fixed-point writer -- the exact
# command heal.yml runs ("Converge the stage0 mirrors to the seed's own fixed point"),
# `target/release/claim_executor --regen-round-cost --source-root dag --source-root
# src/v2`. The mirror is NEVER hand-edited and never resolved by hand during a merge.
# This dispatch only regenerates; the bytes come back base64+gzip so they can be
# committed verbatim, and the printed sha256 is what the A/B asserts on.
R=/root/workspace/repo-root
SHA="$1"; TARGET="$2"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/root/sd503_target}
D=/root/regen
git -C $R worktree prune >/dev/null 2>&1 || true
git -C $R fetch --quiet --force origin "$SHA:refs/sd503/regen" || { echo "FETCH-FAILED"; exit 6; }
[ "$(git -C $R rev-parse refs/sd503/regen|head -1)" = "$SHA" ] || { echo "SHA-MISMATCH"; exit 6; }
rm -rf $D; git -C $R worktree add --detach $D "$SHA" >/dev/null 2>&1
cd $D
echo "regen HEAD=$(git rev-parse HEAD)"
echo "BEFORE sha256=$(sha256sum $TARGET | cut -d' ' -f1) lines=$(wc -l < $TARGET)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/regenbuild.log 2>&1 || { echo "BUILD-FAILED"; tail -30 /tmp/regenbuild.log; exit 11; }
echo "built $(date -u +%H:%M:%S)"
set +e
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.log 2>&1
echo "regen_rc=$?"
set -e
grep -iE "changed_paths|convergence|installs|round" /tmp/regen.log | tail -20
echo "AFTER sha256=$(sha256sum $TARGET | cut -d' ' -f1) lines=$(wc -l < $TARGET)"
git status --porcelain | sed 's/^/CHANGED /'
echo "MARKERS gate=$(grep -c 'fn call_needs_generic_rebinding_pass' $TARGET || true) recheck=$(grep -c argument_needs_generic_recheck $TARGET || true) stale=$(grep -c typed_expr_is_stale $TARGET || true) lambda=$(grep -c expr_contains_lambda $TARGET || true)"
echo "B64-BEGIN"
gzip -c $TARGET | base64 -w 100
echo "B64-END"
