#!/bin/bash
# Reproduce the v2-bootstrap self-host lane (the corpus that produced the 220 rebind
# refusals in run 38918165053) LOCALLY on the runner, at the exact head, in both seed
# variants. Turns the census predictions into measurements on the exact head.
# NO git checkout / git clean on the runner; mirrors restored by cp from carriers.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
use_green() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
use_red()   { for f in $MIRROR_FILES; do cp "seedred/$(basename $f)" "$f"; done; }
echo "HEAD=$(git rev-parse --short HEAD) dagFold=$(grep -c 'bound as' src/v1/04_infer.dag)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -3

echo "########## SELFHOST A: fold seed (green, UnifyGenericsAcc=36) ##########"
use_green
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_green.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sh_green.log) lines=$(wc -l < /tmp/sh_green.log)"
tail -6 /tmp/sh_green.log | cut -c1-200

echo "########## SELFHOST B: pre-fold seed (red, UnifyGenericsAcc=0) ##########"
use_red
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_red.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sh_red.log) lines=$(wc -l < /tmp/sh_red.log)"
tail -6 /tmp/sh_red.log | cut -c1-200

use_green
echo "restored: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
cp /tmp/sh_green.log sh_green.log; cp /tmp/sh_red.log sh_red.log
echo SH_DONE
