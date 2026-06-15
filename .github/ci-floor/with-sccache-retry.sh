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
#   3. If transport retries exhaust and stderr shows fleet EAGAIN pressure,
#      one cold retry with CARGO_BUILD_JOBS=1 (observed layering_imports run
#      27579125729 / #4978: sccache transport + rustc ctrlc EAGAIN). Otherwise
#      fail loud on persistent transport failure.
#   4. A failure that is NOT the transport signature: if stderr matches cargo
#      EAGAIN thread-spawn pressure under fleet parallel compiles (observed
#      v4_lens_ci run 27576127936 / #4978), retry ONCE cold with
#      CARGO_BUILD_JOBS=1 and no RUSTC_WRAPPER. Otherwise fail fast.
#
# Usage: bash .github/ci-floor/with-sccache-retry.sh <cmd> [args...]
#
# Authority: companion to the ci.yml sccache gate health-probe; durable fix is
# the modeled cache_interface projection (dsl/std/cache_interface.dag). dissolve-on-arrival.

set -uo pipefail

# Only these stderr shapes are treated as transient transport faults. Kept
# narrow on purpose -- a broader match would retry (and mask) genuine errors.
SIG='failed to execute compile|send data to or receive data from server|read response header|failed to fill whole buffer|failed to connect to server'
EAGAIN_SIG='Resource temporarily unavailable|failed to spawn thread|Unable to install ctrlc handler'

RETRIES="${SCCACHE_RETRY_ATTEMPTS:-2}"

if [ "$#" -eq 0 ]; then
  echo "with-sccache-retry: no command given" >&2
  exit 2
fi

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

attempt=1
rc=0  # initialized so the final `exit $rc` is defined even if RETRIES=0 skips the loop (set -u)
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
    if grep -qiE "$EAGAIN_SIG" "$log"; then
      echo "::warning::cargo EAGAIN under fleet compile pressure; one cold retry with CARGO_BUILD_JOBS=1 (no sccache)"
      unset RUSTC_WRAPPER
      if CARGO_BUILD_JOBS=1 "$@"; then
        exit 0
      fi
    fi
    exit "$rc"
  fi
  echo "::warning::sccache transport failure (attempt ${attempt}/${RETRIES}, rc=${rc}); restarting server and retrying"
  # Fleet crisis (#4991): do NOT stop-server — kills in-flight compiles on shared runners.
  sccache --start-server >/dev/null 2>&1 || true
  attempt=$((attempt + 1))
done

# Persistent transport failure: cold-retry once when fleet EAGAIN is present,
# otherwise fail loud. rc holds the last attempt's exit code.
if grep -qiE "$EAGAIN_SIG" "$log"; then
  echo "::warning::fleet compile pressure (EAGAIN) after sccache transport retries; one cold retry with CARGO_BUILD_JOBS=1"
  unset RUSTC_WRAPPER
  if CARGO_BUILD_JOBS=1 "$@"; then
    exit 0
  fi
fi
echo "::error::sccache unreachable after ${RETRIES} attempts (transport failure persists); failing the build instead of building cold"
exit "$(( rc != 0 ? rc : 1 ))"
