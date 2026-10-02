#!/usr/bin/env python3
"""Run real compiler access refusals without nesting a compiler inside a DAG test."""
import argparse
import hashlib
import json
import re
import subprocess
import uuid
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path.cwd()
output = args.output.resolve()
if not output.is_relative_to((root / 'target').resolve()):
    parser.error('--output must be beneath this checkout target/')
binary = args.binary.resolve(strict=True)
subprocess.run([
    'python3', 'tools/tests/dag_validation_closure.py', '--output', str(output),
    'gunbc.workspace_commissioning_apply_readback', 'gunbc.workspace_slot_mutation',
], check=True)
fixture = Path('dag/test/claim/workspace_commissioning_seal_witness_test.dag').read_text()
cases = []
for name, declaration, subject in [
    ('release', 'commissioning_release_bypass_source', 'workspace_slot_mutation_release_fenced'),
    ('cleanup', 'commissioning_cleanup_bypass_source', 'workspace_commissioning_credential_remove'),
]:
    match = re.search(r'^data ' + declaration + r': String = ("(?:\\.|[^"\\])*")$', fixture, re.M)
    if not match:
        raise SystemExit('missing negative specimen: ' + declaration)
    cases.append((name, json.loads(match[1]), subject))
cases.append(('public', '''module test.probe.public_release
import std.types { Bool }
import gunbc.workspace_slot_mutation { WorkspaceSlotMutationLease, workspace_slot_mutation_release }
import gunbc.workspace_commissioning_apply_readback { WorkspaceCommissioningApplyReadback }
fn route(lease: WorkspaceSlotMutationLease) -> Bool { workspace_slot_mutation_release(lease: lease) }
''', None))
results = []
for name, source, subject in cases:
    probe = output / (name + '-probe.dag')
    module, body = source.split('\n', 1)
    probe.write_text(module + '\nimport std.process { ProcessExit, ExitSuccess }\n' + body + '\nfn check() -> ProcessExit { ExitSuccess }\n')
    log = output / (name + '.log')
    command = [
        'systemd-run', '--user', '--scope', '--unit=commissioning-access-' + uuid.uuid4().hex,
        '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet',
        'env', 'MALLOC_ARENA_MAX=2', str(binary), 'run', '--source-root', str(output),
        '--entry', str(probe), '--function', 'check',
    ]
    with log.open('w') as stream:
        result = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT)
    body = log.read_text()
    expected = "constructor call admission refused: 'gunbc."
    passed = result.returncode == 0 if subject is None else (
        result.returncode != 0 and expected in body and
        any(subject in line and 'constructor call admission refused:' in line
            for line in body.splitlines())
    )
    row = {'probe': name, 'exit_code': result.returncode, 'passed': passed,
           'source_sha256': hashlib.sha256(probe.read_bytes()).hexdigest()}
    results.append(row)
    print(json.dumps(row), flush=True)
    (output / 'results.json').write_text(json.dumps({
        'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'results': results,
    }, indent=2) + '\n')
    if not passed:
        raise SystemExit('unexpected compiler standing; read ' + str(log))
