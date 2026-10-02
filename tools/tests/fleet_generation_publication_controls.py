#!/usr/bin/env python3
"""Execute the emitted generation protocol in temporary stores, under real flock."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--script', type=Path, required=True)
    args = parser.parse_args()
    script = args.script.resolve()
    passed = []
    a, b = '0123456789abcdef', 'fedcba9876543210'
    with tempfile.TemporaryDirectory(prefix='fleet-generation-controls-') as temp:
        root = Path(temp)
        lock = root / 'existing.lock'
        lock.touch()

        def run(head, action, prior, next_generation, digest=a, *, ok=True, env=None):
            result = subprocess.run(['flock', '-x', str(lock), 'bash', str(script), action,
                                     str(head), str(prior), str(next_generation), digest],
                                    text=True, capture_output=True, env=env, timeout=15)
            assert (result.returncode == 0) == ok, (action, result.returncode, result.stdout, result.stderr)
            return result.stdout.strip()

        head = root / 'generation'
        head.write_text('17\n')
        assert run(head, 'admit', 17, 18) == 'apply'
        assert run(head, 'publish', 17, 18) == 'committed'
        assert head.read_text() == f'18 {a}\n'
        assert run(head, 'admit', 17, 18) == 'recover'
        passed.append('legacy counter transitions to attributable publication; retry skips effects')
        run(head, 'admit', 17, 18, b, ok=False)
        passed.append('same generation with another reviewed bundle refuses')
        assert run(head, 'admit', 18, 19, b) == 'apply'
        assert run(head, 'publish', 18, 19, b) == 'committed'
        assert run(head, 'observe', 17, 18) == 'committed'
        assert run(head, 'publish', 17, 18) == 'committed'
        assert head.read_text() == f'19 {b}\n'
        passed.append('historical receipt recovers after later apply without rolling head back')

        for body in ('garbage', '0 '+a, '18 '+a+' extra', '9223372036854775808', '-1'):
            bad = root / 'malformed'
            bad.write_text(body)
            run(bad, 'admit', 17, 18, ok=False)
        passed.append('malformed and out-of-range heads refuse before apply')
        run(head, 'admit', 19, 21, ok=False)
        passed.append('non-adjacent generation refuses')
        broken = root / 'broken'
        broken.symlink_to(root / 'missing')
        run(broken, 'admit', 0, 1, ok=False)
        passed.append('dangling head is unreadable, not a new store')

        wrappers = root / 'wrappers'
        wrappers.mkdir()
        env = dict(os.environ, PATH=str(wrappers)+os.pathsep+os.environ['PATH'])
        fault = wrappers / 'mv'
        fault.write_text('#!/bin/bash\nexit 71\n')
        fault.chmod(0o755)
        pending = root / 'pending'
        pending.write_text('17\n')
        run(pending, 'publish', 17, 18, ok=False, env=env)
        assert pending.read_text() == '17\n'
        assert run(pending, 'admit', 17, 18) == 'recover'
        run(pending, 'admit', 17, 18, b, ok=False)
        assert run(pending, 'observe', 17, 18, ok=False) == ''
        assert run(pending, 'publish', 17, 18) == 'committed'
        passed.append('interruption before head replacement retains receipt; only exact owner can recover')

        fault.write_text('#!/bin/bash\n/usr/bin/mv "$@"\nexit 72\n')
        after = root / 'after-rename'
        after.write_text('17\n')
        run(after, 'publish', 17, 18, ok=False, env=env)
        assert after.read_text() == f'18 {a}\n'
        assert run(after, 'admit', 17, 18) == 'recover'
        assert run(after, 'publish', 17, 18) == 'committed'
        passed.append('lost head-publication response recovers exact committed generation')

        fault.unlink()
        durability_fault = wrappers / 'sync'
        durability_fault.write_text('#!/bin/bash\n[[ ${2:-} != *.receipts ]] || exit 73\nexec /usr/bin/sync "$@"\n')
        durability_fault.chmod(0o755)
        unsynced = root / 'unsynced-receipt'
        unsynced.write_text('17\n')
        run(unsynced, 'publish', 17, 18, ok=False, env=env)
        assert (root / 'unsynced-receipt.receipts' / '18').exists()
        assert unsynced.read_text() == '17\n'
        # A surviving link is not proof that the previous directory sync succeeded.
        run(unsynced, 'publish', 17, 18, ok=False, env=env)
        assert unsynced.read_text() == '17\n'
        assert run(unsynced, 'publish', 17, 18) == 'committed'
        passed.append('recovery re-syncs an existing receipt before publishing the head')

        transaction = '''set -euo pipefail
script=$1; head=$2; digest=$3; effects=$4
decision=$(bash "$script" admit "$head" 0 1 "$digest")
if [[ $decision == apply ]]; then printf 'effect\n' >> "$effects"; fi
bash "$script" publish "$head" 0 1 "$digest"
'''
        for distinct in (False, True):
            name = 'distinct' if distinct else 'same'
            concurrent = root / name
            effects = root / (name+'-effects')
            procs = [subprocess.Popen(['flock', '-x', str(lock), 'bash', '-c', transaction,
                     'generation-transaction', str(script), str(concurrent), digest, str(effects)],
                     stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for digest in ([a, b] if distinct else [a, a, a, a])]
            results = [(proc.communicate(timeout=15), proc.returncode) for proc in procs]
            assert effects.read_text() == 'effect\n', results
            assert sum(code == 0 for _, code in results) == (1 if distinct else 4), results
            passed.append('competing bundles serialize with one winner' if distinct else 'racing retries execute one effect')
    print(json.dumps({'passed': len(passed), 'controls': passed, 'scope': 'temporary local stores; no fleet mutation'}, indent=2))


if __name__ == '__main__':
    main()
