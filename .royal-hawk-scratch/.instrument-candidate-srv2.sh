set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
candidate_repo=/home/briansrls/royal-hawk-241/instrument.rWMaHh/repo
candidate_run=$(mktemp -d /home/briansrls/royal-hawk-241/instrument-candidate.XXXXXX)
cd "$candidate_repo"
test "$(git rev-parse HEAD)" = ef06471dc0e876d3220ee46a8fab874b9a044f0d
test -z "$(git status --porcelain --untracked-files=no)"
printf '%s\n' "$candidate_run"
# Generation-only: preserve candidate bytes even when drift returns a nonzero exit.
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$candidate_run" GITHUB_SHA=ef06471dc0e876d3220ee46a8fab874b9a044f0d CTRL_BUILD_MODE=local ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 --regen-candidate-dir "$candidate_run/candidate" --regen-receipt "$candidate_run/receipt.json" > "$candidate_run/generation.log" 2>&1 && candidate_status=0 || candidate_status=$?
printf 'CANDIDATE_DONE rc=%s root=%s\n' "$candidate_status" "$candidate_run"
for candidate_name in std_compiler_entry.rs v1_compiler_emit_rust.rs; do
  if [ -f "$candidate_run/candidate/src/$candidate_name" ]; then
    sha256sum "$candidate_run/candidate/src/$candidate_name"
  fi
done
exit "$candidate_status"
