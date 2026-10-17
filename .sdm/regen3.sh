#!/usr/bin/env bash
# Regenerate the stage0 infer mirror for BOTH candidate .dag variants in ONE dispatch,
# using the seed's own fixed-point writer -- the exact command heal.yml:94 runs:
#   target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2
# The mirror is never hand-edited, and never hand-resolved during a merge; this is the
# only thing that may change src/v1/stage0/src/v1_compiler_infer.rs.
#
# WHY NO GIT HERE: earlier versions fetched into /root/workspace/repo-root, a repo another
# session owns; that hung for 15 minutes before the first echo. A control probe (plain
# echo + sleep) reached the log fine, proving dispatch output was not the problem -- so the
# tree is delivered by ctrl-build and touched by no git command.
#
# WHY ONE DISPATCH FOR BOTH VARIANTS: each dispatch pays a cold ~7 minute build. Building
# once and running the writer twice (once per .dag) gives both mirrors for one build.
set -u
echo "############ REGEN3 start $(date -u +%H:%M:%S)"
echo "cwd=$PWD head=$(git rev-parse HEAD 2>/dev/null || echo NO-GIT)"
DAG=src/v1/04_infer.dag
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs
for f in "$DAG" "$MIRROR" .sdm/dag_variants/keep/04_infer.dag .sdm/dag_variants/delete/04_infer.dag; do
  [ -f "$f" ] || { echo "MISSING $f"; exit 2; }
done
marks() {
  printf "gate=%s contains_lambda=%s stale=%s planned=%s recheck=%s still_missing_after=%s lines=%s sha=%s" \
    "$(grep -c 'fn call_needs_generic_rebinding_pass' "$1" || true)" \
    "$(grep -c 'fn expr_contains_lambda' "$1" || true)" \
    "$(grep -c 'fn typed_expr_is_stale' "$1" || true)" \
    "$(grep -c 'fn infer_planned_lambda_recheck' "$1" || true)" \
    "$(grep -c 'fn argument_needs_generic_recheck' "$1" || true)" \
    "$(grep -c 'still_missing_after' "$1" || true)" \
    "$(wc -l < "$1")" "$(sha256sum "$1" | cut -c1-16)"
}
echo "START dag    $(marks $DAG)"
echo "START mirror $(marks $MIRROR)"
unset CARGO_TARGET_DIR RUST_TARGET_PATH
echo "building $(date -u +%H:%M:%S)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/regenbuild.log 2>&1 &
bp=$!
while kill -0 $bp 2>/dev/null; do sleep 30; echo "hb-build $(wc -l < /tmp/regenbuild.log 2>/dev/null || echo 0) lines $(date -u +%H:%M:%S)"; done
wait $bp || { echo "BUILD-FAILED"; tail -40 /tmp/regenbuild.log; exit 11; }
echo "built $(date -u +%H:%M:%S)"
CE="target/release/claim_executor"
[ -x "$CE" ] || { echo "NO-CLAIM-EXECUTOR at $CE"; exit 13; }

gen() { # variant
  V="$1"
  cp ".sdm/dag_variants/$V/04_infer.dag" "$DAG"
  echo "--- variant $V dag $(marks $DAG)"
  set +e
  "$CE" --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen_$V.log 2>&1
  echo "regen_rc=$? for $V $(date -u +%H:%M:%S)"
  set -e
  grep -iE "changed_paths|convergence|refused" /tmp/regen_$V.log | tail -8
  echo "variant $V mirror $(marks $MIRROR)"
  cp "$MIRROR" ".sdm/gen_${V}_v1_compiler_infer.rs"
  echo "MIRROR-B64-BEGIN $V"
  gzip -c "$MIRROR" | base64 -w 100
  echo "MIRROR-B64-END $V"
}
gen keep
gen delete
# leave the tree on the keep variant, which is the PR-bound authority
cp .sdm/dag_variants/keep/04_infer.dag "$DAG"
echo "############ REGEN3 done $(date -u +%H:%M:%S)"
