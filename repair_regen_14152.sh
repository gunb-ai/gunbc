#!/usr/bin/env bash
# Dispatch 1 of the producer repair: apply the .dag patch, REGENERATE the mirrors to a fixed
# point, and hand back the regenerated files. Deliberately does NOT run the test crate: a
# cold cargo build plus regeneration does not fit in one dispatch, and the runner gives every
# dispatch a fresh /tmp (a detached child dies with its container -- tried, SIZE=0 twice), so
# the jobs have to be split instead.
#
# Measured on a FRESH CLONE at a printed SHA (never the shared /root/workspace/repo-root,
# which is dirty with other sessions' .dag edits and produced a fake drift result in this
# lane). Prints every number it relies on; a missing marker is FATAL, not a silent pass.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
SHA="${1:?usage: repair_regen_14152.sh <sha>}"
URL=https://github.com/gunb-ai/gunbc
INSTR_REF="scratch/rebind-census"
D=/tmp/repair_regen
LOGS=/tmp/repair_regen_logs
INFER_DAG=src/v1/04_infer.dag
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs
CARGO_TIMEOUT="${CARGO_TIMEOUT:-1200}"

say(){ echo "=== $* ==="; }
seed(){ sha256sum "$1" | cut -c1-16; }

say "environment"
command -v cargo || { echo "FATAL no cargo"; exit 1; }
nproc | sed 's/^/NPROC /'
free -m 2>/dev/null | head -2 | tr '\n' ' ' | sed 's/^/MEM /'

say "clone at $SHA"
rm -rf "$D" "$LOGS"; mkdir -p "$D" "$LOGS"
timeout 600 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
timeout 300 git fetch -q origin "$SHA" "$INSTR_REF" || { echo "FATAL fetch"; exit 1; }
timeout 120 git checkout -q "$SHA" || { echo "FATAL checkout"; exit 1; }
echo "MEASURED_AT $(git rev-parse HEAD)"
git status --porcelain | grep -v '^??' | head -3 | sed 's/^/DIRTY /'
mkdir -p "$D/instr"
git archive "origin/$INSTR_REF" | tar -x -C "$D/instr" || { echo "FATAL instr"; exit 1; }
echo "INSTR_AT $(git rev-parse origin/$INSTR_REF)"

say "pristine seeds"
sha256sum "$INFER_DAG" "$MIRROR" | sed 's/^/SEED_BEFORE /'
cp "$MIRROR" "$LOGS/mirror.before"

say "apply the producer patch (fresh call-site-keyed variables)"
python3 "$D/instr/patch_producer_instantiate.py" "$D/repo" || { echo "FATAL patch"; exit 1; }
sha256sum "$INFER_DAG" | sed 's/^/SEED_AFTER_DAGPATCH /'
if [ "$(seed "$MIRROR")" = "$(seed "$LOGS/mirror.before")" ]; then
  echo "MIRROR_UNMOVED_OK (expected: the .dag alone cannot change the mirror; this is the"
  echo "  StaleBinary fact that makes regeneration mandatory, not optional)"
fi

say "REGENERATE mirrors from the patched .dag (fixed point)"
# Build flags only, cannot change a test outcome; debuginfo is the dominant term for a crate
# this size and the previous dispatch spent its whole 45-minute budget in a cold build.
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
timeout -k 30 "$CARGO_TIMEOUT" bash "$D/instr/instruments/regen_full.sh" >"$LOGS/regen.log" 2>&1
echo "REGEN_RC=$?"
grep -E "HEAD=|RR_EXIT|RC_EXIT|SEED|installed|round|CONVERGED|REGEN_FULL_DONE|candidates:|unmerged|FATAL" "$LOGS/regen.log" | tail -20 | sed 's/^/REGEN /'
tail -25 "$LOGS/regen.log" | sed 's/^/REGEN_TAIL /'
sha256sum "$MIRROR" | sed 's/^/SEED_AFTER_REGEN /'
if [ "$(seed "$MIRROR")" = "$(seed "$LOGS/mirror.before")" ]; then
  echo "REGEN_NOOP FATAL -- regeneration left the mirror byte-identical: the emitter did not"
  echo "  see the .dag change, or the change is inert (my first repair failed exactly this"
  echo "  way and I nearly shipped it). Nothing below is meaningful."
  echo "BEGIN-LOG regen.log $(wc -c <"$LOGS/regen.log")"; base64 -w0 "$LOGS/regen.log"; echo; echo "END-LOG regen.log"
  exit 4
fi

say "the emitted shape of the seeded region (read, do not guess)"
FIRST=$(grep -n "uninstantiated_call_generics\|fresh_call_variables" "$MIRROR" | head -1 | cut -d: -f1)
if [ -n "${FIRST:-}" ]; then
  echo "EMITTED_AT_LINE $FIRST"
  sed -n "$((FIRST-6)),$((FIRST+46))p" "$MIRROR" | sed 's/^/EMIT /'
else
  echo "EMITTED_MISSING_FATAL -- the seeded code is not in the regenerated mirror"
fi

say "which mirrors changed (the diff by name, for the commit message)"
git status --porcelain | grep -v '^??' | sed 's/^/CHANGED /'
git diff --stat | tail -8 | sed 's/^/DIFFSTAT /'

say "hand back the changed files"
tar -czf "$LOGS/changed.tgz" $(git status --porcelain | grep -v '^??' | awk '{print $2}') 2>/dev/null
echo "CHANGED_TAR_BYTES $(wc -c <"$LOGS/changed.tgz")"
for f in "$LOGS/changed.tgz" "$LOGS/regen.log"; do
  echo "BEGIN-LOG $(basename "$f") $(wc -c <"$f")"; base64 -w0 "$f"; echo; echo "END-LOG $(basename "$f")"
done
echo "ALLDONE"
