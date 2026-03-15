#!/usr/bin/env bash
set -euo pipefail

LABEL="${OPENCLAW_GATEWAY_LABEL:-ai.openclaw.gateway.headless}"
SYSTEM_PLIST="/Library/LaunchDaemons/${LABEL}.plist"

sudo launchctl bootout "system/${LABEL}" >/dev/null 2>&1 || true
sudo launchctl disable "system/${LABEL}" >/dev/null 2>&1 || true
sudo rm -f "$SYSTEM_PLIST"

echo "Removed ${SYSTEM_PLIST}"
