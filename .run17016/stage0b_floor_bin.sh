#!/usr/bin/env bash
# STAGE 0b: build the FLOOR binary from main-$MB in its OWN acquisition (binary and tree agree).
. ./stage_common.sh
export RUSTUP_TOOLCHAIN=1.93.0-aarch64-unknown-linux-gnu
MB=d196f49052362ff8b3ac32d60928bd99f3f3be4a
test -e "main-$MB/.git" || { echo MAIN_TREE_MISSING; exit 8; }
command -v cargo > /dev/null 2>&1; echo "NO_CARGO_PROBE_RC=$?"
cargo --version; echo "CARGO_VERSION_RC=$?"
rustc --version; echo "RUSTC_VERSION_RC=$?"
echo "PRE_LOAD1MIN=$(awk '{print $1}' /proc/loadavg)"
cd "main-$MB" || exit 9
cargo build --release -q -p v1-compiler --bin gunbc 2> build_floor.err; RC=$?
echo "FLOOR_CARGO_RC=$RC"; tail -3 build_floor.err
mkdir -p bin
test -x target/release/gunbc && cp -f target/release/gunbc bin/gunbc-main-$MB
echo "BINARY_PROVENANCE=main-built manifest-unmatched sha256=$(sha256sum bin/gunbc-main-$MB | awk '{print $1}') built_from=$(git rev-parse HEAD)"
exit $RC
