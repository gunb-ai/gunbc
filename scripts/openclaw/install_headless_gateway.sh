#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

LABEL="${OPENCLAW_GATEWAY_LABEL:-ai.openclaw.gateway.headless}"
USERNAME="${OPENCLAW_GATEWAY_USERNAME:-$(id -un)}"
USER_ID="${OPENCLAW_GATEWAY_UID:-$(id -u)}"
HOME_DIR="${OPENCLAW_GATEWAY_HOME:-$HOME}"
WORKDIR="${OPENCLAW_GATEWAY_WORKDIR:-$REPO_ROOT}"
OPENCLAW_BIN="${OPENCLAW_GATEWAY_BIN:-$(command -v openclaw)}"
LOG_DIR="${OPENCLAW_GATEWAY_LOG_DIR:-$HOME_DIR/.openclaw/logs}"
GENERATED_DIR="${OPENCLAW_GATEWAY_GENERATED_DIR:-$REPO_ROOT/target/openclaw/launchd}"
GENERATED_PLIST="${GENERATED_DIR}/${LABEL}.plist"
SYSTEM_PLIST="/Library/LaunchDaemons/${LABEL}.plist"

mkdir -p "$LOG_DIR" "$GENERATED_DIR"

python3 "${SCRIPT_DIR}/render_headless_launchdaemon.py" \
  --label "$LABEL" \
  --openclaw-bin "$OPENCLAW_BIN" \
  --username "$USERNAME" \
  --home "$HOME_DIR" \
  --workdir "$WORKDIR" \
  --stdout-path "${LOG_DIR}/gateway-daemon.out.log" \
  --stderr-path "${LOG_DIR}/gateway-daemon.err.log" \
  --output "$GENERATED_PLIST" >/dev/null

plutil -lint "$GENERATED_PLIST"

echo "Installing ${LABEL} to ${SYSTEM_PLIST}"
sudo install -o root -g wheel -m 0644 "$GENERATED_PLIST" "$SYSTEM_PLIST"

sudo launchctl bootout "system/${LABEL}" >/dev/null 2>&1 || true
sudo launchctl bootstrap system "$SYSTEM_PLIST"
sudo launchctl enable "system/${LABEL}"
sudo launchctl kickstart -k "system/${LABEL}"

sleep 2

echo
echo "LaunchDaemon installed."
echo "Label: ${LABEL}"
echo "User: ${USERNAME} (${USER_ID})"
echo "Binary: ${OPENCLAW_BIN}"
echo "Workdir: ${WORKDIR}"
echo "Logs:"
echo "  ${LOG_DIR}/gateway-daemon.out.log"
echo "  ${LOG_DIR}/gateway-daemon.err.log"
echo
openclaw health
