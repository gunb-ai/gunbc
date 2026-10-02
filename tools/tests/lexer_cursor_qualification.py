#!/usr/bin/env python3
"""Reproduce cursor/reference equivalence using pinned source and compact receipts.

Run inside the qualification cgroup (6 GiB, no swap). Each sample receives a
fresh process and a 600-second bound; no generated source snapshot is committed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('--gunbc', type=Path, required=True)
parser.add_argument('--diagnostic-no-memo', action='store_true', help='Semantic comparison only; disable existing eval memo diagnostic switch')
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
binary = args.gunbc.resolve()
reference = '85f92ac853cba7149c0d6f66c08190f2277e56fa'
allocation = '359661af2d7'
receipts = root / 'docs/plans/receipts/lexer-cursor-native-blocker-2026-10-02'
out = Path(tempfile.mkdtemp(prefix='cursor-qualification-', dir=root / 'target'))
closure = out / 'source'
subprocess.run([sys.executable, str(root / 'tools/tests/dag_validation_closure.py'),
                '--output', str(closure), 'v2.compiler.tokenize',
                'v2.extdeps.languages.dag', 'extdeps.filesystem.filesystem_io',
                'std.process'], cwd=root, check=True)
prior = subprocess.check_output(['git', 'show', reference + ':src/v2/compiler/01_tokenize.dag'], cwd=root, text=True)
(closure / 'tools.reference_tokenize.dag').write_text(prior.replace('v2.compiler.tokenize', 'tools.reference_tokenize'))
(closure / 'tools.cursor_equivalence.dag').write_bytes((receipts / 'equivalence-harness.dag.txt').read_bytes())
source = (root / 'dag/test/claim/live_deploy/emit_test.dag').read_bytes()
boundaries = [m.start() for m in re.finditer(rb'(?m)^(?:test )?fn ', source)]
samples = []
for size in (16, 32, 64):
    end = max(n for n in boundaries if n <= size * 1024)
    samples.append((f'emit-{size}k', source[:end]))
samples.append(('fleet-full', subprocess.check_output(['git', 'show', allocation + ':dag/test/claim/fleet/fleet_converge_plan_witness_test.dag'], cwd=root)))
receipt = {'reference_revision': reference, 'allocation_revision': allocation,
           'compiler_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
           'source_manifest': json.loads((closure / 'sources.json').read_text()),
           'standing': 'incomplete', 'samples': [],
           'eval_memo': 'diagnostic-disabled' if args.diagnostic_no_memo else 'enabled',
           'performance_qualification': not args.diagnostic_no_memo}
environment = os.environ.copy()
environment['GUNBC_EVAL_MEMO'] = '0' if args.diagnostic_no_memo else '1'
for name, data in samples:
    path = out / (name + '.dag'); path.write_bytes(data)
    marker = out / (name + '.cursor-done')
    events = []; start = time.monotonic()
    command = [str(binary), 'run', '--source-root', str(closure), '--entry',
               str(closure / 'tools.cursor_equivalence.dag'), '--arg',
               'input_path=' + str(path), '--arg', 'marker_path=' + str(marker)]
    process = subprocess.Popen(command, cwd=root, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True, env=environment)
    def consume():
        with (out / (name + '.log')).open('w') as log:
            for line in process.stdout:
                log.write(line); log.flush()
                if 'Filesystem.' in line:
                    events.append({'seconds': round(time.monotonic() - start, 3), 'event': line.strip()})
    reader = threading.Thread(target=consume); reader.start()
    try:
        code = process.wait(timeout=600)
    except subprocess.TimeoutExpired:
        process.kill(); process.wait(); code = 124
    reader.join()
    receipt['samples'].append({'name': name, 'bytes': len(data),
                              'sha256': hashlib.sha256(data).hexdigest(), 'exit': code,
                              'seconds': round(time.monotonic() - start, 3), 'events': events})
    receipt['standing'] = 'failed' if code else 'incomplete'
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    if code:
        print(out / 'receipt.json'); raise SystemExit(1)
receipt['standing'] = 'passed'
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(out / 'receipt.json')
