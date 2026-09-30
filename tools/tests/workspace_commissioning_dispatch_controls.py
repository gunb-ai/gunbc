#!/usr/bin/env python3
"""Execute the exported dispatch transport against explicit process-boundary doubles.

No host units, protected stores or fleet state are touched. The independent child holding
an actual flock represents the installed executor, not the commissioning authority itself.
"""
import argparse
import fcntl
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument('--script', required=True)
args = parser.parse_args()
script = Path(args.script).resolve()
root = Path('/tmp/workspace-dispatch-controls')
root.mkdir(exist_ok=True)
control_lock = open(root / 'controls.lock', 'a+')
fcntl.lockf(control_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
(root / 'fleet.lock').touch()
fixture = r'''
import fcntl,json,os,subprocess,sys,time
from pathlib import Path
root=Path('/tmp/workspace-dispatch-controls'); case=Path(os.environ['DISPATCH_CASE'])
mode=os.environ['DISPATCH_MODE']; action=Path(sys.argv[0]).name
inv='a'*32; bundle='0123456789abcdef'; state=case/'state.json'
def put(value):
 p=case/'state.pending'; p.write_text(json.dumps(value)); p.replace(state)
def log(value):
 with (case/'calls').open('a') as f:f.write(value+'\n')
if action=='worker':
 with (root/'fleet.lock').open() as lock:
  fcntl.flock(lock,fcntl.LOCK_EX)
  (case/'owned').write_text('yes')
  time.sleep(0.5 if mode!='pending' else 30)
  put({'active':'failed' if mode=='failed-unit' else 'active','pid':'0','inv':inv,'job':''})
  (case/'finished').write_text('yes')
 sys.exit(0)
if action=='stage':
 assert sys.stdin.read()==bundle+'\ncanonical-test-input'
 with (root/'fleet.lock').open() as lock:
  try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  except BlockingIOError:pass
  else:raise AssertionError('stage did not retain the existing fleet lock')
 log('stage')
 if mode=='stage-refused':sys.exit(8)
 if mode=='bad-stage':print('invalid');sys.exit(0)
 if not state.exists():
  put({'active':'activating','pid':'1','inv':inv,'job':''})
  p=subprocess.Popen([str(root/'worker')],env=os.environ,start_new_session=True,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
  (case/'worker.pid').write_text(str(p.pid));log('start')
 print(inv);sys.exit(0)
if action=='systemctl':
 assert sys.argv[1]=='show'
 value=json.loads(state.read_text()); log('observe')
 seen='b'*32 if mode=='changed-invocation' else value['inv']
 print('Id=gunbc-workspace-commissioning@srv1-13.service')
 print('LoadState=loaded'); print('ActiveState='+value['active'])
 print('MainPID='+value['pid']); print('InvocationID='+seen)
 if mode!='missing-job':print('Job='+value['job'])
 sys.exit(0)
if action=='complete':
 assert sys.stdin.read()==bundle+'\nsrv1-13'
 assert (case/'finished').exists(), 'completion ran before executor released lock'
 log('complete')
 if mode=='uncommitted':sys.exit(9)
 if mode=='readable-refusal':print('refused: commissioning failed');sys.exit(0)
 if mode=='wrong-bundle':print('commissioning-fleet-generation-committed:'+'f'*16+':'+inv);sys.exit(0)
 if mode=='changed-after-completion':put({'active':'active','pid':'0','inv':'b'*32,'job':''})
 print('commissioning-fleet-generation-committed:'+bundle+':'+inv);sys.exit(0)
raise AssertionError(action)
'''
for name in ('stage', 'complete', 'systemctl', 'worker'):
    path = root / name
    path.write_text('#!' + sys.executable + '\n' + fixture)
    path.chmod(0o700)

def start(case, mode, bundle='0123456789abcdef'):
    env = dict(os.environ, DISPATCH_CASE=str(case), DISPATCH_MODE=mode)
    p = subprocess.Popen(['bash', str(script), bundle], env=env, stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    p.stdin.write(b'canonical-test-input')
    p.stdin.close()
    p.stdin = None
    return p

def await_file(path):
    deadline = time.monotonic() + 8
    while not path.exists():
        if time.monotonic() > deadline:
            raise AssertionError('timed out waiting for ' + str(path))
        time.sleep(.02)

def cleanup(case, p=None):
    if p is not None and p.poll() is None:
        os.killpg(p.pid, signal.SIGKILL)
        p.wait()
    if (case/'worker.pid').exists():
        pid = int((case/'worker.pid').read_text())
        try:
            if str(root / 'worker').encode() in Path('/proc', str(pid), 'cmdline').read_bytes():
                os.killpg(pid, signal.SIGKILL)
        except (FileNotFoundError, ProcessLookupError): pass

results=[]
for mode in ('success','stage-refused','bad-stage','failed-unit','changed-invocation',
             'missing-job','uncommitted','readable-refusal','wrong-bundle','changed-after-completion'):
    with tempfile.TemporaryDirectory(prefix='dispatch-case-') as td:
        case=Path(td);p=start(case,mode)
        try:
            out,err=p.communicate(timeout=10)
            assert (p.returncode==0)==(mode=='success'), (mode,p.returncode,out,err)
            if mode=='success':
                assert out.strip()==b'commissioning-fleet-generation-committed:0123456789abcdef:'+b'a'*32
                assert (case/'owned').exists()
            elif mode in ('stage-refused','bad-stage','failed-unit','changed-invocation','missing-job'):
                assert 'complete' not in (case/'calls').read_text().splitlines()
            results.append({'case':mode,'passed':True})
        finally:cleanup(case,p)

for mode in ('cancel-and-reattach','lost-response-and-retry','concurrent-callers'):
    with tempfile.TemporaryDirectory(prefix='dispatch-case-') as td:
        case=Path(td);p=start(case,'success');q=None
        try:
            await_file(case/'owned')
            if mode=='cancel-and-reattach':
                os.killpg(p.pid,signal.SIGTERM);p.wait(timeout=3)
            elif mode=='lost-response-and-retry':
                out,err=p.communicate(timeout=10);assert p.returncode==0,(out,err)
            q=start(case,'success');out,err=q.communicate(timeout=10)
            assert q.returncode==0,(mode,out,err)
            if p.poll() is None:
                out,err=p.communicate(timeout=10);assert p.returncode==0,(mode,out,err)
            assert (case/'calls').read_text().splitlines().count('start')==1
            results.append({'case':mode,'passed':True})
        finally:cleanup(case,p);cleanup(case,q)

with tempfile.TemporaryDirectory(prefix='dispatch-case-') as td:
    case=Path(td);p=start(case,'pending')
    try:
        await_file(case/'owned');time.sleep(.3)
        assert p.poll() is None
        assert 'complete' not in (case/'calls').read_text().splitlines()
        results.append({'case':'pending-executor-never-advertises-completion','passed':True})
    finally:cleanup(case,p)
print(json.dumps(results,indent=2))
