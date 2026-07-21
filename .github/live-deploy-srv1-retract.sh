set -euo pipefail
# GENERATED RETRACT by dag/gunbc/live_deploy/emit.dag host=srv1
# owned-artifacts only — ensured deps (tailscale, tmux) intentionally NOT purged
# Principal: UNPRIVILEGED ci-runner. dispatch-worktrees teardown is rm -d (granted rm,
# empty-dir only): live worktrees fail it LOUDLY — the same empty-dir refusal wall.
sudo -n tailscale serve reset || true
sudo -n systemctl disable --now gunbc-roadmap.service 2>/dev/null || true
sudo -n rm -f /etc/systemd/system/gunbc-roadmap.service
sudo -n systemctl daemon-reload
sudo -n rm -d /opt/gunbc/dispatch-worktrees
sudo -n rm -f /opt/gunbc/bin/gunbc
sudo -n rm -f /etc/systemd/system/gunbc-tree-sync.service /etc/gunbc-tree-sync.env
sudo -n systemctl daemon-reload
sudo -n rm -rf /opt/gunbc/gunbc
echo live-deploy-receipt host=srv1 fold=retract verdict=converged