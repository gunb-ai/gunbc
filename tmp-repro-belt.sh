#!/bin/bash
set -u
BIN=./target/release/gunbc
if [ ! -x "$BIN" ]; then
  cargo build --release -p v1-compiler
fi

# Control that MUST fail: missing --output-dir prints usage and must not look like a clean compile.
"$BIN" compile --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --target rust > /tmp/ctrl-no-outdir.txt 2>&1
echo CTRL_NO_OUTDIR_EXIT:$?
if rg -q "required arguments were not provided|--output-dir" /tmp/ctrl-no-outdir.txt; then
  echo CTRL_NO_OUTDIR: usage-refusal-ok
else
  echo CTRL_NO_OUTDIR: UNEXPECTED
  cat /tmp/ctrl-no-outdir.txt
  exit 1
fi

mkdir -p /tmp/gunbc-out-gate /tmp/gunbc-out-own
"$BIN" compile --output-dir /tmp/gunbc-out-own --source-root dag --source-root src/v2 --entry dag/gunbc/roadmap/roadmap_belt_actuate.dag --target rust > /tmp/belt-own.txt 2>&1
echo OWN_EXIT:$?
rg -n "Primitive.Observed|if branches resolve" /tmp/belt-own.txt || echo OWN_NO_SPECIMEN
tail -3 /tmp/belt-own.txt

"$BIN" compile --output-dir /tmp/gunbc-out-gate --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --target rust > /tmp/belt-gate.txt 2>&1
echo GATE_EXIT:$?
rg -n "Primitive.Observed|if branches resolve|roadmap_belt_actuate:3532" /tmp/belt-gate.txt || echo GATE_NO_SPECIMEN
tail -5 /tmp/belt-gate.txt
