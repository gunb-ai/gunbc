set -euo pipefail
# GENERATED RETRACT by dag/gunbc/live_deploy/emit.dag host=srv1
# owned-artifacts only — ensured deps (tailscale, nodejs) intentionally NOT purged
# Runs UNPRIVILEGED; privileged ops via sudo -n <command> (tailscale, systemctl, rm).
sudo -n tailscale serve reset || true
sudo -n systemctl disable --now gunbc-roadmap.service 2>/dev/null || true
sudo -n rm -f /etc/systemd/system/gunbc-roadmap.service
sudo -n systemctl daemon-reload
sudo -n rm -f /opt/gunbc/server.js
echo live-deploy-receipt host=srv1 fold=retract verdict=converged