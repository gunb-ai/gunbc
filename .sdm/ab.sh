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
  # Fixtures must exist in EXACTLY ONE place under the source root. Source discovery
  # enumerates every .dag under --source-root and refuses the whole compile when a module
  # path is declared by two files -- which is what voided the previous dispatch (the
  # witness was copied to .sdm/witness.dag while the tracked copy stayed in place). So the
  # fixtures land in dag/test/ only, and the .sdm copy is deleted rather than left to be
  # discovered a second time.
  cp $A/harness/.sdm/fixtures/gap_depth_*.dag $D/dag/test/
  cp $A/harness/.sdm/fixtures/garr_*.dag $D/dag/test/
  rm -rf $D/.sdm/fixtures
  echo "fixtures in $(basename $D): $(ls $D/dag/test/gap_depth_*.dag $D/dag/test/garr_*.dag 2>/dev/null | wc -l) files, .sdm/fixtures present=$([ -d $D/.sdm/fixtures ] && echo YES-BUG || echo no)"
done
# The witness is SELF-CONTAINED: its specimens are inline `data ... : String` sources
# fed to total_blocking_count_for / blocking_class_count_for, so there are no separate
# subject fixtures to fetch. Materialising a second copy was a real bug: source-discovery
# then sees `module test.claim.generic_argument_recheck_witness` declared by both
# dag/test/claim/... and .sdm/witness.dag and refuses the WHOLE compile, voiding every
# measurement. The file is already tracked in both trees (control keeps the treatment's
# witness so both trees compose); we only assert blob identity against the candidate.
for D in $A/ctrl $A/treat; do
  W=dag/test/claim/generic_argument_recheck_witness.dag
  rm -f $D/.sdm/witness.dag
  WANT=$(git -C $R rev-parse "$CAND_SHA:$W" 2>/dev/null || echo NOBLOB)
  GOT=$(git -C $D rev-parse "HEAD:$W" 2>/dev/null || echo MISSING)
  if [ "$WANT" = "$GOT" ]; then echo "witness identity OK $(basename $D) blob=$GOT"
  else echo "WITNESS-IDENTITY-MISMATCH $(basename $D) want=$WANT got=$GOT"; fi
done
echo "harness=$HSHA"

# ---- build + measure --------------------------------------------------------------
build() { # dir label
  D="$1"; V="$2"; cd $D
  # A bare grep under `set -e` aborted the whole action when the build produced no
  # BUILD_RC line -- which is exactly what happens when the compiler is SIGKILLed
  # (OOM). So: cap the job count, report the memory envelope, and treat a missing
  # BUILD_RC as a named failure instead of an unexplained exit 1.
  ( cargo build --release -j 6 -p v1-compiler --bin gunbc --bin claim_batch > /tmp/build_$V.log 2>&1; echo "BUILD_RC=$?" >> /tmp/build_$V.log ) &
  bp=$!
  while kill -0 $bp 2>/dev/null; do
    sleep 25
    echo "hb $V $(wc -l < /tmp/build_$V.log 2>/dev/null || echo 0) lines $(free -m | awk 'NR==2{print "avail="$7"MB"}') $(date -u +%H:%M:%S)"
  done
  wait $bp || true
  echo "build log tail for $V:"
  tail -6 /tmp/build_$V.log || echo "(no build log)"
  if ! grep -q "^BUILD_RC=0$" /tmp/build_$V.log 2>/dev/null; then
    echo "BUILD-FAILED $V rc_line=$(grep -c '^BUILD_RC=' /tmp/build_$V.log 2>/dev/null || echo none)"
    echo "mem: $(free -m | awk 'NR==2{print "total="$2"MB avail="$7"MB"}')"
    echo "disk: $(df -h /root | tail -1)"
    echo "ulimit: $(ulimit -a | tr '\n' ';')"
    echo "oom-kernel: $(dmesg 2>/dev/null | grep -iE "killed process|out of memory" | tail -5 || echo none-or-no-permission)"
    echo "log-signal-lines: $(grep -inE "killed|signal|no space|error\[" /tmp/build_$V.log 2>/dev/null | tail -12 || echo none)"
    echo "log-size: $(wc -c < /tmp/build_$V.log 2>/dev/null) bytes, last 40:"
    tail -40 /tmp/build_$V.log || true
    return 1
  fi
  local B="$CARGO_TARGET_DIR/release/gunbc"
  if [ ! -x "$B" ]; then echo "BUILD-OK-NO-BINARY $V at $B"; return 1; fi
  echo "built $V $(date -u +%H:%M:%S) target=$(du -sh $CARGO_TARGET_DIR 2>/dev/null | cut -f1) bin_sha256=$(sha256sum "$B" | cut -d' ' -f1) bin_bytes=$(stat -c %s "$B")"
}
run_variant() { # dir label hooksidew fatal skiphook
  D="$1"; V="$2"; H="$3"; FATAL="${4:-yes}"; SKIP="${5:-no}"
  echo "@@@@ VARIANT=$V start $(date -u +%H:%M:%S)"
  cd $D
  # control2 deliberately re-measures the tree control already left behind: the hook
  # ran (and post-verified, from disk) earlier in this same dispatch, so re-applying
  # it is a no-op that the fail-closed hook correctly refuses. Skipping the re-apply
  # is not skipping the check -- the instrument sha is printed instead.
  if [ "$SKIP" = yes ]; then
    echo "instrument sha256=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -d' ' -f1)"
  elif ! python3 .sdm/hooks.py "$H" .sdm/gap_block.rs src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_compile.rs; then
    echo "HOOK-FAILED $V"; [ "$FATAL" = yes ] && exit 10
  fi
  if ! build "$D" "$V"; then echo "VARIANT-SKIPPED-BUILD-FAILED $V"; return 0; fi
  bash .sdm/measure_variant.sh "$V" || echo "MEASURE-REPORTED-FAILURE $V"
  echo "@@@@ VARIANT=$V end $(date -u +%H:%M:%S)"
}
START_EPOCH=$(date +%s)
run_variant $A/ctrl  control    control   yes
run_variant $A/treat treatment  treatment yes
# The runner has killed an action at ~18 min once (DeadlineExceeded), and a killed
# action loses the repetitions after it. control2 is a repetition, not an authority,
# so it is dropped rather than risking the two real variants.
ELAPSED=$(( $(date +%s) - START_EPOCH ))
if [ "$ELAPSED" -lt 900 ]; then
  run_variant $A/ctrl  control2   control   yes    yes
else
  echo "SKIPPED control2 (elapsed ${ELAPSED}s > 900s deadline guard)"
fi
echo "AB_DONE $(date -u +%H:%M:%S)"
