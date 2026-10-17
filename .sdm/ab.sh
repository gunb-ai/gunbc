set -e
# ONE dispatch, ONE runner, ONE toolchain, ONE env, ONE shared cargo target:
#   control    candidate tree with the #14075 04_infer.dag change reverted to main's
#              old second-pass algorithm (.dag AND its paired mirror, both from the
#              merge-base, so the generated file is still its authority's regen output)
#   treatment  the candidate exactly as committed
#   control2   control again, last, so a drift between the two control runs is visible
# GUNBC_NATIVE_PRODUCT_REUSE=fresh as parent required.
R=/root/workspace/repo-root
CAND_SHA="$1"; BASE_SHA="$2"; HARNESS="${3:-sd503-instrument}"; NONCE="${4:-0}"
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
export CARGO_TARGET_DIR=/root/sd503_target
A=/root/ab
echo "############ AB nonce=$NONCE cand=$CAND_SHA base=$BASE_SHA $(date -u +%H:%M:%S)"

# ---- trees, each verified by hash -------------------------------------------------
git -C $R worktree prune >/dev/null 2>&1 || true
mkdir -p $A
mkwt() { # dir sha
  D="$1"; SHA="$2"
  git -C $R fetch --quiet --force origin "$SHA:refs/sd503/wt" || { echo "FETCH-FAILED $SHA"; exit 6; }
  [ "$(git -C $R rev-parse refs/sd503/wt | head -1)" = "$SHA" ] || { echo "SHA-MISMATCH want=$SHA"; exit 6; }
  rm -rf "$D"; git -C $R worktree add --detach "$D" "$SHA" || { echo "WORKTREE-FAILED $D"; exit 7; }
  echo "$(basename $D) HEAD=$(git -C $D rev-parse HEAD)"
}
mkwt $A/treat "$CAND_SHA"
mkwt $A/ctrl  "$CAND_SHA"
# control: revert ONLY the #14075 infer change, keeping the rest of the composed tree.
cd $A/ctrl
git -C $R cat-file -p "$BASE_SHA:src/v1/04_infer.dag" > src/v1/04_infer.dag || { echo "NO-BASE-DAG"; exit 6; }
git -C $R cat-file -p "$BASE_SHA:src/v1/stage0/src/v1_compiler_infer.rs" > src/v1/stage0/src/v1_compiler_infer.rs || { echo "NO-BASE-MIRROR"; exit 6; }
echo "control authority check: gate=$(grep -c 'fn call_needs_generic_rebinding_pass' src/v1/04_infer.dag) recheck=$(grep -c argument_needs_generic_recheck src/v1/04_infer.dag) (want 1 / 0)"
[ "$(grep -c argument_needs_generic_recheck src/v1/04_infer.dag)" = "0" ] || { echo "CONTROL-NOT-OLD-ALGORITHM"; exit 9; }
# the enrolled witness belongs to the treatment; control keeps the file so the tree
# composes, but a control witness failure is reported, not fatal.
cp -r $A/treat/.sdm $A/ctrl/.sdm 2>/dev/null || true

# ---- harness ---------------------------------------------------------------------
git -C $R fetch --quiet --force origin "$HARNESS:refs/sd503/harness" || true
HSHA=$(git -C $R rev-parse refs/sd503/harness 2>/dev/null | head -1)
[ -n "$HSHA" ] || { echo "HARNESS-MISSING"; exit 8; }
rm -rf $A/harness; git -C $R worktree add --detach $A/harness "$HSHA" >/dev/null 2>&1
for D in $A/ctrl $A/treat; do
  rm -rf $D/.sdm; cp -r $A/harness/.sdm $D/.sdm
  cp $A/harness/.sdm/fixtures/gap_depth_*.dag $D/dag/test/ 2>/dev/null || true
  cp $A/harness/.sdm/fixtures/garr_*.dag $D/dag/test/ 2>/dev/null || true
done
cp $A/treat/.sdm/witness.dag $A/ctrl/.sdm/witness.dag 2>/dev/null || true
echo "harness=$HSHA"

# ---- build + measure --------------------------------------------------------------
build() { # dir label
  D="$1"; V="$2"; cd $D
  ( cargo build --release -p v1-compiler --bin gunbc > /tmp/build_$V.log 2>&1; echo "BUILD_RC=$?" >> /tmp/build_$V.log ) &
  bp=$!
  while kill -0 $bp 2>/dev/null; do sleep 20; echo "hb $V $(wc -l < /tmp/build_$V.log 2>/dev/null || echo 0) $(date -u +%H:%M:%S)"; done
  wait $bp || true
  grep -E "^BUILD_RC=" /tmp/build_$V.log
  grep -q "^BUILD_RC=0$" /tmp/build_$V.log || { echo "BUILD-FAILED $V"; tail -25 /tmp/build_$V.log; exit 11; }
  echo "built $V $(date -u +%H:%M:%S) target=$(du -sh $CARGO_TARGET_DIR 2>/dev/null | cut -f1)"
}
run_variant() { # dir label hooksidew fatal
  D="$1"; V="$2"; H="$3"; FATAL="${4:-yes}"
  echo "@@@@ VARIANT=$V start $(date -u +%H:%M:%S)"
  cd $D
  if ! python3 .sdm/hooks.py "$H" .sdm/gap_block.rs src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_compile.rs; then
    echo "HOOK-FAILED $V"; [ "$FATAL" = yes ] && exit 10
  fi
  build "$D" "$V"
  bash .sdm/measure_variant.sh "$V" || echo "MEASURE-REPORTED-FAILURE $V"
  echo "@@@@ VARIANT=$V end $(date -u +%H:%M:%S)"
}
run_variant $A/ctrl  control    control   yes
run_variant $A/treat treatment  treatment yes
run_variant $A/ctrl  control2   control   yes
echo "AB_DONE $(date -u +%H:%M:%S)"
