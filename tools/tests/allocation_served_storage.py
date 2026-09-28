#!/usr/bin/env python3
"""Real authenticated DAG storage route; exact above-cap and transport-failure controls."""
import argparse
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

p = argparse.ArgumentParser()
p.add_argument('--binary', required=True)
p.add_argument('--server-source', required=True)
p.add_argument('--client-source', required=True)
a = p.parse_args()
root = Path(tempfile.mkdtemp(prefix='gunbc-hold-storage-'))
key = root / 'fixture-key'
key.write_text(secrets.token_hex(32))
key.chmod(0o600)
env = dict(os.environ, GUNBC_HOLD_STORAGE_ROOT=str(root), GUNBC_FABRIC_STATE_KEY_FILE=str(key))
with socket.socket() as sock:
    sock.bind(('127.0.0.1', 0))
    port = sock.getsockname()[1]
origin = f'http://127.0.0.1:{port}'
bounded = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet', str(Path(a.binary).resolve())]
server = None
logs = []
results = []

def start(name):
    global server
    log = (root / f'{name}.log').open('w')
    logs.append(log)
    server = subprocess.Popen(bounded + ['serve', '--source-root', a.server_source, '--entry', str(Path(a.server_source) / 'test.manual.fabric_state_hold_server.dag'), '--function', 'served_storage_hold_handler', '--host', '127.0.0.1', '--port', str(port), '--release-revision', subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()], env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(f'server exited; inspect {root}')
        try:
            urllib.request.urlopen(origin, timeout=.5)
        except urllib.error.HTTPError as error:
            if error.code == 405:
                return
        except (urllib.error.URLError, TimeoutError):
            pass
        time.sleep(.2)
    raise TimeoutError(f'server startup; inspect {root}')

def stop():
    global server
    if server is not None and server.poll() is None:
        os.killpg(server.pid, signal.SIGTERM)
        try:
            server.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(server.pid, signal.SIGKILL)
            server.wait()
    server = None

def check(step, expected=0, suffix=''):
    with (root / f'{step}{suffix}.log').open('w') as log:
        result = subprocess.run(bounded + ['run', '--source-root', a.client_source, '--entry', str(Path(a.client_source) / 'test.manual.roadmap_served_storage_hold_wet.dag'), '--function', 'served_storage_hold_check', '--arg', f'endpoint={origin}/fabric-storage/state', '--arg', f'step={step}'], env=env, stdout=log, stderr=subprocess.STDOUT, timeout=600)
    if result.returncode != expected:
        raise RuntimeError(f'{step}{suffix}: exit {result.returncode}, expected {expected}; inspect {root}')
    results.append({'step': step+suffix, 'exit': result.returncode})

try:
    start('server')
    for step in ('empty', 'append', 'read', 'above-cap'):
        check(step)
    stop()
    start('restarted-server')
    check('read', suffix='-after-restart')
    stop()
    check('above-cap', expected=1, suffix='-transport-refusal')
    receipt = {'fixture': str(root), 'results': results}
    (root / 'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps(receipt))
finally:
    stop()
    for log in logs:
        log.close()
    key.unlink(missing_ok=True)
