set -euo pipefail
# GENERATED RETRACT by dag/gunbc/live_deploy/emit.dag host=srv1
# owned-artifacts only — ensured deps (tailscale, tmux) intentionally NOT purged
# Principal: ROOT. dispatch-worktrees teardown is rmdir: live worktrees fail it LOUDLY.
sudo -n tailscale serve reset || true
sudo -n systemctl disable --now gunbc-roadmap.service 2>/dev/null || true
sudo -n rm -f /etc/systemd/system/gunbc-roadmap.service
sudo -n systemctl daemon-reload
rmdir /opt/gunbc/dispatch-worktrees
sudo -n rm -f /opt/gunbc/bin/gunbc
rm -rf /opt/gunbc/gunbc
echo live-deploy-receipt host=srv1 fold=retract verdict=converged