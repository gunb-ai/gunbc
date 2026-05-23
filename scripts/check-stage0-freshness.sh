#!/usr/bin/env bash
# Compatibility entrypoint for the v2 stage0 freshness gate.
set -euo pipefail
exec cargo run -p v2-compiler --bin regen_stage0 -- --verify "$@"
