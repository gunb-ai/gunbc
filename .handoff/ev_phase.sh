#!/bin/bash
# evidence phase runner. $1 = AFTER | BEFORE | INSTR
# One ctrl-build dispatch per phase; every dispatch syncs the same PUSHED head, so the tree is
# identical across phases. BEFORE state is produced from the committed .evidence/ fixtures (the
# identity-head file states), never from the runner's stale .git.
set -uo pipefail
PHASE="${1:?AFTER|BEFORE|INSTR}"
MT=$(awk '/^MemTotal/{print $2}' /proc/meminfo)
NC=$(nproc 2>/dev/null || echo "?")
echo "GUARD MemTotal_kb=$MT NPROC=$NC"
if [ "${MT:-0}" -lt 41943040 ]; then echo "REFUSE_SMALL_RUNNER"; exit 9; fi
if [ "${NC:-0}" -lt 8 ]; then echo "REFUSE_SMALL_CPU"; exit 9; fi
free -k | awk '/^Mem/{print "MEMKB",$2}'
uname -a
# The reader-touched files come from the committed manifest, not a list maintained here: if the reader
# commit gains a file, the BEFORE state cannot silently forget to revert it.
test -f .evidence/MANIFEST.txt || { echo NO_MANIFEST; exit 4; }
read -r -a FILEARR < <(awk '{print $1}' .evidence/MANIFEST.txt)
FILES="${FILEARR[*]}"
echo "FILES=$FILES"
while read -r f after before; do
  h=$(md5sum < "$f" 2>/dev/null | cut -d' ' -f1)
  if [ "$h" != "$after" ]; then echo "TREE_MISMATCH $f got=$h want=$after"; exit 6; fi
done < .evidence/MANIFEST.txt
echo "TREE_INTEGRITY_OK"
echo "=== sync sanity ==="
echo "HEAD=$(git rev-parse HEAD)"
for f in src/v2/test/claim/optional_match_denotation.dag src/v2/std/qualified_name.dag .evidence/src_v2_compiler_infer.dag; do
  test -f "$f" || { echo "MISSING_REQUIRED $f"; exit 4; }
done
test -x ./target/release/gunbc || { echo NO_GUNBC_YET; exit 4; }
export GITHUB_ACTIONS=true
GUNBC=./target/release/gunbc

ts() { echo "@@@ $* ($(date -u +%H:%M:%S))"; }

ts "rustup i686"
rustup target add i686-unknown-linux-gnu >/dev/null 2>&1 || true

ts "build gunbc"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
test -x ./target/release/gunbc || { echo "BUILD_MISSING_GUNBC"; exit 5; }
./target/release/gunbc --version 2>&1 | head -1
ts "build done"

mkdir -p /tmp/head_state
for f in $FILES; do cp "$f" "/tmp/head_state/$(basename $f)"; done

set_state() { # $1 = BEFORE|AFTER
  if [ "$1" = BEFORE ]; then
    while read -r f after before; do
      cp ".evidence/$(echo "$f" | tr / _)" "$f"
      h=$(md5sum < "$f" | cut -d' ' -f1)
      [ "$h" = "$before" ] || echo "BEFORE_FIXTURE_MISMATCH $f got=$h want=$before"
    done < .evidence/MANIFEST.txt
  else
    for f in $FILES; do cp "/tmp/head_state/$(basename $f)" "$f"; done
  fi
  echo "STATE $1 reader-calls-in-infer=$(grep -c 'declaration_denotation_of_type(t:' src/v2/compiler/infer.dag) hand-reads-in-infer=$(grep -c 'declaration_reference_path_optional(node:' src/v2/compiler/infer.dag) denotation-fn-in-qn=$(grep -c 'fn declaration_denotation_of_type' src/v2/std/qualified_name.dag)"
}

claimrows() { # $1=tag $2=file
  echo "### CLAIM $1 $2"
  local t0=$(date +%s)
  timeout 1500 $GUNBC run --source-root src/v2 --source-root dag --entry "$2" --claim-run > "/tmp/${1}_${2//\//_}.log" 2>&1
  local rc=$?
  echo "CLAIM $1 exit=$rc elapsed=$(( $(date +%s) - t0 ))s"
  grep -E '^(PASS|FAIL|SKIP)' "/tmp/${1}_${2//\//_}.log" | sed "s/^/${1} /" | head -25
  echo "${1} TAIL:"; tail -6 "/tmp/${1}_${2//\//_}.log" | sed "s/^/${1} | /"
}

probe() { # $1=tag $2=entry
  local t0=$(date +%s)
  timeout 240 $GUNBC run --source-root src/v2 --source-root dag --entry "$2" --function zz_no_such_fn > /tmp/probe_${1}.log 2>&1
  local rc=$?
  local S
  if grep -q "NoSuchFunction" /tmp/probe_${1}.log; then S=RESOLVED; else S=NOT-RESOLVED; fi
  echo "PROBE $1 rc=$rc $S $2 ($(( $(date +%s) - t0 ))s)"
}

ENTRIES="dag/gunbc/auth/approval_device_enrolment_code_issue.dag dag/gunbc/auth/approval_keyring_converge.dag dag/gunbc/auth/branch_run_grant_await.dag dag/gunbc/auth/branch_run_grant_broker.dag dag/gunbc/auth/branch_run_grant.dag dag/gunbc/auth/ci_app_key_rotation.dag dag/gunbc/auth/gcp_iam_converge_run.dag dag/gunbc/cloudflare/r2_bucket_ensure.dag dag/gunbc/cloudflare/r2_conditional_put_race_probe.dag dag/gunbc/cloudflare/r2_permission_group_observe.dag dag/gunbc/cloudflare/r2_token_mint_run.dag dag/gunbc/fabric/fabric_writer_identity_observe.dag dag/gunbc/fabric/workspace_checkpoint_instrument.dag dag/gunbc/fabric/workspace_source.dag dag/gunbc/fabric/workspace_transcript_rate.dag dag/gunbc/fleet/app_control_plane_inspection.dag dag/gunbc/fleet/fleet_converge_plan_cli.dag dag/gunbc/fleet/fleet_multi_principal_probe.dag dag/gunbc/fleet/fleet_wireless_link_converge.dag dag/gunbc/fleet/hetzner_cloud_server_control.dag dag/gunbc/fleet/host_credential_custody_converge.dag dag/gunbc/fleet/org_actions_inspection.dag dag/gunbc/fleet/organization_runner_roster_read.dag dag/gunbc/fleet/printer_workflow_run.dag dag/gunbc/fleet/site_pxe_edge_converge.dag dag/gunbc/heal_candidate_stage0.dag dag/gunbc/heal_publication.dag dag/gunbc/host/host_memory_envelope_observe.dag dag/gunbc/host/host_reset_return_run.dag dag/gunbc/instruments/seed_build_artifacts_regen.dag dag/gunbc/instruments/tier_o_drift_check.dag dag/gunbc/live_deploy/apply.dag dag/gunbc/machine_intake/arrival_admission.dag dag/gunbc/machine_intake/arrival_run.dag dag/gunbc/machine_intake/mtcollins1_boot_admission.dag dag/gunbc/machine_intake/mtcollins1_boot_run.dag dag/gunbc/machine_intake/mtcollins1_census_image_publish.dag dag/gunbc/machine_intake/mtcollins1_census_member_readback.dag dag/gunbc/machine_intake/mtcollins1_census_qemu_host_observe.dag dag/gunbc/machine_intake/mtcollins1_census_qemu_toolchain_converge.dag dag/gunbc/machine_intake/mtcollins1_fan_observe.dag dag/gunbc/machine_intake/mtcollins1_kvm_observer_observe.dag dag/gunbc/machine_intake/mtcollins1_sol_notice.dag dag/gunbc/namecheap/observe.dag dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag dag/gunbc/runner/runner_browser_toolchain.dag dag/gunbc/runner/runner_group_restriction_ensure_run.dag dag/gunbc/runner/runner_guest_image.dag dag/gunbc/runner/runner_host_file_converge.dag dag/gunbc/runner/runner_microvm_boot_probe.dag dag/gunbc/runner/runner_microvm_host_ready.dag dag/gunbc/runner/runner_microvm_network_apply.dag dag/gunbc/runner/runner_microvm_network_observe.dag dag/gunbc/runner/runner_microvm_slot_reserve.dag dag/gunbc/runner/runner_microvm_slot_start.dag dag/gunbc/runner/runner_password_session_tool_converge.dag dag/gunbc/spark/managed_access_apply.dag dag/gunbc/spark/managed_grant_install.dag dag/gunbc/workspace_image_delivery.dag"

case "$PHASE" in
  AFTER)
    set_state AFTER
    ts "claims AFTER"
    claimrows AFTER src/v2/test/claim/optional_match_denotation.dag
    claimrows AFTER src/v2/test/claim/optional_declaration_identity.dag
    ts "probes AFTER"
    for e in $ENTRIES; do probe AFTER "$e"; done
    ;;
  BEFORE)
    set_state BEFORE
    ts "claims BEFORE"
    claimrows BEFORE src/v2/test/claim/optional_match_denotation.dag
    ts "probes BEFORE"
    for e in $ENTRIES; do probe BEFORE "$e"; done
    ;;
  INSTR)
    set_state AFTER
    ts "native-cli (v2.cli.compile_cli closure) AFTER"
    t0=$(date +%s)
    timeout 2400 $GUNBC test //gunbc/instruments:v2-native-cli > /tmp/native_cli.log 2>&1
    echo "NATIVE_CLI exit=$? elapsed=$(( $(date +%s) - t0 ))s"
    grep -nE 'compile_cli|PASS|FAIL|error|rows_|closures' /tmp/native_cli.log | head -30
    echo "NATIVE_CLI TAIL:"; tail -8 /tmp/native_cli.log
    ts "census AFTER"
    t0=$(date +%s)
    timeout 2400 $GUNBC test //gunbc/instruments:v2-native-census > /tmp/census_after.log 2>&1
    echo "CENSUS_AFTER exit=$? elapsed=$(( $(date +%s) - t0 ))s"
    grep -nE 'census_root|census_residual|file_refusal|cause_group|std/optional' /tmp/census_after.log | head -50
    ts "census BEFORE"
    set_state BEFORE
    t0=$(date +%s)
    timeout 2400 $GUNBC test //gunbc/instruments:v2-native-census > /tmp/census_before.log 2>&1
    echo "CENSUS_BEFORE exit=$? elapsed=$(( $(date +%s) - t0 ))s"
    grep -nE 'census_root|census_residual|file_refusal|cause_group|std/optional' /tmp/census_before.log | head -50
    set_state AFTER
    ;;
  *) echo "BAD_PHASE"; exit 2 ;;
esac

while read -r f after before; do cp "/tmp/head_state/$(basename $f)" "$f"
  h=$(md5sum < "$f" | cut -d' ' -f1); [ "$h" = "$after" ] || echo "RESTORE_MISMATCH $f"; done < .evidence/MANIFEST.txt
echo "RESTORE hand-reads-in-infer=$(grep -c 'declaration_reference_path_optional(node:' src/v2/compiler/infer.dag)"
echo "PHASE-$PHASE-DONE"
