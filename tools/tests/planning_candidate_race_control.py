"""Two independent CLI writers race one stored issue revision in real Fabric file storage."""
import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import tempfile

root = Path(tempfile.mkdtemp(prefix='gunbc-planning-race-'))
output = Path('target/planning-validation')
binary = os.environ.get('GUNBC_TEST_BINARY', 'gunbc')
base = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet', binary, 'run', '--source-root', 'target/planning-validation/browser-source', '--entry', 'target/planning-validation/browser-source/test.manual.planning_editor_fixture.dag', '--function', 'planning_fixture_run', '--arg', 'root='+str(root)]
def run(name, *args):
    with (output/(name+'.log')).open('w') as f:
        return subprocess.run(base+list(args), stdout=f, stderr=subprocess.STDOUT).returncode
assert run('race-seed', '--arg', 'mode=seed') == 0
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    outcomes = list(pool.map(lambda identity: run(identity, '--arg', 'mode=accept', '--arg', 'identity='+identity), ['race-one','race-two']))
assert sorted(outcomes) == [0,1], outcomes
loser = ['race-one','race-two'][outcomes.index(1)]
assert 'CONFLICT:' in (output/(loser+'.log')).read_text()
assert run('race-readback', '--arg', 'mode=read') == 0
winner = ['race-one','race-two'][outcomes.index(0)]
assert run('race-retry', '--arg', 'mode=accept', '--arg', 'identity='+winner) == 0
receipt = {'result':'PASS','writers':dict(zip(['race-one','race-two'],outcomes)), 'committed_events':5, 'winner_retry':'original result', 'store':str(root), 'scope':'independent CLI processes, real Fabric file-store head CAS'}
(output/'race-result.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt))
