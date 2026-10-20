#!/usr/bin/env bash
# Regenerate the stage0 mirrors for the STRIP branch (scratch/acc-without-equality-widening),
# where src/v1/04_types.dag's node_type_equals_core was reverted byte-identical to main to answer
# stern-bat-401's consumer question (msg_37357ae1).
#
# WHY THIS EXISTS AT ALL: the gate builds gunbc from the REF, and the crate compiles the stage0
# MIRRORS -- so a .dag-only strip would measure nothing. Mirrors are emitter output, never hand-
# edited or line-merged (the #14287 silent-merge incident is the standing lesson). If the emitter
# refuses to emit, that is REPORTED, not patched around.
#
# Transport: anonymous clones cannot push, so changed mirrors come back as gzip+base64 bands with
# explicit hashes; the installer at home must verify the hash BEFORE committing (a half-decoded
# mirror would be a red tree pushed under a green instrument's name).
set -u
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_regen}"
mkdir -p "$CARGO_TARGET_DIR" 2>/dev/null || true
REPO=https://github.com/gunb-ai/gunbc
REF=${REF:-refs/heads/scratch/acc-without-equality-widening}
D=${D:-/tmp/stripregen}
rm -rf $D
git clone -q --depth 1 -b $REF $REPO $D || { echo "VOID_CLONE_FAILED"; exit 0; }
cd $D
echo "MEASURED_AT=$(git rev-parse HEAD) REF=$REF"

MIRRORS="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
echo "PRE hashes:"
for f in $MIRRORS; do echo "PRE sha=$(sha256sum "$f" 2>/dev/null | cut -c1-16) $f"; done
# The strip must be VISIBLE in the .dag before we trust any regen: node_type_equals_core must have
# no type_heads_agree call site, and the mirror must still contain one (proving drift exists).
echo "STRIP_VISIBLE dag_calls_type_heads_agree=$(grep -c 'type_heads_agree(left' src/v1/04_types.dag)"
echo "DRIFT_EXISTS mirror_calls_type_heads_agree=$(grep -c 'type_heads_agree(' src/v1/stage0/src/v1_compiler_infer_types.rs)"

CE="$CARGO_TARGET_DIR/release/claim_executor"
build_ce(){ timeout -k 30 "${BUILD_TIMEOUT:-1500}" cargo build --release --bin claim_executor 2>&1 | grep -E "^error|Finished" | tail -3; }

round=0; INST=0
while [ "$round" -lt "${MAX_ROUNDS:-2}" ]; do
  round=$((round+1))
  echo "########## round $round: build claim_executor from the CURRENT mirrors ##########"
  build_ce
  [ -x "$CE" ] || { echo "FATAL no claim_executor at $CE"; exit 1; }
  echo "########## round $round: emit candidate ##########"
  rm -rf target/stage0-regen-candidate
  timeout -k 30 "${GEN_TIMEOUT:-800}" "$CE" --required-regen --source-root dag --source-root src/v2 > "/tmp/rr_$round.log" 2>&1
  echo "RR_EXIT=$? round=$round  (1 expected: the .dag changed, so drift exists)"
  grep -aE "first_generation_equal|candidate=|declared_divergent|FAIL generated surface|EmissionRefused|hard diagnostic|type mismatch|elapsed_ms" "/tmp/rr_$round.log" | tail -8 | sed 's/^/RR /'
  [ -d target/stage0-regen-candidate/src ] || { echo "NO_CANDIDATE_DIR round=$round"; break; }
  INST=0
  for f in $MIRRORS; do
    b=$(basename "$f")
    if [ -f "target/stage0-regen-candidate/src/$b" ]; then
      if ! cmp -s "target/stage0-regen-candidate/src/$b" "$f"; then
        cp "target/stage0-regen-candidate/src/$b" "$f"; INST=$((INST+1)); echo "INSTALLED $b"
      fi
    fi
  done
  echo "ROUND $round installed=$INST"
  # Round 2 must install 0: a fixed point means the installed mirrors ARE the emitter's output for
  # this .dag. Anything else means we are shipping something the emitter did not produce.
  [ "$INST" -eq 0 ] && break
done
echo "FIXED_POINT(installed=0 on last round)=$([ "$INST" -eq 0 ] && echo YES || echo NO_DO_NOT_TRUST)"

echo "POST hashes:"
for f in $MIRRORS; do echo "POST sha=$(sha256sum "$f" 2>/dev/null | cut -c1-16) $f"; done
echo "----TRACKED MOVES (report only; nothing is committed here)----"
git status --porcelain | grep -v '^??'
echo "----PUSH CAPABILITY PROBE (informational)----"
timeout 60 git push --dry-run origin HEAD:$REF 2>&1 | tail -2

# Transport for the mirrors that changed: one band per changed file, gzip+base64, with the hash the
# installer must reproduce. No tail/head anywhere: a truncated band is unreadable, not red.
mkdir -p /tmp/stripout; rm -f /tmp/stripout/*
for f in $MIRRORS; do
  b=$(basename "$f")
  if ! git diff --quiet -- "$f" 2>/dev/null || [ -n "$(git status --porcelain -- "$f")" ]; then
    gzip -9 -c "$f" | base64 -w0 > "/tmp/stripout/$b.b64"
    echo "----BEGIN-MIRROR $b sha256=$(sha256sum "$f" | cut -d' ' -f1) bytes=$(wc -c < "$f") b64=$(wc -c < /tmp/stripout/$b.b64)----"
    cat "/tmp/stripout/$b.b64"
    echo ""
    echo "----END-MIRROR $b----"
  fi
done
echo "ALLDONE round=$round installed=$INST"
