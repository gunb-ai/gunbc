#!/usr/bin/env python3
"""Two real DAG transaction consumers released after both read the live transaction."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument('--binary', required=True)
parser.add_argument('--source-root', required=True)
args = parser.parse_args()
root = Path(tempfile.mkdtemp(prefix='gunbc-login-race-'))
base = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet',
        str(Path(args.binary).resolve()), 'run', '--source-root', args.source_root,
        '--entry', str(Path(args.source_root) / 'test.manual.login_consumer_race.dag')]

def command(function, *extra):
    return base + ['--function', function, '--arg', f'root={root}', *extra]

with (root / 'prepare.log').open('w') as log:
    subprocess.run(command('prepare'), stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
processes = []
logs = []
try:
    for name in ('a', 'b'):
        log = (root / f'{name}.log').open('w')
        logs.append(log)
        processes.append(subprocess.Popen(command('consume', '--arg', f'contender={name}'), stdout=log, stderr=subprocess.STDOUT))
    deadline = time.monotonic() + 600
    while not all((root / f'ready-{name}').exists() for name in ('a', 'b')):
        if any(p.poll() is not None for p in processes):
            raise RuntimeError(f'consumer exited before barrier; inspect {root}')
        if time.monotonic() >= deadline:
            raise TimeoutError(f'consumers did not reach barrier; inspect {root}')
        time.sleep(.1)
    (root / 'go').write_text('go')
    for process in processes:
        if process.wait(timeout=60) != 0:
            raise RuntimeError(f'consumer failed; inspect {root}')
    results = {name: (root / f'result-{name}').read_text() for name in ('a', 'b')}
    if sorted(results.values()) != ['loser', 'winner']:
        raise AssertionError(results)
    receipt = {'fixture': str(root), 'results': results, 'both_read_before_release': True}
    (root / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))
finally:
    for process in processes:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
    for log in logs:
        log.close()
