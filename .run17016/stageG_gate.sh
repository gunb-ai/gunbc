#!/usr/bin/env bash
# STAGE G: compile gate, both targets, both entries, control, name-refusal. Head-built binary.
# All invocations run FROM the head worktree (workspace_root rule, cli_run.rs:570).
. ./stage_common.sh
HEAD="$1"; T="head-$HEAD"; B="$PWD/$T/target/release/gunbc"
test -x "$B" || { echo "NO_BINARY $B"; exit 9; }
echo "BINARY_PROVENANCE=head-built manifest-unmatched sha256=$(sha256sum "$B" | awk "{print \$1}") built_from=$(git -C "$T" rev-parse HEAD)"
# B-amended freshness DIAGNOSTICS (authority remains gunbc test refusal; these print, they do not gate):
echo "DIAG_FINGERPRINT_HITS=$(find "$(dirname "$B")/.fingerprint" -name "dep-bin-gunbc*" 2>/dev/null | wc -l) (expect >=1, under v1-compiler-<hash>/)"; find "$(dirname "$B")/.fingerprint" -name "dep-bin-gunbc*" -print 2>/dev/null | head -4
DEPS_DIR="$(dirname "$B")/../release/deps"; TWIN=$(ls "$DEPS_DIR"/gunbc-* 2>/dev/null | head -1); echo "DIAG_TWIN_MAIN=$(stat -c "%i %h" "$B" 2>/dev/null | tr " " ":") DIAG_TWIN_PATH=$TWIN DIAG_TWIN_SAME=$(stat -c "%i %h" "$TWIN" 2>/dev/null | tr " " ":") (expect equal inodes, link count 2)"
echo "DIAG_BUILD_ROOT_NOTE=dep-info build_root corroborated per-claim by the [workspace-root] discovered line in each gate log"
echo "GATE1MIN=$(awk "{print \$1}" /proc/loadavg)"
cd "$T" || exit 9
"$B" compile --help > ../gate_help.log 2>&1; CH=$?
echo "COMPILE_HELP_RC=$CH (manager rule: same receipt as stageG; a flag change fails as a named route failure)"
"$B" compile --entry dag/test/claim/filesystem_effect_reach_wall.dag --target dag  --source-root dag --source-root src/v2 --output-dir ../gate_dag  > ../gate.dag.log  2>&1; G1=$?
echo GATE_RC_DAG=$G1; grep -m1 -E "blocking error" ../gate.dag.log
"$B" compile --entry dag/test/claim/filesystem_effect_reach_wall.dag --target rust --source-root dag --source-root src/v2 --output-dir ../gate_rust > ../gate.rust.log 2>&1; G2=$?
echo GATE_RC_RUST=$G2; grep -m1 -E "blocking error" ../gate.rust.log
"$B" compile --entry dag/test/claim/a_payload_surplus_variant_wall.dag --target dag  --source-root dag --source-root src/v2 --output-dir ../gate2_dag  > ../gate2.dag.log  2>&1; G3=$?
echo GATE2_RC_DAG=$G3; grep -m1 -E "blocking error" ../gate2.dag.log
"$B" compile --entry dag/test/claim/a_payload_surplus_variant_wall.dag --target rust --source-root dag --source-root src/v2 --output-dir ../gate2_rust > ../gate2.rust.log 2>&1; G4=$?
echo GATE2_RC_RUST=$G4; grep -m1 -E "blocking error" ../gate2.rust.log
"$B" compile --entry dag/test/claim/occurrence_binding_resolve_witness.dag --target dag  --source-root dag --source-root src/v2 --output-dir ../ctrl_dag  > ../ctrl.dag.log  2>&1; C1=$?
echo CONTROL_RC_DAG=$C1; grep -m1 -E "blocking error" ../ctrl.dag.log
"$B" compile --entry dag/test/claim/occurrence_binding_resolve_witness.dag --target rust --source-root dag --source-root src/v2 --output-dir ../ctrl_rust > ../ctrl.rust.log 2>&1; C2=$?
echo CONTROL_RC_RUST=$C2; grep -m1 -E "blocking error" ../ctrl_rust.log
"$B" test "//dag/test/claim/filesystem_effect_reach_wall:definitely_not_a_real_fn_zzz" --source-root dag --source-root src/v2 > ../gate_name.log 2>&1; N1=$?
echo "NAME_RC=$N1 (expect 2)"
if grep -qi panic ../gate_name.log ../gate.dag.log ../gate.rust.log ../gate2.dag.log ../gate2.rust.log ../ctrl.dag.log ../ctrl_rust.log; then echo GATE_VERDICT VOID_STOP_PANIC; cd ..; exit 6; fi
[ "$N1" -eq 2 ] && grep -m2 -iE "refus|unknown|not found|no such" ../gate_name.log
if [ "$CH" -eq 0 ] && [ "$G1" -eq 0 ] && [ "$G2" -eq 0 ] && [ "$G3" -eq 0 ] && [ "$G4" -eq 0 ] && [ "$C1" -eq 0 ] && [ "$C2" -eq 0 ] && [ "$N1" -eq 2 ]; then echo "GATE_VERDICT GREEN all rc as stated"; else echo "GATE_VERDICT VOID_STOP rc help=$CH dag=$G1 rust=$G2 dag2=$G3 rust2=$G4 ctrl_dag=$C1 ctrl_rust=$C2 name=$N1"; cd ..; exit 6; fi
cd .. || exit 9
