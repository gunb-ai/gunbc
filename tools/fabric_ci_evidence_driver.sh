#!/usr/bin/env bash
# Shared wet-gate driver: status and exact named value files only; diagnostics are never data.
# SCAFFOLD — dissolve-on: modeled lifecycle actuation sufficient to sequence a wet gate from .dag.
fabric_ci_driver_init() {
  : "${FABRIC_CI_GUNBC_BIN:?}"; : "${FABRIC_CI_SOURCE_ROOT:?}"; : "${FABRIC_CI_ENTRY:?}"
  : "${FABRIC_CI_LOG:?}"; : "${FABRIC_CI_VALUE_ROOT:?}"
}
fabric_ci_run_assertion() {
  local function_name=$1; shift
  "$FABRIC_CI_GUNBC_BIN" run --source-root "$FABRIC_CI_SOURCE_ROOT/dag" --source-root "$FABRIC_CI_SOURCE_ROOT/src/v2" --entry "$FABRIC_CI_ENTRY" --function "$function_name" "$@" 2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr
}
fabric_ci_observe_status() {
  local expected_status=$1; shift; local status
  if "$@" 2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr; then status=0; else status=$?; fi
  [[ $status -eq $expected_status ]]
}
fabric_ci_capture_transport() {
  local function_name=$1 coordinate=$2; shift 2
  [[ $coordinate =~ ^[a-z0-9-]+$ ]] || return 1
  local value_path="$FABRIC_CI_VALUE_ROOT/$coordinate.wire" value
  rm -f -- "$value_path"
  fabric_ci_run_assertion "$function_name" --arg path="$value_path" "$@" >/dev/null || return 1
  [[ -f $value_path && ! -L $value_path ]] || return 1
  value=$(<"$value_path")
  [[ $value =~ ^FCIE0X[0-9A-F]+$ ]] || return 1
  printf '%s' "$value"
}
