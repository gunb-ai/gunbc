#!/usr/bin/env bash
# Regenerate ONE generated mirror using the seed's own fixed-point writer -- the exact
# command heal.yml:94 runs:
#   target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2
# The mirror is never hand-edited and never hand-resolved during a merge.
#
# WHY THIS SCRIPT TOUCHES NO GIT:
# The previous version fetched into /root/workspace/repo-root, which is ANOTHER SESSION'S
# repo shared with us; it hung there for 15 minutes before its first echo (proved by a
# control probe in which plain `echo`+`sleep` reached the log fine, so dispatch output was
# not the problem). ctrl-build already delivers THIS branch's working tree into the action
# cwd, so the tree is simply what is already here: the launcher checks the target commit
# out locally and dispatches from it. No remote git, no shared repo, no locks.
set -u
TARGET="${1:?usage: regen2.sh <target-file>}"
echo "############ REGEN2 start $(date -u +%H:%M:%S) target=$TARGET"
echo "cwd=$PWD"
echo "head=$(git rev-parse HEAD 2>/dev/null || echo NO-GIT)"
[ -f "$TARGET" ] || { echo "MISSING-TARGET $TARGET"; exit 2; }
[ -f src/v1/04_infer.dag ] || { echo "NOT-A-CHECKOUT (no src/v1/04_infer.dag)"; exit 3; }
# State proof by CONTENT, not by git: which decisions the .dag authority really holds, and
# what the mirror currently holds. A mismatch here is exactly what this dispatch exists to
# clear, so both sides are printed before and after.
dag_marks() { # file
  printf "gate=%s contains_lambda=%s recheck=%s stale=%s planned=%s still_missing_after=%s" \
    "$(grep -c 'fn call_needs_generic_rebinding_pass' "$1" || true)" \
    "$(grep -c 'fn expr_contains_lambda' "$1" || true)" \
    "$(grep -c 'fn argument_needs_generic_recheck' "$1" || true)" \
    "$(grep -c 'fn typed_expr_is_stale' "$1" || true)" \
    "$(grep -c 'fn infer_planned_lambda_recheck' "$1" || true)" \
    "$(grep -c 'still_missing_after' "$1" || true)"
}
echo "BEFORE dag    marks: $(dag_marks src/v1/04_infer.dag)"
echo "BEFORE mirror marks: $(dag_marks "$TARGET")"
echo "BEFORE mirror sha256=$(sha256sum "$TARGET" | cut -d' ' -f1) lines=$(wc -l < "$TARGET")"
# NO CARGO_TARGET_DIR here: the writer re-executes the relative path target/release/
# claim_executor mid-round, and a shared target dir makes that round fail with
# StageOutputExecutableUnbound, which REFUSES the whole regen (rc=1) while still printing
# phase ticks -- easy to mistake for a successful no-op.
unset CARGO_TARGET_DIR RUST_TARGET_PATH
echo "building $(date -u +%H:%M:%S)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/regenbuild.log 2>&1 &
bp=$!
while kill -0 $bp 2>/dev/null; do sleep 30; echo "hb-build $(wc -l < /tmp/regenbuild.log 2>/dev/null || echo 0) lines $(date -u +%H:%M:%S)"; done
wait $bp || { echo "BUILD-FAILED"; tail -40 /tmp/regenbuild.log; exit 11; }
echo "built $(date -u +%H:%M:%S)"
CE="target/release/claim_executor"
[ -x "$CE" ] || { echo "NO-CLAIM-EXECUTOR at $CE"; ls -la target/release 2>/dev/null | head -20; exit 13; }
set +e
"$CE" --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.log 2>&1
rc=$?
set -e
echo "regen_rc=$rc $(date -u +%H:%M:%S)"
grep -iE "changed_paths|convergence|refused|round|installs" /tmp/regen.log | tail -20
echo "regen log tail:"; tail -20 /tmp/regen.log
echo "AFTER mirror marks: $(dag_marks "$TARGET")"
echo "AFTER mirror sha256=$(sha256sum "$TARGET" | cut -d' ' -f1) lines=$(wc -l < "$TARGET")"
git status --porcelain 2>/dev/null | sed 's/^/CHANGED /' | head -10
echo "B64-BEGIN"
gzip -c "$TARGET" | base64 -w 100
echo "B64-END"
echo "############ REGEN2 done $(date -u +%H:%M:%S) rc=$rc"
exit $rc
