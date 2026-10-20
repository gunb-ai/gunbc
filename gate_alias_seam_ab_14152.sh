#!/usr/bin/env bash
# Prereq (b) alias-seam A/B, single-binary design (stern-bat-401 msg_11dcdef2).
#
# Question: does substituting in transparent_alias_rhs move the diagnostic produced at
# variant_reference_inferred_node? The params>0 term of transparent_alias_rhs is env-gated on the probe
# ref (GUNBC_ALIAS_GUARD_OFF), so ONE built binary answers by running twice. Two separate builds could
# differ by more than the guard and produce a byte-identical output for the wrong reason; one binary
# cannot, which removes the NULL-result confound I had to pre-register against.
#
# The four programs are read from the authority CI already runs
# (dag/test/claim/algebra_receiver_alias_witness.dag) and judged by the exact host builtin its claim rows
# call (compile_dag_diagnostic_census), so the severity policy and class identity come from the same place
# CI reads them. Prints the census ARM, so NotRunnable is never read as a clean compile.
#
# USAGE: gate_alias_seam_ab_14152.sh <sha> <label>
set -u
SHA="${1:?sha}"; LABEL="${2:?label}"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_seamab}"
mkdir -p "$CARGO_TARGET_DIR"
D=/tmp/seamab_$(echo "$LABEL" | tr -c 'a-zA-Z0-9' '_')
stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }
stamp "alias A/B begin sha=$SHA label=$LABEL target=$(du -sh "$CARGO_TARGET_DIR" 2>/dev/null|cut -f1||echo none)"

rm -rf "$D"; mkdir -p "$D"
CLONED=0
for a in 1 2 3; do
  rm -rf "$D/repo"
  if timeout 900 git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$D/repo" 2>"$D/cerr.$a"; then CLONED=1; stamp "clone ok attempt $a"; break; fi
  stamp "clone attempt $a failed: $(tr '\n' ' ' < "$D/cerr.$a"|cut -c1-200)"; sleep 20
done
[ "$CLONED" = 1 ] || { echo "VOID clone failed (3 attempts)"; exit 7; }
cd "$D/repo" || { echo "VOID cd"; exit 7; }
timeout 300 git fetch -q origin "$SHA" && timeout 120 git checkout -q "$SHA" || { echo "VOID checkout $SHA"; exit 7; }
echo "MEASURED_AT $(git rev-parse HEAD)"
grep -q GUNBC_ALIAS_GUARD_OFF src/v1/stage0/src/v1_compiler_infer_env.rs \
  || { echo "VOID the guard is not env-gated at this sha; the A/B would be a null comparison"; exit 8; }
grep -q "mod alias_seam_ab_test;" src/v1/tests/src/lib.rs \
  || { echo "VOID the A/B row is not registered at this sha"; exit 8; }

stamp "build one binary (used for BOTH sides)"
timeout 3000 cargo build --release -p v1-compiler-tests --test '*' 2>&1 | tail -3
BUILD_RC=$?
echo "build rc=$BUILD_RC"
[ "$BUILD_RC" = 0 ] || { echo "VOID the test binary did not build; there is no A/B to read"; exit 6; }

for side in CONTROL MUTANT; do
  echo "==================== SIDE=$side ===================="
  if [ "$side" = MUTANT ]; then export GUNBC_ALIAS_GUARD_OFF=1; else unset GUNBC_ALIAS_GUARD_OFF; fi
  echo "side=$side GUNBC_ALIAS_GUARD_OFF=${GUNBC_ALIAS_GUARD_OFF-<unset>}"
  timeout 1800 cargo test --release -p v1-compiler-tests alias_seam_a_b -- --nocapture --exact \
      alias_seam_ab_test::alias_seam_a_b > "$D/$side.log" 2>&1
  rc=$?
  echo "${side}_TEST_RC=$rc log_bytes=$(wc -c < "$D/$side.log")"
  echo "----BEGIN $side----"
  grep -E "SEAM |test result:|panicked|error\[|error:" "$D/$side.log" | head -70
  echo "-----END $side----"
  # An absence is not a pass: the row must have produced SEAM lines.
  SEAMS=$(grep -c "^SEAM " "$D/$side.log" || true)
  echo "$side SEAM_LINES=$SEAMS"
  [ "$SEAMS" -ge 8 ] || echo "$side VOID_TO0: fewer than 8 SEAM lines means no measurement was produced (rc=$rc)"
done
echo "GATE DONE"
