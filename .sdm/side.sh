set -e
# ONE self-contained dispatch for ONE tree. The remote is a shared, possibly
# different runner per action, so nothing may be carried between dispatches: this
# fetches its own SHA by hash, verifies it, builds once, then measures and emits.
R=/root/workspace/repo-root
SRC="$PWD"   # ctrl-build lands the uncommitted files in the ACTION cwd, not repo-root
SHA="$1"; SIDE="$2"; MODE="${3:-both}"; NONCE="${4:-0}"   # NONCE: unique per dispatch
# A NONCE goes in the ctrl-build command string because every retry that reuses an identical
# command string is reported as "invocation cancelled": unique per dispatch
echo "dispatch-nonce=$NONCE"
W=/root/sd503_${SIDE}
echo "############ SIDE $SIDE sha=$SHA $(date -u +%H:%M:%S)"
git -C $R worktree prune >/dev/null 2>&1 || true
ok=0
for i in 1 2 3 4 5; do
  if git -C $R fetch --quiet origin "$SHA:refs/sd503/$SIDE" >/dev/null 2>&1; then ok=1; break; fi
  sleep 5
done
[ $ok = 1 ] || { echo "FETCH-FAILED $SHA"; exit 6; }
GOT=$(git -C $R rev-parse "refs/sd503/$SIDE" 2>/dev/null | head -1)
echo "resolved=$GOT"
[ "$GOT" = "$SHA" ] || { echo "SHA-MISMATCH $SIDE got=$GOT want=$SHA"; exit 6; }
rm -rf $W
git -C $R worktree prune
if ! git -C $R worktree add --detach $W "$SHA"; then
  echo "WORKTREE-ADD-FAILED rc=$?; retrying via archive checkout"
  rm -rf $W; mkdir -p $W
  git -C $R archive "$SHA" | tar -x -C $W || { echo "ARCHIVE-FAILED"; exit 9; }
  echo "archived tree at $W files=$(find $W -maxdepth 1 -type d | wc -l)"
fi
cd $W
echo "worktree HEAD=$(git rev-parse HEAD)"
echo "dirty_on_arrival=$(git status --porcelain | wc -l)"

# carry the uncommitted instrument + measurement fixtures (synced as dirty files)
mkdir -p .sdm dag/test
for f in gap_block.rs hooks.py measure_side.sh; do
  [ -f "$SRC/.sdm/$f" ] || { echo "MISSING-INSTRUMENT $SRC/.sdm/$f"; exit 8; }
  cp "$SRC/.sdm/$f" "$W/.sdm/"
done
ls "$SRC"/.sdm/fixtures/gap_depth_*.dag >/dev/null 2>&1 || { echo "MISSING-FIXTURES"; exit 8; }
cp "$SRC"/.sdm/fixtures/gap_depth_*.dag "$W"/dag/test/
if git cat-file -e "$SHA:dag/test/claim/generic_argument_recheck_witness.dag" 2>/dev/null; then
  git cat-file -p "$SHA:dag/test/claim/generic_argument_recheck_witness.dag" > .sdm/witness.dag
  echo "WITNESS present at $SHA"
else
  echo "WITNESS absent at $SHA -- controls are candidate-side only"
fi

echo "--- build $(date -u +%H:%M:%S) ---"
BINS="--bin gunbc"   # measure+closures need only gunbc (the witness runs via gunbc run --claim-run)
if [ "$MODE" != closures ]; then
  python3 .sdm/hooks.py $SIDE .sdm/gap_block.rs
fi
cargo build --release -p v1-compiler $BINS 2>&1 | tail -3
ls -la target/release/gunbc | awk '{print "bin",$5,$9}'
if [ "$MODE" != closures ]; then
  echo "--- measured $(date -u +%H:%M:%S) ---"
  bash .sdm/measure_side.sh $SIDE
fi

if [ "$MODE" != measure ]; then
echo "--- closures $(date -u +%H:%M:%S) ---"
for LANE in selfhost v2native; do
  case $LANE in
    selfhost) ENTRY=src/v2/compiler/source_root_eval.dag ;;
    v2native) ENTRY=src/v2/cli/compile_cli.dag ;;
  esac
  OUT=/tmp/cl_${SIDE}_${LANE}
  rm -rf $OUT
  set +e
  timeout 1200 ./target/release/gunbc compile --source-root dag --source-root src/v2 \
    --entry "$ENTRY" --target rust --output-dir $OUT > /tmp/cl_${SIDE}_${LANE}.log 2>&1
  rc=$?
  set -e
  n=$(find $OUT -type f 2>/dev/null | wc -l)
  echo "EMIT $SIDE $LANE entry=$ENTRY rc=$rc files=$n"
  [ $rc -ne 0 ] && tail -8 /tmp/cl_${SIDE}_${LANE}.log
  if [ -d $OUT ]; then
    ( cd $OUT && find . -type f | sort | while read -r p; do
        printf "CLOSURE %s %s %s\n" "$SIDE" "$LANE" "$p"; done )
    ( cd $OUT && find . -type f | sort | while read -r p; do
        printf "DIGEST %s %s %s %s\n" "$SIDE" "$LANE" "$(sha256sum "$p" | cut -d' ' -f1)" "$p"; done )
  fi
done
fi
echo "SIDE $SIDE DONE $(date -u +%H:%M:%S)"
echo SIDE_DONE
