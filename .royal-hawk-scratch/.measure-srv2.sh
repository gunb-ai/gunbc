set -euo pipefail
# Both revisions must include A's actual execution instrumentation; the first must precede B.
# This is the bounded MEASUREMENT SUBJECT, N=1, not full-population acceptance.
measurement_before=$1
measurement_after=$2
measurement_root=/home/briansrls/royal-hawk-241
measurement_recipe=$3
measurement_checker=$4
# The original recipe parked imports required by selected subjects. Refuse that known
# invalid population; use A's corrected, completed-run-verified recipe for both sides.
measurement_recipe_sha=$(sha256sum "$measurement_recipe")
case "$measurement_recipe_sha" in
  4b790d92469bae02b7295453e1d5385c1cf9115eba4d2e9896ceab32331fb9e7*)
    printf '%s\n' 'Refusing obsolete bounded recipe: selected subjects lose imported modules.' >&2
    exit 1
    ;;
esac
mkdir -p "$measurement_root"
measurement_pair=$(mktemp -d "$measurement_root/measurement-pair.XXXXXX")
for measurement_side in before after; do
  if [ "$measurement_side" = before ]; then
    measurement_revision=$measurement_before
  else
    measurement_revision=$measurement_after
  fi
  measurement_run="$measurement_pair/$measurement_side"
  mkdir -p "$measurement_run/runner-temp"
  git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$measurement_run/repo"
  git -C "$measurement_run/repo" fetch origin "$measurement_revision"
  git -C "$measurement_run/repo" checkout --detach "$measurement_revision"
  python3 "$measurement_recipe" "$measurement_run/repo" 200 > "$measurement_run/subject.txt"
  (
    cd "$measurement_run/repo"
    systemd-run --user --scope -p MemoryMax=24G env RUNNER_TEMP="$measurement_run/runner-temp" GITHUB_SHA="$measurement_revision" CTRL_BUILD_MODE=local bash -c '
      set -euo pipefail
      cargo build --release -p v1-compiler --bin claim_executor
      ./target/release/claim_executor --v2-native-route --source-root dag --source-root src/v2
    '
  ) > "$measurement_run/run.log" 2>&1 && measurement_status=0 || measurement_status=$?
  printf '%s\n' "$measurement_status" > "$measurement_run/process-status.txt"
  # A refused admission still carries completed measurements; absence of its marker refuses
  # in the evidence reader below. Preserve the actual status rather than treating it as a pass.
done
python3 "$measurement_checker" \
  "$measurement_pair/before/repo/target/v2-native-lane/driver-rows.jsonl" \
  "$measurement_pair/after/repo/target/v2-native-lane/driver-rows.jsonl" \
  "$measurement_pair/before/run.log" "$measurement_pair/after/run.log" \
  > "$measurement_pair/comparison.jsonl" 2>&1
printf '%s\n' "$measurement_pair"
