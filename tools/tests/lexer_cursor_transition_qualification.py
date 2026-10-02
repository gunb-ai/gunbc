#!/usr/bin/env python3
"""Compose bounded legacy lexer transitions into a full-file equivalence proof.

The input is NEVER segmented. At each boundary the cursor state is related to
legacy state by replacing source+index with that exact source suffix. All other
semantic state fields are compared, including cumulative tokens, annotations,
layout, byte position, trivia state, errors and EOF. Legacy dispatch is rebuilt
from the same immutable rules; its step always carries that dispatch unchanged.
Starting at the genuine legacy initial state, contiguous passing intervals prove
by induction that every checkpoint (including EOF) is its actual legacy state.
No production interpreter, memo setting, lexer code or resource limit changes.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

REF = '85f92ac853cba7149c0d6f66c08190f2277e56fa'
SUBJECT = '359661af2d7'
SIZE = 2048

def aggregate(rows, total, tokens):
    ordered = sorted(rows, key=lambda row: row['start'])
    roster = list(range(0, total, SIZE))
    return (len(ordered) == len(roster) and [r['start'] for r in ordered] == roster
            and all(r['exit'] == 0 and r['end'] == min(r['start'] + SIZE, total)
                    and r['done'] == (r['end'] == total) for r in ordered)
            and ordered[-1]['tokens'] == tokens)

def negative_aggregate_controls(rows, total, tokens):
    overlap = [dict(row) for row in rows]
    overlap[0]['end'] += 1
    no_eof = [dict(row) for row in rows]
    max(no_eof, key=lambda row: row['end'])['done'] = False
    return {
        'missing_interval_refuses': not aggregate(rows[1:], total, tokens),
        'duplicate_interval_refuses': not aggregate(rows + rows[:1], total, tokens),
        'overlap_refuses': not aggregate(overlap, total, tokens),
        'missing_eof_refuses': not aggregate(no_eof, total, tokens),
        'wrong_token_count_refuses': not aggregate(rows, total, tokens + 1),
    }

parser = argparse.ArgumentParser()
parser.add_argument('--gunbc', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
binary = args.gunbc.resolve()
out = Path(tempfile.mkdtemp(prefix='cursor-transitions-', dir=root / 'target'))
source = out / 'source'
subprocess.run([sys.executable, str(root / 'tools/tests/dag_validation_closure.py'),
                '--output', str(source), '--preserve-source-paths',
                'v2.compiler.tokenize', 'v2.extdeps.languages.dag',
                'extdeps.filesystem.filesystem_io', 'std.process'], cwd=root, check=True)
reference = subprocess.check_output(['git', 'show', REF + ':src/v2/compiler/01_tokenize.dag'], cwd=root)
walk = reference.decode().split('fn lex_walk_step(', 1)[1].split('fn lex_walk_source_algebra(', 1)[0]
assignments = re.findall(r'(?m)^\s*dispatch:\s*([^,\n]+)', walk)
if not assignments or any(a.strip() != 'acc.dispatch' for a in assignments):
    raise SystemExit('reference dispatch immutability premise requires review')
(source / 'reference.dag').write_bytes(reference.replace(b'v2.compiler.tokenize', b'tools.reference_tokenize'))
fixtures = root / 'tools/tests/fixtures/lexer_cursor'
for name in ('transitions', 'census'):
    (source / (name + '.dag')).write_bytes((fixtures / (name + '.dag.txt')).read_bytes())
subject = out / 'input.dag'
subject.write_bytes(subprocess.check_output(['git', 'show', SUBJECT + ':dag/test/claim/fleet/fleet_converge_plan_witness_test.dag'], cwd=root))
def identities():
    files = [binary, subject] + sorted(source.rglob('*.dag'))
    return {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
pinned = identities()
receipt = {'reference_revision': REF, 'subject_revision': SUBJECT, 'identities': pinned,
           'standing': 'incomplete', 'memory_max_gib_per_process': 6,
           'swap_max': 0, 'timeout_seconds': 300, 'workers': 2,
           'eval_memo': 'enabled', 'interval_size': SIZE, 'rows': [],
           'legacy_dispatch_assignments': assignments,
           'qualifier_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
def save():
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(out / 'receipt.json', flush=True)
def run(name, entry, extra):
    result_path = out / (name + '.txt')
    argv = ['systemd-run', '--user', '--scope', '--quiet', '-p', 'MemoryMax=6G',
            '-p', 'MemorySwapMax=0', 'env', 'MALLOC_ARENA_MAX=2', 'GUNBC_EVAL_MEMO=1',
            'timeout', '300', str(binary), 'run', '--source-root', str(source),
            '--entry', str(source / entry), '--arg', 'input_path=' + str(subject),
            '--arg', 'receipt_path=' + str(result_path)] + extra
    started = time.monotonic()
    with (out / (name + '.log')).open('w') as log:
        code = subprocess.run(argv, cwd=root, stdout=log, stderr=subprocess.STDOUT).returncode
    return code, round(time.monotonic() - started, 3), result_path
save()
code, seconds, path = run('census', 'census.dag', [])
receipt['census_exit'] = code
if code:
    receipt['standing'] = 'failed:census'; save(); raise SystemExit(1)
total, tokens, scalars = map(int, path.read_text().split())
receipt['census'] = {'transitions': total, 'tokens': tokens, 'scalars': scalars, 'seconds': seconds}
code, seconds, path = run('corrupt-position', 'transitions.dag', ['--function', 'negative'])
negative = code != 0 and code != 124 and not path.exists() and 'reference transition differs' in (out / 'corrupt-position.log').read_text()
receipt['corrupt_position_refused'] = negative
if not negative:
    receipt['standing'] = 'failed:negative-control'; save(); raise SystemExit(1)
roster = list(range(0, total, SIZE))
def shard(start):
    code, seconds, path = run('interval-' + str(start), 'transitions.dag',
                              ['--arg', 'start=' + str(start), '--arg', 'count=' + str(min(SIZE, total-start))])
    row = {'start': start, 'exit': code, 'seconds': seconds}
    if code == 0 and path.exists():
        actual_start, end, done, count = path.read_text().split()
        row.update(start=int(actual_start), end=int(end), done=done == 'true', tokens=int(count))
    return row
with ThreadPoolExecutor(max_workers=2) as pool:
    for future in as_completed([pool.submit(shard, start) for start in roster]):
        row = future.result(); receipt['rows'].append(row); save(); print(json.dumps(row), flush=True)
passed = aggregate(receipt['rows'], total, tokens)
# A missing interval and a duplicate interval must fail even when all present rows pass.
receipt['aggregate_negative_controls'] = negative_aggregate_controls(receipt['rows'], total, tokens)
receipt['identities_unchanged'] = identities() == pinned
receipt['standing'] = 'passed' if passed and receipt['identities_unchanged'] and all(receipt['aggregate_negative_controls'].values()) else 'failed:aggregate'
save()
raise SystemExit(0 if receipt['standing'] == 'passed' else 1)
