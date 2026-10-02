#!/usr/bin/env python3
"""Emit the shared list-at fixture, build its modeled native driver, and run it.

Run under the qualification cgroup. A build failure is a failure, never a frontier
pass. The driver comes from rust_native_driver (also used by identity-cast tests).
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument('--gunbc', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
gunbc = args.gunbc.resolve()
(root / 'target').mkdir(exist_ok=True)
out = Path(tempfile.mkdtemp(prefix='list-at-native-', dir=root / 'target'))
crate = out / 'emitted'
receipt = {'compiler_sha256': hashlib.sha256(gunbc.read_bytes()).hexdigest(),
           'standing': 'incomplete', 'stages': [], 'cases': []}

def save():
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

def stage(name, command):
    start = time.monotonic()
    with (out / (name + '.log')).open('wb') as log:
        try:
            result = subprocess.run(command, cwd=root, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=600)
            code = result.returncode
        except subprocess.TimeoutExpired:
            code = 124
    receipt['stages'].append({'name': name, 'exit': code,
                              'seconds': round(time.monotonic() - start, 3)})
    save()
    if code:
        receipt['standing'] = 'failed:' + name
        save()
        print(out / 'receipt.json')
        raise SystemExit(1)

closure = out / 'source'
stage('closure', [sys.executable, str(root / 'tools/tests/dag_validation_closure.py'),
                  '--output', str(closure), 'v2.test.fixture.list_at_native',
                  'v2.test.fixture.list_at_native_driver'])
receipt['closure_manifest'] = json.loads((closure / 'sources.json').read_text())
stage('emit', [str(gunbc), 'compile', '--source-root', str(closure), '--entry',
              str(closure / 'v2.test.fixture.list_at_native.dag'), '--output-dir', str(crate)])
stage('driver', [str(gunbc), 'run', '--source-root', str(closure), '--entry',
                str(closure / 'v2.test.fixture.list_at_native_driver.dag'),
                '--arg', 'output_path=' + str(crate / 'src/main.rs')])
receipt['emitted_sha256'] = {str(p.relative_to(crate)): hashlib.sha256(p.read_bytes()).hexdigest()
                            for p in sorted(crate.rglob('*')) if p.is_file()}
save()
stage('build', ['cargo', 'build', '--offline', '-j2', '--manifest-path',
                str(crate / 'Cargo.toml'), '--target-dir', str(out / 'build')])
binary = out / 'build/debug/v1_compiled'
# Explicit expected answers, independently of the runtime implementation.
for member, index, expected in [('apply', 0, 10), ('apply', 1, 20), ('apply', 2, 30),
                                ('apply', -1, -1), ('apply', 3, -1), ('apply', 100, -1),
                                ('empty', 0, -1), ('empty', -1, -1)]:
    result = subprocess.run([str(binary), member, str(index)], capture_output=True, timeout=10)
    passed = result.returncode == 0 and result.stdout == b'\0' + struct.pack('<i', expected)
    receipt['cases'].append({'member': member, 'index': index, 'expected': expected, 'passed': passed})
for argv in [('apply', 'not-an-int'), ('unknown-member', '0')]:
    result = subprocess.run([str(binary), *argv], capture_output=True, timeout=10)
    receipt['cases'].append({'argv': argv, 'passed': result.returncode != 0 and not result.stdout})
receipt['standing'] = 'passed' if all(r['passed'] for r in receipt['cases']) else 'failed:execution'
save()
print(out / 'receipt.json')
raise SystemExit(0 if receipt['standing'] == 'passed' else 1)
