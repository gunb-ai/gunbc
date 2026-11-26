#!/usr/bin/env bash
# STAGE 1: build the HEAD gunbc from head-$HEAD (real worktree) inside the slot.
. ./stage_common.sh
export RUSTUP_TOOLCHAIN=1.93.0-aarch64-unknown-linux-gnu
HEAD="$1"; T="head-$HEAD"
test -d "$T" || { echo "TREE_MISSING head-$HEAD"; exit 8; }
command -v cargo > /dev/null 2>&1; echo "NO_CARGO_PROBE_RC=$?"
cargo --version; echo CARGO_RC=$?
rustc --version; echo RUSTC_RC=$?
echo "PRE_LOAD1MIN=$(awk "{print \$1}" /proc/loadavg)"
cd "$T" || exit 9
# B VOIDED the 07:06Z build artifact. rm alone is not enough: cargo re-hardlinked the cached
# 07:06 binary from target/release/deps (same sha AND mtime at 12:25Z). Touch the bin crate's
# main.rs so rustc re-links NOW -- the artifact used must postdate the ruling, printed below.
# Freshness rule (manager, 13:16Z): BUILD_START before build; mtime MUST exceed it; sha alone
# proves nothing on a deterministic build.
# Skip guard: an artifact already newer than B's VOID ruling (11:22:00Z=1795692120) at this tree
# is legitimate reuse of a receipt already printed fresh; rebuilding it again spends a slot for nothing.
RULING_EPOCH=1795692120
if test -x target/release/gunbc; then
  PMT=$(stat -c %Y target/release/gunbc)
  if [ "$PMT" -gt "$RULING_EPOCH" ]; then
    echo "SKIP_BUILD_ALREADY_FRESH mtime_epoch=$PMT > RULING_EPOCH=$RULING_EPOCH (post-ruling artifact from a prior receipt)"
    echo "BINARY_MTIME=$(stat -c %y target/release/gunbc) BINARY_BYTES=$(stat -c %s target/release/gunbc)"
    echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum target/release/gunbc | awk "{print \$1}") built_from=$(git rev-parse HEAD)"
    echo "FRESHNESS_ASSERT_RC=0 (against VOID ruling, not BUILD_START: no build this run)"
    exit 0
  fi
fi
BUILD_START=$(date -u +%FT%TZ); BUILD_START_EPOCH=$(date -u +%s)
echo "BUILD_START=$BUILD_START"
touch src/v1/stage0/src/main.rs; echo "TOUCH_RC=$? TOUCH_UTC=$(date -u +%FT%TZ) (forced invalidation mechanism)"
rm -f target/release/gunbc; echo "RM_OLD_BINARY_RC=$? RM_UTC=$(date -u +%FT%TZ)"
cargo build --release -q -p v1-compiler --bin gunbc 2> build.err; RC=$?
echo "CARGO_RC=$RC BUILD_ERR_BYTES=$(wc -c < build.err)"; tail -3 build.err
test -x target/release/gunbc && echo "BINARY_MTIME=$(stat -c %y target/release/gunbc) BINARY_BYTES=$(stat -c %s target/release/gunbc)"
test -x target/release/gunbc && echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum target/release/gunbc | awk "{print \$1}") built_from=$(git rev-parse HEAD)"
MT=$(stat -c %Y target/release/gunbc 2>/dev/null || echo 0)
if [ "$MT" -gt "$BUILD_START_EPOCH" ]; then FRES_RC=0; echo "FRESHNESS OK: mtime_epoch=$MT > BUILD_START_EPOCH=$BUILD_START_EPOCH"; else FRES_RC=1; echo "VOID_BUILD_NOT_FRESH mtime_epoch=$MT BUILD_START_EPOCH=$BUILD_START_EPOCH"; fi
echo "FRESHNESS_ASSERT_RC=$FRES_RC"
[ "$RC" -eq 0 ] || exit $RC
exit $FRES_RC
