#!/usr/bin/env python3
"""Qualify the full preview entry without sending requests or installing a service.

Build the branch's binary first. Run this script inside the declared serving-budget
scope with swap disabled; the witness-test cap is a separate qualification.
"""
import argparse
import hashlib
import json
import re
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', default='target/release/gunbc')
parser.add_argument('--output', required=True)
parser.add_argument('--timeout', type=int, default=900)
parser.add_argument('--source-root', action='append')
parser.add_argument('--entry', default='dag/gunbc/roadmap/roadmap_serve.dag')
args = parser.parse_args()
out = Path(args.output)
out.mkdir(parents=True, exist_ok=True)
binary = Path(args.binary).resolve()
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
diff = subprocess.check_output(['git', 'diff', 'HEAD', '--', 'dag', 'src/v2'])
roots = args.source_root or ['dag', 'src/v2']
command = [str(binary), 'serve', *[part for root in roots for part in ['--source-root', root]],
           '--entry', args.entry,
           '--function', 'roadmap_serve_handle_srv2_preview',
           '--host', '127.0.0.1', '--port', '0', '--release-revision', revision]
receipt = {'revision': revision, 'source_diff_sha256': hashlib.sha256(diff).hexdigest(),
           'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
           'command': command, 'ready': False}
log_path = out / 'serve.log'
started = time.monotonic()
with log_path.open('w') as log:
    child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
    try:
        while time.monotonic() - started < args.timeout:
            text = log_path.read_text()
            match = re.search(r'^gunbc serve listening on 127\.0\.0\.1:([1-9][0-9]*) -> roadmap_serve_handle_srv2_preview\(\)', text, re.M)
            if match and child.poll() is None:
                receipt.update(ready=True, port=int(match.group(1)))
                break
            if child.poll() is not None:
                receipt['exit_code'] = child.returncode
                break
            time.sleep(1)
        else:
            receipt['failure'] = 'readiness deadline exceeded'
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
receipt['duration_seconds'] = round(time.monotonic() - started, 3)
for line in Path('/proc/self/cgroup').read_text().splitlines():
    if line.startswith('0::'):
        cgroup = Path('/sys/fs/cgroup') / line[3:].lstrip('/')
        receipt['memory'] = {name: (cgroup / name).read_text().strip()
                             for name in ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']
                             if (cgroup / name).is_file()}

(out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt, indent=2))
raise SystemExit(0 if receipt['ready'] else 1)
