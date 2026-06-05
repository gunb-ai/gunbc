#!/usr/bin/env bash
# .github/ci-floor/with-sccache-retry.sh
#
# Wrap a build command so a TRANSIENT sccache server-transport failure
# self-heals, WITHOUT masking real compile errors.
#
# The CI sccache server is lazily auto-started and shared across jobs on the
# host (ctrl#1419). The gate step in ci.yml health-probes + warms it before the
# build; this wrapper covers a server death DURING the build (idle-shutdown
# race, OOM, crash). When that happens the client aborts mid-handshake --
#   sccache: error: failed to execute compile
#   caused by: Failed to send data to or receive data from server
#   caused by: Failed to read response header / failed to fill whole buffer
# -- and the whole build fails exit 2 even though the code is fine (observed:
# gunbc ci.yml affected job, run 26985377093, 2026-06-04).
#
# Behaviour:
#   1. Run the command. On success, exit 0.
#   2. On failure, ONLY if captured stderr matches the sccache transport
#      signature: restart the server and retry (RETRIES attempts total).
#   3. If the signature persists after the last attempt, run ONCE more with
#      RUSTC_WRAPPER unset (cold -- uncached but correct).
#   4. A failure that is NOT the transport signature (a real compile error)
#      exits immediately with the original code: no retry, no masking.
#
# Usage: bash .github/ci-floor/with-sccache-retry.sh <cmd> [args...]
#
# Authority: companion to the ci.yml sccache gate health-probe; durable fix is
# the modeled cache_interface projection (src/v4/workflow/ci.dag). dissolve-on-arrival.

set -uo pipefail

# Only these stderr shapes are treated as transient transport faults. Kept
# narrow on purpose -- a broader match would retry (and mask) genuine errors.
SIG='failed to execute compile|send data to or receive data from server|read response header|failed to fill whole buffer|failed to connect to server'

RETRIES="${SCCACHE_RETRY_ATTEMPTS:-2}"

if [ "$#" -eq 0 ]; then
  echo "with-sccache-retry: no command given" >&2
  exit 2
fi

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

attempt=1
while [ "$attempt" -le "$RETRIES" ]; do
  rc=0
  # Capture stderr to inspect the failure shape; surface it afterward so the
  # step log still shows compiler output. stdout streams through untouched
  # (callers like detect-ci-affected-components write to $GITHUB_OUTPUT by path,
  # not via our stdout).
  "$@" 2>"$log" || rc=$?
  cat "$log" >&2
  if [ "$rc" -eq 0 ]; then
    exit 0
  fi
  if ! grep -qiE "$SIG" "$log"; then
    # Real failure -- fail fast with the original code.
    exit "$rc"
  fi
  echo "::warning::sccache transport failure (attempt ${attempt}/${RETRIES}, rc=${rc}); restarting server and retrying"
  sccache --stop-server >/dev/null 2>&1 || true
  sccache --start-server >/dev/null 2>&1 || true
  attempt=$((attempt + 1))
done

# Persistent transport failure: last-resort cold build -- correct, just uncached.
echo "::warning::sccache still unhealthy after ${RETRIES} attempts; building cold (RUSTC_WRAPPER unset)"
exec env RUSTC_WRAPPER= "$@"
