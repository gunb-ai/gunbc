#!/usr/bin/env bash
# Compatibility entrypoint for v2 stage0 regeneration.
set -euo pipefail
exec cargo run -p v2-compiler --bin regen_stage0 -- "$@"
