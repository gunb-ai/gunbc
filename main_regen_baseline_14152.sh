#!/usr/bin/env bash
# Does MAIN, alone, pass --required-regen when the emitter is built from main's own mirrors?
#
# WHY THIS EXISTS. The merge regen reported `FAIL generated surface drift: std_decimal.rs` on the
# merged tree, while neither side had touched that file since the merge base (0 changed on both
# sides, 0 diff lines between them). So the drift is the emitter's candidate disagreeing with a
# committed mirror. Three explanations remain and they need different actions:
#   (a) main's committed std_decimal.rs is already stale against main's own authorities -- then it is
#       a pre-existing condition, not something my repair introduced, and my head inherits it;
#   (b) my repair changes what the compiler emits for decimal.dag -- then the drift is MINE, the
#       regenerated std_decimal.rs belongs in my head, and my two-closure numbers must be re-measured
#       at the merged tree because generated output moved beyond the two stage0 mirrors;
#   (c) the text-merge of the two mirrors produced an emitter that is neither side's compiler -- an
#       artifact of merging, resolved by regenerating to a fixed point.
# Running the SAME pass on main isolates (a): same clone discipline, same emitter build from the
# tree's own mirrors, same two --source-root flags. If main is clean, the drift is not pre-existing.
#
# usage: main_regen_baseline_14152.sh <main-sha>
set -uo pipefail
MAIN="${1:?usage: main_regen_baseline_14152.sh <main-sha>}"
D=/tmp/mrb14152; LOGS="${LOGS:-/tmp/mrb14152_logs}"
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_mrb}"
rm -rf "$D" "$LOGS"; mkdir -p "$D" "$LOGS"
stamp(){ printf 'T=%s %s\n' "$(date -u +%H:%M:%S)" "$*"; }
URL="${REPO_URL:-https://github.com/gunb-ai/gunbc}"
stamp "clone"
timeout 600 git clone -q "$URL" "$D/repo" 2>"$LOGS/clone.err" || { echo "FATAL clone"; cat "$LOGS/clone.err"; exit 1; }
cd "$D/repo"
timeout 300 git fetch -q origin "$MAIN" 2>>"$LOGS/clone.err" || { echo "FATAL fetch"; cat "$LOGS/clone.err"; exit 1; }
git checkout -q "$MAIN" || { echo "FATAL checkout"; exit 1; }
echo "BASELINE_AT $(git rev-parse HEAD)"
printf 'SEED_MIRROR %s\n' "$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
stamp "build the emitter from main's own mirrors"
( cargo build --release --bin claim_executor ) > "$LOGS/emitter_build.log" 2>&1
brc=$?; echo "EMITTER_BUILD rc=$brc"
[ $brc -eq 0 ] || { grep -aE "^error" "$LOGS/emitter_build.log" | head -6; exit 6; }
CE_BIN="$CARGO_TARGET_DIR/release/claim_executor"
stamp "--required-regen on main"
timeout -k 60 "${REGEN_TIMEOUT:-1800}" "$CE_BIN" --required-regen --source-root dag --source-root src/v2 \
  > "$LOGS/regen.log" 2>&1
rrc=$?
echo "REQUIRED_REGEN rc=$rrc  (0 = main is a fixed point as committed)"
grep -aE "first_generation_equal|declared_divergent|FAIL generated|admitted edges|planned=" "$LOGS/regen.log" | tail -8 | sed 's/^/RR /'
# The candidate dir is where the emitter wrote what it WOULD install; naming every file that differs
# from the tree is the answer to "is this drift pre-existing on main".
if [ -d target/stage0-regen-candidate/src ]; then
  n=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f "src/v1/stage0/src/$f" ]; then
      cmp -s "target/stage0-regen-candidate/src/$f" "src/v1/stage0/src/$f" || { echo "MAIN_DRIFTS $f"; n=$((n+1)); }
    fi
  done
  echo "MAIN_DRIFT_COUNT $n"
else
  echo "MAIN_NO_CANDIDATE_DIR (nothing emitted -- read the RR lines above before concluding anything)"
fi
tar czf "$LOGS/changed.tgz" "$LOGS/regen.log" 2>/dev/null
echo "BEGIN-LOG changed.tgz"; base64 -w0 "$LOGS/changed.tgz"; echo; echo "END-LOG changed.tgz"
stamp "ALLDONE"
