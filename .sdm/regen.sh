set -e
# The stage0 mirror is produced by the seed's own fixed-point writer -- the exact
# command heal.yml runs ("Converge the stage0 mirrors to the seed's own fixed point"),
# `target/release/claim_executor --regen-round-cost --source-root dag --source-root
# src/v2`. The mirror is NEVER hand-edited and never resolved by hand during a merge.
# This dispatch only regenerates; the bytes come back base64+gzip so they can be
# committed verbatim, and the printed sha256 is what the A/B asserts on.
R=/root/workspace/repo-root
SHA="$1"; TARGET="$2"
# DO NOT set CARGO_TARGET_DIR here. heal.yml:94 runs the producer as
# `target/release/claim_executor --regen-round-cost ...`, and the fixed-point writer
# re-reads that executable by relative path during its seed-build round. With a shared
# target dir the round fails with
#   StageOutputExecutableUnbound: read .../target/release/claim_executor: No such file
# and the whole regen REFUSES (rc=1) while still printing phase ticks -- easy to mistake
# for a no-op. The runner is stateless per dispatch, so an in-tree target costs nothing
# extra. (The A/B dispatch DOES use a shared target dir, because it reads binaries by
# absolute path and reuses dependency compilation across variants.)
D=/root/regen
git -C $R worktree prune >/dev/null 2>&1 || true
git -C $R fetch --quiet --force origin "$SHA:refs/sd503/regen" || { echo "FETCH-FAILED"; exit 6; }
[ "$(git -C $R rev-parse refs/sd503/regen|head -1)" = "$SHA" ] || { echo "SHA-MISMATCH"; exit 6; }
rm -rf $D; git -C $R worktree add --detach $D "$SHA" >/dev/null 2>&1
cd $D
echo "regen HEAD=$(git rev-parse HEAD)"
echo "BEFORE sha256=$(sha256sum $TARGET | cut -d' ' -f1) lines=$(wc -l < $TARGET)"
# The previous regen dispatch died with no output for 20 minutes and no completion line,
# which is indistinguishable from a long quiet build. Heartbeat so "stuck" and "working"
# are told apart from the log alone.
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/regenbuild.log 2>&1 &
bp=$!
while kill -0 $bp 2>/dev/null; do sleep 30; echo "hb-build $(wc -l < /tmp/regenbuild.log 2>/dev/null || echo 0) lines $(date -u +%H:%M:%S)"; done
wait $bp || { echo "BUILD-FAILED"; tail -40 /tmp/regenbuild.log; exit 11; }
echo "built $(date -u +%H:%M:%S)"
CE="target/release/claim_executor"
# With a shared CARGO_TARGET_DIR the binaries are NOT under ./target (that path gave
# rc=127 -- a "command not found" that looked like a regen failure). Assert it exists.
if [ ! -x "$CE" ]; then echo "NO-CLAIM-EXECUTOR at $CE"; ls -la target/release 2>/dev/null | head -20; exit 13; fi
set +e
"$CE" --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.log 2>&1
echo "regen_rc=$?"
set -e
echo "regen log tail:"; tail -25 /tmp/regen.log
grep -iE "changed_paths|convergence|installs|round" /tmp/regen.log | tail -20
echo "AFTER sha256=$(sha256sum $TARGET | cut -d' ' -f1) lines=$(wc -l < $TARGET)"
git status --porcelain | sed 's/^/CHANGED /'
echo "MARKERS gate=$(grep -c 'fn call_needs_generic_rebinding_pass' $TARGET || true) recheck=$(grep -c argument_needs_generic_recheck $TARGET || true) stale=$(grep -c typed_expr_is_stale $TARGET || true) lambda=$(grep -c expr_contains_lambda $TARGET || true)"
echo "B64-BEGIN"
gzip -c $TARGET | base64 -w 100
echo "B64-END"
