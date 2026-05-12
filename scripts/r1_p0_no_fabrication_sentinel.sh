#!/usr/bin/env bash
# R1C-B / T-P0 — host receipt for `p0_no_fabrication_sentinel`.
# Host-side ratchet: two load-bearing surfaces must stay free of `__BUG_NO_PROFILE_`
# (historically mirrored v2 `bug_sentinel_ratchet`; v2 tree removed under T-V2-Retirement G-2).
set -euo pipefail
SENTINEL="__BUG_NO_PROFILE_"
root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "${root}" ]]; then
  echo "r1_p0_no_fabrication_sentinel: not inside a git work tree" >&2
  exit 1
fi
for f in "${root}/dsl/std/types.dag" "${root}/src/v3/compiler/src/infer.rs"; do
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
