set -euo pipefail
# GENERATED APPLY by dag/gunbc/live_deploy/emit.dag host=srv1
# Principal: UNPRIVILEGED ci-runner (fleet_posix_accounts FleetAccountCiRunner) — every host
# mutation rides its five sudo -n grants (apt-get, install, systemctl, tailscale, rm);
# the tree syncs AS the service user via gunbc-tree-sync.service (owner writes its own tree).
_gunbc_stage="$(mktemp -d)"
trap 'rm -rf "$_gunbc_stage"' EXIT
if ! dpkg -s tailscale >/dev/null 2>&1; then sudo -n apt-get install --yes tailscale; fi
if ! dpkg -s tmux >/dev/null 2>&1; then sudo -n apt-get install --yes tmux; fi
sudo -n install -d -m 0755 /opt/gunbc
sudo -n install -d -m 0755 -o briansrls -g briansrls /opt/gunbc/gunbc
printf 'GUNBC_TREE_SRC=%s\n' "$PWD" > "$_gunbc_stage/gunbc-tree-sync.env"
sudo -n install -m 0644 "$_gunbc_stage/gunbc-tree-sync.env" /etc/gunbc-tree-sync.env
cat > "$_gunbc_stage/gunbc-tree-sync.service" <<'GUNBC_TREE_SYNC_EOF'
[Unit]
Description=gunbc source tree sync (service-user-principal rsync from CI runner checkout)

[Service]
Type=oneshot
User=briansrls
Group=briansrls
EnvironmentFile=/etc/gunbc-tree-sync.env
ExecStart=/usr/bin/rsync -rlpt --delete --exclude /target --exclude /.git "${GUNBC_TREE_SRC}/" /opt/gunbc/gunbc/
ExecStart=/usr/bin/rsync -rlpt "${GUNBC_TREE_SRC}/.git/" /opt/gunbc/gunbc/.git/
GUNBC_TREE_SYNC_EOF
sudo -n install -m 0644 "$_gunbc_stage/gunbc-tree-sync.service" /etc/systemd/system/gunbc-tree-sync.service
sudo -n systemctl daemon-reload
sudo -n systemctl restart gunbc-tree-sync.service
sudo -n install -d -m 0755 /opt/gunbc/bin
sudo -n install -m 0755 -o briansrls -g briansrls target/release/gunbc /opt/gunbc/bin/gunbc
sudo -n install -d -m 0755 -o briansrls -g briansrls /opt/gunbc/dispatch-worktrees
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