#!/usr/bin/env bash
set -euo pipefail
# This realizes the ordered typed rows emitted by fabric_ci_calibration_write_plan.
# SCAFFOLD — dissolve-on: modeled lifecycle actuation sufficient to sequence a wet gate from .dag.
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1 || { echo 'CalibrationRefused: derived repo_root is not a git checkout' >&2; exit 2; }
cd "$repo_root" || { echo 'CalibrationRefused: cannot enter derived repo_root' >&2; exit 2; }
export FABRIC_CI_GUNBC_BIN=${FABRIC_CI_GUNBC_BIN:-"$repo_root/target/release/gunbc"}
export FABRIC_CI_SOURCE_ROOT=$repo_root FABRIC_CI_ENTRY="$repo_root/dag/gunbc/instruments/fabric_ci_evidence.dag"
export FABRIC_CI_LOG FABRIC_CI_VALUE_ROOT
FABRIC_CI_LOG=$(mktemp); FABRIC_CI_VALUE_ROOT=$(mktemp -d); snapshot=$(mktemp)
mutation_subject= mutation_snapshot=
plan="$FABRIC_CI_VALUE_ROOT/calibration.plan"
cp "$FABRIC_CI_ENTRY" "$snapshot"
cleanup() {
  cp "$snapshot" "$FABRIC_CI_ENTRY"
  if [[ -n $mutation_subject && -n $mutation_snapshot && -f $mutation_snapshot ]]; then cp "$mutation_snapshot" "$mutation_subject"; fi
  rm -f "$snapshot" "$FABRIC_CI_LOG" ${mutation_snapshot:+"$mutation_snapshot"}
  rm -rf "$FABRIC_CI_VALUE_ROOT"
}
trap cleanup EXIT
source "$repo_root/tools/fabric_ci_evidence_driver.sh"
fabric_ci_driver_init
[[ -x $FABRIC_CI_GUNBC_BIN ]] || exit 2
sha256sum "$FABRIC_CI_ENTRY"; cmp -s "$snapshot" "$FABRIC_CI_ENTRY"
fabric_ci_run_assertion fabric_ci_calibration_write_plan --arg path="$plan"
[[ -f $plan && ! -L $plan ]]

while IFS='|' read -r operation a b c; do
  case "$operation" in
    expect-status)
      fabric_ci_observe_status "$b" "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" --entry "$FABRIC_CI_ENTRY" --function "$a"
      ;;
    transport-assert)
      wire=$(fabric_ci_capture_transport "$a" "$b")
      fabric_ci_run_assertion "$c" --arg wire="$wire"
      ;;
    transport-cross-red)
      wire=$(fabric_ci_capture_transport "$a" "$b")
      if fabric_ci_run_assertion "$c" --arg wire="$wire"; then exit 1; fi
      ;;
    diagnostic-not-channel)
      if fabric_ci_capture_transport "$a" "$b" >/dev/null; then exit 1; fi
      ;;
    altered-byte-red)
      wire=$(fabric_ci_capture_transport "$a" "$b")
      last=${wire: -1}; [[ $last == 0 ]] && replacement=1 || replacement=0
      if fabric_ci_run_assertion "$c" --arg wire="${wire::-1}$replacement"; then exit 1; fi
      ;;
    assertion-red)
      if fabric_ci_run_assertion "$a"; then exit 1; fi
      ;;
    write-failure-red)
      if fabric_ci_run_assertion "$a" --arg path="$b"; then exit 1; fi
      ;;
    ignored-write-red)
      if fabric_ci_capture_transport "$a" "$b" >/dev/null; then exit 1; fi
      ;;
    delete-projection-arm-red)
      mutation_subject="$repo_root/$a"
      mutation_snapshot=$(mktemp)
      zero_match_subject=$(mktemp)
      two_match_subject=$(mktemp)
      cp "$mutation_subject" "$mutation_snapshot"
      selector_occurs_exactly_once() { [[ $(grep -Fc "$b" "$1") -eq 1 ]]; }
      selector_occurs_exactly_once "$mutation_subject"
      sed "\|$b|d" "$mutation_subject" > "$zero_match_subject"
      if selector_occurs_exactly_once "$zero_match_subject"; then exit 1; fi
      cp "$mutation_subject" "$two_match_subject"
      printf '%s\n' "$b" >> "$two_match_subject"
      if selector_occurs_exactly_once "$two_match_subject"; then exit 1; fi
      sed -i "\|$b|d" "$mutation_subject"
      if "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_assertion_success 2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr; then exit 1; fi
      cp "$mutation_snapshot" "$mutation_subject"; cmp -s "$mutation_snapshot" "$mutation_subject"
      rm -f "$mutation_snapshot" "$zero_match_subject" "$two_match_subject"
      mutation_subject= mutation_snapshot=
      ;;
    final-assertion)
      fabric_ci_run_assertion "$a"
      ;;
    *) echo "FCI-EVIDENCE-0 unknown modeled calibration operation: $operation" >&2; exit 2 ;;
  esac
done < "$plan"
echo 'FCI-EVIDENCE-0 calibration held'
