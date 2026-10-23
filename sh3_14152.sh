#!/bin/bash
# Self-host corpus + census in ONE dispatch (runner /tmp does not survive across
# dispatches). Arm A: fold seed -> reproduce the refusal corpus, run the COMMITTED
# instrument over it, write both into the worktree so they can be committed. Arm B:
# pre-fold seed -> the compile errors the fold prevents (the RED). NO git checkout/clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
use_green() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
use_red()   { for f in $MIRROR_FILES; do cp "seedred/$(basename $f)" "$f"; done; }
echo "HEAD=$(git rev-parse --short HEAD)"

echo "########## A: fold seed (UnifyGenericsAcc=36) ##########"
use_green
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > selfhost_fold_seed.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' selfhost_fold_seed.log)"
echo "----- CENSUS over the reproduced corpus (committed instrument) -----"
bash tools/rebind_census.sh selfhost_fold_seed.log --json selfhost_fold_seed_census.json 2>&1 | sed -n '1,20p'
cp selfhost_fold_seed_census.json /tmp/selfhost_census.json 2>/dev/null

echo "########## B: pre-fold seed (UnifyGenericsAcc=0) ##########"
use_red
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > selfhost_prefold_seed.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' selfhost_prefold_seed.log)"
echo "----- the compile errors the fold prevents -----"
grep -E "^error(\[|:)" selfhost_prefold_seed.log | head -10 | cut -c1-300
use_green
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo SH3_DONE
