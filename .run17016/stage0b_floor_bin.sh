#!/usr/bin/env bash
# STAGE 0b: build the FLOOR binary from the cutover main tree, inside the slot.
# Everything floor-side (stage2b/stageF) runs with THIS binary, labelled
# 'main-built, manifest-unmatched' with its on-disk sha256.
. ./stage_common.sh
MB=4036f188602
test -d "main-$MB" || { echo "MAIN_TREE_MISSING"; exit 8; }
echo "PRE_LOAD1MIN=$(awk '{print $1}' /proc/loadavg)"
cd "main-$MB" || exit 9
cargo build --release -q -p gunbc 2> build_floor.err; RC=$?
echo "CARGO_RC=$RC"; tail -3 build_floor.err
mkdir -p ../bin
test -x target/release/gunbc && cp -f target/release/gunbc ../bin/gunbc-main-$MB
sha256sum ../bin/gunbc-main-$MB | sed 's/^/FLOOR_BIN_SHA256=/'
echo "FLOOR_BUILT_FROM=$(git rev-parse --short HEAD) main-built, manifest-unmatched"
exit $RC
