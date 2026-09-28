#!/usr/bin/env python3
"""Real authenticated DAG storage route; exact above-cap and transport-failure controls."""
import argparse
import json
import http.client
import pwd
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
p.add_argument('--unix', action='store_true', help='Exercise the real protected Unix-socket door and peer refusal')
p.add_argument('--binary', required=True)
p.add_argument('--server-source', required=True)
p.add_argument('--client-source', required=True)
a = p.parse_args()
root = Path(tempfile.mkdtemp(prefix='gunbc-hold-storage-'))
key = root / 'fixture-key'
key.write_text(secrets.token_hex(32))
key.chmod(0o600)
writer_file = root / 'fixture-writer'
writer_name = pwd.getpwuid(os.geteuid()).pw_name
writer_file.write_text(writer_name)
env = dict(os.environ, GUNBC_HOLD_STORAGE_ROOT=str(root), GUNBC_FABRIC_STATE_KEY_FILE=str(key), GUNBC_HOLD_LOCAL_WRITER_FILE=str(writer_file))
door = root / 'door.sock'

class UnixHTTPConnection(http.client.HTTPConnection):
    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.connect(str(door))

def socket_request(path, cookie=None, duplicate=False):
    conn = UnixHTTPConnection('localhost', timeout=2)
    try:
        conn.putrequest('GET', path)
        if cookie is not None:
            conn.putheader('Cookie', cookie)
            if duplicate:
                conn.putheader('Cookie', 'fixture=other')
        conn.endheaders()
        response = conn.getresponse()
        return response.status, dict(response.getheaders()), response.read().decode()
    finally:
        conn.close()

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
    module = 'test.manual.fabric_state_socket_hold_server' if a.unix else 'test.manual.fabric_state_hold_server'
    function = 'served_storage_socket_hold_handler' if a.unix else 'served_storage_hold_handler'
    listener = ['--unix-socket', str(door)] if a.unix else ['--host', '127.0.0.1', '--port', str(port)]
    server = subprocess.Popen(bounded + ['serve', '--source-root', a.server_source, '--entry', str(Path(a.server_source) / (module+'.dag')), '--function', function, *listener, '--release-revision', subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()], env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(f'server exited; inspect {root}')
        try:
            if a.unix:
                if socket_request('/fixture/session')[0] == 200:
                    return
            else:
                urllib.request.urlopen(origin, timeout=.5)
        except urllib.error.HTTPError as error:
            if error.code == 405:
                return
        except (urllib.error.URLError, TimeoutError, OSError, http.client.HTTPException):
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
        result = subprocess.run(bounded + ['run', '--source-root', a.client_source, '--entry', str(Path(a.client_source) / 'test.manual.roadmap_served_storage_hold_wet.dag'), '--function', 'served_storage_hold_check', '--arg', f'endpoint={"unix:"+str(door) if a.unix else origin+"/fabric-storage/state"}', '--arg', f'step={step}'], env=env, stdout=log, stderr=subprocess.STDOUT, timeout=600)
    if result.returncode != expected:
        raise RuntimeError(f'{step}{suffix}: exit {result.returncode}, expected {expected}; inspect {root}')
    results.append({'step': step+suffix, 'exit': result.returncode})

try:
    start('server')
    if a.unix:
        status, headers, body = socket_request('/fixture/session', cookie='fixture=incoming')
        assert status == 200 and body == 'fixture=incoming'
        assert headers.get('Set-Cookie') == 'fixture=socket; HttpOnly; SameSite=Lax'
        assert socket_request('/fixture/session', cookie='fixture=one', duplicate=True)[0] == 400
        results.append({'step': 'socket-cookie-and-response-headers', 'exit': 0})
    for step in ('empty', 'append', 'read', 'above-cap'):
        check(step)
    if a.unix:
        writer_file.write_text('unrostered-fixture-account')
        check('read', expected=1, suffix='-unrostered-peer')
        writer_file.write_text(writer_name)
        bad_key = root / 'fixture-wrong-key'
        bad_key.write_text(secrets.token_hex(32))
        bad_key.chmod(0o600)
        env['GUNBC_FABRIC_STATE_KEY_FILE'] = str(bad_key)
        check('read', expected=1, suffix='-wrong-signing-key')
        env['GUNBC_FABRIC_STATE_KEY_FILE'] = str(key)
        bad_key.unlink()
        check('read', suffix='-after-refusals')
    stop()
    start('restarted-server')
    check('read', suffix='-after-restart')
    stop()
    check('above-cap', expected=1, suffix='-transport-refusal')
    receipt = {'fixture': str(root), 'transport': 'unix' if a.unix else 'tcp', 'results': results}
    (root / 'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps(receipt))
finally:
    stop()
    for log in logs:
        log.close()
    key.unlink(missing_ok=True)
