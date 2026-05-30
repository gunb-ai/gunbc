#!/usr/bin/env bash
# Hermetic smoke: pinned GUNBC_VERSION must reach install.sh (README contract).
#
# Regression: `GUNBC_VERSION=v0.1.0 curl ... | sh` assigns VERSION only to curl's
# environment, so the installer still follows the "latest" download path.
#
# Exit codes:
#   0 — pinned and latest paths behave as documented
#   1 — assertion failed

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAKE_BIN="$TMP/bin"
INSTALL_DIR="$TMP/install"
CURL_LOG="$TMP/curl.log"
mkdir -p "$FAKE_BIN" "$INSTALL_DIR"
: >"$CURL_LOG"

cat >"$FAKE_BIN/curl" <<'EOF'
#!/usr/bin/env sh
set -eu
log="${CURL_LOG:?CURL_LOG unset}"
printf 'curl %s\n' "$*" >>"$log"
out=""
url=""
while [ $# -gt 0 ]; do
  case "$1" in
    -o)
      out=$2
      shift 2
      ;;
    -fsSL)
      shift
      ;;
    -*)
      shift
      ;;
    *)
      url=$1
      shift
      ;;
  esac
done
if [ -z "$out" ] || [ -z "$url" ]; then
  echo "fake curl: expected -fsSL <url> -o <path>" >&2
  exit 1
fi
printf '#!/bin/sh\necho gunbc-smoke\n' >"$out"
chmod +x "$out"
EOF
chmod +x "$FAKE_BIN/curl"

run_install() {
  : >"$CURL_LOG"
  (
    cd "$ROOT"
    export PATH="$FAKE_BIN:$PATH"
    export CURL_LOG
    export GUNBC_INSTALL_DIR="$INSTALL_DIR"
    "$@"
  )
}

assert_log_contains() {
  local needle=$1
  if ! grep -Fq "$needle" "$CURL_LOG"; then
    echo "FAIL: curl log missing: $needle" >&2
    echo "--- curl log ---" >&2
    cat "$CURL_LOG" >&2
    return 1
  fi
}

assert_log_lacks() {
  local needle=$1
  if grep -Fq "$needle" "$CURL_LOG"; then
    echo "FAIL: curl log must not contain: $needle" >&2
    echo "--- curl log ---" >&2
    cat "$CURL_LOG" >&2
    return 1
  fi
}

# Documented pin: variable on the shell side of the pipe (README). Use `cat` instead of
# network curl so the smoke stays hermetic; env scoping matches `… | GUNBC_VERSION=… sh`.
run_install env -u GUNBC_VERSION sh -c 'cat ./install.sh | GUNBC_VERSION=v0.1.0 sh'
assert_log_contains 'releases/download/v0.1.0/gunbc-'
assert_log_lacks 'releases/latest/download/gunbc-'

# Direct invocation (install.sh header Usage).
run_install env GUNBC_VERSION=v0.1.0 sh ./install.sh
assert_log_contains 'releases/download/v0.1.0/gunbc-'
assert_log_lacks 'releases/latest/download/gunbc-'

# Latest when unset.
run_install env -u GUNBC_VERSION sh ./install.sh
assert_log_contains 'releases/latest/download/gunbc-'

echo "OK: install.sh pinned-version smoke passed"
