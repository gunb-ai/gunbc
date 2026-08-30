#!/usr/bin/env bash

# Shared wet-gate run-boundary driver. This file is sourced by gate instruments.
# It sequences `gunbc run` and reads process status; it never decides a fabric-domain fact.
# SCAFFOLD — dissolve-on: first-class gunbc scalar-value transport plus modeled lifecycle actuation
# lets wet-gate orchestration be emitted from the .dag evidence contract, deleting this shell carrier.

fabric_ci_driver_init() {
  : "${FABRIC_CI_GUNBC_BIN:?FABRIC_CI_GUNBC_BIN must name the exact-tree compiler}"
  : "${FABRIC_CI_SOURCE_ROOT:?FABRIC_CI_SOURCE_ROOT must name the repository root}"
  : "${FABRIC_CI_ENTRY:?FABRIC_CI_ENTRY must name the .dag entry}"
  : "${FABRIC_CI_LOG:?FABRIC_CI_LOG must name the unfiltered log}"
}

fabric_ci_run_assertion() {
  local function_name=$1
  shift
  "$FABRIC_CI_GUNBC_BIN" run \
    --source-root "$FABRIC_CI_SOURCE_ROOT/dag" \
    --source-root "$FABRIC_CI_SOURCE_ROOT/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function "$function_name" "$@" \
    2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr
}

fabric_ci_observe_status() {
  local expected_status=$1
  shift
  local status
  if "$@" 2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr; then status=0; else status=$?; fi
  [[ $status -eq $expected_status ]]
}

# Transport entries intentionally return String, for which gunbc refuses status mapping with 2.
# Only the FCIE0X[0-9A-F]+ framed alphabet is extracted; arbitrary diagnostic payload cannot forge
# the interpreter-owned apostrophe/newline envelope. The returned bytes remain non-deciding until a
# ProcessExit assertion entry decodes and judges them.
fabric_ci_capture_transport() {
  local function_name=$1
  shift
  local raw status value
  if raw=$("$FABRIC_CI_GUNBC_BIN" run \
      --source-root "$FABRIC_CI_SOURCE_ROOT/dag" \
      --source-root "$FABRIC_CI_SOURCE_ROOT/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function "$function_name" "$@" \
      2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr); then
    status=0
  else
    status=$?
  fi
  if [[ $status -ne 2 || $raw != *' returned `FCIE0X'*'`, not `ProcessExit`.'* ]]; then
    echo "FCI-EVIDENCE-0 driver refused transport: expected framed String with status 2" >&2
    return 1
  fi
  value=FCIE0X${raw#*' returned `FCIE0X'}
  value=${value%%'`, not `ProcessExit`.'*}
  if [[ ! $value =~ ^FCIE0X[0-9A-F]+$ ]]; then
    echo "FCI-EVIDENCE-0 driver refused transport: wire escaped its canonical alphabet" >&2
    return 1
  fi
  printf '%s' "$value"
}
