"""Build this branch and require its complete production preview serve entry to listen.

No request is sent: handler evaluation can access protected runtime state. The server
binds an OS-selected loopback port and is terminated after the compiler's readiness line.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time

root = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip())
os.chdir(root)
output = root / 'target/serve-integration'
output.mkdir(parents=True, exist_ok=True)
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
memory = os.environ.get('GUNBC_SERVE_CHECK_MEMORY', '26G')
scope = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=' + memory,
         '-p', 'MemorySwapMax=0', '--quiet']
with (output / 'build.log').open('w') as log:
    subprocess.run(scope + ['cargo', 'build', '--release', '-p', 'v1-compiler', '--bin', 'gunbc'],
                   stdout=log, stderr=subprocess.STDOUT, check=True)
binary = root / 'target/release/gunbc'
entry = 'roadmap_serve_handle_srv2_preview'
command = scope + [str(binary), 'serve', '--source-root', 'dag', '--source-root', 'src/v2',
                   '--entry', 'dag/gunbc/roadmap/roadmap_serve.dag', '--function', entry,
                   '--host', '127.0.0.1', '--port', '0', '--release-revision', revision]
started = time.monotonic()
ready = None
with (output / 'serve.log').open('w') as log:
    process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        while time.monotonic() - started < 900:
            contents = (output / 'serve.log').read_text()
            ready = re.search(r'gunbc serve listening on 127\.0\.0\.1:(\d+) -> '
                              + entry + r'\(\) release_revision=' + revision, contents)
            if ready:
                if process.poll() is not None:
                    raise RuntimeError('server exited after readiness; see ' + str(output / 'serve.log'))
                break
            if process.poll() is not None:
                raise RuntimeError('full serve closure failed, exit ' + str(process.returncode)
                                   + '; see ' + str(output / 'serve.log'))
            time.sleep(0.25)
        if not ready:
            raise RuntimeError('full serve readiness timed out; see ' + str(output / 'serve.log'))
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
receipt = {
    'result': 'PASS', 'revision': revision,
    'worktree_status': subprocess.check_output(['git', 'status', '--porcelain'], text=True),
    'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
    'entry': entry, 'memory_limit': memory, 'readiness': ready.group(0),
    'elapsed_seconds': round(time.monotonic() - started, 2),
    'scope': 'branch-built binary; whole serve import closure; OS-bound loopback listener; no HTTP requests',
}
(output / 'readiness.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt))
