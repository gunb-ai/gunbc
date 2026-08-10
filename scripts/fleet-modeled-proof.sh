#!/usr/bin/env bash
# Modeled DEVBOOT two-client fleet proof — host timer shell ONLY invokes devboot_serve_once_entry (no shell git ops).
set -euo pipefail
MAIN=69d07ed69d3
BRANCH=session/keen-moth-104
ENDPOINT="git://192.168.1.221:19419/store.git"
LOG="${LOG:-/tmp/fleet-modeled-proof.log}"

exec > >(tee -a "$LOG") 2>&1
echo "=== fleet modeled proof start $(date -Is) ==="

srv3_setup() {
  ssh briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 \"ssh -i ~/.node-orch/id_ed25519 ubuntu@192.168.1.221 bash -s\"" <<'REMOTE'
set -euo pipefail
MAIN=69d07ed69d3
BRANCH=session/keen-moth-104
source "$HOME/.cargo/env" 2>/dev/null || true

pkill -f 'serve-loop.sh' 2>/dev/null || true
pkill -f 'modeled-serve-loop.sh' 2>/dev/null || true
sleep 1

if [ ! -d /tmp/gunbc-8078/.git ]; then
  git clone --depth 50 https://github.com/gunb-ai/gunbc.git /tmp/gunbc-8078
fi
cd /tmp/gunbc-8078
git fetch origin "refs/heads/${BRANCH}:refs/remotes/origin/${BRANCH}" "refs/heads/main:refs/remotes/origin/main"
git checkout -B work "refs/remotes/origin/${BRANCH}"
cargo build -p v1-compiler --bin gunbc --release

ROOT=/tmp/devboot-fleet
mkdir -p "$ROOT"
if [ ! -d "$ROOT/gunbc/.git" ]; then
  git clone https://github.com/gunb-ai/gunbc.git "$ROOT/gunbc"
fi
cd "$ROOT/gunbc"
git fetch origin main
git checkout -f "$MAIN"
git clean -fdx -e target || true

echo "=== reset bare store ==="
rm -rf "$ROOT/store.git" "$ROOT/work"
mkdir -p "$ROOT/work"
git init --bare "$ROOT/store.git"
git --git-dir="$ROOT/store.git" config user.email "devboot@gunbc.local"
git --git-dir="$ROOT/store.git" config user.name "devboot"

if ! pgrep -f 'git-daemon.*19419' >/dev/null; then
  nohup git daemon --reuseaddr --base-path="$ROOT" --export-all \
    --enable=receive-pack --listen=0.0.0.0 --port=19419 \
    >>"$ROOT/daemon.log" 2>&1 &
  echo $! > "$ROOT/daemon.pid"
fi

cat > "$ROOT/modeled-serve-loop.sh" <<'LOOP'
#!/usr/bin/env bash
set -euo pipefail
ROOT=/tmp/devboot-fleet
STORE="$ROOT/store.git"
WORK="$ROOT/work"
CORPUS="$ROOT/gunbc"
BRANCH=/tmp/gunbc-8078
GUNBC="$BRANCH/target/release/gunbc"
source "$HOME/.cargo/env"
: > "$ROOT/modeled-serve.log"
while true; do
  RECEIPT="$WORK/serve-once.receipt"
  set +e
  cd "$BRANCH"
  "$GUNBC" run --source-root "$BRANCH/dag" --source-root "$BRANCH/src/v2" \
    --entry dag/gunbc/devboot/serve.dag \
    --function devboot_serve_once_entry \
    --arg store_root="$STORE" \
    --arg repo_root="$CORPUS" \
    --arg work_root="$WORK" \
    --arg cargo_bin="$HOME/.cargo/bin/cargo" \
    --arg rustup_home="$HOME/.rustup" \
    --arg cargo_home="$HOME/.cargo" \
    --arg build_jobs=4 \
    --arg receipt_path="$RECEIPT"
  ec=$?
  set -e
  body=$(tr '\n' ' ' < "$RECEIPT" 2>/dev/null || echo missing)
  echo "$(date -Is) exit=$ec $body" >> "$ROOT/modeled-serve.log"
  sleep 3
done
LOOP
chmod +x "$ROOT/modeled-serve-loop.sh"
: > "$ROOT/modeled-serve.log"
nohup "$ROOT/modeled-serve-loop.sh" >>"$ROOT/modeled-serve-loop.nohup" 2>&1 &

echo "srv3 ready branch=$(cd /tmp/gunbc-8078 && git rev-parse --short HEAD) main=$(cd /tmp/devboot-fleet/gunbc && git rev-parse --short HEAD) gunbc=$(/tmp/gunbc-8078/target/release/gunbc --version 2>&1 | head -1)"
REMOTE
}

run_client() {
  local host_label="$1"
  local ssh_cmd="$2"
  eval "$ssh_cmd" <<REMOTE
set -euo pipefail
MAIN=69d07ed69d3
BRANCH=session/keen-moth-104
ENDPOINT="$ENDPOINT"
source "\$HOME/.cargo/env" 2>/dev/null || true
if [ ! -d /tmp/gunbc-8078/.git ]; then
  git clone --depth 50 https://github.com/gunb-ai/gunbc.git /tmp/gunbc-8078
fi
cd /tmp/gunbc-8078
git fetch origin "refs/heads/\$BRANCH:refs/remotes/origin/\$BRANCH" "refs/heads/main:refs/remotes/origin/main"
git checkout -B work "refs/remotes/origin/\$BRANCH"
cargo build -p v1-compiler --bin gunbc --release
ROOT=/tmp/devboot-fleet
mkdir -p "\$ROOT"
if [ ! -d "\$ROOT/gunbc/.git" ]; then git clone https://github.com/gunb-ai/gunbc.git "\$ROOT/gunbc"; fi
cd "\$ROOT/gunbc"
git fetch origin main
git checkout -f "\$MAIN"
NAME="$host_label"
WORK="\$ROOT/work/\$NAME"
OUT="\$ROOT/out-\$NAME"
RECEIPT="\$ROOT/receipt-\$NAME.txt"
GUNBC=/tmp/gunbc-8078/target/release/gunbc
unset GIT_DIR GIT_WORK_TREE
rm -rf "\$WORK" "\$OUT" "\$RECEIPT" "\${OUT}.devboot-receipt"
mkdir -p "\$WORK"
cd /tmp/gunbc-8078
echo "START \$NAME \$(date -Is) corpus=\$(cd \$ROOT/gunbc && git rev-parse HEAD)"
set +e
DEVBOOT_ENDPOINT="\$ENDPOINT" "\$GUNBC" build gunbc --repo-root "\$ROOT/gunbc" --endpoint "\$ENDPOINT" --out "\$OUT"
ec=\$?
set -e
echo "EXIT=\$ec"
if [ -f "\${OUT}.devboot-receipt" ]; then cat "\${OUT}.devboot-receipt"; fi
if [ "\$ec" -eq 0 ]; then sha256sum "\$OUT"; "\$OUT" --help 2>&1 | head -2; fi
exit "\$ec"
REMOTE
}

srv3_setup
echo "=== srv1 client ==="
run_client srv1 "ssh briansrls@100.69.18.126 bash -s"
echo "=== srv2 client ==="
run_client srv2 "ssh briansrls@100.69.18.126 'ssh briansrls@100.73.169.40 bash -s'"

echo "=== srv3 producer log tail ==="
ssh briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 \"ssh -i ~/.node-orch/id_ed25519 ubuntu@192.168.1.221 'tail -15 /tmp/devboot-fleet/modeled-serve.log; echo ---; git --git-dir=/tmp/devboot-fleet/store.git for-each-ref refs/devboot/; echo ---; grep -c cargo /tmp/devboot-fleet/modeled-serve.log || true'\""

CAPTURE_ROOT="${CAPTURE_ROOT:-target/devboot-fleet-proof-observations}"
mkdir -p "$CAPTURE_ROOT"
echo "=== capture observations to ${CAPTURE_ROOT} ==="
ssh briansrls@100.69.18.126 "cat /tmp/devboot-fleet/out-srv1.devboot-receipt" >"${CAPTURE_ROOT}/srv1.devboot-receipt" 2>/dev/null || true
ssh briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 cat /tmp/devboot-fleet/out-srv2.devboot-receipt" >"${CAPTURE_ROOT}/srv2.devboot-receipt" 2>/dev/null || true
ssh briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 \"ssh -i ~/.node-orch/id_ed25519 ubuntu@192.168.1.221 cat /tmp/devboot-fleet/modeled-serve.log\"" >"${CAPTURE_ROOT}/modeled-serve.log" 2>/dev/null || true
ssh briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 \"ssh -i ~/.node-orch/id_ed25519 ubuntu@192.168.1.221 git --git-dir=/tmp/devboot-fleet/store.git for-each-ref refs/devboot/\"" >"${CAPTURE_ROOT}/store-for-each-ref.txt" 2>/dev/null || true
cat >"${CAPTURE_ROOT}/capture-provenance.txt" <<PROV
captured_at: $(date -Is)
capture_runner_host: $(hostname -f 2>/dev/null || hostname)
srv1_client_receipt_host: $(ssh -o ConnectTimeout=5 briansrls@100.69.18.126 hostname -f 2>/dev/null || echo srv1-unreachable)
srv2_client_receipt_host: $(ssh -o ConnectTimeout=5 briansrls@100.69.18.126 "ssh -o ConnectTimeout=5 briansrls@100.73.169.40 hostname -f" 2>/dev/null || echo srv2-unreachable)
modeled_serve_log_host: $(ssh -o ConnectTimeout=5 briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 ssh -i ~/.node-orch/id_ed25519 -o ConnectTimeout=5 ubuntu@192.168.1.221 hostname -f" 2>/dev/null || echo srv3-unreachable)
store_for_each_ref_host: $(ssh -o ConnectTimeout=5 briansrls@100.69.18.126 "ssh briansrls@100.73.169.40 ssh -i ~/.node-orch/id_ed25519 -o ConnectTimeout=5 ubuntu@192.168.1.221 hostname -f" 2>/dev/null || echo srv3-unreachable)
PROV
ls -la "$CAPTURE_ROOT"

echo "=== done $(date -Is) ==="
