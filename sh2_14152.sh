#!/bin/bash
# Self-host lane reproduction, round 2. Round 1 was refused cause=StaleBinary: gunbc
# refuses when one of its own inputs is newer than the binary, so the build must come
# AFTER each mirror swap, inside the arm. NO git checkout/clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
use_green() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
use_red()   { for f in $MIRROR_FILES; do cp "seedred/$(basename $f)" "$f"; done; }
echo "HEAD=$(git rev-parse --short HEAD) dagBoundAs=$(grep -c 'bound as' src/v1/04_infer.dag)"

echo "########## A: fold seed (UnifyGenericsAcc=36) ##########"
use_green
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > /tmp/shA.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/shA.log) lines=$(wc -l < /tmp/shA.log)"
tail -4 /tmp/shA.log | cut -c1-220

echo "########## B: pre-fold seed (UnifyGenericsAcc=0) ##########"
use_red
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > /tmp/shB.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/shB.log) lines=$(wc -l < /tmp/shB.log)"
tail -4 /tmp/shB.log | cut -c1-220

use_green
echo "restored: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
cp /tmp/shA.log shA.log; cp /tmp/shB.log shB.log
echo SH2_DONE
