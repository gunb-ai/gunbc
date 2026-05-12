#!/usr/bin/env bash
# Fail closed if `v3-compiler` gains a new top-level integration test binary
# (`tests/*.rs`) that is not wired into the split full-suite timings in
# `scripts/ci-binary-shim.sh` (each such target must run with `--report-time`
# so `scripts/check-test-timeout.sh` sees per-test lines).
#
# 🟡 BRIDGE — shell runner gate behind T-WAD BinaryShim. **Authority:** repo
# BinaryShim runner split-suite shape +
# `cargo metadata` as the mechanical source of truth for integration test targets.
# **Named trigger:** self-hosted v3 suite split for log/timing survival (#2681).
# **Dissolution:** fold this invariant into the Workflow-as-Data / single `.dag`
# CI surface so adding a gate edits one `.dag` file; delete this script and
# drop the BinaryShim runner call in the **same** commit.
#
# Uses Python's stdlib JSON parser only (no `jq`); the v3 CI job sets up
# Python before this step.
#
# `cargo metadata` output is materialized to a temp file (not process
# substitution) so `set -e` surfaces producer failures and we never read an
# empty target list as success (openai-pro review on #2681).
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "${repo_root}"

runner=scripts/ci-binary-shim.sh
if [[ ! -f "${runner}" ]]; then
  echo "::error::missing ${runner}"
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "::error::check-v3-full-suite-split-test-targets.sh requires python3 (stdlib json only)"
  exit 1
fi

metadata_json=$(mktemp)
names_out=$(mktemp)
trap 'rm -f "${metadata_json}" "${names_out}"' EXIT

cargo metadata --no-deps --format-version 1 >"${metadata_json}"

python3 -c '
import json, sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    data = json.load(handle)
names = []
for pkg in data.get("packages", []):
    if pkg.get("name") != "v3-compiler":
        continue
    for target in pkg.get("targets", []):
        if target.get("kind") == ["test"]:
            names.append(target["name"])
if not names:
    print("::error::no v3-compiler integration test targets in cargo metadata", file=sys.stderr)
    sys.exit(1)
for name in sorted(set(names)):
    print(name)
' "${metadata_json}" >"${names_out}"

fail=false
while IFS= read -r name; do
  [[ -z "${name}" ]] && continue
  # One line must invoke this test target with libtest timing output enabled.
  if ! grep -qE "cargo test -p v3-compiler --test ${name}[[:space:]].*--report-time" "${runner}"; then
    echo "::error::v3-compiler integration test target '${name}' has no split full-suite command with --report-time in ${runner}. Add a command (mirror determinism_test / integration) or fold the module into an existing tests/*.rs harness."
    fail=true
  fi
done <"${names_out}"

if [[ "${fail}" == true ]]; then
  exit 1
fi

echo "v3 full-suite split covers all cargo integration test targets: OK"
