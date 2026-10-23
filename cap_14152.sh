#!/bin/bash
# Capture-discriminator measurement on the SHIPPED SEED (parent msg_d5018d11: "add a
# discriminating control ... two generic fns that both use T, one called inside the other's
# argument, with different settled types"). Seed identity guarded; lib.rs restored from a
# /tmp backup afterwards. NO git checkout/clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16) (want 36 / 5e9c37585fe088ac)"
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error|panicked|must not collide|must refuse|is legal" | head -40
echo CAP_DONE
