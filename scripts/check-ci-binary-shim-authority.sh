#!/usr/bin/env bash
# Ratchet for T-WAD `ci_yml_hand_authority_dissolved`.
set -euo pipefail

workflow=.github/workflows/ci.yml

if [[ "$(head -n 1 "${workflow}")" != "# AUTO-GENERATED BinaryShim from dsl/gunbc/ci_emission.dag -- DO NOT EDIT." ]]; then
  echo "::error::${workflow} must be the generated BinaryShim artifact from dsl/gunbc/ci_emission.dag"
  exit 1
fi

if grep -nE 'cargo (test|clippy|fmt|run|build)|scripts/check-|git diff --name-only|grep -nE|tee /tmp/v3-test-timings' "${workflow}"; then
  echo "::error::durable CI policy found in ${workflow}; put execution policy behind scripts/ci-binary-shim.sh / ci_emission.dag"
  exit 1
fi

if ! grep -q 'bash scripts/ci-binary-shim.sh' "${workflow}"; then
  echo "::error::${workflow} does not invoke the BinaryShim runtime"
  exit 1
fi

echo "ci BinaryShim authority: OK"
