#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# T-38 TestClaim manual-corpus eval host transport (RR-A §5.2 / A.3b step-3).
# Invokes the stage0 `gunbc test` harness — no host-owned compile loop (#4091 §1.2).
# Authority: v4.workflow.ci `TestClaimCorpusEvalCommand` + `ci_upsert_testclaim_corpus_eval_*`.
#
# Env:
#   V2_COMPILER                      — gunbc binary (default: target/release/gunbc)
#   V4_TESTCLAIM_CORPUS_EVAL_STRICT  — if 1, exit non-zero when gunbc is missing (default: 0)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
strict="${V4_TESTCLAIM_CORPUS_EVAL_STRICT:-0}"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  if [[ "$strict" == "1" ]]; then
    echo "::error title=testclaim corpus eval setup::gunbc missing at $bin" >&2
    exit 2
  fi
  echo "::notice title=testclaim corpus eval::skipped — gunbc missing at $bin"
  exit 0
fi

exec "$bin" test
