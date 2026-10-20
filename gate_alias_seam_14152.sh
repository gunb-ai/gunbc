#!/usr/bin/env bash
# Prereq (b) A/B, using the repo's OWN authority rather than a hand-written probe.
#
# The refusal under investigation is already reproduced on main by a CI-executed claim module,
# dag/test/claim/algebra_receiver_alias_witness.dag: `-> FreeMonoid<Int>` accepts while the same body
# written `-> List<Int>` refuses with class MethodNotFound / subject Coproduct(FreeMonoid), at return and
# at field position. Those four rows are the fixtures; I do not retype them and I do not hand-build node
# trees (a hand-built tree would test my own walk, not the compiler).
#
# The question (stern-bat-401, msg_11dcdef2): does substituting in transparent_alias_rhs move the
# diagnostic produced at variant_reference_inferred_node? Answer it by running the four rows through the
# real binary on (a) the control tree and (b) a mutant whose params>0 term in transparent_alias_rhs is
# constant-false. If the two refusing rows go clean on the mutant, the guard IS the boundary; if they
# refuse identically, the seams are different and prereq (b) re-scopes to the owner-identity carry at
# variant_reference_inferred_node (already owned by
# gunbc.guarantee_stall.algebra_application_owner_resolution_stall).
#
# WHY THE BINARY AND NOT A V1 RUST PROBE: three earlier probe dispatches lost their verdict to capture
# bugs of mine (a base64 tail cut, a keyword grep that filtered out the panic MESSAGE line which wraps to
# the next line, and a write-path/read-path mismatch). Running the existing claim module removes the
# probe from the causal chain entirely -- the assertion text, the fixture strings, and the severity policy
# all come from the authority CI already runs, and a claim row's exit code is itself the verdict.
#
# USAGE: gate_alias_seam_14152.sh <sha> <label>
set -u
SHA="${1:?sha}"; LABEL="${2:?label}"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_seam}"
mkdir -p "$CARGO_TARGET_DIR"
D=/tmp/seam_$(echo "$LABEL" | tr -c 'a-zA-Z0-9' '_')
stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }
stamp "alias seam gate begin sha=$SHA label=$LABEL"

rm -rf "$D"; mkdir -p "$D"
CLONED=0
for a in 1 2 3; do
  rm -rf "$D/repo"
  if timeout 900 git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$D/repo" 2>"$D/cerr.$a"; then CLONED=1; stamp "clone ok attempt $a"; break; fi
  stamp "clone attempt $a failed: $(tr '\n' ' ' < "$D/cerr.$a" | cut -c1-200)"; sleep 20
done
[ "$CLONED" = 1 ] || { echo "VOID clone failed (3 attempts)"; exit 7; }
cd "$D/repo" || { echo "VOID cd"; exit 7; }
timeout 300 git fetch -q origin "$SHA" && timeout 120 git checkout -q "$SHA" || { echo "VOID checkout $SHA"; exit 7; }
echo "MEASURED_AT $(git rev-parse HEAD)"

# Prove the two trees differ in exactly the guard, so a null result cannot be read as "the guard is
# irrelevant" when the real cause is that the mutant never applied.
echo "GUARD_STATE params_term_in_mirror=$(grep -c 'pre.params.clone().len() as i64) > 0' src/v1/stage0/src/v1_compiler_infer_env.rs) params_term_in_dag=$(grep -c 'pre.params |> count) > 0' src/v1/04_env.dag) neutered_in_mirror=$(grep -c '(0 as i64) > 0' src/v1/stage0/src/v1_compiler_infer_env.rs) neutered_in_dag=$(grep -c '(0 > 0)' src/v1/04_env.dag)"

stamp "build release binary begin"
timeout 2400 cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
BUILD_RC=$?
echo "build rc=$BUILD_RC"
[ "$BUILD_RC" = 0 ] || { echo "VOID binary did not build"; exit 6; }
BIN="$CARGO_TARGET_DIR/release/gunbc"
echo "binary=$BIN exists=$([ -x "$BIN" ] && echo yes || echo no)"
[ -x "$BIN" ] || { echo "VOID expected binary absent (not falling back to PATH)"; exit 6; }
case "$BIN" in *ctrl-build-shims*) echo "VOID resolved a ctrl-build shim"; exit 6 ;; esac

# The four CI-run claim rows, by name, exactly as the authority defines them.
FNS="nominal_return_is_an_acceptance_fixture alias_return_is_a_refusal_fixture alias_field_is_a_refusal_fixture explicit_initializer_is_an_acceptance_fixture"
ENTRY="dag/test/claim/algebra_receiver_alias_witness.dag"
for f in $FNS; do
  "$BIN" run --source-root "$D/repo/dag" --source-root "$D/repo/src/v2" \
     --entry "$ENTRY" --function "$f" > "$D/out.$f" 2>&1
  rc=$?
  # Print the WHOLE relevant region, never a keyword filter: a keyword grep is what hid a panic message
  # on the previous attempt. The log is tiny (a single claim row's verdict).
  echo "ROW $f rc=$rc bytes=$(wc -c < "$D/out.$f")"
  sed 's/^/  | /' "$D/out.$f" | head -25
done
echo "GATE DONE"
