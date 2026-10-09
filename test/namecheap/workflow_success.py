"""Run the emitted observer shell with a successful observer stand-in, offline."""
import os
import pathlib
import re
import subprocess
import tempfile

import yaml

root = pathlib.Path.cwd()
workflow = yaml.safe_load((root / '.github/workflows/fleet-converge.yml').read_text())
job = workflow['jobs']['namecheap-observe']
script = next(s['run'] for s in job['steps'] if s.get('id') == 'namecheap_observe')
upload = next(s for s in job['steps'] if s.get('id') == 'namecheap_secret_receipt')
# The stand-in produces only the artifact declared by the observer, independently
# of the generated console consumer and uploader under test.
authority = (root / 'dag/gunbc/namecheap/observation_artifact.dag').read_text()
receipt = re.search(r'data namecheap_observation_receipt_path: String = "([^"]+)"', authority)[1]
assert upload['with']['path'] == receipt
assert upload['if'] == 'success()'
with tempfile.TemporaryDirectory(prefix='namecheap-success-') as temp:
    sandbox = pathlib.Path(temp)
    binary = sandbox / 'target/release/gunbc'
    binary.parent.mkdir(parents=True)
    binary.write_text('#!/bin/sh\nset -eu\nprintf \'{"fixture":"successful-observation"}\\n\' > "$DECLARED_RECEIPT"\n')
    binary.chmod(0o700)
    env = dict(os.environ, DECLARED_RECEIPT=receipt)
    # Avoid inheriting the invoking checkout through Git environment overrides.
    env = {k: v for k, v in env.items() if not k.startswith('GIT_')}
    run = subprocess.run(['bash', '--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', script],
                         cwd=sandbox, env=env, capture_output=True, text=True, timeout=15)
    assert run.returncode == 0, run.stderr
    assert (sandbox / upload['with']['path']).read_text() == '{"fixture":"successful-observation"}\n'
    assert sorted(str(p.relative_to(sandbox)) for p in sandbox.rglob('*') if p.is_file()) == sorted([receipt, 'target/release/gunbc'])
    # Planted original defect must fail even after the legitimate receipt exists.
    broken = script + '\ncat "$ROOT/target/unproduced-execution.txt"\n'
    bad = subprocess.run(['bash', '--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', broken],
                         cwd=sandbox, env=env, capture_output=True, text=True, timeout=15)
    assert bad.returncode != 0
print('Emitted success path passed; exact upload artifact exists; unmatched receipt read fails')
