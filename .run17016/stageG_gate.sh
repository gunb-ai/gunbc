#!/usr/bin/env bash
# STAGE G (warm-fox's compile gate), head-built binary from head-$HEAD.
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY $B"; exit 9; }
echo "BINARY_SHA256=$(sha256sum "$B" | awk '{print $1}')"
echo "BUILT_FROM=$(git -C "$T" rev-parse HEAD) head-built, manifest-unmatched"
echo "GATE1MIN=$(awk '{print $1}' /proc/loadavg)"
# (a) both targets on the wall module
"$B" compile --entry "$T/dag/test/claim/filesystem_effect_reach_wall.dag" --target dag  --source-root "$T/dag" --source-root "$T/src/v2" --output-dir gate_dag  > gate.dag.log  2>&1; G1=$?
echo GATE_RC_DAG=$G1; grep -E "blocking error" gate.dag.log | head -1
"$B" compile --entry "$T/dag/test/claim/filesystem_effect_reach_wall.dag" --target rust --source-root "$T/dag" --source-root "$T/src/v2" --output-dir gate_rust > gate.rust.log 2>&1; G2=$?
echo GATE_RC_RUST=$G2; grep -E "blocking error" gate.rust.log | head -1
# (b) second entry: the sibling module, both targets
"$B" compile --entry "$T/dag/test/claim/a_payload_surplus_variant_wall.dag" --target dag  --source-root "$T/dag" --source-root "$T/src/v2" --output-dir gate2_dag  > gate2.dag.log  2>&1; G3=$?
echo GATE2_RC_DAG=$G3; grep -E "blocking error" gate2.dag.log | head -1
"$B" compile --entry "$T/dag/test/claim/a_payload_surplus_variant_wall.dag" --target rust --source-root "$T/dag" --source-root "$T/src/v2" --output-dir gate2_rust > gate2.rust.log 2>&1; G4=$?
echo GATE2_RC_RUST=$G4; grep -E "blocking error" gate2.rust.log | head -1
# (c) control: known-good sibling, both targets
"$B" compile --entry "$T/dag/test/claim/resolve_canonical_witness.dag" --target dag  --source-root "$T/dag" --source-root "$T/src/v2" --output-dir ctrl_dag  > ctrl.dag.log  2>&1; C1=$?
echo CONTROL_RC_DAG=$C1; grep -E "blocking error" ctrl.dag.log | head -1
"$B" compile --entry "$T/dag/test/claim/resolve_canonical_witness.dag" --target rust --source-root "$T/dag" --source-root "$T/src/v2" --output-dir ctrl_rust > ctrl.rust.log 2>&1; C2=$?
echo CONTROL_RC_RUST=$C2; grep -E "blocking error" ctrl.rust.log | head -1
# (d) name-refusal gate: unknown name must refuse with rc 2 and refusal text
"$B" test --entry "$T/dag/test/claim/filesystem_effect_reach_wall.dag" --source-root "$T/dag" --source-root "$T/src/v2" --tests "src/v2/test/claim/filesystem_effect_reach_wall:definitely_not_a_real_fn_zzz" > gate_name.log 2>&1; N1=$?
echo NAME_RC=$N1
[ "$N1" -eq 2 ] && grep -m2 -iE "refus|unknown|not found|no such" gate_name.log | head -2
if [ "$G1" -eq 0 ] && [ "$G2" -eq 0 ] && [ "$G3" -eq 0 ] && [ "$G4" -eq 0 ] && [ "$C1" -eq 0 ] && [ "$C2" -eq 0 ] && [ "$N1" -eq 2 ]; then
  echo "GATE_VERDICT GREEN all rc as stated"
else
  echo "GATE_VERDICT VOID_STOP rc dag=$G1 rust=$G2 dag2=$G3 rust2=$G4 ctrl_dag=$C1 ctrl_rust=$C2 name=$N1"
fi
