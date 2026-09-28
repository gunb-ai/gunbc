"""Run actual .dag commands against a disposable Git remote, including a forced CAS race."""
import concurrent.futures
import json
import subprocess
import tempfile
from pathlib import Path

repo = Path(__file__).resolve().parents[2]
root = Path(tempfile.mkdtemp(prefix='gunbc-issue-command-'))
fixture = root/'repo'
def git(*args, cwd=None):
    return subprocess.run(['/usr/bin/git', *args], cwd=cwd, check=True, capture_output=True, text=True).stdout

git('init', '--bare', str(root/'remote.git'))
git('clone', str(root/'remote.git'), str(fixture))
git('config', 'user.name', 'Issue Fixture', cwd=fixture)
git('config', 'user.email', 'fixture@example.invalid', cwd=fixture)
git('commit', '--allow-empty', '-m', 'fixture root', cwd=fixture)
git('push', 'origin', 'HEAD', cwd=fixture)
(fixture/'instance').mkdir()
base = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet',
    str(repo/'target/release/gunbc'), 'run', '--source-root', 'target/issue-command-wet',
    '--entry', 'target/issue-command-wet/test.manual.issue_command_wet.dag', '--function', 'issue_command_wet', '--arg', 'repo='+str(fixture)]
with (root/'sequential.log').open('w') as out:
    subprocess.run(base, cwd=repo, stdout=out, stderr=subprocess.STDOUT, check=True)
barrier = root/'barrier'
barrier.mkdir()
wrapper = root/'git-race'
wrapper.write_text('''#!/usr/bin/python3
import os,sys,time
from pathlib import Path
barrier=Path(__file__).parent/'barrier'
if sys.argv[1:2] == ['push'] and any(x.startswith('--force-with-lease=') for x in sys.argv):
    (barrier/str(os.getpid())).touch()
    deadline=time.monotonic()+300
    while len(list(barrier.iterdir())) < 2:
        if time.monotonic() > deadline: sys.exit(91)
        time.sleep(.05)
os.execv('/usr/bin/git',['git',*sys.argv[1:]])
''')
wrapper.chmod(0o700)
def race(name):
    with (root/(name+'.log')).open('w') as out:
        return subprocess.run(base+['--arg', 'mode='+name, '--arg', 'git_program='+str(wrapper)], cwd=repo, stdout=out, stderr=subprocess.STDOUT).returncode
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    results = list(pool.map(race, ['race-a', 'race-b']))
assert sorted(results) == [0,1], (root, results)
files = git('--git-dir='+str(root/'remote.git'), 'ls-tree', '-r', '--name-only', 'roadmap-events', 'events/raced-issue/').splitlines()
assert len(files) == 1, (root, files)
receipts = git('--git-dir='+str(root/'remote.git'), 'ls-tree', '-r', '--name-only', 'roadmap-events', 'operations/raced-issue/').splitlines()
assert len(receipts) == 1, (root, receipts)
(root/'receipt.json').write_text(json.dumps({'sequential': 'passed', 'race_exit_codes': results, 'events': files, 'operations': receipts}, indent=2))
print('PASS actual command CAS, replay, stale writer, different payload, causal readback, concurrent publication:', root)
