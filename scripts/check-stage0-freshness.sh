#!/usr/bin/env bash
# Historical v2 stage0 freshness gate — **retired** with `src/v2/stage0/`.
#
# Committed v3 compiler sources are guarded by `regen_bootstrap --verify` (CI)
# and SG-0 / regen driver tests — not by diffing against a `.dag`→stage0 pipe.
#
set -euo pipefail
echo "ERROR: scripts/check-stage0-freshness.sh is retired — no v2 stage0 directory." >&2
exit 1
