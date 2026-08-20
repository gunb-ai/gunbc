set -e
unset RUSTC_WRAPPER || true
export RUSTC_WRAPPER=
FULL=064e50bdb016042cb3311acb1fb08ed64fd819a7
git fetch origin "$FULL"
git checkout --force -B mirror-verify-scratch "$FULL"
GOT=$(git rev-parse HEAD)
if [ "$GOT" != "$FULL" ]; then
  echo "SHA MISMATCH got=$GOT want=$FULL"
  exit 99
fi
echo "PINNED_OK $FULL"
cd src/v1/stage0
rm -rf target
cargo test --release --package v1-compiler required_regen_host::tests::scratch_content_sync_two_files -- --nocapture --exact
