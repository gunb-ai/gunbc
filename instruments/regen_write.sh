#!/usr/bin/env bash
# Regenerate the stage0 mirrors from a patched .dag, in the form that actually WRITES.
#
# Why not instruments/regen_full.sh: it runs `--required-regen` (verify, ~420s) AND
# `--regen-round-cost` (~another pass) back to back, and a dispatch died at its own timeout
# partway through the second. `--required-regen` is the form that EMITS the candidate tree
# (it exits 1 on drift, which is EXPECTED here -- I just changed the .dag); the install is
# the proven candidate->tree copy with an installed count, and the fixed point is the round
# that installs 0. Nothing here is a guess about output wording: the loop below is the one
# that produced this branch's previous regeneration (round 1 installed 4, round 2 installed 0).
#
# Build once into a PERSISTED target dir: a cold release build is ~330s, the runner's /root
# survives between dispatches and /tmp does not.
set -u
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_regen}"
mkdir -p "$CARGO_TARGET_DIR"
MAX_ROUNDS="${MAX_ROUNDS:-2}"   # round 1 installs, round 2 must install 0; a 3rd would risk the 45-min client cap
GEN_TIMEOUT="${GEN_TIMEOUT:-800}"
BUILD_TIMEOUT="${BUILD_TIMEOUT:-1500}"
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs

echo "HEAD=$(git rev-parse HEAD)"
echo "unmerged=$(git diff --name-only --diff-filter=U | tr '\n' ' ')"
du -sh "$CARGO_TARGET_DIR" 2>/dev/null | sed 's/^/TARGET_CACHE /'
seed(){ sha256sum "$MIRROR" 2>/dev/null | cut -c1-16; }
echo "SEED_BEFORE $(seed)"

echo "########## build claim_executor ##########"
timeout -k 30 "$BUILD_TIMEOUT" cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -3
[ -x target/release/claim_executor ] || { echo "FATAL no claim_executor"; exit 1; }

round=0
while [ "$round" -lt "$MAX_ROUNDS" ]; do
  round=$((round+1))
  echo "########## round $round: emit candidate ##########"
  rm -rf target/stage0-regen-candidate
  timeout -k 30 "$GEN_TIMEOUT" ./target/release/claim_executor --required-regen \
      --source-root dag --source-root src/v2 > "/tmp/rr_$round.log" 2>&1
  echo "RR_EXIT=$? round=$round   (1 is expected: the .dag changed, so there IS drift)"
  grep -aE "first_generation_equal|candidate=|declared_divergent|FAIL generated surface|admitted edges|elapsed_ms" \
    "/tmp/rr_$round.log" | tail -6 | sed 's/^/RR /'
  [ -d target/stage0-regen-candidate/src ] || { echo "NO_CANDIDATE_DIR round=$round -- cannot install"; break; }

  echo "-------- install (candidate vs tree, by file) --------"
  INST=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f "src/v1/stage0/src/$f" ]; then
      if ! cmp -s "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"; then
        cp "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f"
        INST=$((INST+1)); echo "installed $f"
      fi
    fi
  done
  echo "ROUND $round installed=$INST  SEED sha=$(seed)"
  [ "$INST" = "0" ] && { echo "CONVERGED at round=$round  SEED sha=$(seed)"; break; }
done

echo "SEED_AFTER $(seed)"
echo "mirror fold refs=$(grep -c UnifyGenericsAcc "$MIRROR")"
echo "mirror repair marks=$(grep -c fresh_call_variables "$MIRROR")"
git status --porcelain | grep -v '^??' | sed 's/^/CHANGED /'
# Hand back what changed so the commit step uses the generator's bytes, never hand-written ones.
tar -czf /tmp/changed.tgz $(git diff --name-only | grep -E '\.rs$|\.dag$' || true) 2>/dev/null
echo "CHANGED_TAR_BYTES $(wc -c < /tmp/changed.tgz 2>/dev/null || echo 0)"
echo REGEN_WRITE_DONE
