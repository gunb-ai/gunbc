#!/usr/bin/env bash
# Enable sccache client for CI compile jobs without stopping a live shared server.
#
# Fleet crisis directive (2026-06-15): do NOT `sccache --stop-server` at job start —
# that kills in-flight compiles across concurrent jobs on srv1/srv2. Only attach the
# client when the runner-provisioned socket is healthy; start-server is idempotent.
#
# Usage: source via `bash .github/ci-floor/enable-sccache-client.sh` (writes GITHUB_ENV).

set -euo pipefail

echo "CARGO_HOME=$RUNNER_TEMP/cargo" >>"$GITHUB_ENV"
echo "RUSTUP_HOME=$RUNNER_TEMP/rustup" >>"$GITHUB_ENV"
echo "CARGO_BUILD_JOBS=2" >>"$GITHUB_ENV"

if [ -n "${SCCACHE_SERVER_UDS:-}" ] && [ -w "$(dirname "$SCCACHE_SERVER_UDS")" ] \
  && sccache --start-server >/dev/null 2>&1 \
  && sccache --show-stats >/dev/null 2>&1; then
  echo "RUSTC_WRAPPER=sccache" >>"$GITHUB_ENV"
  echo "CARGO_INCREMENTAL=0" >>"$GITHUB_ENV"
else
  echo "::warning::sccache server (SCCACHE_SERVER_UDS=${SCCACHE_SERVER_UDS:-unset}) absent, unwritable, or unreachable; building cold"
fi
