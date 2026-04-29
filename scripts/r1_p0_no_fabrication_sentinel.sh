#!/usr/bin/env bash
# R1C-B / T-P0 — host receipt for `p0_no_fabrication_sentinel`.
# Mirrors `src/v2/tests/src/bug_sentinel_ratchet.rs` (no `__BUG_NO_PROFILE_` substring).
set -euo pipefail
SENTINEL="__BUG_NO_PROFILE_"
root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "${root}" ]]; then
  echo "r1_p0_no_fabrication_sentinel: not inside a git work tree" >&2
  exit 1
fi
for f in "${root}/dsl/std/types.dag" "${root}/src/v2/tests/src/infer_semantics.rs"; do
  if [[ ! -f "${f}" ]]; then
    echo "r1_p0_no_fabrication_sentinel: missing ${f}" >&2
    exit 1
  fi
  if grep -qF "${SENTINEL}" "${f}"; then
    echo "r1_p0_no_fabrication_sentinel: forbidden substring in ${f}" >&2
    exit 1
  fi
done
exit 0
