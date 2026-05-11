#!/usr/bin/env bash
# Historical v2 stage0 regeneration — **retired** (T-V2-Retirement / `src/v2/` removed).
#
# Use v3 Pure-Bootstrap regeneration instead, e.g.:
#   cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap -- …
#   ./src/v3/compiler/src/bin/regen_*.rs drivers as documented in `docs/design-pure-bootstrap-zero.md`.
#
set -euo pipefail
echo "ERROR: scripts/regenerate-stage0.sh is retired — the v2 stage0 tree no longer exists." >&2
echo "See docs/design-pure-bootstrap-zero.md and v3-compiler regen bins." >&2
exit 1
