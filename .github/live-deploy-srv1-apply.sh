set -euo pipefail
# GENERATED APPLY by dag/gunbc/live_deploy/emit.dag host=srv1
# Principal: ROOT (actions-runner User=root). rsync/chown run bare and REQUIRE it;
# sudo -n stays on the historical grant-set ops (apt-get, install, systemctl, tailscale),
# redundant under root, load-bearing for a future unprivileged principal.
_gunbc_stage="$(mktemp -d)"
trap 'rm -rf "$_gunbc_stage"' EXIT
if ! dpkg -s tailscale >/dev/null 2>&1; then sudo -n apt-get install --yes tailscale; fi
if ! dpkg -s tmux >/dev/null 2>&1; then sudo -n apt-get install --yes tmux; fi
sudo -n install -d -m 0755 /opt/gunbc
rsync -a --delete --exclude /target --exclude /.git ./ /opt/gunbc/gunbc/
rsync -a ./.git/ /opt/gunbc/gunbc/.git/
chown -R briansrls:briansrls /opt/gunbc/gunbc
sudo -n install -d -m 0755 /opt/gunbc/bin
sudo -n install -m 0755 target/release/gunbc /opt/gunbc/bin/gunbc
chown -R briansrls:briansrls /opt/gunbc/bin
sudo -n install -d -m 0755 /opt/gunbc/dispatch-worktrees
chown -R briansrls:briansrls /opt/gunbc/dispatch-worktrees
cat > "$_gunbc_stage/gunbc-roadmap.service" <<'GUNBC_UNIT_EOF'
[Unit]
Description=gunbc roadmap HTTP server (gunbc serve)
After=network-online.target tailscaled.service
Wants=network-online.target

[Service]
Type=simple
User=briansrls
Group=briansrls
WorkingDirectory=/opt/gunbc/gunbc
ExecStart=/opt/gunbc/bin/gunbc serve --source-root dag --source-root src/v2 --entry dag/gunbc/roadmap_serve.dag --function roadmap_serve_handle --host 0.0.0.0 --port 8080
Restart=on-failure

[Install]
WantedBy=multi-user.target
GUNBC_UNIT_EOF
sudo -n install -m 0644 "$_gunbc_stage/gunbc-roadmap.service" /etc/systemd/system/gunbc-roadmap.service
sudo -n systemctl daemon-reload
sudo -n systemctl enable gunbc-roadmap.service
sudo -n systemctl restart gunbc-roadmap.service
if ! sudo -n tailscale serve status 2>/dev/null | grep -q ':8080'; then sudo -n tailscale serve --bg 8080; fi
echo live-deploy-receipt host=srv1 fold=apply verdict=converged