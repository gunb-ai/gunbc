#!/usr/bin/env bash
# STAGE 1: build the head's gunbc from head-$HEAD inside the slot.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"
test -d "$T" || { echo "TREE_MISSING head-$HEAD"; exit 8; }
echo "PRE_LOAD1MIN=$(awk '{print $1}' /proc/loadavg)"
cd "$T" || exit 9
cargo build --release -q -p gunbc 2> build.err; RC=$?
echo "CARGO_RC=$RC"; tail -3 build.err
test -x target/release/gunbc && echo "BINARY_SHA256=$(sha256sum target/release/gunbc | awk '{print $1}')"
echo "BUILT_FROM=$(git rev-parse HEAD) head-built, manifest-unmatched"
exit $RC
