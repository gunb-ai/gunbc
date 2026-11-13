#!/bin/bash
# Build the seed, build the v2-native CLI (gen-one), then run v2 `emit --entry` on named entries.
set -u
cd "$(dirname "$0")/.."
export RUNNER_TEMP=$PWD/.measure/rt; mkdir -p $RUNNER_TEMP
echo "== seed build $(date +%T)"
cargo build --release -p v1-compiler --bin gunbc > .measure/build.log 2>&1; echo "BUILD_EXIT=$? $(date +%T)"; tail -3 .measure/build.log
echo "== v2-native-cli $(date +%T)"
/usr/bin/time -v target/release/gunbc test //gunbc/instruments:v2-native-cli > .measure/cli.log 2>&1; echo "CLI_EXIT=$? $(date +%T)"
grep -E 'wall_s|Maximum resident|Elapsed|REFUSAL' .measure/cli.log | head; tail -5 .measure/cli.log
G=$(ls $RUNNER_TEMP/v2-native-cli-gen-one* 2>/dev/null | head -1); [ -z "$G" ] && G=$(find / -name 'v2-native-cli-gen-one*' -type f 2>/dev/null | head -1)
echo "GEN_ONE=$G"
for e in gunbc.witness.compiler_gate_workflow gunbc.heal_candidate_stage0 gunbc.instruments.generated_artifact_gate; do
  echo "==== $e $(date +%T)"
  /usr/bin/time -v "$G" emit --entry $e --source-root dag --source-root src/v2 > .measure/$e.out 2> .measure/$e.err; echo "EXIT=$?"
  grep -E 'Elapsed|Maximum resident' .measure/$e.err
  echo "stdout bytes: $(wc -c < .measure/$e.out)"
  echo "-- stderr head"; grep -v -E '^\s+(Command|User|System|Percent|Average|Exit|Socket|Signals|Swaps|File system|Page size|Voluntary|Involuntary|Minor|Major)' .measure/$e.err | head -c 6000
  echo; echo "-- stdout head"; head -c 4000 .measure/$e.out; echo
  echo "-- refusal tokens"; grep -o -E '[A-Za-z_]*[Uu]nbound[A-Za-z_]*|resolve_reason_[a-z_]+' .measure/$e.out .measure/$e.err | sort | uniq -c | sort -rn | head -20
done
echo ALL_DONE
