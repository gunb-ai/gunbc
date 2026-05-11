#!/usr/bin/env bash
# Fail closed if `v3-compiler` gains a new top-level integration test binary
# (`tests/*.rs`) that is not wired into the split full-suite timings in
# `.github/workflows/ci.yml` (each such target must run with `--report-time`
# so `scripts/check-test-timeout.sh` sees per-test lines).
#
# Uses Python's stdlib JSON parser only (no `jq`); the v3 CI job sets up
# Python before this step.
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "${repo_root}"

workflow=.github/workflows/ci.yml
if [[ ! -f "${workflow}" ]]; then
  echo "::error::missing ${workflow}"
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "::error::check-v3-full-suite-split-test-targets.sh requires python3 (stdlib json only)"
  exit 1
fi

fail=false
while IFS= read -r name; do
  [[ -z "${name}" ]] && continue
  # One line must invoke this test target with libtest timing output enabled.
  if ! grep -qE "cargo test -p v3-compiler --test ${name}[[:space:]].*--report-time" "${workflow}"; then
    echo "::error::v3-compiler integration test target '${name}' has no split full-suite step with --report-time in ${workflow}. Add a step (mirror determinism_test / integration) or fold the module into an existing tests/*.rs harness."
    fail=true
  fi
done < <(
  cargo metadata --no-deps --format-version 1 |
    python3 -c '
import json, sys
data = json.load(sys.stdin)
names = []
for pkg in data.get("packages", []):
    if pkg.get("name") != "v3-compiler":
        continue
    for target in pkg.get("targets", []):
        if target.get("kind") == ["test"]:
            names.append(target["name"])
for name in sorted(set(names)):
    print(name)
'
)

if [[ "${fail}" == true ]]; then
  exit 1
fi

echo "v3 full-suite split covers all cargo integration test targets: OK"
